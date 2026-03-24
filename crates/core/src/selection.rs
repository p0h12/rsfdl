//! DL-001: File selection for downloads.
//!
//! Manages which files from an SFDL container are selected for download,
//! with support for exclusion patterns, per-file toggle, per-package toggle,
//! and computed statistics (count, size).

use crate::filter::is_excluded;
use crate::sfdl::models::SfdlContainer;

/// A file selection over an SFDL container's flattened file list.
///
/// The selection is a flat boolean list aligned with the files across all
/// packages (package 0 files first, then package 1, etc.).
#[derive(Debug, Clone)]
pub struct FileSelection {
	selected: Vec<bool>,
	file_sizes: Vec<u64>,
	package_ranges: Vec<PackageRange>,
}

#[derive(Debug, Clone)]
struct PackageRange {
	start: usize,
	count: usize,
}

impl FileSelection {
	/// DL-001 Main Success (Steps 1-3): Create initial selection from container + exclusion patterns.
	///
	/// All files are initially selected, then exclusion patterns (DL-002) are applied
	/// to deselect matching files.
	pub fn new(container: &SfdlContainer, patterns: &[String]) -> Self {
		let mut selected = Vec::new();
		let mut file_sizes = Vec::new();
		let mut package_ranges = Vec::new();

		for pkg in &container.packages {
			let start = selected.len();
			for file in &pkg.file_list {
				selected.push(!is_excluded(&file.file_name, patterns));
				file_sizes.push(file.file_size);
			}
			package_ranges.push(PackageRange {
				start,
				count: pkg.file_list.len(),
			});
		}

		Self {
			selected,
			file_sizes,
			package_ranges,
		}
	}

	/// Total number of files (selected + unselected).
	pub fn total_count(&self) -> usize {
		self.selected.len()
	}

	/// Number of currently selected files.
	pub fn selected_count(&self) -> usize {
		self.selected.iter().filter(|&&s| s).count()
	}

	/// BR-DL-002: Total size of all files in bytes.
	pub fn total_size(&self) -> u64 {
		self.file_sizes.iter().sum()
	}

	/// BR-DL-002: Total size of selected files in bytes.
	pub fn selected_size(&self) -> u64 {
		self.selected
			.iter()
			.zip(&self.file_sizes)
			.filter(|&(&sel, _)| sel)
			.map(|&(_, &size)| size)
			.sum()
	}

	/// Whether at least one file is selected (download can start).
	pub fn can_download(&self) -> bool {
		self.selected.iter().any(|&s| s)
	}

	/// Whether a specific file is selected (by flat index).
	pub fn is_selected(&self, index: usize) -> bool {
		self.selected.get(index).copied().unwrap_or(false)
	}

	/// Toggle a single file by flat index.
	pub fn toggle_file(&mut self, index: usize) {
		if let Some(s) = self.selected.get_mut(index) {
			*s = !*s;
		}
	}

	/// Set selection state for a single file.
	pub fn set_file(&mut self, index: usize, selected: bool) {
		if let Some(s) = self.selected.get_mut(index) {
			*s = selected;
		}
	}

	/// Toggle all files in a package by package index.
	///
	/// If any file in the package is unselected, selects all.
	/// If all files in the package are selected, deselects all.
	pub fn toggle_package(&mut self, package_index: usize) {
		let Some(range) = self.package_ranges.get(package_index) else {
			return;
		};
		let end = range.start + range.count;
		let all_selected = self.selected[range.start..end].iter().all(|&s| s);
		let new_value = !all_selected;
		for s in &mut self.selected[range.start..end] {
			*s = new_value;
		}
	}

	/// Select all files.
	pub fn select_all(&mut self) {
		self.selected.iter_mut().for_each(|s| *s = true);
	}

	/// Deselect all files.
	pub fn deselect_all(&mut self) {
		self.selected.iter_mut().for_each(|s| *s = false);
	}

	/// Number of packages.
	pub fn package_count(&self) -> usize {
		self.package_ranges.len()
	}

	/// Number of selected files in a specific package.
	pub fn package_selected_count(&self, package_index: usize) -> usize {
		let Some(range) = self.package_ranges.get(package_index) else {
			return 0;
		};
		let end = range.start + range.count;
		self.selected[range.start..end].iter().filter(|&&s| s).count()
	}

	/// Total number of files in a specific package.
	pub fn package_total_count(&self, package_index: usize) -> usize {
		self.package_ranges.get(package_index).map(|r| r.count).unwrap_or(0)
	}

