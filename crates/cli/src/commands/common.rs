use clap::Args;
use rsfdl_core::container::{DecryptionStatus, LoadResult, load_sfdl};
use rsfdl_core::error::AppError;
use rsfdl_core::settings::{self, Settings};
use rsfdl_core::sfdl::crypto::EncryptedSfdl;
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
}

pub use rsfdl_core::container::DecryptionStatus as DecryptOutcome;

/// BR-CLI-007: CLI error types mapped directly to exit codes.
#[derive(Debug)]
pub enum CliError {
	/// Exit 1: File not found or not readable.
	FileError(String),
	/// Exit 2: Invalid SFDL format.
	ParseError(String),
	/// Exit 3: Password required (non-interactive).
	PasswordRequired,
	/// Exit 4: Wrong password.
	InvalidPassword,
}

impl CliError {
	/// BR-CLI-007: Map error to exit code.
	pub fn exit_code(&self) -> i32 {
		match self {
			CliError::FileError(_) => 1,
			CliError::ParseError(_) => 2,
			CliError::PasswordRequired => 3,
			CliError::InvalidPassword => 4,
		}
	}
}

impl std::fmt::Display for CliError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			CliError::FileError(msg) => write!(f, "{msg}"),
			CliError::ParseError(msg) => write!(f, "{msg}"),
			CliError::PasswordRequired => write!(f, "File is encrypted. Provide a password with -p <password>"),
			CliError::InvalidPassword => write!(f, "Invalid password"),
		}
	}
}

/// SFDL-001 + CLI-004: Load, parse, and decrypt an SFDL container.
///
/// Steps:
/// 1. Read file from disk (A2 on error → exit 1)
/// 2. Parse XML (A3 on error → exit 2)
/// 3. If encrypted: resolve password and decrypt (CLI-004)
pub fn load_and_decrypt(args: &SfdlArgs, auto_passwords: &[String]) -> Result<(SfdlContainer, Settings, DecryptOutcome), CliError> {
	// Load settings (path from RSFDL_CONFIG env or platform default)
	let settings_path = settings::config_path();
	let result = settings::load(&settings_path);
	for w in &result.warnings {
		eprintln!("Warning: {w}");
	}
	let settings = result.settings;

	// SFDL-001: Read and parse
	let xml = std::fs::read_to_string(&args.file).map_err(|e| CliError::FileError(format!("Cannot read file '{}': {}", args.file, e)))?;

	// BR-CLI-018: Password priority — --password flag skips auto-list entirely.
	// Only use auto-passwords when no explicit --password was given.
	let auto_list = if args.password.is_some() {
		Vec::new()
	} else {
		let mut all = auto_passwords.to_vec();
		for pw in &settings.auto_passwords {
			if !all.contains(pw) {
				all.push(pw.clone());
			}
		}
		all
	};

	let load_result = load_sfdl(&xml, &auto_list).map_err(|e| match &e {
		AppError::InvalidPassword => CliError::InvalidPassword,
		AppError::Decrypt(_) => CliError::InvalidPassword,
		AppError::Parse(_) => CliError::ParseError(e.to_string()),
		AppError::Ftp(_) => CliError::ParseError(e.to_string()),
	})?;

	// CLI-004: Resolve password and decrypt if needed
	let is_terminal = std::io::IsTerminal::is_terminal(&std::io::stderr());
	let (container, outcome) = match load_result {
		LoadResult::Ready(container, status) => {
			if matches!(status, DecryptionStatus::AutoDecrypted { .. }) {
				eprintln!("Auto-decrypted with password from list");
			}
			(container, status)
		}
		LoadResult::NeedsPassword(encrypted) => {
			let (container, pw) = resolve_password_and_decrypt(encrypted, args.password.as_deref(), is_terminal)?;
			(container, DecryptionStatus::AutoDecrypted { password: pw })
		}
	};

	Ok((container, settings, outcome))
}

