use std::time::Instant;

use dioxus::prelude::*;
use tokio::sync::mpsc;

use rsfdl_core::download::manager::DownloadManager;
use rsfdl_core::download::progress::ProgressEvent;

use crate::components::container_info::ContainerInfo;
use crate::components::file_list::FileList;
use crate::components::password_dialog::PasswordDialog;
use crate::components::progress_panel::ProgressPanel;
use crate::components::summary_banner::SummaryBanner;
use crate::state::{
    AppState, DownloadPhase, DownloadSummary, FileDownloadState, FileStatus, GlobalProgressState,
};

#[component]
pub fn MainView() -> Element {
    let mut state = use_context::<AppState>();
    let has_container = state.container.read().is_some();
    let needs_password = *state.needs_password.read();
    let phase = *state.download_phase.read();

    rsx! {
        div { class: "flex flex-col flex-1 overflow-hidden",
            // Password dialog (modal overlay)
            PasswordDialog {}

            if has_container && !needs_password {
                // Container info
                ContainerInfo {}

                // File list
                FileList {}

                // Action buttons
                div { class: "px-4 py-3 border-t bg-white flex justify-center gap-3",
                    match phase {
                        DownloadPhase::Idle => rsx! {
                            button {
                                class: "px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded text-sm font-medium",
                                onclick: move |_| start_download(state),
                                "Start Download"
                            }
                        },
                        DownloadPhase::Downloading => rsx! {
                            button {
                                class: "px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded text-sm font-medium",
                                onclick: move |_| {
                                    if let Some(token) = state.cancel_token.read().as_ref() {
                                        token.cancel();
                                    }
                                },
                                "Cancel"
                            }
                        },
                        DownloadPhase::Done => rsx! {
                            button {
                                class: "px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded text-sm font-medium",
                                onclick: move |_| {
                                    state.reset_download_state();
                                },
                                "Reset"
                            }
                        },
                    }
                }

                // Progress panel (during/after download)
                ProgressPanel {}

                // Summary banner
                SummaryBanner {}

            } else if !has_container {
                // Empty state
                div { class: "flex-1 flex items-center justify-center text-gray-400",
                    p { class: "text-lg", "Open an .sfdl file to begin" }
                }
            }
        }
    }
}

fn start_download(mut state: AppState) {
    let container_opt = state.container.read().clone();
    let Some(mut container) = container_opt else {
        return;
    };

    let settings = state.settings.read().clone();

    // Filter out unselected files
    let selected = state.selected_files.read().clone();
    let mut sel_iter = selected.iter();
    for package in &mut container.packages {
        package
            .file_list
            .retain(|_| *sel_iter.next().unwrap_or(&true));
    }

    // Check if there's anything to download
    let has_files = container
        .packages
        .iter()
        .any(|p| !p.file_list.is_empty() || !p.bulk_folder_list.is_empty());
    if !has_files {
        state
            .error_message
            .set(Some("No files selected".to_string()));
        return;
    }

    // Create download manager
    let (manager, cancel_token, file_cancel_tx) = DownloadManager::new(container, &settings);
    state.cancel_token.set(Some(cancel_token));
    state
        .file_cancel_tx
        .set(Some(std::sync::Arc::new(file_cancel_tx)));
    state.download_phase.set(DownloadPhase::Downloading);
    state.summary.set(None);
    state.file_states.write().clear();
    state.error_message.set(None);
    state.global_progress.set(GlobalProgressState::default());

    let (tx, mut rx) = mpsc::unbounded_channel::<ProgressEvent>();

    // Spawn download manager
    spawn(async move {
        if let Err(e) = manager.run(tx).await {
            state
                .error_message
                .set(Some(format!("Download error: {e}")));
            state.download_phase.set(DownloadPhase::Done);
        }
    });

    // Spawn progress event consumer with throttling
    spawn(async move {
        let mut last_update = Instant::now();
        let throttle = std::time::Duration::from_millis(100);
        let mut pending_bytes: u64 = 0;

        // Set download start time
        state.global_progress.write().started_at = Some(Instant::now());

        while let Some(event) = rx.recv().await {
            match event {
                ProgressEvent::Started {
                    item_id,
                    file_name,
                    total_bytes,
                } => {
                    state.file_states.write().insert(
                        item_id,
                        FileDownloadState {
                            file_name,
                            total_bytes,
                            bytes_written: 0,
                            status: FileStatus::Downloading,
                            error: None,
                        },
                    );
                    let mut gp = state.global_progress.write();
                    gp.files_total += 1;
                    gp.total_bytes_all += total_bytes;
                }
                ProgressEvent::BytesWritten {
                    item_id,
                    bytes_delta,
                    total_written,
                    ..
                } => {
                    pending_bytes += bytes_delta;

                    // Throttle UI updates to avoid excessive re-renders
                    let now = Instant::now();
                    if now.duration_since(last_update) >= throttle {
                        if let Some(fs) = state.file_states.write().get_mut(&item_id) {
                            fs.bytes_written = total_written;
                        }
                        state.global_progress.write().total_written_all += pending_bytes;
                        pending_bytes = 0;
                        last_update = now;
                    }
                }
                ProgressEvent::Completed { item_id } => {
                    if let Some(fs) = state.file_states.write().get_mut(&item_id) {
                        fs.bytes_written = fs.total_bytes;
                        fs.status = FileStatus::Completed;
                    }
                    state.global_progress.write().files_done += 1;
                }
                ProgressEvent::Skipped {
                    item_id, file_name, ..
                } => {
                    state.file_states.write().insert(
                        item_id,
                        FileDownloadState {
                            file_name,
                            total_bytes: 0,
                            bytes_written: 0,
                            status: FileStatus::Skipped,
                            error: None,
                        },
                    );
                    let mut gp = state.global_progress.write();
                    gp.files_total += 1;
                    gp.files_done += 1;
                }
                ProgressEvent::Failed { item_id, error } => {
                    if let Some(fs) = state.file_states.write().get_mut(&item_id) {
                        fs.status = FileStatus::Failed;
                        fs.error = Some(error);
                    }
                    state.global_progress.write().files_done += 1;
                }
                ProgressEvent::Cancelled { item_id } => {
                    if let Some(fs) = state.file_states.write().get_mut(&item_id) {
                        fs.status = FileStatus::Cancelled;
                    }
                    state.global_progress.write().files_done += 1;
                }
                ProgressEvent::AllDone {
                    total_files,
                    completed,
                    failed,
                    cancelled,
                    skipped,
                } => {
                    if pending_bytes > 0 {
                        state.global_progress.write().total_written_all += pending_bytes;
                        pending_bytes = 0;
                    }
                    state.download_phase.set(DownloadPhase::Done);
                    state.summary.set(Some(DownloadSummary {
                        total_files,
                        completed,
                        failed,
                        cancelled,
                        skipped,
                    }));
                }
                // UC-14: Extraction events (GUI integration deferred)
                ProgressEvent::ExtractionStarted { .. }
                | ProgressEvent::ExtractionProgress { .. }
                | ProgressEvent::ExtractionCompleted { .. }
                | ProgressEvent::ExtractionFailed { .. }
                | ProgressEvent::ExtractionAllDone { .. } => {}
            }
        }
    });
}
