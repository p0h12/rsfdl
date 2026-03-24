//! Archive extraction module for UC-14.
//!
//! Detects and extracts RAR and ZIP archives after download completes.
//! Extraction is non-fatal — failures are reported but don't affect download success.

pub mod detector;
pub mod rar;
pub mod zip;

use std::path::Path;

use tokio::sync::mpsc;

use crate::download::progress::ProgressEvent;
use detector::{ArchiveType, DetectedArchive, detect_archives};

/// Result summary after all extractions complete.
pub struct ExtractionResult {
	pub total_archives: u32,
	pub extracted: u32,
	pub failed: u32,
}

/// Detect and extract archives in a directory.
/// Sends ProgressEvents for UI updates.
///
/// Called after download completes. Errors are non-fatal — each archive
/// is handled independently, and failures don't stop other extractions.
pub async fn extract_archives(directory: &Path, delete_after: bool, progress_tx: &mpsc::UnboundedSender<ProgressEvent>) -> ExtractionResult {
	let archives = detect_archives(directory);
	let total = archives.len() as u32;

	if total == 0 {
		let _ = progress_tx.send(ProgressEvent::ExtractionAllDone {
			total_archives: 0,
			extracted: 0,
			failed: 0,
		});
		return ExtractionResult {
			total_archives: 0,
			extracted: 0,
			failed: 0,
		};
	}

	let mut extracted = 0u32;
	let mut failed = 0u32;

	for archive in &archives {
		let archive_name = archive.main_file.file_name().unwrap_or_default().to_string_lossy().to_string();

		let _ = progress_tx.send(ProgressEvent::ExtractionStarted {
			archive_path: archive.main_file.clone(),
			archive_name: archive_name.clone(),
		});

		// A3: Check multi-part completeness before extracting
		if let Err(msg) = detector::check_multipart_complete(archive) {
			failed += 1;
			let _ = progress_tx.send(ProgressEvent::ExtractionFailed {
				archive_path: archive.main_file.clone(),
				error: msg,
			});
			continue;
		}

		let dest_dir = archive.main_file.parent().unwrap_or(directory).to_path_buf();

		let archive_path = archive.main_file.clone();
		let tx = progress_tx.clone();

		let result = match archive.archive_type {
			ArchiveType::Rar => {
				let ap = archive_path.clone();
				tokio::task::spawn_blocking(move || {
					rar::extract_rar(&ap, &dest_dir, |percent| {
						let _ = tx.send(ProgressEvent::ExtractionProgress {
							archive_path: archive_path.clone(),
							percent,
						});
					})
				})
				.await
			}
			ArchiveType::Zip => {
				let ap = archive_path.clone();
				tokio::task::spawn_blocking(move || {
					zip::extract_zip(&ap, &dest_dir, |percent| {
						let _ = tx.send(ProgressEvent::ExtractionProgress {
							archive_path: archive_path.clone(),
							percent,
						});
					})
				})
				.await
			}
		};

		match result {
			Ok(Ok(())) => {
				extracted += 1;
				let _ = progress_tx.send(ProgressEvent::ExtractionCompleted {
					archive_path: archive.main_file.clone(),
				});

				if delete_after {
					delete_archive_parts(archive);
				}
			}
			Ok(Err(e)) => {
				failed += 1;
				let _ = progress_tx.send(ProgressEvent::ExtractionFailed {
					archive_path: archive.main_file.clone(),
					error: e.to_string(),
				});
			}
			Err(e) => {
				// JoinError from spawn_blocking
				failed += 1;
				let _ = progress_tx.send(ProgressEvent::ExtractionFailed {
					archive_path: archive.main_file.clone(),
					error: format!("Task panicked: {e}"),
				});
			}
		}
	}

	let _ = progress_tx.send(ProgressEvent::ExtractionAllDone {
		total_archives: total,
		extracted,
		failed,
	});

	ExtractionResult {
		total_archives: total,
		extracted,
		failed,
	}
}

