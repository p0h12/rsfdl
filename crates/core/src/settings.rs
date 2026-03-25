//! CFG-001: Settings management.
//!
//! Load, save, validate, and reset application settings in TOML format.
//! The config file path is always provided by the calling layer (CLI, GUI, Mobile).

use std::fmt::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::SettingsError;

/// Application settings per BR-CFG-002.
///
/// All fields have `#[serde(default)]` so that missing keys in the TOML file
/// are filled with defaults instead of triggering a parse error.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Settings {
	pub download_directory: PathBuf,
	pub max_threads: u32,
	pub max_speed_kbps: u32,
	pub max_retries: u32,
	pub retry_delay_seconds: u32,
	pub auto_extract: bool,
	pub delete_archives_after_extract: bool,
	pub strict_disk_check: bool,
	pub ftp_timeout_seconds: u32,
	pub exclusion_patterns: Vec<String>,
	pub auto_passwords: Vec<String>,
	pub speedreport_template: String,
}

impl Default for Settings {
	fn default() -> Self {
		Self {
			download_directory: dirs::download_dir().unwrap_or_else(|| PathBuf::from(".")).join("rsfdl"),
			max_threads: 3,
			max_speed_kbps: 0,
			max_retries: 3,
			retry_delay_seconds: 10,
			auto_extract: false,
			delete_archives_after_extract: false,
			strict_disk_check: false,
			ftp_timeout_seconds: 30,
			exclusion_patterns: vec!["*.nfo".into(), "*.jpg".into(), "*.png".into(), "*.txt".into(), "*sample*".into()],
			auto_passwords: Vec::new(),
			speedreport_template: crate::speedreport::DEFAULT_TEMPLATE.to_string(),
		}
	}
}

/// Validate settings per BR-CFG-002.
/// Returns a list of (field_name, reason) pairs for all invalid values.
pub fn validate(settings: &Settings) -> Vec<(String, String)> {
	let mut errors = Vec::new();

	if settings.max_threads < 1 || settings.max_threads > 20 {
		errors.push(("max_threads".into(), format!("must be 1–20, got {}", settings.max_threads)));
	}

	if settings.max_retries > 50 {
		errors.push(("max_retries".into(), format!("must be 0–50, got {}", settings.max_retries)));
	}

	if settings.retry_delay_seconds < 1 || settings.retry_delay_seconds > 3600 {
		errors.push(("retry_delay_seconds".into(), format!("must be 1–3600, got {}", settings.retry_delay_seconds)));
	}

	if settings.ftp_timeout_seconds > 300 {
		errors.push(("ftp_timeout_seconds".into(), format!("must be 0–300, got {}", settings.ftp_timeout_seconds)));
	}

	for (i, pattern) in settings.exclusion_patterns.iter().enumerate() {
		if pattern.is_empty() {
			errors.push((format!("exclusion_patterns[{}]", i), "pattern must not be empty".into()));
		}
	}

	errors
}

/// Fix invalid values by replacing them with defaults (A3 flow).
/// Returns the list of fields that were corrected.
pub fn fix_invalid(settings: &mut Settings) -> Vec<String> {
	let errors = validate(settings);
	if errors.is_empty() {
		return Vec::new();
	}

	let defaults = Settings::default();
	let mut corrected = Vec::new();

	for (field, _) in &errors {
		match field.as_str() {
			"max_threads" => settings.max_threads = defaults.max_threads,
			"max_retries" => settings.max_retries = defaults.max_retries,
			"retry_delay_seconds" => settings.retry_delay_seconds = defaults.retry_delay_seconds,
			"ftp_timeout_seconds" => settings.ftp_timeout_seconds = defaults.ftp_timeout_seconds,
			_ => continue,
		}
		corrected.push(field.clone());
	}

	settings.exclusion_patterns.retain(|p| !p.is_empty());

	corrected
}

/// CFG-002: Determine the settings file path.
///
/// Priority (BR-CFG-005): `RSFDL_CONFIG` env var > platform default (BR-CFG-004).
pub fn config_path() -> PathBuf {
	let env_override = std::env::var("RSFDL_CONFIG").ok();
	resolve_config_path(env_override.as_deref())
}

