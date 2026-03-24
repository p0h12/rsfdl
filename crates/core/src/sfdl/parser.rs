use quick_xml::de::from_str;
use quick_xml::se::to_string as xml_to_string;
use serde::{Deserialize, Serialize};

use crate::error::SfdlError;
use crate::sfdl::models::*;

/// Detects the SFDL version from raw XML content.
pub fn detect_version(xml: &str) -> Result<SfdlVersion, SfdlError> {
	if xml.contains("<ContainerVersion>") {
		Ok(SfdlVersion::V3)
	} else if xml.contains("<SFDLFileVersion>") {
		Ok(SfdlVersion::V2)
	} else {
		Err(SfdlError::ParseError("Cannot detect SFDL version: no ContainerVersion or SFDLFileVersion element found".into()))
	}
}

/// Parses an SFDL file (v2 or v3) into a unified SfdlContainer.
pub fn parse_sfdl(xml: &str) -> Result<SfdlContainer, SfdlError> {
	let version = detect_version(xml)?;
	match version {
		SfdlVersion::V3 => parse_v3(xml),
		SfdlVersion::V2 => parse_v2(xml),
	}
}

// --- v3 parsing ---

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "Container")]
struct RawContainerV3 {
	#[serde(rename = "ContainerVersion", default)]
	container_version: u32,
	#[serde(rename = "Description", default)]
	description: String,
	#[serde(rename = "Uploader", default)]
	uploader: String,
	#[serde(rename = "Encrypted", default)]
	encrypted: bool,
	#[serde(rename = "MaxDownloadThreads", default = "default_threads")]
	max_download_threads: u32,
	#[serde(rename = "Connection")]
	connection: RawConnectionV3,
	#[serde(rename = "Packages")]
	packages: RawPackagesV3,
}

fn default_threads() -> u32 {
	3
}

#[derive(Debug, Deserialize, Serialize)]
struct RawConnectionV3 {
	#[serde(rename = "Host", default)]
	host: String,
	#[serde(rename = "Port", default = "default_port")]
	port: u16,
	#[serde(rename = "Username", default)]
	username: String,
	#[serde(rename = "Password", default)]
	password: String,
	#[serde(rename = "AuthRequired", default)]
	auth_required: bool,
	#[serde(rename = "DataConnectionType", default)]
	data_connection_type: FtpDataConnectionType,
	#[serde(rename = "DataType", default)]
	data_type: FtpDataType,
	#[serde(rename = "CharacterEncoding", default)]
	character_encoding: CharacterEncoding,
	#[serde(rename = "SSLProtocol", default)]
	ssl_protocol: SslProtocol,
	#[serde(rename = "ConnectTimeout", default = "default_timeout")]
	connect_timeout: u32,
	#[serde(rename = "CommandTimeout", default = "default_timeout")]
	command_timeout: u32,
}

fn default_port() -> u16 {
	21
}
fn default_timeout() -> u32 {
	10
}
fn default_true() -> bool {
	true
}

#[derive(Debug, Deserialize, Serialize)]
struct RawPackagesV3 {
	#[serde(rename = "Package", default)]
	packages: Vec<RawPackageV3>,
}

#[derive(Debug, Deserialize, Serialize)]
struct RawPackageV3 {
	#[serde(rename = "Name", default)]
	name: String,
	#[serde(rename = "BulkFolderMode", default)]
	bulk_folder_mode: bool,
	#[serde(rename = "FileList", default)]
	file_list: Option<RawFileListV3>,
	#[serde(rename = "BulkFolderList", default)]
	bulk_folder_list: Option<RawBulkFolderListV3>,
}

#[derive(Debug, Deserialize, Serialize)]
struct RawFileListV3 {
	#[serde(rename = "FileItem", default)]
	items: Vec<RawFileItemV3>,
}

