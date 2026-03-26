use crate::error::FtpError;
use crate::ftp::client::FtpClient;
use crate::sfdl::models::{BulkFolder, Connection, FileItem, HashType};

/// Resolve a single BulkFolder using an existing FTP connection.
pub(crate) async fn resolve_with_client(client: &mut FtpClient, bulk: &BulkFolder) -> Result<Vec<FileItem>, FtpError> {
	let mut items = Vec::new();
	recursive_list(client, &bulk.bulk_folder_path, bulk, &mut items).await?;
	Ok(items)
}

/// Resolve a single BulkFolder (connects and disconnects internally).
pub async fn resolve_bulk_folder(conn: &Connection, bulk: &BulkFolder, timeout_seconds: u32) -> Result<Vec<FileItem>, FtpError> {
	let mut client = FtpClient::connect(conn, timeout_seconds).await?;
	let items = resolve_with_client(&mut client, bulk).await?;
	client.disconnect().await;
	Ok(items)
}

/// Resolve all BulkFolders for an entire container and return the resolved FileItems.
/// This is a standalone function for preview purposes (before starting downloads).
/// Uses a single FTP connection for all folders.
pub async fn resolve_container_bulk_folders(conn: &Connection, packages: &[crate::sfdl::models::Package], timeout_seconds: u32) -> Result<Vec<FileItem>, FtpError> {
	let mut client = FtpClient::connect(conn, timeout_seconds).await?;
	let mut all_items = Vec::new();
	for pkg in packages {
		if pkg.bulk_folder_mode && !pkg.bulk_folder_list.is_empty() {
			for folder in &pkg.bulk_folder_list {
				let items = resolve_with_client(&mut client, folder).await?;
				all_items.extend(items);
			}
		}
	}
	client.disconnect().await;
	Ok(all_items)
}

/// Resolve all BulkFolders using a single FTP connection.
pub async fn resolve_all_bulk_folders(conn: &Connection, folders: &[BulkFolder], timeout_seconds: u32) -> Result<Vec<FileItem>, FtpError> {
	let mut client = FtpClient::connect(conn, timeout_seconds).await?;
	let mut all_items = Vec::new();
	for folder in folders {
		let items = resolve_with_client(&mut client, folder).await?;
		all_items.extend(items);
	}
	client.disconnect().await;
	Ok(all_items)
}

fn recursive_list<'a>(
	client: &'a mut FtpClient,
	path: &'a str,
	bulk: &'a BulkFolder,
	items: &'a mut Vec<FileItem>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), FtpError>> + Send + 'a>> {
	Box::pin(async move {
		let entries = client.list_dir(path).await?;

		for entry in entries {
			// Skip . and ..
			if entry.name == "." || entry.name == ".." {
				continue;
			}

			let full = normalize_path(&format!("{}/{}", path, entry.name));

			if entry.is_directory {
				recursive_list(client, &full, bulk, items).await?;
			} else {
				items.push(FileItem {
					file_name: entry.name,
					directory_root: bulk.bulk_folder_path.clone(),
					directory_path: path.to_string(),
					full_path: full,
					file_size: entry.size,
					hash_type: HashType::None,
					file_hash: String::new(),
					package_name: bulk.package_name.clone(),
				});
			}
		}

		Ok(())
	})
}

/// Remove duplicate slashes from path.
fn normalize_path(path: &str) -> String {
	let mut result = String::with_capacity(path.len());
	let mut prev_slash = false;
	for ch in path.chars() {
		if ch == '/' {
			if !prev_slash {
				result.push(ch);
			}
			prev_slash = true;
		} else {
			result.push(ch);
			prev_slash = false;
		}
	}
	result
}

#[cfg(test)]
mod tests {
	use super::*;

	/// SFDL-003 | BR-SFDL-006: Normalize path removes double slashes.
	#[test]
	fn sfdl003_normalize_path_removes_double_slashes() {
		assert_eq!(normalize_path("/a//b///c"), "/a/b/c");
		assert_eq!(normalize_path("/release/test/"), "/release/test/");
		assert_eq!(normalize_path("//root//dir//file"), "/root/dir/file");
	}

	/// SFDL-003 | BR-SFDL-006: Preserve single slashes.
	#[test]
	fn sfdl003_normalize_path_preserves_single_slashes() {
		assert_eq!(normalize_path("/a/b/c"), "/a/b/c");
	}

	/// SFDL-003 | Edge: Empty path stays empty.
	#[test]
	fn sfdl003_normalize_path_empty() {
		assert_eq!(normalize_path(""), "");
	}
}
