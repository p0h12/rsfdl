use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use dioxus::prelude::*;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use rsfdl_core::settings::AppSettings;
use rsfdl_core::sfdl::models::SfdlContainer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppView {
    Main,
    Settings,
    Creator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadPhase {
    Idle,
    Downloading,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum FileStatus {
    Pending,
    Downloading,
    Completed,
    Failed,
    Cancelled,
    Skipped,
}

#[derive(Debug, Clone)]
pub struct FileDownloadState {
    pub file_name: String,
    pub total_bytes: u64,
    pub bytes_written: u64,
    pub status: FileStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DownloadSummary {
    pub total_files: u32,
    pub completed: u32,
    pub failed: u32,
    pub cancelled: u32,
    pub skipped: u32,
}

#[derive(Debug, Clone, Default)]
pub struct GlobalProgressState {
    pub total_bytes_all: u64,
    pub total_written_all: u64,
    pub files_done: u32,
    pub files_total: u32,
    pub started_at: Option<Instant>,
}

impl GlobalProgressState {
    pub fn speed_bytes_per_sec(&self) -> f64 {
        let Some(started) = self.started_at else {
            return 0.0;
        };
        let elapsed = started.elapsed().as_secs_f64();
        if elapsed < 0.1 {
            return 0.0;
        }
        self.total_written_all as f64 / elapsed
    }

    pub fn eta_seconds(&self) -> Option<f64> {
        let speed = self.speed_bytes_per_sec();
        if speed < 1.0 || self.total_bytes_all == 0 {
            return None;
        }
        let remaining = self.total_bytes_all.saturating_sub(self.total_written_all);
        Some(remaining as f64 / speed)
    }

    pub fn percent(&self) -> f64 {
        if self.total_bytes_all == 0 {
            return 0.0;
        }
        (self.total_written_all as f64 / self.total_bytes_all as f64 * 100.0).min(100.0)
    }
}

/// Central app state provided via use_context_provider at root.
/// All fields are Signal handles (Clone + Copy).
#[derive(Clone, Copy)]
pub struct AppState {
    pub current_view: Signal<AppView>,
    pub container: Signal<Option<SfdlContainer>>,
    pub container_path: Signal<Option<String>>,
    pub needs_password: Signal<bool>,
    pub password_error: Signal<Option<String>>,
    pub selected_files: Signal<Vec<bool>>,
    pub download_phase: Signal<DownloadPhase>,
    pub file_states: Signal<HashMap<Uuid, FileDownloadState>>,
    pub cancel_token: Signal<Option<CancellationToken>>,
    pub file_cancel_tx: Signal<Option<Arc<mpsc::UnboundedSender<Uuid>>>>,
    pub summary: Signal<Option<DownloadSummary>>,
    pub settings: Signal<AppSettings>,
    pub error_message: Signal<Option<String>>,
    pub global_progress: Signal<GlobalProgressState>,
    pub resolving_bulk_folders: Signal<bool>,
}

impl AppState {
    pub fn new() -> Self {
        // Load settings from DB, fall back to defaults
        let settings = Self::load_settings_from_file();

        Self {
            current_view: Signal::new(AppView::Main),
            container: Signal::new(None),
            container_path: Signal::new(None),
            needs_password: Signal::new(false),
            password_error: Signal::new(None),
            selected_files: Signal::new(Vec::new()),
            download_phase: Signal::new(DownloadPhase::Idle),
            file_states: Signal::new(HashMap::new()),
            cancel_token: Signal::new(None),
            file_cancel_tx: Signal::new(None),
            summary: Signal::new(None),
            settings: Signal::new(settings),
            error_message: Signal::new(None),
            global_progress: Signal::new(GlobalProgressState::default()),
            resolving_bulk_folders: Signal::new(false),
        }
    }

    fn load_settings_from_file() -> AppSettings {
        let path = rsfdl_core::settings::default_settings_path();
        rsfdl_core::settings::load_settings(&path)
    }

    /// Flat list of all FileItems across all packages.
    pub fn all_files(&self) -> Vec<rsfdl_core::sfdl::models::FileItem> {
        match self.container.read().as_ref() {
            Some(c) => c
                .packages
                .iter()
                .flat_map(|p| p.file_list.iter().cloned())
                .collect(),
            None => Vec::new(),
        }
    }

    /// Total size of all files in bytes.
    pub fn total_size(&self) -> u64 {
        match self.container.read().as_ref() {
            Some(c) => c
                .packages
                .iter()
                .flat_map(|p| p.file_list.iter())
                .map(|f| f.file_size)
                .sum(),
            None => 0,
        }
    }

    /// Sum of sizes of only selected files.
    pub fn selected_size(&self) -> u64 {
        let files = self.all_files();
        let selected = self.selected_files.read();
        files
            .iter()
            .enumerate()
            .filter(|(i, _)| selected.get(*i).copied().unwrap_or(false))
            .map(|(_, f)| f.file_size)
            .sum()
    }

    /// Count of selected files.
    pub fn selected_count(&self) -> usize {
        self.selected_files.read().iter().filter(|&&s| s).count()
    }

    /// Reset download-related state (phase, file states, summary, progress).
    pub fn reset_download_state(&mut self) {
        self.download_phase.set(DownloadPhase::Idle);
        self.file_states.write().clear();
        self.summary.set(None);
        self.global_progress.set(GlobalProgressState::default());
    }

    /// Reset all container/download state for loading a new container.
    pub fn reset_for_new_container(&mut self) {
        self.needs_password.set(false);
        self.password_error.set(None);
        self.error_message.set(None);
        self.reset_download_state();
    }

    /// Total bulk folder count.
    #[allow(dead_code)]
    pub fn bulk_folder_count(&self) -> usize {
        match self.container.read().as_ref() {
            Some(c) => c.packages.iter().map(|p| p.bulk_folder_list.len()).sum(),
            None => 0,
        }
    }
}