/// CFG-002 pure logic: resolve settings path from optional env override.
///
/// Platform defaults (BR-CFG-004):
/// - Linux: `~/.config/rsfdl/settings.toml`
/// - macOS: `~/Library/Application Support/rsfdl/settings.toml`
/// - Windows: `%APPDATA%\rsfdl\settings.toml`
fn resolve_config_path(env_override: Option<&str>) -> PathBuf {
	if let Some(path) = env_override {
		return PathBuf::from(path);
	}
	let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("rsfdl");
	config_dir.join("settings.toml")
}

/// Load result returned by [`load`].
pub struct LoadResult {
	pub settings: Settings,
	pub warnings: Vec<String>,
}

/// CFG-001 Variante A: Load settings from a TOML file.
///
/// - File missing → create with defaults (A1).
/// - File corrupt → rename to `.bak`, use defaults (A2).
/// - Invalid values → fix silently, report corrected fields (A3).
pub fn load(path: &Path) -> LoadResult {
	let mut warnings = Vec::new();

	let content = match std::fs::read_to_string(path) {
		Ok(c) => c,
		Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
			// A1: File not found → defaults
			let settings = Settings::default();
			if let Err(e) = save(path, &settings) {
				warnings.push(format!("Could not write default settings: {e}"));
			}
			return LoadResult { settings, warnings };
		}
		Err(e) => {
			warnings.push(format!("Cannot read config file: {e}"));
			return LoadResult {
				settings: Settings::default(),
				warnings,
			};
		}
	};

	let mut settings: Settings = match toml::from_str(&content) {
		Ok(s) => s,
		Err(e) => {
			// A2: Corrupt TOML → .bak + defaults
			warnings.push(format!("Konfigurationsdatei beschädigt. Standardwerte werden verwendet. ({e})"));
			let bak = path.with_extension("toml.bak");
			if let Err(rename_err) = std::fs::rename(path, &bak) {
				warnings.push(format!("Could not rename corrupt file to .bak: {rename_err}"));
			}
			return LoadResult {
				settings: Settings::default(),
				warnings,
			};
		}
	};

	// A3: Fix invalid values
	let corrected = fix_invalid(&mut settings);
	if !corrected.is_empty() {
		warnings.push(format!("Ungültige Werte korrigiert: {}", corrected.join(", ")));
	}

	LoadResult { settings, warnings }
}

/// CFG-001 Variante B: Save settings to a TOML file.
///
/// Validates before saving. Returns error on validation failure (A4)
/// or IO failure (A5).
pub fn save(path: &Path, settings: &Settings) -> Result<(), SettingsError> {
	let errors = validate(settings);
	if !errors.is_empty() {
		let msg = errors.iter().map(|(field, reason)| format!("{field}: {reason}")).collect::<Vec<_>>().join("; ");
		return Err(SettingsError::Validation(msg));
	}

	if let Some(parent) = path.parent() {
		std::fs::create_dir_all(parent)?;
	}

	let toml_str = toml::to_string_pretty(settings).map_err(|e| SettingsError::TomlSerialize(e.to_string()))?;
	std::fs::write(path, toml_str)?;
	Ok(())
}

/// CFG-001 Variante C: Reset to defaults and write to disk.
pub fn reset(path: &Path) -> Result<Settings, SettingsError> {
	let settings = Settings::default();
	save(path, &settings)?;
	Ok(settings)
}

