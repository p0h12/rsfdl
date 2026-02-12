use dioxus::prelude::*;

use crate::state::{AppState, DownloadPhase, FileStatus};
use rsfdl_core::format_bytes;

#[component]
pub fn FileRow(
    index: usize,
    file_name: String,
    file_size: u64,
    status: Option<FileStatus>,
) -> Element {
    let mut state = use_context::<AppState>();
    let selected = state.selected_files.read();
    let is_selected = selected.get(index).copied().unwrap_or(false);
    let downloading = *state.download_phase.read() != DownloadPhase::Idle;

    let (status_text, status_color) = match status {
        Some(FileStatus::Downloading) => ("downloading", "text-blue-600"),
        Some(FileStatus::Completed) => ("completed", "text-green-600"),
        Some(FileStatus::Failed) => ("failed", "text-red-600"),
        Some(FileStatus::Cancelled) => ("cancelled", "text-yellow-600"),
        Some(FileStatus::Skipped) => ("skipped", "text-gray-400"),
        Some(FileStatus::Pending) | None => ("", "text-gray-400"),
    };

    rsx! {
        div { class: "flex items-center px-4 py-1.5 hover:bg-gray-50 text-sm border-b border-gray-100",
            input {
                r#type: "checkbox",
                class: "mr-3",
                checked: is_selected,
                disabled: downloading,
                onchange: move |_| {
                    let mut sel = state.selected_files.write();
                    if let Some(v) = sel.get_mut(index) {
                        *v = !*v;
                    }
                },
            }
            span { class: "flex-1 truncate text-gray-800", "{file_name}" }
            span { class: "w-24 text-right text-gray-500 mr-4",
                "{format_bytes(file_size)}"
            }
            span { class: "w-24 text-right {status_color}", "{status_text}" }
        }
    }
}
