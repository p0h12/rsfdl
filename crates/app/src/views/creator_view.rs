use dioxus::prelude::*;

use rsfdl_core::sfdl::builder::build_bulk_package;
use rsfdl_core::sfdl::crypto::encrypt_container;
use rsfdl_core::sfdl::models::{Connection, SfdlContainer};
use rsfdl_core::sfdl::parser::serialize_v3;

use crate::state::{AppState, AppView};

#[component]
pub fn CreatorView() -> Element {
	let mut state = use_context::<AppState>();
	let mut host = use_signal(String::new);
	let mut port = use_signal(|| 21u16);
	let mut username = use_signal(String::new);
	let mut password = use_signal(String::new);
	let mut remote_path = use_signal(String::new);
	let mut bulk_folder_mode = use_signal(|| true);
	let mut description = use_signal(String::new);
	let mut uploader = use_signal(|| "rsfdl".to_string());
	let mut threads = use_signal(|| 3u32);
	let mut encrypt_password = use_signal(String::new);
	let busy = use_signal(|| false);

	let is_busy = *busy.read();

	rsx! {
			div { class: "flex flex-col flex-1 overflow-hidden",
					// Sub-header
					div {
							class: "flex items-center justify-between px-4 py-3",
							style: "background: var(--color-surface); border-bottom: 1px solid var(--color-border);",
							h2 {
									class: "text-lg font-bold",
									style: "color: var(--color-text-primary);",
									"Create SFDL"
							}
							div { class: "flex items-center gap-2",
									button {
											class: "btn btn-accent",
											disabled: is_busy,
											onclick: {
													move |_| {
															let mut busy = busy;
															let host = host.read().clone();
															let port = *port.read();
															let username = username.read().clone();
															let password = password.read().clone();
															let remote_path = remote_path.read().clone();
															let bulk_folder_mode = *bulk_folder_mode.read();
															let description = description.read().clone();
															let uploader = uploader.read().clone();
															let threads = *threads.read();
															let encrypt_password = encrypt_password.read().clone();
															spawn(async move {
																	busy.set(true);
																	create_sfdl(
																			state,
																			host,
																			port,
																			username,
																			password,
																			remote_path,
																			bulk_folder_mode,
																			description,
																			uploader,
																			threads,
																			encrypt_password,
																	)
																	.await;
																	busy.set(false);
															});
													}
											},
											if is_busy { "Creating..." } else { "Create SFDL" }
									}
									button {
											class: "btn btn-ghost",
											onclick: move |_| {
													state.current_view.set(AppView::Main);
											},
											"Back"
									}
							}
					}

					// Form
					div { class: "flex-1 overflow-y-auto p-6 space-y-6",

							// FTP Connection
							div { class: "settings-card",
									h3 {
											class: "sg-title",
											"FTP Connection"
									}
									div { class: "grid grid-cols-3 gap-4",
											div { class: "col-span-2",
													label {
															class: "block text-sm font-medium mb-1",
															style: "color: var(--color-text-primary);",
															"Host"
													}
													input {
															class: "themed-input w-full",
															r#type: "text",
															placeholder: "ftp.example.com",
															value: "{host}",
															oninput: move |e| host.set(e.value()),
													}
											}
											div {
													label {
															class: "block text-sm font-medium mb-1",
															style: "color: var(--color-text-primary);",
															"Port"
													}
													input {
															class: "themed-input w-24",
															r#type: "number",
															min: "1",
															max: "65535",
															value: "{port}",
															onchange: move |e| {
																	if let Ok(n) = e.value().parse::<u16>() {
																			port.set(n.max(1));
																	}
															},
													}
											}
									}
									div { class: "grid grid-cols-2 gap-4 mt-3",
											div {
													label {
															class: "block text-sm font-medium mb-1",
															style: "color: var(--color-text-primary);",
															"Username"
													}
													input {
															class: "themed-input w-full",
															r#type: "text",
															placeholder: "ftpuser",
															value: "{username}",
															oninput: move |e| username.set(e.value()),
													}
											}
											div {
													label {
															class: "block text-sm font-medium mb-1",
															style: "color: var(--color-text-primary);",
															"Password"
													}
													input {
															class: "themed-input w-full",
															r#type: "password",
															placeholder: "\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}",
															value: "{password}",
															oninput: move |e| password.set(e.value()),
													}
											}
									}
							}

							// Content
							div { class: "settings-card",
									h3 {
											class: "sg-title",
											"Content"
									}
									div {
											label {
													class: "block text-sm font-medium mb-1",
													style: "color: var(--color-text-primary);",
													"Remote Path"
											}
											input {
													class: "themed-input w-full",
													r#type: "text",
													placeholder: "/releases/movie/",
													value: "{remote_path}",
													oninput: move |e| remote_path.set(e.value()),
											}
									}
									div { class: "flex gap-6 mt-3",
											label { class: "flex items-center gap-2 cursor-pointer",
													input {
															r#type: "radio",
															name: "mode",
															class: "w-4 h-4",
															checked: *bulk_folder_mode.read(),
															onchange: move |_| bulk_folder_mode.set(true),
													}
													div {
															span {
																	class: "text-sm font-medium",
																	style: "color: var(--color-text-primary);",
																	"BulkFolder"
															}
															span {
																	class: "block text-xs",
																	style: "color: var(--color-text-tertiary);",
																	"Store path only, no FTP listing"
															}
													}
											}
											label { class: "flex items-center gap-2 cursor-pointer",
													input {
															r#type: "radio",
															name: "mode",
															class: "w-4 h-4",
															checked: !*bulk_folder_mode.read(),
															onchange: move |_| bulk_folder_mode.set(false),
													}
													div {
															span {
																	class: "text-sm font-medium",
																	style: "color: var(--color-text-primary);",
																	"FileList"
															}
															span {
																	class: "block text-xs",
																	style: "color: var(--color-text-tertiary);",
																	"Connect to FTP and list files with sizes"
															}
													}
											}
									}
							}

							// Metadata
							div { class: "settings-card",
									h3 {
											class: "sg-title",
											"Metadata"
									}
									div { class: "grid grid-cols-3 gap-4",
											div { class: "col-span-2",
													label {
															class: "block text-sm font-medium mb-1",
															style: "color: var(--color-text-primary);",
															"Description"
													}
													input {
															class: "themed-input w-full",
															r#type: "text",
															placeholder: "Movie.2026.1080p",
															value: "{description}",
															oninput: move |e| description.set(e.value()),
													}
											}
											div {
													label {
															class: "block text-sm font-medium mb-1",
															style: "color: var(--color-text-primary);",
															"Threads"
													}
													input {
															class: "themed-input w-24",
															r#type: "number",
															min: "1",
															max: "10",
															value: "{threads}",
															onchange: move |e| {
																	if let Ok(n) = e.value().parse::<u32>() {
																			threads.set(n.clamp(1, 10));
																	}
															},
													}
											}
									}
									div { class: "mt-3",
											label {
													class: "block text-sm font-medium mb-1",
													style: "color: var(--color-text-primary);",
													"Uploader"
											}
											input {
													class: "themed-input w-full",
													r#type: "text",
													value: "{uploader}",
													oninput: move |e| uploader.set(e.value()),
											}
									}
							}

							// Encryption
							div { class: "settings-card",
									h3 {
											class: "sg-title",
											"Encryption (optional)"
									}
									div {
											label {
													class: "block text-sm font-medium mb-1",
													style: "color: var(--color-text-primary);",
													"Password"
											}
											input {
													class: "themed-input w-full",
													r#type: "password",
													placeholder: "Leave empty for no encryption",
													value: "{encrypt_password}",
													oninput: move |e| encrypt_password.set(e.value()),
											}
											p {
													class: "text-xs mt-1",
													style: "color: var(--color-text-tertiary);",
													"AES-128-CBC encryption, same as SFDL.NET"
											}
									}
							}
					}
			}
	}
}

