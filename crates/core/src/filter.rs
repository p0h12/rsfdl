//! File exclusion filter for UC-15.
//!
//! Provides glob-based file name matching to exclude unwanted files
//! (e.g. `.nfo`, `.jpg`, samples) from downloads.

use crate::sfdl::models::FileItem;

/// Check if a file name matches any of the exclusion patterns.
/// Glob matching is case-insensitive.
/// Returns `false` if patterns is empty.
pub fn is_excluded(file_name: &str, patterns: &[String]) -> bool {
    if patterns.is_empty() {
        return false;
    }
    let lower = file_name.to_lowercase();
    patterns
        .iter()
        .any(|p| glob_match_simple(&p.to_lowercase(), &lower))
}

/// Compute an exclusion mask for a flat list of FileItems.
/// Returns a Vec<bool> where `true` means the file is excluded.
/// The vector is indexed in the same order as the input slice.
pub fn compute_exclusion_mask(files: &[FileItem], patterns: &[String]) -> Vec<bool> {
    files
        .iter()
        .map(|f| is_excluded(&f.file_name, patterns))
        .collect()
}

/// Simple glob matching supporting `*` (any chars) and `?` (one char).
/// Does NOT support bracket expressions like `[0-9]`.
fn glob_match_simple(pattern: &str, text: &str) -> bool {
    let pat: Vec<char> = pattern.chars().collect();
    let txt: Vec<char> = text.chars().collect();
    glob_match_recursive(&pat, &txt, 0, 0)
}

