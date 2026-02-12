use dioxus::prelude::*;

use crate::state::AppState;
use rsfdl_core::format_bytes;

#[component]
pub fn ContainerInfo() -> Element {
    let state = use_context::<AppState>();
    let container = state.container.read();
    let Some(container) = container.as_ref() else {
        return rsx! {};
    };

    let description = &container.description;
    let uploader = &container.uploader;
    let host = &container.connection.host;
    let port = container.connection.port;
    let file_count: usize = container.packages.iter().map(|p| p.file_list.len()).sum();
    let bulk_count: usize = container
        .packages
        .iter()
        .map(|p| p.bulk_folder_list.len())
        .sum();
    let total_size = state.total_size();
    let selected_count = state.selected_count();
    let selected_size = state.selected_size();
    let resolving = *state.resolving_bulk_folders.read();

    rsx! {
        div { class: "px-4 py-3 bg-gray-50 border-b text-sm space-y-1",
            if !description.is_empty() {
                p { class: "font-medium text-gray-900", "{description}" }
            }
            div { class: "flex flex-wrap gap-x-6 gap-y-1 text-gray-600",
                span { "Server: {host}:{port}" }
                if !uploader.is_empty() {
                    span { "Uploader: {uploader}" }
                }
                span { "Files: {selected_count}/{file_count}" }
                if bulk_count > 0 {
                    span { "Bulk Folders: {bulk_count}" }
                }
                span { "Selected: {format_bytes(selected_size)} / {format_bytes(total_size)}" }
            }
            if resolving {
                div { class: "flex items-center gap-2 text-blue-600",
                    // Simple spinner
                    svg {
                        class: "animate-spin h-4 w-4",
                        xmlns: "http://www.w3.org/2000/svg",
                        fill: "none",
                        view_box: "0 0 24 24",
                        circle {
                            class: "opacity-25",
                            cx: "12",
                            cy: "12",
                            r: "10",
                            stroke: "currentColor",
                            stroke_width: "4",
                        }
                        path {
                            class: "opacity-75",
                            fill: "currentColor",
                            d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z",
                        }
                    }
                    span { "Resolving bulk folders via FTP..." }
                }
            }
        }
    }
}
