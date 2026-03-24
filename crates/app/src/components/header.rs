use dioxus::prelude::*;

use rsfdl_core::container::{compute_file_selection, load_sfdl, DecryptionStatus};

use crate::icons;
use crate::state::{AppState, AppView, ContainerPhase, Theme};

#[component]
pub fn Header() -> Element {
	let mut state = use_context::<AppState>();
	let theme = *state.theme.read();

	rsx! {
			div {
					class: "flex items-center justify-between px-4",
					style: "background: var(--color-background-primary); border-bottom: 0.5px solid var(--color-border-tertiary); height: 52px;",

					// Logo
					div { class: "flex items-center gap-2.5",
							div {
									class: "w-8 h-8 rounded-lg flex items-center justify-center text-white",
									style: "background: var(--color-accent);",
									span {
											style: "width: 18px; height: 18px;",
											dangerous_inner_html: icons::DOWNLOAD,
									}
							}
							span {
									class: "font-semibold text-base",
									style: "color: var(--color-text-primary);",
									"rsfdl"
							}
					}

					// Actions
					div { class: "flex items-center gap-1.5",
							// Open File button
							button {
									class: "btn btn-ghost btn-sm",
									onclick: move |_| {
											spawn(async move {
													open_sfdl_files_from_dialog(state).await;
											});
									},
									span {
											style: "width: 14px; height: 14px;",
											dangerous_inner_html: icons::FOLDER_OPEN,
									}
									"Open"
							}

							// Settings button
							button {
									class: "btn-icon",
									onclick: move |_| {
											state.current_view.set(AppView::Settings);
									},
									span {
											style: "width: 18px; height: 18px;",
											dangerous_inner_html: icons::SETTINGS,
									}
							}

							// Theme toggle
							ThemeToggle { theme }
					}
			}
	}
}

#[component]
fn ThemeToggle(theme: Theme) -> Element {
	let mut state = use_context::<AppState>();

	rsx! {
			div { class: "theme-toggle",
					button {
							class: if theme == Theme::Light { "active" } else { "" },
							onclick: move |_| state.theme.set(Theme::Light),
							span {
									style: "width: 14px; height: 14px;",
									dangerous_inner_html: icons::SUN,
							}
					}
					button {
							class: if theme == Theme::System { "active" } else { "" },
							onclick: move |_| state.theme.set(Theme::System),
							span {
									style: "width: 14px; height: 14px;",
									dangerous_inner_html: icons::MONITOR,
							}
					}
					button {
							class: if theme == Theme::Dark { "active" } else { "" },
							onclick: move |_| state.theme.set(Theme::Dark),
							span {
									style: "width: 14px; height: 14px;",
									dangerous_inner_html: icons::MOON,
							}
					}
			}
	}
}

/// Open one or more SFDL files via native dialog and add them to the container list.
pub async fn open_sfdl_files_from_dialog(mut state: AppState) {
	let files = rfd::AsyncFileDialog::new().add_filter("SFDL Files", &["sfdl"]).pick_files().await;

	let Some(files) = files else { return };

	let auto_passwords = state.settings.read().auto_passwords.clone();

	for file in files {
		let path = file.path().to_string_lossy().to_string();
		let data = match tokio::fs::read_to_string(file.path()).await {
			Ok(s) => s,
			Err(e) => {
				state.error_message.set(Some(format!("Cannot read file: {e}")));
				continue;
			}
		};

		match load_sfdl(&data, &auto_passwords) {
			Ok(loaded) => match loaded.status {
				DecryptionStatus::NotEncrypted | DecryptionStatus::AutoDecrypted { .. } => {
					let container_id = state.add_container(path, loaded.container, ContainerPhase::Ready);
					spawn(async move {
						resolve_bulk_folders_for(state, container_id).await;
					});
				}
				DecryptionStatus::NeedsPassword => {
					state.add_container(path, loaded.container, ContainerPhase::NeedsPassword);
				}
			},
			Err(e) => {
				state.error_message.set(Some(format!("Parse error: {e}")));
			}
		}
	}
}

/// SFDL-003: Resolve BulkFolders via FTP for a specific container.
pub async fn resolve_bulk_folders_for(mut state: AppState, container_id: u32) {
	let timeout = state.settings.read().ftp_timeout_seconds;

	let mut container = {
		let list = state.containers.read();
		let Some(cs) = list.iter().find(|c| c.id == container_id) else {
			return;
		};
		let has_bulk = cs.container.packages.iter().any(|p| p.bulk_folder_mode && !p.bulk_folder_list.is_empty());
		if !has_bulk {
			return;
		}
		cs.container.clone()
	};

	state.with_container_mut(container_id, |cs| {
		cs.phase = ContainerPhase::ResolvingBulk;
	});

	match rsfdl_core::container::resolve_bulk_folders(&mut container, timeout).await {
		Ok(()) => {
			let patterns = state.settings.read().exclusion_patterns.clone();
			let selection = compute_file_selection(&container, &patterns);
			state.with_container_mut(container_id, |cs| {
				cs.container = container;
				cs.selected_files = selection;
				cs.phase = ContainerPhase::Ready;
			});
		}
		Err(e) => {
			state.error_message.set(Some(format!("Failed to resolve bulk folders: {e}")));
			state.with_container_mut(container_id, |cs| {
				cs.phase = ContainerPhase::Ready;
			});
		}
	}
}

/// Decrypt a container with the given password and update its state.
pub fn try_decrypt_container(mut state: AppState, container_id: u32, password: &str) {
	let mut container = {
		let list = state.containers.read();
		let Some(cs) = list.iter().find(|c| c.id == container_id) else {
			return;
		};
		cs.container.clone()
	};

	match rsfdl_core::container::decrypt_with_password(&mut container, password) {
		Ok(()) => {
			let patterns = state.settings.read().exclusion_patterns.clone();
			let selection = compute_file_selection(&container, &patterns);
			state.with_container_mut(container_id, |cs| {
				cs.container = container;
				cs.selected_files = selection;
				cs.phase = ContainerPhase::Ready;
				cs.password_error = None;
			});
			spawn(async move {
				resolve_bulk_folders_for(state, container_id).await;
			});
		}
		Err(rsfdl_core::error::AppError::InvalidPassword) => {
			state.with_container_mut(container_id, |cs| {
				cs.password_error = Some("Invalid password".to_string());
			});
		}
		Err(e) => {
			state.with_container_mut(container_id, |cs| {
				cs.password_error = Some(format!("Decryption failed: {e}"));
			});
		}
	}
}
