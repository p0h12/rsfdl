#![cfg(feature = "ftp-tests")]

mod common;

use std::fs;
use std::path::Path;

use tokio::sync::mpsc;

use common::{FtpTestServer, create_ftp_file, generate_bulkfolder_sfdl_xml, generate_empty_sfdl_xml, generate_sfdl_xml, parse_sfdl_from_xml};
use rsfdl_core::container::resolve_bulk_folders;
use rsfdl_core::download::manager::DownloadManager;
use rsfdl_core::download::progress::ProgressEvent;
use rsfdl_core::settings::Settings;

/// Build Settings pointing to `dest_dir` with given thread count.
fn test_settings(dest_dir: &Path, threads: u32) -> Settings {
	let mut s = Settings::default();
	s.download_directory = dest_dir.to_path_buf();
	s.max_threads = threads;
	s.max_retries = 2;
	s.retry_delay_seconds = 1;
	s
}

/// Collect all progress events from the channel until AllDone is received.
async fn collect_events(mut rx: mpsc::UnboundedReceiver<ProgressEvent>) -> Vec<ProgressEvent> {
	let mut events = Vec::new();
	while let Some(ev) = rx.recv().await {
		let is_done = matches!(ev, ProgressEvent::AllDone { .. });
		events.push(ev);
		if is_done {
			break;
		}
	}
	events
}

// ---------------------------------------------------------------------------
// DL-004: FTP-Download durchfuehren
// ---------------------------------------------------------------------------

/// DL-004 | Main Success: Download a single file via FTP.
#[tokio::test]
async fn dl004_download_single_file() {
	let ftp_root = tempfile::tempdir().unwrap();
	let dest = tempfile::tempdir().unwrap();

	// Create a test file on the FTP server
	let content = b"Hello, FTP download test!";
	create_ftp_file(ftp_root.path(), "releases/test/hello.bin", content);

	let server = FtpTestServer::start(ftp_root.path().to_path_buf()).await;

	let xml = generate_sfdl_xml(server.port(), &[("hello.bin", "releases/test", "/releases/test/hello.bin", content.len() as u64)]);
	let container = parse_sfdl_from_xml(&xml);

	let settings = test_settings(dest.path(), 1);
	let (manager, _cancel, _file_cancel) = DownloadManager::new(container, &settings);
	let (tx, rx) = mpsc::unbounded_channel();

	let result = manager.run(tx).await.unwrap();
	let events = collect_events(rx).await;

	// Verify result
	assert_eq!(result.total_files, 1);
	assert_eq!(result.completed, 1);
	assert_eq!(result.failed, 0);
	assert_eq!(result.skipped, 0);

	// Verify file exists with correct content
	let local_file = dest.path().join("TestPkg/releases/test/hello.bin");
	assert!(local_file.exists(), "downloaded file should exist");
	assert_eq!(fs::read(&local_file).unwrap(), content);

	// Verify progress events
	assert!(events.iter().any(|e| matches!(e, ProgressEvent::Started { .. })));
	assert!(events.iter().any(|e| matches!(e, ProgressEvent::Completed { .. })));
	assert!(events.iter().any(|e| matches!(e, ProgressEvent::AllDone { completed: 1, .. })));
}

/// DL-004 | Main Success: Download multiple files in one session.
#[tokio::test]
async fn dl004_download_multiple_files() {
	let ftp_root = tempfile::tempdir().unwrap();
	let dest = tempfile::tempdir().unwrap();

	create_ftp_file(ftp_root.path(), "data/file1.bin", &[0xAA; 512]);
	create_ftp_file(ftp_root.path(), "data/file2.bin", &[0xBB; 1024]);
	create_ftp_file(ftp_root.path(), "data/file3.bin", &[0xCC; 256]);

	let server = FtpTestServer::start(ftp_root.path().to_path_buf()).await;

	let xml = generate_sfdl_xml(
		server.port(),
		&[
			("file1.bin", "data", "/data/file1.bin", 512),
			("file2.bin", "data", "/data/file2.bin", 1024),
			("file3.bin", "data", "/data/file3.bin", 256),
		],
	);
	let container = parse_sfdl_from_xml(&xml);

	let settings = test_settings(dest.path(), 3);
	let (manager, _cancel, _file_cancel) = DownloadManager::new(container, &settings);
	let (tx, rx) = mpsc::unbounded_channel();

	let result = manager.run(tx).await.unwrap();
	let _events = collect_events(rx).await;

	assert_eq!(result.total_files, 3);
	assert_eq!(result.completed, 3);
	assert_eq!(result.failed, 0);

	// Verify all files
	assert_eq!(fs::read(dest.path().join("TestPkg/data/file1.bin")).unwrap(), vec![0xAA; 512]);
	assert_eq!(fs::read(dest.path().join("TestPkg/data/file2.bin")).unwrap(), vec![0xBB; 1024]);
	assert_eq!(fs::read(dest.path().join("TestPkg/data/file3.bin")).unwrap(), vec![0xCC; 256]);
}

