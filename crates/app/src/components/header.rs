use dioxus::prelude::*;

use rsfdl_core::container::{DecryptionStatus, compute_file_selection, load_sfdl, resolve_bulk_folders};

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
	let file = rfd::AsyncFileDialog::new().add_filter("SFDL Files", &["sfdl"]).pick_file().await;

	let Some(file) = file else { return };

	let path = file.path().to_string_lossy().to_string();
	let data = match tokio::fs::read_to_string(file.path()).await {
		Ok(s) => s,
		Err(e) => {
			state.error_message.set(Some(format!("Cannot read file: {e}")));
			return;
		}
	};

	let auto_passwords = state.settings.read().auto_password_list.clone();

	match load_sfdl(&data, &auto_passwords) {
		Ok(loaded) => match loaded.status {
			DecryptionStatus::NotEncrypted | DecryptionStatus::AutoDecrypted { .. } => {
				finish_container_load(&mut state, loaded.container, path);
			}
			DecryptionStatus::NeedsPassword => {
				state.container.set(Some(loaded.container));
				state.container_path.set(Some(path));
				state.reset_for_new_container();
				state.needs_password.set(true);
			}
		},
		Err(e) => {
			state.error_message.set(Some(format!("Parse error: {e}")));
		}
	}
}

/// Finish loading a (decrypted) container: set state and trigger BulkFolder resolution.
pub fn finish_container_load(state: &mut AppState, container: rsfdl_core::sfdl::models::SfdlContainer, path: String) {
	let patterns = state.settings.read().file_exclusion_patterns.clone();
	let selection = compute_file_selection(&container, &patterns);

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

/// UC-SFDL-003: Resolve BulkFolders via FTP and update container state.
async fn resolve_bulk_folders_async(state: &mut AppState) {
	let timeout = state.settings.read().ftp_timeout_seconds;

	let mut container = {
		let guard = state.container.read();
		let Some(c) = guard.as_ref() else { return };
		let has_bulk = c.packages.iter().any(|p| p.bulk_folder_mode && !p.bulk_folder_list.is_empty());
		if !has_bulk {
			return;
		}
		c.clone()
	};

	state.resolving_bulk_folders.set(true);

	match resolve_bulk_folders(&mut container, timeout).await {
		Ok(()) => {
			let patterns = state.settings.read().file_exclusion_patterns.clone();
			let selection = compute_file_selection(&container, &patterns);
			state.container.set(Some(container));
			state.selected_files.set(selection);
		}
		Err(e) => {
			state.error_message.set(Some(format!("Failed to resolve bulk folders: {e}")));
		}
	}

	state.resolving_bulk_folders.set(false);
}