/// Format settings as human-readable key=value output for `config show`.
/// Passwords are masked — only the count is shown.
pub fn format_settings(path: &Path, settings: &Settings) -> String {
	let mut out = String::new();
	writeln!(out, "Settings file: {}", path.display()).unwrap();
	writeln!(out).unwrap();
	writeln!(out, "download_directory            = {}", settings.download_directory.display()).unwrap();
	writeln!(out, "max_threads                  = {}", settings.max_threads).unwrap();
	writeln!(out, "max_speed_kbps               = {}", settings.max_speed_kbps).unwrap();
	writeln!(out, "max_retries                  = {}", settings.max_retries).unwrap();
	writeln!(out, "retry_delay_seconds          = {}", settings.retry_delay_seconds).unwrap();
	writeln!(out, "ftp_timeout_seconds          = {}", settings.ftp_timeout_seconds).unwrap();
	writeln!(out, "auto_extract                 = {}", settings.auto_extract).unwrap();
	writeln!(out, "delete_archives_after_extract = {}", settings.delete_archives_after_extract).unwrap();
	writeln!(out, "strict_disk_check            = {}", settings.strict_disk_check).unwrap();
	if settings.exclusion_patterns.is_empty() {
		writeln!(out, "exclusion_patterns           = (none)").unwrap();
	} else {
		writeln!(out, "exclusion_patterns           = {}", settings.exclusion_patterns.join(", ")).unwrap();
	}
	writeln!(out, "auto_passwords               = ({} entries)", settings.auto_passwords.len()).unwrap();
	if settings.speedreport_template == crate::speedreport::DEFAULT_TEMPLATE || settings.speedreport_template.is_empty() {
		writeln!(out, "speedreport_template         = (default)").unwrap();
	} else {
		writeln!(out, "speedreport_template         = (custom)").unwrap();
	}
	out
}

#[cfg(test)]
mod tests {
	use super::*;

	// -------------------------------------------------------
	// CFG-001 / BR-CFG-002: Default values
	// -------------------------------------------------------

	/// CFG-001 | BR-CFG-001: All default values match the specification.
	#[test]
	fn cfg001_defaults_match_spec() {
		let s = Settings::default();
		assert!(s.download_directory.ends_with("rsfdl"));
		assert_eq!(s.max_threads, 3);
		assert_eq!(s.max_speed_kbps, 0);
		assert_eq!(s.max_retries, 3);
		assert_eq!(s.retry_delay_seconds, 10);
		assert!(!s.auto_extract);
		assert!(!s.delete_archives_after_extract);
		assert!(!s.strict_disk_check);
		assert_eq!(s.ftp_timeout_seconds, 30);
		assert_eq!(s.exclusion_patterns, vec!["*.nfo", "*.jpg", "*.png", "*.txt", "*sample*"]);
		assert!(s.auto_passwords.is_empty());
		assert_eq!(s.speedreport_template, crate::speedreport::DEFAULT_TEMPLATE);
	}

	// -------------------------------------------------------
	// CFG-001 / BR-CFG-002: Validation
	// -------------------------------------------------------

	/// CFG-001 | BR-CFG-002: Default values pass validation.
	#[test]
	fn cfg001_validate_accepts_defaults() {
		let s = Settings::default();
		assert!(validate(&s).is_empty());
	}

	/// CFG-001 | BR-CFG-002: max_threads must be 1–20.
	#[test]
	fn cfg001_validate_max_threads_bounds() {
		let mut s = Settings {
			max_threads: 0,
			..Settings::default()
		};
		assert!(!validate(&s).is_empty());

		s.max_threads = 21;
		assert!(!validate(&s).is_empty());

		s.max_threads = 1;
		assert!(validate(&s).is_empty());

		s.max_threads = 20;
		assert!(validate(&s).is_empty());
	}

	/// CFG-001 | BR-CFG-002: max_retries must be 0–50.
	#[test]
	fn cfg001_validate_max_retries_bounds() {
		let mut s = Settings {
			max_retries: 51,
			..Settings::default()
		};
		assert!(!validate(&s).is_empty());

		s.max_retries = 0;
		assert!(validate(&s).is_empty());

		s.max_retries = 50;
		assert!(validate(&s).is_empty());
	}

	/// CFG-001 | BR-CFG-002: retry_delay_seconds must be 1–3600.
	#[test]
	fn cfg001_validate_retry_delay_bounds() {
		let mut s = Settings {
			retry_delay_seconds: 0,
			..Settings::default()
		};
		assert!(!validate(&s).is_empty());

		s.retry_delay_seconds = 3601;
		assert!(!validate(&s).is_empty());

		s.retry_delay_seconds = 1;
		assert!(validate(&s).is_empty());

		s.retry_delay_seconds = 3600;
		assert!(validate(&s).is_empty());
	}

	/// CFG-001 | BR-CFG-002: exclusion_patterns must be valid glob syntax (non-empty).
	#[test]
	fn cfg001_validate_empty_exclusion_pattern() {
		let mut s = Settings::default();
		s.exclusion_patterns.push(String::new());
		let errors = validate(&s);
		assert!(errors.iter().any(|(f, _)| f.contains("exclusion_patterns")));
	}

