use std::path::Path;
use std::time::Duration;

use futures_lite::AsyncReadExt;
use suppaftp::async_native_tls::TlsConnector;
use suppaftp::types::FileType;
use suppaftp::{AsyncNativeTlsConnector, AsyncNativeTlsFtpStream, Mode};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::download::progress::ProgressEvent;
use crate::error::{DownloadError, FtpError};
use crate::sfdl::models::{Connection, FtpsMode, HashType};

/// Entry from an FTP directory listing.
#[derive(Debug, Clone)]
pub struct ListEntry {
	pub name: String,
	pub is_directory: bool,
	pub size: u64,
}

/// Async FTP client wrapping suppaftp.
/// One instance per download task (per-file connection pattern).
pub struct FtpClient {
	stream: AsyncNativeTlsFtpStream,
}

impl FtpClient {
	/// Connect and authenticate using SFDL Connection settings.
	/// `timeout_seconds` controls the connect timeout (0 = no timeout).
	///
	/// Supports plain FTP, Explicit FTPS (AUTH TLS), and Implicit FTPS
	/// based on the container's `SslProtocol` field.
	pub async fn connect(conn: &Connection, timeout_seconds: u32) -> Result<Self, FtpError> {
		let addr = format!("{}:{}", conn.host, conn.port);
		let ftps_mode = conn.ssl_protocol.ftps_mode();
		let timeout = Duration::from_secs(timeout_seconds as u64);

		let mut stream = match ftps_mode {
			FtpsMode::None => Self::plain_connect(&addr, timeout_seconds).await?,
			FtpsMode::Explicit => {
				// Connect plain TCP, then upgrade via AUTH TLS
				let plain = Self::plain_connect(&addr, timeout_seconds).await?;
				let tls_ctx = Self::build_tls_connector(&conn.host);
				tracing::debug!(host = %conn.host, "Upgrading to Explicit FTPS (AUTH TLS)");
				if timeout_seconds > 0 {
					tokio::time::timeout(timeout, plain.into_secure(tls_ctx, &conn.host))
						.await
						.map_err(|_| FtpError::Timeout)?
						.map_err(FtpError::from)?
				} else {
					plain.into_secure(tls_ctx, &conn.host).await.map_err(FtpError::from)?
				}
			}
			FtpsMode::Implicit => {
				// Connect directly over TLS (legacy implicit FTPS)
				let tls_ctx = Self::build_tls_connector(&conn.host);
				tracing::debug!(host = %conn.host, "Connecting via Implicit FTPS");
				let connect_fut = AsyncNativeTlsFtpStream::connect_secure_implicit(&addr, tls_ctx, &conn.host);
				if timeout_seconds > 0 {
					tokio::time::timeout(timeout, connect_fut).await.map_err(|_| FtpError::Timeout)?.map_err(FtpError::from)?
				} else {
					connect_fut.await.map_err(FtpError::from)?
				}
			}
		};

		// TODO: Respect conn.data_connection_type (Active, ExtendedPassive, etc.)
		// Currently hardcoded to Passive — the SFDL container's FtpDataConnectionType is ignored.
		stream.set_mode(Mode::Passive);

		// Login
		let (user, pass) = if conn.auth_required {
			(conn.username.as_str(), conn.password.as_str())
		} else {
			("anonymous", "anonymous@rsfdl")
		};
		stream.login(user, pass).await.map_err(FtpError::from)?;

		tracing::debug!(
			host = %conn.host, port = conn.port,
			ftps = ?ftps_mode,
			"FTP connected"
		);

		Ok(Self { stream })
	}

	/// Establish a plain (unencrypted) TCP connection to the FTP server.
	async fn plain_connect(addr: &str, timeout_seconds: u32) -> Result<AsyncNativeTlsFtpStream, FtpError> {
		if timeout_seconds > 0 {
			tokio::time::timeout(Duration::from_secs(timeout_seconds as u64), AsyncNativeTlsFtpStream::connect(addr))
				.await
				.map_err(|_| FtpError::Timeout)?
				.map_err(FtpError::from)
		} else {
			AsyncNativeTlsFtpStream::connect(addr).await.map_err(FtpError::from)
		}
	}

	/// Build a TLS connector that accepts invalid/self-signed certificates.
	///
	/// SFDL FTP servers typically use self-signed certificates,
	/// matching SFDL.NET's default behavior of not validating certs.
	fn build_tls_connector(domain: &str) -> AsyncNativeTlsConnector {
		let tls = TlsConnector::new().danger_accept_invalid_certs(true);
		tracing::debug!(domain, "Built TLS connector (accepting self-signed certs)");
		AsyncNativeTlsConnector::from(tls)
	}

	/// List directory contents, parsing LIST output.
	pub async fn list_dir(&mut self, path: &str) -> Result<Vec<ListEntry>, FtpError> {
		let lines = self.stream.list(Some(path)).await.map_err(FtpError::from)?;

		let mut entries = Vec::new();
		for line in &lines {
			if let Ok(file) = suppaftp::list::File::try_from(line.as_str()) {
				entries.push(ListEntry {
					name: file.name().to_string(),
					is_directory: file.is_directory(),
					size: file.size() as u64,
				});
			}
		}
		Ok(entries)
	}