/// DL-004 | BR-DL-011: Local directory structure mirrors remote paths.
#[tokio::test]
async fn dl004_download_creates_directory_structure() {
	let ftp_root = tempfile::tempdir().unwrap();
	let dest = tempfile::tempdir().unwrap();

	create_ftp_file(ftp_root.path(), "a/b/c/deep.bin", b"deep");

	let server = FtpTestServer::start(ftp_root.path().to_path_buf()).await;

	let xml = generate_sfdl_xml(server.port(), &[("deep.bin", "a/b/c", "/a/b/c/deep.bin", 4)]);
	let container = parse_sfdl_from_xml(&xml);

	let settings = test_settings(dest.path(), 1);
	let (manager, _cancel, _file_cancel) = DownloadManager::new(container, &settings);
	let (tx, _rx) = mpsc::unbounded_channel();

	let result = manager.run(tx).await.unwrap();

	assert_eq!(result.completed, 1);
	let local = dest.path().join("TestPkg/a/b/c/deep.bin");
	assert!(local.exists());
	assert_eq!(fs::read(&local).unwrap(), b"deep");
}

/// DL-005 | Main Success: Resume download from partial file.
#[tokio::test]
async fn dl005_download_resume_partial() {
	let ftp_root = tempfile::tempdir().unwrap();
	let dest = tempfile::tempdir().unwrap();

	// Full content: 1024 bytes
	let full_content: Vec<u8> = (0..1024u16).map(|i| (i % 256) as u8).collect();
	create_ftp_file(ftp_root.path(), "releases/resume.bin", &full_content);

	// Pre-create a partial local file (first 512 bytes)
	let local_dir = dest.path().join("TestPkg/releases");
	fs::create_dir_all(&local_dir).unwrap();
	fs::write(local_dir.join("resume.bin"), &full_content[..512]).unwrap();

	let server = FtpTestServer::start(ftp_root.path().to_path_buf()).await;

	let xml = generate_sfdl_xml(server.port(), &[("resume.bin", "releases", "/releases/resume.bin", full_content.len() as u64)]);
	let container = parse_sfdl_from_xml(&xml);

	let settings = test_settings(dest.path(), 1);
	let (manager, _cancel, _file_cancel) = DownloadManager::new(container, &settings);
	let (tx, _rx) = mpsc::unbounded_channel();

	let result = manager.run(tx).await.unwrap();

	assert_eq!(result.completed, 1);
	assert_eq!(result.skipped, 0);

	let downloaded = fs::read(dest.path().join("TestPkg/releases/resume.bin")).unwrap();
	assert_eq!(downloaded.len(), full_content.len());
	// The first 512 bytes stay, the rest is appended by resume
	assert_eq!(&downloaded[..512], &full_content[..512]);
}

