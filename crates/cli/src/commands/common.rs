use std::path::PathBuf;

use clap::Args;
use rsfdl_core::container::{DecryptionStatus, LoadedContainer, load_sfdl};
use rsfdl_core::error::AppError;
use rsfdl_core::settings::AppSettings;
use rsfdl_core::sfdl::models::SfdlContainer;

/// Shared CLI arguments for SFDL file operations.
#[derive(Args)]
pub struct SfdlArgs {
	/// Path to .sfdl file
	pub file: String,
	/// Decryption password
	#[arg(short, long)]
	pub password: Option<String>,
	/// File with passwords to try (one per line)
	#[arg(long)]
	pub password_file: Option<String>,
	/// Path to settings file (default: platform config dir)
	#[arg(long)]
	pub config_file: Option<String>,
}

/// How decryption was resolved — re-exported from rsfdl-app for CLI use.
pub use rsfdl_core::container::DecryptionStatus as DecryptOutcome;

/// Load, parse, and optionally decrypt an SFDL container.
/// Returns the container, loaded settings, and how decryption happened.
pub fn load_and_decrypt(args: &SfdlArgs, password_list: &[String]) -> Result<(SfdlContainer, AppSettings, DecryptOutcome), String> {
	let xml = std::fs::read_to_string(&args.file).map_err(|e| format!("Cannot read file '{}': {}", args.file, e))?;

	let settings_path = args.config_file.as_deref().map(PathBuf::from).unwrap_or_else(rsfdl_core::settings::default_settings_path);
	let settings = rsfdl_core::settings::load_settings(&settings_path);

	// Merge password lists: CLI --password-file first, then settings auto_password_list
	let mut all_passwords = password_list.to_vec();
	for pw in &settings.auto_password_list {
		if !all_passwords.contains(pw) {
			all_passwords.push(pw.clone());
		}
	}

	let LoadedContainer { mut container, status } = load_sfdl(&xml, &all_passwords).map_err(|e| e.to_string())?;

	let outcome = match status {
		DecryptionStatus::NotEncrypted => DecryptionStatus::NotEncrypted,
		DecryptionStatus::AutoDecrypted { password } => {
			eprintln!("Auto-decrypted with password from list");
			DecryptionStatus::AutoDecrypted { password }
		}
		DecryptionStatus::NeedsPassword => {
			// Manual password required
			if let Some(pw) = args.password.as_deref() {
				rsfdl_core::container::decrypt_with_password(&mut container, pw).map_err(|e| match e {
					AppError::InvalidPassword => "Invalid password".to_string(),
					other => format!("Decryption failed: {other}"),
				})?;
				DecryptionStatus::AutoDecrypted { password: pw.to_string() }
			} else {
				return Err("File is encrypted. Provide a password with -p <password>".into());
			}
		}
	};

	Ok((container, settings, outcome))
}

pub fn load_password_file(path: Option<&str>) -> Vec<String> {
	let Some(path) = path else { return Vec::new() };
	match std::fs::read_to_string(path) {
		Ok(content) => content.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect(),
		Err(e) => {
			eprintln!("Warning: Cannot read password file '{}': {}", path, e);
			Vec::new()
		}
	}
}