#[derive(Debug, Deserialize, Serialize)]
struct RawFileItemV3 {
	#[serde(rename = "FileName", default)]
	file_name: String,
	#[serde(rename = "DirectoryRoot", default)]
	directory_root: String,
	#[serde(rename = "DirectoryPath", default)]
	directory_path: String,
	#[serde(rename = "FullPath", default)]
	full_path: String,
	#[serde(rename = "FileSize", default)]
	file_size: u64,
	#[serde(rename = "HashType", default)]
	hash_type: HashType,
	#[serde(rename = "FileHash", default)]
	file_hash: String,
	#[serde(rename = "PackageName", default)]
	package_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct RawBulkFolderListV3 {
	#[serde(rename = "BulkFolder", default)]
	items: Vec<RawBulkFolderV3>,
}

#[derive(Debug, Deserialize, Serialize)]
struct RawBulkFolderV3 {
	#[serde(rename = "BulkFolderPath", default)]
	bulk_folder_path: String,
	#[serde(rename = "PackageName", default)]
	package_name: String,
}

fn parse_v3(xml: &str) -> Result<SfdlContainer, SfdlError> {
	let raw: RawContainerV3 = from_str(xml).map_err(|e| SfdlError::ParseError(e.to_string()))?;

	Ok(SfdlContainer {
		container_version: raw.container_version,
		version: SfdlVersion::V3,
		description: raw.description,
		uploader: raw.uploader,
		encrypted: raw.encrypted,
		max_download_threads: raw.max_download_threads,
		connection: Connection {
			host: raw.connection.host,
			port: raw.connection.port,
			username: raw.connection.username,
			password: raw.connection.password,
			auth_required: raw.connection.auth_required,
			data_connection_type: raw.connection.data_connection_type,
			data_type: raw.connection.data_type,
			character_encoding: raw.connection.character_encoding,
			ssl_protocol: raw.connection.ssl_protocol,
			connect_timeout: raw.connection.connect_timeout,
			command_timeout: raw.connection.command_timeout,
		},
		packages: raw
			.packages
			.packages
			.into_iter()
			.map(|p| Package {
				name: p.name,
				bulk_folder_mode: p.bulk_folder_mode,
				file_list: p
					.file_list
					.map(|fl| {
						fl.items
							.into_iter()
							.map(|f| FileItem {
								file_name: f.file_name,
								directory_root: f.directory_root,
								directory_path: f.directory_path,
								full_path: f.full_path,
								file_size: f.file_size,
								hash_type: f.hash_type,
								file_hash: f.file_hash,
								package_name: f.package_name,
							})
							.collect()
					})
					.unwrap_or_default(),
				bulk_folder_list: p
					.bulk_folder_list
					.map(|bl| {
						bl.items
							.into_iter()
							.map(|b| BulkFolder {
								bulk_folder_path: b.bulk_folder_path,
								package_name: b.package_name,
							})
							.collect()
					})
					.unwrap_or_default(),
			})
			.collect(),
	})
}

// --- v3 serialization ---

/// Serializes an SfdlContainer to SFDL v3 XML.
pub fn serialize_v3(container: &SfdlContainer) -> Result<String, SfdlError> {
	let raw = to_raw_v3(container);
	let xml_body = xml_to_string(&raw).map_err(|e| SfdlError::SerializeError(e.to_string()))?;
	Ok(format!("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n{}", xml_body))
}

fn to_raw_v3(c: &SfdlContainer) -> RawContainerV3 {
	RawContainerV3 {
		container_version: c.container_version,
		description: c.description.clone(),
		uploader: c.uploader.clone(),
		encrypted: c.encrypted,
		max_download_threads: c.max_download_threads,
		connection: RawConnectionV3 {
			host: c.connection.host.clone(),
			port: c.connection.port,
			username: c.connection.username.clone(),
			password: c.connection.password.clone(),
			auth_required: c.connection.auth_required,
			data_connection_type: c.connection.data_connection_type,
			data_type: c.connection.data_type,
			character_encoding: c.connection.character_encoding,
			ssl_protocol: c.connection.ssl_protocol,
			connect_timeout: c.connection.connect_timeout,
			command_timeout: c.connection.command_timeout,
		},
		packages: RawPackagesV3 {
			packages: c
				.packages
				.iter()
				.map(|p| RawPackageV3 {
					name: p.name.clone(),
					bulk_folder_mode: p.bulk_folder_mode,
					file_list: if p.file_list.is_empty() {
						None
					} else {
						Some(RawFileListV3 {
							items: p
								.file_list
								.iter()
								.map(|f| RawFileItemV3 {
									file_name: f.file_name.clone(),
									directory_root: f.directory_root.clone(),
									directory_path: f.directory_path.clone(),
									full_path: f.full_path.clone(),
									file_size: f.file_size,
									hash_type: f.hash_type,
									file_hash: f.file_hash.clone(),
									package_name: f.package_name.clone(),
								})
								.collect(),
						})
					},
					bulk_folder_list: if p.bulk_folder_list.is_empty() {
						None
					} else {
						Some(RawBulkFolderListV3 {
							items: p
								.bulk_folder_list
								.iter()
								.map(|b| RawBulkFolderV3 {
									bulk_folder_path: b.bulk_folder_path.clone(),
									package_name: b.package_name.clone(),
								})
								.collect(),
						})
					},
				})
				.collect(),
		},
	}
}

// --- v2 parsing ---

#[derive(Debug, Deserialize)]
#[serde(rename = "SFDLFile")]
struct RawSfdlV2 {
	#[serde(rename = "Description", default)]
	description: String,
	#[serde(rename = "Uploader", default)]
	uploader: String,
	#[serde(rename = "SFDLFileVersion", default)]
	_version: String,
	#[serde(rename = "Encrypted", default)]
	encrypted: bool,
	#[serde(rename = "ConnectionInfo")]
	connection_info: RawConnectionV2,
	#[serde(rename = "Packages")]
	packages: RawPackagesV2,
	#[serde(rename = "MaxDownloadThreads", default = "default_threads")]
	max_download_threads: u32,
}

#[derive(Debug, Deserialize)]
struct RawConnectionV2 {
	#[serde(rename = "Host", default)]
	host: String,
	#[serde(rename = "Port", default = "default_port")]
	port: u16,
	#[serde(rename = "Username", default)]
	username: String,
	#[serde(rename = "Password", default)]
	password: String,
	#[serde(rename = "AuthRequired", default)]
	auth_required: bool,
	#[serde(rename = "DataConnectionType", default)]
	data_connection_type: FtpDataConnectionType,
	#[serde(rename = "DataType", default)]
	data_type: FtpDataType,
	#[serde(rename = "CharacterEncoding", default)]
	character_encoding: CharacterEncoding,
	#[serde(rename = "EncryptionMode", default)]
	ssl_protocol: SslProtocol,
	#[serde(rename = "ConnectTimeout", default = "default_timeout")]
	connect_timeout: u32,
	#[serde(rename = "CommandTimeout", default = "default_timeout")]
	command_timeout: u32,
}

#[derive(Debug, Deserialize)]
struct RawPackagesV2 {
	#[serde(rename = "SFDLPackage", default)]
	packages: Vec<RawPackageV2>,
}

#[derive(Debug, Deserialize)]
struct RawPackageV2 {
	#[serde(rename = "Packagename", default)]
	name: String,
	#[serde(rename = "BulkFolderMode", default = "default_true")]
	bulk_folder_mode: bool,
	#[serde(rename = "BulkFolderList", default)]
	bulk_folder_list: Option<RawBulkFolderListV2>,
}

#[derive(Debug, Deserialize)]
struct RawBulkFolderListV2 {
	#[serde(rename = "BulkFolder", default)]
	items: Vec<RawBulkFolderV2>,
}

#[derive(Debug, Deserialize)]
struct RawBulkFolderV2 {
	#[serde(rename = "BulkFolderPath", default)]
	bulk_folder_path: String,
	#[serde(rename = "PackageName", default)]
	package_name: String,
}

fn parse_v2(xml: &str) -> Result<SfdlContainer, SfdlError> {
	let raw: RawSfdlV2 = from_str(xml).map_err(|e| SfdlError::ParseError(e.to_string()))?;

	Ok(SfdlContainer {
		container_version: 2,
		version: SfdlVersion::V2,
		description: raw.description,
		uploader: raw.uploader,
		encrypted: raw.encrypted,
		max_download_threads: raw.max_download_threads,
		connection: Connection {
			host: raw.connection_info.host,
			port: raw.connection_info.port,
			username: raw.connection_info.username,
			password: raw.connection_info.password,
			auth_required: raw.connection_info.auth_required,
			data_connection_type: raw.connection_info.data_connection_type,
			data_type: raw.connection_info.data_type,
			character_encoding: raw.connection_info.character_encoding,
			ssl_protocol: raw.connection_info.ssl_protocol,
			connect_timeout: raw.connection_info.connect_timeout,
			command_timeout: raw.connection_info.command_timeout,
		},
		packages: raw
			.packages
			.packages
			.into_iter()
			.map(|p| Package {
				name: p.name,
				bulk_folder_mode: p.bulk_folder_mode,
				file_list: Vec::new(),
				bulk_folder_list: p
					.bulk_folder_list
					.map(|bl| {
						bl.items
							.into_iter()
							.map(|b| BulkFolder {
								bulk_folder_path: b.bulk_folder_path,
								package_name: b.package_name,
							})
							.collect()
					})
					.unwrap_or_default(),
			})
			.collect(),
	})
}

#[cfg(test)]
mod tests {
	use super::*;

