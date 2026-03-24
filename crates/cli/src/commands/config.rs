use rsfdl_core::settings::{self, Settings, default_settings_path, format_settings};

pub fn run_show() {
	let path = default_settings_path();
	let result = settings::load(&path);
	for w in &result.warnings {
		eprintln!("Warning: {w}");
	}
	print!("{}", format_settings(&path, &result.settings));
}

pub fn run_path() {
	println!("{}", default_settings_path().display());
}

pub fn run_edit() -> std::io::Result<()> {
	let path = default_settings_path();

	if !path.exists() {
		settings::save(&path, &Settings::default()).map_err(|e| std::io::Error::other(e.to_string()))?;
	}

	let editor = std::env::var("EDITOR").unwrap_or_else(|_| if cfg!(windows) { "notepad".into() } else { "vi".into() });

	let status = std::process::Command::new(&editor).arg(&path).status()?;

	if !status.success() {
		return Err(std::io::Error::other(format!("Editor '{}' exited with {}", editor, status)));
	}

	// Validate after editing
	let result = settings::load(&path);
	for w in &result.warnings {
		eprintln!("Warning: {w}");
	}

	Ok(())
}
