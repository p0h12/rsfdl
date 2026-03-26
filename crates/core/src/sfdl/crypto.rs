use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};
use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use md5::{Digest, Md5};
use rand::random;

use crate::error::CryptoError;
use crate::sfdl::models::SfdlContainer;

type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;
type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;

/// Decrypts a single AES-128-CBC encrypted, Base64-encoded string.
pub fn decrypt_string(ciphertext_b64: &str, password: &str) -> Result<String, CryptoError> {
	if ciphertext_b64.is_empty() {
		return Ok(String::new());
	}

	// Try UTF-8 key derivation first, then Latin-1 fallback
	decrypt_string_with_encoding(ciphertext_b64, password.as_bytes()).or_else(|_| {
		// `as u8` truncates chars > U+00FF — correct per SFDL spec (Latin-1 encoding).
		let latin1_bytes: Vec<u8> = password.chars().map(|c| c as u8).collect();
		decrypt_string_with_encoding(ciphertext_b64, &latin1_bytes)
	})
}

fn decrypt_string_with_encoding(ciphertext_b64: &str, password_bytes: &[u8]) -> Result<String, CryptoError> {
	// Key = MD5(password)
	let mut hasher = Md5::new();
	hasher.update(password_bytes);
	let key = hasher.finalize();

	// Base64 decode
	let decoded = B64.decode(ciphertext_b64).map_err(|e| CryptoError::Base64Error(e.to_string()))?;

	if decoded.len() < 16 {
		return Err(CryptoError::DecryptionFailed("Ciphertext too short (< 16 bytes)".into()));
	}

	// IV = first 16 bytes, ciphertext = rest
	let iv = &decoded[..16];
	let ciphertext = &decoded[16..];

	if ciphertext.is_empty() || ciphertext.len() % 16 != 0 {
		return Err(CryptoError::DecryptionFailed("Ciphertext length is not a multiple of block size".into()));
	}

	// Decrypt
	let mut buf = ciphertext.to_vec();
	let plaintext = Aes128CbcDec::new(key.as_slice().into(), iv.into())
		.decrypt_padded_mut::<Pkcs7>(&mut buf)
		.map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;

	String::from_utf8(plaintext.to_vec()).map_err(|e| CryptoError::DecryptionFailed(e.to_string()))
}

/// Validates a password by attempting to decrypt the host field
/// and checking if the result looks like a valid hostname.
pub fn validate_password(container: &SfdlContainer, password: &str) -> bool {
	if !container.encrypted {
		return true;
	}

	match decrypt_string(&container.connection.host, password) {
		Ok(host) => {
			// A valid hostname should contain only printable ASCII chars
			// and typically has at least one dot or is an IP address
			!host.is_empty() && host.is_ascii() && host.chars().all(|c| c.is_alphanumeric() || ".-_:".contains(c))
		}
		Err(e) => {
			tracing::debug!("Password validation failed: {e}");
			false
		}
	}
}

/// Tries a list of passwords and returns the first one that validates.
pub fn try_passwords(container: &SfdlContainer, passwords: &[String]) -> Option<String> {
	passwords.iter().find(|pw| validate_password(container, pw)).cloned()
}

/// Apply a transform function to all encrypted fields in a container.
///
/// Single definition of which fields are encrypted — used by both
/// encrypt and decrypt to ensure they stay in sync.
fn transform_fields(container: &mut SfdlContainer, f: impl Fn(&str) -> Result<String, CryptoError>) -> Result<(), CryptoError> {
	container.description = f(&container.description)?;
	container.uploader = f(&container.uploader)?;
	container.connection.host = f(&container.connection.host)?;
	container.connection.username = f(&container.connection.username)?;
	container.connection.password = f(&container.connection.password)?;

	for package in &mut container.packages {
		package.name = f(&package.name)?;

		for file_item in &mut package.file_list {
			file_item.file_name = f(&file_item.file_name)?;
			file_item.directory_root = f(&file_item.directory_root)?;
			file_item.directory_path = f(&file_item.directory_path)?;
			file_item.full_path = f(&file_item.full_path)?;
			file_item.package_name = f(&file_item.package_name)?;
		}

		for bulk_folder in &mut package.bulk_folder_list {
			bulk_folder.bulk_folder_path = f(&bulk_folder.bulk_folder_path)?;
			bulk_folder.package_name = f(&bulk_folder.package_name)?;
		}
	}

	Ok(())
}

