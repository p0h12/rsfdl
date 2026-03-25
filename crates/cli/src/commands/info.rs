use rsfdl_core::container::DecryptionStatus;
use rsfdl_core::format_bytes;
use rsfdl_core::sfdl::models::{FtpsMode, SfdlContainer, SfdlVersion};

use super::common::{SfdlArgs, load_and_decrypt};

/// CLI-001: Display SFDL container metadata.
pub fn run(args: &SfdlArgs, password_list: &[String], json: bool) {
	let (container, _settings, outcome) = match load_and_decrypt(args, password_list) {
		Ok(result) => result,
		Err(e) => {
			eprintln!("Error: {e}");
			std::process::exit(e.exit_code());
		}
	};

	if json {
		print_json(&container, &outcome);
	} else {
		print_text(&container, &outcome);
	}
}

/// BR-CLI-009: Standard text output — key-value pairs on stdout.
fn print_text(c: &SfdlContainer, outcome: &DecryptionStatus) {
	let version_label = match c.version {
		SfdlVersion::V3 => "v3",
		SfdlVersion::V2 => "v2",
	};

	let encrypted_label = match outcome {
		DecryptionStatus::NotEncrypted => "no".to_string(),
		DecryptionStatus::AutoDecrypted { .. } => "yes (auto-decrypted)".to_string(),
		DecryptionStatus::NeedsPassword => "yes (not decrypted)".to_string(),
	};

	let ssl_label = match c.connection.ssl_protocol.ftps_mode() {
		FtpsMode::None => "FTP",
		FtpsMode::Explicit => "FTPS (explicit)",
		FtpsMode::Implicit => "FTPS (implicit)",
	};

	let file_count: usize = c.packages.iter().map(|p| p.file_list.len()).sum();
	let bulk_count: usize = c.packages.iter().map(|p| p.bulk_folder_list.len()).sum();
	let total_bytes: u64 = c.packages.iter().flat_map(|p| p.file_list.iter()).map(|f| f.file_size).sum();

	println!("Container:     {}", c.description);
	println!("Uploader:      {}", c.uploader);
	println!("Host:          {}:{} ({})", c.connection.host, c.connection.port, ssl_label);
	println!("Version:       {}", version_label);
	println!("Encrypted:     {}", encrypted_label);
	println!("Packages:      {}", c.packages.len());
	if file_count > 0 {
		println!("Files:         {}", file_count);
		println!("Size:          {}", format_bytes(total_bytes));
	}
	if bulk_count > 0 {
		println!("Bulk folders:  {} (resolve with `rsfdl list -r`)", bulk_count);
	}
}

/// BR-CLI-009: JSON output — structured object on stdout.
fn print_json(c: &SfdlContainer, outcome: &DecryptionStatus) {
	let file_count: usize = c.packages.iter().map(|p| p.file_list.len()).sum();
	let total_bytes: u64 = c.packages.iter().flat_map(|p| p.file_list.iter()).map(|f| f.file_size).sum();

	let was_encrypted = !matches!(outcome, DecryptionStatus::NotEncrypted);
	let protocol = match c.connection.ssl_protocol.ftps_mode() {
		FtpsMode::None => "FTP",
		FtpsMode::Explicit => "FTPS-explicit",
		FtpsMode::Implicit => "FTPS-implicit",
	};

	let json = serde_json::json!({
		"description": c.description,
		"uploader": c.uploader,
		"host": c.connection.host,
		"port": c.connection.port,
		"protocol": protocol,
		"encrypted": was_encrypted,
		"packages": c.packages.len(),
		"total_files": file_count,
		"total_bytes": total_bytes,
	});

	println!("{}", serde_json::to_string_pretty(&json).unwrap());
}
