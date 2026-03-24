use thiserror::Error;

#[derive(Debug, Error)]
pub enum SfdlError {
	#[error("Failed to parse SFDL file: {0}")]
	ParseError(String),

	#[error("Unsupported SFDL version: {0}")]
	UnsupportedVersion(u32),

	#[error("Crypto error: {0}")]
	Crypto(#[from] CryptoError),

	#[error("Failed to serialize SFDL file: {0}")]
	SerializeError(String),

	#[error("IO error: {0}")]
	Io(#[from] std::io::Error),

	#[error("File error: {0}")]
	FileError(String),
}

#[derive(Debug, Error)]
pub enum CryptoError {
	#[error("Decryption failed: {0}")]
	DecryptionFailed(String),

	#[error("Invalid password")]
	InvalidPassword,

	#[error("Base64 decode error: {0}")]
	Base64Error(String),
}

#[derive(Debug, Error)]
pub enum FtpError {
	#[error("Connection failed: {0}")]
	ConnectionFailed(String),

	#[error("Authentication failed")]
	AuthFailed,

	#[error("Transfer error: {0}")]
	TransferError(String),

	#[error("Listing error: {0}")]
	ListingError(String),

	#[error("Connection timeout")]
	Timeout,
}

impl From<suppaftp::FtpError> for FtpError {
	fn from(err: suppaftp::FtpError) -> Self {
		match &err {
			suppaftp::FtpError::ConnectionError(_) => FtpError::ConnectionFailed(err.to_string()),
			suppaftp::FtpError::SecureError(msg) => FtpError::ConnectionFailed(format!("TLS error: {msg}")),
			suppaftp::FtpError::UnexpectedResponse(resp) => {
				let code = resp.status.code();
				match code {
					430 | 530 => FtpError::AuthFailed,
					421 => FtpError::ConnectionFailed(format!("Server unavailable ({})", code)),
					_ => FtpError::TransferError(format!("FTP {}: {}", code, resp.status)),
				}
			}
			suppaftp::FtpError::BadResponse => FtpError::TransferError("Bad FTP response".into()),
			suppaftp::FtpError::InvalidAddress(e) => FtpError::ConnectionFailed(e.to_string()),
		}
	}
}

#[derive(Debug, Error)]
pub enum ExtractionError {
	#[error("RAR extraction failed: {0}")]
	Rar(String),

	#[error("ZIP extraction failed: {0}")]
	Zip(String),

	#[error("Password-protected archive")]
	PasswordProtected,

	#[error("IO error: {0}")]
	Io(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum DownloadError {
	#[error("FTP error: {0}")]
	Ftp(#[from] FtpError),

	#[error("IO error: {0}")]
	Io(#[from] std::io::Error),

	#[error("Cancelled")]
	Cancelled,

	#[error("Insufficient disk space")]
	InsufficientDiskSpace,
}

impl DownloadError {
	/// DL-007 / BR-DL-010: Whether this error type is retryable.
	///
	/// Retryable: ConnectionFailed, AuthFailed, Timeout (transient FTP errors)
	/// Permanent: IO errors, Cancelled, InsufficientDiskSpace, FileNotFound (550)
	pub fn is_retryable(&self) -> bool {
		match self {
			DownloadError::Ftp(ftp_err) => ftp_err.is_retryable(),
			DownloadError::Io(_) => false,
			DownloadError::Cancelled => false,
			DownloadError::InsufficientDiskSpace => false,
		}
	}

	/// Whether this is a ServerFull (421) error that warrants exponential backoff.
	pub fn is_server_full(&self) -> bool {
		matches!(self, DownloadError::Ftp(FtpError::ConnectionFailed(msg)) if msg.contains("421"))
	}
}

impl FtpError {
	/// Whether this FTP error is retryable.
	pub fn is_retryable(&self) -> bool {
		match self {
			FtpError::ConnectionFailed(_) => true, // includes 421 ServerFull
			FtpError::AuthFailed => true,          // may be temporary rate limit
			FtpError::Timeout => true,
			FtpError::TransferError(msg) => {
				// 550 FileNotFound is permanent
				!msg.contains("550") && !msg.contains("FileNotFound")
			}
			FtpError::ListingError(_) => false,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn suppaftp_connection_error_maps_to_connection_failed() {
		let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
		let suppa_err = suppaftp::FtpError::ConnectionError(io_err);
		let our_err: FtpError = suppa_err.into();
		assert!(matches!(our_err, FtpError::ConnectionFailed(_)));
		assert!(our_err.to_string().contains("refused"));
	}

	#[test]
	fn suppaftp_bad_response_maps_to_transfer_error() {
		let suppa_err = suppaftp::FtpError::BadResponse;
		let our_err: FtpError = suppa_err.into();
		assert!(matches!(our_err, FtpError::TransferError(_)));
	}

	#[test]
	fn suppaftp_530_maps_to_auth_failed() {
		let resp = suppaftp::types::Response {
			status: suppaftp::Status::from(530),
			body: Vec::new(),
		};
		let suppa_err = suppaftp::FtpError::UnexpectedResponse(resp);
		let our_err: FtpError = suppa_err.into();
		assert!(matches!(our_err, FtpError::AuthFailed));
	}

	// --- UC-14: ExtractionError ---

	#[test]
	fn extraction_error_rar_display() {
		let err = ExtractionError::Rar("CRC mismatch".into());
		assert_eq!(err.to_string(), "RAR extraction failed: CRC mismatch");
	}

	#[test]
	fn extraction_error_zip_display() {
		let err = ExtractionError::Zip("invalid header".into());
		assert_eq!(err.to_string(), "ZIP extraction failed: invalid header");
	}

	#[test]
	fn extraction_error_password_display() {
		let err = ExtractionError::PasswordProtected;
		assert_eq!(err.to_string(), "Password-protected archive");
	}

	#[test]
	fn extraction_error_io_from_conversion() {
		let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
		let err: ExtractionError = io_err.into();
		assert!(matches!(err, ExtractionError::Io(_)));
		assert!(err.to_string().contains("not found"));
	}
}

#[derive(Debug, Error)]
pub enum SettingsError {
	#[error("IO error: {0}")]
	Io(#[from] std::io::Error),

	#[error("TOML serialize error: {0}")]
	TomlSerialize(String),

	#[error("Validation failed: {0}")]
	Validation(String),
}

#[derive(Debug, Error)]
pub enum AppError {
	#[error("Parse error: {0}")]
	Parse(#[from] SfdlError),

	#[error("Decryption failed: {0}")]
	Decrypt(#[from] CryptoError),

	#[error("Invalid password")]
	InvalidPassword,

	#[error("FTP error: {0}")]
	Ftp(#[from] FtpError),
}
