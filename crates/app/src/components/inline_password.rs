use dioxus::prelude::*;

use crate::components::header::try_decrypt_container;
use crate::icons;
use crate::state::{AppState, ContainerId};

#[component]
pub fn InlinePassword(container_id: ContainerId) -> Element {
	let state = use_context::<AppState>();
	let mut password = use_signal(String::new);
	let mut show_password = use_signal(|| false);

	let containers = state.containers.read();
	let Some(cs) = containers.iter().find(|c| c.id == container_id) else {
		return rsx! {};
	};
	let error = cs.password_error.clone();
	drop(containers);

	let cid = container_id;
	let input_type = if *show_password.read() { "text" } else { "password" };

	rsx! {
			div {
					class: "px-4 py-5 text-center",
					div { class: "max-w-[360px] mx-auto",
							// Key icon
							div {
									class: "w-12 h-12 mx-auto mb-3 rounded-full flex items-center justify-center",
									style: "background: var(--color-warning-bg); color: var(--color-warning);",
									span {
											style: "width: 22px; height: 22px;",
											dangerous_inner_html: icons::KEY_ROUND,
									}
							}

							// Title
							p {
									class: "text-[15px] font-medium mb-1",
									style: "color: var(--color-text-primary);",
									"Passwort erforderlich"
							}
							p {
									class: "text-[13px] mb-4",
									style: "color: var(--color-text-secondary);",
									"Dieser Container ist verschluesselt. Bitte Passwort eingeben."
							}

							// Input + button row
							div { class: "flex gap-2 mb-2",
									div { class: "flex-1 relative",
											input {
													class: "themed-input w-full",
													style: "padding-right: 36px;",
													r#type: input_type,
													placeholder: "Passwort",
													value: "{password}",
													oninput: move |e| password.set(e.value()),
													onkeydown: move |e| {
															if e.key() == Key::Enter {
																	let pw = password.read().clone();
																	try_decrypt_container(state, cid, &pw);
															}
													},
											}
											button {
													class: "btn-icon",
													style: "position: absolute; right: 4px; top: 50%; transform: translateY(-50%); padding: 4px;",
													onclick: move |_| {
															let current = *show_password.read();
															show_password.set(!current);
													},
													span {
															style: "width: 14px; height: 14px;",
															dangerous_inner_html: if *show_password.read() { icons::EYE_OFF } else { icons::EYE },
													}
											}
									}
									button {
											class: "btn btn-accent",
											onclick: move |_| {
													let pw = password.read().clone();
													try_decrypt_container(state, cid, &pw);
											},
											span {
													style: "width: 14px; height: 14px;",
													dangerous_inner_html: icons::UNLOCK,
											}
											"Entschluesseln"
									}
							}

							// Error message
							if let Some(err) = &error {
									p {
											class: "text-xs mt-1",
											style: "color: var(--color-error);",
											"{err}"
									}
							}
					}
			}
	}
}