/// Decrypts all encrypted fields in a container in-place.
pub fn decrypt_container(container: &mut SfdlContainer, password: &str) -> Result<(), CryptoError> {
	if !container.encrypted {
		return Ok(());
	}
	transform_fields(container, |s| decrypt_string(s, password))?;
	container.encrypted = false;
	Ok(())
}

/// Encrypts a single string with AES-128-CBC and returns a Base64-encoded result.
/// Output format: base64(IV || ciphertext) with a random IV.
pub fn encrypt_string(plaintext: &str, password: &str) -> String {
	if plaintext.is_empty() {
		return String::new();
	}

	// Key = MD5(password)
	let mut hasher = Md5::new();
	hasher.update(password.as_bytes());
	let key = hasher.finalize();

	// Random IV
	let iv: [u8; 16] = random();

	// Encrypt with PKCS7 padding
	let ct = Aes128CbcEnc::new(key.as_slice().into(), iv.as_slice().into()).encrypt_padded_vec_mut::<Pkcs7>(plaintext.as_bytes());

	// Output = base64(IV || ciphertext)
	let mut result = iv.to_vec();
	result.extend_from_slice(&ct);
	B64.encode(&result)
}

/// Encrypts all sensitive fields in a container in-place.
pub fn encrypt_container(container: &mut SfdlContainer, password: &str) {
	if container.encrypted {
		return;
	}
	// encrypt_string is infallible — wrap in Ok for transform_fields
	transform_fields(container, |s| Ok(encrypt_string(s, password))).expect("encrypt_string is infallible");
	container.encrypted = true;
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::sfdl::models::Connection;

	/// Encrypts a string with AES-128-CBC using a deterministic IV for reproducible tests.
	fn encrypt_string_deterministic(plaintext: &str, password: &str) -> String {
		// Key = MD5(password)
		let mut hasher = Md5::new();
		hasher.update(password.as_bytes());
		let key = hasher.finalize();

		// Deterministic IV = MD5(password || plaintext)
		let mut iv_hasher = Md5::new();
		iv_hasher.update(password.as_bytes());
		iv_hasher.update(plaintext.as_bytes());
		let iv = iv_hasher.finalize();

		// Encrypt with PKCS7 padding
		let ct = Aes128CbcEnc::new(key.as_slice().into(), iv.as_slice().into()).encrypt_padded_vec_mut::<Pkcs7>(plaintext.as_bytes());

		// Output = base64(IV || ciphertext)
		let mut result = iv.to_vec();
		result.extend_from_slice(&ct);
		B64.encode(&result)
	}

	// --- SFDL-002: Decrypt container ---

	/// SFDL-002 | BR-SFDL-003: AES-128-CBC decryption with known test vectors.
	#[test]
	fn sfdl002_decrypt_known_values() {
		let vectors = [
			("test", "ftp.example.com", "FA9p93TaRSx1Bap096qqevmwi8vGbaEXtXRbnLmbUr8="),
			("test", "username", "M+L1kyRobW53b5zxcNjY3WSmr20xEF588T3CW4p3mdc="),
			("test", "password123", "X4rJjxT4PVWfDPFkv8evv1mZNGVZLc6DKOGq4MjnxUM="),
			("test", "/path/to/files", "go44RBh+Dt5Fszl5QuFQiRHYiEfAMCRr/5sIUvXAg7s="),
			("test", "Release.Name.2026", "tYCEYc8OgB57NgHimeQvQY5621qYe8/xL7YSpTGfbt3JG2ck/kJWSb851+tky7fp"),
		];

		for (password, expected_plaintext, ciphertext_b64) in vectors {
			let result = decrypt_string(ciphertext_b64, password).unwrap_or_else(|e| panic!("Failed to decrypt '{}' with password '{}': {}", ciphertext_b64, password, e));
			assert_eq!(result, expected_plaintext, "Decryption mismatch for password '{}'", password);
		}
	}

	/// SFDL-002 | BR-SFDL-003: Encrypt then decrypt preserves plaintext.
	#[test]
	fn sfdl002_encrypt_decrypt_round_trip() {
		let test_cases = [
			("test", "ftp.example.com"),
			("test", "username"),
			("test", "password123"),
			("test", "/path/to/files"),
			("test", "Release.Name.2026"),
			("test", ""),
			("example.org", "ftp.example.com"),
			("example.org", "anonymous"),
			("S€cr3t!", "Ünïcödé-Téxt"),
		];

		for (password, plaintext) in test_cases {
			let encrypted = encrypt_string_deterministic(plaintext, password);
			let decrypted = decrypt_string(&encrypted, password).unwrap_or_else(|e| panic!("Round-trip failed for password='{}', plaintext='{}': {}", password, plaintext, e));
			assert_eq!(decrypted, plaintext, "Round-trip mismatch for password='{}'", password);
		}
	}

	/// SFDL-002 | BR-SFDL-003: Empty ciphertext returns empty string.
	#[test]
	fn sfdl002_decrypt_empty_string() {
		let result = decrypt_string("", "test").unwrap();
		assert_eq!(result, "");
	}

	/// SFDL-002 | A2: Wrong password produces wrong output or error.
	#[test]
	fn sfdl002_decrypt_wrong_password() {
		let ciphertext = "FA9p93TaRSx1Bap096qqevmwi8vGbaEXtXRbnLmbUr8=";
		let result = decrypt_string(ciphertext, "wrong_password");
		// Should either fail or produce garbage (not "ftp.example.com")
		match result {
			Err(_) => {} // Expected: decryption error
			Ok(text) => assert_ne!(text, "ftp.example.com"),
		}
	}

	/// SFDL-002 | A2: Invalid Base64 returns error.
	#[test]
	fn sfdl002_decrypt_invalid_base64() {
		let result = decrypt_string("not-valid-base64!!!", "test");
		assert!(result.is_err());
	}

	/// SFDL-002 | A2: Too-short ciphertext returns error.
	#[test]
	fn sfdl002_decrypt_too_short() {
		// Valid base64 but too short for AES (< 16 bytes)
		let result = decrypt_string("AQIDBA==", "test");
		assert!(result.is_err());
	}

	/// SFDL-002 | BR-SFDL-004: Correct password validates successfully.
	#[test]
	fn sfdl002_validate_password_correct() {
		let container = SfdlContainer {
			encrypted: true,
			connection: Connection {
				host: "FA9p93TaRSx1Bap096qqevmwi8vGbaEXtXRbnLmbUr8=".into(),
				..Connection::default()
			},
			..SfdlContainer::default()
		};
		assert!(validate_password(&container, "test"));
	}

	/// SFDL-002 | BR-SFDL-004: Wrong password fails validation.
	#[test]
	fn sfdl002_validate_password_wrong() {
		let container = SfdlContainer {
			encrypted: true,
			connection: Connection {
				host: "FA9p93TaRSx1Bap096qqevmwi8vGbaEXtXRbnLmbUr8=".into(),
				..Connection::default()
			},
			..SfdlContainer::default()
		};
		assert!(!validate_password(&container, "wrong"));
	}

	/// SFDL-002 | BR-SFDL-004: Unencrypted container always validates.
	#[test]
	fn sfdl002_validate_password_unencrypted() {
		let container = SfdlContainer {
			encrypted: false,
			..SfdlContainer::default()
		};
		assert!(validate_password(&container, "anything"));
	}

	/// SFDL-002 | Main Success: Auto-password list finds the correct one.
	#[test]
	fn sfdl002_try_passwords_finds_correct() {
		let container = SfdlContainer {
			encrypted: true,
			connection: Connection {
				host: "FA9p93TaRSx1Bap096qqevmwi8vGbaEXtXRbnLmbUr8=".into(),
				..Connection::default()
			},
			..SfdlContainer::default()
		};
		let passwords = vec!["wrong1".into(), "test".into(), "wrong2".into()];
		assert_eq!(try_passwords(&container, &passwords), Some("test".into()));
	}

	/// SFDL-002 | A1: No auto-password matches.
	#[test]
	fn sfdl002_try_passwords_none_match() {
		let container = SfdlContainer {
			encrypted: true,
			connection: Connection {
				host: "FA9p93TaRSx1Bap096qqevmwi8vGbaEXtXRbnLmbUr8=".into(),
				..Connection::default()
			},
			..SfdlContainer::default()
		};
		let passwords = vec!["wrong1".into(), "wrong2".into()];
		assert_eq!(try_passwords(&container, &passwords), None);
	}

	// --- CR-004: Encrypt container ---

	/// CR-004 | Main Success: Public encrypt/decrypt round-trip.
	#[test]
	fn cr004_encrypt_decrypt_round_trip() {
		let cases = [("test", "ftp.example.com"), ("test", "username"), ("S€cr3t!", "Ünïcödé-Téxt"), ("pw", "")];
		for (password, plaintext) in cases {
			let encrypted = encrypt_string(plaintext, password);
			if plaintext.is_empty() {
				assert_eq!(encrypted, "");
				continue;
			}
			let decrypted = decrypt_string(&encrypted, password).unwrap();
			assert_eq!(decrypted, plaintext);
		}
	}

	/// CR-004 | BR: Random IV produces different ciphertexts.
	#[test]
	fn cr004_encrypt_random_iv_different_ciphertexts() {
		let a = encrypt_string("hello", "test");
		let b = encrypt_string("hello", "test");
		assert_ne!(a, b, "Two encryptions should produce different ciphertexts due to random IV");
		// But both should decrypt to the same thing
		assert_eq!(decrypt_string(&a, "test").unwrap(), "hello");
		assert_eq!(decrypt_string(&b, "test").unwrap(), "hello");
	}

	/// CR-004 | Main Success: Full container encrypt/decrypt round-trip.
	#[test]
	fn cr004_encrypt_container_round_trip() {
		use crate::sfdl::models::{BulkFolder, FileItem, Package};

		let mut container = SfdlContainer {
			description: "Test.Release".into(),
			uploader: "user123".into(),
			encrypted: false,
			connection: Connection {
				host: "ftp.example.com".into(),
				username: "ftpuser".into(),
				password: "ftppass".into(),
				..Connection::default()
			},
			packages: vec![Package {
				name: "Pkg1".into(),
				bulk_folder_mode: false,
				file_list: vec![FileItem {
					file_name: "movie.rar".into(),
					directory_root: "/".into(),
					directory_path: "releases".into(),
					full_path: "/releases/movie.rar".into(),
					file_size: 1024,
					package_name: "Pkg1".into(),
					..FileItem::default()
				}],
				bulk_folder_list: vec![BulkFolder {
					bulk_folder_path: "/releases/".into(),
					package_name: "Pkg1".into(),
				}],
			}],
			..SfdlContainer::default()
		};

		// Save originals
		let orig_desc = container.description.clone();
		let orig_host = container.connection.host.clone();
		let orig_user = container.connection.username.clone();
		let orig_pw = container.connection.password.clone();
		let orig_file_name = container.packages[0].file_list[0].file_name.clone();
		let orig_bulk_path = container.packages[0].bulk_folder_list[0].bulk_folder_path.clone();

		// Encrypt
		encrypt_container(&mut container, "secret");
		assert!(container.encrypted);
		assert_ne!(container.description, orig_desc);
		assert_ne!(container.connection.host, orig_host);

		// Decrypt
		decrypt_container(&mut container, "secret").unwrap();
		assert!(!container.encrypted);
		assert_eq!(container.description, orig_desc);
		assert_eq!(container.connection.host, orig_host);
		assert_eq!(container.connection.username, orig_user);
		assert_eq!(container.connection.password, orig_pw);
		assert_eq!(container.packages[0].file_list[0].file_name, orig_file_name);
		assert_eq!(container.packages[0].bulk_folder_list[0].bulk_folder_path, orig_bulk_path);
	}

	/// CR-004 | BR: Already encrypted container is a no-op.
	#[test]
	fn cr004_encrypt_already_encrypted_is_noop() {
		let mut container = SfdlContainer {
			encrypted: true,
			description: "already-encrypted-data".into(),
			..SfdlContainer::default()
		};
		encrypt_container(&mut container, "test");
		assert_eq!(container.description, "already-encrypted-data");
	}
}
