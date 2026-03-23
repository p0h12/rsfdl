use std::path::Path;
use std::time::Duration;

use futures_lite::AsyncReadExt;
use suppaftp::types::FileType;
use suppaftp::{AsyncNativeTlsFtpStream, Mode};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::download::progress::ProgressEvent;
use crate::error::{DownloadError, FtpError};
use crate::sfdl::models::{Connection, SslProtocol};

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
	pub async fn connect(conn: &Connection, timeout_seconds: u32) -> Result<Self, FtpError> {
		// Warn if container requires TLS (not yet implemented)
		if conn.ssl_protocol != SslProtocol::None {
			tracing::warn!(
					protocol = ?conn.ssl_protocol,
					"Container requires TLS ({:?}), but TLS is not yet supported. Connecting without encryption.",
					conn.ssl_protocol,
			);
		}

		let addr = format!("{}:{}", conn.host, conn.port);

		let mut stream = if timeout_seconds > 0 {
			tokio::time::timeout(Duration::from_secs(timeout_seconds as u64), AsyncNativeTlsFtpStream::connect(&addr))
				.await
				.map_err(|_| FtpError::Timeout)?
				.map_err(FtpError::from)?
		} else {
			AsyncNativeTlsFtpStream::connect(&addr).await.map_err(FtpError::from)?
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

		tracing::debug!(host = %conn.host, port = conn.port, "FTP connected");

		Ok(Self { stream })
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

	/// Download a file with streaming, resume, progress reporting, and cancellation.
	/// Returns total bytes written (including resume offset).
	pub async fn download_file(
		&mut self,
		remote_path: &str,
		local_path: &Path,
		resume_offset: u64,
		item_id: Uuid,
		progress_tx: &mpsc::UnboundedSender<ProgressEvent>,
		cancel_token: &CancellationToken,
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

		// Chunked read loop
		let mut buf = [0u8; 32768]; // 32KB buffer
		let mut total_written = resume_offset;

		loop {
			// Check cancellation
			if cancel_token.is_cancelled() {
				drop(data_stream);
				return Err(DownloadError::Cancelled);
			}

			let n = data_stream.read(&mut buf).await.map_err(|e| DownloadError::Io(std::io::Error::other(e)))?;

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
		}

		// Finalize the RETR transfer
		self.stream.finalize_retr_stream(data_stream).await.map_err(FtpError::from)?;

		Ok(total_written)
	}

	/// Gracefully disconnect.
	pub async fn disconnect(mut self) {
		let _ = self.stream.quit().await;
	}
}
