//! CLI-002: List files in an SFDL container.

use std::io::Write;

use rsfdl_core::filter::{is_excluded, resolve_patterns};
use rsfdl_core::format_bytes;
use rsfdl_core::sfdl::models::SfdlContainer;

use super::common::{SfdlArgs, load_and_decrypt};

/// CLI-002 exit code for BulkFolder resolution failure (A4).
const EXIT_BULK_FOLDER_FAILED: i32 = 5;

/// CLI-002: List files in an SFDL container.
pub async fn run(args: &SfdlArgs, password_list: &[String], resolve: bool, json: bool, cli_exclude: &[String], no_exclude: bool, show_excluded: bool) {
	let (mut container, settings, _outcome) = match load_and_decrypt(args, password_list) {
		Ok(result) => result,
		Err(e) => {
			eprintln!("Error: {e}");
			std::process::exit(e.exit_code());
		}
	};

	// SFDL-003: Resolve bulk folders if requested
	let mut bulk_failed = false;
	if resolve {
		let has_bulk = container.packages.iter().any(|p| p.bulk_folder_mode && !p.bulk_folder_list.is_empty());
		if has_bulk {
			eprintln!("Resolving bulk folders via FTP...");
			let warnings = rsfdl_core::container::resolve_bulk_folders(&mut container, settings.ftp_timeout_seconds).await;
			for w in &warnings {
				eprintln!("Warning: {}", w);
			}
			bulk_failed = !warnings.is_empty();
		}
	}

	// DL-002 / A5: Resolve exclusion patterns
	let patterns = resolve_patterns(&settings.exclusion_patterns, cli_exclude, no_exclude);

	let mut out = std::io::stdout().lock();
	if json {
		format_json(&mut out, &container, &patterns);
	} else {
		format_text(&mut out, &container, &patterns, show_excluded);
	}

	// A4: BulkFolder resolution failed → exit 5
	if bulk_failed {
		std::process::exit(EXIT_BULK_FOLDER_FAILED);
	}
}

/// BR-CLI-010 / BR-CLI-011: Format file listing as text.
///
/// Groups files by package. Shows filename + size per line.
/// With `show_excluded`: marks excluded files with `[excluded]`.
/// Ends with summary line: "N files (X), M excluded".
fn format_text(w: &mut impl Write, container: &SfdlContainer, patterns: &[String], show_excluded: bool) {
	let mut total_files = 0usize;
	let mut excluded_count = 0usize;
	let mut bulk_count = 0usize;

	for pkg in &container.packages {
		if !pkg.name.is_empty() {
			writeln!(w, "Package: {}", pkg.name).unwrap();
		}

		for file in &pkg.file_list {
			let excluded = is_excluded(&file.file_name, patterns);
			if excluded {
				excluded_count += 1;
				if show_excluded {
					writeln!(w, "  {:<55} {:>10}  [excluded]", file.file_name, format_bytes(file.file_size)).unwrap();
				}
			} else {
				writeln!(w, "  {:<55} {:>10}", file.file_name, format_bytes(file.file_size)).unwrap();
			}
			total_files += 1;
		}

		for bulk in &pkg.bulk_folder_list {
			writeln!(w, "  [DIR] {}", bulk.bulk_folder_path).unwrap();
			bulk_count += 1;
		}

		if !pkg.file_list.is_empty() || !pkg.bulk_folder_list.is_empty() {
			writeln!(w).unwrap();
		}
	}

	// BR-CLI-011: Summary line
	let selected = total_files - excluded_count;
	let selected_bytes: u64 = container
		.packages
		.iter()
		.flat_map(|p| &p.file_list)
		.filter(|f| !is_excluded(&f.file_name, patterns))
		.map(|f| f.file_size)
		.sum();

	writeln!(w, "{} files ({}), {} excluded", selected, format_bytes(selected_bytes), excluded_count).unwrap();

	if bulk_count > 0 {
		writeln!(w, "{} bulk folder(s) (use --resolve to list contents via FTP)", bulk_count).unwrap();
	}
}

/// BR-CLI-009: Format file listing as JSON.
///
/// Produces a JSON object with `packages[]` and `summary` per spec.
fn format_json(w: &mut impl Write, container: &SfdlContainer, patterns: &[String]) {
	let mut packages_json = Vec::new();

	let mut total_files = 0usize;
	let mut selected_files = 0usize;
	let mut total_bytes = 0u64;
	let mut selected_bytes = 0u64;

	for pkg in &container.packages {
		let mut files_json = Vec::new();

		for file in &pkg.file_list {
			let excluded = is_excluded(&file.file_name, patterns);
			total_files += 1;
			total_bytes += file.file_size;
			if !excluded {
				selected_files += 1;
				selected_bytes += file.file_size;
			}

			let mut file_obj = serde_json::json!({
				"filename": file.file_name,
				"size_bytes": file.file_size,
				"excluded": excluded,
			});

			if excluded && let Some(pattern) = patterns.iter().find(|p| is_excluded(&file.file_name, &[p.to_string()])) {
				file_obj["exclude_pattern"] = serde_json::json!(pattern);
			}

			files_json.push(file_obj);
		}

		packages_json.push(serde_json::json!({
			"name": pkg.name,
			"files": files_json,
		}));
	}

	let excluded_files = total_files - selected_files;

	let output = serde_json::json!({
		"packages": packages_json,
		"summary": {
			"total_files": total_files,
			"selected_files": selected_files,
			"excluded_files": excluded_files,
			"total_bytes": total_bytes,
			"selected_bytes": selected_bytes,
		}
	});

	writeln!(w, "{}", serde_json::to_string_pretty(&output).unwrap()).unwrap();
}

