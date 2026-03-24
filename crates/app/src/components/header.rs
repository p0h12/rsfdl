use dioxus::prelude::*;

use rsfdl_core::container::{DecryptionStatus, load_sfdl};
use rsfdl_core::selection::FileSelection;

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

/// BR-UI-017: Check if a filename has the `.sfdl` extension (case-insensitive).
pub fn is_sfdl_file(name: &str) -> bool {
	name.contains('.') && name.rsplit('.').next().is_some_and(|ext| ext.eq_ignore_ascii_case("sfdl"))
}

/// Open one or more SFDL files via native dialog and add them to the container list.
pub async fn open_sfdl_files_from_dialog(mut state: AppState) {
	let files = rfd::AsyncFileDialog::new().add_filter("SFDL Files", &["sfdl"]).pick_files().await;

	let Some(files) = files else { return };

	for file in files {
		let path = file.path().to_string_lossy().to_string();
		let data = match tokio::fs::read_to_string(file.path()).await {
			Ok(s) => s,
			Err(e) => {
				state.error_message.set(Some(format!("Cannot read file: {e}")));
				continue;
			}
		};

		process_sfdl_content(state, &path, &data);
	}
}

/// UI-006: Process SFDL file content and add to container list.
///
/// Shared by dialog-based open (UI-001) and drag-and-drop (UI-006).
pub fn process_sfdl_content(mut state: AppState, file_path: &str, xml_content: &str) {
	let auto_passwords = state.settings.read().auto_passwords.clone();

	match load_sfdl(xml_content, &auto_passwords) {
		Ok(loaded) => match loaded.status {
			DecryptionStatus::NotEncrypted | DecryptionStatus::AutoDecrypted { .. } => {
				let container_id = state.add_container(file_path.to_string(), loaded.container, ContainerPhase::Ready);
				spawn(async move {
					resolve_bulk_folders_for(state, container_id).await;
				});
			}
			DecryptionStatus::NeedsPassword => {
				state.add_container(file_path.to_string(), loaded.container, ContainerPhase::NeedsPassword);
			}
		},
		Err(e) => {
			state.error_message.set(Some(format!("Parse error: {e}")));
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

	let warnings = rsfdl_core::container::resolve_bulk_folders(&mut container, timeout).await;

	if !warnings.is_empty() {
		state.error_message.set(Some(warnings.join("; ")));
	}

	let patterns = state.settings.read().exclusion_patterns.clone();
	let selection = FileSelection::new(&container, &patterns);
	state.with_container_mut(container_id, |cs| {
		cs.container = container;
		cs.selection = selection;
		cs.phase = ContainerPhase::Ready;
	});
}

/// UI-002: Result of a password decrypt attempt.
#[derive(Debug)]
pub enum DecryptOutcome {
	/// Main Success: container decrypted, selection computed.
	Success {
		container: rsfdl_core::sfdl::models::SfdlContainer,
		selection: FileSelection,
	},
	/// A1: Invalid password.
	InvalidPassword,
	/// A2: Other decryption error.
	OtherError(String),
}

/// UI-002: Attempt to decrypt a container with the given password (pure logic).
///
/// Returns a DecryptOutcome without touching UI state, for testability.
pub fn attempt_decrypt(container: &rsfdl_core::sfdl::models::SfdlContainer, password: &str, exclusion_patterns: &[String]) -> DecryptOutcome {
	let mut c = container.clone();
	match rsfdl_core::container::decrypt_with_password(&mut c, password) {
		Ok(()) => {
			let selection = FileSelection::new(&c, exclusion_patterns);
			DecryptOutcome::Success { container: c, selection }
		}
		Err(rsfdl_core::error::AppError::InvalidPassword) => DecryptOutcome::InvalidPassword,
		Err(e) => DecryptOutcome::OtherError(format!("Decryption failed: {e}")),
	}
}

/// UI-002: Decrypt a container with the given password and update app state.
///
/// Delegates to [`attempt_decrypt`] for the core logic, then applies the
/// outcome to the Dioxus state. On success, triggers BulkFolder resolution.
pub fn try_decrypt_container(mut state: AppState, container_id: u32, password: &str) {
	let container = {
		let list = state.containers.read();
		let Some(cs) = list.iter().find(|c| c.id == container_id) else {
			return;
		};
		cs.container.clone()
	};

	let patterns = state.settings.read().exclusion_patterns.clone();
	match attempt_decrypt(&container, password, &patterns) {
		DecryptOutcome::Success { container: decrypted, selection } => {
			state.with_container_mut(container_id, |cs| {
				cs.container = decrypted;
				cs.selection = selection;
				cs.phase = ContainerPhase::Ready;
				cs.password_error = None;
			});
			spawn(async move {
				resolve_bulk_folders_for(state, container_id).await;
			});
		}
		DecryptOutcome::InvalidPassword => {
			state.with_container_mut(container_id, |cs| {
				cs.password_error = Some("Invalid password".to_string());
			});
		}
		DecryptOutcome::OtherError(msg) => {
			state.with_container_mut(container_id, |cs| {
				cs.password_error = Some(msg);
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use rsfdl_core::sfdl::models::{Connection, SfdlContainer};

	fn encrypted_container() -> SfdlContainer {
		SfdlContainer {
			encrypted: true,
			connection: Connection {
				host: "FA9p93TaRSx1Bap096qqevmwi8vGbaEXtXRbnLmbUr8=".into(),
				..Connection::default()
			},
			..SfdlContainer::default()
		}
	}

	/// UI-002 | Main Success: Correct password decrypts container.
	#[test]
	fn ui002_correct_password_decrypts() {
		let c = encrypted_container();
		let result = attempt_decrypt(&c, "test", &[]);
		assert!(matches!(result, DecryptOutcome::Success { .. }));
		if let DecryptOutcome::Success { container, .. } = result {
			assert!(!container.encrypted);
			assert_eq!(container.connection.host, "ftp.example.com");
		}
	}

	/// UI-002 | A1: Wrong password returns InvalidPassword.
	#[test]
	fn ui002_wrong_password() {
		let c = encrypted_container();
		let result = attempt_decrypt(&c, "wrong", &[]);
		assert!(matches!(result, DecryptOutcome::InvalidPassword));
	}

	/// UI-002 | Main Success: Decryption computes file selection.
	#[test]
	fn ui002_decrypt_computes_selection() {
		let c = encrypted_container();
		let result = attempt_decrypt(&c, "test", &[]);
		if let DecryptOutcome::Success { selection, .. } = result {
			// Default container has no files, so selection is empty
			assert_eq!(selection.total_count(), 0);
		} else {
			panic!("expected Success");
		}
	}

	/// UI-002 | A1: Wrong password preserves original container state.
	#[test]
	fn ui002_wrong_password_preserves_container() {
		let c = encrypted_container();
		let _ = attempt_decrypt(&c, "wrong", &[]);
		// Original container unchanged (it was passed by reference)
		assert!(c.encrypted);
		assert_eq!(c.connection.host, "FA9p93TaRSx1Bap096qqevmwi8vGbaEXtXRbnLmbUr8=");
	}

	// -------------------------------------------------------
	// UI-006 | BR-UI-017: SFDL file extension check
	// -------------------------------------------------------

	/// UI-006 | BR-UI-017: .sfdl extension accepted.
	#[test]
	fn ui006_sfdl_extension_accepted() {
		assert!(is_sfdl_file("test.sfdl"));
		assert!(is_sfdl_file("My.Release.2026.sfdl"));
	}

	/// UI-006 | BR-UI-017: Case-insensitive extension check.
	#[test]
	fn ui006_sfdl_extension_case_insensitive() {
		assert!(is_sfdl_file("test.SFDL"));
		assert!(is_sfdl_file("test.Sfdl"));
		assert!(is_sfdl_file("test.sFdL"));
	}

	/// UI-006 | BR-UI-017: Non-sfdl extensions rejected.
	#[test]
	fn ui006_non_sfdl_rejected() {
		assert!(!is_sfdl_file("test.txt"));
		assert!(!is_sfdl_file("test.xml"));
		assert!(!is_sfdl_file("test.rar"));
		assert!(!is_sfdl_file("sfdl")); // no extension
	}

	/// UI-006 | BR-UI-017: Path with directories works.
	#[test]
	fn ui006_sfdl_with_path() {
		assert!(is_sfdl_file("/home/user/downloads/release.sfdl"));
		assert!(is_sfdl_file("C:\\Users\\test\\file.sfdl"));
	}

	/// UI-006 | BR-UI-017: Empty string rejected.
	#[test]
	fn ui006_empty_string_rejected() {
		assert!(!is_sfdl_file(""));
		assert!(!is_sfdl_file("."));
	}
}
