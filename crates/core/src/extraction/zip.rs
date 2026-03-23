//! ZIP extraction via the `zip` crate (UC-14).

use std::fs;
use std::io;
use std::path::Path;

use crate::error::ExtractionError;

/// Extract a ZIP file to the destination directory.
///
/// Calls `on_progress` with a percentage (0–100) as files are extracted.
pub fn extract_zip(zip_file: &Path, dest_dir: &Path, mut on_progress: impl FnMut(u8)) -> Result<(), ExtractionError> {
	let file = fs::File::open(zip_file).map_err(ExtractionError::Io)?;
	let mut archive = zip::ZipArchive::new(file).map_err(|e| ExtractionError::Zip(e.to_string()))?;

	let total = archive.len();
	if total == 0 {
		on_progress(100);
		return Ok(());
	}

	for i in 0..total {
		let mut entry = archive.by_index(i).map_err(|e| ExtractionError::Zip(e.to_string()))?;

		let out_path = dest_dir.join(entry.mangled_name());

		if entry.is_dir() {
			fs::create_dir_all(&out_path)?;
		} else {
			if let Some(parent) = out_path.parent() {
				fs::create_dir_all(parent)?;
			}
			let mut out_file = fs::File::create(&out_path)?;
			io::copy(&mut entry, &mut out_file)?;
		}

		let percent = ((i + 1) as f64 / total as f64 * 100.0) as u8;
		on_progress(percent);
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::io::Write;
	use tempfile::TempDir;

	fn create_test_zip(dir: &Path, name: &str, files: &[(&str, &[u8])]) -> std::path::PathBuf {
		let zip_path = dir.join(name);
		let file = fs::File::create(&zip_path).unwrap();
		let mut writer = zip::ZipWriter::new(file);

		for (fname, content) in files {
			writer.start_file::<_, ()>(*fname, zip::write::SimpleFileOptions::default()).unwrap();
			writer.write_all(content).unwrap();
		}

		writer.finish().unwrap();
		zip_path
	}

	// =================================================================
	// AT-22: ZIP extraction
	// =================================================================

	/// AT-22: Single file ZIP is extracted with correct content
	#[test]
	fn extract_zip_single_file() {
		let dir = TempDir::new().unwrap();
		let zip_path = create_test_zip(dir.path(), "test.zip", &[("hello.txt", b"Hello World")]);

		let dest = TempDir::new().unwrap();
		let mut progress_values = Vec::new();
		extract_zip(&zip_path, dest.path(), |p| progress_values.push(p)).unwrap();

		assert!(dest.path().join("hello.txt").exists());
		let content = fs::read_to_string(dest.path().join("hello.txt")).unwrap();
		assert_eq!(content, "Hello World");
		assert_eq!(*progress_values.last().unwrap(), 100);
	}

	/// AT-22: Multiple files in ZIP are all extracted
	#[test]
	fn extract_zip_multiple_files() {
		let dir = TempDir::new().unwrap();
		let zip_path = create_test_zip(dir.path(), "multi.zip", &[("file1.txt", b"Content 1"), ("file2.txt", b"Content 2"), ("file3.txt", b"Content 3")]);

		let dest = TempDir::new().unwrap();
		extract_zip(&zip_path, dest.path(), |_| {}).unwrap();

		assert!(dest.path().join("file1.txt").exists());
		assert!(dest.path().join("file2.txt").exists());
		assert!(dest.path().join("file3.txt").exists());
	}

	/// ZIP with subdirectory structure is preserved
	#[test]
	fn extract_zip_preserves_subdirectories() {
		let dir = TempDir::new().unwrap();
		let zip_path = create_test_zip(dir.path(), "subdir.zip", &[("sub/nested.txt", b"Nested content")]);

		let dest = TempDir::new().unwrap();
		extract_zip(&zip_path, dest.path(), |_| {}).unwrap();

		assert!(dest.path().join("sub").join("nested.txt").exists());
	}

	// =================================================================
	// Progress reporting
	// =================================================================

	/// Progress callback receives monotonically increasing percentages ending at 100
	#[test]
	fn extract_zip_progress_monotonic_to_100() {
		let dir = TempDir::new().unwrap();
		let zip_path = create_test_zip(dir.path(), "progress.zip", &[("a.txt", b"A"), ("b.txt", b"B"), ("c.txt", b"C"), ("d.txt", b"D")]);

		let dest = TempDir::new().unwrap();
		let mut progress_values = Vec::new();
		extract_zip(&zip_path, dest.path(), |p| progress_values.push(p)).unwrap();

		for window in progress_values.windows(2) {
			assert!(window[1] >= window[0], "progress must be monotonic");
		}
		assert_eq!(*progress_values.last().unwrap(), 100);
	}

	/// Empty ZIP reports 100% immediately
	#[test]
	fn extract_zip_empty_archive() {
		let dir = TempDir::new().unwrap();
		let zip_path = create_test_zip(dir.path(), "empty.zip", &[]);

		let dest = TempDir::new().unwrap();
		let mut progress_values = Vec::new();
		extract_zip(&zip_path, dest.path(), |p| progress_values.push(p)).unwrap();

		assert_eq!(progress_values, vec![100]);
	}

	// =================================================================
	// A2: Error handling
	// =================================================================

	/// A2: Nonexistent file returns ExtractionError::Io
	#[test]
	fn extract_zip_nonexistent_file_errors() {
		let dest = TempDir::new().unwrap();
		let result = extract_zip(Path::new("/nonexistent/fake.zip"), dest.path(), |_| {});
		assert!(result.is_err());
		assert!(matches!(result.unwrap_err(), ExtractionError::Io(_)));
	}

	/// A2: Invalid ZIP data returns ExtractionError::Zip
	#[test]
	fn extract_zip_invalid_data_errors() {
		let dir = TempDir::new().unwrap();
		fs::write(dir.path().join("bad.zip"), b"not a zip").unwrap();

		let dest = TempDir::new().unwrap();
		let result = extract_zip(&dir.path().join("bad.zip"), dest.path(), |_| {});
		assert!(result.is_err());
		assert!(matches!(result.unwrap_err(), ExtractionError::Zip(_)));
	}

	// =================================================================
	// BR-004: Overwrite behavior
	// =================================================================

	/// BR-004: Existing files are overwritten during extraction
	#[test]
	fn extract_zip_overwrites_existing_files() {
		let dir = TempDir::new().unwrap();
		let zip_path = create_test_zip(dir.path(), "overwrite.zip", &[("file.txt", b"New content")]);

		let dest = TempDir::new().unwrap();
		// Pre-create file with old content
		fs::write(dest.path().join("file.txt"), b"Old content").unwrap();

		extract_zip(&zip_path, dest.path(), |_| {}).unwrap();

		let content = fs::read_to_string(dest.path().join("file.txt")).unwrap();
		assert_eq!(content, "New content");
	}
}