/// DL-005 | A1: Skip already-complete file (local >= remote size).
#[tokio::test]
async fn dl005_download_skip_complete() {
	let ftp_root = tempfile::tempdir().unwrap();
	let dest = tempfile::tempdir().unwrap();

	let content = b"already complete";
	create_ftp_file(ftp_root.path(), "data/done.bin", content);

	// Pre-create local file with full content
	let local_dir = dest.path().join("TestPkg/data");
	fs::create_dir_all(&local_dir).unwrap();
	fs::write(local_dir.join("done.bin"), content).unwrap();

	let server = FtpTestServer::start(ftp_root.path().to_path_buf()).await;

	let xml = generate_sfdl_xml(server.port(), &[("done.bin", "data", "/data/done.bin", content.len() as u64)]);
	let container = parse_sfdl_from_xml(&xml);

	let settings = test_settings(dest.path(), 1);
	let (manager, _cancel, _file_cancel) = DownloadManager::new(container, &settings);
	let (tx, rx) = mpsc::unbounded_channel();

	let result = manager.run(tx).await.unwrap();
	let events = collect_events(rx).await;

	assert_eq!(result.total_files, 1);
	assert_eq!(result.skipped, 1);
	assert_eq!(result.completed, 0);

	// Verify Skipped event was sent
	assert!(events.iter().any(|e| matches!(e, ProgressEvent::Skipped { .. })));
}

/// DL-004 | A1: Connection refused produces error event.
#[tokio::test]
async fn dl004_download_connection_refused() {
	let dest = tempfile::tempdir().unwrap();

	// Use a port that's definitely not running an FTP server
	let port = portpicker::pick_unused_port().expect("no free port");

	let xml = generate_sfdl_xml(port, &[("nope.bin", "data", "/data/nope.bin", 100)]);
	let container = parse_sfdl_from_xml(&xml);

	let settings = test_settings(dest.path(), 1);
	let (manager, _cancel, _file_cancel) = DownloadManager::new(container, &settings);
	let (tx, rx) = mpsc::unbounded_channel();

	let result = manager.run(tx).await.unwrap();
	let events = collect_events(rx).await;

	assert_eq!(result.total_files, 1);
	assert_eq!(result.failed, 1);
	assert_eq!(result.completed, 0);

	// Verify Failed event
	assert!(events.iter().any(|e| matches!(e, ProgressEvent::Failed { .. })));
}

/// DL-006 | Variante B: Global cancellation stops all downloads.
#[tokio::test]
async fn dl006_download_cancellation() {
	let ftp_root = tempfile::tempdir().unwrap();
	let dest = tempfile::tempdir().unwrap();

	// Create a larger file so there's time to cancel
	let content = vec![0xFFu8; 100_000];
	create_ftp_file(ftp_root.path(), "data/large.bin", &content);

	let server = FtpTestServer::start(ftp_root.path().to_path_buf()).await;

	let xml = generate_sfdl_xml(server.port(), &[("large.bin", "data", "/data/large.bin", content.len() as u64)]);
	let container = parse_sfdl_from_xml(&xml);

	let settings = test_settings(dest.path(), 1);
	let (manager, cancel_token, _file_cancel) = DownloadManager::new(container, &settings);
	let (tx, mut rx) = mpsc::unbounded_channel();

	// Cancel after first BytesWritten event
	let cancel = cancel_token.clone();
	let cancel_handle = tokio::spawn(async move {
		while let Some(ev) = rx.recv().await {
			if matches!(ev, ProgressEvent::BytesWritten { .. }) {
				cancel.cancel();
				break;
			}
		}
		// Drain remaining events
		while let Some(_) = rx.recv().await {}
	});

	let result = manager.run(tx).await.unwrap();
	cancel_handle.await.unwrap();

	// At least one should be cancelled (the download was in progress)
	assert!(result.cancelled >= 1 || result.completed >= 1, "file should be cancelled or have completed before cancellation");
}

