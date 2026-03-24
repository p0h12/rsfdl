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
use super::info::classify_exit_code;

/// CLI-003: Exit codes per spec.
const EXIT_PARTIAL_FAILURE: i32 = 10;
const EXIT_ALL_FAILED: i32 = 11;

#[allow(clippy::too_many_arguments)]
pub async fn run(
	args: &SfdlArgs,
	password_list: &[String],
	dest: Option<&str>,
	threads: Option<u32>,
	max_speed: Option<u32>,
	retries: Option<u32>,
	retry_delay: Option<u32>,
	strict_disk_check: bool,
	cli_exclude: &[String],
	no_exclude: bool,
	quiet: bool,
) {
	// SFDL-001 + SFDL-002: Parse and decrypt
	let (mut container, mut settings, _outcome) = match load_and_decrypt(args, password_list) {
		Ok(result) => result,
		Err(e) => {
			eprintln!("Error: {e}");
			std::process::exit(classify_exit_code(&e));
		}
	};

	// A6: CLI parameter overrides (BR-CFG-004)
	if let Some(d) = dest {
		settings.download_directory = PathBuf::from(d);
	} else {
		settings.download_directory = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
	}
	if let Some(t) = threads {
		settings.max_threads = t;
	}
	if let Some(s) = max_speed {
		settings.max_speed_kbps = s;
	}
	if let Some(r) = retries {
		settings.max_retries = r;
	}
	if let Some(d) = retry_delay {
		settings.retry_delay_seconds = d;
	}
	if strict_disk_check {
		settings.strict_disk_check = true;
	}

	// SFDL-003: Resolve BulkFolders before download
	if rsfdl_core::container::has_unresolved_bulk_folders(&container) {
		if !quiet {
			eprintln!("Resolving bulk folders via FTP...");
		}
		let warnings = rsfdl_core::container::resolve_bulk_folders(&mut container, settings.ftp_timeout_seconds).await;
		for w in &warnings {
			eprintln!("Warning: {}", w);
		}
	}

	// DL-002: Resolve exclusion patterns (settings + CLI)
	let patterns = rsfdl_core::filter::resolve_patterns(&settings.exclusion_patterns, cli_exclude, no_exclude);

	// DL-001: Compute file selection
	let selection = rsfdl_core::container::compute_file_selection(&container, &patterns);
	let excluded_count = selection.total_count() - selection.selected_count();
	if excluded_count > 0 && !quiet {
		eprintln!("Excluded {} file(s) matching patterns", excluded_count);
	}
	rsfdl_core::container::filter_container(&mut container, &selection);

	// Count files
	let file_count: usize = container.packages.iter().map(|p| p.file_list.len()).sum();
	let total_bytes: u64 = container.packages.iter().flat_map(|p| p.file_list.iter()).map(|f| f.file_size).sum();

	if !quiet {
		eprintln!("Connecting to {}:{}...", container.connection.host, container.connection.port);
		if file_count > 0 {
			eprintln!("Downloading {} files ({}) to {}", file_count, format_bytes(total_bytes), settings.download_directory.display());
		}
	}

	// DL-004: Create download manager
	let (manager, cancel_token, _file_cancel_tx) = DownloadManager::new(container, &settings);

	// DL-006 / BR-CLI-003-002: Ctrl+C handler
	let cancel_for_signal = cancel_token.clone();
	tokio::spawn(async move {
		tokio::signal::ctrl_c().await.ok();
		if !quiet {
			eprintln!("\nCancelling downloads...");
		}
		cancel_for_signal.cancel();
	});

	// Progress channel
	let (tx, mut rx) = mpsc::unbounded_channel::<ProgressEvent>();

	// Run downloads in background
	let manager_handle = tokio::spawn(async move { manager.run(tx).await });

	// Progress display
	let started_at = Instant::now();
	let mut final_result = None;

	if quiet {
		// Quiet mode: just drain events until AllDone
		while let Some(event) = rx.recv().await {
			if let ProgressEvent::AllDone {
				total_files,
				completed,
				failed,
				cancelled,
				skipped,
			} = event
			{
				final_result = Some((total_files, completed, failed, cancelled, skipped));
				break;
			}
		}
	} else {
		// Normal mode: progress bars
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
				ProgressEvent::Retry {
					item_id,
					attempt,
					max_retries,
					delay_seconds,
					error,
				} => {
					if let Some(bar) = bars.get(&item_id) {
						bar.set_message(format!("retry {}/{} in {}s: {}", attempt, max_retries, delay_seconds, error));
					}
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
					final_result = Some((total_files, completed, failed, cancelled, skipped));
					break;
				}
				ProgressEvent::ExtractionStarted { .. }
				| ProgressEvent::ExtractionProgress { .. }
				| ProgressEvent::ExtractionCompleted { .. }
				| ProgressEvent::ExtractionFailed { .. }
				| ProgressEvent::ExtractionAllDone { .. } => {}
			}
		}
	}

	// Print summary
	let elapsed = started_at.elapsed();
	if let Some((total_files, completed, failed, cancelled, skipped)) = final_result {
		eprintln!(
			"\nDone: {} total, {} completed, {} skipped, {} failed, {} cancelled ({:.1}s)",
			total_files,
			completed,
			skipped,
			failed,
			cancelled,
			elapsed.as_secs_f64()
		);

		// POST-002: Auto-extraction
		if settings.auto_extract {
			run_extraction(&settings, quiet).await;
		}

		// Determine exit code
		let exit_code = if cancelled > 0 {
			12 // A5: Signal abort
		} else if failed > 0 && completed == 0 {
			EXIT_ALL_FAILED // A4
		} else if failed > 0 {
			EXIT_PARTIAL_FAILURE // A3
		} else {
			0
		};

		if exit_code != 0 {
			std::process::exit(exit_code);
		}
	}

	// Check manager result for errors (e.g. disk space)
	match manager_handle.await {
		Ok(Ok(_)) => {}
		Ok(Err(e)) => {
			eprintln!("Error: {}", e);
			if e.to_string().contains("disk space") {
				std::process::exit(6); // A2: Insufficient disk space
			}
			std::process::exit(1);
		}
		Err(e) => {
			eprintln!("Error: {}", e);
			std::process::exit(1);
		}
	}
}

