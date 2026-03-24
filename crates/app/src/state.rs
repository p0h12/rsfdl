use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use dioxus::prelude::*;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use rsfdl_core::selection::FileSelection;
use rsfdl_core::settings::Settings;
use rsfdl_core::sfdl::models::SfdlContainer;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppView {
	Main,
	Settings,
	Creator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
	Light,
	Dark,
	System,
}

/// Per-container lifecycle phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerPhase {
	/// Encrypted, waiting for password input.
	NeedsPassword,
	/// Resolving BulkFolders via FTP.
	ResolvingBulk,
	/// Decrypted/unencrypted, showing file tree, ready to download.
	Ready,
	/// Download in progress.
	Downloading,
	/// Download finished (success, partial, or cancelled).
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

// ---------------------------------------------------------------------------
// Per-file download state
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Per-container state
// ---------------------------------------------------------------------------

pub type ContainerId = u32;

#[derive(Debug, Clone)]
pub struct ContainerState {
	pub id: ContainerId,
	pub file_path: String,
	pub container: SfdlContainer,
	pub phase: ContainerPhase,
	pub expanded: bool,

	// Password state (when phase == NeedsPassword)
	pub password_error: Option<String>,

	// File selection (when phase >= Ready)
	pub selection: FileSelection,

	// Download state (when phase == Downloading or Done)
	pub file_states: HashMap<Uuid, FileDownloadState>,
	pub cancel_token: Option<CancellationToken>,
	pub file_cancel_tx: Option<Arc<mpsc::UnboundedSender<Uuid>>>,
	pub global_progress: GlobalProgressState,
	pub summary: Option<DownloadSummary>,
}

impl ContainerState {
	/// Flat list of all FileItems across all packages.
	pub fn all_files(&self) -> Vec<rsfdl_core::sfdl::models::FileItem> {
		self.container.packages.iter().flat_map(|p| p.file_list.iter().cloned()).collect()
	}

	/// Total size of all files in bytes.
	pub fn total_size(&self) -> u64 {
		self.selection.total_size()
	}

	/// Sum of sizes of only selected files.
	pub fn selected_size(&self) -> u64 {
		self.selection.selected_size()
	}

	/// Count of selected files.
	pub fn selected_count(&self) -> usize {
		self.selection.selected_count()
	}

	/// Reset download-related state to allow re-downloading.
	pub fn reset_download(&mut self) {
		self.phase = ContainerPhase::Ready;
		self.file_states.clear();
		self.cancel_token = None;
		self.file_cancel_tx = None;
		self.global_progress = GlobalProgressState::default();
		self.summary = None;
	}

	/// Display name derived from container description or file path.
	pub fn display_name(&self) -> &str {
		let desc = &self.container.description;
		if !desc.is_empty() { desc } else { self.file_path.rsplit('/').next().unwrap_or(&self.file_path) }
	}

	/// Whether this container is encrypted.
	pub fn is_encrypted(&self) -> bool {
		self.container.encrypted
	}

	/// SFDL version string (e.g. "v3").
	pub fn version_tag(&self) -> &str {
		match self.container.version {
			rsfdl_core::sfdl::models::SfdlVersion::V3 => "v3",
			rsfdl_core::sfdl::models::SfdlVersion::V2 => "v2",
		}
	}

	/// Total number of files across all packages.
	pub fn total_file_count(&self) -> usize {
		self.container.packages.iter().map(|p| p.file_list.len()).sum()
	}

	/// Total number of packages.
	pub fn package_count(&self) -> usize {
		self.container.packages.len()
	}

	/// FTP host string (host:port) if available.
	pub fn host_display(&self) -> Option<String> {
		let conn = &self.container.connection;
		if conn.host.is_empty() {
			return None;
		}
		Some(format!("{}:{}", conn.host, conn.port))
	}
}

// ---------------------------------------------------------------------------
// Central app state
// ---------------------------------------------------------------------------

