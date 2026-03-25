//! POST-003: Speed-Report generation.
//!
//! Generates a formatted report from download session statistics.

use std::time::Duration;

use crate::{format_bytes, format_speed};

/// Statistics from a completed download session.
#[derive(Debug, Clone)]
pub struct SessionStats {
	pub total_files: u32,
	pub completed_files: u32,
	pub failed_files: u32,
	pub skipped_files: u32,
	pub total_bytes: u64,
	pub duration: Duration,
	pub max_threads: u32,
	pub container_name: String,
	pub uploader: String,
}

/// BR-POST-006: Default template.
pub const DEFAULT_TEMPLATE: &str = "\
rsfdl v{{version}} speed report

SFDL: {{container_name}}
Uploader: {{uploader}}
{{total_size_formatted}} in {{duration}} heruntergeladen - ⌀Speed: {{avg_speed_formatted}}
{{total_files}} Dateien ({{completed_files}}✓ {{failed_files}}✗ {{skipped_files}}⊘)

Besten Dank!";

/// POST-003: Generate a speed report from session statistics.
///
/// Uses `custom_template` if non-empty, otherwise falls back to the default template (A1).
pub fn generate(stats: &SessionStats, custom_template: &str) -> String {
	let template = if custom_template.is_empty() { DEFAULT_TEMPLATE } else { custom_template };

	render(template, stats)
}

/// Single-pass template rendering. Replaces `{{var}}` tokens with computed values.
///
/// Uses single-pass to avoid double-expansion: field values containing `{{...}}`
/// are inserted literally and never re-interpreted as template variables.
fn render(template: &str, stats: &SessionStats) -> String {
	let duration_secs_f64 = stats.duration.as_secs_f64();
	let duration_secs = stats.duration.as_secs();
	let hours = duration_secs / 3600;
	let minutes = (duration_secs % 3600) / 60;
	let seconds = duration_secs % 60;

	let total_size_mb = stats.total_bytes as f64 / (1024.0 * 1024.0);
	let total_size_gb = stats.total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

	let avg_bytes_per_sec = if duration_secs_f64 > 0.0 { stats.total_bytes as f64 / duration_secs_f64 } else { 0.0 };
	let avg_speed_kbps = avg_bytes_per_sec / 1024.0;
	let avg_speed_mbps = avg_speed_kbps / 1024.0;

	let vars: &[(&str, String)] = &[
		("{{version}}", env!("CARGO_PKG_VERSION").to_owned()),
		("{{uploader}}", stats.uploader.clone()),
		("{{container_name}}", stats.container_name.clone()),
		("{{total_files}}", stats.total_files.to_string()),
		("{{completed_files}}", stats.completed_files.to_string()),
		("{{failed_files}}", stats.failed_files.to_string()),
		("{{skipped_files}}", stats.skipped_files.to_string()),
		("{{total_size_formatted}}", format_bytes(stats.total_bytes)),
		("{{total_size_mb}}", format!("{:.2}", total_size_mb)),
		("{{total_size_gb}}", format!("{:.2}", total_size_gb)),
		("{{duration}}", format!("{:02}:{:02}:{:02}", hours, minutes, seconds)),
		("{{avg_speed_formatted}}", format_speed(avg_bytes_per_sec)),
		("{{avg_speed_mbps}}", format!("{:.2}", avg_speed_mbps)),
		("{{avg_speed_kbps}}", format!("{:.2}", avg_speed_kbps)),
		("{{max_threads}}", stats.max_threads.to_string()),
	];

	let mut out = String::with_capacity(template.len() + 128);
	let mut rest = template;
	'outer: while !rest.is_empty() {
		if rest.starts_with("{{") {
			for (key, val) in vars {
				if rest.starts_with(*key) {
					out.push_str(val);
					rest = &rest[key.len()..];
					continue 'outer;
				}
			}
		}
		let Some(c) = rest.chars().next() else { break };
		out.push(c);
		rest = &rest[c.len_utf8()..];
	}
	out
}

#[cfg(test)]
mod tests {
	use super::*;

	fn sample_stats() -> SessionStats {
		SessionStats {
			total_files: 19,
			completed_files: 17,
			failed_files: 1,
			skipped_files: 1,
			total_bytes: 5 * 1024 * 1024 * 1024, // 5 GiB
			duration: Duration::from_secs(437),  // 7:17
			max_threads: 3,
			container_name: "Movie.2026.720p".into(),
			uploader: "SceneGroup".into(),
		}
	}

	/// POST-003 | A1: Empty custom template uses default.
	#[test]
	fn post003_empty_template_uses_default() {
		let stats = sample_stats();
		let report = generate(&stats, "");
		assert!(report.starts_with("rsfdl v"));
		assert!(report.contains("speed report"));
		assert!(report.contains("SFDL: Movie.2026.720p"));
	}

	/// POST-003 | Main Success: Custom template is used when provided.
	#[test]
	fn post003_custom_template_used() {
		let stats = sample_stats();
		let report = generate(&stats, "Report: {{total_files}} files in {{duration}}");
		assert_eq!(report, "Report: 19 files in 00:07:17");
	}

