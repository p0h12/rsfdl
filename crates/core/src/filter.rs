//! DL-002: Exclusion pattern matching.
//!
//! Provides glob-based file name matching to exclude unwanted files
//! (e.g. `.nfo`, `.jpg`, samples) from downloads.
//!
//! Core functions:
//! - [`is_excluded`] — check a single filename against patterns
//! - [`resolve_patterns`] — merge settings + CLI patterns per BR-DL-005

/// BR-DL-003: Check if a file name matches any of the exclusion patterns.
///
/// Glob matching is case-insensitive. Supports `*` (zero or more chars)
/// and `?` (exactly one char). Patterns are matched against the filename
/// only, not the full path.
pub fn is_excluded(file_name: &str, patterns: &[String]) -> bool {
	if patterns.is_empty() || file_name.is_empty() {
		return false;
	}
	let lower = file_name.to_lowercase();
	patterns.iter().any(|p| glob_match(&p.to_lowercase(), &lower))
}

/// BR-DL-005: Resolve the effective exclusion patterns from settings and CLI overrides.
///
/// - `no_exclude=true`: returns empty list (all patterns disabled)
/// - Otherwise: settings patterns + additional CLI patterns merged
pub fn resolve_patterns(settings_patterns: &[String], cli_extra: &[String], no_exclude: bool) -> Vec<String> {
	if no_exclude {
		return Vec::new();
	}
	let mut patterns = settings_patterns.to_vec();
	for p in cli_extra {
		let trimmed = p.trim().to_string();
		if !trimmed.is_empty() && !patterns.contains(&trimmed) {
			patterns.push(trimmed);
		}
	}
	patterns
}

/// Simple glob matching supporting `*` (any chars) and `?` (one char).
///
/// Does NOT support bracket expressions like `[0-9]`.
fn glob_match(pattern: &str, text: &str) -> bool {
	let pat: Vec<char> = pattern.chars().collect();
	let txt: Vec<char> = text.chars().collect();
	glob_dp(&pat, &txt)
}

/// Dynamic-programming glob match (avoids stack overflow on long inputs).
fn glob_dp(pat: &[char], txt: &[char]) -> bool {
	let m = pat.len();
	let n = txt.len();

	// dp[i][j] = does pat[0..i] match txt[0..j]?
	let mut dp = vec![vec![false; n + 1]; m + 1];
	dp[0][0] = true;

	// Leading *'s can match empty text
	for i in 1..=m {
		if pat[i - 1] == '*' {
			dp[i][0] = dp[i - 1][0];
		} else {
			break;
		}
	}

	for i in 1..=m {
		for j in 1..=n {
			match pat[i - 1] {
				'*' => {
					// * matches zero chars (dp[i-1][j]) or one more char (dp[i][j-1])
					dp[i][j] = dp[i - 1][j] || dp[i][j - 1];
				}
				'?' => {
					dp[i][j] = dp[i - 1][j - 1];
				}
				c => {
					dp[i][j] = dp[i - 1][j - 1] && txt[j - 1] == c;
				}
			}
		}
	}

	dp[m][n]
}

#[cfg(test)]
mod tests {
	use super::*;

	// =======================================================
	// DL-002 | Main Success: Pattern matching
	// =======================================================

	/// DL-002 | Main Success: *.nfo matches .nfo files.
	#[test]
	fn dl002_matches_nfo() {
		assert!(is_excluded("info.nfo", &["*.nfo".into()]));
	}

	/// DL-002 | Main Success: *.jpg matches .jpg files.
	#[test]
	fn dl002_matches_jpg() {
		assert!(is_excluded("cover.jpg", &["*.jpg".into()]));
	}

	/// DL-002 | Main Success: *sample* matches sample anywhere in name.
	#[test]
	fn dl002_matches_sample_wildcard() {
		let p = vec!["*sample*".into()];
		assert!(is_excluded("sample.mkv", &p));
		assert!(is_excluded("movie-sample-720p.mkv", &p));
		assert!(is_excluded("Sample.avi", &p)); // case-insensitive
	}

	/// DL-002 | Main Success: Non-matching files are NOT excluded.
	#[test]
	fn dl002_does_not_match_rar() {
		let p = vec!["*.nfo".into(), "*.jpg".into(), "*sample*".into()];
		assert!(!is_excluded("movie.rar", &p));
	}

	/// DL-002 | Main Success: Multiple patterns, mixed results.
	#[test]
	fn dl002_multiple_patterns() {
		let p = vec!["*.nfo".into(), "*.jpg".into(), "*.txt".into()];
		assert!(is_excluded("info.nfo", &p));
		assert!(is_excluded("cover.jpg", &p));
		assert!(is_excluded("readme.txt", &p));
		assert!(!is_excluded("movie.rar", &p));
		assert!(!is_excluded("movie.mkv", &p));
	}

	// =======================================================
	// DL-002 | BR-DL-003: Glob syntax
	// =======================================================

	/// DL-002 | BR-DL-003: Case-insensitive matching.
	#[test]
	fn dl002_case_insensitive() {
		let p = vec!["*.nfo".into()];
		assert!(is_excluded("INFO.NFO", &p));
		assert!(is_excluded("Info.Nfo", &p));
		assert!(is_excluded("info.nfo", &p));
	}

