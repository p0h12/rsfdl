//! UI-003: Download progress panel.

use dioxus::prelude::*;

use crate::icons;
use crate::state::{AppState, ContainerId, ContainerPhase, FileDownloadState, FileStatus};
use rsfdl_core::format_bytes;

/// UI-003: Download progress panel shown during active downloads.
#[component]
pub fn ProgressPanel(container_id: ContainerId) -> Element {
	let state = use_context::<AppState>();
	let containers = state.containers.read();
	let Some(cs) = containers.iter().find(|c| c.id == container_id) else {
		return rsx! {};
	};

	// BR-UI-010: Only visible during Downloading phase
	if cs.phase != ContainerPhase::Downloading {
		return rsx! {};
	}

	let gp = cs.global_progress.clone();

	// BR-UI-008: Sort entries by status
	let mut entries: Vec<_> = cs.file_states.iter().map(|(id, fs)| (*id, fs.clone())).collect();
	entries.sort_by_key(|(_, fs)| sort_key(fs.status));

	let global_percent = gp.percent();
	let speed = gp.speed_bytes_per_sec();
	let eta = gp.eta_seconds();
	let speed_text = if speed > 0.0 { format!("{}/s", format_bytes(speed as u64)) } else { String::new() };
	let eta_text = format_eta(eta);

	// Cancel tokens
	let cancel_token = cs.cancel_token.clone();
	let file_cancel_tx = cs.file_cancel_tx.clone();
	drop(containers);

	rsx! {
			div {
					class: "px-4 py-3",
					style: "border-top: 0.5px solid var(--color-border-tertiary);",

					// Header with stats
					div { class: "flex items-center justify-between mb-2.5",
							span {
									class: "text-[13px] font-medium",
									style: "color: var(--color-text-primary);",
									"{gp.files_done}/{gp.files_total} files"
							}
							div {
									class: "flex gap-3",
									style: "font-size: 12px; font-family: var(--font-mono); color: var(--color-text-secondary);",
									span { "{format_bytes(gp.total_written_all)}/{format_bytes(gp.total_bytes_all)}" }
									if !speed_text.is_empty() {
											span { "{speed_text}" }
									}
									if !eta_text.is_empty() {
											span { "{eta_text}" }
									}
							}
					}

					// Global progress bar
					div { class: "progress-bar-track mb-2.5",
							div {
									class: "progress-bar-fill",
									style: "width: {global_percent:.1}%;",
							}
					}

					// Per-file rows
					for (item_id, fs) in entries.iter() {
							{
									let item_id = *item_id;
									let percent = file_percent(fs);
									let s_text = status_text(fs);
									let s_color = status_color(fs.status);
									let is_downloading = fs.status == FileStatus::Downloading;

									// A1: Per-file cancel via file_cancel_tx
									let tx_for_button = file_cancel_tx.clone();

									rsx! {
											div {
													class: "flex items-center gap-2 py-1.5",
													style: "font-size: 12px; border-bottom: 0.5px solid var(--color-border-tertiary);",
													span {
															class: "flex-1 min-w-0 truncate",
															style: "font-family: var(--font-mono); color: var(--color-text-primary);",
															"{fs.file_name}"
													}
													span {
															style: "font-family: var(--font-mono); color: {s_color}; white-space: nowrap; flex-shrink: 0;",
															"{s_text}"
													}
													if is_downloading {
															div {
																	style: "width: 60px; height: 3px; background: var(--color-background-tertiary); border-radius: 2px; overflow: hidden; flex-shrink: 0;",
																	div {
																			style: "width: {percent:.1}%; height: 100%; background: var(--color-accent); border-radius: 2px; transition: width .3s;",
																	}
															}
															// A1: Per-file cancel button
															{
																	if let Some(tx) = tx_for_button {
																			rsx! {
																					button {
																							class: "btn-icon btn-danger flex-shrink-0",
																							style: "padding: 2px;",
																							onclick: move |_| { let _ = tx.send(item_id); },
																							span {
																									style: "width: 12px; height: 12px;",
																									dangerous_inner_html: icons::X,
																							}
																					}
																			}
																	} else {
																			rsx! {}
																	}
															}
													}
											}
									}
							}
					}

					// A2: Cancel all button
					if let Some(token) = cancel_token {
							div { class: "flex justify-end mt-3",
									button {
											class: "btn btn-ghost btn-sm btn-danger",
											onclick: move |_| { token.cancel(); },
											"Abbrechen"
									}
							}
					}
			}
	}
}

/// BR-UI-008: Sort key for file status ordering.
pub fn sort_key(status: FileStatus) -> u8 {
	match status {
		FileStatus::Downloading => 0,
		FileStatus::Pending => 1,
		FileStatus::Completed => 2,
		FileStatus::Skipped => 3,
		FileStatus::Failed => 4,
		FileStatus::Cancelled => 5,
	}
}

/// UI-003: Status text for a file download entry.
pub fn status_text(fs: &FileDownloadState) -> String {
	match fs.status {
		FileStatus::Downloading => format!("{}/{}", format_bytes(fs.bytes_written), format_bytes(fs.total_bytes)),
		FileStatus::Completed => "completed".to_string(),
		FileStatus::Failed => fs.error.clone().unwrap_or_else(|| "failed".to_string()),
		FileStatus::Cancelled => "cancelled".to_string(),
		FileStatus::Skipped => "skipped".to_string(),
		FileStatus::Pending => "pending".to_string(),
	}
}

/// UI-003: Status color CSS variable for file status.
pub fn status_color(status: FileStatus) -> &'static str {
	match status {
		FileStatus::Downloading => "var(--color-accent)",
		FileStatus::Completed => "var(--color-success)",
		FileStatus::Failed => "var(--color-error)",
		_ => "var(--color-text-tertiary)",
	}
}

