//! UI-001: Application state management.

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
#[allow(dead_code)]
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

/// UI-001 / BR-UI-002: Per-container lifecycle phase.
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
	#[allow(dead_code)]
	pub fn total_size(&self) -> u64 {
		self.selection.total_size()
	}

	/// Sum of sizes of only selected files.
	#[allow(dead_code)]
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

/// UI-001: Central app state provided via use_context_provider at root.
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
	#[allow(dead_code)]
	pub fn container_count(&self) -> usize {
		self.containers.read().len()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use rsfdl_core::selection::FileSelection;
	use rsfdl_core::sfdl::models::{Connection, FileItem, Package, SfdlContainer, SfdlVersion};

	fn test_container_state(description: &str, host: &str, packages: Vec<Package>) -> ContainerState {
		let container = SfdlContainer {
			description: description.into(),
			connection: Connection {
				host: host.into(),
				port: 21,
				..Connection::default()
			},
			packages,
			..SfdlContainer::default()
		};
		let selection = FileSelection::new(&container, &[]);
		ContainerState {
			id: 1,
			file_path: "/tmp/test.sfdl".into(),
			container,
			phase: ContainerPhase::Ready,
			expanded: true,
			password_error: None,
			selection,
			file_states: HashMap::new(),
			cancel_token: None,
			file_cancel_tx: None,
			global_progress: GlobalProgressState::default(),
			summary: None,
		}
	}

	fn test_package(files: Vec<(&str, u64)>) -> Package {
		Package {
			name: "TestPkg".into(),
			file_list: files
				.into_iter()
				.map(|(name, size)| FileItem {
					file_name: name.into(),
					file_size: size,
					..FileItem::default()
				})
				.collect(),
			..Package::default()
		}
	}

	// -------------------------------------------------------
	// UI-001 | ContainerState: display_name
	// -------------------------------------------------------

	/// UI-001 | ContainerState: display_name returns description when non-empty.
	#[test]
	fn ui001_display_name_from_description() {
		let cs = test_container_state("My.Release.2026", "ftp.example.com", vec![]);
		assert_eq!(cs.display_name(), "My.Release.2026");
	}

	/// UI-001 | ContainerState: display_name falls back to filename when description empty.
	#[test]
	fn ui001_display_name_from_path() {
		let cs = test_container_state("", "ftp.example.com", vec![]);
		assert_eq!(cs.display_name(), "test.sfdl");
	}

	// -------------------------------------------------------
	// UI-001 | ContainerState: version_tag
	// -------------------------------------------------------

	/// UI-001 | ContainerState: version_tag for v3.
	#[test]
	fn ui001_version_tag_v3() {
		let cs = test_container_state("test", "", vec![]);
		assert_eq!(cs.version_tag(), "v3"); // default is V3
	}

	/// UI-001 | ContainerState: version_tag for v2.
	#[test]
	fn ui001_version_tag_v2() {
		let mut cs = test_container_state("test", "", vec![]);
		cs.container.version = SfdlVersion::V2;
		assert_eq!(cs.version_tag(), "v2");
	}

	// -------------------------------------------------------
	// UI-001 | ContainerState: host_display
	// -------------------------------------------------------

	/// UI-001 | ContainerState: host_display with host.
	#[test]
	fn ui001_host_display_with_host() {
		let cs = test_container_state("test", "ftp.example.com", vec![]);
		assert_eq!(cs.host_display(), Some("ftp.example.com:21".into()));
	}

	/// UI-001 | ContainerState: host_display with empty host returns None.
	#[test]
	fn ui001_host_display_empty() {
		let cs = test_container_state("test", "", vec![]);
		assert_eq!(cs.host_display(), None);
	}

	// -------------------------------------------------------
	// UI-001 | ContainerState: file counts and selection
	// -------------------------------------------------------

	/// UI-001 | ContainerState: total_file_count and selected_count.
	#[test]
	fn ui001_file_counts() {
		let pkg = test_package(vec![("a.rar", 1000), ("b.rar", 2000), ("c.nfo", 100)]);
		let cs = test_container_state("test", "", vec![pkg]);
		assert_eq!(cs.total_file_count(), 3);
		assert_eq!(cs.selected_count(), 3); // no exclusion patterns
		assert_eq!(cs.package_count(), 1);
	}

	// -------------------------------------------------------
	// UI-001 | A7: reset_download
	// -------------------------------------------------------

	/// UI-001 | A7: reset_download resets to Ready with cleared state.
	#[test]
	fn ui001_reset_download() {
		let pkg = test_package(vec![("a.rar", 1000)]);
		let mut cs = test_container_state("test", "", vec![pkg]);
		cs.phase = ContainerPhase::Done;
		cs.summary = Some(DownloadSummary {
			total_files: 1,
			completed: 1,
			failed: 0,
			cancelled: 0,
			skipped: 0,
		});

		cs.reset_download();

		assert_eq!(cs.phase, ContainerPhase::Ready);
		assert!(cs.summary.is_none());
		assert!(cs.file_states.is_empty());
		assert!(cs.cancel_token.is_none());
	}

	// -------------------------------------------------------
	// UI-001 | GlobalProgressState
	// -------------------------------------------------------

	/// UI-001 | GlobalProgressState: percent with zero total.
	#[test]
	fn ui001_progress_percent_zero() {
		let gp = GlobalProgressState::default();
		assert_eq!(gp.percent(), 0.0);
	}

	/// UI-001 | GlobalProgressState: percent normal calculation.
	#[test]
	fn ui001_progress_percent_normal() {
		let gp = GlobalProgressState {
			total_bytes_all: 1000,
			total_written_all: 500,
			..Default::default()
		};
		assert!((gp.percent() - 50.0).abs() < 0.1);
	}

	/// UI-001 | GlobalProgressState: percent capped at 100.
	#[test]
	fn ui001_progress_percent_capped() {
		let gp = GlobalProgressState {
			total_bytes_all: 100,
			total_written_all: 200,
			..Default::default()
		};
		assert_eq!(gp.percent(), 100.0);
	}

	/// UI-001 | GlobalProgressState: eta_seconds returns None when no speed.
	#[test]
	fn ui001_eta_no_speed() {
		let gp = GlobalProgressState::default();
		assert!(gp.eta_seconds().is_none());
	}

	/// UI-001 | GlobalProgressState: speed is 0 when no start time.
	#[test]
	fn ui001_speed_no_start() {
		let gp = GlobalProgressState {
			total_written_all: 1000,
			..Default::default()
		};
		assert_eq!(gp.speed_bytes_per_sec(), 0.0);
	}
}
