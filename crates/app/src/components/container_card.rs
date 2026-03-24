use dioxus::prelude::*;

use crate::components::container_info::ContainerInfo;
use crate::components::file_list::FileList;
use crate::components::inline_password::InlinePassword;
use crate::components::progress_panel::ProgressPanel;
use crate::components::summary_banner::SummaryBanner;
use crate::icons;
use crate::state::{AppState, ContainerId, ContainerPhase};

#[component]
pub fn ContainerCard(container_id: ContainerId) -> Element {
	let mut state = use_context::<AppState>();
	let containers = state.containers.read();
	let Some(cs) = containers.iter().find(|c| c.id == container_id) else {
		return rsx! {};
	};

	let name = cs.display_name().to_string();
	let expanded = cs.expanded;
	let phase = cs.phase;
	let is_encrypted = cs.is_encrypted();
	let version_tag = cs.version_tag().to_string();
	let is_downloading = phase == ContainerPhase::Downloading;

	// Card state icon
	let icon_svg = if phase == ContainerPhase::NeedsPassword { icons::LOCK } else { icons::FILE_ARCHIVE };
	let icon_class = if phase == ContainerPhase::NeedsPassword {
		"background: var(--color-warning-bg); color: var(--color-warning);"
	} else {
		"background: var(--color-success-bg); color: var(--color-success);"
	};

	drop(containers);

	let cid = container_id;
	let cid2 = container_id;
	let cid3 = container_id;

	rsx! {
			div { class: "sfdl-card",
					// Card header
					div {
							class: "flex items-center gap-2.5 px-4 py-3.5 cursor-pointer select-none",
							style: "border-bottom: 0.5px solid var(--color-border-tertiary);",
							onclick: move |_| { state.toggle_expanded(cid); },

							// Move up/down buttons
							div { class: "flex flex-col gap-0.5",
									style: "color: var(--color-text-tertiary);",
									button {
											class: "btn-icon",
											style: "padding: 1px;",
											onclick: move |evt| {
													evt.stop_propagation();
													state.move_up(cid);
											},
											span {
													style: "width: 12px; height: 12px;",
													dangerous_inner_html: icons::CHEVRON_UP,
											}
									}
									button {
											class: "btn-icon",
											style: "padding: 1px;",
											onclick: move |evt| {
													evt.stop_propagation();
													state.move_down(cid);
											},
											span {
													style: "width: 12px; height: 12px;",
													dangerous_inner_html: icons::CHEVRON_DOWN,
											}
									}
							}

							// State icon
							div {
									class: "w-9 h-9 rounded-lg flex items-center justify-center flex-shrink-0",
									style: "{icon_class}",
									span {
											style: "width: 18px; height: 18px;",
											dangerous_inner_html: icon_svg,
									}
							}

							// Name and badges
							div { class: "flex-1 min-w-0",
									div {
											class: "text-sm font-medium truncate",
											style: "color: var(--color-text-primary);",
											"{name}"
									}
									div {
											class: "flex items-center gap-2 mt-0.5 flex-wrap",
											style: "font-size: 12px; color: var(--color-text-secondary); font-family: var(--font-mono);",
											if is_encrypted {
													span { class: "card-badge badge-encrypted", "ENCRYPTED" }
											}
											span {
													class: "card-badge {badge_class(&version_tag)}",
													"{version_tag}"
											}
											if is_downloading {
													span { class: "card-badge badge-downloading", "DOWNLOADING" }
											}
									}
							}

							// Remove button
							button {
									class: "btn-icon btn-danger flex-shrink-0",
									onclick: move |evt| {
											evt.stop_propagation();
											state.remove_container(cid2);
									},
									span {
											style: "width: 16px; height: 16px;",
											dangerous_inner_html: icons::X,
									}
							}

							// Chevron
							div {
									style: "color: var(--color-text-tertiary); transition: transform .15s; flex-shrink: 0;",
									class: if expanded { "rotate-180" } else { "" },
									span {
											style: "width: 16px; height: 16px;",
											dangerous_inner_html: icons::CHEVRON_DOWN,
									}
							}
					}

					// Card body (when expanded)
					if expanded {
							div {
									match phase {
											ContainerPhase::NeedsPassword => rsx! {
													InlinePassword { container_id: cid3 }
											},
											ContainerPhase::ResolvingBulk => rsx! {
													div {
															class: "px-4 py-8 text-center",
															style: "color: var(--color-text-secondary);",
															"Resolving BulkFolders..."
													}
											},
											ContainerPhase::Ready => rsx! {
													ContainerInfo { container_id: cid3 }
													FileList { container_id: cid3 }
													DownloadActions { container_id: cid3 }
											},
											ContainerPhase::Downloading => rsx! {
													ProgressPanel { container_id: cid3 }
											},
											ContainerPhase::Done => rsx! {
													SummaryBanner { container_id: cid3 }
											},
									}
							}
					}
			}
	}
}

fn badge_class(version: &str) -> &str {
	if version == "v3" { "badge-v3" } else { "badge-v2" }
}

#[component]
fn DownloadActions(container_id: ContainerId) -> Element {
	let state = use_context::<AppState>();
	let containers = state.containers.read();
	let Some(cs) = containers.iter().find(|c| c.id == container_id) else {
		return rsx! {};
	};

	let selected = cs.selected_count();
	let can_start = selected > 0;
	drop(containers);

	let cid = container_id;

	rsx! {
			div {
					class: "flex items-center justify-between px-4 py-3 flex-wrap gap-2",
					style: "border-top: 0.5px solid var(--color-border-tertiary);",
					span {
							style: "font-size: 12px; color: var(--color-text-secondary); font-family: var(--font-mono);",
							"{selected} Dateien zum Download"
					}
					button {
							class: "btn btn-accent",
							disabled: !can_start,
							style: if !can_start { "opacity: 0.4; pointer-events: none;" } else { "" },
							onclick: move |_| {
									crate::components::download_handler::start_download(state, cid);
							},
							span {
									style: "width: 15px; height: 15px;",
									dangerous_inner_html: icons::DOWNLOAD,
							}
							"Download starten"
					}
			}
	}
}
