use quick_xml::de::from_str;
use quick_xml::se::to_string as xml_to_string;
use serde::{Deserialize, Serialize};

use crate::error::SfdlError;
use crate::sfdl::crypto::EncryptedSfdl;
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

/// Parses an SFDL file (v2 or v3) into a typed [`SfdlFile`].
///
/// Returns [`SfdlFile::Encrypted`] when the XML `<Encrypted>` flag is `true`,
/// and [`SfdlFile::Decrypted`] for plaintext containers.
pub fn parse_sfdl(xml: &str) -> Result<SfdlFile, SfdlError> {
	let version = detect_version(xml)?;
	let (container, was_encrypted) = match version {
		SfdlVersion::V3 => parse_v3(xml)?,
		SfdlVersion::V2 => parse_v2(xml)?,
	};
	// A3: Validate structural requirements (applies to all containers)
	validate_structural(&container)?;
	if was_encrypted {
		Ok(SfdlFile::Encrypted(EncryptedSfdl(container)))
	} else {
		// A3: For unencrypted containers also validate connection fields
		validate_decrypted(&container)?;
		Ok(SfdlFile::Decrypted(container))
	}
}

/// SFDL-001 / A4: Read and parse an SFDL file from disk.
pub fn load_sfdl_file(path: &std::path::Path) -> Result<SfdlFile, SfdlError> {
	let xml = std::fs::read_to_string(path).map_err(|e| SfdlError::FileError(format!("{}: {}", path.display(), e)))?;
	parse_sfdl(&xml)
}

/// Validates structural requirements that apply to all containers (encrypted or not).
fn validate_structural(container: &SfdlContainer) -> Result<(), SfdlError> {
	if container.packages.is_empty() {
		return Err(SfdlError::ParseError("SFDL file has no packages".into()));
	}
	Ok(())
}

