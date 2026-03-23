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

#[derive(Debug, Clone, Copy)]
pub enum ResumeAction {
	StartFresh,
	Resume(u64),
	AlreadyComplete,
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

	/// Check if local file exists, determine what to do.
	pub fn check_local_state(&self) -> ResumeAction {
		match std::fs::metadata(&self.local_path) {
			Ok(meta) => {
				let local_size = meta.len();
				if self.file_item.file_size > 0 && local_size >= self.file_item.file_size {
					ResumeAction::AlreadyComplete
				} else if local_size > 0 {
					ResumeAction::Resume(local_size)
				} else {
					ResumeAction::StartFresh
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

	#[test]
	fn from_file_item_builds_correct_path() {
		let fi = sample_file_item("movie.rar", 1000);
		let item = DownloadItem::from_file_item(&fi, Path::new("/tmp/dl"), "TestPkg", true);
		assert_eq!(item.local_path, PathBuf::from("/tmp/dl/TestPkg/release/sub/movie.rar"));
		assert_eq!(item.status, DownloadStatus::Pending);
	}

	#[test]
	fn from_file_item_no_package_subfolder() {
		let fi = sample_file_item("movie.rar", 1000);
		let item = DownloadItem::from_file_item(&fi, Path::new("/tmp/dl"), "TestPkg", false);
		assert_eq!(item.local_path, PathBuf::from("/tmp/dl/release/sub/movie.rar"));
	}

	#[test]
	fn from_file_item_no_package_name() {
		let fi = sample_file_item("movie.rar", 1000);
		let item = DownloadItem::from_file_item(&fi, Path::new("/tmp/dl"), "", true);
		assert_eq!(item.local_path, PathBuf::from("/tmp/dl/release/sub/movie.rar"));
	}

	#[test]
	fn from_file_item_empty_dir_path() {
		let fi = FileItem {
			file_name: "file.txt".into(),
			directory_path: String::new(),
			..sample_file_item("file.txt", 100)
		};
		let item = DownloadItem::from_file_item(&fi, Path::new("/tmp"), "pkg", true);
		assert_eq!(item.local_path, PathBuf::from("/tmp/pkg/file.txt"));
	}

	#[test]
	fn progress_percent_normal() {
		let fi = sample_file_item("f.rar", 1000);
		let mut item = DownloadItem::from_file_item(&fi, Path::new("/tmp"), "", true);
		item.bytes_downloaded = 500;
		assert!((item.progress_percent() - 50.0).abs() < 0.01);
	}

	#[test]
	fn progress_percent_zero_size() {
		let fi = sample_file_item("f.rar", 0);
		let item = DownloadItem::from_file_item(&fi, Path::new("/tmp"), "", true);
		assert!((item.progress_percent() - 0.0).abs() < 0.01);
	}

	#[test]
	fn check_local_state_no_file() {
		let fi = sample_file_item("nonexistent_abc123.rar", 1000);
		let item = DownloadItem::from_file_item(&fi, Path::new("/tmp/no_such_dir_xyz"), "", true);
		assert!(matches!(item.check_local_state(), ResumeAction::StartFresh));
	}

	#[test]
	fn check_local_state_partial_file() {
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

	#[test]
	fn check_local_state_complete_file() {
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
		assert!(matches!(item.check_local_state(), ResumeAction::AlreadyComplete));
	}
}