	const UNENCRYPTED_V3: &str = include_str!("../../tests/fixtures/unencrypted_v3.sfdl");
	const UNENCRYPTED_V2: &str = include_str!("../../tests/fixtures/unencrypted_v2.sfdl");
	const ENCRYPTED_V3: &str = include_str!("../../tests/fixtures/encrypted_v3.sfdl");
	const BULKFOLDER_V3: &str = include_str!("../../tests/fixtures/bulkfolder_v3.sfdl");
	const ENCRYPTED_BULKFOLDER_V3: &str = include_str!("../../tests/fixtures/encrypted_bulkfolder_v3.sfdl");
	const INVALID: &str = include_str!("../../tests/fixtures/invalid.sfdl");

	// --- SFDL-001: Parse SFDL file ---

	/// SFDL-001 | BR-SFDL-001: Detect v3 container version.
	#[test]
	fn sfdl001_detect_version_v3() {
		assert_eq!(detect_version(UNENCRYPTED_V3).unwrap(), SfdlVersion::V3);
	}

	/// SFDL-001 | Main Success: Parse unencrypted v3 container with all fields.
	#[test]
	fn sfdl001_parse_unencrypted_v3() {
		let container = parse_sfdl(UNENCRYPTED_V3).unwrap();

		assert_eq!(container.container_version, 10);
		assert_eq!(container.description, "Test.Release.2026.1080p");
		assert_eq!(container.uploader, "testuser");
		assert!(!container.encrypted);
		assert_eq!(container.max_download_threads, 3);

		// Connection
		assert_eq!(container.connection.host, "ftp.example.com");
		assert_eq!(container.connection.port, 21);
		assert_eq!(container.connection.username, "ftpuser");
		assert_eq!(container.connection.password, "ftppass");
		assert!(container.connection.auth_required);
		assert_eq!(container.connection.data_connection_type, FtpDataConnectionType::Passive);
		assert_eq!(container.connection.data_type, FtpDataType::Binary);
		assert_eq!(container.connection.character_encoding, CharacterEncoding::Utf8);
		assert_eq!(container.connection.ssl_protocol, SslProtocol::None);

		// Packages
		assert_eq!(container.packages.len(), 1);
		let pkg = &container.packages[0];
		assert_eq!(pkg.name, "Package1");
		assert!(!pkg.bulk_folder_mode);
		assert_eq!(pkg.file_list.len(), 2);

		// FileItems
		let f1 = &pkg.file_list[0];
		assert_eq!(f1.file_name, "movie.part1.rar");
		assert_eq!(f1.directory_root, "/");
		assert_eq!(f1.directory_path, "releases/test");
		assert_eq!(f1.full_path, "/releases/test/movie.part1.rar");
		assert_eq!(f1.file_size, 104_857_600);
		assert_eq!(f1.hash_type, HashType::MD5);
		assert_eq!(f1.file_hash, "d41d8cd98f00b204e9800998ecf8427e");
		assert_eq!(f1.package_name, "Package1");

		let f2 = &pkg.file_list[1];
		assert_eq!(f2.file_name, "movie.part2.rar");
		assert_eq!(f2.hash_type, HashType::None);
		assert_eq!(f2.file_hash, "");
	}

