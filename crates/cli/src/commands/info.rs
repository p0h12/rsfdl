use rsfdl_core::container::DecryptionStatus;
use rsfdl_core::format_bytes;
use rsfdl_core::sfdl::models::{SfdlContainer, SfdlVersion, SslProtocol};

use super::common::{SfdlArgs, load_and_decrypt};

/// CLI-001: Exit codes per spec.
const EXIT_FILE_ERROR: i32 = 1;
const EXIT_PARSE_ERROR: i32 = 2;
const EXIT_PASSWORD_REQUIRED: i32 = 3;
const EXIT_WRONG_PASSWORD: i32 = 4;

pub fn run(args: &SfdlArgs, password_list: &[String], json: bool) {
	let (container, _settings, outcome) = match load_and_decrypt(args, password_list) {
		Ok(result) => result,
		Err(e) => {
			eprintln!("Error: {e}");
			let code = classify_exit_code(&e);
			std::process::exit(code);
		}
	};

	if json {
		print_json(&container, &outcome);
	} else {
		print_text(&container, &outcome);
	}
}

pub fn classify_exit_code(error: &str) -> i32 {
	if error.contains("Cannot read file") || error.contains("No such file") {
		EXIT_FILE_ERROR
	} else if error.contains("Invalid password") {
		EXIT_WRONG_PASSWORD
	} else if error.contains("encrypted") || error.contains("Provide a password") {
		EXIT_PASSWORD_REQUIRED
	} else if error.contains("parse") || error.contains("Parse") || error.contains("Unsupported") {
		EXIT_PARSE_ERROR
	} else {
		EXIT_FILE_ERROR
	}
}

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

	let ssl_label = match c.connection.ssl_protocol {
		SslProtocol::None => "FTP",
		_ => "FTPS",
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

fn print_json(c: &SfdlContainer, outcome: &DecryptionStatus) {
	let file_count: usize = c.packages.iter().map(|p| p.file_list.len()).sum();
	let total_bytes: u64 = c.packages.iter().flat_map(|p| p.file_list.iter()).map(|f| f.file_size).sum();

	let was_encrypted = !matches!(outcome, DecryptionStatus::NotEncrypted);
	let protocol = match c.connection.ssl_protocol {
		SslProtocol::None => "FTP",
		_ => "FTPS",
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
