use dioxus::prelude::*;

use crate::state::{AppState, ContainerId};

#[component]
pub fn ContainerInfo(container_id: ContainerId) -> Element {
	let state = use_context::<AppState>();
	let containers = state.containers.read();
	let Some(cs) = containers.iter().find(|c| c.id == container_id) else {
		return rsx! {};
	};

	let host = cs.host_display().unwrap_or_default();
	let description = cs.container.description.clone();
	let packages = cs.package_count();
	let files = cs.total_file_count();

	rsx! {
			div {
					class: "grid gap-3 px-4 py-3.5",
					style: "grid-template-columns: repeat(auto-fit, minmax(140px, 1fr)); border-bottom: 0.5px solid var(--color-border-tertiary);",

					if !host.is_empty() {
							InfoItem { label: "Server", value: host }
					}
					if !description.is_empty() {
							InfoItem { label: "Beschreibung", value: description }
					}
					InfoItem { label: "Pakete", value: packages.to_string() }
					InfoItem { label: "Dateien", value: files.to_string() }
			}
	}
}

#[component]
fn InfoItem(label: String, value: String) -> Element {
	rsx! {
			div {
					div {
							style: "font-size: 10px; font-weight: 500; letter-spacing: .06em; text-transform: uppercase; color: var(--color-text-tertiary); margin-bottom: 3px;",
							"{label}"
					}
					div {
							style: "font-size: 13px; color: var(--color-text-primary); font-family: var(--font-mono); white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
							"{value}"
					}
			}
	}
}
