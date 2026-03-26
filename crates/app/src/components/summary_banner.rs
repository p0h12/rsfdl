//! UI-004: Download result summary banner.

use dioxus::prelude::*;

use crate::components::progress_panel::{sort_key, status_color, status_text};
use crate::state::{AppState, ContainerId, ContainerPhase, DownloadSummary};

/// UI-004: Summary banner shown after download completes.
///
/// Shows color-coded banner (BR-UI-011), statistics (BR-UI-012),
/// per-file status list, and Reset button (A1).
#[component]
pub fn SummaryBanner(container_id: ContainerId) -> Element {
	let mut state = use_context::<AppState>();
	let containers = state.containers.read();
	let Some(cs) = containers.iter().find(|c| c.id == container_id) else {
		return rsx! {};
	};

	if cs.phase != ContainerPhase::Done {
		return rsx! {};
	}

	let Some(s) = &cs.summary else {
		return rsx! {};
	};

	let variant = banner_variant(s);
	let text = summary_text(s);

	// Per-file status list (spec: "Pro-Datei-Status bleibt sichtbar")
	let mut entries: Vec<_> = cs.file_states.values().cloned().collect();
	entries.sort_by_key(|fs| sort_key(fs.status));

	drop(containers);

	let cid = container_id;

	rsx! {
			div { class: "p-4",
					// BR-UI-011: Color-coded summary banner
					div {
							style: "padding: 14px 16px; border-radius: var(--border-radius-md); display: flex; align-items: center; gap: 12px; background: var(--color-{variant}-bg); border: 0.5px solid; color: var(--color-{variant});",
							span { class: "text-[13px] font-medium", "{text}" }
					}

					// Per-file status detail list
					if !entries.is_empty() {
							div {
									class: "mt-3",
									style: "border-top: 0.5px solid var(--color-border-tertiary);",
									for (idx, fs) in entries.iter().enumerate() {
											{
													let s_text = status_text(fs);
													let s_color = status_color(fs.status);
													rsx! {
															div {
															key: "{idx}",
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
															}
													}
											}
									}
							}
					}

					// A1: Reset button
					div { class: "flex justify-end mt-3",
							button {
									class: "btn btn-ghost btn-sm",
									onclick: move |_| {
											state.with_container_mut(cid, |cs| {
													cs.reset_download();
											});
									},
									"Reset"
							}
					}
			}
	}
}

/// BR-UI-011: Determine banner color variant from summary.
///
/// - `failed > 0` → "error" (red)
/// - `cancelled > 0` (no failures) → "warning" (yellow)
/// - else → "success" (green)
pub fn banner_variant(s: &DownloadSummary) -> &'static str {
	if s.failed > 0 {
		"error"
	} else if s.cancelled > 0 {
		"warning"
	} else {
		"success"
	}
}

/// BR-UI-012: Format summary statistics text.
pub fn summary_text(s: &DownloadSummary) -> String {
	format!(
		"Done: {} total, {} completed, {} skipped, {} failed, {} cancelled",
		s.total_files, s.completed, s.skipped, s.failed, s.cancelled
	)
}

#[cfg(test)]
mod tests {
	use super::*;

	fn summary(completed: u32, failed: u32, cancelled: u32, skipped: u32) -> DownloadSummary {
		DownloadSummary {
			total_files: completed + failed + cancelled + skipped,
			completed,
			failed,
			cancelled,
			skipped,
		}
	}

	// -------------------------------------------------------
	// UI-004 | BR-UI-011: Banner color variant
	// -------------------------------------------------------

	/// UI-004 | BR-UI-011: All success → green.
	#[test]
	fn ui004_banner_all_success() {
		assert_eq!(banner_variant(&summary(5, 0, 0, 0)), "success");
	}

	/// UI-004 | BR-UI-011: Some skipped, no failures → green.
	#[test]
	fn ui004_banner_with_skipped() {
		assert_eq!(banner_variant(&summary(3, 0, 0, 2)), "success");
	}

	/// UI-004 | BR-UI-011: Cancelled (no failures) → yellow.
	#[test]
	fn ui004_banner_cancelled() {
		assert_eq!(banner_variant(&summary(2, 0, 3, 0)), "warning");
	}

	/// UI-004 | BR-UI-011: Failed → red (even with cancelled).
	#[test]
	fn ui004_banner_failed() {
		assert_eq!(banner_variant(&summary(1, 2, 1, 0)), "error");
	}

	/// UI-004 | BR-UI-011: Failed only → red.
	#[test]
	fn ui004_banner_all_failed() {
		assert_eq!(banner_variant(&summary(0, 5, 0, 0)), "error");
	}

	// -------------------------------------------------------
	// UI-004 | BR-UI-012: Summary text format
	// -------------------------------------------------------

	/// UI-004 | BR-UI-012: Summary text matches spec format.
	#[test]
	fn ui004_summary_text_format() {
		let s = summary(3, 1, 1, 2);
		let text = summary_text(&s);
		assert_eq!(text, "Done: 7 total, 3 completed, 2 skipped, 1 failed, 1 cancelled");
	}

	/// UI-004 | BR-UI-012: Summary text all zeros.
	#[test]
	fn ui004_summary_text_empty() {
		let s = summary(0, 0, 0, 0);
		let text = summary_text(&s);
		assert_eq!(text, "Done: 0 total, 0 completed, 0 skipped, 0 failed, 0 cancelled");
	}

	/// UI-004 | BR-UI-012: Summary text all completed.
	#[test]
	fn ui004_summary_text_all_completed() {
		let s = summary(10, 0, 0, 0);
		let text = summary_text(&s);
		assert!(text.contains("10 total"));
		assert!(text.contains("10 completed"));
		assert!(text.contains("0 failed"));
	}
}
