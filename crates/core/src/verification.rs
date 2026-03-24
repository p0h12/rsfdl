use std::path::Path;

use crc32fast::Hasher as Crc32Hasher;
use md5::Md5;
use sha1::Sha1;
use tokio::io::AsyncReadExt;

use crate::error::VerificationError;
use crate::sfdl::models::HashType;

/// POST-001: Outcome of a single file hash verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationOutcome {
	/// Hash matched.
	Valid,
	/// Hash did not match.
	Invalid,
	/// No hash available (neither container nor server).
	NoHash,
}

/// POST-001: Full verification result for a file.
#[derive(Debug, Clone)]
pub struct HashVerification {
	pub hash_type: HashType,
	pub expected: String,
	pub actual: String,
	pub outcome: VerificationOutcome,
}

/// POST-001: Verify a downloaded file against an expected hash.
///
/// If `expected_hash` is empty and `hash_type` is None, returns `NoHash`.
/// Otherwise computes the local hash and compares.
pub async fn verify_file(local_path: &Path, hash_type: HashType, expected_hash: &str) -> Result<HashVerification, VerificationError> {
	if hash_type == HashType::None || expected_hash.is_empty() {
		return Ok(HashVerification {
			hash_type,
			expected: expected_hash.to_string(),
			actual: String::new(),
			outcome: VerificationOutcome::NoHash,
		});
	}

	let actual = compute_hash(local_path, hash_type).await?;
	let outcome = if actual.eq_ignore_ascii_case(expected_hash) {
		VerificationOutcome::Valid
	} else {
		VerificationOutcome::Invalid
	};

	Ok(HashVerification {
		hash_type,
		expected: expected_hash.to_string(),
		actual,
		outcome,
	})
}

/// POST-001: Verify using a server-provided hash (A1 fallback).
///
/// `server_hash` is the hex string returned by the FTP server (XMD5, XSHA1, XCRC).
pub async fn verify_file_with_server_hash(local_path: &Path, hash_type: HashType, server_hash: &str) -> Result<HashVerification, VerificationError> {
	let actual = compute_hash(local_path, hash_type).await?;
	let outcome = if actual.eq_ignore_ascii_case(server_hash) {
		VerificationOutcome::Valid
	} else {
		VerificationOutcome::Invalid
	};

	Ok(HashVerification {
		hash_type,
		expected: server_hash.to_string(),
		actual,
		outcome,
	})
}

/// Compute a hash of the file at `path` using the specified algorithm.
///
/// BR-POST-001: Supports SHA1, MD5, CRC32.
pub async fn compute_hash(path: &Path, hash_type: HashType) -> Result<String, VerificationError> {
	let mut file = tokio::fs::File::open(path).await.map_err(VerificationError::Io)?;

	let mut buf = [0u8; 65536];

	match hash_type {
		HashType::MD5 => {
			use md5::Digest;
			let mut hasher = Md5::new();
			loop {
				let n = file.read(&mut buf).await.map_err(VerificationError::Io)?;
				if n == 0 {
					break;
				}
				hasher.update(&buf[..n]);
			}
			Ok(format!("{:x}", hasher.finalize()))
		}
		HashType::SHA1 => {
			use sha1::Digest;
			let mut hasher = Sha1::new();
			loop {
				let n = file.read(&mut buf).await.map_err(VerificationError::Io)?;
				if n == 0 {
					break;
				}
				hasher.update(&buf[..n]);
			}
			Ok(format!("{:x}", hasher.finalize()))
		}
		HashType::CRC => {
			let mut hasher = Crc32Hasher::new();
			loop {
				let n = file.read(&mut buf).await.map_err(VerificationError::Io)?;
				if n == 0 {
					break;
				}
				hasher.update(&buf[..n]);
			}
			Ok(format!("{:08x}", hasher.finalize()))
		}
		HashType::None => Ok(String::new()),
	}
}