	/// DL-002 | BR-DL-003: Pattern itself can be any case.
	#[test]
	fn dl002_pattern_case_insensitive() {
		assert!(is_excluded("info.nfo", &["*.NFO".into()]));
	}

	/// DL-002 | BR-DL-003: ? matches exactly one character.
	#[test]
	fn dl002_question_mark() {
		let p = vec!["file?.txt".into()];
		assert!(is_excluded("file1.txt", &p));
		assert!(is_excluded("fileA.txt", &p));
		assert!(!is_excluded("file12.txt", &p));
		assert!(!is_excluded("file.txt", &p));
	}

	/// DL-002 | BR-DL-003: * matches zero or more characters.
	#[test]
	fn dl002_star_matches_zero() {
		let p = vec!["movie*.rar".into()];
		assert!(is_excluded("movie.rar", &p));
		assert!(is_excluded("movie-part1.rar", &p));
	}

	/// DL-002 | BR-DL-003: Exact match without wildcards.
	#[test]
	fn dl002_exact_match() {
		let p = vec!["thumbs.db".into()];
		assert!(is_excluded("thumbs.db", &p));
		assert!(!is_excluded("thumbs.db.bak", &p));
		assert!(!is_excluded("other.db", &p));
	}

	/// DL-002 | BR-DL-003: Multiple dots in filename.
	#[test]
	fn dl002_multiple_dots() {
		assert!(is_excluded("movie.release.2026.nfo", &["*.nfo".into()]));
		assert!(!is_excluded("movie.release.2026.rar", &["*.nfo".into()]));
	}

	/// DL-002 | BR-DL-003: * alone matches everything.
	#[test]
	fn dl002_star_only() {
		let p = vec!["*".into()];
		assert!(is_excluded("anything.rar", &p));
	}

	// =======================================================
	// DL-002 | A1: No patterns
	// =======================================================

	/// DL-002 | A1: Empty pattern list excludes nothing.
	#[test]
	fn dl002_empty_patterns() {
		assert!(!is_excluded("info.nfo", &[]));
		assert!(!is_excluded("movie.rar", &[]));
	}

	/// DL-002 | A1: Empty filename is never excluded.
	#[test]
	fn dl002_empty_filename() {
		assert!(!is_excluded("", &["*.nfo".into()]));
	}

	// =======================================================
	// DL-002 | BR-DL-005: resolve_patterns (CLI override)
	// =======================================================

	/// DL-002 | BR-DL-005: Settings patterns used when no CLI override.
	#[test]
	fn dl002_resolve_settings_only() {
		let settings = vec!["*.nfo".into(), "*.jpg".into()];
		let result = resolve_patterns(&settings, &[], false);
		assert_eq!(result, vec!["*.nfo", "*.jpg"]);
	}

	/// DL-002 | BR-DL-005: CLI --exclude adds to settings patterns.
	#[test]
	fn dl002_resolve_cli_adds() {
		let settings = vec!["*.nfo".into()];
		let cli = vec!["*.txt".into()];
		let result = resolve_patterns(&settings, &cli, false);
		assert_eq!(result, vec!["*.nfo", "*.txt"]);
	}

	/// DL-002 | BR-DL-005: CLI --exclude does not duplicate existing patterns.
	#[test]
	fn dl002_resolve_no_duplicates() {
		let settings = vec!["*.nfo".into()];
		let cli = vec!["*.nfo".into(), "*.txt".into()];
		let result = resolve_patterns(&settings, &cli, false);
		assert_eq!(result, vec!["*.nfo", "*.txt"]);
	}

	/// DL-002 | BR-DL-005: --no-exclude disables all patterns.
	#[test]
	fn dl002_resolve_no_exclude() {
		let settings = vec!["*.nfo".into(), "*.jpg".into()];
		let cli = vec!["*.txt".into()];
		let result = resolve_patterns(&settings, &cli, true);
		assert!(result.is_empty());
	}

	/// DL-002 | BR-DL-005: Empty CLI extra patterns don't change settings.
	#[test]
	fn dl002_resolve_empty_cli() {
		let settings = vec!["*.nfo".into()];
		let result = resolve_patterns(&settings, &[], false);
		assert_eq!(result, vec!["*.nfo"]);
	}

	/// DL-002 | BR-DL-005: Whitespace-only CLI patterns are ignored.
	#[test]
	fn dl002_resolve_whitespace_cli_ignored() {
		let settings = vec!["*.nfo".into()];
		let cli = vec!["  ".into(), "".into(), "*.txt".into()];
		let result = resolve_patterns(&settings, &cli, false);
		assert_eq!(result, vec!["*.nfo", "*.txt"]);
	}

	// =======================================================
	// Edge cases / regression
	// =======================================================

	/// DL-002 | Edge: Pattern with no wildcard must match exactly (case-insensitive).
	#[test]
	fn dl002_no_wildcard_case_insensitive() {
		assert!(is_excluded("Thumbs.db", &["Thumbs.db".into()]));
		assert!(is_excluded("thumbs.db", &["Thumbs.db".into()]));
		assert!(!is_excluded("Thumbs.db.bak", &["Thumbs.db".into()]));
	}

	/// DL-002 | Edge: Long filename with many * wildcards (DP avoids stack overflow).
	#[test]
	fn dl002_long_pattern_no_stack_overflow() {
		let long_name = "a".repeat(200) + ".nfo";
		assert!(is_excluded(&long_name, &["*.nfo".into()]));
	}
}
