use std::path::PathBuf;

use clap::Args;
use rsfdl_core::container::{DecryptionStatus, LoadedContainer, load_sfdl};
use rsfdl_core::error::AppError;
use rsfdl_core::settings::{self, Settings};
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

pub use rsfdl_core::container::DecryptionStatus as DecryptOutcome;

/// SFDL-001 + CLI-004: Load, parse, and decrypt an SFDL container.
///
/// Steps:
/// 1. Read file from disk (SFDL-001 / A4 on error)
/// 2. Parse XML (SFDL-001 / A3 on error)
/// 3. If encrypted: resolve password and decrypt (CLI-004)
pub fn load_and_decrypt(args: &SfdlArgs, auto_passwords: &[String]) -> Result<(SfdlContainer, Settings, DecryptOutcome), String> {
	// Load settings
	let settings_path = args.config_file.as_deref().map(PathBuf::from).unwrap_or_else(settings::default_settings_path);
	let result = settings::load(&settings_path);
	for w in &result.warnings {
		eprintln!("Warning: {w}");
	}
	let settings = result.settings;

	// SFDL-001: Read and parse
	let xml = std::fs::read_to_string(&args.file).map_err(|e| format!("Cannot read file '{}': {}", args.file, e))?;

	// Merge password lists: CLI --password-file first, then settings auto_passwords
	let mut all_passwords = auto_passwords.to_vec();
	for pw in &settings.auto_passwords {
		if !all_passwords.contains(pw) {
			all_passwords.push(pw.clone());
		}
	}

	let LoadedContainer { mut container, status } = load_sfdl(&xml, &all_passwords).map_err(|e| e.to_string())?;

	// CLI-004: Resolve password and decrypt if needed
	let outcome = resolve_password_and_decrypt(&mut container, status, args.password.as_deref())?;

	Ok((container, settings, outcome))
}

/// CLI-004: Passwort ermitteln und entschluesseln.
///
/// Implements BR-CLI-018 priority chain:
/// 1. --password flag (if provided)
/// 2. Auto-password list (already tried by load_sfdl, reflected in status)
/// 3. Interactive prompt (if terminal available)
/// 4. Error (exit code 3)
///
/// Returns the decryption outcome or an error string.
fn resolve_password_and_decrypt(
	container: &mut SfdlContainer,
	status: DecryptionStatus,
	password_flag: Option<&str>,
) -> Result<DecryptOutcome, String> {
	match status {
		// Not encrypted — nothing to do
		DecryptionStatus::NotEncrypted => Ok(DecryptionStatus::NotEncrypted),

		// Main Success: Auto-password matched
		DecryptionStatus::AutoDecrypted { password } => {
			eprintln!("Auto-decrypted with password from list");
			Ok(DecryptionStatus::AutoDecrypted { password })
		}

		// Need to find a password
		DecryptionStatus::NeedsPassword => {
			// A1: --password flag
			let pw = if let Some(pw) = password_flag {
				pw.to_string()
			}
			// A2: Interactive prompt (terminal available)
			else if std::io::IsTerminal::is_terminal(&std::io::stderr()) {
				eprintln!("File is encrypted. Enter password:");
				rpassword::read_password().map_err(|e| format!("Failed to read password: {e}"))?
			}
			// A3: No password available (non-interactive)
			else {
				return Err("File is encrypted. Provide a password with -p <password>".into());
			};

			// A4: Wrong password → "Invalid password"
			rsfdl_core::container::decrypt_with_password(container, &pw).map_err(|e| match e {
				AppError::InvalidPassword => "Invalid password".to_string(),
				other => format!("Decryption failed: {other}"),
			})?;

			Ok(DecryptionStatus::AutoDecrypted { password: pw })
		}
	}
}

pub fn load_password_file(path: Option<&str>) -> Vec<String> {
	let Some(path) = path else {
		return Vec::new();
	};
	match std::fs::read_to_string(path) {
		Ok(content) => content.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect(),
		Err(e) => {
			eprintln!("Warning: Cannot read password file '{}': {}", path, e);
			Vec::new()
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	/// CLI-004 | Main Success: NotEncrypted passes through.
	#[test]
	fn cli004_not_encrypted_passthrough() {
		let mut container = SfdlContainer::default();
		let result = resolve_password_and_decrypt(&mut container, DecryptionStatus::NotEncrypted, None).unwrap();
		assert!(matches!(result, DecryptionStatus::NotEncrypted));
	}

	/// CLI-004 | Main Success: AutoDecrypted passes through.
	#[test]
	fn cli004_auto_decrypted_passthrough() {
		let mut container = SfdlContainer::default();
		let result = resolve_password_and_decrypt(&mut container, DecryptionStatus::AutoDecrypted { password: "test".into() }, None).unwrap();
		assert!(matches!(result, DecryptionStatus::AutoDecrypted { .. }));
	}

	/// CLI-004 | A3: NeedsPassword without flag or terminal → error.
	#[test]
	fn cli004_needs_password_no_flag_no_terminal() {
		let mut container = SfdlContainer::default();
		container.encrypted = true;
		// In test context, stderr is not a terminal → should error
		let result = resolve_password_and_decrypt(&mut container, DecryptionStatus::NeedsPassword, None);
		assert!(result.is_err());
		assert!(result.unwrap_err().contains("encrypted"));
	}

	/// CLI-004 | A4: Wrong password → "Invalid password".
	#[test]
	fn cli004_wrong_password() {
		use rsfdl_core::sfdl::models::Connection;

		let mut container = SfdlContainer {
			encrypted: true,
			connection: Connection {
				host: "FA9p93TaRSx1Bap096qqevmwi8vGbaEXtXRbnLmbUr8=".into(),
				..Connection::default()
			},
			..SfdlContainer::default()
		};

		let result = resolve_password_and_decrypt(&mut container, DecryptionStatus::NeedsPassword, Some("wrong"));
		assert!(result.is_err());
		assert!(result.unwrap_err().contains("Invalid password"));
	}

	/// CLI-004 | A1: Correct --password flag → decrypts.
	#[test]
	fn cli004_correct_password_flag() {
		use rsfdl_core::sfdl::models::Connection;

		// Build an encrypted container with known ciphertext
		let mut container = SfdlContainer {
			encrypted: true,
			connection: Connection {
				host: "FA9p93TaRSx1Bap096qqevmwi8vGbaEXtXRbnLmbUr8=".into(),
				..Connection::default()
			},
			..SfdlContainer::default()
		};

		let result = resolve_password_and_decrypt(&mut container, DecryptionStatus::NeedsPassword, Some("test")).unwrap();
		assert!(matches!(result, DecryptionStatus::AutoDecrypted { .. }));
		assert!(!container.encrypted);
		assert_eq!(container.connection.host, "ftp.example.com");
	}

	/// CLI-004 | BR-CLI-018: --password flag takes priority over auto-list.
	#[test]
	fn cli004_flag_priority_over_auto() {
		let mut container = SfdlContainer::default();
		// AutoDecrypted already happened, but flag is also present — auto wins (already decrypted)
		let result = resolve_password_and_decrypt(&mut container, DecryptionStatus::AutoDecrypted { password: "auto".into() }, Some("manual")).unwrap();
		if let DecryptionStatus::AutoDecrypted { password } = result {
			assert_eq!(password, "auto"); // auto already succeeded, flag ignored
		} else {
			panic!("expected AutoDecrypted");
		}
	}
}