	/// SFDL-001 | BR-SFDL-002: Detect v2 container version.
	#[test]
	fn sfdl001_detect_version_v2() {
		assert_eq!(detect_version(UNENCRYPTED_V2).unwrap(), SfdlVersion::V2);
	}

	/// SFDL-001 | Main Success: Parse unencrypted v2 container.
	#[test]
	fn sfdl001_parse_unencrypted_v2() {
		let container = parse_sfdl(UNENCRYPTED_V2).unwrap();

		assert_eq!(container.container_version, 2);
		assert_eq!(container.description, "Test.Release.v2");
		assert_eq!(container.uploader, "testuser");
		assert!(!container.encrypted);

		assert_eq!(container.connection.host, "ftp.example.com");
		assert_eq!(container.connection.port, 21);
		assert_eq!(container.connection.username, "ftpuser");

		// v2 normalization: packages use BulkFolderMode
		assert_eq!(container.packages.len(), 1);
		let pkg = &container.packages[0];
		assert!(pkg.bulk_folder_mode);
		assert!(pkg.file_list.is_empty());
		assert_eq!(pkg.bulk_folder_list.len(), 1);
		assert_eq!(pkg.bulk_folder_list[0].bulk_folder_path, "/releases/test/");
	}

	/// SFDL-001 | Main Success: Parse encrypted v3 container (fields are Base64).
	#[test]
	fn sfdl001_parse_encrypted_v3_raw() {
		let container = parse_sfdl(ENCRYPTED_V3).unwrap();

		assert_eq!(container.container_version, 10);
		assert!(container.encrypted);
		// Fields should be Base64 strings (not yet decrypted)
		assert_ne!(container.description, "Test.Release.2026.1080p");
		assert_ne!(container.connection.host, "ftp.example.com");
		// Port is not encrypted
		assert_eq!(container.connection.port, 21);
	}