async fn run_extraction(settings: &rsfdl_core::settings::Settings, quiet: bool) {
	let (ext_tx, mut ext_rx) = mpsc::unbounded_channel::<ProgressEvent>();
	let ext_dir = settings.download_directory.clone();
	let delete_after = settings.delete_archives_after_extract;

	let ext_handle = tokio::spawn(async move { rsfdl_core::extraction::extract_archives(&ext_dir, delete_after, &ext_tx).await });

	while let Some(event) = ext_rx.recv().await {
		match event {
			ProgressEvent::ExtractionStarted { archive_name, .. } => {
				if !quiet {
					eprintln!("  [EXTRACT] {}", archive_name);
				}
			}
			ProgressEvent::ExtractionCompleted { archive_path } => {
				if !quiet {
					let name = archive_path.file_name().unwrap_or_default().to_string_lossy();
					eprintln!("  [DONE] Extracted {}", name);
				}
			}
			ProgressEvent::ExtractionFailed { archive_path, error } => {
				let name = archive_path.file_name().unwrap_or_default().to_string_lossy();
				eprintln!("  [FAIL] {}: {}", name, error);
			}
			ProgressEvent::ExtractionAllDone { total_archives, extracted, failed } => {
				if total_archives > 0 && !quiet {
					eprintln!("Extraction: {} archives, {} extracted, {} failed", total_archives, extracted, failed);
				}
			}
			_ => {}
		}
	}

	let _ = ext_handle.await;
}

fn truncate_name(name: &str, max: usize) -> String {
	if name.chars().count() <= max {
		name.to_string()
	} else {
		let suffix: String = name.chars().rev().take(max - 3).collect::<Vec<_>>().into_iter().rev().collect();
		format!("...{suffix}")
	}
}
