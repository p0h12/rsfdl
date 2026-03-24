use dioxus::prelude::*;

use crate::icons;
use crate::state::{AppState, ContainerId};
use rsfdl_core::format_bytes;

#[component]
pub fn FileList(container_id: ContainerId) -> Element {
	let mut state = use_context::<AppState>();
	let containers = state.containers.read();
	let Some(cs) = containers.iter().find(|c| c.id == container_id) else {
		return rsx! {};
	};

	let files = cs.all_files();
	let selected = cs.selected_files.clone();
	let total = files.len();
	let selected_count = selected.iter().filter(|&&s| s).count();
	drop(containers);

	let cid = container_id;
	let cid2 = container_id;

	rsx! {
			div {
					// File toolbar
					div {
							class: "flex items-center justify-between px-4 py-2.5 flex-wrap gap-2",
							style: "border-bottom: 0.5px solid var(--color-border-tertiary);",
							div { class: "flex items-center gap-2",
									button {
											class: "btn btn-ghost btn-sm",
											onclick: move |_| {
													state.with_container_mut(cid, |cs| {
															cs.selected_files.iter_mut().for_each(|s| *s = true);
													});
											},
											"Alle"
									}
									button {
											class: "btn btn-ghost btn-sm",
											onclick: move |_| {
													state.with_container_mut(cid2, |cs| {
															cs.selected_files.iter_mut().for_each(|s| *s = false);
													});
											},
											"Keine"
									}
							}
							span {
									style: "font-size: 12px; color: var(--color-text-secondary); font-family: var(--font-mono);",
									"{selected_count} von {total} ausgewaehlt"
							}
					}

					// File rows
					for (idx, file) in files.iter().enumerate() {
							{
									let is_selected = selected.get(idx).copied().unwrap_or(false);
									let name = file.file_name.clone();
									let size = format_bytes(file.file_size);
									let cid3 = container_id;
									rsx! {
											div {
													class: "flex items-center gap-2 px-4 py-1.5 text-[13px]",
													style: "padding-left: 40px; border-bottom: 0.5px solid var(--color-border-tertiary);",
													onmouseenter: |_| {},

													// Checkbox
													div {
															class: "w-4 h-4 rounded flex items-center justify-center flex-shrink-0 cursor-pointer",
															style: if is_selected {
																	"background: var(--color-accent); border: 0.5px solid var(--color-accent);"
															} else {
																	"background: var(--color-background-secondary); border: 0.5px solid var(--color-border-secondary);"
															},
															onclick: move |_| {
																	state.with_container_mut(cid3, |cs| {
																			if let Some(s) = cs.selected_files.get_mut(idx) {
																					*s = !*s;
																			}
																	});
															},
															if is_selected {
																	span {
																			style: "width: 10px; height: 10px; color: white;",
																			dangerous_inner_html: icons::CHECK,
																	}
															}
													}

													// File name
													span {
															class: "flex-1 min-w-0 truncate",
															style: "color: var(--color-text-primary);",
															"{name}"
													}

													// Size
													span {
															style: "font-size: 11px; font-family: var(--font-mono); color: var(--color-text-tertiary); white-space: nowrap; flex-shrink: 0;",
															"{size}"
													}
											}
									}
							}
					}
			}
	}
}