	/// SFDL-001 | Main Success: Parse v3 BulkFolder container.
	#[test]
	fn sfdl001_parse_bulkfolder_v3() {
		let container = parse_sfdl(BULKFOLDER_V3).unwrap();

		assert_eq!(container.container_version, 10);
		assert_eq!(container.description, "BulkFolder.Test.2026");
		assert!(!container.encrypted);

		// Package
		assert_eq!(container.packages.len(), 1);
		let pkg = &container.packages[0];
		assert_eq!(pkg.name, "BulkPkg1");
		assert!(pkg.bulk_folder_mode);
		assert!(pkg.file_list.is_empty());

		// BulkFolderList
		assert_eq!(pkg.bulk_folder_list.len(), 2);
		assert_eq!(pkg.bulk_folder_list[0].bulk_folder_path, "/releases/movie/");
		assert_eq!(pkg.bulk_folder_list[0].package_name, "BulkPkg1");
		assert_eq!(pkg.bulk_folder_list[1].bulk_folder_path, "/releases/extras/");
		assert_eq!(pkg.bulk_folder_list[1].package_name, "BulkPkg1");
	}

	/// SFDL-001 | Main Success: BulkFolder container has correct connection data.
	#[test]
	fn sfdl001_parse_bulkfolder_v3_connection() {
		let container = parse_sfdl(BULKFOLDER_V3).unwrap();

		assert_eq!(container.connection.host, "ftp.example.com");
		assert_eq!(container.connection.port, 21);
		assert_eq!(container.connection.username, "ftpuser");
		assert_eq!(container.connection.password, "ftppass");
		assert!(container.connection.auth_required);
	}

	/// SFDL-001 | Main Success: Parse encrypted v3 BulkFolder (fields encrypted).
	#[test]
	fn sfdl001_parse_encrypted_bulkfolder_v3_raw() {
		let container = parse_sfdl(ENCRYPTED_BULKFOLDER_V3).unwrap();

		assert_eq!(container.container_version, 10);
		assert!(container.encrypted);

		// Fields should be Base64-encoded ciphertext
		assert_ne!(container.description, "BulkFolder.Test.2026");
		assert_ne!(container.connection.host, "ftp.example.com");

		// Structure: BulkFolderMode=true, empty FileList, populated BulkFolderList
		let pkg = &container.packages[0];
		assert!(pkg.bulk_folder_mode);
		assert!(pkg.file_list.is_empty());
		assert_eq!(pkg.bulk_folder_list.len(), 2);

		// Paths are still encrypted
		assert_ne!(pkg.bulk_folder_list[0].bulk_folder_path, "/releases/movie/");
	}

	/// SFDL-001 | A1: Invalid XML returns error.
	#[test]
	fn sfdl001_parse_invalid_xml() {
		let result = parse_sfdl(INVALID);
		assert!(result.is_err());
	}

	/// SFDL-001 | A1: Random text is not a valid SFDL.
	#[test]
	fn sfdl001_detect_version_invalid() {
		let result = detect_version("just some random text");
		assert!(result.is_err());
	}

	/// SFDL-001 | A1: Empty input returns error.
	#[test]
	fn sfdl001_detect_version_empty() {
		let result = detect_version("");
		assert!(result.is_err());
	}

	// --- CR-005: Serialize container ---

