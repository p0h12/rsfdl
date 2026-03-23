use dioxus::prelude::*;

use crate::state::{AppState, DownloadPhase};

#[component]
pub fn SummaryBanner() -> Element {
	let state = use_context::<AppState>();

	if *state.download_phase.read() != DownloadPhase::Done {
		return rsx! {};
	}

	let summary = state.summary.read();
	let Some(s) = summary.as_ref() else {
		return rsx! {};
	};

	let bg = if s.failed > 0 {
		"bg-red-100 text-red-800"
	} else if s.cancelled > 0 {
		"bg-yellow-100 text-yellow-800"
	} else {
		"bg-green-100 text-green-800"
	};

	rsx! {
			div { class: "px-4 py-3 text-sm font-medium {bg}",
					"Done: {s.total_files} total, {s.completed} completed, {s.skipped} skipped, {s.failed} failed, {s.cancelled} cancelled"
			}
	}
}
