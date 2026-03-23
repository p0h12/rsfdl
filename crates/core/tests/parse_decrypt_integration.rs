use rsfdl_core::sfdl::crypto::decrypt_container;
use rsfdl_core::sfdl::parser::parse_sfdl;

const ENCRYPTED_V3: &str = include_str!("fixtures/encrypted_v3.sfdl");
const UNENCRYPTED_V3: &str = include_str!("fixtures/unencrypted_v3.sfdl");
const BULKFOLDER_V3: &str = include_str!("fixtures/bulkfolder_v3.sfdl");
const ENCRYPTED_BULKFOLDER_V3: &str = include_str!("fixtures/encrypted_bulkfolder_v3.sfdl");

/// AT-03: Parse encrypted v3 file, then decrypt with correct password.
/// Result should match the unencrypted fixture.
#[test]
fn parse_and_decrypt_v3() {
	let mut container = parse_sfdl(ENCRYPTED_V3).unwrap();
	assert!(container.encrypted);

	decrypt_container(&mut container, "test").unwrap();

	assert!(!container.encrypted);
	assert_eq!(container.description, "Test.Release.2026.1080p");
	assert_eq!(container.uploader, "testuser");
	assert_eq!(container.connection.host, "ftp.example.com");
	assert_eq!(container.connection.username, "ftpuser");
	assert_eq!(container.connection.password, "ftppass");
	assert_eq!(container.connection.port, 21);

	// Package
	assert_eq!(container.packages.len(), 1);
	let pkg = &container.packages[0];
	assert_eq!(pkg.name, "Package1");
	assert_eq!(pkg.file_list.len(), 2);

	// Files
	assert_eq!(pkg.file_list[0].file_name, "movie.part1.rar");
	assert_eq!(pkg.file_list[0].full_path, "/releases/test/movie.part1.rar");
	assert_eq!(pkg.file_list[1].file_name, "movie.part2.rar");
}

/// AT-04: Parse encrypted file, then try decrypt with wrong password.
#[test]
fn decrypt_with_wrong_password_fails() {
	let mut container = parse_sfdl(ENCRYPTED_V3).unwrap();
	let result = decrypt_container(&mut container, "wrong_password");
	assert!(result.is_err());
}

/// Parse unencrypted v3 BulkFolder file.
#[test]
fn parse_bulkfolder_v3() {
	let container = parse_sfdl(BULKFOLDER_V3).unwrap();

	assert!(!container.encrypted);
	assert_eq!(container.description, "BulkFolder.Test.2026");

	let pkg = &container.packages[0];
	assert_eq!(pkg.name, "BulkPkg1");
	assert!(pkg.bulk_folder_mode);
	assert!(pkg.file_list.is_empty());
	assert_eq!(pkg.bulk_folder_list.len(), 2);
	assert_eq!(pkg.bulk_folder_list[0].bulk_folder_path, "/releases/movie/");
	assert_eq!(pkg.bulk_folder_list[1].bulk_folder_path, "/releases/extras/");
}

/// Parse encrypted v3 BulkFolder file, then decrypt with correct password.
#[test]
fn parse_and_decrypt_bulkfolder_v3() {
	let mut container = parse_sfdl(ENCRYPTED_BULKFOLDER_V3).unwrap();
	assert!(container.encrypted);

	decrypt_container(&mut container, "test").unwrap();

	assert!(!container.encrypted);
	assert_eq!(container.description, "BulkFolder.Test.2026");
	assert_eq!(container.uploader, "testuser");
	assert_eq!(container.connection.host, "ftp.example.com");
	assert_eq!(container.connection.username, "ftpuser");
	assert_eq!(container.connection.password, "ftppass");

	let pkg = &container.packages[0];
	assert_eq!(pkg.name, "BulkPkg1");
	assert!(pkg.bulk_folder_mode);
	assert!(pkg.file_list.is_empty());
	assert_eq!(pkg.bulk_folder_list.len(), 2);
	assert_eq!(pkg.bulk_folder_list[0].bulk_folder_path, "/releases/movie/");
	assert_eq!(pkg.bulk_folder_list[0].package_name, "BulkPkg1");
	assert_eq!(pkg.bulk_folder_list[1].bulk_folder_path, "/releases/extras/");
	assert_eq!(pkg.bulk_folder_list[1].package_name, "BulkPkg1");
}

/// Encrypted v3 BulkFolder with wrong password fails.
#[test]
fn decrypt_bulkfolder_with_wrong_password_fails() {
	let mut container = parse_sfdl(ENCRYPTED_BULKFOLDER_V3).unwrap();
	let result = decrypt_container(&mut container, "wrong_password");
	assert!(result.is_err());
}

/// Unencrypted container: decrypt_container is a no-op.
#[test]
fn decrypt_unencrypted_is_noop() {
	let mut container = parse_sfdl(UNENCRYPTED_V3).unwrap();
	let original_host = container.connection.host.clone();

	decrypt_container(&mut container, "anything").unwrap();

	assert_eq!(container.connection.host, original_host);
}
