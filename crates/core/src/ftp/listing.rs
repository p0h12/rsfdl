use crate::error::FtpError;
use crate::ftp::client::FtpClient;
use crate::sfdl::models::{BulkFolder, Connection, FileItem, HashType};

/// Resolve a single BulkFolder into FileItems by recursively listing the FTP directory.
pub async fn resolve_bulk_folder(conn: &Connection, bulk: &BulkFolder, timeout_seconds: u32) -> Result<Vec<FileItem>, FtpError> {
	let mut client = FtpClient::connect(conn, timeout_seconds).await?;
	let mut items = Vec::new();

	recursive_list(&mut client, &bulk.bulk_folder_path, bulk, &mut items).await?;

	client.disconnect().await;
	Ok(items)
}

/// Resolve all BulkFolders for an entire container and return the resolved FileItems.
/// This is a standalone function for preview purposes (before starting downloads).
pub async fn resolve_container_bulk_folders(conn: &Connection, packages: &[crate::sfdl::models::Package], timeout_seconds: u32) -> Result<Vec<FileItem>, FtpError> {
	let mut all_items = Vec::new();
	for pkg in packages {
		if pkg.bulk_folder_mode && !pkg.bulk_folder_list.is_empty() {
			let items = resolve_all_bulk_folders(conn, &pkg.bulk_folder_list, timeout_seconds).await?;
			all_items.extend(items);
		}
	}
	Ok(all_items)
}

/// Resolve all BulkFolders sequentially (one connection per folder).
pub async fn resolve_all_bulk_folders(conn: &Connection, folders: &[BulkFolder], timeout_seconds: u32) -> Result<Vec<FileItem>, FtpError> {
	let mut all_items = Vec::new();
	for folder in folders {
		let items = resolve_bulk_folder(conn, folder, timeout_seconds).await?;
		all_items.extend(items);
	}
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

	#[test]
	fn normalize_path_removes_double_slashes() {
		assert_eq!(normalize_path("/a//b///c"), "/a/b/c");
		assert_eq!(normalize_path("/release/test/"), "/release/test/");
		assert_eq!(normalize_path("//root//dir//file"), "/root/dir/file");
	}

	#[test]
	fn normalize_path_preserves_single_slashes() {
		assert_eq!(normalize_path("/a/b/c"), "/a/b/c");
	}

	#[test]
	fn normalize_path_empty() {
		assert_eq!(normalize_path(""), "");
	}
}
