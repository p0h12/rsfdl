use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::sfdl::models::FileItem;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadStatus {
	Pending,
	Running,
	Completed,
	Failed,
	Cancelled,
	Skipped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResumeAction {
	/// No local file exists — download from scratch.
	StartFresh,
	/// Local file partially downloaded — resume from offset.
	Resume(u64),
	/// Local file matches remote size — skip download.
	AlreadyComplete,
	/// Local file is oversized or remote size unknown — delete and restart (A2).
	DeleteAndRestart,
}

#[derive(Debug, Clone)]
pub struct DownloadItem {
	pub id: Uuid,
	pub file_item: FileItem,
	pub local_path: PathBuf,
	pub status: DownloadStatus,
	pub bytes_downloaded: u64,
	pub error_message: Option<String>,
}

impl DownloadItem {
	/// Create from a FileItem and a base download directory.
	/// Builds local path: base_dir / [package_name] / directory_path / file_name
	/// When `create_package_subfolder` is false, the package_name level is skipped.
	pub fn from_file_item(file_item: &FileItem, base_dir: &Path, package_name: &str, create_package_subfolder: bool) -> Self {
		let mut local_path = base_dir.to_path_buf();

		if create_package_subfolder && !package_name.is_empty() {
			local_path.push(package_name);
		}

		// directory_path may contain leading slashes — strip them for local path
		let dir_path = file_item.directory_path.trim_start_matches('/');
		if !dir_path.is_empty() {
			local_path.push(dir_path);
		}

		local_path.push(&file_item.file_name);

		Self {
			id: Uuid::new_v4(),
			file_item: file_item.clone(),
			local_path,
			status: DownloadStatus::Pending,
			bytes_downloaded: 0,
			error_message: None,
		}
	}

	/// DL-005 / BR-DL-012: Check local file state to determine resume action.
	///
	/// - No local file → StartFresh
	/// - Local size == remote size → AlreadyComplete (skip)
	/// - Local size < remote size → Resume(offset)
	/// - Local size > remote size → DeleteAndRestart (A2: possibly corrupt)
	/// - Remote size unknown (0) + local file exists → DeleteAndRestart (can't verify)
	pub fn check_local_state(&self) -> ResumeAction {
		match std::fs::metadata(&self.local_path) {
			Ok(meta) => {
				let local_size = meta.len();
				if local_size == 0 {
					ResumeAction::StartFresh
				} else if self.file_item.file_size == 0 {
					// Remote size unknown — can't tell if local is complete or corrupt
					ResumeAction::DeleteAndRestart
				} else if local_size == self.file_item.file_size {
					ResumeAction::AlreadyComplete
				} else if local_size < self.file_item.file_size {
					ResumeAction::Resume(local_size)
				} else {
					// local_size > file_size — oversized, possibly corrupt (A2)
					ResumeAction::DeleteAndRestart
				}
			}
			Err(_) => ResumeAction::StartFresh,
		}
	}

	pub fn progress_percent(&self) -> f64 {
		if self.file_item.file_size == 0 {
			return 0.0;
		}
		(self.bytes_downloaded as f64 / self.file_item.file_size as f64) * 100.0
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::sfdl::models::HashType;
	use std::io::Write;

	fn sample_file_item(name: &str, size: u64) -> FileItem {
		FileItem {
			file_name: name.into(),
			directory_root: "/release/".into(),
			directory_path: "/release/sub/".into(),
			full_path: format!("/release/sub/{}", name),
			file_size: size,
			hash_type: HashType::None,
			file_hash: String::new(),
			package_name: "TestPkg".into(),
		}
	}

	/// DL-004 | BR-DL-011: Local path mirrors remote structure with package subfolder.
	#[test]
	fn dl004_from_file_item_builds_correct_path() {
		let fi = sample_file_item("movie.rar", 1000);
		let item = DownloadItem::from_file_item(&fi, Path::new("/tmp/dl"), "TestPkg", true);
		assert_eq!(item.local_path, PathBuf::from("/tmp/dl/TestPkg/release/sub/movie.rar"));
		assert_eq!(item.status, DownloadStatus::Pending);
	}

	/// DL-004 | BR-DL-011: Without package subfolder, path skips package name.
	#[test]
	fn dl004_from_file_item_no_package_subfolder() {
		let fi = sample_file_item("movie.rar", 1000);
		let item = DownloadItem::from_file_item(&fi, Path::new("/tmp/dl"), "TestPkg", false);
		assert_eq!(item.local_path, PathBuf::from("/tmp/dl/release/sub/movie.rar"));
	}

	/// DL-004 | BR-DL-011: Empty package name is skipped.
	#[test]
	fn dl004_from_file_item_no_package_name() {
		let fi = sample_file_item("movie.rar", 1000);
		let item = DownloadItem::from_file_item(&fi, Path::new("/tmp/dl"), "", true);
		assert_eq!(item.local_path, PathBuf::from("/tmp/dl/release/sub/movie.rar"));
	}

	/// DL-004 | BR-DL-011: Empty directory path produces flat structure.
	#[test]
	fn dl004_from_file_item_empty_dir_path() {
		let fi = FileItem {
			file_name: "file.txt".into(),
			directory_path: String::new(),
			..sample_file_item("file.txt", 100)
		};
		let item = DownloadItem::from_file_item(&fi, Path::new("/tmp"), "pkg", true);
		assert_eq!(item.local_path, PathBuf::from("/tmp/pkg/file.txt"));
	}

	/// DL-004 | Progress: Percentage calculation mid-download.
	#[test]
	fn dl004_progress_percent_normal() {
		let fi = sample_file_item("f.rar", 1000);
		let mut item = DownloadItem::from_file_item(&fi, Path::new("/tmp"), "", true);
		item.bytes_downloaded = 500;
		assert!((item.progress_percent() - 50.0).abs() < 0.01);
	}

	/// DL-004 | Progress: Zero-size file returns 0%.
	#[test]
	fn dl004_progress_percent_zero_size() {
		let fi = sample_file_item("f.rar", 0);
		let item = DownloadItem::from_file_item(&fi, Path::new("/tmp"), "", true);
		assert!((item.progress_percent() - 0.0).abs() < 0.01);
	}

	/// DL-005 | Main Success: No local file → StartFresh.
	#[test]
	fn dl005_check_local_state_no_file() {
		let fi = sample_file_item("nonexistent_abc123.rar", 1000);
		let item = DownloadItem::from_file_item(&fi, Path::new("/tmp/no_such_dir_xyz"), "", true);
		assert!(matches!(item.check_local_state(), ResumeAction::StartFresh));
	}

	/// DL-005 | Main Success: Partial local file → Resume(offset).
	#[test]
	fn dl005_check_local_state_partial_file() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("partial.rar");
		{
			let mut f = std::fs::File::create(&path).unwrap();
			f.write_all(&[0u8; 500]).unwrap();
		}
		let fi = FileItem {
			file_name: "partial.rar".into(),
			directory_path: String::new(),
			file_size: 1000,
			..sample_file_item("partial.rar", 1000)
		};
		let mut item = DownloadItem::from_file_item(&fi, dir.path(), "", true);
		item.local_path = path;
		match item.check_local_state() {
			ResumeAction::Resume(offset) => assert_eq!(offset, 500),
			other => panic!("Expected Resume, got {:?}", other),
		}
	}

	/// DL-005 | BR-DL-012: Complete local file (exact size) → AlreadyComplete.
	#[test]
	fn dl005_check_local_state_complete_file() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("complete.rar");
		{
			let mut f = std::fs::File::create(&path).unwrap();
			f.write_all(&[0u8; 1000]).unwrap();
		}
		let fi = FileItem {
			file_name: "complete.rar".into(),
			directory_path: String::new(),
			file_size: 1000,
			..sample_file_item("complete.rar", 1000)
		};
		let mut item = DownloadItem::from_file_item(&fi, dir.path(), "", true);
		item.local_path = path;
		assert_eq!(item.check_local_state(), ResumeAction::AlreadyComplete);
	}

	/// DL-005 | A2: Local file larger than remote → DeleteAndRestart.
	#[test]
	fn dl005_check_local_state_oversized() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("oversized.rar");
		{
			let mut f = std::fs::File::create(&path).unwrap();
			f.write_all(&[0u8; 2000]).unwrap(); // 2000 > 1000
		}
		let fi = FileItem {
			file_name: "oversized.rar".into(),
			directory_path: String::new(),
			file_size: 1000,
			..sample_file_item("oversized.rar", 1000)
		};
		let mut item = DownloadItem::from_file_item(&fi, dir.path(), "", true);
		item.local_path = path;
		assert_eq!(item.check_local_state(), ResumeAction::DeleteAndRestart);
	}

	/// DL-005 | A2: Remote size unknown (0) + local file exists → DeleteAndRestart.
	#[test]
	fn dl005_check_local_state_unknown_remote_size() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("unknown.rar");
		{
			let mut f = std::fs::File::create(&path).unwrap();
			f.write_all(&[0u8; 500]).unwrap();
		}
		let fi = FileItem {
			file_name: "unknown.rar".into(),
			directory_path: String::new(),
			file_size: 0, // unknown remote size
			..sample_file_item("unknown.rar", 0)
		};
		let mut item = DownloadItem::from_file_item(&fi, dir.path(), "", true);
		item.local_path = path;
		assert_eq!(item.check_local_state(), ResumeAction::DeleteAndRestart);
	}

	/// DL-005 | Edge: Empty local file (0 bytes) → StartFresh.
	#[test]
	fn dl005_check_local_state_empty_file() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("empty.rar");
		std::fs::File::create(&path).unwrap(); // 0 bytes
		let fi = FileItem {
			file_name: "empty.rar".into(),
			directory_path: String::new(),
			file_size: 1000,
			..sample_file_item("empty.rar", 1000)
		};
		let mut item = DownloadItem::from_file_item(&fi, dir.path(), "", true);
		item.local_path = path;
		assert_eq!(item.check_local_state(), ResumeAction::StartFresh);
	}
}
