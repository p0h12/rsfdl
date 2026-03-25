use serde::{Deserialize, Serialize};

// --- Enums ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FtpDataConnectionType {
	#[default]
	Passive,
	Active,
	AutoPassive,
	ExtendedPassive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FtpDataType {
	#[serde(alias = "ASCII")]
	Ascii,
	#[default]
	#[serde(alias = "default")]
	Binary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CharacterEncoding {
	Standard,
	#[default]
	#[serde(alias = "default")]
	#[serde(rename = "UTF8")]
	Utf8,
	#[serde(rename = "UTF7")]
	Utf7,
	#[serde(rename = "ASCII")]
	Ascii,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SslProtocol {
	#[default]
	None,
	Tls,
	Tls11,
	Tls12,
	Ssl2,
	Ssl3,
}

/// FTPS connection mode derived from SslProtocol.
///
/// Matches SFDL.NET behavior (FTPHelper.vb):
/// - Tls/Tls11/Tls12 → FtpES (Explicit FTPS via AUTH TLS)
/// - Ssl2/Ssl3 → FtpS (Implicit FTPS, direct TLS connection)
/// - None → plain FTP
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FtpsMode {
	None,
	Explicit,
	Implicit,
}

impl SslProtocol {
	pub fn ftps_mode(&self) -> FtpsMode {
		match self {
			SslProtocol::Tls | SslProtocol::Tls11 | SslProtocol::Tls12 => FtpsMode::Explicit,
			SslProtocol::Ssl2 | SslProtocol::Ssl3 => FtpsMode::Implicit,
			SslProtocol::None => FtpsMode::None,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HashType {
	MD5,
	CRC,
	SHA1,
	#[default]
	#[serde(alias = "default")]
	None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SfdlVersion {
	V2,
	#[default]
	V3,
}

// --- Core data models ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SfdlContainer {
	pub container_version: u32,
	#[serde(skip)]
	pub version: SfdlVersion,
	pub description: String,
	pub uploader: String,
	pub encrypted: bool,
	pub max_download_threads: u32,
	pub connection: Connection,
	pub packages: Vec<Package>,
}

impl Default for SfdlContainer {
	fn default() -> Self {
		Self {
			container_version: 10,
			version: SfdlVersion::V3,
			description: String::new(),
			uploader: String::new(),
			encrypted: false,
			max_download_threads: 3,
			connection: Connection::default(),
			packages: Vec::new(),
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
	pub host: String,
	pub port: u16,
	pub username: String,
	pub password: String,
	pub auth_required: bool,
	pub data_connection_type: FtpDataConnectionType,
	pub data_type: FtpDataType,
	pub character_encoding: CharacterEncoding,
	pub ssl_protocol: SslProtocol,
	pub connect_timeout: u32,
	pub command_timeout: u32,
}

impl Default for Connection {
	fn default() -> Self {
		Self {
			host: String::new(),
			port: 21,
			username: String::new(),
			password: String::new(),
			auth_required: false,
			data_connection_type: FtpDataConnectionType::Passive,
			data_type: FtpDataType::Binary,
			character_encoding: CharacterEncoding::Standard,
			ssl_protocol: SslProtocol::None,
			connect_timeout: 10,
			command_timeout: 10,
		}
	}
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Package {
	pub name: String,
	pub bulk_folder_mode: bool,
	pub file_list: Vec<FileItem>,
	pub bulk_folder_list: Vec<BulkFolder>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FileItem {
	pub file_name: String,
	pub directory_root: String,
	pub directory_path: String,
	pub full_path: String,
	pub file_size: u64,
	pub hash_type: HashType,
	pub file_hash: String,
	pub package_name: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BulkFolder {
	pub bulk_folder_path: String,
	pub package_name: String,
}

#[cfg(test)]
mod tests {
	use super::*;

	/// FTPS-001: SslProtocol::None maps to plain FTP.
	#[test]
	fn ftps_mode_none() {
		assert_eq!(SslProtocol::None.ftps_mode(), FtpsMode::None);
	}

	/// FTPS-001: TLS variants map to Explicit FTPS (AUTH TLS).
	#[test]
	fn ftps_mode_explicit_tls_variants() {
		assert_eq!(SslProtocol::Tls.ftps_mode(), FtpsMode::Explicit);
		assert_eq!(SslProtocol::Tls11.ftps_mode(), FtpsMode::Explicit);
		assert_eq!(SslProtocol::Tls12.ftps_mode(), FtpsMode::Explicit);
	}

	/// FTPS-001: SSL variants map to Implicit FTPS (direct TLS).
	#[test]
	fn ftps_mode_implicit_ssl_variants() {
		assert_eq!(SslProtocol::Ssl2.ftps_mode(), FtpsMode::Implicit);
		assert_eq!(SslProtocol::Ssl3.ftps_mode(), FtpsMode::Implicit);
	}

	/// FTPS-001: Default SslProtocol is None → plain FTP.
	#[test]
	fn ftps_mode_default_is_plain() {
		assert_eq!(SslProtocol::default().ftps_mode(), FtpsMode::None);
	}
}