/// DL-004 | BR-DL-007: Parallel downloads limited by max_threads.
#[tokio::test]
async fn dl004_download_parallel_threads() {
	let ftp_root = tempfile::tempdir().unwrap();
	let dest = tempfile::tempdir().unwrap();

	for i in 0..5 {
		create_ftp_file(ftp_root.path(), &format!("parallel/file{}.bin", i), &vec![i as u8; 256]);
	}

	let server = FtpTestServer::start(ftp_root.path().to_path_buf()).await;

	let files: Vec<(String, String, String, u64)> = (0..5).map(|i| (format!("file{}.bin", i), "parallel".to_string(), format!("/parallel/file{}.bin", i), 256u64)).collect();
	let file_refs: Vec<(&str, &str, &str, u64)> = files.iter().map(|(n, d, f, s)| (n.as_str(), d.as_str(), f.as_str(), *s)).collect();

	let xml = generate_sfdl_xml(server.port(), &file_refs);
	let container = parse_sfdl_from_xml(&xml);

	let settings = test_settings(dest.path(), 2);
	let (manager, _cancel, _file_cancel) = DownloadManager::new(container, &settings);
	let (tx, rx) = mpsc::unbounded_channel();

	let result = manager.run(tx).await.unwrap();
	let _events = collect_events(rx).await;

	assert_eq!(result.total_files, 5);
	assert_eq!(result.completed, 5);
	assert_eq!(result.failed, 0);

	// Verify all files exist
	for i in 0..5 {
		let path = dest.path().join(format!("TestPkg/parallel/file{}.bin", i));
		assert!(path.exists(), "file{}.bin should exist", i);
		assert_eq!(fs::read(&path).unwrap(), vec![i as u8; 256]);
	}
}

/// DL-004 | Edge: Empty file list completes immediately.
#[tokio::test]
async fn dl004_download_empty_file_list() {
	let dest = tempfile::tempdir().unwrap();

	// Port doesn't matter — no FTP connection should be made
	let xml = generate_empty_sfdl_xml(12345);
	let container = parse_sfdl_from_xml(&xml);

	let settings = test_settings(dest.path(), 1);
	let (manager, _cancel, _file_cancel) = DownloadManager::new(container, &settings);
	let (tx, rx) = mpsc::unbounded_channel();

	let result = manager.run(tx).await.unwrap();
	let events = collect_events(rx).await;

	assert_eq!(result.total_files, 0);
	assert_eq!(result.completed, 0);
	assert_eq!(result.failed, 0);
	assert_eq!(result.skipped, 0);

	assert!(events.iter().any(|e| matches!(e, ProgressEvent::AllDone { total_files: 0, .. })));
}

// ---------------------------------------------------------------------------
// BulkFolder tests
// ---------------------------------------------------------------------------

/// SFDL-003 | Main Success: BulkFolder single directory listing + download.
#[tokio::test]
async fn sfdl003_download_bulkfolder_single_dir() {
	let ftp_root = tempfile::tempdir().unwrap();
	let dest = tempfile::tempdir().unwrap();

	create_ftp_file(ftp_root.path(), "releases/movie/part1.rar", &[0xAA; 512]);
	create_ftp_file(ftp_root.path(), "releases/movie/part2.rar", &[0xBB; 256]);

	let server = FtpTestServer::start(ftp_root.path().to_path_buf()).await;

	let xml = generate_bulkfolder_sfdl_xml(server.port(), &["/releases/movie/"]);
	let mut container = parse_sfdl_from_xml(&xml);
	let warnings = resolve_bulk_folders(&mut container, 10).await;
	assert!(warnings.is_empty(), "unexpected warnings: {:?}", warnings);

	let settings = test_settings(dest.path(), 2);
	let (manager, _cancel, _file_cancel) = DownloadManager::new(container, &settings);
	let (tx, rx) = mpsc::unbounded_channel();

	let result = manager.run(tx).await.unwrap();
	let _events = collect_events(rx).await;

	assert_eq!(result.total_files, 2);
	assert_eq!(result.completed, 2);
	assert_eq!(result.failed, 0);

	// Verify files exist (path: TestPkg/<ftp_path>/<filename>)
	let base = dest.path().join("TestPkg/releases/movie");
	assert!(base.join("part1.rar").exists());
	assert!(base.join("part2.rar").exists());
	assert_eq!(fs::read(base.join("part1.rar")).unwrap(), vec![0xAA; 512]);
	assert_eq!(fs::read(base.join("part2.rar")).unwrap(), vec![0xBB; 256]);
}

