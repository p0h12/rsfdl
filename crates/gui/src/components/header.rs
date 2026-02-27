use dioxus::prelude::*;

use rsfdl_core::ftp::listing::resolve_container_bulk_folders;
use rsfdl_core::sfdl::crypto::{decrypt_container, try_passwords};
use rsfdl_core::sfdl::parser::parse_sfdl;

use crate::state::{AppState, AppView, DownloadPhase};

#[component]
pub fn Header() -> Element {
    let mut state = use_context::<AppState>();
    let downloading = *state.download_phase.read() == DownloadPhase::Downloading;

    rsx! {
        div { class: "flex items-center justify-between px-4 py-3 bg-gray-800 text-white",
            h1 { class: "text-lg font-bold", "rsfdl" }
            div { class: "flex gap-2",
                button {
                    class: "px-3 py-1.5 bg-blue-600 hover:bg-blue-700 rounded text-sm font-medium disabled:opacity-50 disabled:cursor-not-allowed",
                    disabled: downloading,
                    onclick: move |_| {
                        spawn(async move {
                            open_sfdl_file(state).await;
                        });
                    },
                    "Open File"
                }
                button {
                    class: "px-3 py-1.5 bg-green-600 hover:bg-green-700 rounded text-sm font-medium",
                    onclick: move |_| {
                        state.current_view.set(AppView::Creator);
                    },
                    "Create"
                }
                button {
                    class: "px-3 py-1.5 bg-gray-600 hover:bg-gray-700 rounded text-sm",
                    onclick: move |_| {
                        state.current_view.set(AppView::Settings);
                    },
                    "Settings"
                }
            }
        }
    }
}

async fn open_sfdl_file(mut state: AppState) {
    let file = rfd::AsyncFileDialog::new()
        .add_filter("SFDL Files", &["sfdl"])
        .pick_file()
        .await;

    let Some(file) = file else { return };

    let path = file.path().to_string_lossy().to_string();
    let data = match tokio::fs::read_to_string(file.path()).await {
        Ok(s) => s,
        Err(e) => {
            state
                .error_message
                .set(Some(format!("Cannot read file: {e}")));
            return;
        }
    };

    match parse_sfdl(&data) {
        Ok(mut container) => {
            if container.encrypted {
                // Try auto-password list first
                let passwords = state.settings.read().auto_password_list.clone();
                if let Some(pw) = try_passwords(&container, &passwords) {
                    if let Err(e) = decrypt_container(&mut container, &pw) {
                        state
                            .error_message
                            .set(Some(format!("Auto-decrypt failed: {e}")));
                        return;
                    }
                    // Auto-decrypt succeeded
                    finish_container_load(&mut state, container, path);
                    return;
                }
                // No auto-password matched — show dialog
                state.container.set(Some(container));
                state.container_path.set(Some(path));
                state.reset_for_new_container();
                state.needs_password.set(true);
            } else {
                finish_container_load(&mut state, container, path);
            }
        }
        Err(e) => {
            state.error_message.set(Some(format!("Parse error: {e}")));
        }
    }
}

/// Finish loading a (decrypted) container: set state and trigger BulkFolder resolution.
pub fn finish_container_load(
    state: &mut AppState,
    container: rsfdl_core::sfdl::models::SfdlContainer,
    path: String,
) {
    let patterns = state.settings.read().file_exclusion_patterns.clone();
    let selection: Vec<bool> = container
        .packages
        .iter()
        .flat_map(|p| &p.file_list)
        .map(|f| !rsfdl_core::filter::is_excluded(&f.file_name, &patterns))
        .collect();

    state.container.set(Some(container));
    state.container_path.set(Some(path));
    state.selected_files.set(selection);
    state.reset_for_new_container();

    // Trigger async BulkFolder resolution
    let mut state_copy = *state;
    spawn(async move {
        resolve_bulk_folders_async(&mut state_copy).await;
    });
}

/// Resolves BulkFolders via FTP and adds the resulting files to the container.
async fn resolve_bulk_folders_async(state: &mut AppState) {
    let (conn, packages, timeout) = {
        let container_guard = state.container.read();
        let Some(container) = container_guard.as_ref() else {
            return;
        };
        let has_bulk = container
            .packages
            .iter()
            .any(|p| p.bulk_folder_mode && !p.bulk_folder_list.is_empty());
        if !has_bulk {
            return;
        }
        let timeout = state.settings.read().ftp_timeout_seconds;
        (
            container.connection.clone(),
            container.packages.clone(),
            timeout,
        )
    };

    state.resolving_bulk_folders.set(true);

    match resolve_container_bulk_folders(&conn, &packages, timeout).await {
        Ok(resolved_files) => {
            if !resolved_files.is_empty() {
                // Group resolved files by package_name
                let mut files_by_package: std::collections::HashMap<
                    String,
                    Vec<rsfdl_core::sfdl::models::FileItem>,
                > = std::collections::HashMap::new();
                for file in resolved_files {
                    files_by_package
                        .entry(file.package_name.clone())
                        .or_default()
                        .push(file);
                }

                // Add resolved files to their respective packages
                let mut container = { state.container.read().clone() };
                if let Some(container) = container.as_mut() {
                    for pkg in &mut container.packages {
                        if let Some(new_files) = files_by_package.remove(&pkg.name) {
                            pkg.file_list.extend(new_files);
                        }
                        // Clear bulk folders since they've been resolved
                        pkg.bulk_folder_list.clear();
                        pkg.bulk_folder_mode = false;
                    }

                    let patterns = state.settings.read().file_exclusion_patterns.clone();
                    let selection: Vec<bool> = container
                        .packages
                        .iter()
                        .flat_map(|p| &p.file_list)
                        .map(|f| !rsfdl_core::filter::is_excluded(&f.file_name, &patterns))
                        .collect();

                    state.container.set(Some(container.clone()));
                    state.selected_files.set(selection);
                }
            }
        }
        Err(e) => {
            state
                .error_message
                .set(Some(format!("Failed to resolve bulk folders: {e}")));
        }
    }

    state.resolving_bulk_folders.set(false);
}
