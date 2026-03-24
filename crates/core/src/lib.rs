pub mod container;
pub mod diskspace;
pub mod download;
pub mod error;
pub mod extraction;
pub mod filter;
pub mod ftp;
pub mod selection;
pub mod settings;
pub mod sfdl;

pub fn format_bytes(bytes: u64) -> String {
	const KB: u64 = 1024;
	const MB: u64 = 1024 * KB;
	const GB: u64 = 1024 * MB;
	const TB: u64 = 1024 * GB;

	if bytes >= TB {
		format!("{:.1} TB", bytes as f64 / TB as f64)
	} else if bytes >= GB {
		format!("{:.1} GB", bytes as f64 / GB as f64)
	} else if bytes >= MB {
		format!("{:.1} MB", bytes as f64 / MB as f64)
	} else if bytes >= KB {
		format!("{:.1} KB", bytes as f64 / KB as f64)
	} else {
		format!("{} B", bytes)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn format_bytes_units() {
		assert_eq!(format_bytes(0), "0 B");
		assert_eq!(format_bytes(512), "512 B");
		assert_eq!(format_bytes(1024), "1.0 KB");
		assert_eq!(format_bytes(1536), "1.5 KB");
		assert_eq!(format_bytes(1048576), "1.0 MB");
		assert_eq!(format_bytes(1073741824), "1.0 GB");
		assert_eq!(format_bytes(1099511627776), "1.0 TB");
	}
}
