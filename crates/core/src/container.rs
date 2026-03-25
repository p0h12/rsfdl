use crate::ftp::listing::resolve_bulk_folder;
use crate::selection::FileSelection;
use crate::sfdl::crypto::{decrypt_container, try_passwords, validate_password};
use crate::sfdl::models::SfdlContainer;
use crate::sfdl::parser::parse_sfdl;

use crate::error::AppError;

/// Result of loading and attempting auto-decryption of an SFDL container.
pub struct LoadedContainer {
	pub container: SfdlContainer,
	pub status: DecryptionStatus,
}

/// How decryption was resolved during loading.
#[derive(Debug)]
pub enum DecryptionStatus {
	/// Container was not encrypted.
	NotEncrypted,
	/// Auto-decrypted using a password from the provided list.
	AutoDecrypted { password: String },
	/// Encrypted but no auto-password matched — manual password required.
	NeedsPassword,
}

/// SFDL-001 + SFDL-002: Parse SFDL XML and attempt auto-decryption.
///
/// If the container is encrypted and a matching password is found in `auto_passwords`,
/// decryption is performed automatically. Otherwise `DecryptionStatus::NeedsPassword`
/// is returned and the caller must use [`decrypt_with_password`] to proceed.
pub fn load_sfdl(xml: &str, auto_passwords: &[String]) -> Result<LoadedContainer, AppError> {
	let mut container = parse_sfdl(xml)?;

	if !container.encrypted {
		return Ok(LoadedContainer {
			container,
			status: DecryptionStatus::NotEncrypted,
		});
	}

	if let Some(pw) = try_passwords(&container, auto_passwords) {
		decrypt_container(&mut container, &pw)?;
		return Ok(LoadedContainer {
			container,
			status: DecryptionStatus::AutoDecrypted { password: pw },
		});
	}

	Ok(LoadedContainer {
		container,
		status: DecryptionStatus::NeedsPassword,
	})
}

/// SFDL-002: Decrypt a container with a user-provided password.
///
/// Validates the password first, then decrypts all encrypted fields in-place.
pub fn decrypt_with_password(container: &mut SfdlContainer, password: &str) -> Result<(), AppError> {
	if !validate_password(container, password) {
		return Err(AppError::InvalidPassword);
	}
	decrypt_container(container, password)?;
	Ok(())
}

/// DL-001 + DL-002: Compute file selection with exclusion patterns.
///
/// Convenience wrapper around [`FileSelection::new`].
pub fn compute_file_selection(container: &SfdlContainer, patterns: &[String]) -> FileSelection {
	FileSelection::new(container, patterns)
}

/// SFDL-003: Resolve BulkFolders via FTP and merge results into the container.
///
/// After resolution, bulk folder entries are cleared and their files are added
/// to the respective package's file list. Individual BulkFolder failures are
/// logged but don't abort resolution of other folders (A1/A3).
///
/// Returns list of warnings for failed folders.
pub async fn resolve_bulk_folders(container: &mut SfdlContainer, timeout: u32) -> Vec<String> {
	let mut warnings = Vec::new();

	for pkg in &mut container.packages {
		if !pkg.bulk_folder_mode || pkg.bulk_folder_list.is_empty() {
			continue;
		}

		let mut resolved_files = Vec::new();
		let mut resolved_indices = Vec::new();

		for (i, bulk) in pkg.bulk_folder_list.iter().enumerate() {
			match resolve_bulk_folder(&container.connection, bulk, timeout).await {
				Ok(files) => {
					if files.is_empty() {
						// A2: Empty directory
						tracing::info!(path = %bulk.bulk_folder_path, "BulkFolder directory is empty");
					}
					resolved_files.extend(files);
					resolved_indices.push(i);
				}
				Err(e) => {
					// A1/A3: FTP error — log warning, continue with others
					let msg = format!("Failed to resolve {}: {}", bulk.bulk_folder_path, e);
					tracing::warn!("{}", msg);
					warnings.push(msg);
				}
			}
		}

		pkg.file_list.extend(resolved_files);

		// Remove successfully resolved folders (in reverse to preserve indices)
		for i in resolved_indices.into_iter().rev() {
			pkg.bulk_folder_list.remove(i);
		}
		if pkg.bulk_folder_list.is_empty() {
			pkg.bulk_folder_mode = false;
		}
	}

	warnings
}

/// Check if a container has unresolved BulkFolders.
pub fn has_unresolved_bulk_folders(container: &SfdlContainer) -> bool {
	container.packages.iter().any(|p| p.bulk_folder_mode && !p.bulk_folder_list.is_empty())
}

