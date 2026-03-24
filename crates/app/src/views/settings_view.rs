use dioxus::prelude::*;

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
	let passwords = settings.auto_passwords.join("\n");
	let exclusion_patterns = settings.exclusion_patterns.join("\n");

	rsx! {
			div { class: "flex flex-col flex-1 overflow-hidden",
					// Header
					div { class: "flex items-center justify-between px-4 py-3 bg-gray-100 border-b",
							h2 { class: "text-lg font-bold text-gray-900", "Settings" }
							div { class: "flex items-center gap-2",
									button {
											class: "px-4 py-1.5 bg-green-600 hover:bg-green-700 text-white rounded text-sm font-medium",
											onclick: move |_| {
													save_settings_to_file(state);
											},
											"Save"
									}
									button {
											class: "px-3 py-1.5 bg-red-100 hover:bg-red-200 text-red-700 rounded text-sm",
											onclick: move |_| {
													reset_settings(state);
											},
											"Reset"
									}
									button {
											class: "px-3 py-1.5 bg-gray-200 hover:bg-gray-300 rounded text-sm",
											onclick: move |_| {
													state.current_view.set(AppView::Main);
											},
											"Back"
									}
							}
					}

					div { class: "flex-1 overflow-y-auto p-6 space-y-6",
							// Download directory
							div {
									label { class: "block text-sm font-medium text-gray-700 mb-1",
											"Download Directory"
									}
									div { class: "flex gap-2",
											input {
													class: "flex-1 px-3 py-2 border rounded text-sm bg-gray-50",
													r#type: "text",
													readonly: true,
													value: "{download_dir}",
											}
											button {
													class: "px-3 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded text-sm",
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
							}

							// Number settings grid
							div { class: "grid grid-cols-2 gap-4",
									// Max threads
									div {
											label { class: "block text-sm font-medium text-gray-700 mb-1",
													"Max Download Threads (1-20)"
											}
											input {
													class: "w-24 px-3 py-2 border rounded text-sm",
													r#type: "number",
													min: "1",
													max: "20",
													value: "{max_threads}",
													onchange: move |e| {
															if let Ok(n) = e.value().parse::<u32>() {
																	state.settings.write().max_threads = n.clamp(1, 20);
															}
													},
											}
									}

									// Max speed
									div {
											label { class: "block text-sm font-medium text-gray-700 mb-1",
													"Max Speed (KB/s, 0 = unlimited)"
											}
											input {
													class: "w-24 px-3 py-2 border rounded text-sm",
													r#type: "number",
													min: "0",
													value: "{max_speed}",
													onchange: move |e| {
															if let Ok(n) = e.value().parse::<u32>() {
																	state.settings.write().max_speed_kbps = n;
															}
													},
											}
									}

									// Max retries
									div {
											label { class: "block text-sm font-medium text-gray-700 mb-1",
													"Max Retries (0-50)"
											}
											input {
													class: "w-24 px-3 py-2 border rounded text-sm",
													r#type: "number",
													min: "0",
													max: "50",
													value: "{max_retries}",
													onchange: move |e| {
															if let Ok(n) = e.value().parse::<u32>() {
																	state.settings.write().max_retries = n.clamp(0, 50);
															}
													},
											}
									}

									// Retry delay
									div {
											label { class: "block text-sm font-medium text-gray-700 mb-1",
													"Retry Delay (seconds, 1-3600)"
											}
											input {
													class: "w-24 px-3 py-2 border rounded text-sm",
													r#type: "number",
													min: "1",
													max: "3600",
													value: "{retry_delay}",
													onchange: move |e| {
															if let Ok(n) = e.value().parse::<u32>() {
																	state.settings.write().retry_delay_seconds = n.clamp(1, 3600);
															}
													},
											}
									}
							}

							// Toggle settings
							div { class: "space-y-3",
									// Auto extract
									label { class: "flex items-center gap-3 cursor-pointer",
											input {
													r#type: "checkbox",
													class: "w-4 h-4",
													checked: auto_extract,
													onchange: move |_| {
															let mut s = state.settings.write();
															s.auto_extract = !s.auto_extract;
													},
											}
											div {
													span { class: "block text-sm font-medium text-gray-700", "Auto Extract Archives" }
													span { class: "block text-xs text-gray-500", "Automatically extract archives after download" }
											}
									}

									// Delete after extract
									label { class: "flex items-center gap-3 cursor-pointer",
											input {
													r#type: "checkbox",
													class: "w-4 h-4",
													checked: delete_after_extract,
													disabled: !auto_extract,
													onchange: move |_| {
															let mut s = state.settings.write();
															s.delete_archives_after_extract = !s.delete_archives_after_extract;
													},
											}
											div {
													span { class: "block text-sm font-medium text-gray-700", "Delete Archives After Extraction" }
													span { class: "block text-xs text-gray-500", "Remove archive files after successful extraction" }
											}
									}

									// Strict disk check
									label { class: "flex items-center gap-3 cursor-pointer",
											input {
													r#type: "checkbox",
													class: "w-4 h-4",
													checked: strict_disk,
													onchange: move |_| {
															let mut s = state.settings.write();
															s.strict_disk_check = !s.strict_disk_check;
													},
											}
											div {
													span { class: "block text-sm font-medium text-gray-700", "Strict Disk Check" }
													span { class: "block text-xs text-gray-500", "Abort download if insufficient disk space" }
											}
									}
							}

							// Exclusion patterns
							div {
									label { class: "block text-sm font-medium text-gray-700 mb-1",
											"File Exclusion Patterns"
									}
									p { class: "text-xs text-gray-500 mb-2",
											"One glob pattern per line. Matching files are excluded from download."
									}
									textarea {
											class: "w-full px-3 py-2 border rounded text-sm font-mono h-24 resize-y",
											placeholder: "*.nfo\n*.jpg\n*sample*",
											value: "{exclusion_patterns}",
											onchange: move |e| {
													let list: Vec<String> = e.value()
															.lines()
															.map(|l| l.trim().to_string())
															.filter(|l| !l.is_empty())
															.collect();
													state.settings.write().exclusion_patterns = list;
											},
									}
							}

							// Password list
							div {
									label { class: "block text-sm font-medium text-gray-700 mb-1",
											"Auto Password List"
									}
									p { class: "text-xs text-gray-500 mb-2",
											"One password per line. Tried automatically when opening encrypted containers."
									}
									textarea {
											class: "w-full px-3 py-2 border rounded text-sm font-mono h-24 resize-y",
											value: "{passwords}",
											onchange: move |e| {
													let list: Vec<String> = e.value()
															.lines()
															.map(|l| l.trim().to_string())
															.filter(|l| !l.is_empty())
															.collect();
													state.settings.write().auto_passwords = list;
											},
									}
							}

					}
			}
	}
}

fn save_settings_to_file(mut state: AppState) {
	let settings = state.settings.read().clone();
	let path = rsfdl_core::settings::default_settings_path();

	if let Err(e) = rsfdl_core::settings::save(&path, &settings) {
		state.error_message.set(Some(format!("Failed to save settings: {e}")));
	}
}

fn reset_settings(mut state: AppState) {
	let path = rsfdl_core::settings::default_settings_path();
	match rsfdl_core::settings::reset(&path) {
		Ok(defaults) => {
			state.settings.set(defaults);
		}
		Err(e) => {
			state.error_message.set(Some(format!("Failed to reset settings: {e}")));
		}
	}
}
