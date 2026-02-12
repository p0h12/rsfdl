use std::path::PathBuf;

use crate::settings::AppSettings;

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
pub fn load_settings(path: &std::path::Path) -> AppSettings {
    match std::fs::read_to_string(path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_else(|e| {
            tracing::warn!("Failed to parse settings file: {e}");
            AppSettings::default()
        }),
        Err(_) => AppSettings::default(),
    }
}

/// Save settings to a JSON file. Creates parent directories if needed.
pub fn save_settings(path: &std::path::Path, settings: &AppSettings) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(settings)
        .map_err(std::io::Error::other)?;
    std::fs::write(path, json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_and_load_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");

        let mut settings = AppSettings::default();
        settings.max_download_threads = 5;
        settings.resume_downloads = false;
        settings.auto_password_list = vec!["pw1".into(), "pw2".into()];
        settings.ftp_timeout_seconds = 60;

        save_settings(&path, &settings).unwrap();
        let loaded = load_settings(&path);

        assert_eq!(loaded.max_download_threads, 5);
        assert!(!loaded.resume_downloads);
        assert_eq!(loaded.auto_password_list, vec!["pw1", "pw2"]);
        assert_eq!(loaded.ftp_timeout_seconds, 60);
    }

    #[test]
    fn load_returns_defaults_for_missing_file() {
        let path = std::path::Path::new("/tmp/rsfdl_nonexistent_test/settings.json");
        let loaded = load_settings(path);
        let defaults = AppSettings::default();
        assert_eq!(loaded.max_download_threads, defaults.max_download_threads);
        assert_eq!(loaded.resume_downloads, defaults.resume_downloads);
    }

    #[test]
    fn save_overwrites_existing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");

        let mut s1 = AppSettings::default();
        s1.max_download_threads = 3;
        save_settings(&path, &s1).unwrap();

        let mut s2 = AppSettings::default();
        s2.max_download_threads = 7;
        save_settings(&path, &s2).unwrap();

        let loaded = load_settings(&path);
        assert_eq!(loaded.max_download_threads, 7);
    }

    /// Covers AT-24 / UC-15: file_exclusion_patterns round-trip
    #[test]
    fn save_and_load_exclusion_patterns() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");

        let mut settings = AppSettings::default();
        settings.file_exclusion_patterns = vec![
            "*.nfo".into(),
            "*.jpg".into(),
            "*sample*".into(),
        ];

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

    // =================================================================
    // UC-14: Auto-extraction settings
    // =================================================================

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

        let mut settings = AppSettings::default();
        settings.auto_extract_archives = true;
        settings.delete_archives_after_extraction = true;

        save_settings(&path, &settings).unwrap();
        let loaded = load_settings(&path);

        assert!(loaded.auto_extract_archives);
        assert!(loaded.delete_archives_after_extraction);
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