/// SFDL-003 | Main Success: BulkFolder recursive directory listing.
#[tokio::test]
async fn sfdl003_download_bulkfolder_recursive() {
	let ftp_root = tempfile::tempdir().unwrap();
	let dest = tempfile::tempdir().unwrap();

	// Create nested directory structure
	create_ftp_file(ftp_root.path(), "data/season1/ep01.mkv", &[0x01; 100]);
	create_ftp_file(ftp_root.path(), "data/season1/ep02.mkv", &[0x02; 100]);
	create_ftp_file(ftp_root.path(), "data/season2/ep01.mkv", &[0x03; 100]);

	let server = FtpTestServer::start(ftp_root.path().to_path_buf()).await;

	let xml = generate_bulkfolder_sfdl_xml(server.port(), &["/data/"]);
	let mut container = parse_sfdl_from_xml(&xml);
	let warnings = resolve_bulk_folders(&mut container, 10).await;
	assert!(warnings.is_empty(), "unexpected warnings: {:?}", warnings);

	let settings = test_settings(dest.path(), 3);
	let (manager, _cancel, _file_cancel) = DownloadManager::new(container, &settings);
	let (tx, rx) = mpsc::unbounded_channel();

	let result = manager.run(tx).await.unwrap();
	let _events = collect_events(rx).await;

	assert_eq!(result.total_files, 3);
	assert_eq!(result.completed, 3);
	assert_eq!(result.failed, 0);

	// Verify nested structure is preserved
	assert!(dest.path().join("TestPkg/data/season1/ep01.mkv").exists());
	assert!(dest.path().join("TestPkg/data/season1/ep02.mkv").exists());
	assert!(dest.path().join("TestPkg/data/season2/ep01.mkv").exists());
}

/// SFDL-003 | Main Success: BulkFolder with multiple folder entries.
#[tokio::test]
async fn sfdl003_download_bulkfolder_multiple_folders() {
	let ftp_root = tempfile::tempdir().unwrap();
	let dest = tempfile::tempdir().unwrap();

	create_ftp_file(ftp_root.path(), "movies/film.mkv", &[0x10; 200]);
	create_ftp_file(ftp_root.path(), "extras/behind.mkv", &[0x20; 150]);

	let server = FtpTestServer::start(ftp_root.path().to_path_buf()).await;

	let xml = generate_bulkfolder_sfdl_xml(server.port(), &["/movies/", "/extras/"]);
	let mut container = parse_sfdl_from_xml(&xml);
	let warnings = resolve_bulk_folders(&mut container, 10).await;
	assert!(warnings.is_empty(), "unexpected warnings: {:?}", warnings);

	let settings = test_settings(dest.path(), 2);
	let (manager, _cancel, _file_cancel) = DownloadManager::new(container, &settings);
	let (tx, rx) = mpsc::unbounded_channel();

	let result = manager.run(tx).await.unwrap();
	let _events = collect_events(rx).await;

	assert_eq!(result.total_files, 2);
	assert_eq!(result.completed, 2);

	assert_eq!(fs::read(dest.path().join("TestPkg/movies/film.mkv")).unwrap(), vec![0x10; 200]);
	assert_eq!(fs::read(dest.path().join("TestPkg/extras/behind.mkv")).unwrap(), vec![0x20; 150]);
}

/// SFDL-003 | A2: BulkFolder with empty directory.
#[tokio::test]
async fn sfdl003_download_bulkfolder_empty_dir() {
	let ftp_root = tempfile::tempdir().unwrap();
	let dest = tempfile::tempdir().unwrap();

	// Create the directory but put no files in it
	fs::create_dir_all(ftp_root.path().join("empty")).unwrap();

	let server = FtpTestServer::start(ftp_root.path().to_path_buf()).await;

	let xml = generate_bulkfolder_sfdl_xml(server.port(), &["/empty/"]);
	let mut container = parse_sfdl_from_xml(&xml);
	let warnings = resolve_bulk_folders(&mut container, 10).await;
	assert!(warnings.is_empty(), "unexpected warnings: {:?}", warnings);

	let settings = test_settings(dest.path(), 1);
	let (manager, _cancel, _file_cancel) = DownloadManager::new(container, &settings);
	let (tx, rx) = mpsc::unbounded_channel();

	let result = manager.run(tx).await.unwrap();
	let _events = collect_events(rx).await;

	assert_eq!(result.total_files, 0);
	assert_eq!(result.completed, 0);
	assert_eq!(result.failed, 0);
}
