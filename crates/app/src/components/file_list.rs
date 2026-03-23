use std::collections::HashMap;

use dioxus::prelude::*;

use crate::components::file_row::FileRow;
use crate::state::{AppState, DownloadPhase, FileStatus};

#[component]
pub fn FileList() -> Element {
	let mut state = use_context::<AppState>();
	let downloading = *state.download_phase.read() != DownloadPhase::Idle;

	let container = state.container.read();
	let Some(container) = container.as_ref() else {
		return rsx! {};
	};

	// Build name → status map once (O(n)) instead of per-row scanning
	let file_status_map: HashMap<String, FileStatus> = {
		let file_states = state.file_states.read();
		file_states.values().map(|fs| (fs.file_name.clone(), fs.status)).collect()
	};

	// Build package groups: (package_name, start_index, file_count)
	#[allow(clippy::type_complexity)]
	let mut groups: Vec<(String, usize, Vec<(usize, String, u64)>)> = Vec::new();
	let mut global_idx = 0;
	for pkg in &container.packages {
		let mut files = Vec::new();
		for file in &pkg.file_list {
			files.push((global_idx, file.file_name.clone(), file.file_size));
			global_idx += 1;
		}
		if !files.is_empty() {
			groups.push((pkg.name.clone(), files[0].0, files));
		}
	}

	if groups.is_empty() {
		return rsx! {};
	}

	let selected = state.selected_files.read();
	let all_selected = !selected.is_empty() && selected.iter().all(|&s| s);
	let total_count = selected.len();

	rsx! {
			div { class: "flex-1 overflow-y-auto",
					// Select All header
					div { class: "flex items-center px-4 py-2 bg-gray-100 border-b text-sm font-medium text-gray-700",
							input {
									r#type: "checkbox",
									class: "mr-3",
									checked: all_selected,
									disabled: downloading,
									onchange: move |_| {
											let new_val = !all_selected;
											state.selected_files.set(vec![new_val; total_count]);
									},
							}
							span { "Select All" }
					}
					// Package groups
					for (pkg_name, _start_idx, files) in groups.iter() {
							{render_package_group(state, pkg_name, files, downloading, &file_status_map)}
					}
			}
	}
}

fn render_package_group(mut state: AppState, pkg_name: &str, files: &[(usize, String, u64)], downloading: bool, file_status_map: &HashMap<String, FileStatus>) -> Element {
	let indices: Vec<usize> = files.iter().map(|(i, _, _)| *i).collect();
	let selected = state.selected_files.read();
	let pkg_all_selected = indices.iter().all(|i| selected.get(*i).copied().unwrap_or(false));

	let show_header = !pkg_name.is_empty() && !files.is_empty();

	rsx! {
			if show_header {
					div { class: "flex items-center px-4 py-1.5 bg-gray-50 border-b text-sm font-medium text-gray-600",
							input {
									r#type: "checkbox",
									class: "mr-3",
									checked: pkg_all_selected,
									disabled: downloading,
									onchange: {
											let indices = indices.clone();
											move |_| {
													let new_val = !pkg_all_selected;
													let mut sel = state.selected_files.write();
													for &idx in &indices {
															if let Some(v) = sel.get_mut(idx) {
																	*v = new_val;
															}
													}
											}
									},
							}
							span { "{pkg_name}" }
					}
			}
			for (idx, file_name, file_size) in files.iter() {
					FileRow {
							key: "{idx}",
							index: *idx,
							file_name: file_name.clone(),
							file_size: *file_size,
							status: file_status_map.get(file_name).copied(),
					}
			}
	}
}
