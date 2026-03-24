use std::sync::Arc;
use std::time::Instant;

use dioxus::prelude::*;
use tokio::sync::mpsc;

use rsfdl_core::download::manager::DownloadManager;
use rsfdl_core::download::progress::ProgressEvent;

use crate::state::{AppState, ContainerId, ContainerPhase, DownloadSummary, FileDownloadState, FileStatus, GlobalProgressState};

/// Start downloading files for a specific container.
pub fn start_download(mut state: AppState, container_id: ContainerId) {
	// Extract data from container state
	let (container, settings) = {
		let containers = state.containers.read();
		let Some(cs) = containers.iter().find(|c| c.id == container_id) else {
			return;
		};

		let mut container = cs.container.clone();
		rsfdl_core::container::filter_container(&mut container, &cs.selection);

		let has_files = container.packages.iter().any(|p| !p.file_list.is_empty() || !p.bulk_folder_list.is_empty());
		if !has_files {
			drop(containers);
			state.error_message.set(Some("No files selected".to_string()));
			return;
		}

		let settings = state.settings.read().clone();
		(container, settings)
	};

	// Create download manager
	let (manager, cancel_token, file_cancel_tx) = DownloadManager::new(container, &settings);

	// Update container state
	state.with_container_mut(container_id, |cs| {
		cs.cancel_token = Some(cancel_token);
		cs.file_cancel_tx = Some(Arc::new(file_cancel_tx));
		cs.phase = ContainerPhase::Downloading;
		cs.summary = None;
		cs.file_states.clear();
		cs.global_progress = GlobalProgressState::default();
	});

	let (tx, mut rx) = mpsc::unbounded_channel::<ProgressEvent>();

	// Spawn download manager
	spawn(async move {
		if let Err(e) = manager.run(tx).await {
			state.error_message.set(Some(format!("Download error: {e}")));
			state.with_container_mut(container_id, |cs| {
				cs.phase = ContainerPhase::Done;
			});
		}
	});

	// Spawn progress event consumer with throttling
	spawn(async move {
		let mut last_update = Instant::now();
		let throttle = std::time::Duration::from_millis(100);
		let mut pending_bytes: u64 = 0;

		// Set download start time
		state.with_container_mut(container_id, |cs| {
			cs.global_progress.started_at = Some(Instant::now());
		});

		while let Some(event) = rx.recv().await {
			match event {
				ProgressEvent::Started { item_id, file_name, total_bytes } => {
					state.with_container_mut(container_id, |cs| {
						cs.file_states.insert(
							item_id,
							FileDownloadState {
								file_name,
								total_bytes,
								bytes_written: 0,
								status: FileStatus::Downloading,
								error: None,
							},
						);
						cs.global_progress.files_total += 1;
						cs.global_progress.total_bytes_all += total_bytes;
					});
				}
				ProgressEvent::BytesWritten {
					item_id, bytes_delta, total_written, ..
				} => {
					pending_bytes += bytes_delta;

					let now = Instant::now();
					if now.duration_since(last_update) >= throttle {
						state.with_container_mut(container_id, |cs| {
							if let Some(fs) = cs.file_states.get_mut(&item_id) {
								fs.bytes_written = total_written;
							}
							cs.global_progress.total_written_all += pending_bytes;
						});
						pending_bytes = 0;
						last_update = now;
					}
				}
				ProgressEvent::Completed { item_id } => {
					state.with_container_mut(container_id, |cs| {
						if let Some(fs) = cs.file_states.get_mut(&item_id) {
							fs.bytes_written = fs.total_bytes;
							fs.status = FileStatus::Completed;
						}
						cs.global_progress.files_done += 1;
					});
				}
				ProgressEvent::Skipped { item_id, file_name, .. } => {
					state.with_container_mut(container_id, |cs| {
						cs.file_states.insert(
							item_id,
							FileDownloadState {
								file_name,
								total_bytes: 0,
								bytes_written: 0,
								status: FileStatus::Skipped,
								error: None,
							},
						);
						cs.global_progress.files_total += 1;
						cs.global_progress.files_done += 1;
					});
				}
				ProgressEvent::Failed { item_id, error } => {
					state.with_container_mut(container_id, |cs| {
						if let Some(fs) = cs.file_states.get_mut(&item_id) {
							fs.status = FileStatus::Failed;
							fs.error = Some(error);
						}
						cs.global_progress.files_done += 1;
					});
				}
				ProgressEvent::Retry {
					item_id,
					attempt,
					max_retries,
					delay_seconds,
					error,
				} => {
					// DL-007: Update file status to show retry info
					state.with_container_mut(container_id, |cs| {
						if let Some(fs) = cs.file_states.get_mut(&item_id) {
							fs.error = Some(format!("Retry {}/{} in {}s: {}", attempt, max_retries, delay_seconds, error));
						}
					});
				}
				ProgressEvent::Cancelled { item_id } => {
					state.with_container_mut(container_id, |cs| {
						if let Some(fs) = cs.file_states.get_mut(&item_id) {
							fs.status = FileStatus::Cancelled;
						}
						cs.global_progress.files_done += 1;
					});
				}
				ProgressEvent::AllDone {
					total_files,
					completed,
					failed,
					cancelled,
					skipped,
				} => {
					if pending_bytes > 0 {
						state.with_container_mut(container_id, |cs| {
							cs.global_progress.total_written_all += pending_bytes;
						});
					}
					state.with_container_mut(container_id, |cs| {
						cs.phase = ContainerPhase::Done;
						cs.summary = Some(DownloadSummary {
							total_files,
							completed,
							failed,
							cancelled,
							skipped,
						});
					});

					// Queue: Auto-start next container if none is downloading
					if !state.is_any_downloading()
						&& let Some(next_id) = state.next_queued()
					{
						start_download(state, next_id);
					}
				}
				// Extraction events (deferred)
				ProgressEvent::ExtractionStarted { .. }
				| ProgressEvent::ExtractionProgress { .. }
				| ProgressEvent::ExtractionCompleted { .. }
				| ProgressEvent::ExtractionFailed { .. }
				| ProgressEvent::ExtractionAllDone { .. } => {}
			}
		}
	});
}
