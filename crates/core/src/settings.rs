use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub download_directory: PathBuf,
    pub max_download_threads: u32,
    pub max_retries: u32,
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
            download_directory: dirs::download_dir()
                .unwrap_or_else(|| PathBuf::from(".")),
            max_download_threads: 3,
            max_retries: 3,
            retry_wait_seconds: 10,
            auto_password_list: Vec::new(),
            resume_downloads: true,
            create_package_subfolder: true,
            ftp_timeout_seconds: 30,
            file_exclusion_patterns: vec![
                "*.scr".into(),
                "*.lnk".into(),
                "*.nfo".into(),
            ],
            auto_extract_archives: false,
            delete_archives_after_extraction: false,
        }
    }
}
