use dioxus::prelude::*;

use crate::components::container_card::ContainerCard;
use crate::components::drop_zone::DropZone;
use crate::icons;
use crate::state::AppState;

#[component]
pub fn MainView() -> Element {
	let mut state = use_context::<AppState>();
	let containers = state.containers.read();
	let count = containers.len();
	let container_ids: Vec<u32> = containers.iter().map(|c| c.id).collect();
	drop(containers);

	if count == 0 {
		return rsx! { DropZone {} };
	}

	rsx! {
			div { class: "flex-1 overflow-y-auto",
					div {
							class: "max-w-[900px] mx-auto px-4 py-5",

							// Container toolbar
							div { class: "flex items-center justify-between mb-4 flex-wrap gap-2",
									div { class: "flex items-center gap-2",
											span {
													style: "font-size: 13px; color: var(--color-text-secondary); font-family: var(--font-mono);",
													"{count} Container"
											}
									}
									div { class: "flex gap-1.5",
											button {
													class: "btn btn-ghost btn-sm",
													onclick: move |_| {
															spawn(async move {
																	crate::components::header::open_sfdl_files_from_dialog(state).await;
															});
													},
													span {
															style: "width: 14px; height: 14px;",
															dangerous_inner_html: icons::PLUS,
													}
													"Hinzufügen"
											}
											button {
													class: "btn btn-ghost btn-sm btn-danger",
													onclick: move |_| {
															state.remove_all();
													},
													span {
															style: "width: 14px; height: 14px;",
															dangerous_inner_html: icons::TRASH_2,
													}
													"Alle entfernen"
											}
									}
							}

							// Container cards
							for id in container_ids {
									ContainerCard { key: "{id}", container_id: id }
							}
					}
			}
	}
}