fn delete_archive_parts(archive: &DetectedArchive) {
	for part in &archive.all_parts {
		if part.exists() {
			if let Err(e) = std::fs::remove_file(part) {
				tracing::warn!(?part, error = %e, "Failed to delete archive part");
			} else {
				tracing::info!(?part, "Deleted archive part");
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::fs;
	use std::io::Write;
	use tempfile::TempDir;

	fn create_test_zip(dir: &Path, name: &str, files: &[(&str, &[u8])]) -> std::path::PathBuf {
		let zip_path = dir.join(name);
		let file = fs::File::create(&zip_path).unwrap();
		let mut writer = ::zip::ZipWriter::new(file);

		for (fname, content) in files {
			writer.start_file::<_, ()>(*fname, ::zip::write::SimpleFileOptions::default()).unwrap();
			writer.write_all(content).unwrap();
		}

		writer.finish().unwrap();
		zip_path
	}

	// =================================================================
	// A1: Kein Archiv erkannt → keine Aktion
	// =================================================================

	/// A1: No archives in directory → ExtractionResult all zeros
	#[tokio::test]
	async fn extract_archives_no_archives_returns_zero() {
		let dir = TempDir::new().unwrap();
		fs::write(dir.path().join("readme.txt"), b"hello").unwrap();

		let (tx, _rx) = mpsc::unbounded_channel();
		let result = extract_archives(dir.path(), false, &tx).await;

		assert_eq!(result.total_archives, 0);
		assert_eq!(result.extracted, 0);
		assert_eq!(result.failed, 0);
	}

	/// A1: No archives → ExtractionAllDone event with zeros is sent
	#[tokio::test]
	async fn extract_archives_no_archives_sends_all_done_event() {
		let dir = TempDir::new().unwrap();
		fs::write(dir.path().join("readme.txt"), b"hello").unwrap();

		let (tx, mut rx) = mpsc::unbounded_channel();
		extract_archives(dir.path(), false, &tx).await;

		let event = rx.recv().await.unwrap();
		assert!(matches!(
			event,
			ProgressEvent::ExtractionAllDone {
				total_archives: 0,
				extracted: 0,
				failed: 0,
			}
		));
	}

	// =================================================================
	// AT-22: Auto-Extraktion ZIP
	// =================================================================

	/// AT-22: ZIP is extracted, files appear in download directory
	#[tokio::test]
	async fn extract_archives_zip_extracts_content() {
		let dir = TempDir::new().unwrap();
		create_test_zip(dir.path(), "files.zip", &[("hello.txt", b"Hello World")]);

		let (tx, _rx) = mpsc::unbounded_channel();
		let result = extract_archives(dir.path(), false, &tx).await;

		assert_eq!(result.total_archives, 1);
		assert_eq!(result.extracted, 1);
		assert_eq!(result.failed, 0);
		assert!(dir.path().join("hello.txt").exists());
		assert_eq!(fs::read_to_string(dir.path().join("hello.txt")).unwrap(), "Hello World");
	}

	/// AT-22: After ZIP extraction with delete_after=false, archive still exists
	#[tokio::test]
	async fn extract_archives_zip_keeps_archive_when_not_deleting() {
		let dir = TempDir::new().unwrap();
		create_test_zip(dir.path(), "files.zip", &[("hello.txt", b"Hello World")]);

		let (tx, _rx) = mpsc::unbounded_channel();
		extract_archives(dir.path(), false, &tx).await;

		assert!(dir.path().join("files.zip").exists());
	}

	/// BR-006: Extraction sends Started → Completed → AllDone event sequence
	#[tokio::test]
	async fn extract_archives_zip_sends_correct_event_sequence() {
		let dir = TempDir::new().unwrap();
		create_test_zip(dir.path(), "files.zip", &[("hello.txt", b"Hello World")]);

		let (tx, mut rx) = mpsc::unbounded_channel();
		extract_archives(dir.path(), false, &tx).await;

		let mut events = Vec::new();
		while let Ok(evt) = rx.try_recv() {
			events.push(evt);
		}

		assert!(events.iter().any(|e| matches!(e, ProgressEvent::ExtractionStarted { .. })));
		assert!(events.iter().any(|e| matches!(e, ProgressEvent::ExtractionCompleted { .. })));
		assert!(events.iter().any(|e| matches!(e, ProgressEvent::ExtractionAllDone { extracted: 1, failed: 0, .. })));
	}

	// =================================================================
	// Archive deletion after extraction
	// =================================================================

	/// delete_after=true: ZIP archive is deleted after successful extraction
	#[tokio::test]
	async fn extract_archives_zip_deletes_archive_when_configured() {
		let dir = TempDir::new().unwrap();
		create_test_zip(dir.path(), "files.zip", &[("hello.txt", b"Hello World")]);

		let (tx, _rx) = mpsc::unbounded_channel();
		let result = extract_archives(dir.path(), true, &tx).await;

		assert_eq!(result.extracted, 1);
		assert!(dir.path().join("hello.txt").exists());
		assert!(!dir.path().join("files.zip").exists(), "archive should be deleted");
	}

	// =================================================================
	// A2: Extraktion schlägt fehl
	// =================================================================

	/// A2: Invalid archive → extraction fails, failure is counted
	#[tokio::test]
	async fn extract_archives_invalid_zip_reports_failure() {
		let dir = TempDir::new().unwrap();
		// Write invalid data as .zip
		fs::write(dir.path().join("broken.zip"), b"not a zip file").unwrap();

		let (tx, _rx) = mpsc::unbounded_channel();
		let result = extract_archives(dir.path(), false, &tx).await;

		assert_eq!(result.total_archives, 1);
		assert_eq!(result.extracted, 0);
		assert_eq!(result.failed, 1);
	}

	/// A2: Failed extraction sends ExtractionFailed event
	#[tokio::test]
	async fn extract_archives_invalid_zip_sends_failed_event() {
		let dir = TempDir::new().unwrap();
		fs::write(dir.path().join("broken.zip"), b"not a zip file").unwrap();

		let (tx, mut rx) = mpsc::unbounded_channel();
		extract_archives(dir.path(), false, &tx).await;

		let mut events = Vec::new();
		while let Ok(evt) = rx.try_recv() {
			events.push(evt);
		}

		assert!(events.iter().any(|e| matches!(e, ProgressEvent::ExtractionFailed { .. })));
	}

	/// A2: Failed extraction → archive is NOT deleted even with delete_after=true
	#[tokio::test]
	async fn extract_archives_failed_keeps_archive_even_with_delete() {
		let dir = TempDir::new().unwrap();
		fs::write(dir.path().join("broken.zip"), b"not a zip file").unwrap();

		let (tx, _rx) = mpsc::unbounded_channel();
		extract_archives(dir.path(), true, &tx).await;

		assert!(dir.path().join("broken.zip").exists(), "broken archive should not be deleted");
	}

	// =================================================================
	// BR-006: Non-blocking — one failure doesn't stop others
	// =================================================================

	/// BR-006: One broken archive doesn't prevent other archives from extracting
	#[tokio::test]
	async fn extract_archives_continues_after_failure() {
		let dir = TempDir::new().unwrap();
		// One valid ZIP, one broken ZIP
		create_test_zip(dir.path(), "good.zip", &[("extracted.txt", b"Good content")]);
		fs::write(dir.path().join("broken.zip"), b"not a zip").unwrap();

		let (tx, _rx) = mpsc::unbounded_channel();
		let result = extract_archives(dir.path(), false, &tx).await;

		assert_eq!(result.total_archives, 2);
		assert_eq!(result.extracted, 1);
		assert_eq!(result.failed, 1);
		assert!(dir.path().join("extracted.txt").exists());
	}

	// =================================================================
	// AT-23: Auto-Extraktion deaktiviert (caller-side)
	// =================================================================

	/// AT-23: When called on empty dir (simulating disabled feature), does nothing
	#[tokio::test]
	async fn extract_archives_empty_dir_returns_zero() {
		let dir = TempDir::new().unwrap();
		let (tx, _rx) = mpsc::unbounded_channel();
		let result = extract_archives(dir.path(), false, &tx).await;
		assert_eq!(result.total_archives, 0);
	}

	// =================================================================
	// Multiple archives in one directory
	// =================================================================

	/// Multiple ZIP files are each extracted independently
	#[tokio::test]
	async fn extract_archives_multiple_zips() {
		let dir = TempDir::new().unwrap();
		create_test_zip(dir.path(), "a.zip", &[("a.txt", b"Content A")]);
		create_test_zip(dir.path(), "b.zip", &[("b.txt", b"Content B")]);

		let (tx, _rx) = mpsc::unbounded_channel();
		let result = extract_archives(dir.path(), false, &tx).await;

		assert_eq!(result.total_archives, 2);
		assert_eq!(result.extracted, 2);
		assert!(dir.path().join("a.txt").exists());
		assert!(dir.path().join("b.txt").exists());
	}

	// =================================================================
	// POST-002: Recursive extraction in subdirectories
	// =================================================================

	/// POST-002 | Main Success: ZIP in subdirectory is extracted.
	#[tokio::test]
	async fn post002_extract_archives_in_subdirectory() {
		let dir = TempDir::new().unwrap();
		let sub = dir.path().join("pkg").join("release");
		std::fs::create_dir_all(&sub).unwrap();
		create_test_zip(&sub, "files.zip", &[("data.txt", b"Subdirectory content")]);

		let (tx, _rx) = mpsc::unbounded_channel();
		let result = extract_archives(dir.path(), false, &tx).await;

		assert_eq!(result.total_archives, 1);
		assert_eq!(result.extracted, 1);
		assert!(sub.join("data.txt").exists());
	}

	/// POST-002 | BR-POST-004: Existing files not overwritten during extraction.
	#[tokio::test]
	async fn post002_extract_archives_no_overwrite() {
		let dir = TempDir::new().unwrap();
		create_test_zip(dir.path(), "files.zip", &[("existing.txt", b"New")]);
		// Pre-create the file
		fs::write(dir.path().join("existing.txt"), b"Old").unwrap();

		let (tx, _rx) = mpsc::unbounded_channel();
		let result = extract_archives(dir.path(), false, &tx).await;

		assert_eq!(result.extracted, 1);
		let content = fs::read_to_string(dir.path().join("existing.txt")).unwrap();
		assert_eq!(content, "Old", "existing file must not be overwritten");
	}

	/// POST-002 | A3: Incomplete multi-part RAR → fails with descriptive error.
	#[tokio::test]
	async fn post002_incomplete_multipart_rar_fails() {
		let dir = TempDir::new().unwrap();
		// Create part01 and part03 but skip part02
		fs::write(dir.path().join("movie.part01.rar"), b"fake").unwrap();
		fs::write(dir.path().join("movie.part03.rar"), b"fake").unwrap();

		let (tx, mut rx) = mpsc::unbounded_channel();
		let result = extract_archives(dir.path(), false, &tx).await;

		assert_eq!(result.failed, 1);

		let mut events = Vec::new();
		while let Ok(evt) = rx.try_recv() {
			events.push(evt);
		}

		let failed_event = events.iter().find(|e| matches!(e, ProgressEvent::ExtractionFailed { .. }));
		assert!(failed_event.is_some(), "should emit ExtractionFailed");
		if let Some(ProgressEvent::ExtractionFailed { error, .. }) = failed_event {
			assert!(error.contains("unvollständig"), "error should mention incomplete: {error}");
		}
	}

	/// POST-002 | A3: Incomplete multi-part → archive files are preserved.
	#[tokio::test]
	async fn post002_incomplete_multipart_preserves_files() {
		let dir = TempDir::new().unwrap();
		fs::write(dir.path().join("movie.part01.rar"), b"fake").unwrap();
		fs::write(dir.path().join("movie.part03.rar"), b"fake").unwrap();

		let (tx, _rx) = mpsc::unbounded_channel();
		extract_archives(dir.path(), true, &tx).await;

		assert!(dir.path().join("movie.part01.rar").exists(), "part01 should be preserved");
		assert!(dir.path().join("movie.part03.rar").exists(), "part03 should be preserved");
	}

	/// POST-002 | BR-POST-005: Feature toggle — extraction only runs when called.
	/// (auto_extract check is in the caller, not in extract_archives itself)
	#[tokio::test]
	async fn post002_feature_toggle_empty_dir_noop() {
		let dir = TempDir::new().unwrap();
		let (tx, _rx) = mpsc::unbounded_channel();
		let result = extract_archives(dir.path(), false, &tx).await;
		assert_eq!(result.total_archives, 0);
	}
}
