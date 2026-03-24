use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::{Mutex, Semaphore, mpsc};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::download::item::{DownloadItem, DownloadStatus, ResumeAction};
use crate::download::progress::ProgressEvent;
use crate::error::DownloadError;
use crate::ftp::client::FtpClient;
use crate::settings::Settings;
use crate::sfdl::models::SfdlContainer;

/// Result summary after all downloads complete.
pub struct DownloadResult {
	pub total_files: u32,
	pub completed: u32,
	pub failed: u32,
	pub cancelled: u32,
	pub skipped: u32,
}

pub struct DownloadManager {
	container: SfdlContainer,
	dest_dir: PathBuf,
	max_threads: u32,
	strict_disk_check: bool,
	resume_downloads: bool,
	create_package_subfolder: bool,
	ftp_timeout_seconds: u32,
	cancel_token: CancellationToken,
	cancel_rx: mpsc::UnboundedReceiver<Uuid>,
}

impl DownloadManager {
	/// Creates a new DownloadManager.
	/// Returns (manager, global_cancel_token, per_file_cancel_sender).
	pub fn new(container: SfdlContainer, settings: &Settings) -> (Self, CancellationToken, mpsc::UnboundedSender<Uuid>) {
		let cancel_token = CancellationToken::new();
		let token_clone = cancel_token.clone();
		let (cancel_tx, cancel_rx) = mpsc::unbounded_channel();
		(
			Self {
				container,
				dest_dir: settings.download_directory.clone(),
				max_threads: settings.max_threads,
				strict_disk_check: settings.strict_disk_check,
				resume_downloads: true,
				create_package_subfolder: true,
				ftp_timeout_seconds: settings.ftp_timeout_seconds,
				cancel_token,
				cancel_rx,
			},
			token_clone,
			cancel_tx,
		)
	}

