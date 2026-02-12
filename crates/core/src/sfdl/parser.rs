use quick_xml::de::from_str;
use serde::Deserialize;

use crate::error::SfdlError;
use crate::sfdl::models::*;

/// Detects the SFDL version from raw XML content.
pub fn detect_version(xml: &str) -> Result<SfdlVersion, SfdlError> {
    if xml.contains("<ContainerVersion>") {
        Ok(SfdlVersion::V3)
    } else if xml.contains("<SFDLFileVersion>") {
        Ok(SfdlVersion::V2)
    } else {
        Err(SfdlError::ParseError(
            "Cannot detect SFDL version: no ContainerVersion or SFDLFileVersion element found"
                .into(),
        ))
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

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
struct RawPackagesV3 {
    #[serde(rename = "Package", default)]
    packages: Vec<RawPackageV3>,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
struct RawFileListV3 {
    #[serde(rename = "FileItem", default)]
    items: Vec<RawFileItemV3>,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
struct RawBulkFolderListV3 {
    #[serde(rename = "BulkFolder", default)]
    items: Vec<RawBulkFolderV3>,
}

#[derive(Debug, Deserialize)]
struct RawBulkFolderV3 {
    #[serde(rename = "BulkFolderPath", default)]
    bulk_folder_path: String,
    #[serde(rename = "PackageName", default)]
    package_name: String,
}

fn parse_v3(xml: &str) -> Result<SfdlContainer, SfdlError> {
    let raw: RawContainerV3 =
        from_str(xml).map_err(|e| SfdlError::ParseError(e.to_string()))?;

    Ok(SfdlContainer {
        container_version: raw.container_version,
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
    let raw: RawSfdlV2 =
        from_str(xml).map_err(|e| SfdlError::ParseError(e.to_string()))?;

    Ok(SfdlContainer {
        container_version: 2,
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
    const INVALID: &str = include_str!("../../tests/fixtures/invalid.sfdl");

    // --- AT-01: Unverschlüsselte v3-Datei parsen ---

    #[test]
    fn detect_version_v3() {
        assert_eq!(detect_version(UNENCRYPTED_V3).unwrap(), SfdlVersion::V3);
    }

    #[test]
    fn parse_unencrypted_v3() {
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

    // --- AT-02: Unverschlüsselte v2-Datei parsen ---

    #[test]
    fn detect_version_v2() {
        assert_eq!(detect_version(UNENCRYPTED_V2).unwrap(), SfdlVersion::V2);
    }

    #[test]
    fn parse_unencrypted_v2() {
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

    // --- AT-03: Verschlüsselte v3-Datei parsen (Felder sind noch Base64) ---

    #[test]
    fn parse_encrypted_v3_raw() {
        let container = parse_sfdl(ENCRYPTED_V3).unwrap();

        assert_eq!(container.container_version, 10);
        assert!(container.encrypted);
        // Fields should be Base64 strings (not yet decrypted)
        assert_ne!(container.description, "Test.Release.2026.1080p");
        assert_ne!(container.connection.host, "ftp.example.com");
        // Port is not encrypted
        assert_eq!(container.connection.port, 21);
    }

    // --- AT-06: Ungültiges XML ---

    #[test]
    fn parse_invalid_xml() {
        let result = parse_sfdl(INVALID);
        assert!(result.is_err());
    }

    #[test]
    fn detect_version_invalid() {
        let result = detect_version("just some random text");
        assert!(result.is_err());
    }

    #[test]
    fn detect_version_empty() {
        let result = detect_version("");
        assert!(result.is_err());
    }
}