	/// The selection as a boolean slice.
	pub fn as_slice(&self) -> &[bool] {
		&self.selected
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::sfdl::models::{FileItem, Package, SfdlContainer};

	fn make_file(name: &str, size: u64) -> FileItem {
		FileItem {
			file_name: name.to_string(),
			file_size: size,
			..FileItem::default()
		}
	}

	fn make_container(packages: Vec<(&str, Vec<(&str, u64)>)>) -> SfdlContainer {
		SfdlContainer {
			packages: packages
				.into_iter()
				.map(|(name, files)| Package {
					name: name.to_string(),
					file_list: files.into_iter().map(|(n, s)| make_file(n, s)).collect(),
					..Package::default()
				})
				.collect(),
			..SfdlContainer::default()
		}
	}

	// --- DL-001 | Main Success: Initial selection ---

	/// DL-001 | Main Success (Step 1): All files initially selected when no patterns.
	#[test]
	fn dl001_new_all_selected_without_patterns() {
		let container = make_container(vec![("Pkg", vec![("a.rar", 100), ("b.rar", 200)])]);
		let sel = FileSelection::new(&container, &[]);

		assert_eq!(sel.total_count(), 2);
		assert_eq!(sel.selected_count(), 2);
		assert!(sel.can_download());
	}

	/// DL-001 | Main Success (Step 2): Exclusion patterns deselect matching files.
	#[test]
	fn dl001_new_applies_exclusion_patterns() {
		let container = make_container(vec![("Pkg", vec![("movie.rar", 1000), ("info.nfo", 10), ("cover.jpg", 50)])]);
		let patterns = vec!["*.nfo".into(), "*.jpg".into()];
		let sel = FileSelection::new(&container, &patterns);

		assert_eq!(sel.total_count(), 3);
		assert_eq!(sel.selected_count(), 1);
		assert!(sel.is_selected(0)); // movie.rar
		assert!(!sel.is_selected(1)); // info.nfo
		assert!(!sel.is_selected(2)); // cover.jpg
	}

	/// DL-001 | Main Success (Step 3): Selected size computed correctly.
	#[test]
	fn dl001_new_computes_sizes() {
		let container = make_container(vec![("Pkg", vec![("movie.rar", 1000), ("info.nfo", 10)])]);
		let patterns = vec!["*.nfo".into()];
		let sel = FileSelection::new(&container, &patterns);

		assert_eq!(sel.total_size(), 1010);
		assert_eq!(sel.selected_size(), 1000);
	}

	// --- DL-001 | A1: All files excluded ---

	/// DL-001 | A1: All files excluded by patterns — can_download is false.
	#[test]
	fn dl001_all_excluded_cannot_download() {
		let container = make_container(vec![("Pkg", vec![("info.nfo", 10), ("cover.jpg", 50)])]);
		let patterns = vec!["*.nfo".into(), "*.jpg".into()];
		let sel = FileSelection::new(&container, &patterns);

		assert_eq!(sel.selected_count(), 0);
		assert!(!sel.can_download());
	}

	// --- DL-001 | Step 5: Manual selection changes ---

	/// DL-001 | Step 5: Toggle individual file.
	#[test]
	fn dl001_toggle_file() {
		let container = make_container(vec![("Pkg", vec![("a.rar", 100), ("b.rar", 200)])]);
		let mut sel = FileSelection::new(&container, &[]);

		assert!(sel.is_selected(1));
		sel.toggle_file(1);
		assert!(!sel.is_selected(1));
		assert_eq!(sel.selected_count(), 1);
		assert_eq!(sel.selected_size(), 100);

		sel.toggle_file(1);
		assert!(sel.is_selected(1));
		assert_eq!(sel.selected_count(), 2);
	}

	/// DL-001 | Step 5: Set specific file selection.
	#[test]
	fn dl001_set_file() {
		let container = make_container(vec![("Pkg", vec![("a.rar", 100), ("b.rar", 200)])]);
		let mut sel = FileSelection::new(&container, &[]);

		sel.set_file(0, false);
		assert!(!sel.is_selected(0));
		assert_eq!(sel.selected_count(), 1);

		sel.set_file(0, true);
		assert!(sel.is_selected(0));
		assert_eq!(sel.selected_count(), 2);
	}

	/// DL-001 | Step 5: Toggle package — deselects all when all are selected.
	#[test]
	fn dl001_toggle_package_deselect() {
		let container = make_container(vec![
			("Pkg1", vec![("a.rar", 100), ("b.rar", 200)]),
			("Pkg2", vec![("c.rar", 300)]),
		]);
		let mut sel = FileSelection::new(&container, &[]);

		sel.toggle_package(0);
		assert!(!sel.is_selected(0));
		assert!(!sel.is_selected(1));
		assert!(sel.is_selected(2)); // Pkg2 unaffected
		assert_eq!(sel.selected_count(), 1);
	}

	/// DL-001 | Step 5: Toggle package — selects all when some are unselected.
	#[test]
	fn dl001_toggle_package_select() {
		let container = make_container(vec![("Pkg", vec![("a.rar", 100), ("b.rar", 200), ("c.rar", 300)])]);
		let mut sel = FileSelection::new(&container, &[]);

		sel.set_file(1, false); // deselect b.rar
		assert_eq!(sel.selected_count(), 2);

		sel.toggle_package(0); // should select all (since not all are selected)
		assert_eq!(sel.selected_count(), 3);
		assert!(sel.is_selected(1));
	}

	/// DL-001 | Step 5: Select all / deselect all.
	#[test]
	fn dl001_select_all_deselect_all() {
		let container = make_container(vec![("Pkg", vec![("a.rar", 100), ("b.rar", 200)])]);
		let mut sel = FileSelection::new(&container, &[]);

		sel.deselect_all();
		assert_eq!(sel.selected_count(), 0);
		assert!(!sel.can_download());

		sel.select_all();
		assert_eq!(sel.selected_count(), 2);
		assert!(sel.can_download());
	}

	// --- DL-001 | A2: User deselects all ---

	/// DL-001 | A2: Deselect all — can_download is false, size is 0.
	#[test]
	fn dl001_deselect_all_cannot_download() {
		let container = make_container(vec![("Pkg", vec![("a.rar", 100)])]);
		let mut sel = FileSelection::new(&container, &[]);

		sel.deselect_all();
		assert!(!sel.can_download());
		assert_eq!(sel.selected_size(), 0);
		assert_eq!(sel.selected_count(), 0);
	}

	// --- DL-001 | A3: CLI mode (no manual selection) ---

	/// DL-001 | A3: CLI mode — initial selection is used as-is.
	#[test]
	fn dl001_cli_mode_initial_selection() {
		let container = make_container(vec![("Pkg", vec![("movie.rar", 1000), ("info.nfo", 10)])]);
		let patterns = vec!["*.nfo".into()];
		let sel = FileSelection::new(&container, &patterns);

		// CLI just uses the initial selection without changes
		assert!(sel.can_download());
		assert_eq!(sel.selected_count(), 1);
		assert_eq!(sel.selected_size(), 1000);
		assert_eq!(sel.as_slice(), &[true, false]);
	}

	// --- Multi-package tests ---

	/// DL-001 | BR-DL-001: Multi-package container with mixed selection.
	#[test]
	fn dl001_multi_package_selection() {
		let container = make_container(vec![
			("Movie", vec![("movie.part1.rar", 500), ("movie.part2.rar", 500), ("info.nfo", 5)]),
			("Subs", vec![("subs.srt", 20)]),
		]);
		let patterns = vec!["*.nfo".into()];
		let sel = FileSelection::new(&container, &patterns);

		assert_eq!(sel.total_count(), 4);
		assert_eq!(sel.selected_count(), 3); // movie.part1, movie.part2, subs.srt
		assert_eq!(sel.selected_size(), 1020);
		assert_eq!(sel.package_count(), 2);
		assert_eq!(sel.package_selected_count(0), 2); // Movie: 2 of 3
		assert_eq!(sel.package_total_count(0), 3);
		assert_eq!(sel.package_selected_count(1), 1); // Subs: 1 of 1
		assert_eq!(sel.package_total_count(1), 1);
	}

	// --- Edge cases ---

	/// DL-001 | Edge: Empty container — no files, no selection.
	#[test]
	fn dl001_empty_container() {
		let container = make_container(vec![]);
		let sel = FileSelection::new(&container, &[]);

		assert_eq!(sel.total_count(), 0);
		assert_eq!(sel.selected_count(), 0);
		assert!(!sel.can_download());
		assert_eq!(sel.selected_size(), 0);
	}

	/// DL-001 | Edge: Package with no files.
	#[test]
	fn dl001_empty_package() {
		let container = make_container(vec![("Empty", vec![])]);
		let sel = FileSelection::new(&container, &[]);

		assert_eq!(sel.total_count(), 0);
		assert_eq!(sel.package_count(), 1);
		assert_eq!(sel.package_total_count(0), 0);
	}

	/// DL-001 | Edge: Toggle file with out-of-bounds index is a no-op.
	#[test]
	fn dl001_toggle_out_of_bounds() {
		let container = make_container(vec![("Pkg", vec![("a.rar", 100)])]);
		let mut sel = FileSelection::new(&container, &[]);

		sel.toggle_file(999);
		assert_eq!(sel.selected_count(), 1); // unchanged
	}

	/// DL-001 | Edge: Toggle package with out-of-bounds index is a no-op.
	#[test]
	fn dl001_toggle_package_out_of_bounds() {
		let container = make_container(vec![("Pkg", vec![("a.rar", 100)])]);
		let mut sel = FileSelection::new(&container, &[]);

		sel.toggle_package(999);
		assert_eq!(sel.selected_count(), 1); // unchanged
	}

	/// DL-001 | BR-DL-002: Zero-size files are counted but don't affect size.
	#[test]
	fn dl001_zero_size_files() {
		let container = make_container(vec![("Pkg", vec![("movie.rar", 1000), ("empty.bin", 0)])]);
		let sel = FileSelection::new(&container, &[]);

		assert_eq!(sel.total_count(), 2);
		assert_eq!(sel.selected_count(), 2);
		assert_eq!(sel.selected_size(), 1000);
		assert_eq!(sel.total_size(), 1000);
	}
}