	/// POST-003 | BR-POST-006: All template variables are replaced.
	#[test]
	fn post003_all_variables_replaced() {
		let stats = sample_stats();
		let template = "{{version}} {{uploader}} {{total_files}} {{completed_files}} {{failed_files}} {{skipped_files}} {{total_size_formatted}} {{total_size_mb}} {{total_size_gb}} {{duration}} {{avg_speed_formatted}} {{avg_speed_mbps}} {{avg_speed_kbps}} {{max_threads}} {{container_name}}";
		let report = generate(&stats, template);

		assert!(!report.contains("{{"), "all variables should be replaced: {report}");
		assert!(report.contains("SceneGroup"));
		assert!(report.contains("19"));
		assert!(report.contains("17"));
		assert!(report.contains("Movie.2026.720p"));
	}

	/// POST-003 | BR-POST-006: Duration formatted as HH:MM:SS.
	#[test]
	fn post003_duration_format() {
		let mut stats = sample_stats();
		stats.duration = Duration::from_secs(3661); // 1h 1m 1s
		let report = generate(&stats, "{{duration}}");
		assert_eq!(report, "01:01:01");
	}

	/// POST-003 | BR-POST-006: Duration zero seconds.
	#[test]
	fn post003_duration_zero() {
		let mut stats = sample_stats();
		stats.duration = Duration::from_secs(0);
		let report = generate(&stats, "{{duration}}");
		assert_eq!(report, "00:00:00");
	}

	/// POST-003 | BR-POST-006: Speed zero duration shows 0.
	#[test]
	fn post003_speed_zero_duration() {
		let mut stats = sample_stats();
		stats.duration = Duration::from_secs(0);
		let report = generate(&stats, "{{avg_speed_formatted}}");
		assert_eq!(report, "0 B/s");
	}

	/// POST-003 | BR-POST-006: Formatted speed auto-scales.
	#[test]
	fn post003_speed_formatted() {
		let stats = SessionStats {
			total_files: 1,
			completed_files: 1,
			failed_files: 0,
			skipped_files: 0,
			total_bytes: 1024 * 1024 * 100, // 100 MiB
			duration: Duration::from_secs(100),
			max_threads: 1,
			container_name: String::new(),
			uploader: String::new(),
		};
		// 100 MiB / 100s = 1 MiB/s
		let report = generate(&stats, "{{avg_speed_formatted}}");
		assert_eq!(report, "1.00 MiB/s");
	}

	/// POST-003 | BR-POST-006: Formatted size auto-scales.
	#[test]
	fn post003_size_formatted() {
		let stats = SessionStats {
			total_files: 1,
			completed_files: 1,
			failed_files: 0,
			skipped_files: 0,
			total_bytes: 5 * 1024 * 1024 * 1024, // 5 GiB
			duration: Duration::from_secs(60),
			max_threads: 1,
			container_name: String::new(),
			uploader: String::new(),
		};
		let report = generate(&stats, "{{total_size_formatted}}");
		assert_eq!(report, "5.0 GiB");
	}

	/// POST-003 | BR-POST-006: Zero bytes downloaded.
	#[test]
	fn post003_zero_bytes() {
		let stats = SessionStats {
			total_files: 0,
			completed_files: 0,
			failed_files: 0,
			skipped_files: 0,
			total_bytes: 0,
			duration: Duration::from_secs(0),
			max_threads: 3,
			container_name: String::new(),
			uploader: String::new(),
		};
		let report = generate(&stats, "{{total_size_formatted}} {{avg_speed_formatted}}");
		assert_eq!(report, "0 B 0 B/s");
	}

	/// POST-003 | Default template contains expected structure.
	#[test]
	fn post003_default_template_structure() {
		let stats = sample_stats();
		let report = generate(&stats, "");
		assert!(report.starts_with("rsfdl v"));
		assert!(report.contains("speed report"));
		assert!(report.contains("SFDL: Movie.2026.720p"));
		assert!(report.contains("Uploader: SceneGroup"));
		assert!(report.contains("heruntergeladen"));
		assert!(report.contains("17✓"));
		assert!(report.contains("1✗"));
		assert!(report.contains("1⊘"));
		assert!(report.contains("00:07:17"));
		assert!(report.contains("Besten Dank!"));
	}

	/// POST-003 | Template with unknown variables leaves them as-is.
	#[test]
	fn post003_unknown_variables_preserved() {
		let stats = sample_stats();
		let report = generate(&stats, "{{unknown_var}} and {{total_files}}");
		assert!(report.contains("{{unknown_var}}"));
		assert!(report.contains("19"));
	}

	/// POST-003 | Field values containing template tokens are not double-expanded.
	#[test]
	fn post003_no_double_expansion() {
		let mut stats = sample_stats();
		stats.uploader = "{{container_name}}".into();
		let report = generate(&stats, "{{uploader}}");
		assert_eq!(report, "{{container_name}}");
	}
}