	// -------------------------------------------------------
	// CFG-001 A3: fix_invalid — auto-correct out-of-range values
	// -------------------------------------------------------

	/// CFG-001 | A3: Out-of-range values are replaced with defaults.
	#[test]
	fn cfg001_fix_invalid_corrects_out_of_range() {
		let mut s = Settings {
			max_threads: 100,
			max_retries: 999,
			retry_delay_seconds: 0,
			..Settings::default()
		};
		s.exclusion_patterns.push(String::new());

		let corrected = fix_invalid(&mut s);

		assert!(corrected.contains(&"max_threads".to_string()));
		assert!(corrected.contains(&"max_retries".to_string()));
		assert!(corrected.contains(&"retry_delay_seconds".to_string()));
		assert_eq!(s.max_threads, 3);
		assert_eq!(s.max_retries, 3);
		assert_eq!(s.retry_delay_seconds, 10);
		assert!(!s.exclusion_patterns.iter().any(|p| p.is_empty()));
	}

	/// CFG-001 | A3: Valid values are not touched by fix_invalid.
	#[test]
	fn cfg001_fix_invalid_leaves_valid_unchanged() {
		let mut s = Settings {
			max_threads: 10,
			..Settings::default()
		};
		let corrected = fix_invalid(&mut s);
		assert!(corrected.is_empty());
		assert_eq!(s.max_threads, 10);
	}

	// -------------------------------------------------------
	// CFG-001 Variante A+B: TOML round-trip (save + load)
	// -------------------------------------------------------

	/// CFG-001 | Variante A+B: Save then load preserves all values.
	#[test]
	fn cfg001_save_and_load_round_trip() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("settings.toml");

		let settings = Settings {
			max_threads: 5,
			max_speed_kbps: 1024,
			auto_passwords: vec!["pw1".into(), "pw2".into()],
			..Settings::default()
		};

		save(&path, &settings).unwrap();
		let result = load(&path);

