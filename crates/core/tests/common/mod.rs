use std::fs;
use std::path::{Path, PathBuf};

use tokio::task::JoinHandle;

/// Embedded FTP server for integration tests.
/// Serves files from `ftp_root`. Shuts down when dropped.
pub struct FtpTestServer {
	port: u16,
	_handle: JoinHandle<()>,
}

impl FtpTestServer {
	/// Start an anonymous FTP server serving `ftp_root` on a random port.
	pub async fn start(ftp_root: PathBuf) -> Self {
		use unftp_sbe_fs::ServerExt;

		let port = portpicker::pick_unused_port().expect("no free port");
		let passive_start = portpicker::pick_unused_port().expect("no free passive port");
		let passive_end = passive_start.saturating_add(10);

		let server = libunftp::Server::with_fs(ftp_root)
			.passive_ports(passive_start..passive_end)
			.build()
			.expect("failed to build FTP server");

		let addr = format!("127.0.0.1:{}", port);
		let handle = tokio::spawn(async move {
			let _ = server.listen(addr).await;
		});

		// Wait for server to accept connections
		wait_for_port(port).await;

		Self { port, _handle: handle }
	}

	pub fn port(&self) -> u16 {
		self.port
	}
}

impl Drop for FtpTestServer {
	fn drop(&mut self) {
		self._handle.abort();
	}
}

/// Poll until a TCP connection to 127.0.0.1:port succeeds.
async fn wait_for_port(port: u16) {
	let addr = format!("127.0.0.1:{}", port);
	for _ in 0..50 {
		if tokio::net::TcpStream::connect(&addr).await.is_ok() {
			return;
		}
		tokio::time::sleep(std::time::Duration::from_millis(50)).await;
	}
	panic!("FTP server did not start on port {} within 2.5s", port);
}

/// Create a file inside the FTP root at the given relative path.
pub fn create_ftp_file(ftp_root: &Path, relative_path: &str, content: &[u8]) {
	let full = ftp_root.join(relative_path.trim_start_matches('/'));
	if let Some(parent) = full.parent() {
		fs::create_dir_all(parent).expect("create parent dirs");
	}
	fs::write(&full, content).expect("write test file");
}

/// Generate a valid SFDL v3 XML string with anonymous auth pointing to 127.0.0.1:port.
/// `files` is a slice of (file_name, directory_path, full_path, file_size).
pub fn generate_sfdl_xml(port: u16, files: &[(&str, &str, &str, u64)]) -> String {
	let file_items: String = files
		.iter()
		.map(|(name, dir, full, size)| {
			format!(
				r#"        <FileItem>
          <FileName>{name}</FileName>
          <DirectoryRoot>/</DirectoryRoot>
          <DirectoryPath>{dir}</DirectoryPath>
          <FullPath>{full}</FullPath>
          <FileSize>{size}</FileSize>
          <HashType>None</HashType>
          <FileHash></FileHash>
          <PackageName>TestPkg</PackageName>
        </FileItem>"#
			)
		})
		.collect::<Vec<_>>()
		.join("\n");

	format!(
		r#"<?xml version="1.0" encoding="utf-8"?>
<Container>
  <ContainerVersion>10</ContainerVersion>
  <Description>Download.Test</Description>
  <Uploader>test</Uploader>
  <Encrypted>false</Encrypted>
  <MaxDownloadThreads>3</MaxDownloadThreads>
  <Connection>
    <Host>127.0.0.1</Host>
    <Port>{port}</Port>
    <Username>anonymous</Username>
    <Password></Password>
    <AuthRequired>false</AuthRequired>
    <DataConnectionType>Passive</DataConnectionType>
    <DataType>Binary</DataType>
    <CharacterEncoding>UTF8</CharacterEncoding>
    <SSLProtocol>None</SSLProtocol>
    <ConnectTimeout>10</ConnectTimeout>
    <CommandTimeout>10</CommandTimeout>
  </Connection>
  <Packages>
    <Package>
      <Name>TestPkg</Name>
      <BulkFolderMode>false</BulkFolderMode>
      <FileList>
{file_items}
      </FileList>
      <BulkFolderList />
    </Package>
  </Packages>
</Container>"#
	)
}

/// Generate SFDL XML with an empty file list.
pub fn generate_empty_sfdl_xml(port: u16) -> String {
	generate_sfdl_xml(port, &[])
}

/// Write SFDL XML to a file in the given directory. Returns the file path.
#[allow(dead_code)]
pub fn write_sfdl_to_file(dir: &Path, xml: &str) -> PathBuf {
	let path = dir.join("test.sfdl");
	fs::write(&path, xml).expect("write sfdl file");
	path
}

/// Helper to build an SfdlContainer from generated XML.
pub fn parse_sfdl_from_xml(xml: &str) -> rsfdl_core::sfdl::models::SfdlContainer {
	rsfdl_core::sfdl::parser::parse_sfdl(xml).expect("parse generated SFDL")
}
