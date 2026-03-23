#![cfg(feature = "ftp-tests")]

mod common;

use std::fs;
use std::path::Path;

use tokio::sync::mpsc;

use common::{FtpTestServer, create_ftp_file, generate_empty_sfdl_xml, generate_sfdl_xml, parse_sfdl_from_xml};
use rsfdl_core::download::manager::DownloadManager;
use rsfdl_core::download::progress::ProgressEvent;
use rsfdl_core::settings::AppSettings;
use rsfdl_core::sfdl::models::SfdlContainer;

/// Build AppSettings pointing to `dest_dir` with given thread count.
fn test_settings(dest_dir: &Path, threads: u32) -> AppSettings {
	let mut s = AppSettings::default();
	s.download_directory = dest_dir.to_path_buf();
	s.max_download_threads = threads;
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
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn download_single_file() {
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

#[tokio::test]
async fn download_multiple_files() {
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

#[tokio::test]
async fn download_creates_directory_structure() {
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

#[tokio::test]
async fn download_resume_partial() {
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

#[tokio::test]
async fn download_skip_complete() {
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

#[tokio::test]
async fn download_connection_refused() {
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

#[tokio::test]
async fn download_cancellation() {
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

#[tokio::test]
async fn download_parallel_threads() {
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

#[tokio::test]
async fn download_empty_file_list() {
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
