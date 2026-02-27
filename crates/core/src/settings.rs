use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub download_directory: PathBuf,
    pub max_download_threads: u32,
    /// Not yet used by the download manager — retry logic is planned but unimplemented.
    pub max_retries: u32,
    /// Not yet used by the download manager — retry logic is planned but unimplemented.
    pub retry_wait_seconds: u32,
    pub auto_password_list: Vec<String>,
    pub resume_downloads: bool,
    pub create_package_subfolder: bool,
    pub ftp_timeout_seconds: u32,
    /// Glob patterns for files to exclude from download (UC-15).
    /// Case-insensitive matching on file_name only. Empty = no exclusions.
    #[serde(default)]
    pub file_exclusion_patterns: Vec<String>,
    /// Automatically extract archives after download completes (UC-14).
    /// Default: false (disabled).
    #[serde(default)]
    pub auto_extract_archives: bool,
    /// Delete archive files after successful extraction (UC-14).
    /// Default: false (archives are kept).
    #[serde(default)]
    pub delete_archives_after_extraction: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            download_directory: dirs::download_dir().unwrap_or_else(|| PathBuf::from(".")),
            max_download_threads: 3,
            max_retries: 3,
            retry_wait_seconds: 10,
            auto_password_list: Vec::new(),
            resume_downloads: true,
            create_package_subfolder: true,
            ftp_timeout_seconds: 30,
            file_exclusion_patterns: vec!["*.scr".into(), "*.lnk".into(), "*.nfo".into()],
            auto_extract_archives: false,
            delete_archives_after_extraction: false,
        }
    }
}

/// Get the default settings file path.
/// - macOS: ~/Library/Application Support/rsfdl/settings.json
/// - Linux: ~/.config/rsfdl/settings.json
/// - Windows: C:\Users\<user>\AppData\Roaming\rsfdl\settings.json
pub fn default_settings_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("rsfdl");
    config_dir.join("settings.json")
}

/// Load settings from a JSON file. Returns defaults if the file doesn't exist or is invalid.
pub fn load_settings(path: &Path) -> AppSettings {
    match std::fs::read_to_string(path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_else(|e| {
            tracing::warn!("Failed to parse settings file: {e}");
            AppSettings::default()
        }),
        Err(_) => AppSettings::default(),
    }
}

/// Format settings as human-readable key=value output for `config show`.
/// Passwords are masked — only the count is shown.
pub fn format_settings(path: &Path, settings: &AppSettings) -> String {
    let mut out = String::new();
    writeln!(out, "Settings file: {}", path.display()).unwrap();
    writeln!(out).unwrap();
    writeln!(
        out,
        "download_directory       = {}",
        settings.download_directory.display()
    )
    .unwrap();
    writeln!(
        out,
        "max_download_threads     = {}",
        settings.max_download_threads
    )
    .unwrap();
    writeln!(out, "max_retries              = {}", settings.max_retries).unwrap();
    writeln!(
        out,
        "retry_wait_seconds       = {}",
        settings.retry_wait_seconds
    )
    .unwrap();
    writeln!(
        out,
        "ftp_timeout_seconds      = {}",
        settings.ftp_timeout_seconds
    )
    .unwrap();
    writeln!(
        out,
        "resume_downloads         = {}",
        settings.resume_downloads
    )
    .unwrap();
    writeln!(
        out,
        "create_package_subfolder = {}",
        settings.create_package_subfolder
    )
    .unwrap();
    writeln!(
        out,
        "auto_extract_archives    = {}",
        settings.auto_extract_archives
    )
    .unwrap();
    writeln!(
        out,
        "delete_archives_after_extraction = {}",
        settings.delete_archives_after_extraction
    )
    .unwrap();
    if settings.file_exclusion_patterns.is_empty() {
        writeln!(out, "file_exclusion_patterns  = (none)").unwrap();
    } else {
        writeln!(
            out,
            "file_exclusion_patterns  = {}",
            settings.file_exclusion_patterns.join(", ")
        )
        .unwrap();
    }
    writeln!(
        out,
        "auto_password_list       = ({} entries)",
        settings.auto_password_list.len()
    )
    .unwrap();
    out
}

