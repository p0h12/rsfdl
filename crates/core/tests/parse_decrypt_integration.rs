use rsfdl_core::container::{DecryptionStatus, LoadResult, load_sfdl};
use rsfdl_core::sfdl::models::SfdlFile;
use rsfdl_core::sfdl::parser::parse_sfdl;

const ENCRYPTED_V3: &str = include_str!("fixtures/encrypted_v3.sfdl");
const UNENCRYPTED_V3: &str = include_str!("fixtures/unencrypted_v3.sfdl");
const BULKFOLDER_V3: &str = include_str!("fixtures/bulkfolder_v3.sfdl");
const ENCRYPTED_BULKFOLDER_V3: &str = include_str!("fixtures/encrypted_bulkfolder_v3.sfdl");

// =======================================================
// SFDL-002 | Main Success: Parse + decrypt with correct password
// =======================================================

/// SFDL-002 | Main Success: Decrypt encrypted v3 FileList container.
#[test]
fn sfdl002_parse_and_decrypt_v3() {
	let SfdlFile::Encrypted(enc) = parse_sfdl(ENCRYPTED_V3).unwrap() else {
		panic!("expected Encrypted");
	};

	let container = enc.decrypt("test").unwrap();

	assert_eq!(container.description, "Test.Release.2026.1080p");
	assert_eq!(container.uploader, "testuser");
	assert_eq!(container.connection.host, "ftp.example.com");
	assert_eq!(container.connection.username, "ftpuser");
	assert_eq!(container.connection.password, "ftppass");
	assert_eq!(container.connection.port, 21);

	let pkg = &container.packages[0];
	assert_eq!(pkg.name, "Package1");
	assert_eq!(pkg.file_list.len(), 2);
	assert_eq!(pkg.file_list[0].file_name, "movie.part1.rar");
	assert_eq!(pkg.file_list[0].full_path, "/releases/test/movie.part1.rar");
	assert_eq!(pkg.file_list[1].file_name, "movie.part2.rar");
}

/// SFDL-002 | Main Success: Decrypt encrypted v3 BulkFolder container.
#[test]
fn sfdl002_parse_and_decrypt_bulkfolder_v3() {
	let SfdlFile::Encrypted(enc) = parse_sfdl(ENCRYPTED_BULKFOLDER_V3).unwrap() else {
		panic!("expected Encrypted");
	};

	let container = enc.decrypt("test").unwrap();

	assert_eq!(container.description, "BulkFolder.Test.2026");
	assert_eq!(container.connection.host, "ftp.example.com");

	let pkg = &container.packages[0];
	assert_eq!(pkg.name, "BulkPkg1");
	assert!(pkg.bulk_folder_mode);
	assert_eq!(pkg.bulk_folder_list.len(), 2);
	assert_eq!(pkg.bulk_folder_list[0].bulk_folder_path, "/releases/movie/");
	assert_eq!(pkg.bulk_folder_list[1].bulk_folder_path, "/releases/extras/");
}

// =======================================================
// SFDL-002 | A2: Wrong password
// =======================================================

/// SFDL-002 | A2: Wrong password on FileList container.
#[test]
fn sfdl002_wrong_password_fails() {
	let SfdlFile::Encrypted(enc) = parse_sfdl(ENCRYPTED_V3).unwrap() else {
		panic!("expected Encrypted");
	};
	let result = enc.decrypt("wrong_password");
	assert!(result.is_err());
}

/// SFDL-002 | A2: Wrong password on BulkFolder container.
#[test]
fn sfdl002_wrong_password_bulkfolder_fails() {
	let SfdlFile::Encrypted(enc) = parse_sfdl(ENCRYPTED_BULKFOLDER_V3).unwrap() else {
		panic!("expected Encrypted");
	};
	let result = enc.decrypt("wrong_password");
	assert!(result.is_err());
}

// =======================================================
// SFDL-002 | Unencrypted parsing
// =======================================================

/// SFDL-002 | Edge: Unencrypted container parses as Decrypted variant.
#[test]
fn sfdl002_decrypt_unencrypted_is_noop() {
	let SfdlFile::Decrypted(container) = parse_sfdl(UNENCRYPTED_V3).unwrap() else {
		panic!("expected Decrypted");
	};
	// Host is plaintext and accessible directly
	assert_eq!(container.connection.host, "ftp.example.com");
}

// =======================================================
// SFDL-002 | load_sfdl flow: auto-decrypt orchestration
// =======================================================

/// SFDL-002 | Main Success (Step 1-2): Auto-password finds correct password.
#[test]
fn sfdl002_load_sfdl_auto_decrypt() {
	let auto_passwords = vec!["wrong1".into(), "test".into(), "wrong2".into()];
	let result = load_sfdl(ENCRYPTED_V3, &auto_passwords).unwrap();

	let LoadResult::Ready(container, status) = result else {
		panic!("expected Ready");
	};
	assert!(matches!(status, DecryptionStatus::AutoDecrypted { .. }));
	if let DecryptionStatus::AutoDecrypted { password } = &status {
		assert_eq!(password, "test");
	}
	assert_eq!(container.connection.host, "ftp.example.com");
}

/// SFDL-002 | A1: No auto-password matches → NeedsPassword.
#[test]
fn sfdl002_load_sfdl_no_auto_match() {
	let auto_passwords = vec!["wrong1".into(), "wrong2".into()];
	let result = load_sfdl(ENCRYPTED_V3, &auto_passwords).unwrap();
	assert!(matches!(result, LoadResult::NeedsPassword(_)));
}

/// SFDL-002 | A1: Empty auto-password list → NeedsPassword.
#[test]
fn sfdl002_load_sfdl_empty_auto_list() {
	let result = load_sfdl(ENCRYPTED_V3, &[]).unwrap();
	assert!(matches!(result, LoadResult::NeedsPassword(_)));
}

/// SFDL-002 | Edge: Unencrypted container → Ready(NotEncrypted).
#[test]
fn sfdl002_load_sfdl_not_encrypted() {
	let result = load_sfdl(UNENCRYPTED_V3, &["test".into()]).unwrap();
	let LoadResult::Ready(_, status) = result else {
		panic!("expected Ready");
	};
	assert!(matches!(status, DecryptionStatus::NotEncrypted));
}

// =======================================================
// SFDL-001 | Unencrypted BulkFolder (structural test)
// =======================================================

/// SFDL-001 | Main Success: Unencrypted BulkFolder parses correctly.
#[test]
fn sfdl001_parse_bulkfolder_v3() {
	let SfdlFile::Decrypted(container) = parse_sfdl(BULKFOLDER_V3).unwrap() else {
		panic!("expected Decrypted");
	};

	assert_eq!(container.description, "BulkFolder.Test.2026");

	let pkg = &container.packages[0];
	assert_eq!(pkg.name, "BulkPkg1");
	assert!(pkg.bulk_folder_mode);
	assert!(pkg.file_list.is_empty());
	assert_eq!(pkg.bulk_folder_list.len(), 2);
}
