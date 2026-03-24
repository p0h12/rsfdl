#![cfg(feature = "ftp-tests")]

mod common;

use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;

use common::{FtpTestServer, create_ftp_file, generate_sfdl_xml, write_sfdl_to_file};

fn cmd() -> Command {
	Command::cargo_bin("rsfdl-cli").unwrap()
}

/// Create a temp settings file with fast retry settings for tests.
fn fast_settings() -> (tempfile::TempDir, String) {
	let dir = tempfile::tempdir().unwrap();
	let path = dir.path().join("settings.toml");
	fs::write(
		&path,
		r#"
max_retries = 1
retry_delay_seconds = 1
ftp_timeout_seconds = 2
"#,
	)
	.unwrap();
	let path_str = path.to_str().unwrap().to_string();
	(dir, path_str)
}

#[tokio::test]
async fn cli_download_success() {
	let ftp_root = tempfile::tempdir().unwrap();
	let dest = tempfile::tempdir().unwrap();
	let sfdl_dir = tempfile::tempdir().unwrap();

	let content = b"CLI download test content";
	create_ftp_file(ftp_root.path(), "releases/test/movie.rar", content);

	let server = FtpTestServer::start(ftp_root.path().to_path_buf()).await;

	let xml = generate_sfdl_xml(server.port(), &[("movie.rar", "releases/test", "/releases/test/movie.rar", content.len() as u64)]);
	let sfdl_path = write_sfdl_to_file(sfdl_dir.path(), &xml);
	let sfdl_str = sfdl_path.to_str().unwrap().to_string();
	let dest_str = dest.path().to_str().unwrap().to_string();

	let (_cfg_dir, cfg_path) = fast_settings();

	let output = tokio::task::spawn_blocking(move || {
		cmd()
			.args(["download", &sfdl_str, "-o", &dest_str])
			.env("RSFDL_CONFIG", &cfg_path)
			.assert()
			.success()
			.stderr(predicate::str::contains("Done:"))
			.stderr(predicate::str::contains("1 completed"))
	})
	.await
	.unwrap();

	let _ = output;

	let local = dest.path().join("TestPkg/releases/test/movie.rar");
	assert!(local.exists(), "downloaded file should exist");
	assert_eq!(fs::read(&local).unwrap(), content);
}

#[tokio::test]
async fn cli_download_with_dest_flag() {
	let ftp_root = tempfile::tempdir().unwrap();
	let dest = tempfile::tempdir().unwrap();
	let sfdl_dir = tempfile::tempdir().unwrap();

	create_ftp_file(ftp_root.path(), "data/file.bin", b"dest-test");

	let server = FtpTestServer::start(ftp_root.path().to_path_buf()).await;

	let xml = generate_sfdl_xml(server.port(), &[("file.bin", "data", "/data/file.bin", 9)]);
	let sfdl_path = write_sfdl_to_file(sfdl_dir.path(), &xml);
	let sfdl_str = sfdl_path.to_str().unwrap().to_string();
	let dest_str = dest.path().to_str().unwrap().to_string();

	let (_cfg_dir, cfg_path) = fast_settings();

	tokio::task::spawn_blocking(move || {
		cmd().args(["download", &sfdl_str, "-o", &dest_str]).env("RSFDL_CONFIG", &cfg_path).assert().success();
	})
	.await
	.unwrap();

	assert!(dest.path().join("TestPkg/data/file.bin").exists());
}

#[tokio::test]
async fn cli_download_connection_error() {
	let sfdl_dir = tempfile::tempdir().unwrap();
	let dest = tempfile::tempdir().unwrap();

	let port = portpicker::pick_unused_port().expect("no free port");
	let xml = generate_sfdl_xml(port, &[("nope.bin", "data", "/data/nope.bin", 100)]);
	let sfdl_path = write_sfdl_to_file(sfdl_dir.path(), &xml);
	let sfdl_str = sfdl_path.to_str().unwrap().to_string();
	let dest_str = dest.path().to_str().unwrap().to_string();

	let (_cfg_dir, cfg_path) = fast_settings();

	tokio::task::spawn_blocking(move || {
		cmd()
			.args(["download", &sfdl_str, "-o", &dest_str])
			.env("RSFDL_CONFIG", &cfg_path)
			.assert()
			.failure()
			.stderr(predicate::str::contains("1 failed"));
	})
	.await
	.unwrap();
}

#[tokio::test]
async fn cli_download_nonexistent_sfdl() {
	tokio::task::spawn_blocking(move || {
		cmd().args(["download", "/tmp/nonexistent_12345.sfdl"]).assert().failure();
	})
	.await
	.unwrap();
}
