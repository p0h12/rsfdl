use dioxus::prelude::*;

use crate::icons;
use crate::state::AppState;

#[component]
pub fn DropZone() -> Element {
	let state = use_context::<AppState>();

	rsx! {
			div { class: "flex-1 flex items-center justify-center px-4 py-8",
					div { class: "max-w-[900px] w-full mx-auto",
							div {
									class: "drop-zone",
									onclick: move |_| {
											spawn(async move {
													crate::components::header::open_sfdl_files_from_dialog(state).await;
											});
									},

									// Icon
									div {
											class: "w-14 h-14 mx-auto mb-5 rounded-lg flex items-center justify-center",
											style: "background: var(--color-background-secondary); border: 0.5px solid var(--color-border-tertiary); color: var(--color-text-secondary);",
											span {
													style: "width: 26px; height: 26px;",
													dangerous_inner_html: icons::FILE_DOWN,
											}
									}

									// Title
									p {
											class: "text-lg font-medium mb-1",
											style: "color: var(--color-text-primary);",
											"SFDL-Container laden"
									}

									// Subtitle
									p {
											class: "text-sm mb-5",
											style: "color: var(--color-text-secondary);",
											"Klicken um Dateien auszuwaehlen"
									}

									// Button
									button {
											class: "btn btn-primary btn-lg",
											onclick: move |evt| {
													evt.stop_propagation();
													spawn(async move {
															crate::components::header::open_sfdl_files_from_dialog(state).await;
													});
											},
											span {
													style: "width: 16px; height: 16px;",
													dangerous_inner_html: icons::FOLDER_OPEN,
											}
											"Datei auswaehlen"
									}

									// Hint
									p {
											class: "mt-5",
											style: "font-size: 12px; color: var(--color-text-tertiary); font-family: var(--font-mono);",
											".sfdl (v2 / v3)"
									}
							}
					}
			}
	}
}
