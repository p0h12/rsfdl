//! RAR extraction via the `unrar` crate (UC-14).

use std::path::Path;

use unrar::error::Code;
use unrar::Archive;

use crate::error::ExtractionError;

/// Extract a RAR archive (including multi-part) to the destination directory.
/// For multi-part archives, `main_file` should be the first part (.rar or .part01.rar).
///
/// Calls `on_progress` with a percentage (0–100) as files are extracted.
pub fn extract_rar(
    main_file: &Path,
    dest_dir: &Path,
    mut on_progress: impl FnMut(u8),
) -> Result<(), ExtractionError> {
    // First pass: count entries for progress calculation
    let entry_count = count_entries(main_file)?;

    // Second pass: extract
    let mut archive = Archive::new(main_file)
        .open_for_processing()
        .map_err(|e| map_unrar_error(&e))?;

    let mut extracted = 0u32;
    while let Some(header) = archive.read_header().map_err(|e| map_unrar_error(&e))? {
        let entry = header.entry();
        archive = if entry.is_file() {
            header
                .extract_with_base(dest_dir)
                .map_err(|e| map_unrar_error(&e))?
        } else {
            header.skip().map_err(|e| map_unrar_error(&e))?
        };

        extracted += 1;
        if entry_count > 0 {
            let percent = ((extracted as f64 / entry_count as f64) * 100.0).min(100.0) as u8;
            on_progress(percent);
        }
    }

    on_progress(100);
    Ok(())
}

fn count_entries(main_file: &Path) -> Result<u32, ExtractionError> {
    let mut archive = Archive::new(main_file)
        .open_for_processing()
        .map_err(|e| map_unrar_error(&e))?;

    let mut count = 0u32;
    while let Some(header) = archive.read_header().map_err(|e| map_unrar_error(&e))? {
        count += 1;
        archive = header.skip().map_err(|e| map_unrar_error(&e))?;
    }
    Ok(count)
}

fn map_unrar_error(e: &unrar::error::UnrarError) -> ExtractionError {
    match e.code {
        Code::MissingPassword | Code::BadPassword => ExtractionError::PasswordProtected,
        _ => ExtractionError::Rar(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // =================================================================
    // A2: Error handling for invalid/missing RAR files
    // =================================================================

    /// A2: Nonexistent file returns ExtractionError::Rar
    #[test]
    fn extract_rar_nonexistent_file_errors() {
        let dest = TempDir::new().unwrap();
        let result = extract_rar(Path::new("/nonexistent/fake.rar"), dest.path(), |_| {});
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ExtractionError::Rar(_)));
    }

    /// A2: Invalid RAR data returns ExtractionError::Rar
    #[test]
    fn extract_rar_invalid_data_errors() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("bad.rar"), b"not a rar archive").unwrap();

        let dest = TempDir::new().unwrap();
        let result = extract_rar(&dir.path().join("bad.rar"), dest.path(), |_| {});
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ExtractionError::Rar(_)));
    }

    /// A2: Empty file returns ExtractionError::Rar
    #[test]
    fn extract_rar_empty_file_errors() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("empty.rar"), b"").unwrap();

        let dest = TempDir::new().unwrap();
        let result = extract_rar(&dir.path().join("empty.rar"), dest.path(), |_| {});
        assert!(result.is_err());
    }

    // =================================================================
    // A3: map_unrar_error — password error mapping
    // =================================================================

    /// A3: MissingPassword maps to ExtractionError::PasswordProtected
    #[test]
    fn map_error_missing_password() {
        let err = unrar::error::UnrarError {
            code: Code::MissingPassword,
            when: unrar::error::When::Process,
        };
        let mapped = map_unrar_error(&err);
        assert!(matches!(mapped, ExtractionError::PasswordProtected));
    }

    /// A3: BadPassword maps to ExtractionError::PasswordProtected
    #[test]
    fn map_error_bad_password() {
        let err = unrar::error::UnrarError {
            code: Code::BadPassword,
            when: unrar::error::When::Process,
        };
        let mapped = map_unrar_error(&err);
        assert!(matches!(mapped, ExtractionError::PasswordProtected));
    }

    /// Other errors map to ExtractionError::Rar
    #[test]
    fn map_error_other_codes() {
        let err = unrar::error::UnrarError {
            code: Code::BadData,
            when: unrar::error::When::Process,
        };
        let mapped = map_unrar_error(&err);
        assert!(matches!(mapped, ExtractionError::Rar(_)));
    }
}
