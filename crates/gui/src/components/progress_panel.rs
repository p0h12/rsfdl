use dioxus::prelude::*;
use uuid::Uuid;

use rsfdl_core::format_bytes;

use crate::state::{AppState, DownloadPhase, FileDownloadState, FileStatus};

#[component]
pub fn ProgressPanel() -> Element {
	let state = use_context::<AppState>();
	let phase = *state.download_phase.read();

	if phase == DownloadPhase::Idle {
		return rsx! {};
	}

	let gp = state.global_progress.read().clone();
	let file_states = state.file_states.read();
	let mut entries: Vec<_> = file_states.iter().map(|(id, fs)| (*id, fs.clone())).collect();
	// Show active downloads first, then completed/failed
	entries.sort_by_key(|(_, fs)| match fs.status {
		FileStatus::Downloading => 0,
		FileStatus::Pending => 1,
		FileStatus::Completed => 2,
		FileStatus::Skipped => 3,
		FileStatus::Failed => 4,
		FileStatus::Cancelled => 5,
	});

	let global_percent = gp.percent();
	let speed = gp.speed_bytes_per_sec();
	let eta = gp.eta_seconds();

	let speed_text = if speed > 0.0 { format!("{}/s", format_bytes(speed as u64)) } else { String::new() };

	let eta_text = match eta {
		Some(secs) if secs > 0.0 => {
			let mins = (secs / 60.0).floor() as u64;
			let secs = (secs % 60.0).floor() as u64;
			if mins > 0 { format!("ETA {}:{:02}", mins, secs) } else { format!("ETA {}s", secs) }
		}
		_ => String::new(),
	};

	rsx! {
			div { class: "border-t bg-white",
					// Global progress
					div { class: "px-4 py-2 bg-gray-100 border-b",
							div { class: "flex justify-between text-sm text-gray-700 mb-1",
									span { class: "font-medium",
											"{gp.files_done}/{gp.files_total} files"
									}
									span { class: "text-gray-500",
											"{format_bytes(gp.total_written_all)}/{format_bytes(gp.total_bytes_all)} {speed_text} {eta_text}"
									}
							}
							div { class: "w-full bg-gray-200 rounded-full h-2",
									div {
											class: "h-2 rounded-full bg-cyan-500 transition-all duration-200",
											style: "width: {global_percent:.1}%",
									}
							}
					}
					// Per-file progress
					div { class: "max-h-48 overflow-y-auto",
							for (item_id, fs) in entries.iter() {
									{render_progress_row(state, *item_id, fs)}
							}
					}
			}
	}
}

fn render_progress_row(state: AppState, item_id: Uuid, fs: &FileDownloadState) -> Element {
	let percent = if fs.total_bytes > 0 {
		(fs.bytes_written as f64 / fs.total_bytes as f64 * 100.0).min(100.0)
	} else {
		0.0
	};

	let (bar_color, status_text) = match fs.status {
		FileStatus::Downloading => ("bg-blue-500", format!("{} / {}", format_bytes(fs.bytes_written), format_bytes(fs.total_bytes))),
		FileStatus::Completed => ("bg-green-500", "completed".to_string()),
		FileStatus::Failed => ("bg-red-500", fs.error.clone().unwrap_or("failed".to_string())),
		FileStatus::Cancelled => ("bg-yellow-500", "cancelled".to_string()),
		FileStatus::Skipped => ("bg-gray-300", "skipped".to_string()),
		FileStatus::Pending => ("bg-gray-300", "pending".to_string()),
	};

	let is_downloading = fs.status == FileStatus::Downloading;

	rsx! {
			div { class: "px-4 py-1.5 border-b border-gray-100 text-sm",
					div { class: "flex justify-between items-center mb-1",
							span { class: "truncate text-gray-800 mr-2", "{fs.file_name}" }
							div { class: "flex items-center gap-2 whitespace-nowrap",
									span { class: "text-gray-500", "{status_text}" }
									if is_downloading {
											button {
													class: "text-red-500 hover:text-red-700 text-xs font-medium px-1",
													title: "Cancel this file",
													onclick: move |_| {
															if let Some(tx) = state.file_cancel_tx.read().as_ref() {
																	let _ = tx.send(item_id);
															}
													},
													"X"
											}
									}
							}
					}
					if is_downloading {
							div { class: "w-full bg-gray-200 rounded-full h-1.5",
									div {
											class: "h-1.5 rounded-full {bar_color}",
											style: "width: {percent:.1}%",
									}
							}
					}
			}
	}
}