/// Filter a container to only keep selected files.
///
/// Removes files where the corresponding entry in the selection is `false`.
/// The selection must have been built from the same container state.
pub fn filter_container(container: &mut SfdlContainer, selection: &FileSelection) {
	debug_assert_eq!(
		selection.total_count(),
		container.packages.iter().map(|p| p.file_list.len()).sum::<usize>(),
		"FileSelection must match the container's file count"
	);
	let selected = selection.as_slice();
	let mut idx = 0;
	for package in &mut container.packages {
		package.file_list.retain(|_| {
			let keep = selected.get(idx).copied().unwrap_or(true);
			idx += 1;
			keep
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::sfdl::models::{Connection, FileItem, Package};

	fn make_container_with_files(names: &[&str]) -> SfdlContainer {
		SfdlContainer {
			packages: vec![Package {
				name: "TestPkg".into(),
				file_list: names
					.iter()
					.map(|n| FileItem {
						file_name: n.to_string(),
						..FileItem::default()
					})
					.collect(),
				..Package::default()
			}],
			..SfdlContainer::default()
		}
	}

	/// DL-001 | BR-DL-001: Exclusion patterns mark matching files as unselected.
	#[test]
	fn dl001_compute_selection_excludes_matching() {
		let container = make_container_with_files(&["movie.rar", "info.nfo", "cover.jpg"]);
		let patterns = vec!["*.nfo".into(), "*.jpg".into()];

		let selection = compute_file_selection(&container, &patterns);
		assert_eq!(selection.as_slice(), &[true, false, false]);
	}

	/// DL-001 | BR-DL-001: Empty patterns mean all files are selected.
	#[test]
	fn dl001_compute_selection_empty_patterns() {
		let container = make_container_with_files(&["movie.rar", "info.nfo"]);
		let selection = compute_file_selection(&container, &[]);
		assert_eq!(selection.as_slice(), &[true, true]);
	}

	/// DL-001 | Main Success: filter_container removes unselected files.
	#[test]
	fn dl001_filter_container_removes_unselected() {
		let mut container = make_container_with_files(&["a.rar", "b.nfo", "c.jpg"]);
		let patterns = vec!["*.nfo".into()];
		let selection = compute_file_selection(&container, &patterns);

		filter_container(&mut container, &selection);

		let names: Vec<&str> = container.packages[0].file_list.iter().map(|f| f.file_name.as_str()).collect();
		assert_eq!(names, vec!["a.rar", "c.jpg"]);
	}

	/// SFDL-001 | Main Success: load_sfdl parses unencrypted container.
	#[test]
	fn sfdl001_load_not_encrypted() {
		let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<Container>
  <ContainerVersion>10</ContainerVersion>
  <Description>Test</Description>
  <Uploader>user</Uploader>
  <Encrypted>false</Encrypted>
  <MaxDownloadThreads>3</MaxDownloadThreads>
  <Connection>
    <Host>ftp.example.com</Host>
    <Port>21</Port>
    <Username>anon</Username>
    <Password>pass</Password>
    <AuthRequired>false</AuthRequired>
    <DataConnectionType>Passive</DataConnectionType>
    <DataType>Binary</DataType>
    <CharacterEncoding>UTF8</CharacterEncoding>
    <SSLProtocol>None</SSLProtocol>
  </Connection>
  <Packages>
    <Package>
      <Name>Pkg1</Name>
      <BulkFolderMode>false</BulkFolderMode>
      <FileList></FileList>
      <BulkFolderList></BulkFolderList>
    </Package>
  </Packages>
</Container>"#;

		let result = load_sfdl(xml, &[]).unwrap();
		assert!(matches!(result.status, DecryptionStatus::NotEncrypted));
		assert_eq!(result.container.description, "Test");
	}

	/// SFDL-002 | A2: decrypt_with_password with wrong password returns InvalidPassword.
	#[test]
	fn sfdl002_decrypt_with_password_invalid() {
		let mut container = SfdlContainer {
			encrypted: true,
			connection: Connection {
				host: "FA9p93TaRSx1Bap096qqevmwi8vGbaEXtXRbnLmbUr8=".into(),
				..Connection::default()
			},
			..SfdlContainer::default()
		};

		let result = decrypt_with_password(&mut container, "wrong");
		assert!(matches!(result, Err(AppError::InvalidPassword)));
	}

	// =======================================================
	// SFDL-003: BulkFolder detection
	// =======================================================

	/// SFDL-003 | Precondition: Container with BulkFolders detected.
	#[test]
	fn sfdl003_has_unresolved_bulk_folders() {
		use crate::sfdl::models::BulkFolder;

		let container = SfdlContainer {
			packages: vec![Package {
				name: "Pkg".into(),
				bulk_folder_mode: true,
				bulk_folder_list: vec![BulkFolder {
					bulk_folder_path: "/data/".into(),
					package_name: "Pkg".into(),
				}],
				..Package::default()
			}],
			..SfdlContainer::default()
		};
		assert!(has_unresolved_bulk_folders(&container));
	}

	/// SFDL-003 | Edge: No BulkFolders → false.
	#[test]
	fn sfdl003_no_bulk_folders() {
		let container = make_container_with_files(&["a.rar"]);
		assert!(!has_unresolved_bulk_folders(&container));
	}

	/// SFDL-003 | Edge: BulkFolder mode but empty list → false.
	#[test]
	fn sfdl003_bulk_mode_empty_list() {
		let container = SfdlContainer {
			packages: vec![Package {
				name: "Pkg".into(),
				bulk_folder_mode: true,
				bulk_folder_list: vec![], // empty
				..Package::default()
			}],
			..SfdlContainer::default()
		};
		assert!(!has_unresolved_bulk_folders(&container));
	}
}
