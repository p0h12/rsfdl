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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HashType {
	MD5,
	CRC,
	SHA1,
	#[default]
	#[serde(alias = "default")]
	None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SfdlVersion {
	V2,
	V3,
}

// --- Core data models ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SfdlContainer {
	pub container_version: u32,
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
