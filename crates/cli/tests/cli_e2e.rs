use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn cmd() -> Command {
	assert_cmd::cargo_bin_cmd!("rsfdl-cli")
}

fn fixture(name: &str) -> String {
	let base = env!("CARGO_MANIFEST_DIR");
	format!("{}/../../crates/core/tests/fixtures/{}", base, name)
}

/// Helper: create a settings.json in a temp dir and return (dir, path)
fn create_settings_file(json: &str) -> (TempDir, String) {
	let dir = tempfile::tempdir().unwrap();
	let path = dir.path().join("settings.json");
	std::fs::write(&path, json).unwrap();
	(dir, path.to_string_lossy().into_owned())
}

// --- AT-18: CLI Help ---

#[test]
fn help_shows_subcommands() {
	cmd()
		.arg("--help")
		.assert()
		.success()
		.stdout(predicate::str::contains("info"))
		.stdout(predicate::str::contains("list"))
		.stdout(predicate::str::contains("download"));
}

// --- AT-07: CLI info (unencrypted) ---

#[test]
fn info_unencrypted_v3() {
	cmd()
		.args(["info", &fixture("unencrypted_v3.sfdl")])
		.assert()
		.success()
		.stdout(predicate::str::contains("Test.Release.2026.1080p"))
		.stdout(predicate::str::contains("testuser"))
		.stdout(predicate::str::contains("ftp.example.com"))
		.stdout(predicate::str::contains("21"))
		.stdout(predicate::str::contains("2"));
}

// --- AT-07: CLI info (encrypted with password) ---

#[test]
fn info_encrypted_v3_with_password() {
	cmd()
		.args(["info", &fixture("encrypted_v3.sfdl"), "-p", "test"])
		.assert()
		.success()
		.stdout(predicate::str::contains("Test.Release.2026.1080p"))
		.stdout(predicate::str::contains("ftp.example.com"));
}

// --- AT-16: Encrypted without password ---

#[test]
fn info_encrypted_without_password_fails() {
	cmd()
		.args(["info", &fixture("encrypted_v3.sfdl")])
		.assert()
		.failure()
		.stderr(predicate::str::is_match("(?i)password").unwrap());
}

// --- AT-04: Wrong password ---

#[test]
fn info_encrypted_wrong_password_fails() {
	cmd()
		.args(["info", &fixture("encrypted_v3.sfdl"), "-p", "wrong"])
		.assert()
		.failure()
		.stderr(predicate::str::is_match("(?i)password").unwrap());
}

// --- File not found ---

#[test]
fn info_nonexistent_file_fails() {
	cmd().args(["info", "/tmp/nonexistent_file_12345.sfdl"]).assert().failure();
}

// --- Invalid SFDL ---

#[test]
fn info_invalid_file_fails() {
	cmd().args(["info", &fixture("invalid.sfdl")]).assert().failure();
}

// --- AT-08: CLI list (unencrypted) ---

#[test]
fn list_unencrypted_v3() {
	cmd()
		.args(["list", &fixture("unencrypted_v3.sfdl")])
		.assert()
		.success()
		.stdout(predicate::str::contains("movie.part1.rar"))
		.stdout(predicate::str::contains("movie.part2.rar"))
		.stdout(predicate::str::contains("2 files"));
}

// --- CLI list (encrypted with password) ---

#[test]
fn list_encrypted_v3_with_password() {
	cmd()
		.args(["list", &fixture("encrypted_v3.sfdl"), "-p", "test"])
		.assert()
		.success()
		.stdout(predicate::str::contains("movie.part1.rar"))
		.stdout(predicate::str::contains("movie.part2.rar"));
}

// --- AT-25: CLI download --exclude flag exists ---

#[test]
fn download_help_shows_exclude_flag() {
	cmd().args(["download", "--help"]).assert().success().stdout(predicate::str::contains("--exclude"));
}

// --- CLI list v2 ---

#[test]
fn list_unencrypted_v2_shows_bulk_folders() {
	cmd()
		.args(["list", &fixture("unencrypted_v2.sfdl")])
		.assert()
		.success()
		.stdout(predicate::str::contains("/releases/test/"));
}

// --- AT-18 update: Help shows config subcommand ---

#[test]
fn help_shows_config_subcommand() {
	cmd().arg("--help").assert().success().stdout(predicate::str::contains("config"));
}

// --- AT-39: config show with existing settings file ---

#[test]
fn config_show_displays_settings_and_path() {
	let (_dir, path) = create_settings_file(
		r#"{
            "download_directory": "/tmp/downloads",
            "max_download_threads": 5,
            "max_retries": 3,
            "retry_wait_seconds": 10,
            "auto_password_list": ["pw1", "pw2"],
            "resume_downloads": true,
            "create_package_subfolder": true,
            "ftp_timeout_seconds": 30
        }"#,
	);

	cmd()
		.args(["config", "show", "--config-file", &path])
		.assert()
		.success()
		.stdout(predicate::str::contains(&path))
		.stdout(predicate::str::contains("/tmp/downloads"))
		.stdout(predicate::str::contains("5"));
}

