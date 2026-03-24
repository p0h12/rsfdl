//! DL-003: Disk space checking before download.
//!
//! Called by the DownloadManager after building the item list and
//! checking resume state, but before starting actual transfers.

use std::path::Path;

use crate::error::DownloadError;

/// Minimum safety buffer in bytes (10 MB).
const MIN_BUFFER_BYTES: u64 = 10 * 1024 * 1024;

/// Safety buffer percentage (1%).
const BUFFER_PERCENT: f64 = 0.01;

/// Result of a disk space check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskSpaceResult {
	pub sufficient: bool,
	pub available_bytes: u64,
	pub required_bytes: u64,
}

/// BR-DL-006: Calculate required bytes from a list of (file_size, local_size) pairs.
///
/// Each pair represents one file to download:
/// - `file_size`: total remote file size
/// - `local_size`: bytes already on disk (0 if new, partial if resume)
///
/// Adds a 1% safety buffer (minimum 10 MB).
pub fn calculate_required(items: &[(u64, u64)]) -> u64 {
	let raw: u64 = items.iter().map(|(total, local)| total.saturating_sub(*local)).sum();
	add_safety_buffer(raw)
}

/// BR-DL-006: Add 1% safety buffer, minimum 10 MB.
pub fn add_safety_buffer(raw_bytes: u64) -> u64 {
	let buffer = ((raw_bytes as f64) * BUFFER_PERCENT) as u64;
	raw_bytes + buffer.max(MIN_BUFFER_BYTES)
}

/// Check disk space: query available space and compare with required.
///
/// Returns `Err(InsufficientDiskSpace)` if `strict` is true and space is insufficient.
pub fn check(dest_dir: &Path, items: &[(u64, u64)], strict: bool) -> Result<DiskSpaceResult, DownloadError> {
	std::fs::create_dir_all(dest_dir)?;

	let available = query_available_space(dest_dir)?;
	let required = calculate_required(items);

	let result = DiskSpaceResult {
		sufficient: available >= required,
		available_bytes: available,
		required_bytes: required,
	};

	if strict && !result.sufficient {
		return Err(DownloadError::InsufficientDiskSpace);
	}

	Ok(result)
}

/// Query available space on the filesystem containing the given path.
#[cfg(unix)]
pub fn query_available_space(path: &Path) -> Result<u64, DownloadError> {
	use std::ffi::CString;
	use std::os::unix::ffi::OsStrExt;

	let c_path = CString::new(path.as_os_str().as_bytes()).map_err(|e| DownloadError::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, e)))?;

	unsafe {
		let mut stat: libc::statvfs = std::mem::zeroed();
		if libc::statvfs(c_path.as_ptr(), &mut stat) != 0 {
			return Err(DownloadError::Io(std::io::Error::last_os_error()));
		}
		Ok(stat.f_bavail as u64 * stat.f_frsize as u64)
	}
}

#[cfg(windows)]
pub fn query_available_space(path: &Path) -> Result<u64, DownloadError> {
	use std::os::windows::ffi::OsStrExt;

	let wide: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
	let mut free_bytes: u64 = 0;

	unsafe {
		if windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW(wide.as_ptr(), &mut free_bytes as *mut u64, std::ptr::null_mut(), std::ptr::null_mut()) == 0 {
			return Err(DownloadError::Io(std::io::Error::last_os_error()));
		}
	}
	Ok(free_bytes)
}

#[cfg(test)]
mod tests {
	use super::*;

	// =======================================================
	// DL-003 | BR-DL-006: Safety buffer
	// =======================================================

	/// DL-003 | BR-DL-006: 1% buffer on large values.
	#[test]
	fn dl003_safety_buffer_percentage() {
		let result = add_safety_buffer(1_073_741_824); // 1 GB
		let expected = 1_073_741_824 + 10_737_418; // +1%
		assert_eq!(result, expected);
	}

	/// DL-003 | BR-DL-006: Minimum 10 MB buffer.
	#[test]
	fn dl003_safety_buffer_minimum() {
		let result = add_safety_buffer(100 * 1024 * 1024); // 100 MB
		let expected = 100 * 1024 * 1024 + MIN_BUFFER_BYTES; // +10 MB (1% would be 1 MB)
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

	/// DL-003 | Main Success: All new files (no local data).
	#[test]
	fn dl003_required_all_new() {
		let items = vec![(1000, 0), (2000, 0)];
		assert_eq!(calculate_required(&items), add_safety_buffer(3000));
	}

	/// DL-003 | Main Success (Step 2): Partial files reduce required space.
	#[test]
	fn dl003_required_with_partial() {
		let items = vec![(1000, 400), (2000, 500)];
		// 600 + 1500 = 2100
		assert_eq!(calculate_required(&items), add_safety_buffer(2100));
	}

	/// DL-003 | Main Success: Complete files need zero additional space.
	#[test]
	fn dl003_required_complete_files() {
		let items = vec![(1000, 1000), (2000, 2000)];
		assert_eq!(calculate_required(&items), add_safety_buffer(0));
	}

	/// DL-003 | Edge: Local size exceeds remote (oversized partial).
	#[test]
	fn dl003_required_local_exceeds_remote() {
		let items = vec![(1000, 5000)]; // local > remote
		assert_eq!(calculate_required(&items), add_safety_buffer(0)); // saturating_sub
	}

	/// DL-003 | Edge: Empty item list.
	#[test]
	fn dl003_required_empty() {
		assert_eq!(calculate_required(&[]), add_safety_buffer(0));
	}

	// =======================================================
	// DL-003 | check(): Full check with OS query
	// =======================================================

	/// DL-003 | Main Success: Real filesystem has enough space for small items.
	#[test]
	fn dl003_check_sufficient() {
		let dir = tempfile::tempdir().unwrap();
		let items = vec![(100, 0), (200, 0)]; // 300 bytes + buffer
		let result = check(dir.path(), &items, false).unwrap();
		assert!(result.sufficient);
		assert!(result.available_bytes > 0);
	}

	/// DL-003 | A2: Strict mode returns error on insufficient space.
	#[test]
	fn dl003_check_strict_insufficient() {
		let dir = tempfile::tempdir().unwrap();
		// Request absurdly large space
		let items = vec![(u64::MAX / 2, 0)];
		let result = check(dir.path(), &items, true);
		assert!(matches!(result, Err(DownloadError::InsufficientDiskSpace)));
	}

	/// DL-003 | A1: Non-strict mode returns result even when insufficient.
	#[test]
	fn dl003_check_nonstrict_insufficient() {
		let dir = tempfile::tempdir().unwrap();
		let items = vec![(u64::MAX / 2, 0)];
		let result = check(dir.path(), &items, false).unwrap();
		assert!(!result.sufficient);
	}

	/// DL-003 | Edge: Creates directory if it doesn't exist.
	#[test]
	fn dl003_check_creates_dir() {
		let dir = tempfile::tempdir().unwrap();
		let sub = dir.path().join("new_sub_dir");
		assert!(!sub.exists());
		let result = check(&sub, &[(100, 0)], false).unwrap();
		assert!(sub.exists());
		assert!(result.sufficient);
	}
}
