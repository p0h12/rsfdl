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
pub mod speedreport;
pub mod verification;

pub fn format_bytes(bytes: u64) -> String {
	const KB: u64 = 1024;
	const MB: u64 = 1024 * KB;
	const GB: u64 = 1024 * MB;
	const TB: u64 = 1024 * GB;

	if bytes >= TB {
		format!("{:.1} TiB", bytes as f64 / TB as f64)
	} else if bytes >= GB {
		format!("{:.1} GiB", bytes as f64 / GB as f64)
	} else if bytes >= MB {
		format!("{:.1} MiB", bytes as f64 / MB as f64)
	} else if bytes >= KB {
		format!("{:.1} KiB", bytes as f64 / KB as f64)
	} else {
		format!("{} B", bytes)
	}
}

pub fn format_speed(bytes_per_sec: f64) -> String {
	const KB: f64 = 1024.0;
	const MB: f64 = 1024.0 * KB;
	const GB: f64 = 1024.0 * MB;

	if bytes_per_sec >= GB {
		format!("{:.2} GiB/s", bytes_per_sec / GB)
	} else if bytes_per_sec >= MB {
		format!("{:.2} MiB/s", bytes_per_sec / MB)
	} else if bytes_per_sec >= KB {
		format!("{:.2} KiB/s", bytes_per_sec / KB)
	} else {
		format!("{:.0} B/s", bytes_per_sec)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn format_bytes_units() {
		assert_eq!(format_bytes(0), "0 B");
		assert_eq!(format_bytes(512), "512 B");
		assert_eq!(format_bytes(1024), "1.0 KiB");
		assert_eq!(format_bytes(1536), "1.5 KiB");
		assert_eq!(format_bytes(1048576), "1.0 MiB");
		assert_eq!(format_bytes(1073741824), "1.0 GiB");
		assert_eq!(format_bytes(1099511627776), "1.0 TiB");
	}

	#[test]
	fn format_speed_units() {
		assert_eq!(format_speed(0.0), "0 B/s");
		assert_eq!(format_speed(512.0), "512 B/s");
		assert_eq!(format_speed(1024.0), "1.00 KiB/s");
		assert_eq!(format_speed(1024.0 * 1024.0), "1.00 MiB/s");
		assert_eq!(format_speed(1024.0 * 1024.0 * 1024.0), "1.00 GiB/s");
		assert_eq!(format_speed(1024.0 * 1024.0 * 11.68), "11.68 MiB/s");
	}
}
