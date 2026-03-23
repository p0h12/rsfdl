use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tokio::sync::mpsc;
use uuid::Uuid;

use rsfdl_core::download::manager::DownloadManager;
use rsfdl_core::download::progress::ProgressEvent;
use rsfdl_core::format_bytes;
use rsfdl_core::sfdl::crypto::{decrypt_container, try_passwords, validate_password};
use rsfdl_core::sfdl::parser::parse_sfdl;

pub async fn run(file: &str, password: Option<&str>, password_list: &[String], dest: Option<&str>, threads: u32, cli_exclude: &[String]) {
	// 1. Parse SFDL
	let xml = match std::fs::read_to_string(file) {
		Ok(s) => s,
		Err(e) => {
			eprintln!("Error: Cannot read file '{}': {}", file, e);
			std::process::exit(1);
		}
	};

	let mut container = match parse_sfdl(&xml) {
		Ok(c) => c,
		Err(e) => {
			eprintln!("Error: {}", e);
			std::process::exit(1);
		}
	};

	// 2. Decrypt if needed
	if container.encrypted {
		if let Some(pw) = password {
			if !validate_password(&container, pw) {
				eprintln!("Error: Invalid password");
				std::process::exit(1);
			}
			if let Err(e) = decrypt_container(&mut container, pw) {
				eprintln!("Error: Decryption failed: {}", e);
				std::process::exit(1);
			}
		} else if let Some(pw) = try_passwords(&container, password_list) {
			if let Err(e) = decrypt_container(&mut container, &pw) {
				eprintln!("Error: Auto-decrypt failed: {}", e);
				std::process::exit(1);
			}
			eprintln!("Auto-decrypted with password from list");
		} else {
			eprintln!("Error: File is encrypted. Provide a password with -p <password>");
			std::process::exit(1);
		}
	}

	// 3. Build settings (load saved, then override with CLI args)
	let saved_settings = rsfdl_core::settings::load_settings(&rsfdl_core::settings::default_settings_path());
	let mut settings = saved_settings;
	if let Some(d) = dest {
		settings.download_directory = PathBuf::from(d);
	} else {
		settings.download_directory = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
	}
	settings.max_download_threads = threads;

	// Apply file exclusion patterns (UC-15)
	let mut all_patterns = settings.file_exclusion_patterns.clone();
	all_patterns.extend_from_slice(cli_exclude);

	if !all_patterns.is_empty() {
		let mut excluded_count = 0usize;
		for package in &mut container.packages {
			let mask = rsfdl_core::filter::compute_exclusion_mask(&package.file_list, &all_patterns);
			let mut idx = 0;
			package.file_list.retain(|_| {
				let excluded = mask[idx];
				idx += 1;
				if excluded {
					excluded_count += 1;
				}
				!excluded
			});
		}
		if excluded_count > 0 {
			eprintln!("Excluded {} file(s) matching patterns", excluded_count);
		}
	}

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

	// 4. Create download manager
	let (manager, cancel_token, _file_cancel_tx) = DownloadManager::new(container, &settings);

	// 5. Ctrl+C handler
	let cancel_for_signal = cancel_token.clone();
	tokio::spawn(async move {
		tokio::signal::ctrl_c().await.ok();
		eprintln!("\nCancelling downloads...");
		cancel_for_signal.cancel();
	});

	// 6. Progress channel
	let (tx, mut rx) = mpsc::unbounded_channel::<ProgressEvent>();

	// 7. Run downloads in background
	let manager_handle = tokio::spawn(async move { manager.run(tx).await });

	// 8. Progress display loop
	let multi = MultiProgress::new();
	let mut bars: HashMap<Uuid, ProgressBar> = HashMap::new();

	// Global progress bar (inserted first, stays at top)
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

	// Global tracking state
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
			// Extraction events handled in extraction loop below
			ProgressEvent::ExtractionStarted { .. }
			| ProgressEvent::ExtractionProgress { .. }
			| ProgressEvent::ExtractionCompleted { .. }
			| ProgressEvent::ExtractionFailed { .. }
			| ProgressEvent::ExtractionAllDone { .. } => {}
		}
	}

	// 9. Check result
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

	// 10. Auto-extraction (UC-14)
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

		// Wait for extraction task to finish
		let _ = ext_handle.await;
	}

	if download_failed {
		std::process::exit(1);
	}
}

fn truncate_name(name: &str, max: usize) -> String {
	if name.len() <= max { name.to_string() } else { format!("...{}", &name[name.len() - (max - 3)..]) }
}
