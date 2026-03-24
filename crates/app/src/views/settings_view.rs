use dioxus::prelude::*;

use crate::icons;
use crate::state::{AppState, AppView};

#[component]
pub fn SettingsView() -> Element {
	let mut state = use_context::<AppState>();
	let settings = state.settings.read();
	let download_dir = settings.download_directory.display().to_string();
	let max_threads = settings.max_threads;
	let max_speed = settings.max_speed_kbps;
	let max_retries = settings.max_retries;
	let retry_delay = settings.retry_delay_seconds;
	let auto_extract = settings.auto_extract;
	let delete_after_extract = settings.delete_archives_after_extract;
	let strict_disk = settings.strict_disk_check;
	let exclusion_patterns = settings.exclusion_patterns.clone();
	let auto_passwords = settings.auto_passwords.clone();
	drop(settings);

	rsx! {
			div { class: "flex-1 overflow-y-auto",
					div { class: "max-w-[900px] mx-auto px-4 py-5",

							// Header
							div { class: "flex items-center gap-2.5 mb-5",
									button {
											class: "btn-icon",
											onclick: move |_| { state.current_view.set(AppView::Main); },
											span {
													style: "width: 18px; height: 18px;",
													dangerous_inner_html: icons::ARROW_LEFT,
											}
									}
									span {
											class: "text-lg font-semibold",
											style: "color: var(--color-text-primary);",
											"Einstellungen"
									}
							}

							// Allgemein
							div { class: "settings-card",
									div { class: "sg-title", "Allgemein" }
									SettingRow { label: "Download-Verzeichnis", sub: "Zielordner fuer Downloads" }
									div { class: "flex gap-2 mb-2.5 mt-[-4px]",
											input {
													class: "themed-input flex-1",
													r#type: "text",
													readonly: true,
													value: "{download_dir}",
											}
											button {
													class: "btn btn-ghost btn-sm",
													onclick: move |_| {
															spawn(async move {
																	if let Some(folder) = rfd::AsyncFileDialog::new().pick_folder().await {
																			state.settings.write().download_directory = folder.path().to_path_buf();
																	}
															});
													},
													"Browse..."
											}
									}
									NumberRow { label: "Max. parallele Downloads", value: max_threads, min: 1, max: 20, on_change: move |v: u32| { state.settings.write().max_threads = v; } }
									NumberRow { label: "Max. Geschwindigkeit (KB/s)", sub: "0 = unbegrenzt", value: max_speed, min: 0, max: 999999, on_change: move |v: u32| { state.settings.write().max_speed_kbps = v; } }
							}

							// Download-Verhalten
							div { class: "settings-card",
									div { class: "sg-title", "Download-Verhalten" }
									NumberRow { label: "Max. Wiederholungen", value: max_retries, min: 0, max: 50, on_change: move |v: u32| { state.settings.write().max_retries = v; } }
									NumberRow { label: "Retry-Wartezeit (Sek.)", value: retry_delay, min: 1, max: 3600, on_change: move |v: u32| { state.settings.write().retry_delay_seconds = v; } }
									ToggleRow { label: "Strikte Speicherplatzpruefung", value: strict_disk, on_toggle: move |_| { let mut s = state.settings.write(); s.strict_disk_check = !s.strict_disk_check; } }
							}

							// Nachbearbeitung
							div { class: "settings-card",
									div { class: "sg-title", "Nachbearbeitung" }
									ToggleRow { label: "Auto-Extraktion", sub: "Archive nach Download entpacken", value: auto_extract, on_toggle: move |_| { let mut s = state.settings.write(); s.auto_extract = !s.auto_extract; } }
									ToggleRow { label: "Archive nach Extraktion loeschen", value: delete_after_extract, on_toggle: move |_| { let mut s = state.settings.write(); s.delete_archives_after_extract = !s.delete_archives_after_extract; } }
							}

							// Ausschlussmuster
							div { class: "settings-card",
									div { class: "sg-title", "Ausschlussmuster" }
									TagList {
											tags: exclusion_patterns,
											on_add: move |tag: String| {
													state.settings.write().exclusion_patterns.push(tag);
											},
											on_remove: move |idx: usize| {
													state.settings.write().exclusion_patterns.remove(idx);
											},
											placeholder: "Neues Muster hinzufuegen...",
											masked: false,
									}
							}

							// Auto-Passwoerter
							div { class: "settings-card",
									div { class: "sg-title", "Auto-Passwoerter" }
									TagList {
											tags: auto_passwords,
											on_add: move |tag: String| {
													state.settings.write().auto_passwords.push(tag);
											},
											on_remove: move |idx: usize| {
													state.settings.write().auto_passwords.remove(idx);
											},
											placeholder: "Neues Passwort...",
											masked: true,
									}
							}

							// Footer
							div { class: "flex justify-end gap-2 mt-5 pb-8",
									button {
											class: "btn btn-ghost",
											onclick: move |_| { state.current_view.set(AppView::Main); },
											"Abbrechen"
									}
									button {
											class: "btn btn-accent",
											onclick: move |_| {
													save_settings(state);
													state.current_view.set(AppView::Main);
											},
											"Speichern"
									}
							}
					}
			}
	}
}