/// CLI-004: Passwort ermitteln und entschlüsseln.
///
/// Implements BR-CLI-018 priority chain:
/// 1. --password flag (if provided)
/// 2. Interactive prompt (if terminal available)
/// 3. Error (exit code 3)
///
/// Returns the decrypted container and the password used.
fn resolve_password_and_decrypt(encrypted: EncryptedSfdl, password_flag: Option<&str>, is_terminal: bool) -> Result<(SfdlContainer, String), CliError> {
	// A1: --password flag
	let pw = if let Some(pw) = password_flag {
		pw.to_string()
	}
	// A2: Interactive prompt (terminal available)
	else if is_terminal {
		eprintln!("File is encrypted. Enter password:");
		rpassword::read_password().map_err(|_| CliError::PasswordRequired)?
	}
	// A3: No password available (non-interactive)
	else {
		return Err(CliError::PasswordRequired);
	};

	// A4: Wrong password → InvalidPassword
	let container = rsfdl_core::container::decrypt_with_password(encrypted, &pw).map_err(|e| match e {
		AppError::InvalidPassword | AppError::Decrypt(_) => CliError::InvalidPassword,
		AppError::Parse(ref pe) => CliError::ParseError(pe.to_string()),
		AppError::Ftp(ref fe) => CliError::ParseError(fe.to_string()),
	})?;

	Ok((container, pw))
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

	// -------------------------------------------------------
	// BR-CLI-007: Exit code mapping
	// -------------------------------------------------------

	/// CLI-001 | BR-CLI-007: FileError maps to exit code 1.
	#[test]
	fn cli001_file_error_exit_code() {
		assert_eq!(CliError::FileError("not found".into()).exit_code(), 1);
	}

	/// CLI-001 | BR-CLI-007: ParseError maps to exit code 2.
	#[test]
	fn cli001_parse_error_exit_code() {
		assert_eq!(CliError::ParseError("bad xml".into()).exit_code(), 2);
	}

	/// CLI-001 | BR-CLI-007: PasswordRequired maps to exit code 3.
	#[test]
	fn cli001_password_required_exit_code() {
		assert_eq!(CliError::PasswordRequired.exit_code(), 3);
	}

	/// CLI-001 | BR-CLI-007: InvalidPassword maps to exit code 4.
	#[test]
	fn cli001_invalid_password_exit_code() {
		assert_eq!(CliError::InvalidPassword.exit_code(), 4);
	}

	// -------------------------------------------------------
	// CLI-004: Password resolution
	// -------------------------------------------------------

	use rsfdl_core::sfdl::crypto::EncryptedSfdl;
	use rsfdl_core::sfdl::models::Connection;

	/// Build an EncryptedSfdl with the host encrypted under password "test".
	/// Ciphertext "FA9p93TaRSx1Bap096qqevmwi8vGbaEXtXRbnLmbUr8=" decrypts to
	/// "ftp.example.com" with password "test".
	fn make_encrypted() -> EncryptedSfdl {
		EncryptedSfdl::from_container(SfdlContainer {
			connection: Connection {
				host: "FA9p93TaRSx1Bap096qqevmwi8vGbaEXtXRbnLmbUr8=".into(),
				..Connection::default()
			},
			..SfdlContainer::default()
		})
	}

	/// CLI-004 | A3: NeedsPassword without flag, no terminal → PasswordRequired.
	#[test]
	fn cli004_needs_password_no_flag_no_terminal() {
		let result = resolve_password_and_decrypt(make_encrypted(), None, false);
		assert!(matches!(result.unwrap_err(), CliError::PasswordRequired));
	}

	/// CLI-004 | A4: Wrong password → InvalidPassword.
	#[test]
	fn cli004_wrong_password() {
		let result = resolve_password_and_decrypt(make_encrypted(), Some("wrong"), false);
		assert!(matches!(result.unwrap_err(), CliError::InvalidPassword));
	}

	/// CLI-004 | A1: Correct --password flag → decrypts and returns container + password.
	#[test]
	fn cli004_correct_password_flag() {
		let (container, pw) = resolve_password_and_decrypt(make_encrypted(), Some("test"), false).unwrap();
		assert_eq!(pw, "test");
		assert_eq!(container.connection.host, "ftp.example.com");
	}

	/// CLI-004 | A1: --password flag value is returned as the used password.
	#[test]
	fn cli004_flag_used_when_needs_password() {
		let (_, pw) = resolve_password_and_decrypt(make_encrypted(), Some("test"), false).unwrap();
		assert_eq!(pw, "test");
	}

	/// CLI-004 | A3: Wrong flag password → InvalidPassword (no fallback to auto-list).
	#[test]
	fn cli004_no_fallback_after_flag_fails() {
		let result = resolve_password_and_decrypt(make_encrypted(), Some("wrong"), false);
		assert!(matches!(result.unwrap_err(), CliError::InvalidPassword));
	}
}
