use dioxus::prelude::*;

use crate::state::{AppState, ContainerId, ContainerPhase};

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

	let (banner_class, icon) = if s.failed > 0 {
		("error", "error")
	} else if s.cancelled > 0 {
		("warning", "warning")
	} else {
		("success", "success")
	};

	let text = format!(
		"Done: {} total, {} completed, {} skipped, {} failed, {} cancelled",
		s.total_files, s.completed, s.skipped, s.failed, s.cancelled
	);
	drop(containers);

	let cid = container_id;

	rsx! {
			div { class: "p-4",
					div {
							class: "result-banner {banner_class}",
							style: "margin: 0; padding: 14px 16px; border-radius: var(--border-radius-md); display: flex; align-items: center; gap: 12px; background: var(--color-{icon}-bg); border: 0.5px solid; color: var(--color-{icon});",
							span { class: "text-[13px] font-medium", "{text}" }
					}

					// Reset button
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
