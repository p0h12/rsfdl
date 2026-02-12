use aes::cipher::{BlockDecryptMut, KeyIvInit, block_padding::Pkcs7};
use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use md5::{Digest, Md5};

use crate::error::CryptoError;
use crate::sfdl::models::SfdlContainer;

type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

/// Decrypts a single AES-128-CBC encrypted, Base64-encoded string.
pub fn decrypt_string(ciphertext_b64: &str, password: &str) -> Result<String, CryptoError> {
    if ciphertext_b64.is_empty() {
        return Ok(String::new());
    }

    // Try UTF-8 key derivation first, then Latin-1 fallback
    decrypt_string_with_encoding(ciphertext_b64, password.as_bytes()).or_else(|_| {
        let latin1_bytes: Vec<u8> = password.chars().map(|c| c as u8).collect();
        decrypt_string_with_encoding(ciphertext_b64, &latin1_bytes)
    })
}

fn decrypt_string_with_encoding(
    ciphertext_b64: &str,
    password_bytes: &[u8],
) -> Result<String, CryptoError> {
    // Key = MD5(password)
    let mut hasher = Md5::new();
    hasher.update(password_bytes);
    let key = hasher.finalize();

    // Base64 decode
    let decoded = B64
        .decode(ciphertext_b64)
        .map_err(|e| CryptoError::Base64Error(e.to_string()))?;

    if decoded.len() < 16 {
        return Err(CryptoError::DecryptionFailed(
            "Ciphertext too short (< 16 bytes)".into(),
        ));
    }

    // IV = first 16 bytes, ciphertext = rest
    let iv = &decoded[..16];
    let ciphertext = &decoded[16..];

    if ciphertext.is_empty() || ciphertext.len() % 16 != 0 {
        return Err(CryptoError::DecryptionFailed(
            "Ciphertext length is not a multiple of block size".into(),
        ));
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
            !host.is_empty()
                && host.is_ascii()
                && host
                    .chars()
                    .all(|c| c.is_alphanumeric() || ".-_:".contains(c))
        }
        Err(_) => false,
    }
}

/// Tries a list of passwords and returns the first one that validates.
pub fn try_passwords(container: &SfdlContainer, passwords: &[String]) -> Option<String> {
    passwords
        .iter()
        .find(|pw| validate_password(container, pw))
        .cloned()
}

/// Decrypts all encrypted fields in a container in-place.
pub fn decrypt_container(container: &mut SfdlContainer, password: &str) -> Result<(), CryptoError> {
    if !container.encrypted {
        return Ok(());
    }

    let d = |s: &str| decrypt_string(s, password);

    container.description = d(&container.description)?;
    container.uploader = d(&container.uploader)?;
    container.connection.host = d(&container.connection.host)?;
    container.connection.username = d(&container.connection.username)?;
    container.connection.password = d(&container.connection.password)?;

    for package in &mut container.packages {
        package.name = d(&package.name)?;

        for file_item in &mut package.file_list {
            file_item.file_name = d(&file_item.file_name)?;
            file_item.directory_root = d(&file_item.directory_root)?;
            file_item.directory_path = d(&file_item.directory_path)?;
            file_item.full_path = d(&file_item.full_path)?;
            file_item.package_name = d(&file_item.package_name)?;
        }

        for bulk_folder in &mut package.bulk_folder_list {
            bulk_folder.bulk_folder_path = d(&bulk_folder.bulk_folder_path)?;
            bulk_folder.package_name = d(&bulk_folder.package_name)?;
        }
    }

    container.encrypted = false;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sfdl::models::Connection;
    use aes::cipher::BlockEncryptMut;

    type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;

    /// Encrypts a string with AES-128-CBC (test helper, mirrors decrypt_string).
    /// Uses a deterministic IV derived from MD5(password || plaintext) for reproducibility.
    fn encrypt_string(plaintext: &str, password: &str) -> String {
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
        let ct = Aes128CbcEnc::new(key.as_slice().into(), iv.as_slice().into())
            .encrypt_padded_vec_mut::<Pkcs7>(plaintext.as_bytes());

        // Output = base64(IV || ciphertext)
        let mut result = iv.to_vec();
        result.extend_from_slice(&ct);
        B64.encode(&result)
    }

    // Static test vectors generated with Go reference implementation.
    // These serve as cross-implementation regression tests.
    #[test]
    fn decrypt_known_values() {
        let vectors = [
            (
                "test",
                "ftp.example.com",
                "FA9p93TaRSx1Bap096qqevmwi8vGbaEXtXRbnLmbUr8=",
            ),
            (
                "test",
                "username",
                "M+L1kyRobW53b5zxcNjY3WSmr20xEF588T3CW4p3mdc=",
            ),
            (
                "test",
                "password123",
                "X4rJjxT4PVWfDPFkv8evv1mZNGVZLc6DKOGq4MjnxUM=",
            ),
            (
                "test",
                "/path/to/files",
                "go44RBh+Dt5Fszl5QuFQiRHYiEfAMCRr/5sIUvXAg7s=",
            ),
            (
                "test",
                "Release.Name.2026",
                "tYCEYc8OgB57NgHimeQvQY5621qYe8/xL7YSpTGfbt3JG2ck/kJWSb851+tky7fp",
            ),
        ];

        for (password, expected_plaintext, ciphertext_b64) in vectors {
            let result = decrypt_string(ciphertext_b64, password).unwrap_or_else(|e| {
                panic!(
                    "Failed to decrypt '{}' with password '{}': {}",
                    ciphertext_b64, password, e
                )
            });
            assert_eq!(
                result, expected_plaintext,
                "Decryption mismatch for password '{}'",
                password
            );
        }
    }

    #[test]
    fn encrypt_decrypt_round_trip() {
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
            let encrypted = encrypt_string(plaintext, password);
            let decrypted = decrypt_string(&encrypted, password).unwrap_or_else(|e| {
                panic!(
                    "Round-trip failed for password='{}', plaintext='{}': {}",
                    password, plaintext, e
                )
            });
            assert_eq!(
                decrypted, plaintext,
                "Round-trip mismatch for password='{}'",
                password
            );
        }
    }

    #[test]
    fn decrypt_empty_string() {
        let result = decrypt_string("", "test").unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn decrypt_wrong_password() {
        let ciphertext = "FA9p93TaRSx1Bap096qqevmwi8vGbaEXtXRbnLmbUr8=";
        let result = decrypt_string(ciphertext, "wrong_password");
        // Should either fail or produce garbage (not "ftp.example.com")
        match result {
            Err(_) => {} // Expected: decryption error
            Ok(text) => assert_ne!(text, "ftp.example.com"),
        }
    }

    #[test]
    fn decrypt_invalid_base64() {
        let result = decrypt_string("not-valid-base64!!!", "test");
        assert!(result.is_err());
    }

    #[test]
    fn decrypt_too_short() {
        // Valid base64 but too short for AES (< 16 bytes)
        let result = decrypt_string("AQIDBA==", "test");
        assert!(result.is_err());
    }

    #[test]
    fn validate_password_correct() {
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

    #[test]
    fn validate_password_wrong() {
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

    #[test]
    fn validate_password_unencrypted() {
        let container = SfdlContainer {
            encrypted: false,
            ..SfdlContainer::default()
        };
        assert!(validate_password(&container, "anything"));
    }

    #[test]
    fn try_passwords_finds_correct() {
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

    #[test]
    fn try_passwords_none_match() {
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
}