// --- AT-40: config show without settings file shows defaults ---

#[test]
fn config_show_without_file_shows_defaults() {
	let dir = tempfile::tempdir().unwrap();
	let path = dir.path().join("nonexistent/settings.json");

	cmd()
		.args(["config", "show", "--config-file", &path.to_string_lossy()])
		.assert()
		.success()
		.stdout(predicate::str::contains("max_download_threads"))
		.stdout(predicate::str::contains("3"));
}

// --- AT-41: config edit creates file if missing ---

#[test]
fn config_edit_creates_default_file() {
	let dir = tempfile::tempdir().unwrap();
	let path = dir.path().join("rsfdl/settings.json");

	// Use `true` as editor — it exits immediately with success
	cmd().args(["config", "edit", "--config-file", &path.to_string_lossy()]).env("EDITOR", "true").assert().success();

	// File should now exist with valid JSON defaults
	assert!(path.exists(), "Settings file should have been created");
	let content = std::fs::read_to_string(&path).unwrap();
	let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
	assert_eq!(parsed["max_download_threads"], 3);
	assert_eq!(parsed["resume_downloads"], true);
}

// --- AT-42: Download override does not modify settings file ---

#[test]
fn download_override_does_not_modify_settings_file() {
	let json = r#"{
        "download_directory": "/tmp/original",
        "max_download_threads": 3,
        "max_retries": 3,
        "retry_wait_seconds": 10,
        "auto_password_list": [],
        "resume_downloads": true,
        "create_package_subfolder": true,
        "ftp_timeout_seconds": 30
    }"#;
	let (_dir, path) = create_settings_file(json);
	let content_before = std::fs::read_to_string(&path).unwrap();

	// Download will fail (no FTP server), but that's OK —
	// we only care that the settings file is unchanged after the attempt.
	let _ = cmd()
		.args(["download", &fixture("unencrypted_v3.sfdl"), "--threads", "7", "--dest", "/tmp/override_dest", "--config-file", &path])
		.assert();

	let content_after = std::fs::read_to_string(&path).unwrap();
	assert_eq!(content_before, content_after, "Settings file must not be modified by CLI overrides");
}

// --- Info/List/Download use auto_password_list from settings ---

#[test]
fn info_uses_auto_password_list_from_settings() {
	let json = r#"{
        "download_directory": "/tmp/downloads",
        "max_download_threads": 3,
        "max_retries": 3,
        "retry_wait_seconds": 10,
        "auto_password_list": ["wrong1", "test", "wrong2"],
        "resume_downloads": true,
        "create_package_subfolder": true,
        "ftp_timeout_seconds": 30
    }"#;
	let (_dir, path) = create_settings_file(json);

	cmd()
		.args(["info", &fixture("encrypted_v3.sfdl"), "--config-file", &path])
		.assert()
		.success()
		.stderr(predicate::str::contains("Auto-decrypted with password from list"))
		.stdout(predicate::str::contains("ftp.example.com"));
}

#[test]
fn list_uses_auto_password_list_from_settings() {
	let json = r#"{
        "download_directory": "/tmp/downloads",
        "max_download_threads": 3,
        "max_retries": 3,
        "retry_wait_seconds": 10,
        "auto_password_list": ["wrong1", "test", "wrong2"],
        "resume_downloads": true,
        "create_package_subfolder": true,
        "ftp_timeout_seconds": 30
    }"#;
	let (_dir, path) = create_settings_file(json);

	cmd()
		.args(["list", &fixture("encrypted_v3.sfdl"), "--config-file", &path])
		.assert()
		.success()
		.stderr(predicate::str::contains("Auto-decrypted with password from list"))
		.stdout(predicate::str::contains("movie.part1.rar"));
}

// --- Download uses auto_password_list from settings ---

#[test]
fn download_uses_auto_password_list_from_settings() {
	let json = r#"{
        "download_directory": "/tmp/downloads",
        "max_download_threads": 3,
        "max_retries": 3,
        "retry_wait_seconds": 10,
        "auto_password_list": ["wrong1", "test", "wrong2"],
        "resume_downloads": true,
        "create_package_subfolder": true,
        "ftp_timeout_seconds": 30
    }"#;
	let (_dir, path) = create_settings_file(json);

	// Download will fail at FTP connection, but decryption should succeed
	// using "test" from auto_password_list — no -p flag needed
	cmd()
		.args(["download", &fixture("encrypted_v3.sfdl"), "--config-file", &path])
		.assert()
		.stderr(predicate::str::contains("Auto-decrypted with password from list"));
}

// --- AT-43: config show with corrupt settings file shows defaults ---

#[test]
fn config_show_with_corrupt_file_shows_defaults() {
	let (_dir, path) = create_settings_file("not valid json {{{");

	cmd()
		.args(["config", "show", "--config-file", &path])
		.assert()
		.success()
		.stdout(predicate::str::contains("max_download_threads"))
		.stdout(predicate::str::contains("3"));
}