#[component]
fn SettingRow(label: String, #[props(default)] sub: String) -> Element {
	rsx! {
			div {
					class: "flex items-center justify-between mb-2.5 gap-3",
					div {
							span {
									class: "text-[13px]",
									style: "color: var(--color-text-primary);",
									"{label}"
							}
							if !sub.is_empty() {
									div {
											class: "text-[11px] mt-0.5",
											style: "color: var(--color-text-tertiary);",
											"{sub}"
									}
							}
					}
			}
	}
}

#[component]
fn NumberRow(label: String, #[props(default)] sub: String, value: u32, min: u32, max: u32, on_change: EventHandler<u32>) -> Element {
	rsx! {
			div {
					class: "flex items-center justify-between mb-2.5 gap-3",
					div { class: "flex-1",
							span {
									class: "text-[13px]",
									style: "color: var(--color-text-primary);",
									"{label}"
							}
							if !sub.is_empty() {
									div {
											class: "text-[11px] mt-0.5",
											style: "color: var(--color-text-tertiary);",
											"{sub}"
									}
							}
					}
					input {
							class: "themed-input w-20 text-right",
							r#type: "number",
							min: "{min}",
							max: "{max}",
							value: "{value}",
							onchange: move |e| {
									if let Ok(n) = e.value().parse::<u32>() {
											on_change.call(n.clamp(min, max));
									}
							},
					}
			}
	}
}

#[component]
fn ToggleRow(label: String, #[props(default)] sub: String, value: bool, on_toggle: EventHandler<()>) -> Element {
	rsx! {
			div {
					class: "flex items-center justify-between mb-2.5 gap-3",
					div { class: "flex-1",
							span {
									class: "text-[13px]",
									style: "color: var(--color-text-primary);",
									"{label}"
							}
							if !sub.is_empty() {
									div {
											class: "text-[11px] mt-0.5",
											style: "color: var(--color-text-tertiary);",
											"{sub}"
									}
							}
					}
					button {
							class: if value { "toggle-switch on" } else { "toggle-switch" },
							onclick: move |_| { on_toggle.call(()); },
					}
			}
	}
}

#[component]
fn TagList(tags: Vec<String>, on_add: EventHandler<String>, on_remove: EventHandler<usize>, placeholder: String, masked: bool) -> Element {
	let mut input_value = use_signal(String::new);

	rsx! {
			// Tag chips
			div { class: "flex flex-wrap gap-1.5 mb-2",
					for (idx, tag) in tags.iter().enumerate() {
							span { class: "tag",
									if masked {
											"\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}"
									} else {
											"{tag}"
									}
									button {
											class: "tag-del",
											onclick: move |_| { on_remove.call(idx); },
											"\u{00D7}"
									}
							}
					}
			}
			// Add input
			div { class: "flex gap-2",
					input {
							class: "themed-input flex-1",
							r#type: if masked { "password" } else { "text" },
							placeholder: "{placeholder}",
							value: "{input_value}",
							oninput: move |e| input_value.set(e.value()),
							onkeydown: move |e| {
									if e.key() == Key::Enter {
											let v = input_value.read().trim().to_string();
											if !v.is_empty() {
													on_add.call(v);
													input_value.set(String::new());
											}
									}
							},
					}
					button {
							class: "btn btn-ghost btn-sm",
							onclick: move |_| {
									let v = input_value.read().trim().to_string();
									if !v.is_empty() {
											on_add.call(v);
											input_value.set(String::new());
									}
							},
							span {
									style: "width: 14px; height: 14px;",
									dangerous_inner_html: icons::PLUS,
							}
					}
			}
	}
}

fn save_settings(mut state: AppState) {
	let settings = state.settings.read().clone();
	let path = rsfdl_core::settings::config_path();

	if let Err(e) = rsfdl_core::settings::save(&path, &settings) {
		state.error_message.set(Some(format!("Failed to save settings: {e}")));
	}
}
