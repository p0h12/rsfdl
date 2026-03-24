//! CLI-005: Manage settings via the command line.

use rsfdl_core::settings::{self, Settings, config_path, format_settings};

/// CLI-005 | config show: Display current settings.
///
/// - Loads settings from config path (CFG-002 → CFG-001 Variante A).
/// - Warnings (corrupt file, corrected values) on stderr.
/// - Key=value output on stdout (BR-CLI-016). Passwords masked (BR-CFG-003).
pub fn run_show() {
	let path = config_path();
	let result = settings::load(&path);
	for w in &result.warnings {
		eprintln!("Warning: {w}");
	}
	print!("{}", format_settings(&path, &result.settings));
}

/// CLI-005 | config path: Print the settings file path (BR-CLI-014).
pub fn run_path() {
	println!("{}", config_path().display());
}

/// CLI-005 | config edit: Open settings file in editor.
///
/// - Creates default file if missing (CFG-001 A1).
/// - Editor via `$EDITOR` env var, fallback: notepad/vi (BR-CLI-015).
/// - Validates after edit, shows warnings on stderr (A2).
/// - A1: Editor fails → exit 1.
/// - A3: Can't write default → exit 1.
pub fn run_edit() -> std::io::Result<()> {
	let path = config_path();

	// CFG-001 A1: Create default file if missing
	if !path.exists() {
		settings::save(&path, &Settings::default()).map_err(|e| std::io::Error::other(e.to_string()))?;
	}

	// BR-CLI-015: Editor from $EDITOR, fallback notepad/vi
	let editor = std::env::var("EDITOR").unwrap_or_else(|_| if cfg!(windows) { "notepad".into() } else { "vi".into() });

	let status = std::process::Command::new(&editor).arg(&path).status()?;

	// A1: Editor exits non-zero
	if !status.success() {
		let code = status.code().map_or("unknown".to_string(), |c| c.to_string());
		return Err(std::io::Error::other(format!("Editor '{}' exited with {}.", editor, code)));
	}

	// A2: Validate after editing — warnings are informativ, exit 0
	let result = settings::load(&path);
	for w in &result.warnings {
		eprintln!("Warning: {w}");
	}

	Ok(())
}
