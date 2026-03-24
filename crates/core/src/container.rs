use crate::filter::is_excluded;
use crate::ftp::listing::resolve_container_bulk_folders;
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
/// Returns a `Vec<bool>` aligned with the flattened file list across all packages.
/// `true` = file is selected (not excluded), `false` = file is excluded.
pub fn compute_file_selection(container: &SfdlContainer, patterns: &[String]) -> Vec<bool> {
	container.packages.iter().flat_map(|p| &p.file_list).map(|f| !is_excluded(&f.file_name, patterns)).collect()
}

/// SFDL-003: Resolve BulkFolders via FTP and merge results into the container.
///
/// After resolution, bulk folder entries are cleared and their files are added
/// to the respective package's file list.
pub async fn resolve_bulk_folders(container: &mut SfdlContainer, timeout: u32) -> Result<(), AppError> {
	let has_bulk = container.packages.iter().any(|p| p.bulk_folder_mode && !p.bulk_folder_list.is_empty());

	if !has_bulk {
		return Ok(());
	}

	let resolved_files = resolve_container_bulk_folders(&container.connection, &container.packages, timeout).await?;

	if !resolved_files.is_empty() {
		// Group resolved files by package name
		let mut files_by_package: std::collections::HashMap<String, Vec<crate::sfdl::models::FileItem>> = std::collections::HashMap::new();
		for file in resolved_files {
			files_by_package.entry(file.package_name.clone()).or_default().push(file);
		}

		for pkg in &mut container.packages {
			if let Some(new_files) = files_by_package.remove(&pkg.name) {
				pkg.file_list.extend(new_files);
			}
			pkg.bulk_folder_list.clear();
			pkg.bulk_folder_mode = false;
		}
	}

	Ok(())
}

/// Filter a container to only keep selected files.
///
/// The `selected` slice must be aligned with the flattened file list across all packages
/// (same order as [`compute_file_selection`] returns). Files where `selected[i]` is `false`
/// are removed from the container.
pub fn filter_container(container: &mut SfdlContainer, selected: &[bool]) {
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
		assert_eq!(selection, vec![true, false, false]);
	}

	/// DL-001 | BR-DL-001: Empty patterns mean all files are selected.
	#[test]
	fn dl001_compute_selection_empty_patterns() {
		let container = make_container_with_files(&["movie.rar", "info.nfo"]);
		let selection = compute_file_selection(&container, &[]);
		assert_eq!(selection, vec![true, true]);
	}

	/// DL-001 | Main Success: filter_container removes unselected files.
	#[test]
	fn dl001_filter_container_removes_unselected() {
		let mut container = make_container_with_files(&["a.rar", "b.nfo", "c.jpg"]);
		let selected = vec![true, false, true];

		filter_container(&mut container, &selected);

		let names: Vec<&str> = container.packages[0].file_list.iter().map(|f| f.file_name.as_str()).collect();
		assert_eq!(names, vec!["a.rar", "c.jpg"]);
	}

	/// SFDL-001 | Main Success: load_sfdl parses unencrypted container.
	#[test]
	fn sfdl001_load_not_encrypted() {
		let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<SFDLFile>
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
    <SFDLPackage>
      <Name>Pkg1</Name>
      <BulkFolderMode>false</BulkFolderMode>
      <FileList></FileList>
      <BulkFolderList></BulkFolderList>
    </SFDLPackage>
  </Packages>
</SFDLFile>"#;

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
}
