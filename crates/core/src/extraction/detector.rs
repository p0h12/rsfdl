//! Archive detection for UC-14.
//!
//! Scans a directory for RAR and ZIP archives, groups multi-part RAR files,
//! and returns only main archive entries (first part for multi-part RARs).

use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

/// A detected archive with all its related files.
#[derive(Debug, Clone)]
pub struct DetectedArchive {
	/// Path to the main file (first part for multi-part RAR).
	pub main_file: PathBuf,
	/// Archive type.
	pub archive_type: ArchiveType,
	/// All files belonging to this archive (including main_file).
	/// Used for deletion after extraction.
	pub all_parts: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveType {
	Rar,
	Zip,
}

// Matches .partNN.rar where NN is digits — used to detect multi-part naming
static PART_RAR_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\.part(\d+)\.rar$").unwrap());

// Matches any RAR-related file: .rar, .partNN.rar, .rNN
static RAR_PART_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\.(rar|part\d+\.rar|r\d{2,3})$").unwrap());

/// Check if a file name is a main RAR file (standalone or first part).
///
/// Accepts: `movie.rar`, `movie.part1.rar`, `movie.part01.rar`, `movie.part001.rar`
/// Rejects: `movie.part02.rar`, `movie.r00`, `movie.zip`
pub fn is_main_rar(file_name: &str) -> bool {
	let lower = file_name.to_lowercase();

	// Must end with .rar (but not .partN.rar where N > 1, and not .rNN)
	if !lower.ends_with(".rar") {
		return false;
	}

	// Check if it's a .partNN.rar pattern
	if let Some(caps) = PART_RAR_RE.captures(file_name) {
		// It's .partNN.rar — only accept if the number is all zeros followed by 1
		// i.e. "1", "01", "001", "0001" etc.
		let num = &caps[1];
		return num.trim_start_matches('0') == "1";
	}

	// Plain .rar (no .partNN pattern) — accept only if it's not .rNN
	// (the .rar ending already confirmed, and .rNN doesn't end with .rar, so this is fine)
	true
}

/// Check if a file name is any part of a RAR archive (main or continuation).
pub fn is_rar_part(file_name: &str) -> bool {
	RAR_PART_RE.is_match(file_name)
}

/// Extract the base name of a RAR archive for grouping parts.
/// E.g. "movie.part01.rar" → "movie", "movie.rar" → "movie", "movie.r00" → "movie"
fn rar_base_name(file_name: &str) -> Option<String> {
	let lower = file_name.to_lowercase();
	// Try .partNN.rar pattern first
	if let Some(pos) = lower.find(".part")
		&& lower.ends_with(".rar")
	{
		return Some(file_name[..pos].to_string());
	}
	// Try .rNN pattern
	if let Some(pos) = lower.rfind('.') {
		let ext = &lower[pos..];
		if ext == ".rar" {
			return Some(file_name[..pos].to_string());
		}
		// .r00, .r01, etc.
		static R_NUM_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\.r\d{2,3}$").unwrap());
		if R_NUM_RE.is_match(file_name) {
			return Some(file_name[..pos].to_string());
		}
	}
	None
}

/// Find all RAR parts that belong to the same archive as the main file.
pub fn find_related_rar_parts(main_file: &Path, directory: &Path) -> Vec<PathBuf> {
	let main_name = match main_file.file_name().and_then(|n| n.to_str()) {
		Some(n) => n,
		None => return vec![main_file.to_path_buf()],
	};
	let base = match rar_base_name(main_name) {
		Some(b) => b.to_lowercase(),
		None => return vec![main_file.to_path_buf()],
	};

	let mut parts = Vec::new();
	if let Ok(entries) = std::fs::read_dir(directory) {
		for entry in entries.flatten() {
			if !entry.file_type().ok().is_some_and(|ft| ft.is_file()) {
				continue;
			}
			let name = entry.file_name().to_string_lossy().to_string();
			if !is_rar_part(&name) {
				continue;
			}
			if let Some(entry_base) = rar_base_name(&name)
				&& entry_base.to_lowercase() == base
			{
				parts.push(entry.path());
			}
		}
	}

	if parts.is_empty() {
		parts.push(main_file.to_path_buf());
	}
	parts
}

/// Scan a directory (non-recursively) for archives.
/// Groups multi-part RAR files into a single DetectedArchive.
pub fn detect_archives(directory: &Path) -> Vec<DetectedArchive> {
	let entries: Vec<_> = std::fs::read_dir(directory)
		.ok()
		.into_iter()
		.flatten()
		.filter_map(|e| e.ok())
		.filter(|e| e.file_type().ok().is_some_and(|ft| ft.is_file()))
		.collect();

	let mut archives = Vec::new();

	// ZIP files
	for entry in &entries {
		let name = entry.file_name().to_string_lossy().to_string();
		if name.to_lowercase().ends_with(".zip") {
			archives.push(DetectedArchive {
				main_file: entry.path(),
				archive_type: ArchiveType::Zip,
				all_parts: vec![entry.path()],
			});
		}
	}

	// RAR main files (standalone or first part)
	for entry in &entries {
		let name = entry.file_name().to_string_lossy().to_string();
		if is_main_rar(&name) {
			let parts = find_related_rar_parts(&entry.path(), directory);
			archives.push(DetectedArchive {
				main_file: entry.path(),
				archive_type: ArchiveType::Rar,
				all_parts: parts,
			});
		}
	}

	archives
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::fs;
	use tempfile::TempDir;

	// =================================================================
	// BR-001: Multi-Part-RAR Erkennung — is_main_rar()
	// =================================================================

	/// BR-001: Standalone .rar is a main RAR file
	#[test]
	fn is_main_rar_standalone() {
		assert!(is_main_rar("movie.rar"));
	}

	/// BR-001: Case-insensitive matching
	#[test]
	fn is_main_rar_case_insensitive() {
		assert!(is_main_rar("Movie.RAR"));
		assert!(is_main_rar("MOVIE.Rar"));
	}

	/// BR-001: .part1.rar is first part (1 digit)
	#[test]
	fn is_main_rar_part1() {
		assert!(is_main_rar("movie.part1.rar"));
	}

	/// BR-001: .part01.rar is first part (2 digits)
	#[test]
	fn is_main_rar_part01() {
		assert!(is_main_rar("movie.part01.rar"));
	}

	/// BR-001: .part001.rar is first part (3 digits)
	#[test]
	fn is_main_rar_part001() {
		assert!(is_main_rar("movie.part001.rar"));
	}

	/// BR-001: .Part01.Rar is first part (mixed case)
	#[test]
	fn is_main_rar_part01_mixed_case() {
		assert!(is_main_rar("movie.Part01.Rar"));
	}

	/// BR-001: .part02.rar is NOT main (second part)
	#[test]
	fn is_main_rar_rejects_part02() {
		assert!(!is_main_rar("movie.part02.rar"));
	}

	/// BR-001: .part2.rar is NOT main
	#[test]
	fn is_main_rar_rejects_part2() {
		assert!(!is_main_rar("movie.part2.rar"));
	}

	/// BR-001: .part003.rar is NOT main (third part, 3 digits)
	#[test]
	fn is_main_rar_rejects_part003() {
		assert!(!is_main_rar("movie.part003.rar"));
	}

	/// BR-001: .part10.rar is NOT main
	#[test]
	fn is_main_rar_rejects_part10() {
		assert!(!is_main_rar("movie.part10.rar"));
	}

	/// BR-001: .r00 is NOT main (old-style continuation)
	#[test]
	fn is_main_rar_rejects_r00() {
		assert!(!is_main_rar("movie.r00"));
	}

	/// BR-001: .r01 is NOT main
	#[test]
	fn is_main_rar_rejects_r01() {
		assert!(!is_main_rar("movie.r01"));
	}

	/// BR-001: .r99 is NOT main
	#[test]
	fn is_main_rar_rejects_r99() {
		assert!(!is_main_rar("movie.r99"));
	}

	/// BR-001: .zip is NOT a RAR file
	#[test]
	fn is_main_rar_rejects_zip() {
		assert!(!is_main_rar("movie.zip"));
	}

	/// BR-001: .mkv is NOT a RAR file
	#[test]
	fn is_main_rar_rejects_mkv() {
		assert!(!is_main_rar("movie.mkv"));
	}

	/// BR-001: .txt is NOT a RAR file
	#[test]
	fn is_main_rar_rejects_txt() {
		assert!(!is_main_rar("readme.txt"));
	}

	// =================================================================
	// BR-002: RAR-Teile für Löschung — is_rar_part()
	// =================================================================

	/// BR-002: .rar matches as RAR part
	#[test]
	fn is_rar_part_matches_rar() {
		assert!(is_rar_part("movie.rar"));
	}

	/// BR-002: .partNN.rar matches as RAR part
	#[test]
	fn is_rar_part_matches_part_rar() {
		assert!(is_rar_part("movie.part01.rar"));
		assert!(is_rar_part("movie.part02.rar"));
	}

	/// BR-002: .r00/.r01 matches as RAR part (old-style)
	#[test]
	fn is_rar_part_matches_r_numbered() {
		assert!(is_rar_part("movie.r00"));
		assert!(is_rar_part("movie.r01"));
		assert!(is_rar_part("movie.r99"));
	}

	/// BR-002: Non-RAR extensions are not RAR parts
	#[test]
	fn is_rar_part_rejects_non_rar() {
		assert!(!is_rar_part("movie.zip"));
		assert!(!is_rar_part("movie.mkv"));
	}

	// =================================================================
	// rar_base_name() — grouping multi-part archives
	// =================================================================

	/// Base name for standalone .rar
	#[test]
	fn base_name_standalone_rar() {
		assert_eq!(rar_base_name("movie.rar"), Some("movie".into()));
	}

	/// Base name for multi-part .partNN.rar
	#[test]
	fn base_name_multipart() {
		assert_eq!(rar_base_name("movie.part01.rar"), Some("movie".into()));
		assert_eq!(rar_base_name("movie.part001.rar"), Some("movie".into()));
	}

	/// Base name for old-style .r00 continuation
	#[test]
	fn base_name_r_numbered() {
		assert_eq!(rar_base_name("movie.r00"), Some("movie".into()));
		assert_eq!(rar_base_name("movie.r01"), Some("movie".into()));
	}

	/// Base name for non-RAR returns None
	#[test]
	fn base_name_non_rar_returns_none() {
		assert_eq!(rar_base_name("movie.zip"), None);
	}

	/// Base name with dots in release name
	#[test]
	fn base_name_with_dots_in_name() {
		assert_eq!(rar_base_name("Movie.2026.720p.part01.rar"), Some("Movie.2026.720p".into()));
	}

	// =================================================================
	// detect_archives() — directory scanning
	// =================================================================

	/// A1: Empty directory → no archives detected
	#[test]
	fn detect_archives_empty_dir() {
		let dir = TempDir::new().unwrap();
		let archives = detect_archives(dir.path());
		assert!(archives.is_empty());
	}

	/// detect_archives finds a single .zip file
	#[test]
	fn detect_archives_finds_zip() {
		let dir = TempDir::new().unwrap();
		fs::write(dir.path().join("files.zip"), b"fake").unwrap();

		let archives = detect_archives(dir.path());
		assert_eq!(archives.len(), 1);
		assert_eq!(archives[0].archive_type, ArchiveType::Zip);
	}

	/// detect_archives finds a standalone .rar
	#[test]
	fn detect_archives_finds_standalone_rar() {
		let dir = TempDir::new().unwrap();
		fs::write(dir.path().join("movie.rar"), b"fake").unwrap();

		let archives = detect_archives(dir.path());
		assert_eq!(archives.len(), 1);
		assert_eq!(archives[0].archive_type, ArchiveType::Rar);
	}

	/// AT-21: Multi-part RAR is grouped into a single DetectedArchive
	#[test]
	fn detect_archives_groups_multipart_rar() {
		let dir = TempDir::new().unwrap();
		fs::write(dir.path().join("archive.part01.rar"), b"fake").unwrap();
		fs::write(dir.path().join("archive.part02.rar"), b"fake").unwrap();
		fs::write(dir.path().join("archive.part03.rar"), b"fake").unwrap();

		let archives = detect_archives(dir.path());
		let rar_archives: Vec<_> = archives.iter().filter(|a| a.archive_type == ArchiveType::Rar).collect();
		assert_eq!(rar_archives.len(), 1, "3 parts should be 1 archive");
		assert_eq!(rar_archives[0].all_parts.len(), 3);
	}

	/// When only continuation parts exist (no part01), no main RAR is detected
	#[test]
	fn detect_archives_skips_when_no_first_part() {
		let dir = TempDir::new().unwrap();
		fs::write(dir.path().join("movie.part02.rar"), b"fake").unwrap();
		fs::write(dir.path().join("movie.part03.rar"), b"fake").unwrap();

		let archives = detect_archives(dir.path());
		let rar_archives: Vec<_> = archives.iter().filter(|a| a.archive_type == ArchiveType::Rar).collect();
		assert_eq!(rar_archives.len(), 0);
	}

	/// Mixed content: finds both ZIP and RAR, ignores non-archives
	#[test]
	fn detect_archives_mixed_content() {
		let dir = TempDir::new().unwrap();
		fs::write(dir.path().join("files.zip"), b"fake").unwrap();
		fs::write(dir.path().join("movie.rar"), b"fake").unwrap();
		fs::write(dir.path().join("readme.txt"), b"text").unwrap();

		let archives = detect_archives(dir.path());
		assert_eq!(archives.len(), 2);
	}

	/// Non-archive files are ignored entirely
	#[test]
	fn detect_archives_ignores_non_archives() {
		let dir = TempDir::new().unwrap();
		fs::write(dir.path().join("movie.mkv"), b"video").unwrap();
		fs::write(dir.path().join("info.nfo"), b"info").unwrap();

		let archives = detect_archives(dir.path());
		assert!(archives.is_empty());
	}

	// =================================================================
	// find_related_rar_parts() — grouping parts for deletion
	// =================================================================

	/// BR-002: Finds old-style .r00/.r01 parts alongside .rar
	#[test]
	fn find_related_includes_old_style_parts() {
		let dir = TempDir::new().unwrap();
		fs::write(dir.path().join("movie.rar"), b"fake").unwrap();
		fs::write(dir.path().join("movie.r00"), b"fake").unwrap();
		fs::write(dir.path().join("movie.r01"), b"fake").unwrap();

		let parts = find_related_rar_parts(&dir.path().join("movie.rar"), dir.path());
		assert_eq!(parts.len(), 3);
	}

	/// BR-002: Finds all .partNN.rar parts
	#[test]
	fn find_related_includes_all_part_numbers() {
		let dir = TempDir::new().unwrap();
		fs::write(dir.path().join("movie.part01.rar"), b"fake").unwrap();
		fs::write(dir.path().join("movie.part02.rar"), b"fake").unwrap();
		fs::write(dir.path().join("movie.part03.rar"), b"fake").unwrap();

		let parts = find_related_rar_parts(&dir.path().join("movie.part01.rar"), dir.path());
		assert_eq!(parts.len(), 3);
	}

	/// Does not include unrelated archives in the same directory
	#[test]
	fn find_related_excludes_unrelated_archives() {
		let dir = TempDir::new().unwrap();
		fs::write(dir.path().join("movie.part01.rar"), b"fake").unwrap();
		fs::write(dir.path().join("movie.part02.rar"), b"fake").unwrap();
		fs::write(dir.path().join("other.rar"), b"fake").unwrap();

		let parts = find_related_rar_parts(&dir.path().join("movie.part01.rar"), dir.path());
		assert_eq!(parts.len(), 2, "other.rar should not be included");
	}
}