#[allow(clippy::too_many_arguments)]
async fn create_sfdl(
	mut state: AppState,
	host: String,
	port: u16,
	username: String,
	password: String,
	remote_path: String,
	bulk_folder_mode: bool,
	description: String,
	uploader: String,
	threads: u32,
	encrypt_password: String,
) {
	// Validate required fields
	if host.is_empty() {
		state.error_message.set(Some("Host is required".to_string()));
		return;
	}
	if remote_path.is_empty() {
		state.error_message.set(Some("Remote path is required".to_string()));
		return;
	}

	// Derive package name from remote path
	let package_name = remote_path.trim_end_matches('/').rsplit('/').next().unwrap_or("Package1").to_string();

	// Build packages
	let packages = if bulk_folder_mode {
		vec![build_bulk_package(&remote_path, &package_name)]
	} else {
		// FileList mode: connect to FTP and list files
		let conn = Connection {
			host: host.clone(),
			port,
			username: username.clone(),
			password: password.clone(),
			auth_required: !username.is_empty(),
			..Connection::default()
		};
		let timeout = state.settings.read().ftp_timeout_seconds;
		match rsfdl_core::ftp::listing::resolve_container_bulk_folders(&conn, &[build_bulk_package(&remote_path, &package_name)], timeout).await {
			Ok(file_items) => {
				vec![rsfdl_core::sfdl::models::Package {
					name: package_name.clone(),
					bulk_folder_mode: false,
					file_list: file_items,
					bulk_folder_list: Vec::new(),
				}]
			}
			Err(e) => {
				state.error_message.set(Some(format!("FTP listing failed: {e}")));
				return;
			}
		}
	};

	// Build container
	let mut container = SfdlContainer {
		description,
		uploader,
		max_download_threads: threads,
		connection: Connection {
			host,
			port,
			username,
			password,
			auth_required: true,
			..Connection::default()
		},
		packages,
		..SfdlContainer::default()
	};

	// Optionally encrypt
	if !encrypt_password.is_empty() {
		encrypt_container(&mut container, &encrypt_password);
	}

	// Serialize
	let xml = match serialize_v3(&container) {
		Ok(xml) => xml,
		Err(e) => {
			state.error_message.set(Some(format!("Serialization failed: {e}")));
			return;
		}
	};

	// Save dialog
	let file = rfd::AsyncFileDialog::new().add_filter("SFDL Files", &["sfdl"]).set_file_name("container.sfdl").save_file().await;

	let Some(file) = file else { return };

	if let Err(e) = tokio::fs::write(file.path(), xml.as_bytes()).await {
		state.error_message.set(Some(format!("Failed to write file: {e}")));
		return;
	}

	state.error_message.set(None);
	// Show success via a temporary green message — reuse error_message with a green banner
	// For now, use the existing error banner (user sees confirmation via save dialog)
}