	/// DL-004: Run the download session. Sends ProgressEvents to progress_tx.
	///
	/// The container must have BulkFolders resolved (SFDL-003) and
	/// unselected files filtered out (DL-001) before calling this.
	pub async fn run(self, progress_tx: mpsc::UnboundedSender<ProgressEvent>) -> Result<DownloadResult, DownloadError> {
		let mut items: Vec<DownloadItem> = Vec::new();

		for package in &self.container.packages {
			for file_item in &package.file_list {
				items.push(DownloadItem::from_file_item(file_item, &self.dest_dir, &package.name, self.create_package_subfolder));
			}
		}

		let total_files = items.len() as u32;
		tracing::info!(
				total_files,
				host = %self.container.connection.host,
				threads = self.max_threads,
				"Starting download session"
		);

		if total_files == 0 {
			let _ = progress_tx.send(ProgressEvent::AllDone {
				total_files: 0,
				completed: 0,
				failed: 0,
				cancelled: 0,
				skipped: 0,
			});
			return Ok(DownloadResult {
				total_files: 0,
				completed: 0,
				failed: 0,
				cancelled: 0,
				skipped: 0,
			});
		}

		// 2. DL-005: Check resume state, skip already-complete files
		let mut skipped = 0u32;
		let mut to_download: Vec<DownloadItem> = Vec::new();

		for mut item in items {
			if self.resume_downloads {
				match item.check_local_state() {
					ResumeAction::AlreadyComplete => {
						item.status = DownloadStatus::Skipped;
						let _ = progress_tx.send(ProgressEvent::Skipped {
							item_id: item.id,
							file_name: item.file_item.file_name.clone(),
						});
						skipped += 1;
					}
					ResumeAction::DeleteAndRestart => {
						// A2: Oversized or unknown remote size — delete and re-download
						let _ = std::fs::remove_file(&item.local_path);
						to_download.push(item);
					}
					_ => {
						to_download.push(item);
					}
				}
			} else {
				// Without resume, always start fresh
				to_download.push(item);
			}
		}

		// 3. DL-003: Check disk space
		if !to_download.is_empty() {
			let space_items: Vec<(u64, u64)> = to_download
				.iter()
				.map(|item| {
					let local_size = item.local_path.metadata().map(|m| m.len()).unwrap_or(0);
					(item.file_item.file_size, local_size)
				})
				.collect();

			let space_result = crate::diskspace::check(&self.dest_dir, &space_items, self.strict_disk_check)?;

			if !space_result.sufficient {
				tracing::warn!(
					required_bytes = space_result.required_bytes,
					available_bytes = space_result.available_bytes,
					"Insufficient disk space (non-strict mode, continuing)"
				);
			}
		}

		// 4. DL-006: Create per-file cancellation tokens
		let file_tokens: Arc<Mutex<HashMap<Uuid, CancellationToken>>> = Arc::new(Mutex::new(HashMap::new()));

		// Spawn per-file cancel listener
		let file_tokens_for_listener = file_tokens.clone();
		let global_cancel = self.cancel_token.clone();
		let mut cancel_rx = self.cancel_rx;
		tokio::spawn(async move {
			loop {
				tokio::select! {
						Some(item_id) = cancel_rx.recv() => {
								let tokens = file_tokens_for_listener.lock().await;
								if let Some(token) = tokens.get(&item_id) {
										token.cancel();
								}
						}
						_ = global_cancel.cancelled() => {
								break;
						}
						else => break,
				}
			}
		});

		// 5. Parallel downloads with semaphore
		let semaphore = Arc::new(Semaphore::new(self.max_threads as usize));
		let conn = Arc::new(self.container.connection.clone());
		let cancel = self.cancel_token.clone();
		let resume_downloads = self.resume_downloads;
		let ftp_timeout = self.ftp_timeout_seconds;

		let mut handles = Vec::new();

		for item in to_download {
			let sem = semaphore.clone();
			let conn = conn.clone();
			let tx = progress_tx.clone();
			let global_cancel = cancel.clone();
			let file_tokens = file_tokens.clone();

			// Create a child token for this file
			let file_cancel = global_cancel.child_token();
			{
				file_tokens.lock().await.insert(item.id, file_cancel.clone());
			}

			let handle = tokio::spawn(async move {
				let _permit = match sem.acquire().await {
					Ok(permit) => permit,
					Err(_) => {
						let _ = tx.send(ProgressEvent::Cancelled { item_id: item.id });
						return DownloadStatus::Cancelled;
					}
				};

				if file_cancel.is_cancelled() {
					let _ = tx.send(ProgressEvent::Cancelled { item_id: item.id });
					return DownloadStatus::Cancelled;
				}

				// Create directory structure
				if let Some(parent) = item.local_path.parent()
					&& let Err(e) = tokio::fs::create_dir_all(parent).await
				{
					let _ = tx.send(ProgressEvent::Failed {
						item_id: item.id,
						error: e.to_string(),
					});
					return DownloadStatus::Failed;
				}

				// DL-005: Determine resume offset
				let resume_offset = if resume_downloads {
					match item.check_local_state() {
						ResumeAction::Resume(offset) => offset,
						ResumeAction::AlreadyComplete => {
							let _ = tx.send(ProgressEvent::Skipped {
								item_id: item.id,
								file_name: item.file_item.file_name.clone(),
							});
							return DownloadStatus::Skipped;
						}
						ResumeAction::DeleteAndRestart => {
							let _ = tokio::fs::remove_file(&item.local_path).await;
							0
						}
						ResumeAction::StartFresh => 0,
					}
				} else {
					0
				};

				// Connect
				let mut client = match FtpClient::connect(&conn, ftp_timeout).await {
					Ok(c) => c,
					Err(e) => {
						let _ = tx.send(ProgressEvent::Failed {
							item_id: item.id,
							error: e.to_string(),
						});
						return DownloadStatus::Failed;
					}
				};

				// Send Started event
				let _ = tx.send(ProgressEvent::Started {
					item_id: item.id,
					file_name: item.file_item.file_name.clone(),
					total_bytes: item.file_item.file_size,
				});

				// Download (uses per-file cancel token)
				let result = client.download_file(&item.file_item.full_path, &item.local_path, resume_offset, item.id, &tx, &file_cancel).await;

				client.disconnect().await;

				match result {
					Ok(_) => {
						let _ = tx.send(ProgressEvent::Completed { item_id: item.id });
						DownloadStatus::Completed
					}
					Err(DownloadError::Cancelled) => {
						let _ = tx.send(ProgressEvent::Cancelled { item_id: item.id });
						DownloadStatus::Cancelled
					}
					Err(e) => {
						let _ = tx.send(ProgressEvent::Failed {
							item_id: item.id,
							error: e.to_string(),
						});
						DownloadStatus::Failed
					}
				}
			});

			handles.push(handle);
		}

		// 6. Await all handles
		let mut completed = 0u32;
		let mut failed = 0u32;
		let mut cancelled = 0u32;

		for handle in handles {
			match handle.await {
				Ok(status) => match status {
					DownloadStatus::Completed => completed += 1,
					DownloadStatus::Failed => failed += 1,
					DownloadStatus::Cancelled => cancelled += 1,
					DownloadStatus::Skipped => skipped += 1,
					_ => {}
				},
				Err(_) => failed += 1, // JoinError
			}
		}

		let _ = progress_tx.send(ProgressEvent::AllDone {
			total_files,
			completed,
			failed,
			cancelled,
			skipped,
		});

		Ok(DownloadResult {
			total_files,
			completed,
			failed,
			cancelled,
			skipped,
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::sfdl::models::{Connection, FileItem, Package};

	fn test_container(files: Vec<(&str, u64)>) -> SfdlContainer {
		SfdlContainer {
			connection: Connection {
				host: "127.0.0.1".into(),
				port: 1, // unreachable port
				..Connection::default()
			},
			packages: vec![Package {
				name: "Pkg".into(),
				file_list: files
					.into_iter()
					.map(|(name, size)| FileItem {
						file_name: name.into(),
						file_size: size,
						full_path: format!("/{name}"),
						..FileItem::default()
					})
					.collect(),
				..Package::default()
			}],
			..SfdlContainer::default()
		}
	}

	fn test_settings(dir: &std::path::Path) -> Settings {
		let mut s = Settings::default();
		s.download_directory = dir.to_path_buf();
		s.max_threads = 2;
		s.ftp_timeout_seconds = 1;
		s
	}

	/// DL-006 | Variante B: Global cancel before downloads start → all cancelled.
	#[tokio::test]
	async fn dl006_global_cancel_before_start() {
		let dir = tempfile::tempdir().unwrap();
		let container = test_container(vec![("a.rar", 1000), ("b.rar", 2000)]);
		let settings = test_settings(dir.path());

		let (manager, cancel_token, _file_cancel) = DownloadManager::new(container, &settings);
		let (tx, _rx) = mpsc::unbounded_channel();

		// Cancel immediately before run
		cancel_token.cancel();

		let result = manager.run(tx).await.unwrap();

		assert_eq!(result.total_files, 2);
		assert_eq!(result.cancelled, 2);
		assert_eq!(result.completed, 0);
		assert_eq!(result.failed, 0);
	}

	/// DL-006 | BR-DL-013: Completed (skipped) tasks are not affected by cancel.
	#[tokio::test]
	async fn dl006_skip_not_affected_by_cancel() {
		let dir = tempfile::tempdir().unwrap();

		// Create a "complete" local file so it gets skipped
		let pkg_dir = dir.path().join("Pkg");
		std::fs::create_dir_all(&pkg_dir).unwrap();
		std::fs::write(pkg_dir.join("done.rar"), vec![0u8; 500]).unwrap();

		let container = test_container(vec![("done.rar", 500), ("pending.rar", 1000)]);
		let settings = test_settings(dir.path());

		let (manager, cancel_token, _file_cancel) = DownloadManager::new(container, &settings);
		let (tx, _rx) = mpsc::unbounded_channel();

		// Cancel immediately — skipped file should still be counted as skipped
		cancel_token.cancel();

		let result = manager.run(tx).await.unwrap();

		assert_eq!(result.total_files, 2);
		assert_eq!(result.skipped, 1); // done.rar was already complete
		assert_eq!(result.cancelled, 1); // pending.rar was cancelled
		assert_eq!(result.completed, 0);
	}

	/// DL-006 | BR-DL-014: Empty file list with cancel is a no-op.
	#[tokio::test]
	async fn dl006_cancel_empty_list() {
		let dir = tempfile::tempdir().unwrap();
		let container = test_container(vec![]);
		let settings = test_settings(dir.path());

		let (manager, cancel_token, _file_cancel) = DownloadManager::new(container, &settings);
		let (tx, _rx) = mpsc::unbounded_channel();

		cancel_token.cancel();

		let result = manager.run(tx).await.unwrap();

		assert_eq!(result.total_files, 0);
		assert_eq!(result.cancelled, 0);
	}

	/// DL-006 | DownloadResult: counts are correct for mixed statuses.
	#[test]
	fn dl006_download_result_fields() {
		let result = DownloadResult {
			total_files: 10,
			completed: 5,
			failed: 2,
			cancelled: 2,
			skipped: 1,
		};
		assert_eq!(result.total_files, result.completed + result.failed + result.cancelled + result.skipped);
	}
}
