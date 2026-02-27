use std::path::PathBuf;

use clap::Args;
use rsfdl_core::settings::AppSettings;
use rsfdl_core::sfdl::crypto::{decrypt_container, try_passwords, validate_password};
use rsfdl_core::sfdl::models::SfdlContainer;
use rsfdl_core::sfdl::parser::parse_sfdl;

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

pub enum DecryptOutcome {
    WasPlaintext,
    Decrypted,
    AutoDecrypted,
}

/// Load, parse, and optionally decrypt an SFDL container.
/// Returns the container, loaded settings, password list, and how decryption happened.
pub fn load_and_decrypt(
    args: &SfdlArgs,
    password_list: &[String],
) -> Result<(SfdlContainer, AppSettings, DecryptOutcome), String> {
    let xml = std::fs::read_to_string(&args.file)
        .map_err(|e| format!("Cannot read file '{}': {}", args.file, e))?;

    let mut container = parse_sfdl(&xml).map_err(|e| e.to_string())?;

    let settings_path = args
        .config_file
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or_else(rsfdl_core::settings::default_settings_path);
    let settings = rsfdl_core::settings::load_settings(&settings_path);

    // Merge password lists: CLI --password-file first, then settings auto_password_list
    let mut all_passwords = password_list.to_vec();
    for pw in &settings.auto_password_list {
        if !all_passwords.contains(pw) {
            all_passwords.push(pw.clone());
        }
    }

    let outcome = if container.encrypted {
        if let Some(pw) = args.password.as_deref() {
            if !validate_password(&container, pw) {
                return Err("Invalid password".into());
            }
            decrypt_container(&mut container, pw).map_err(|e| format!("Decryption failed: {e}"))?;
            DecryptOutcome::Decrypted
        } else if let Some(pw) = try_passwords(&container, &all_passwords) {
            decrypt_container(&mut container, &pw)
                .map_err(|e| format!("Auto-decrypt failed: {e}"))?;
            eprintln!("Auto-decrypted with password from list");
            DecryptOutcome::AutoDecrypted
        } else {
            return Err("File is encrypted. Provide a password with -p <password>".into());
        }
    } else {
        DecryptOutcome::WasPlaintext
    };

    Ok((container, settings, outcome))
}

pub fn load_password_file(path: Option<&str>) -> Vec<String> {
    let Some(path) = path else { return Vec::new() };
    match std::fs::read_to_string(path) {
        Ok(content) => content
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect(),
        Err(e) => {
            eprintln!("Warning: Cannot read password file '{}': {}", path, e);
            Vec::new()
        }
    }
}