/// Save settings to a JSON file. Creates parent directories if needed.
pub fn save_settings(path: &Path, settings: &AppSettings) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(settings).map_err(std::io::Error::other)?;
    std::fs::write(path, json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_and_load_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");

        let settings = AppSettings {
            max_download_threads: 5,
            resume_downloads: false,
            auto_password_list: vec!["pw1".into(), "pw2".into()],
            ftp_timeout_seconds: 60,
            ..Default::default()
        };

        save_settings(&path, &settings).unwrap();
        let loaded = load_settings(&path);

        assert_eq!(loaded.max_download_threads, 5);
        assert!(!loaded.resume_downloads);
        assert_eq!(loaded.auto_password_list, vec!["pw1", "pw2"]);
        assert_eq!(loaded.ftp_timeout_seconds, 60);
    }

    #[test]
    fn load_returns_defaults_for_missing_file() {
        let path = Path::new("/tmp/rsfdl_nonexistent_test/settings.json");
        let loaded = load_settings(path);
        let defaults = AppSettings::default();
        assert_eq!(loaded.max_download_threads, defaults.max_download_threads);
        assert_eq!(loaded.resume_downloads, defaults.resume_downloads);
    }

    #[test]
    fn save_overwrites_existing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");

        let s1 = AppSettings {
            max_download_threads: 3,
            ..Default::default()
        };
        save_settings(&path, &s1).unwrap();

        let s2 = AppSettings {
            max_download_threads: 7,
            ..Default::default()
        };
        save_settings(&path, &s2).unwrap();

        let loaded = load_settings(&path);
        assert_eq!(loaded.max_download_threads, 7);
    }

    /// Covers AT-24 / UC-15: file_exclusion_patterns round-trip
    #[test]
    fn save_and_load_exclusion_patterns() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");

        let settings = AppSettings {
            file_exclusion_patterns: vec!["*.nfo".into(), "*.jpg".into(), "*sample*".into()],
            ..Default::default()
        };

        save_settings(&path, &settings).unwrap();
        let loaded = load_settings(&path);

        assert_eq!(
            loaded.file_exclusion_patterns,
            vec!["*.nfo", "*.jpg", "*sample*"]
        );
    }

    /// Covers UC-15: Existing settings without file_exclusion_patterns
    /// field should load with empty default (backward compatibility).
    #[test]
    fn load_settings_without_exclusion_patterns_defaults_to_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");

        // Write JSON without the new field (simulates old settings file)
        let json = r#"{
            "download_directory": "/tmp",
            "max_download_threads": 3,
            "max_retries": 3,
            "retry_wait_seconds": 10,
            "auto_password_list": [],
            "resume_downloads": true,
            "create_package_subfolder": true,
            "ftp_timeout_seconds": 30
        }"#;
        std::fs::write(&path, json).unwrap();

        let loaded = load_settings(&path);
        assert!(loaded.file_exclusion_patterns.is_empty());
    }

    #[test]
    fn load_returns_defaults_for_invalid_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, "not valid json {{{").unwrap();

        let loaded = load_settings(&path);
        let defaults = AppSettings::default();
        assert_eq!(loaded.max_download_threads, defaults.max_download_threads);
    }

    /// BR-007: auto_extract_archives defaults to false
    #[test]
    fn default_auto_extract_is_false() {
        let defaults = AppSettings::default();
        assert!(!defaults.auto_extract_archives);
    }

    /// BR-007: delete_archives_after_extraction defaults to false
    #[test]
    fn default_delete_archives_is_false() {
        let defaults = AppSettings::default();
        assert!(!defaults.delete_archives_after_extraction);
    }

    /// UC-14: Extraction settings round-trip through JSON
    #[test]
    fn save_and_load_extraction_settings() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");

        let settings = AppSettings {
            auto_extract_archives: true,
            delete_archives_after_extraction: true,
            ..Default::default()
        };

        save_settings(&path, &settings).unwrap();
        let loaded = load_settings(&path);

        assert!(loaded.auto_extract_archives);
        assert!(loaded.delete_archives_after_extraction);
    }

    // --- UC-09: format_settings ---

    /// AT-39 / BR-003: format_settings includes file path
    #[test]
    fn format_settings_includes_path() {
        let settings = AppSettings::default();
        let path = Path::new("/home/user/.config/rsfdl/settings.json");
        let output = format_settings(path, &settings);
        assert!(output.contains("Settings file: /home/user/.config/rsfdl/settings.json"));
    }

    /// AT-39 / BR-003: format_settings shows all fields
    #[test]
    fn format_settings_shows_all_fields() {
        let settings = AppSettings {
            max_download_threads: 5,
            max_retries: 2,
            retry_wait_seconds: 15,
            ftp_timeout_seconds: 60,
            resume_downloads: false,
            create_package_subfolder: false,
            auto_extract_archives: true,
            delete_archives_after_extraction: true,
            file_exclusion_patterns: vec!["*.nfo".into(), "*.jpg".into()],
            auto_password_list: vec!["pw1".into(), "pw2".into(), "pw3".into()],
            ..Default::default()
        };
        let path = Path::new("/tmp/settings.json");
        let output = format_settings(path, &settings);

        assert!(output.contains("max_download_threads"));
        assert!(output.contains("5"));
        assert!(output.contains("max_retries"));
        assert!(output.contains("2"));
        assert!(output.contains("retry_wait_seconds"));
        assert!(output.contains("15"));
        assert!(output.contains("ftp_timeout_seconds"));
        assert!(output.contains("60"));
        assert!(output.contains("resume_downloads"));
        assert!(output.contains("false"));
        assert!(output.contains("create_package_subfolder"));
        assert!(output.contains("auto_extract_archives"));
        assert!(output.contains("true"));
        assert!(output.contains("delete_archives_after_extraction"));
        assert!(output.contains("file_exclusion_patterns"));
        assert!(output.contains("*.nfo"));
        assert!(output.contains("*.jpg"));
    }

    /// AT-39 / BR-003: Passwords are not shown in cleartext
    #[test]
    fn format_settings_hides_passwords() {
        let settings = AppSettings {
            auto_password_list: vec!["secret1".into(), "secret2".into()],
            ..Default::default()
        };
        let path = Path::new("/tmp/settings.json");
        let output = format_settings(path, &settings);

        assert!(!output.contains("secret1"));
        assert!(!output.contains("secret2"));
        assert!(output.contains("2"));
    }

    /// AT-40: format_settings with defaults shows default values
    #[test]
    fn format_settings_with_defaults() {
        let settings = AppSettings::default();
        let path = Path::new("/tmp/settings.json");
        let output = format_settings(path, &settings);

        assert!(output.contains("max_download_threads"));
        assert!(output.contains("3"));
        assert!(output.contains("resume_downloads"));
        assert!(output.contains("true"));
        assert!(output.contains("auto_extract_archives"));
        assert!(output.contains("false"));
    }

    /// UC-14: Old settings file without extraction fields → defaults to false
    #[test]
    fn load_settings_without_extraction_fields_defaults_to_false() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");

        // JSON without UC-14 fields (simulates pre-UC-14 settings file)
        let json = r#"{
            "download_directory": "/tmp",
            "max_download_threads": 3,
            "max_retries": 3,
            "retry_wait_seconds": 10,
            "auto_password_list": [],
            "resume_downloads": true,
            "create_package_subfolder": true,
            "ftp_timeout_seconds": 30
        }"#;
        std::fs::write(&path, json).unwrap();

        let loaded = load_settings(&path);
        assert!(!loaded.auto_extract_archives);
        assert!(!loaded.delete_archives_after_extraction);
    }
}
