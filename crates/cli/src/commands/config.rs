use std::path::PathBuf;

use rsfdl_core::settings::{AppSettings, default_settings_path, format_settings, load_settings, save_settings};

pub fn run_show(config_file: Option<&str>) {
	let path = config_file.map(PathBuf::from).unwrap_or_else(default_settings_path);
	let settings = load_settings(&path);
	print!("{}", format_settings(&path, &settings));
}

pub fn run_edit(config_file: Option<&str>) -> std::io::Result<()> {
	let path = config_file.map(PathBuf::from).unwrap_or_else(default_settings_path);

	if !path.exists() {
		save_settings(&path, &AppSettings::default())?;
	}

	let editor = std::env::var("EDITOR").unwrap_or_else(|_| if cfg!(windows) { "notepad".into() } else { "vi".into() });

	let status = std::process::Command::new(&editor).arg(&path).status()?;

	if !status.success() {
		return Err(std::io::Error::other(format!("Editor '{}' exited with {}", editor, status)));
	}

	Ok(())
}
