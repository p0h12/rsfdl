use rsfdl_core::format_bytes;
use rsfdl_core::sfdl::models::HashType;

use super::common::{SfdlArgs, load_and_decrypt};

pub async fn run(args: &SfdlArgs, password_list: &[String], resolve: bool) {
	let (mut container, settings, _outcome) = match load_and_decrypt(args, password_list) {
		Ok(result) => result,
		Err(e) => {
			eprintln!("Error: {e}");
			std::process::exit(1);
		}
	};

	// UC-SFDL-003: Resolve bulk folders if requested
	if resolve {
		let has_bulk = container.packages.iter().any(|p| p.bulk_folder_mode && !p.bulk_folder_list.is_empty());

		if has_bulk {
			eprintln!("Resolving bulk folders via FTP...");
			let warnings = rsfdl_core::container::resolve_bulk_folders(&mut container, settings.ftp_timeout_seconds).await;
			for w in &warnings {
				eprintln!("Warning: {}", w);
			}
		}
	}

	let mut total_files = 0usize;
	let mut total_bytes = 0u64;
	let mut total_bulk = 0usize;

	for pkg in &container.packages {
		if !pkg.name.is_empty() {
			if pkg.bulk_folder_mode {
				println!("Package: {} (Bulk Folder Mode)", pkg.name);
			} else {
				println!("Package: {}", pkg.name);
			}
		}
		println!();

		for file_item in &pkg.file_list {
			let hash_label = match file_item.hash_type {
				HashType::MD5 => "  [MD5]",
				HashType::CRC => "  [CRC]",
				HashType::SHA1 => "  [SHA1]",
				HashType::None => "",
			};

			println!("  {:<60} {:>10}{}", file_item.full_path, format_bytes(file_item.file_size), hash_label);
			total_files += 1;
			total_bytes += file_item.file_size;
		}

		for bulk in &pkg.bulk_folder_list {
			println!("  [DIR] {}", bulk.bulk_folder_path);
			total_bulk += 1;
		}
	}

	println!();
	if total_files > 0 {
		println!("{} files, {} total", total_files, format_bytes(total_bytes));
	}
	if total_bulk > 0 {
		println!("{} bulk folder(s) (use --resolve to list contents via FTP)", total_bulk);
	}
}
