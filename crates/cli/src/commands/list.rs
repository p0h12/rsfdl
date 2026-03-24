use rsfdl_core::filter::{is_excluded, resolve_patterns};
use rsfdl_core::format_bytes;

use super::common::{SfdlArgs, load_and_decrypt};
use super::info::classify_exit_code;

pub async fn run(
	args: &SfdlArgs,
	password_list: &[String],
	resolve: bool,
	json: bool,
	cli_exclude: &[String],
	no_exclude: bool,
	show_excluded: bool,
) {
	let (mut container, settings, _outcome) = match load_and_decrypt(args, password_list) {
		Ok(result) => result,
		Err(e) => {
			eprintln!("Error: {e}");
			std::process::exit(classify_exit_code(&e));
		}
	};

	// SFDL-003: Resolve bulk folders if requested
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

	// DL-002: Resolve exclusion patterns
	let patterns = resolve_patterns(&settings.exclusion_patterns, cli_exclude, no_exclude);

	if json {
		print_json(&container, &patterns);
	} else {
		print_text(&container, &patterns, show_excluded);
	}
}

fn print_text(
	container: &rsfdl_core::sfdl::models::SfdlContainer,
	patterns: &[String],
	show_excluded: bool,
) {
	let mut total_files = 0usize;
	let mut total_bytes = 0u64;
	let mut excluded_count = 0usize;
	let mut bulk_count = 0usize;

	for pkg in &container.packages {
		if !pkg.name.is_empty() {
			println!("Package: {}", pkg.name);
		}

		for file in &pkg.file_list {
			let excluded = is_excluded(&file.file_name, patterns);
			if excluded {
				excluded_count += 1;
				if show_excluded {
					println!("  {:<55} {:>10}  [excluded]", file.file_name, format_bytes(file.file_size));
				}
			} else {
				println!("  {:<55} {:>10}", file.file_name, format_bytes(file.file_size));
			}
			total_files += 1;
			total_bytes += file.file_size;
		}

		for bulk in &pkg.bulk_folder_list {
			println!("  [DIR] {}", bulk.bulk_folder_path);
			bulk_count += 1;
		}

		if !pkg.file_list.is_empty() || !pkg.bulk_folder_list.is_empty() {
			println!();
		}
	}

	// BR-CLI-002-002: Summary line
	let selected = total_files - excluded_count;
	let selected_bytes: u64 = container
		.packages
		.iter()
		.flat_map(|p| &p.file_list)
		.filter(|f| !is_excluded(&f.file_name, patterns))
		.map(|f| f.file_size)
		.sum();

	println!("{} files ({}), {} excluded", selected, format_bytes(selected_bytes), excluded_count);

	if bulk_count > 0 {
		println!("{} bulk folder(s) (use --resolve to list contents via FTP)", bulk_count);
	}
}

fn print_json(container: &rsfdl_core::sfdl::models::SfdlContainer, patterns: &[String]) {
	let mut packages_json = Vec::new();

	let mut total_files = 0usize;
	let mut selected_files = 0usize;
	let mut total_bytes = 0u64;
	let mut selected_bytes = 0u64;

	for pkg in &container.packages {
		let mut files_json = Vec::new();

		for file in &pkg.file_list {
			let excluded = is_excluded(&file.file_name, patterns);
			total_files += 1;
			total_bytes += file.file_size;
			if !excluded {
				selected_files += 1;
				selected_bytes += file.file_size;
			}

			let mut file_obj = serde_json::json!({
				"filename": file.file_name,
				"size_bytes": file.file_size,
				"excluded": excluded,
			});

			if excluded {
				// Find which pattern matched
				if let Some(pattern) = patterns.iter().find(|p| is_excluded(&file.file_name, &[p.to_string()])) {
					file_obj["exclude_pattern"] = serde_json::json!(pattern);
				}
			}

			files_json.push(file_obj);
		}

		packages_json.push(serde_json::json!({
			"name": pkg.name,
			"files": files_json,
		}));
	}

	let excluded_files = total_files - selected_files;

	let output = serde_json::json!({
		"packages": packages_json,
		"summary": {
			"total_files": total_files,
			"selected_files": selected_files,
			"excluded_files": excluded_files,
			"total_bytes": total_bytes,
			"selected_bytes": selected_bytes,
		}
	});

	println!("{}", serde_json::to_string_pretty(&output).unwrap());
}
