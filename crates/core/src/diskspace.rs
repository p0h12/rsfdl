//! DL-003: Disk space checking before download.
//!
//! Verifies that the target directory has enough free space for the
//! selected files, accounting for partially downloaded files (resume)
//! and a safety buffer.

use std::path::Path;

use crate::error::DownloadError;
use crate::selection::FileSelection;
use crate::sfdl::models::SfdlContainer;

/// Minimum safety buffer in bytes (10 MB).
const MIN_BUFFER_BYTES: u64 = 10 * 1024 * 1024;

/// Safety buffer percentage (1%).
const BUFFER_PERCENT: f64 = 0.01;

/// Result of a disk space check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskSpaceResult {
	/// Whether there is enough space.
	pub sufficient: bool,
	/// Available bytes on the target filesystem.
	pub available_bytes: u64,
	/// Required bytes (including safety buffer).
	pub required_bytes: u64,
	/// Number of files with known size used in calculation.
	pub files_with_known_size: usize,
	/// Total number of selected files.
	pub total_selected_files: usize,
}

impl DiskSpaceResult {
	/// Whether some file sizes are unknown (BulkFolder files before resolution).
	pub fn has_unknown_sizes(&self) -> bool {
		self.files_with_known_size < self.total_selected_files
	}
}

/// BR-DL-006: Calculate the required download space in bytes.
///
/// For each selected file:
/// - If a local partial file exists: only the remaining bytes are needed
/// - If no local file exists: the full file size is needed
/// - Files with size 0 are ignored (unknown size)
///
/// A safety buffer of 1% (minimum 10 MB) is added.
pub fn calculate_required(
	container: &SfdlContainer,
	selection: &FileSelection,
	download_dir: &Path,
) -> (u64, usize, usize) {
	let mut raw_required: u64 = 0;
	let mut files_with_known_size: usize = 0;
	let mut total_selected: usize = 0;

	let mut idx = 0;
	for pkg in &container.packages {
		for file in &pkg.file_list {
			let selected = selection.is_selected(idx);
			idx += 1;

			if !selected {
				continue;
			}

			total_selected += 1;

			if file.file_size == 0 {
				continue;
			}

			files_with_known_size += 1;

			// Check if a partial local file exists
			let local_path = download_dir.join(&file.directory_path).join(&file.file_name);
			let local_size = local_path.metadata().map(|m| m.len()).unwrap_or(0);

			let needed = file.file_size.saturating_sub(local_size);
			raw_required += needed;
		}
	}

	let buffered = add_safety_buffer(raw_required);
	(buffered, files_with_known_size, total_selected)
}

/// BR-DL-006: Add 1% safety buffer, minimum 10 MB.
pub fn add_safety_buffer(raw_bytes: u64) -> u64 {
	let buffer = ((raw_bytes as f64) * BUFFER_PERCENT) as u64;
	let buffer = buffer.max(MIN_BUFFER_BYTES);
	raw_bytes + buffer
}

/// Main entry point: check disk space and return result.
///
/// This queries the actual filesystem for available space.
pub fn check_disk_space(
	container: &SfdlContainer,
	selection: &FileSelection,
	download_dir: &Path,
) -> Result<DiskSpaceResult, DownloadError> {
	// Ensure directory exists for the space query
	std::fs::create_dir_all(download_dir)?;

	let available = query_available_space(download_dir)?;
	let (required, known, total) = calculate_required(container, selection, download_dir);

	Ok(DiskSpaceResult {
		sufficient: available >= required,
		available_bytes: available,
		required_bytes: required,
		files_with_known_size: known,
		total_selected_files: total,
	})
}

/// Check disk space and fail if insufficient in strict mode.
pub fn check_disk_space_strict(
	container: &SfdlContainer,
	selection: &FileSelection,
	download_dir: &Path,
	strict: bool,
) -> Result<DiskSpaceResult, DownloadError> {
	let result = check_disk_space(container, selection, download_dir)?;

	if strict && !result.sufficient {
		return Err(DownloadError::InsufficientDiskSpace);
	}

	Ok(result)
}

/// Query available space on the filesystem containing the given path.
#[cfg(unix)]
fn query_available_space(path: &Path) -> Result<u64, DownloadError> {
	use std::ffi::CString;
	use std::os::unix::ffi::OsStrExt;

	let c_path = CString::new(path.as_os_str().as_bytes())
		.map_err(|e| DownloadError::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, e)))?;

	unsafe {
		let mut stat: libc::statvfs = std::mem::zeroed();
		if libc::statvfs(c_path.as_ptr(), &mut stat) != 0 {
			return Err(DownloadError::Io(std::io::Error::last_os_error()));
		}
		Ok(stat.f_bavail as u64 * stat.f_frsize as u64)
	}
}

