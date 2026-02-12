use dioxus::prelude::*;

use crate::state::{AppState, AppView};

#[component]
pub fn SettingsView() -> Element {
    let mut state = use_context::<AppState>();
    let settings = state.settings.read();
    let download_dir = settings.download_directory.display().to_string();
    let max_threads = settings.max_download_threads;
    let max_retries = settings.max_retries;
    let retry_wait = settings.retry_wait_seconds;
    let ftp_timeout = settings.ftp_timeout_seconds;
    let resume = settings.resume_downloads;
    let pkg_subfolder = settings.create_package_subfolder;
    let passwords = settings.auto_password_list.join("\n");
    let exclusion_patterns = settings.file_exclusion_patterns.join("\n");

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
                            "Max Download Threads"
                        }
                        input {
                            class: "w-24 px-3 py-2 border rounded text-sm",
                            r#type: "number",
                            min: "1",
                            max: "10",
                            value: "{max_threads}",
                            onchange: move |e| {
                                if let Ok(n) = e.value().parse::<u32>() {
                                    state.settings.write().max_download_threads = n.clamp(1, 10);
                                }
                            },
                        }
                    }

                    // Max retries
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            "Max Retries"
                        }
                        input {
                            class: "w-24 px-3 py-2 border rounded text-sm",
                            r#type: "number",
                            min: "0",
                            max: "10",
                            value: "{max_retries}",
                            onchange: move |e| {
                                if let Ok(n) = e.value().parse::<u32>() {
                                    state.settings.write().max_retries = n.clamp(0, 10);
                                }
                            },
                        }
                    }

                    // Retry wait
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            "Retry Wait (seconds)"
                        }
                        input {
                            class: "w-24 px-3 py-2 border rounded text-sm",
                            r#type: "number",
                            min: "1",
                            max: "120",
                            value: "{retry_wait}",
                            onchange: move |e| {
                                if let Ok(n) = e.value().parse::<u32>() {
                                    state.settings.write().retry_wait_seconds = n.clamp(1, 120);
                                }
                            },
                        }
                    }

                    // FTP timeout
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            "FTP Timeout (seconds)"
                        }
                        input {
                            class: "w-24 px-3 py-2 border rounded text-sm",
                            r#type: "number",
                            min: "5",
                            max: "300",
                            value: "{ftp_timeout}",
                            onchange: move |e| {
                                if let Ok(n) = e.value().parse::<u32>() {
                                    state.settings.write().ftp_timeout_seconds = n.clamp(5, 300);
                                }
                            },
                        }
                    }
                }

                // Toggle settings
                div { class: "space-y-3",
                    // Resume downloads
                    label { class: "flex items-center gap-3 cursor-pointer",
                        input {
                            r#type: "checkbox",
                            class: "w-4 h-4",
                            checked: resume,
                            onchange: move |_| {
                                let mut s = state.settings.write();
                                s.resume_downloads = !s.resume_downloads;
                            },
                        }
                        div {
                            span { class: "block text-sm font-medium text-gray-700", "Resume Downloads" }
                            span { class: "block text-xs text-gray-500", "Skip files that are already fully downloaded" }
                        }
                    }

                    // Create package subfolder
                    label { class: "flex items-center gap-3 cursor-pointer",
                        input {
                            r#type: "checkbox",
                            class: "w-4 h-4",
                            checked: pkg_subfolder,
                            onchange: move |_| {
                                let mut s = state.settings.write();
                                s.create_package_subfolder = !s.create_package_subfolder;
                            },
                        }
                        div {
                            span { class: "block text-sm font-medium text-gray-700", "Create Package Subfolder" }
                            span { class: "block text-xs text-gray-500", "Create a subfolder per package in the download directory" }
                        }
                    }
                }

                // Auto-password list
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
                                state.settings.write().auto_password_list = list;
                            },
                        }
                }

                // File exclusion patterns
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
                            state.settings.write().file_exclusion_patterns = list;
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

    if let Err(e) = rsfdl_core::settings::save_settings(&path, &settings) {
        state
            .error_message
            .set(Some(format!("Failed to save settings: {e}")));
    }
}
