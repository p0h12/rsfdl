use std::path::PathBuf;
use uuid::Uuid;

/// Events sent from download tasks to the UI layer (CLI or GUI).
#[derive(Debug, Clone)]
pub enum ProgressEvent {
	/// A download task has started.
	Started { item_id: Uuid, file_name: String, total_bytes: u64 },
	/// Bytes written for a specific file.
	BytesWritten { item_id: Uuid, bytes_delta: u64, total_written: u64 },
	/// A file download completed successfully.
	Completed { item_id: Uuid },
	/// A file was skipped (already fully downloaded).
	Skipped { item_id: Uuid, file_name: String },
	/// A file download failed.
	Failed { item_id: Uuid, error: String },
	/// A file download was cancelled.
	Cancelled { item_id: Uuid },
	/// All downloads finished.
	AllDone {
		total_files: u32,
		completed: u32,
		failed: u32,
		cancelled: u32,
		skipped: u32,
	},
	/// Extraction of an archive started.
	ExtractionStarted { archive_path: PathBuf, archive_name: String },
	/// Extraction progress for an archive.
	ExtractionProgress { archive_path: PathBuf, percent: u8 },
	/// Extraction of an archive completed successfully.
	ExtractionCompleted { archive_path: PathBuf },
	/// Extraction of an archive failed.
	ExtractionFailed { archive_path: PathBuf, error: String },
	/// All extractions finished.
	ExtractionAllDone { total_archives: u32, extracted: u32, failed: u32 },
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn progress_event_debug_and_clone() {
		let id = Uuid::new_v4();
		let evt = ProgressEvent::Started {
			item_id: id,
			file_name: "test.rar".into(),
			total_bytes: 1024,
		};
		let _cloned = evt.clone();
		let _debug = format!("{:?}", evt);
	}

	#[test]
	fn all_done_variant() {
		let evt = ProgressEvent::AllDone {
			total_files: 5,
			completed: 3,
			failed: 1,
			cancelled: 0,
			skipped: 1,
		};
		let _debug = format!("{:?}", evt);
	}

	// --- UC-14: Extraction event variants ---

	#[test]
	fn extraction_started_debug_and_clone() {
		let evt = ProgressEvent::ExtractionStarted {
			archive_path: PathBuf::from("/tmp/movie.rar"),
			archive_name: "movie.rar".into(),
		};
		let _cloned = evt.clone();
		let _debug = format!("{:?}", evt);
	}

	#[test]
	fn extraction_progress_variant() {
		let evt = ProgressEvent::ExtractionProgress {
			archive_path: PathBuf::from("/tmp/movie.rar"),
			percent: 50,
		};
		let _debug = format!("{:?}", evt);
	}

	#[test]
	fn extraction_completed_variant() {
		let evt = ProgressEvent::ExtractionCompleted {
			archive_path: PathBuf::from("/tmp/movie.rar"),
		};
		let _debug = format!("{:?}", evt);
	}

	#[test]
	fn extraction_failed_variant() {
		let evt = ProgressEvent::ExtractionFailed {
			archive_path: PathBuf::from("/tmp/broken.rar"),
			error: "CRC mismatch".into(),
		};
		let _debug = format!("{:?}", evt);
	}

	#[test]
	fn extraction_all_done_variant() {
		let evt = ProgressEvent::ExtractionAllDone {
			total_archives: 3,
			extracted: 2,
			failed: 1,
		};
		let _debug = format!("{:?}", evt);
	}
}