fn glob_match_recursive(pat: &[char], txt: &[char], pi: usize, ti: usize) -> bool {
    if pi == pat.len() {
        return ti == txt.len();
    }

    match pat[pi] {
        '*' => {
            // Try matching zero or more characters
            for skip in 0..=(txt.len() - ti) {
                if glob_match_recursive(pat, txt, pi + 1, ti + skip) {
                    return true;
                }
            }
            false
        }
        '?' => {
            if ti < txt.len() {
                glob_match_recursive(pat, txt, pi + 1, ti + 1)
            } else {
                false
            }
        }
        c => {
            if ti < txt.len() && txt[ti] == c {
                glob_match_recursive(pat, txt, pi + 1, ti + 1)
            } else {
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sfdl::models::{FileItem, HashType};

    fn make_file(name: &str, size: u64) -> FileItem {
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

    // -------------------------------------------------------
    // AT-24: Datei-Ausschluss per Muster
    // -------------------------------------------------------

    /// Covers AT-24: Files matching exclusion patterns are excluded
    #[test]
    fn test_is_excluded_matches_nfo_extension() {
        let patterns = vec!["*.nfo".into()];
        assert!(is_excluded("info.nfo", &patterns));
    }

    /// Covers AT-24: Files matching exclusion patterns are excluded
    #[test]
    fn test_is_excluded_matches_jpg_extension() {
        let patterns = vec!["*.jpg".into()];
        assert!(is_excluded("cover.jpg", &patterns));
    }

    /// Covers AT-24: Files matching exclusion patterns are excluded
    #[test]
    fn test_is_excluded_matches_sample_wildcard() {
        let patterns = vec!["*sample*".into()];
        assert!(is_excluded("sample.mkv", &patterns));
        assert!(is_excluded("movie-sample-720p.mkv", &patterns));
        assert!(is_excluded("Sample.avi", &patterns)); // case-insensitive
    }

    /// Covers AT-24: Non-matching files are NOT excluded
    #[test]
    fn test_is_excluded_does_not_match_rar() {
        let patterns = vec!["*.nfo".into(), "*.jpg".into(), "*sample*".into()];
        assert!(!is_excluded("movie.rar", &patterns));
    }

    /// Covers AT-24: Multiple patterns, mixed matches
    #[test]
    fn test_is_excluded_multiple_patterns() {
        let patterns = vec!["*.nfo".into(), "*.jpg".into(), "*.txt".into()];
        assert!(is_excluded("info.nfo", &patterns));
        assert!(is_excluded("cover.jpg", &patterns));
        assert!(is_excluded("readme.txt", &patterns));
        assert!(!is_excluded("movie.rar", &patterns));
        assert!(!is_excluded("movie.mkv", &patterns));
    }

    // -------------------------------------------------------
    // BR-001: Case-insensitive matching
    // -------------------------------------------------------

    /// Covers BR-001: Matching is case-insensitive
    #[test]
    fn test_is_excluded_case_insensitive() {
        let patterns = vec!["*.nfo".into()];
        assert!(is_excluded("INFO.NFO", &patterns));
        assert!(is_excluded("Info.Nfo", &patterns));
        assert!(is_excluded("info.nfo", &patterns));
    }

    /// Covers BR-001: Pattern itself can be any case
    #[test]
    fn test_is_excluded_pattern_case_insensitive() {
        let patterns = vec!["*.NFO".into()];
        assert!(is_excluded("info.nfo", &patterns));
    }

    // -------------------------------------------------------
    // A1: Empty patterns → no exclusions
    // -------------------------------------------------------

    /// Covers A1: Empty pattern list excludes nothing
    #[test]
    fn test_is_excluded_empty_patterns() {
        let patterns: Vec<String> = vec![];
        assert!(!is_excluded("info.nfo", &patterns));
        assert!(!is_excluded("movie.rar", &patterns));
    }

    // -------------------------------------------------------
    // BR-002: Glob syntax (* and ?)
    // -------------------------------------------------------

    /// Covers BR-002: Question mark matches exactly one character
    #[test]
    fn test_glob_question_mark() {
        let patterns = vec!["file?.txt".into()];
        assert!(is_excluded("file1.txt", &patterns));
        assert!(is_excluded("fileA.txt", &patterns));
        assert!(!is_excluded("file12.txt", &patterns)); // ? matches exactly 1
        assert!(!is_excluded("file.txt", &patterns)); // ? requires 1 char
    }

    /// Covers BR-002: Star matches zero or more characters
    #[test]
    fn test_glob_star_matches_zero_chars() {
        let patterns = vec!["movie*.rar".into()];
        assert!(is_excluded("movie.rar", &patterns)); // * matches zero chars
        assert!(is_excluded("movie-part1.rar", &patterns)); // * matches multiple chars
    }

    /// Covers BR-002: Exact match (no wildcards)
    #[test]
    fn test_glob_exact_match() {
        let patterns = vec!["thumbs.db".into()];
        assert!(is_excluded("thumbs.db", &patterns));
        assert!(!is_excluded("thumbs.db.bak", &patterns));
        assert!(!is_excluded("other.db", &patterns));
    }

    // -------------------------------------------------------
    // compute_exclusion_mask tests
    // -------------------------------------------------------

    /// Covers AT-24: compute_exclusion_mask returns correct mask for mixed files
    #[test]
    fn test_compute_exclusion_mask_mixed_files() {
        let files = vec![
            make_file("movie.rar", 1_000_000),
            make_file("info.nfo", 1_000),
            make_file("cover.jpg", 50_000),
            make_file("sample.mkv", 500_000),
        ];
        let patterns = vec!["*.nfo".into(), "*.jpg".into(), "*sample*".into()];

        let mask = compute_exclusion_mask(&files, &patterns);

        assert_eq!(mask, vec![false, true, true, true]);
    }

    /// Covers AT-24: Only movie.rar should remain for download
    #[test]
    fn test_compute_exclusion_mask_selected_size() {
        let files = vec![
            make_file("movie.rar", 1_000_000),
            make_file("info.nfo", 1_000),
            make_file("cover.jpg", 50_000),
            make_file("sample.mkv", 500_000),
        ];
        let patterns = vec!["*.nfo".into(), "*.jpg".into(), "*sample*".into()];

        let mask = compute_exclusion_mask(&files, &patterns);

        // Calculate size of non-excluded files
        let selected_size: u64 = files
            .iter()
            .zip(mask.iter())
            .filter(|&(_, &excluded)| !excluded)
            .map(|(f, _)| f.file_size)
            .sum();

        assert_eq!(selected_size, 1_000_000); // only movie.rar
    }

    /// Covers A1: Empty patterns → all false (nothing excluded)
    #[test]
    fn test_compute_exclusion_mask_no_patterns() {
        let files = vec![
            make_file("movie.rar", 1_000_000),
            make_file("info.nfo", 1_000),
        ];
        let patterns: Vec<String> = vec![];

        let mask = compute_exclusion_mask(&files, &patterns);

        assert_eq!(mask, vec![false, false]);
    }

    /// Covers A2: All files match patterns → all true
    #[test]
    fn test_compute_exclusion_mask_all_excluded() {
        let files = vec![make_file("info.nfo", 1_000), make_file("cover.jpg", 50_000)];
        let patterns = vec!["*.nfo".into(), "*.jpg".into()];

        let mask = compute_exclusion_mask(&files, &patterns);

        assert_eq!(mask, vec![true, true]);
    }

    /// Empty file list → empty mask
    #[test]
    fn test_compute_exclusion_mask_empty_files() {
        let files: Vec<FileItem> = vec![];
        let patterns = vec!["*.nfo".into()];

        let mask = compute_exclusion_mask(&files, &patterns);

        assert!(mask.is_empty());
    }

    // -------------------------------------------------------
    // Edge cases
    // -------------------------------------------------------

    /// File name with dots in various positions
    #[test]
    fn test_is_excluded_file_with_multiple_dots() {
        let patterns = vec!["*.nfo".into()];
        assert!(is_excluded("movie.release.2026.nfo", &patterns));
        assert!(!is_excluded("movie.release.2026.rar", &patterns));
    }

    /// Pattern with no wildcard must match exactly
    #[test]
    fn test_is_excluded_no_wildcard_exact() {
        let patterns = vec!["Thumbs.db".into()];
        assert!(is_excluded("Thumbs.db", &patterns));
        assert!(is_excluded("thumbs.db", &patterns)); // case-insensitive
        assert!(!is_excluded("Thumbs.db.bak", &patterns));
    }

    /// Star-only pattern matches everything
    #[test]
    fn test_is_excluded_star_only_matches_all() {
        let patterns = vec!["*".into()];
        assert!(is_excluded("anything.rar", &patterns));
        assert!(is_excluded("", &patterns));
    }

    /// Empty file name
    #[test]
    fn test_is_excluded_empty_filename() {
        let patterns = vec!["*.nfo".into()];
        assert!(!is_excluded("", &patterns));
    }
}