#[cfg(windows)]
fn query_available_space(path: &Path) -> Result<u64, DownloadError> {
	use std::os::windows::ffi::OsStrExt;

	let wide: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
	let mut free_bytes: u64 = 0;

	unsafe {
		if windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW(
			wide.as_ptr(),
			&mut free_bytes as *mut u64,
			std::ptr::null_mut(),
			std::ptr::null_mut(),
		) == 0
		{
			return Err(DownloadError::Io(std::io::Error::last_os_error()));
		}
	}
	Ok(free_bytes)
}

/// Format a disk space result as a human-readable message.
pub fn format_result(result: &DiskSpaceResult) -> String {
	let required_mb = result.required_bytes as f64 / (1024.0 * 1024.0);
	let available_mb = result.available_bytes as f64 / (1024.0 * 1024.0);

	if result.sufficient {
		format!("Disk space OK: {:.1} MB required, {:.1} MB available", required_mb, available_mb)
	} else {
		format!(
			"Insufficient disk space: {:.1} MB required, {:.1} MB available",
			required_mb, available_mb
		)
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
			directory_path: "release".to_string(),
			..FileItem::default()
		}
	}

	fn make_container(files: Vec<(&str, u64)>) -> SfdlContainer {
		SfdlContainer {
			packages: vec![Package {
				name: "Pkg".into(),
				file_list: files.into_iter().map(|(n, s)| make_file(n, s)).collect(),
				..Package::default()
			}],
			..SfdlContainer::default()
		}
	}

	// =======================================================
	// DL-003 | BR-DL-006: Safety buffer calculation
	// =======================================================

	/// DL-003 | BR-DL-006: 1% buffer on large values.
	#[test]
	fn dl003_safety_buffer_percentage() {
		// 1 GB -> 1% = 10.48 MB (> MIN_BUFFER of 10 MB)
		let result = add_safety_buffer(1_073_741_824);
		let expected = 1_073_741_824 + 10_737_418; // 1% of 1 GB
		assert_eq!(result, expected);
	}

	/// DL-003 | BR-DL-006: Minimum 10 MB buffer.
	#[test]
	fn dl003_safety_buffer_minimum() {
		// 100 MB -> 1% = 1 MB, but minimum is 10 MB
		let result = add_safety_buffer(100 * 1024 * 1024);
		let expected = 100 * 1024 * 1024 + MIN_BUFFER_BYTES;
		assert_eq!(result, expected);
	}

	/// DL-003 | BR-DL-006: Zero bytes still gets minimum buffer.
	#[test]
	fn dl003_safety_buffer_zero() {
		assert_eq!(add_safety_buffer(0), MIN_BUFFER_BYTES);
	}

	// =======================================================
	// DL-003 | Main Success: Required space calculation
	// =======================================================

	/// DL-003 | Main Success: All new files, no local files.
	#[test]
	fn dl003_required_all_new() {
		let container = make_container(vec![("a.rar", 1000), ("b.rar", 2000)]);
		let selection = FileSelection::new(&container, &[]);
		let dir = tempfile::tempdir().unwrap();

		let (required, known, total) = calculate_required(&container, &selection, dir.path());

		assert_eq!(known, 2);
		assert_eq!(total, 2);
		assert_eq!(required, add_safety_buffer(3000));
	}

	/// DL-003 | Main Success (Step 2): Partial file reduces required space.
	#[test]
	fn dl003_required_with_partial_file() {
		let container = make_container(vec![("a.rar", 1000)]);
		let selection = FileSelection::new(&container, &[]);
		let dir = tempfile::tempdir().unwrap();

		// Create partial local file (400 bytes of 1000)
		let release_dir = dir.path().join("release");
		std::fs::create_dir_all(&release_dir).unwrap();
		std::fs::write(release_dir.join("a.rar"), vec![0u8; 400]).unwrap();

		let (required, known, total) = calculate_required(&container, &selection, dir.path());

		assert_eq!(known, 1);
		assert_eq!(total, 1);
		assert_eq!(required, add_safety_buffer(600)); // 1000 - 400
	}

	/// DL-003 | Main Success: Complete file needs zero additional space.
	#[test]
	fn dl003_required_complete_file_zero() {
		let container = make_container(vec![("a.rar", 1000)]);
		let selection = FileSelection::new(&container, &[]);
		let dir = tempfile::tempdir().unwrap();

		// Create complete local file
		let release_dir = dir.path().join("release");
		std::fs::create_dir_all(&release_dir).unwrap();
		std::fs::write(release_dir.join("a.rar"), vec![0u8; 1000]).unwrap();

		let (required, _, _) = calculate_required(&container, &selection, dir.path());

		assert_eq!(required, add_safety_buffer(0));
	}

	/// DL-003 | Main Success: Unselected files are not counted.
	#[test]
	fn dl003_required_excludes_unselected() {
		let container = make_container(vec![("a.rar", 1000), ("info.nfo", 500)]);
		let selection = FileSelection::new(&container, &["*.nfo".into()]);
		let dir = tempfile::tempdir().unwrap();

		let (required, known, total) = calculate_required(&container, &selection, dir.path());

		assert_eq!(total, 1); // only a.rar selected
		assert_eq!(known, 1);
		assert_eq!(required, add_safety_buffer(1000));
	}

	// =======================================================
	// DL-003 | A3: Unknown file sizes
	// =======================================================

	/// DL-003 | A3: Files with size 0 are counted as unknown.
	#[test]
	fn dl003_unknown_sizes() {
		let container = make_container(vec![("a.rar", 1000), ("b.rar", 0)]);
		let selection = FileSelection::new(&container, &[]);
		let dir = tempfile::tempdir().unwrap();

		let (required, known, total) = calculate_required(&container, &selection, dir.path());

		assert_eq!(total, 2);
		assert_eq!(known, 1); // b.rar has unknown size
		assert_eq!(required, add_safety_buffer(1000)); // only a.rar counted
	}

	// =======================================================
	// DL-003 | DiskSpaceResult
	// =======================================================

	/// DL-003 | Main Success: Sufficient space.
	#[test]
	fn dl003_result_sufficient() {
		let result = DiskSpaceResult {
			sufficient: true,
			available_bytes: 1_000_000,
			required_bytes: 500_000,
			files_with_known_size: 5,
			total_selected_files: 5,
		};
		assert!(!result.has_unknown_sizes());
	}

	/// DL-003 | A1: Insufficient space.
	#[test]
	fn dl003_result_insufficient() {
		let result = DiskSpaceResult {
			sufficient: false,
			available_bytes: 100_000,
			required_bytes: 500_000,
			files_with_known_size: 5,
			total_selected_files: 5,
		};
		assert!(!result.sufficient);
	}

	/// DL-003 | A3: Partial size info.
	#[test]
	fn dl003_result_has_unknown_sizes() {
		let result = DiskSpaceResult {
			sufficient: true,
			available_bytes: 1_000_000,
			required_bytes: 500_000,
			files_with_known_size: 3,
			total_selected_files: 5,
		};
		assert!(result.has_unknown_sizes());
	}

	// =======================================================
	// DL-003 | format_result
	// =======================================================

	/// DL-003 | Output: Sufficient space message.
	#[test]
	fn dl003_format_sufficient() {
		let result = DiskSpaceResult {
			sufficient: true,
			available_bytes: 1_073_741_824,
			required_bytes: 524_288_000,
			files_with_known_size: 5,
			total_selected_files: 5,
		};
		let msg = format_result(&result);
		assert!(msg.contains("OK"));
	}

	/// DL-003 | Output: Insufficient space message.
	#[test]
	fn dl003_format_insufficient() {
		let result = DiskSpaceResult {
			sufficient: false,
			available_bytes: 100_000_000,
			required_bytes: 524_288_000,
			files_with_known_size: 5,
			total_selected_files: 5,
		};
		let msg = format_result(&result);
		assert!(msg.contains("Insufficient"));
	}

	// =======================================================
	// DL-003 | Edge cases
	// =======================================================

	/// DL-003 | Edge: Empty selection requires only buffer.
	#[test]
	fn dl003_empty_selection() {
		let container = make_container(vec![]);
		let selection = FileSelection::new(&container, &[]);
		let dir = tempfile::tempdir().unwrap();

		let (required, known, total) = calculate_required(&container, &selection, dir.path());

		assert_eq!(total, 0);
		assert_eq!(known, 0);
		assert_eq!(required, MIN_BUFFER_BYTES);
	}
}