	/// CR-005 | Main Success: Serialize and re-parse v3 FileList container.
	#[test]
	fn cr005_serialize_v3_round_trip_filelist() {
		let original = parse_sfdl(UNENCRYPTED_V3).unwrap();
		let xml = serialize_v3(&original).unwrap();
		let reparsed = parse_sfdl(&xml).unwrap();

		assert_eq!(reparsed.container_version, original.container_version);
		assert_eq!(reparsed.description, original.description);
		assert_eq!(reparsed.uploader, original.uploader);
		assert_eq!(reparsed.encrypted, original.encrypted);
		assert_eq!(reparsed.max_download_threads, original.max_download_threads);
		assert_eq!(reparsed.connection.host, original.connection.host);
		assert_eq!(reparsed.connection.port, original.connection.port);
		assert_eq!(reparsed.connection.username, original.connection.username);
		assert_eq!(reparsed.connection.password, original.connection.password);
		assert_eq!(reparsed.packages.len(), original.packages.len());

		let orig_pkg = &original.packages[0];
		let re_pkg = &reparsed.packages[0];
		assert_eq!(re_pkg.name, orig_pkg.name);
		assert_eq!(re_pkg.bulk_folder_mode, orig_pkg.bulk_folder_mode);
		assert_eq!(re_pkg.file_list.len(), orig_pkg.file_list.len());
		assert_eq!(re_pkg.file_list[0], orig_pkg.file_list[0]);
		assert_eq!(re_pkg.file_list[1], orig_pkg.file_list[1]);
	}

	/// CR-005 | Main Success: Serialize and re-parse v3 BulkFolder container.
	#[test]
	fn cr005_serialize_v3_round_trip_bulkfolder() {
		let original = parse_sfdl(BULKFOLDER_V3).unwrap();
		let xml = serialize_v3(&original).unwrap();
		let reparsed = parse_sfdl(&xml).unwrap();

		assert_eq!(reparsed.description, original.description);
		assert_eq!(reparsed.uploader, original.uploader);
		let orig_pkg = &original.packages[0];
		let re_pkg = &reparsed.packages[0];
		assert_eq!(re_pkg.name, orig_pkg.name);
		assert!(re_pkg.bulk_folder_mode);
		assert!(re_pkg.file_list.is_empty());
		assert_eq!(re_pkg.bulk_folder_list.len(), orig_pkg.bulk_folder_list.len());
		assert_eq!(re_pkg.bulk_folder_list[0].bulk_folder_path, orig_pkg.bulk_folder_list[0].bulk_folder_path);
		assert_eq!(re_pkg.bulk_folder_list[1].bulk_folder_path, orig_pkg.bulk_folder_list[1].bulk_folder_path);
	}

	/// CR-005 | BR: Serialized XML has proper header.
	#[test]
	fn cr005_serialize_v3_has_xml_header() {
		let container = SfdlContainer::default();
		let xml = serialize_v3(&container).unwrap();
		assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"utf-8\"?>"));
	}

	/// CR-005 | Full Pipeline: Build, encrypt, serialize, parse, decrypt, verify.
	#[test]
	fn cr005_serialize_v3_full_pipeline() {
		use crate::sfdl::crypto::{decrypt_container, encrypt_container};

		// Build a container from scratch
		let mut container = SfdlContainer {
			description: "Pipeline.Test.2026".into(),
			uploader: "rsfdl".into(),
			connection: Connection {
				host: "ftp.test.com".into(),
				port: 21,
				username: "user".into(),
				password: "pass".into(),
				auth_required: true,
				..Connection::default()
			},
			packages: vec![Package {
				name: "TestPkg".into(),
				bulk_folder_mode: true,
				bulk_folder_list: vec![BulkFolder {
					bulk_folder_path: "/data/release/".into(),
					package_name: "TestPkg".into(),
				}],
				..Package::default()
			}],
			..SfdlContainer::default()
		};

		// Encrypt → serialize → parse → decrypt → verify
		encrypt_container(&mut container, "mypassword");
		assert!(container.encrypted);

		let xml = serialize_v3(&container).unwrap();
		let mut reparsed = parse_sfdl(&xml).unwrap();
		assert!(reparsed.encrypted);

		decrypt_container(&mut reparsed, "mypassword").unwrap();
		assert_eq!(reparsed.description, "Pipeline.Test.2026");
		assert_eq!(reparsed.connection.host, "ftp.test.com");
		assert_eq!(reparsed.connection.username, "user");
		assert_eq!(reparsed.packages[0].name, "TestPkg");
		assert_eq!(reparsed.packages[0].bulk_folder_list[0].bulk_folder_path, "/data/release/");
	}
}