/// BR-POST-001: Select the strongest hash type from FEAT capabilities.
///
/// Priority: SHA1 > MD5 > CRC32.
pub fn select_strongest_hash(supports_sha1: bool, supports_md5: bool, supports_crc: bool) -> Option<HashType> {
	if supports_sha1 {
		Some(HashType::SHA1)
	} else if supports_md5 {
		Some(HashType::MD5)
	} else if supports_crc {
		Some(HashType::CRC)
	} else {
		None
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::io::Write;

	/// POST-001 | Main Success: MD5 hash matches → Valid.
	#[tokio::test]
	async fn post001_verify_md5_valid() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("test.bin");
		{
			let mut f = std::fs::File::create(&path).unwrap();
			f.write_all(b"hello world").unwrap();
		}
		// MD5 of "hello world" = 5eb63bbbe01eeed093cb22bb8f5acdc3
		let result = verify_file(&path, HashType::MD5, "5eb63bbbe01eeed093cb22bb8f5acdc3").await.unwrap();
		assert_eq!(result.outcome, VerificationOutcome::Valid);
		assert_eq!(result.actual, "5eb63bbbe01eeed093cb22bb8f5acdc3");
	}

	/// POST-001 | A2: MD5 hash mismatch → Invalid.
	#[tokio::test]
	async fn post001_verify_md5_invalid() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("test.bin");
		{
			let mut f = std::fs::File::create(&path).unwrap();
			f.write_all(b"hello world").unwrap();
		}
		let result = verify_file(&path, HashType::MD5, "0000000000000000000000000000000").await.unwrap();
		assert_eq!(result.outcome, VerificationOutcome::Invalid);
	}

	/// POST-001 | Main Success: SHA1 hash matches → Valid.
	#[tokio::test]
	async fn post001_verify_sha1_valid() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("test.bin");
		{
			let mut f = std::fs::File::create(&path).unwrap();
			f.write_all(b"hello world").unwrap();
		}
		// SHA1 of "hello world" = 2aae6c35c94fcfb415dbe95f408b9ce91ee846ed
		let result = verify_file(&path, HashType::SHA1, "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed").await.unwrap();
		assert_eq!(result.outcome, VerificationOutcome::Valid);
	}

	/// POST-001 | A2: SHA1 hash mismatch → Invalid.
	#[tokio::test]
	async fn post001_verify_sha1_invalid() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("test.bin");
		{
			let mut f = std::fs::File::create(&path).unwrap();
			f.write_all(b"hello world").unwrap();
		}
		let result = verify_file(&path, HashType::SHA1, "000000000000000000000000000000000000000").await.unwrap();
		assert_eq!(result.outcome, VerificationOutcome::Invalid);
	}

	/// POST-001 | Main Success: CRC32 hash matches → Valid.
	#[tokio::test]
	async fn post001_verify_crc32_valid() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("test.bin");
		{
			let mut f = std::fs::File::create(&path).unwrap();
			f.write_all(b"hello world").unwrap();
		}
		// CRC32 of "hello world" = 0d4a1185
		let result = verify_file(&path, HashType::CRC, "0d4a1185").await.unwrap();
		assert_eq!(result.outcome, VerificationOutcome::Valid);
	}

	/// POST-001 | A2: CRC32 hash mismatch → Invalid.
	#[tokio::test]
	async fn post001_verify_crc32_invalid() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("test.bin");
		{
			let mut f = std::fs::File::create(&path).unwrap();
			f.write_all(b"hello world").unwrap();
		}
		let result = verify_file(&path, HashType::CRC, "00000000").await.unwrap();
		assert_eq!(result.outcome, VerificationOutcome::Invalid);
	}

	/// POST-001 | A1: No hash in container (HashType::None) → NoHash.
	#[tokio::test]
	async fn post001_verify_no_hash_type_none() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("test.bin");
		std::fs::write(&path, b"data").unwrap();

		let result = verify_file(&path, HashType::None, "").await.unwrap();
		assert_eq!(result.outcome, VerificationOutcome::NoHash);
	}

	/// POST-001 | A1: Hash type set but empty hash string → NoHash.
	#[tokio::test]
	async fn post001_verify_empty_hash_string() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("test.bin");
		std::fs::write(&path, b"data").unwrap();

		let result = verify_file(&path, HashType::MD5, "").await.unwrap();
		assert_eq!(result.outcome, VerificationOutcome::NoHash);
	}

	/// POST-001 | Main Success: Case-insensitive hash comparison.
	#[tokio::test]
	async fn post001_verify_case_insensitive() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("test.bin");
		std::fs::write(&path, b"hello world").unwrap();

		let result = verify_file(&path, HashType::MD5, "5EB63BBBE01EEED093CB22BB8F5ACDC3").await.unwrap();
		assert_eq!(result.outcome, VerificationOutcome::Valid);
	}

	/// POST-001 | Error: File not found → VerificationError::Io.
	#[tokio::test]
	async fn post001_verify_file_not_found() {
		let result = verify_file(Path::new("/nonexistent_xyz_123"), HashType::MD5, "abc").await;
		assert!(result.is_err());
	}

	/// POST-001 | BR-POST-001: Hash priority SHA1 > MD5 > CRC32.
	#[test]
	fn post001_select_strongest_sha1_preferred() {
		assert_eq!(select_strongest_hash(true, true, true), Some(HashType::SHA1));
	}

	/// POST-001 | BR-POST-001: MD5 selected when no SHA1.
	#[test]
	fn post001_select_strongest_md5_fallback() {
		assert_eq!(select_strongest_hash(false, true, true), Some(HashType::MD5));
	}

	/// POST-001 | BR-POST-001: CRC32 selected when no SHA1/MD5.
	#[test]
	fn post001_select_strongest_crc_fallback() {
		assert_eq!(select_strongest_hash(false, false, true), Some(HashType::CRC));
	}

	/// POST-001 | BR-POST-001: No hash capability → None.
	#[test]
	fn post001_select_strongest_none() {
		assert_eq!(select_strongest_hash(false, false, false), None);
	}

	/// POST-001 | compute_hash: MD5 of empty file.
	#[tokio::test]
	async fn post001_compute_md5_empty_file() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("empty.bin");
		std::fs::File::create(&path).unwrap();

		let hash = compute_hash(&path, HashType::MD5).await.unwrap();
		// MD5 of empty = d41d8cd98f00b204e9800998ecf8427e
		assert_eq!(hash, "d41d8cd98f00b204e9800998ecf8427e");
	}

	/// POST-001 | compute_hash: SHA1 of empty file.
	#[tokio::test]
	async fn post001_compute_sha1_empty_file() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("empty.bin");
		std::fs::File::create(&path).unwrap();

		let hash = compute_hash(&path, HashType::SHA1).await.unwrap();
		// SHA1 of empty = da39a3ee5e6b4b0d3255bfef95601890afd80709
		assert_eq!(hash, "da39a3ee5e6b4b0d3255bfef95601890afd80709");
	}

	/// POST-001 | compute_hash: CRC32 of empty file.
	#[tokio::test]
	async fn post001_compute_crc32_empty_file() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("empty.bin");
		std::fs::File::create(&path).unwrap();

		let hash = compute_hash(&path, HashType::CRC).await.unwrap();
		assert_eq!(hash, "00000000");
	}

	/// POST-001 | compute_hash: None hash type returns empty string.
	#[tokio::test]
	async fn post001_compute_hash_none() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("any.bin");
		std::fs::write(&path, b"data").unwrap();

		let hash = compute_hash(&path, HashType::None).await.unwrap();
		assert!(hash.is_empty());
	}

	/// POST-001 | A1: Server hash fallback valid.
	#[tokio::test]
	async fn post001_verify_server_hash_valid() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("test.bin");
		std::fs::write(&path, b"hello world").unwrap();

		let result = verify_file_with_server_hash(&path, HashType::MD5, "5eb63bbbe01eeed093cb22bb8f5acdc3").await.unwrap();
		assert_eq!(result.outcome, VerificationOutcome::Valid);
	}

	/// POST-001 | A1+A2: Server hash fallback mismatch.
	#[tokio::test]
	async fn post001_verify_server_hash_mismatch() {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("test.bin");
		std::fs::write(&path, b"hello world").unwrap();

		let result = verify_file_with_server_hash(&path, HashType::MD5, "badhash").await.unwrap();
		assert_eq!(result.outcome, VerificationOutcome::Invalid);
	}
}