/// Central app state provided via use_context_provider at root.
/// All fields are Signal handles (Clone + Copy).
#[derive(Clone, Copy)]
pub struct AppState {
	pub current_view: Signal<AppView>,
	pub containers: Signal<Vec<ContainerState>>,
	pub next_id: Signal<ContainerId>,
	pub settings: Signal<Settings>,
	pub error_message: Signal<Option<String>>,
	pub theme: Signal<Theme>,
}

impl AppState {
	pub fn new() -> Self {
		let settings = Self::load_settings_from_file();

		Self {
			current_view: Signal::new(AppView::Main),
			containers: Signal::new(Vec::new()),
			next_id: Signal::new(1),
			settings: Signal::new(settings),
			error_message: Signal::new(None),
			theme: Signal::new(Theme::System),
		}
	}

	fn load_settings_from_file() -> Settings {
		let path = rsfdl_core::settings::config_path();
		rsfdl_core::settings::load(&path).settings
	}

	/// Add a new container to the list. Returns the assigned ID.
	pub fn add_container(&mut self, file_path: String, container: SfdlContainer, phase: ContainerPhase) -> ContainerId {
		let id = *self.next_id.read();
		self.next_id.set(id + 1);

		let patterns = self.settings.read().exclusion_patterns.clone();
		let selection = FileSelection::new(&container, &patterns);

		let cs = ContainerState {
			id,
			file_path,
			container,
			phase,
			expanded: true,
			password_error: None,
			selection,
			file_states: HashMap::new(),
			cancel_token: None,
			file_cancel_tx: None,
			global_progress: GlobalProgressState::default(),
			summary: None,
		};

		self.containers.write().push(cs);
		id
	}

	/// Remove a container by ID.
	pub fn remove_container(&mut self, id: ContainerId) {
		self.containers.write().retain(|c| c.id != id);
	}

	/// Remove all containers.
	pub fn remove_all(&mut self) {
		self.containers.write().clear();
	}

	/// Move a container from one position to another.
	pub fn reorder(&mut self, from_idx: usize, to_idx: usize) {
		let mut list = self.containers.write();
		if from_idx < list.len() && to_idx < list.len() && from_idx != to_idx {
			let item = list.remove(from_idx);
			list.insert(to_idx, item);
		}
	}

	/// Move a container up by one position.
	pub fn move_up(&mut self, id: ContainerId) {
		let list = self.containers.read();
		if let Some(idx) = list.iter().position(|c| c.id == id) {
			drop(list);
			if idx > 0 {
				self.reorder(idx, idx - 1);
			}
		}
	}

	/// Move a container down by one position.
	pub fn move_down(&mut self, id: ContainerId) {
		let list = self.containers.read();
		let len = list.len();
		if let Some(idx) = list.iter().position(|c| c.id == id) {
			drop(list);
			if idx + 1 < len {
				self.reorder(idx, idx + 1);
			}
		}
	}

	/// Check if any container is currently downloading.
	pub fn is_any_downloading(&self) -> bool {
		self.containers.read().iter().any(|c| c.phase == ContainerPhase::Downloading)
	}

	/// Find the next container in sort order that is Ready to download.
	pub fn next_queued(&self) -> Option<ContainerId> {
		self.containers.read().iter().find(|c| c.phase == ContainerPhase::Ready).map(|c| c.id)
	}

	/// Mutate a specific container by ID. Returns false if not found.
	pub fn with_container_mut<F>(&mut self, id: ContainerId, f: F) -> bool
	where
		F: FnOnce(&mut ContainerState),
	{
		let mut list = self.containers.write();
		if let Some(cs) = list.iter_mut().find(|c| c.id == id) {
			f(cs);
			true
		} else {
			false
		}
	}

	/// Toggle expand/collapse for a container.
	pub fn toggle_expanded(&mut self, id: ContainerId) {
		self.with_container_mut(id, |cs| {
			cs.expanded = !cs.expanded;
		});
	}

	/// Container count.
	pub fn container_count(&self) -> usize {
		self.containers.read().len()
	}
}
