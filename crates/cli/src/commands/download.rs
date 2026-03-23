use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tokio::sync::mpsc;
use uuid::Uuid;

use rsfdl_core::download::manager::DownloadManager;
use rsfdl_core::download::progress::ProgressEvent;
use rsfdl_core::format_bytes;

use super::common::{SfdlArgs, load_and_decrypt};

pub async fn run(args: &SfdlArgs, password_list: &[String], dest: Option<&str>, threads: u32, cli_exclude: &[String]) {
	let (mut container, mut settings, _outcome) = match load_and_decrypt(args, password_list) {
		Ok(result) => result,
		Err(e) => {
			eprintln!("Error: {e}");
			std::process::exit(1);
		}
	};

	// Apply CLI overrides to settings
	if let Some(d) = dest {
		settings.download_directory = PathBuf::from(d);
	} else {
		settings.download_directory = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
	}
	settings.max_download_threads = threads;

	// UC-DL-001 + UC-DL-002: Apply file exclusion patterns
	let mut all_patterns = settings.file_exclusion_patterns.clone();
	all_patterns.extend_from_slice(cli_exclude);

	let selection = rsfdl_core::container::compute_file_selection(&container, &all_patterns);
	let excluded_count = selection.iter().filter(|&&keep| !keep).count();
	if excluded_count > 0 {
		eprintln!("Excluded {} file(s) matching patterns", excluded_count);
	}
	rsfdl_core::container::filter_container(&mut container, &selection);

	// Count files
	let file_count: usize = container.packages.iter().map(|p| p.file_list.len()).sum();
	let bulk_count: usize = container.packages.iter().map(|p| p.bulk_folder_list.len()).sum();
	let total_bytes: u64 = container.packages.iter().flat_map(|p| p.file_list.iter()).map(|f| f.file_size).sum();

	eprintln!("Connecting to {}:{}...", container.connection.host, container.connection.port);

	if file_count > 0 {
		eprintln!("Downloading {} files ({}) to {}", file_count, format_bytes(total_bytes), settings.download_directory.display());
	}
	if bulk_count > 0 {
		eprintln!("Resolving {} bulk folder(s) via FTP...", bulk_count);
	}

	// Create download manager
	let (manager, cancel_token, _file_cancel_tx) = DownloadManager::new(container, &settings);

	// Ctrl+C handler
	let cancel_for_signal = cancel_token.clone();
	tokio::spawn(async move {
		tokio::signal::ctrl_c().await.ok();
		eprintln!("\nCancelling downloads...");
		cancel_for_signal.cancel();
	});

	// Progress channel
	let (tx, mut rx) = mpsc::unbounded_channel::<ProgressEvent>();

	// Run downloads in background
	let manager_handle = tokio::spawn(async move { manager.run(tx).await });

	// Progress display loop
	let multi = MultiProgress::new();
	let mut bars: HashMap<Uuid, ProgressBar> = HashMap::new();

	let global_bar = multi.add(ProgressBar::new(0));
	global_bar.set_style(
		ProgressStyle::with_template("{prefix:.bold} [{bar:40.cyan/dim}] {bytes}/{total_bytes} {binary_bytes_per_sec} ETA {eta}")
			.unwrap()
			.progress_chars("=>-"),
	);
	global_bar.set_prefix("[0/0 files]");

	let file_style = ProgressStyle::with_template("  {prefix:.cyan} [{bar:30.green/dim}] {bytes}/{total_bytes} {bytes_per_sec}")
		.unwrap()
		.progress_chars("=>-");

	let mut global_total_bytes: u64 = 0;
	let mut global_written: u64 = 0;
	let mut files_done: u32 = 0;
	let mut files_total: u32 = 0;
	let started_at = Instant::now();

	while let Some(event) = rx.recv().await {
		match event {
			ProgressEvent::Started { item_id, file_name, total_bytes } => {
				files_total += 1;
				global_total_bytes += total_bytes;
				global_bar.set_length(global_total_bytes);
				global_bar.set_prefix(format!("[{}/{} files]", files_done, files_total));

				let bar = multi.add(ProgressBar::new(total_bytes));
				bar.set_style(file_style.clone());
				bar.set_prefix(truncate_name(&file_name, 30));
				bars.insert(item_id, bar);
			}
			ProgressEvent::BytesWritten {
				item_id, bytes_delta, total_written, ..
			} => {
				if let Some(bar) = bars.get(&item_id) {
					bar.set_position(total_written);
				}
				global_written += bytes_delta;
				global_bar.set_position(global_written);
			}
			ProgressEvent::Completed { item_id } => {
				if let Some(bar) = bars.remove(&item_id) {
					bar.finish();
				}
				files_done += 1;
				global_bar.set_prefix(format!("[{}/{} files]", files_done, files_total));
			}
			ProgressEvent::Skipped { file_name, .. } => {
				files_total += 1;
				files_done += 1;
				global_bar.set_prefix(format!("[{}/{} files]", files_done, files_total));
				eprintln!("  [SKIP] {}", file_name);
			}
			ProgressEvent::Failed { item_id, error } => {
				if let Some(bar) = bars.remove(&item_id) {
					bar.abandon_with_message(format!("FAILED: {}", error));
				} else {
					eprintln!("  [FAIL] {}", error);
				}
				files_done += 1;
				global_bar.set_prefix(format!("[{}/{} files]", files_done, files_total));
			}
			ProgressEvent::Cancelled { item_id } => {
				if let Some(bar) = bars.remove(&item_id) {
					bar.abandon_with_message("cancelled");
				}
				files_done += 1;
				global_bar.set_prefix(format!("[{}/{} files]", files_done, files_total));
			}
			ProgressEvent::AllDone {
				total_files,
				completed,
				failed,
				cancelled,
				skipped,
			} => {
				global_bar.finish_and_clear();
				let elapsed = started_at.elapsed();
				eprintln!();
				eprintln!(
					"Done: {} total, {} completed, {} skipped, {} failed, {} cancelled ({:.1}s)",
					total_files,
					completed,
					skipped,
					failed,
					cancelled,
					elapsed.as_secs_f64()
				);
				break;
			}
			ProgressEvent::ExtractionStarted { .. }
			| ProgressEvent::ExtractionProgress { .. }
			| ProgressEvent::ExtractionCompleted { .. }
			| ProgressEvent::ExtractionFailed { .. }
			| ProgressEvent::ExtractionAllDone { .. } => {}
		}
	}

	// Check result
	let download_failed = match manager_handle.await {
		Ok(Ok(result)) => result.failed > 0,
		Ok(Err(e)) => {
			eprintln!("Error: {}", e);
			std::process::exit(1);
		}
		Err(e) => {
			eprintln!("Error: {}", e);
			std::process::exit(1);
		}
	};

	// UC-POST-002: Auto-extraction
	if settings.auto_extract_archives {
		let (ext_tx, mut ext_rx) = mpsc::unbounded_channel::<ProgressEvent>();
		let ext_dir = settings.download_directory.clone();
		let delete_after = settings.delete_archives_after_extraction;

		let ext_handle = tokio::spawn(async move { rsfdl_core::extraction::extract_archives(&ext_dir, delete_after, &ext_tx).await });

		while let Some(event) = ext_rx.recv().await {
			match event {
				ProgressEvent::ExtractionStarted { archive_name, .. } => {
					eprintln!("  [EXTRACT] {}", archive_name);
				}
				ProgressEvent::ExtractionCompleted { archive_path } => {
					let name = archive_path.file_name().unwrap_or_default().to_string_lossy();
					eprintln!("  [DONE] Extracted {}", name);
				}
				ProgressEvent::ExtractionFailed { archive_path, error } => {
					let name = archive_path.file_name().unwrap_or_default().to_string_lossy();
					eprintln!("  [FAIL] {}: {}", name, error);
				}
				ProgressEvent::ExtractionAllDone { total_archives, extracted, failed } => {
					if total_archives > 0 {
						eprintln!("Extraction: {} archives, {} extracted, {} failed", total_archives, extracted, failed);
					}
				}
				_ => {}
			}
		}

		let _ = ext_handle.await;
	}

	if download_failed {
		std::process::exit(1);
	}
}

fn truncate_name(name: &str, max: usize) -> String {
	if name.chars().count() <= max {
		name.to_string()
	} else {
		let suffix: String = name.chars().rev().take(max - 3).collect::<Vec<_>>().into_iter().rev().collect();
		format!("...{suffix}")
	}
}