/// UI-003: Per-file progress percentage.
fn file_percent(fs: &FileDownloadState) -> f64 {
	if fs.total_bytes > 0 {
		(fs.bytes_written as f64 / fs.total_bytes as f64 * 100.0).min(100.0)
	} else {
		0.0
	}
}

/// BR-UI-009: Format ETA from seconds to human-readable string.
fn format_eta(eta: Option<f64>) -> String {
	match eta {
		Some(secs) if secs > 0.0 => {
			let mins = (secs / 60.0).floor() as u64;
			let s = (secs % 60.0).floor() as u64;
			if mins > 0 { format!("ETA {}:{:02}", mins, s) } else { format!("ETA {}s", s) }
		}
		_ => String::new(),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	// -------------------------------------------------------
	// UI-003 | BR-UI-008: Sort order
	// -------------------------------------------------------

	/// UI-003 | BR-UI-008: Downloading sorts first.
	#[test]
	fn ui003_sort_downloading_first() {
		assert!(sort_key(FileStatus::Downloading) < sort_key(FileStatus::Pending));
		assert!(sort_key(FileStatus::Pending) < sort_key(FileStatus::Completed));
		assert!(sort_key(FileStatus::Completed) < sort_key(FileStatus::Skipped));
		assert!(sort_key(FileStatus::Skipped) < sort_key(FileStatus::Failed));
		assert!(sort_key(FileStatus::Failed) < sort_key(FileStatus::Cancelled));
	}

	// -------------------------------------------------------
	// UI-003 | Status text
	// -------------------------------------------------------

	/// UI-003 | Status text for downloading shows byte progress.
	#[test]
	fn ui003_status_text_downloading() {
		let fs = FileDownloadState {
			file_name: "a.rar".into(),
			total_bytes: 1_048_576,
			bytes_written: 524_288,
			status: FileStatus::Downloading,
			error: None,
		};
		let text = status_text(&fs);
		assert!(text.contains("KB")); // 512 KB / 1.0 MB
	}

	/// UI-003 | Status text for completed.
	#[test]
	fn ui003_status_text_completed() {
		let fs = FileDownloadState {
			file_name: "a.rar".into(),
			total_bytes: 1000,
			bytes_written: 1000,
			status: FileStatus::Completed,
			error: None,
		};
		assert_eq!(status_text(&fs), "completed");
	}

	/// UI-003 | Status text for failed shows error.
	#[test]
	fn ui003_status_text_failed_with_error() {
		let fs = FileDownloadState {
			file_name: "a.rar".into(),
			total_bytes: 1000,
			bytes_written: 0,
			status: FileStatus::Failed,
			error: Some("Connection refused".into()),
		};
		assert_eq!(status_text(&fs), "Connection refused");
	}

	/// UI-003 | Status text for failed without error.
	#[test]
	fn ui003_status_text_failed_no_error() {
		let fs = FileDownloadState {
			file_name: "a.rar".into(),
			total_bytes: 1000,
			bytes_written: 0,
			status: FileStatus::Failed,
			error: None,
		};
		assert_eq!(status_text(&fs), "failed");
	}

	// -------------------------------------------------------
	// UI-003 | Status color
	// -------------------------------------------------------

	/// UI-003 | Status color for each status.
	#[test]
	fn ui003_status_colors() {
		assert!(status_color(FileStatus::Downloading).contains("accent"));
		assert!(status_color(FileStatus::Completed).contains("success"));
		assert!(status_color(FileStatus::Failed).contains("error"));
		assert!(status_color(FileStatus::Cancelled).contains("tertiary"));
		assert!(status_color(FileStatus::Skipped).contains("tertiary"));
	}

	// -------------------------------------------------------
	// UI-003 | File percent
	// -------------------------------------------------------

	/// UI-003 | File percent normal.
	#[test]
	fn ui003_file_percent_normal() {
		let fs = FileDownloadState {
			file_name: "a.rar".into(),
			total_bytes: 1000,
			bytes_written: 500,
			status: FileStatus::Downloading,
			error: None,
		};
		assert!((file_percent(&fs) - 50.0).abs() < 0.1);
	}

	/// UI-003 | File percent with zero total.
	#[test]
	fn ui003_file_percent_zero_total() {
		let fs = FileDownloadState {
			file_name: "a.rar".into(),
			total_bytes: 0,
			bytes_written: 0,
			status: FileStatus::Downloading,
			error: None,
		};
		assert_eq!(file_percent(&fs), 0.0);
	}

	/// UI-003 | File percent capped at 100.
	#[test]
	fn ui003_file_percent_capped() {
		let fs = FileDownloadState {
			file_name: "a.rar".into(),
			total_bytes: 100,
			bytes_written: 200,
			status: FileStatus::Downloading,
			error: None,
		};
		assert_eq!(file_percent(&fs), 100.0);
	}

	// -------------------------------------------------------
	// UI-003 | BR-UI-009: ETA formatting
	// -------------------------------------------------------

	/// UI-003 | BR-UI-009: ETA with seconds only.
	#[test]
	fn ui003_eta_seconds_only() {
		assert_eq!(format_eta(Some(45.0)), "ETA 45s");
	}

	/// UI-003 | BR-UI-009: ETA with minutes and seconds.
	#[test]
	fn ui003_eta_minutes() {
		assert_eq!(format_eta(Some(125.0)), "ETA 2:05");
	}

	/// UI-003 | BR-UI-009: ETA with zero returns empty.
	#[test]
	fn ui003_eta_zero() {
		assert_eq!(format_eta(Some(0.0)), "");
	}

	/// UI-003 | BR-UI-009: ETA None returns empty.
	#[test]
	fn ui003_eta_none() {
		assert_eq!(format_eta(None), "");
	}
}
