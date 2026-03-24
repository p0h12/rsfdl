use dioxus::prelude::*;

use crate::state::{AppState, ContainerId, ContainerPhase, FileStatus};
use rsfdl_core::format_bytes;

#[component]
pub fn ProgressPanel(container_id: ContainerId) -> Element {
	let state = use_context::<AppState>();
	let containers = state.containers.read();
	let Some(cs) = containers.iter().find(|c| c.id == container_id) else {
		return rsx! {};
	};

	if cs.phase != ContainerPhase::Downloading {
		return rsx! {};
	}

	let gp = cs.global_progress.clone();
	let mut entries: Vec<_> = cs.file_states.iter().map(|(id, fs)| (*id, fs.clone())).collect();
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
			let s = (secs % 60.0).floor() as u64;
			if mins > 0 { format!("ETA {}:{:02}", mins, s) } else { format!("ETA {}s", s) }
		}
		_ => String::new(),
	};

	// Cancel button
	let cancel_token = cs.cancel_token.clone();
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
					for (_item_id, fs) in entries.iter() {
							{
									let percent = if fs.total_bytes > 0 {
											(fs.bytes_written as f64 / fs.total_bytes as f64 * 100.0).min(100.0)
									} else {
											0.0
									};
									let status_text = match fs.status {
											FileStatus::Downloading => format!("{}/{}", format_bytes(fs.bytes_written), format_bytes(fs.total_bytes)),
											FileStatus::Completed => "completed".to_string(),
											FileStatus::Failed => fs.error.clone().unwrap_or_else(|| "failed".to_string()),
											FileStatus::Cancelled => "cancelled".to_string(),
											FileStatus::Skipped => "skipped".to_string(),
											FileStatus::Pending => "pending".to_string(),
									};
									let status_color = match fs.status {
											FileStatus::Downloading => "var(--color-accent)",
											FileStatus::Completed => "var(--color-success)",
											FileStatus::Failed => "var(--color-error)",
											_ => "var(--color-text-tertiary)",
									};
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
															style: "font-family: var(--font-mono); color: {status_color}; white-space: nowrap; flex-shrink: 0;",
															"{status_text}"
													}
													if fs.status == FileStatus::Downloading {
															div {
																	style: "width: 60px; height: 3px; background: var(--color-background-tertiary); border-radius: 2px; overflow: hidden; flex-shrink: 0;",
																	div {
																			style: "width: {percent:.1}%; height: 100%; background: var(--color-accent); border-radius: 2px; transition: width .3s;",
																	}
															}
													}
											}
									}
							}
					}

					// Cancel button
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