#[cfg(test)]
mod tests {
	use super::*;
	use rsfdl_core::sfdl::models::{FileItem, Package};

	fn test_container() -> SfdlContainer {
		SfdlContainer {
			packages: vec![Package {
				name: "TestPkg".into(),
				file_list: vec![
					FileItem {
						file_name: "movie.part1.rar".into(),
						file_size: 1_000_000,
						..FileItem::default()
					},
					FileItem {
						file_name: "movie.part2.rar".into(),
						file_size: 2_000_000,
						..FileItem::default()
					},
					FileItem {
						file_name: "info.nfo".into(),
						file_size: 500,
						..FileItem::default()
					},
				],
				..Package::default()
			}],
			..SfdlContainer::default()
		}
	}

	// -------------------------------------------------------
	// CLI-002 | Main Success: Text output
	// -------------------------------------------------------

	/// CLI-002 | Main Success: Text output lists all files with sizes.
	#[test]
	fn cli002_text_lists_files() {
		let c = test_container();
		let mut buf = Vec::new();
		format_text(&mut buf, &c, &[], false);
		let output = String::from_utf8(buf).unwrap();

		assert!(output.contains("movie.part1.rar"));
		assert!(output.contains("movie.part2.rar"));
		assert!(output.contains("info.nfo"));
	}

	/// CLI-002 | Main Success: Text output shows package name.
	#[test]
	fn cli002_text_shows_package_name() {
		let c = test_container();
		let mut buf = Vec::new();
		format_text(&mut buf, &c, &[], false);
		let output = String::from_utf8(buf).unwrap();

		assert!(output.contains("Package: TestPkg"));
	}

	/// CLI-002 | BR-CLI-011: Summary line shows file count and size.
	#[test]
	fn cli002_text_summary_line() {
		let c = test_container();
		let mut buf = Vec::new();
		format_text(&mut buf, &c, &[], false);
		let output = String::from_utf8(buf).unwrap();

		assert!(output.contains("3 files"));
		assert!(output.contains("0 excluded"));
	}

	// -------------------------------------------------------
	// CLI-002 | BR-CLI-010: Exclusion display
	// -------------------------------------------------------

	/// CLI-002 | BR-CLI-010: Without --show-excluded, excluded files are hidden.
	#[test]
	fn cli002_text_hides_excluded() {
		let c = test_container();
		let patterns = vec!["*.nfo".into()];
		let mut buf = Vec::new();
		format_text(&mut buf, &c, &patterns, false);
		let output = String::from_utf8(buf).unwrap();

		assert!(!output.contains("info.nfo"));
		assert!(output.contains("2 files"));
		assert!(output.contains("1 excluded"));
	}

	/// CLI-002 | BR-CLI-010: With --show-excluded, excluded files shown with [excluded].
	#[test]
	fn cli002_text_shows_excluded_marker() {
		let c = test_container();
		let patterns = vec!["*.nfo".into()];
		let mut buf = Vec::new();
		format_text(&mut buf, &c, &patterns, true);
		let output = String::from_utf8(buf).unwrap();

		assert!(output.contains("info.nfo"));
		assert!(output.contains("[excluded]"));
	}

	// -------------------------------------------------------
	// CLI-002 | JSON output
	// -------------------------------------------------------

	/// CLI-002 | BR-CLI-009: JSON output has all spec fields.
	#[test]
	fn cli002_json_has_all_fields() {
		let c = test_container();
		let mut buf = Vec::new();
		format_json(&mut buf, &c, &[]);
		let json: serde_json::Value = serde_json::from_slice(&buf).unwrap();

		assert_eq!(json["packages"][0]["name"], "TestPkg");
		assert_eq!(json["packages"][0]["files"].as_array().unwrap().len(), 3);
		assert_eq!(json["summary"]["total_files"], 3);
		assert_eq!(json["summary"]["selected_files"], 3);
		assert_eq!(json["summary"]["excluded_files"], 0);
		assert_eq!(json["summary"]["total_bytes"], 3_000_500);
		assert_eq!(json["summary"]["selected_bytes"], 3_000_500);
	}

	/// CLI-002 | BR-CLI-009: JSON excluded files have exclude_pattern.
	#[test]
	fn cli002_json_excluded_has_pattern() {
		let c = test_container();
		let patterns = vec!["*.nfo".into()];
		let mut buf = Vec::new();
		format_json(&mut buf, &c, &patterns);
		let json: serde_json::Value = serde_json::from_slice(&buf).unwrap();

		assert_eq!(json["summary"]["excluded_files"], 1);
		assert_eq!(json["summary"]["selected_files"], 2);

		let files = json["packages"][0]["files"].as_array().unwrap();
		let nfo = files.iter().find(|f| f["filename"] == "info.nfo").unwrap();
		assert_eq!(nfo["excluded"], true);
		assert_eq!(nfo["exclude_pattern"], "*.nfo");

		let rar = files.iter().find(|f| f["filename"] == "movie.part1.rar").unwrap();
		assert_eq!(rar["excluded"], false);
		assert!(rar.get("exclude_pattern").is_none() || rar["exclude_pattern"].is_null());
	}

	/// CLI-002 | BR-CLI-009: JSON size fields are correct with exclusions.
	#[test]
	fn cli002_json_sizes_with_exclusion() {
		let c = test_container();
		let patterns = vec!["*.nfo".into()];
		let mut buf = Vec::new();
		format_json(&mut buf, &c, &patterns);
		let json: serde_json::Value = serde_json::from_slice(&buf).unwrap();

		assert_eq!(json["summary"]["total_bytes"], 3_000_500);
		assert_eq!(json["summary"]["selected_bytes"], 3_000_000); // 1M + 2M, excludes 500B nfo
	}
}