		assert!(result.warnings.is_empty());
		assert_eq!(result.settings.max_threads, 5);
		assert_eq!(result.settings.max_speed_kbps, 1024);
		assert_eq!(result.settings.auto_passwords, vec!["pw1", "pw2"]);
	}

	/// CFG-001 | Variante B: Save overwrites previous file contents.
	#[test]
	fn cfg001_save_overwrites_existing() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("settings.toml");

		let s1 = Settings {
			max_threads: 3,
			..Settings::default()
		};
		save(&path, &s1).unwrap();

		let s2 = Settings {
			max_threads: 7,
			..Settings::default()
		};
		save(&path, &s2).unwrap();

		let result = load(&path);
		assert_eq!(result.settings.max_threads, 7);
	}

	// -------------------------------------------------------
	// CFG-001 A1: File not found → defaults + create
	// -------------------------------------------------------

	/// CFG-001 | A1: Missing file → returns defaults and creates file on disk.
	#[test]
	fn cfg001_load_missing_file_returns_defaults_and_creates() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("settings.toml");

		assert!(!path.exists());
		let result = load(&path);

		assert_eq!(result.settings, Settings::default());
		assert!(path.exists(), "should create default settings file");
	}

	// -------------------------------------------------------
	// CFG-001 A2: Corrupt TOML → .bak + defaults
	// -------------------------------------------------------

	/// CFG-001 | A2: Corrupt file → renamed to .bak, returns defaults with warning.
	#[test]
	fn cfg001_load_corrupt_toml_renames_to_bak() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("settings.toml");
		std::fs::write(&path, "not valid toml {{{").unwrap();

		let result = load(&path);

		assert_eq!(result.settings, Settings::default());
		assert!(result.warnings.iter().any(|w| w.contains("beschädigt")), "should warn about corrupt file");
		assert!(dir.path().join("settings.toml.bak").exists(), "corrupt file should be renamed to .bak");
	}

	// -------------------------------------------------------
	// CFG-001 A3: Invalid values get fixed silently
	// -------------------------------------------------------

	/// CFG-001 | A3: Load with out-of-range values → auto-corrected with warning.
	#[test]
	fn cfg001_load_fixes_invalid_values() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("settings.toml");

		let s = Settings {
			max_threads: 99,
			..Settings::default()
		};
		// Write directly to bypass validation in save()
		let toml_str = toml::to_string_pretty(&s).unwrap();
		std::fs::write(&path, toml_str).unwrap();

		let result = load(&path);

		assert_eq!(result.settings.max_threads, 3); // fixed to default
		assert!(result.warnings.iter().any(|w| w.contains("max_threads")));
	}

	// -------------------------------------------------------
	// CFG-001 A4: Save rejects invalid settings
	// -------------------------------------------------------

	/// CFG-001 | A4: Save with invalid values → returns SettingsError::Validation.
	#[test]
	fn cfg001_save_rejects_invalid_settings() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("settings.toml");

		let s = Settings {
			max_threads: 0,
			..Settings::default()
		};
		let err = save(&path, &s).unwrap_err();
		assert!(matches!(err, SettingsError::Validation(_)));
		assert!(err.to_string().contains("max_threads"));
	}

	// -------------------------------------------------------
	// CFG-001 A5: Write error
	// -------------------------------------------------------

	/// CFG-001 | A5: Save to unwritable path → returns SettingsError::Io.
	#[test]
	fn cfg001_save_returns_io_error_for_bad_path() {
		let s = Settings::default();
		let err = save(Path::new("/proc/nonexistent/settings.toml"), &s).unwrap_err();
		assert!(matches!(err, SettingsError::Io(_)));
	}

	// -------------------------------------------------------
	// CFG-001 Variante C: Reset
	// -------------------------------------------------------

	/// CFG-001 | Variante C: Reset overwrites file with defaults.
	#[test]
	fn cfg001_reset_writes_defaults_to_disk() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("settings.toml");

		// Save non-default values first
		let s = Settings {
			max_threads: 10,
			..Settings::default()
		};
		save(&path, &s).unwrap();

		// Reset
		let reset_settings = reset(&path).unwrap();
		assert_eq!(reset_settings, Settings::default());

		// Verify file on disk has defaults
		let result = load(&path);
		assert_eq!(result.settings, Settings::default());
	}

	// -------------------------------------------------------
	// C-03: TOML file format
	// -------------------------------------------------------

	/// CFG-001 | C-03: Saved file is valid TOML with expected key-value pairs.
	#[test]
	fn cfg001_saved_file_is_valid_toml() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("settings.toml");

		save(&path, &Settings::default()).unwrap();
		let content = std::fs::read_to_string(&path).unwrap();

		assert!(content.contains("max_threads = 3"));
		assert!(content.contains("max_speed_kbps = 0"));
		assert!(content.contains("auto_extract = false"));
	}

	// -------------------------------------------------------
	// CFG-001 Variante A+B: Exclusion patterns round-trip
	// -------------------------------------------------------

	/// CFG-001 | BR-CFG-002: Exclusion patterns survive TOML round-trip.
	#[test]
	fn cfg001_exclusion_patterns_round_trip() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("settings.toml");

		let s = Settings {
			exclusion_patterns: vec!["*.nfo".into(), "*.jpg".into(), "*sample*".into()],
			..Settings::default()
		};

		save(&path, &s).unwrap();
		let result = load(&path);
		assert_eq!(result.settings.exclusion_patterns, vec!["*.nfo", "*.jpg", "*sample*"]);
	}

	// -------------------------------------------------------
	// CLI-005: format_settings for `config show`
	// -------------------------------------------------------

	/// CFG-001 | CLI-005: format_settings includes the file path.
	#[test]
	fn cfg001_format_settings_includes_path() {
		let s = Settings::default();
		let path = Path::new("/home/user/.config/rsfdl/settings.toml");
		let output = format_settings(path, &s);
		assert!(output.contains("/home/user/.config/rsfdl/settings.toml"));
	}

	/// CFG-001 | CLI-005: format_settings renders all fields.
	#[test]
	fn cfg001_format_settings_shows_all_fields() {
		let s = Settings {
			max_threads: 5,
			max_speed_kbps: 512,
			exclusion_patterns: vec!["*.nfo".into()],
			auto_passwords: vec!["secret".into()],
			..Settings::default()
		};

		let path = Path::new("/tmp/settings.toml");
		let output = format_settings(path, &s);

		assert!(output.contains("max_threads"));
		assert!(output.contains("5"));
		assert!(output.contains("max_speed_kbps"));
		assert!(output.contains("512"));
		assert!(output.contains("exclusion_patterns"));
		assert!(output.contains("*.nfo"));
	}

	/// CFG-001 | BR-CFG-003: Passwords are not shown in cleartext.
	#[test]
	fn cfg001_format_settings_hides_passwords() {
		let s = Settings {
			auto_passwords: vec!["secret1".into(), "secret2".into()],
			..Settings::default()
		};

		let path = Path::new("/tmp/settings.toml");
		let output = format_settings(path, &s);

		assert!(!output.contains("secret1"));
		assert!(!output.contains("secret2"));
		assert!(output.contains("2 entries"));
	}

	/// CFG-001 | CLI-005: format_settings shows speedreport_template status.
	#[test]
	fn cfg001_format_settings_shows_speedreport_template() {
		let path = Path::new("/tmp/settings.toml");

		let s = Settings::default();
		let output = format_settings(path, &s);
		assert!(output.contains("speedreport_template"));
		assert!(output.contains("(default)"));

		let s2 = Settings {
			speedreport_template: "custom template".into(),
			..Settings::default()
		};
		let output2 = format_settings(path, &s2);
		assert!(output2.contains("(custom)"));
		assert!(!output2.contains("custom template"));
	}

	// -------------------------------------------------------
	// CFG-001 Variante A: Partial TOML (missing keys)
	// -------------------------------------------------------

	/// CFG-001 | Variante A: Missing TOML keys are filled with defaults.
	#[test]
	fn cfg001_load_partial_toml_fills_defaults() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("settings.toml");

		// Write TOML with only one field
		std::fs::write(&path, "max_threads = 7\n").unwrap();

		let result = load(&path);

		assert!(result.warnings.is_empty());
		assert_eq!(result.settings.max_threads, 7);
		// All other fields should be defaults
		assert_eq!(result.settings.max_retries, 3);
		assert_eq!(result.settings.retry_delay_seconds, 10);
		assert_eq!(result.settings.ftp_timeout_seconds, 30);
		assert!(!result.settings.auto_extract);
		assert_eq!(result.settings.exclusion_patterns, vec!["*.nfo", "*.jpg", "*.png", "*.txt", "*sample*"]);
	}

	// -------------------------------------------------------
	// CFG-002: Konfigurationspfad ermitteln
	// -------------------------------------------------------

	/// CFG-002 | BR-CFG-005: RSFDL_CONFIG overrides platform default.
	#[test]
	fn cfg002_env_override_uses_custom_path() {
		let path = resolve_config_path(Some("/custom/path/my.toml"));
		assert_eq!(path, PathBuf::from("/custom/path/my.toml"));
	}

	/// CFG-002 | BR-CFG-005: RSFDL_CONFIG can be any path, even without .toml extension.
	#[test]
	fn cfg002_env_override_accepts_arbitrary_path() {
		let path = resolve_config_path(Some("/tmp/rsfdl-config"));
		assert_eq!(path, PathBuf::from("/tmp/rsfdl-config"));
	}

	/// CFG-002 | BR-CFG-004: Without override, path uses platform config dir.
	#[test]
	fn cfg002_platform_default_ends_with_settings_toml() {
		let path = resolve_config_path(None);
		assert_eq!(path.file_name().unwrap(), "settings.toml");
		assert!(path.parent().unwrap().ends_with("rsfdl"));
	}

	/// CFG-002 | BR-CFG-004: Platform default uses dirs::config_dir.
	#[test]
	fn cfg002_platform_default_is_under_config_dir() {
		let path = resolve_config_path(None);
		let expected_parent = dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("rsfdl");
		assert_eq!(path.parent().unwrap(), expected_parent);
	}

	/// CFG-002 | Main Success: config_path always returns a usable path.
	#[test]
	fn cfg002_config_path_returns_path_with_toml() {
		// Integration test using the real function (reads actual env).
		// We can only assert structural properties since RSFDL_CONFIG may or may not be set.
		let path = config_path();
		assert!(!path.as_os_str().is_empty());
	}
}