/// Validates fields that must be present in a decrypted (plaintext) container.
///
/// Encrypted containers skip this — their fields are ciphertext and can only
/// be validated after decryption.
fn validate_decrypted(container: &SfdlContainer) -> Result<(), SfdlError> {
	if container.connection.host.is_empty() {
		return Err(SfdlError::ParseError("SFDL file missing connection host".into()));
	}
	Ok(())
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

fn parse_v3(xml: &str) -> Result<(SfdlContainer, bool), SfdlError> {
	let raw: RawContainerV3 = from_str(xml).map_err(|e| SfdlError::ParseError(e.to_string()))?;

	// BR-SFDL-001: Validate ContainerVersion number
	if raw.container_version != 10 {
		return Err(SfdlError::ParseError(format!(
			"Unsupported ContainerVersion {}. Only version 10 (SFDL v3) is supported.",
			raw.container_version
		)));
	}

	let was_encrypted = raw.encrypted;
	let container = SfdlContainer {
		container_version: raw.container_version,
		version: SfdlVersion::V3,
		description: raw.description,
		uploader: raw.uploader,
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
	};
	Ok((container, was_encrypted))
}

// --- v3 serialization ---

/// Serializes an `SfdlContainer` to SFDL v3 XML.
///
/// The `encrypted` flag is written into the `<Encrypted>` element — callers
/// must pass `true` when serializing an [`EncryptedSfdl`] container.
pub fn serialize_v3(container: &SfdlContainer, encrypted: bool) -> Result<String, SfdlError> {
	let raw = to_raw_v3(container, encrypted);
	let xml_body = xml_to_string(&raw).map_err(|e| SfdlError::SerializeError(e.to_string()))?;
	Ok(format!("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n{}", xml_body))
}

/// Serializes an [`SfdlFile`] to SFDL v3 XML, setting the `<Encrypted>` flag correctly.
pub fn serialize_sfdl(file: &SfdlFile) -> Result<String, SfdlError> {
	match file {
		SfdlFile::Encrypted(enc) => serialize_v3(enc.inner(), true),
		SfdlFile::Decrypted(container) => serialize_v3(container, false),
	}
}

fn to_raw_v3(c: &SfdlContainer, encrypted: bool) -> RawContainerV3 {
	RawContainerV3 {
		container_version: c.container_version,
		description: c.description.clone(),
		uploader: c.uploader.clone(),
		encrypted,
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
	version: String,
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

fn parse_v2(xml: &str) -> Result<(SfdlContainer, bool), SfdlError> {
	let raw: RawSfdlV2 = from_str(xml).map_err(|e| SfdlError::ParseError(e.to_string()))?;

	// BR-SFDL-001: Validate SFDLFileVersion number (6-9 = v2)
	let version_num: u32 = raw.version.trim().parse().unwrap_or(0);
	if !(6..=9).contains(&version_num) {
		return Err(SfdlError::ParseError(format!("Unsupported SFDLFileVersion '{}'. Expected 6-9 for SFDL v2.", raw.version)));
	}

	let was_encrypted = raw.encrypted;
	let container = SfdlContainer {
		container_version: 2,
		version: SfdlVersion::V2,
		description: raw.description,
		uploader: raw.uploader,
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
	};
	Ok((container, was_encrypted))
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
		let file = parse_sfdl(UNENCRYPTED_V3).unwrap();
		let container = match file {
			SfdlFile::Decrypted(c) => c,
			SfdlFile::Encrypted(_) => panic!("expected Decrypted"),
		};

		assert_eq!(container.container_version, 10);
		assert_eq!(container.description, "Test.Release.2026.1080p");
		assert_eq!(container.uploader, "testuser");
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
		let file = parse_sfdl(UNENCRYPTED_V2).unwrap();
		let container = match file {
			SfdlFile::Decrypted(c) => c,
			SfdlFile::Encrypted(_) => panic!("expected Decrypted"),
		};

		assert_eq!(container.container_version, 2);
		assert_eq!(container.description, "Test.Release.v2");
		assert_eq!(container.uploader, "testuser");

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

	/// SFDL-001 | Main Success: Parse encrypted v3 container — returns SfdlFile::Encrypted.
	#[test]
	fn sfdl001_parse_encrypted_v3_raw() {
		let file = parse_sfdl(ENCRYPTED_V3).unwrap();
		let enc = match file {
			SfdlFile::Encrypted(e) => e,
			SfdlFile::Decrypted(_) => panic!("expected Encrypted"),
		};

		let container = enc.inner();
		assert_eq!(container.container_version, 10);
		// Fields should be Base64 strings (not yet decrypted)
		assert_ne!(container.description, "Test.Release.2026.1080p");
		assert_ne!(container.connection.host, "ftp.example.com");
		// Port is not encrypted
		assert_eq!(container.connection.port, 21);
	}

	/// SFDL-001 | Main Success: Parse v3 BulkFolder container.
	#[test]
	fn sfdl001_parse_bulkfolder_v3() {
		let file = parse_sfdl(BULKFOLDER_V3).unwrap();
		let container = match file {
			SfdlFile::Decrypted(c) => c,
			SfdlFile::Encrypted(_) => panic!("expected Decrypted"),
		};

		assert_eq!(container.container_version, 10);
		assert_eq!(container.description, "BulkFolder.Test.2026");

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
		let file = parse_sfdl(BULKFOLDER_V3).unwrap();
		let container = match file {
			SfdlFile::Decrypted(c) => c,
			SfdlFile::Encrypted(_) => panic!("expected Decrypted"),
		};

		assert_eq!(container.connection.host, "ftp.example.com");
		assert_eq!(container.connection.port, 21);
		assert_eq!(container.connection.username, "ftpuser");
		assert_eq!(container.connection.password, "ftppass");
		assert!(container.connection.auth_required);
	}

	/// SFDL-001 | Main Success: Parse encrypted v3 BulkFolder — returns SfdlFile::Encrypted.
	#[test]
	fn sfdl001_parse_encrypted_bulkfolder_v3_raw() {
		let file = parse_sfdl(ENCRYPTED_BULKFOLDER_V3).unwrap();
		let enc = match file {
			SfdlFile::Encrypted(e) => e,
			SfdlFile::Decrypted(_) => panic!("expected Encrypted"),
		};

		let container = enc.inner();
		assert_eq!(container.container_version, 10);

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

	/// SFDL-001 | Main Success: Parsed v3 container has SfdlVersion::V3.
	#[test]
	fn sfdl001_v3_has_version_field() {
		let file = parse_sfdl(UNENCRYPTED_V3).unwrap();
		let container = match file {
			SfdlFile::Decrypted(c) => c,
			SfdlFile::Encrypted(_) => panic!("expected Decrypted"),
		};
		assert_eq!(container.version, SfdlVersion::V3);
	}

	/// SFDL-001 | Main Success: Parsed v2 container has SfdlVersion::V2.
	#[test]
	fn sfdl001_v2_has_version_field() {
		let file = parse_sfdl(UNENCRYPTED_V2).unwrap();
		let container = match file {
			SfdlFile::Decrypted(c) => c,
			SfdlFile::Encrypted(_) => panic!("expected Decrypted"),
		};
		assert_eq!(container.version, SfdlVersion::V2);
	}

	/// SFDL-001 | BR-SFDL-001: ContainerVersion 0 is rejected.
	#[test]
	fn sfdl001_reject_container_version_0() {
		let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<Container>
  <ContainerVersion>0</ContainerVersion>
  <Encrypted>false</Encrypted>
  <Connection><Host>x</Host><Port>21</Port></Connection>
  <Packages><Package><Name>P</Name><BulkFolderMode>false</BulkFolderMode></Package></Packages>
</Container>"#;
		let result = parse_sfdl(xml);
		assert!(result.is_err());
		assert!(result.unwrap_err().to_string().contains("Unsupported ContainerVersion"));
	}

	/// SFDL-001 | BR-SFDL-001: ContainerVersion 5 (v1) is rejected.
	#[test]
	fn sfdl001_reject_container_version_v1() {
		let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<Container>
  <ContainerVersion>5</ContainerVersion>
  <Encrypted>false</Encrypted>
  <Connection><Host>x</Host><Port>21</Port></Connection>
  <Packages><Package><Name>P</Name><BulkFolderMode>false</BulkFolderMode></Package></Packages>
</Container>"#;
		let result = parse_sfdl(xml);
		assert!(result.is_err());
	}

	/// SFDL-001 | BR-SFDL-001: ContainerVersion 11 (>10) is rejected.
	#[test]
	fn sfdl001_reject_container_version_above_10() {
		let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<Container>
  <ContainerVersion>11</ContainerVersion>
  <Encrypted>false</Encrypted>
  <Connection><Host>x</Host><Port>21</Port></Connection>
  <Packages><Package><Name>P</Name><BulkFolderMode>false</BulkFolderMode></Package></Packages>
</Container>"#;
		let result = parse_sfdl(xml);
		assert!(result.is_err());
	}

	/// SFDL-001 | BR-SFDL-001: v2 SFDLFileVersion 3 is rejected (too low).
	#[test]
	fn sfdl001_reject_v2_version_too_low() {
		let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<SFDLFile>
  <SFDLFileVersion>3</SFDLFileVersion>
  <Encrypted>false</Encrypted>
  <ConnectionInfo><Host>x</Host><Port>21</Port></ConnectionInfo>
  <Packages><SFDLPackage><BulkFolderList></BulkFolderList></SFDLPackage></Packages>
</SFDLFile>"#;
		let result = parse_sfdl(xml);
		assert!(result.is_err());
		assert!(result.unwrap_err().to_string().contains("SFDLFileVersion"));
	}

	/// SFDL-001 | A3: No packages → validation error.
	#[test]
	fn sfdl001_reject_no_packages() {
		let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<Container>
  <ContainerVersion>10</ContainerVersion>
  <Encrypted>false</Encrypted>
  <Connection><Host>ftp.example.com</Host><Port>21</Port></Connection>
  <Packages></Packages>
</Container>"#;
		let result = parse_sfdl(xml);
		assert!(result.is_err());
		assert!(result.unwrap_err().to_string().contains("no packages"));
	}

	/// SFDL-001 | A3: Missing host on unencrypted container → validation error.
	#[test]
	fn sfdl001_reject_missing_host() {
		let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<Container>
  <ContainerVersion>10</ContainerVersion>
  <Encrypted>false</Encrypted>
  <Connection><Host></Host><Port>21</Port></Connection>
  <Packages><Package><Name>P</Name><BulkFolderMode>false</BulkFolderMode></Package></Packages>
</Container>"#;
		let result = parse_sfdl(xml);
		assert!(result.is_err());
		assert!(result.unwrap_err().to_string().contains("missing connection host"));
	}

	/// SFDL-001 | A3: Missing host is OK on encrypted container (host is ciphertext).
	#[test]
	fn sfdl001_accept_encrypted_empty_host() {
		let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<Container>
  <ContainerVersion>10</ContainerVersion>
  <Encrypted>true</Encrypted>
  <Connection><Host></Host><Port>21</Port></Connection>
  <Packages><Package><Name>P</Name><BulkFolderMode>false</BulkFolderMode></Package></Packages>
</Container>"#;
		// Encrypted containers may have empty host (it's ciphertext that failed to set)
		let result = parse_sfdl(xml);
		assert!(result.is_ok());
		assert!(matches!(result.unwrap(), SfdlFile::Encrypted(_)));
	}

	/// SFDL-001 | A4: load_sfdl_file with nonexistent path returns FileError.
	#[test]
	fn sfdl001_load_file_not_found() {
		let result = load_sfdl_file(std::path::Path::new("/nonexistent/path/test.sfdl"));
		assert!(result.is_err());
		assert!(result.unwrap_err().to_string().contains("nonexistent"));
	}

	/// SFDL-001 | A4: load_sfdl_file reads and parses a real file.
	#[test]
	fn sfdl001_load_file_success() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("test.sfdl");
		std::fs::write(&path, UNENCRYPTED_V3).unwrap();

		let file = load_sfdl_file(&path).unwrap();
		let container = match file {
			SfdlFile::Decrypted(c) => c,
			SfdlFile::Encrypted(_) => panic!("expected Decrypted"),
		};
		assert_eq!(container.description, "Test.Release.2026.1080p");
		assert_eq!(container.version, SfdlVersion::V3);
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
		let file = parse_sfdl(UNENCRYPTED_V3).unwrap();
		let original = match &file {
			SfdlFile::Decrypted(c) => c,
			SfdlFile::Encrypted(_) => panic!("expected Decrypted"),
		};
		let xml = serialize_v3(original, false).unwrap();
		let reparsed_file = parse_sfdl(&xml).unwrap();
		let reparsed = match &reparsed_file {
			SfdlFile::Decrypted(c) => c,
			SfdlFile::Encrypted(_) => panic!("expected Decrypted after round-trip"),
		};

		assert_eq!(reparsed.container_version, original.container_version);
		assert_eq!(reparsed.description, original.description);
		assert_eq!(reparsed.uploader, original.uploader);
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
		let file = parse_sfdl(BULKFOLDER_V3).unwrap();
		let original = match &file {
			SfdlFile::Decrypted(c) => c,
			SfdlFile::Encrypted(_) => panic!("expected Decrypted"),
		};
		let xml = serialize_v3(original, false).unwrap();
		let reparsed_file = parse_sfdl(&xml).unwrap();
		let reparsed = match &reparsed_file {
			SfdlFile::Decrypted(c) => c,
			SfdlFile::Encrypted(_) => panic!("expected Decrypted after round-trip"),
		};

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
		let xml = serialize_v3(&container, false).unwrap();
		assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"utf-8\"?>"));
	}

	/// CR-005 | Main Success: serialize_sfdl dispatches correctly for Decrypted variant.
	#[test]
	fn cr005_serialize_sfdl_decrypted() {
		let file = parse_sfdl(UNENCRYPTED_V3).unwrap();
		let xml = serialize_sfdl(&file).unwrap();
		assert!(xml.contains("<Encrypted>false</Encrypted>"));
	}

	/// CR-005 | Main Success: serialize_sfdl dispatches correctly for Encrypted variant.
	#[test]
	fn cr005_serialize_sfdl_encrypted() {
		let file = parse_sfdl(ENCRYPTED_V3).unwrap();
		let xml = serialize_sfdl(&file).unwrap();
		assert!(xml.contains("<Encrypted>true</Encrypted>"));
	}

	/// CR-005 | Full Pipeline: Build, encrypt, serialize, parse, decrypt, verify.
	#[test]
	fn cr005_serialize_v3_full_pipeline() {
		use crate::sfdl::crypto::encrypt_container;

		// Build a container from scratch
		let container = SfdlContainer {
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

		// Encrypt (consumes container, returns EncryptedSfdl)
		let encrypted = encrypt_container(container, "mypassword");

		// Serialize the encrypted container
		let xml = serialize_v3(encrypted.inner(), true).unwrap();

		// Parse — should yield SfdlFile::Encrypted
		let reparsed = parse_sfdl(&xml).unwrap();
		let enc = match reparsed {
			SfdlFile::Encrypted(e) => e,
			SfdlFile::Decrypted(_) => panic!("expected Encrypted after serializing encrypted container"),
		};

		// Decrypt and verify
		let decrypted = enc.decrypt("mypassword").unwrap();
		assert_eq!(decrypted.description, "Pipeline.Test.2026");
		assert_eq!(decrypted.connection.host, "ftp.test.com");
		assert_eq!(decrypted.connection.username, "user");
		assert_eq!(decrypted.packages[0].name, "TestPkg");
		assert_eq!(decrypted.packages[0].bulk_folder_list[0].bulk_folder_path, "/data/release/");
	}
}