	/// Get remote file size via SIZE command.
	pub async fn file_size(&mut self, path: &str) -> Result<u64, FtpError> {
		let size = self.stream.size(path).await.map_err(FtpError::from)?;
		Ok(size as u64)
	}

	/// Change working directory.
	pub async fn cwd(&mut self, path: &str) -> Result<(), FtpError> {
		self.stream.cwd(path).await.map_err(FtpError::from)?;
		Ok(())
	}

	/// Download a file with streaming, resume, progress reporting, cancellation, and throttling.
	/// Returns total bytes written (including resume offset).
	#[allow(clippy::too_many_arguments)]
	pub async fn download_file(
		&mut self,
		remote_path: &str,
		local_path: &Path,
		resume_offset: u64,
		item_id: Uuid,
		progress_tx: &mpsc::UnboundedSender<ProgressEvent>,
		cancel_token: &CancellationToken,
		throttle: &mut crate::download::throttle::ThrottleHandle,
	) -> Result<u64, DownloadError> {
		// Set binary transfer mode
		self.stream.transfer_type(FileType::Binary).await.map_err(FtpError::from)?;

		// Set resume offset if needed
		if resume_offset > 0 {
			tracing::debug!(item_id = %item_id, offset = resume_offset, "Resuming download");
			self.stream.resume_transfer(resume_offset as usize).await.map_err(FtpError::from)?;
		}

		// Open data stream
		let mut data_stream = self.stream.retr_as_stream(remote_path).await.map_err(FtpError::from)?;

		// Open local file (append for resume, create for new)
		let mut file = if resume_offset > 0 {
			tokio::fs::OpenOptions::new().append(true).open(local_path).await?
		} else {
			tokio::fs::File::create(local_path).await?
		};

		// DL-006: Chunked read loop with immediate cancellation via select!
		let mut buf = [0u8; 32768]; // BR-DL-008: 32KB buffer
		let mut total_written = resume_offset;

		loop {
			let n = tokio::select! {
				result = data_stream.read(&mut buf) => {
					result.map_err(|e| DownloadError::Io(std::io::Error::other(e)))?
				}
				_ = cancel_token.cancelled() => {
					drop(data_stream);
					return Err(DownloadError::Cancelled);
				}
			};

			if n == 0 {
				break; // EOF
			}

			tokio::io::AsyncWriteExt::write_all(&mut file, &buf[..n]).await?;
			total_written += n as u64;

			let _ = progress_tx.send(ProgressEvent::BytesWritten {
				item_id,
				bytes_delta: n as u64,
				total_written,
			});

			// DL-008: Throttle if over speed limit
			throttle.on_bytes_written(n as u64).await;
		}

		// Finalize the RETR transfer
		self.stream.finalize_retr_stream(data_stream).await.map_err(FtpError::from)?;

		Ok(total_written)
	}

	/// POST-001 / BR-POST-002: Query server FEAT capabilities.
	///
	/// Returns which hash commands the server supports (XMD5, XSHA1, XCRC).
	pub async fn hash_capabilities(&mut self) -> HashCapabilities {
		let features = match self.stream.feat().await {
			Ok(f) => f,
			Err(e) => {
				tracing::debug!("FEAT command failed: {e}");
				return HashCapabilities::default();
			}
		};

		HashCapabilities {
			supports_md5: features.contains_key("MD5") || features.contains_key("XMD5"),
			supports_sha1: features.contains_key("XSHA1") || features.contains_key("SHA1"),
			supports_crc: features.contains_key("XCRC") || features.contains_key("CRC"),
		}
	}

	/// POST-001 / A1: Query a server-side hash for a remote file.
	///
	/// Sends XMD5, XSHA1, or XCRC command depending on hash_type.
	/// Returns the hex hash string on success, or None if the server rejects the command.
	pub async fn server_hash(&mut self, remote_path: &str, hash_type: HashType) -> Option<String> {
		let command = match hash_type {
			HashType::MD5 => format!("XMD5 {remote_path}"),
			HashType::SHA1 => format!("XSHA1 {remote_path}"),
			HashType::CRC => format!("XCRC {remote_path}"),
			HashType::None => return None,
		};

		let expected = &[suppaftp::Status::RequestedFileActionOk, suppaftp::Status::CommandOk];

		match self.stream.custom_command(&command, expected).await {
			Ok(resp) => {
				let body = String::from_utf8_lossy(&resp.body);
				let hash = body.trim().to_string();
				// Some servers return "hash path", extract just the hash
				let hash = hash.split_whitespace().next().unwrap_or("").to_string();
				if hash.is_empty() { None } else { Some(hash) }
			}
			Err(e) => {
				tracing::debug!("Server hash command failed: {e}");
				None
			}
		}
	}

	/// Gracefully disconnect.
	pub async fn disconnect(mut self) {
		let _ = self.stream.quit().await;
	}
}

/// POST-001 / BR-POST-002: Server hash capabilities from FEAT.
#[derive(Debug, Clone, Default)]
pub struct HashCapabilities {
	pub supports_md5: bool,
	pub supports_sha1: bool,
	pub supports_crc: bool,
}
