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

/// Helper: create a settings.toml in a temp dir and return (dir, path)
fn create_settings_file(toml: &str) -> (TempDir, String) {
	let dir = tempfile::tempdir().unwrap();
	let path = dir.path().join("settings.toml");
	std::fs::write(&path, toml).unwrap();
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

/// CLI-001 | A3: Nonexistent file → exit code 1.
#[test]
fn info_nonexistent_file_fails() {
	cmd().args(["info", "/tmp/nonexistent_file_12345.sfdl"]).assert().failure().code(1);
}

/// CLI-001 | A4: Invalid SFDL → exit code 2.
#[test]
fn info_invalid_file_fails() {
	cmd().args(["info", &fixture("invalid.sfdl")]).assert().failure().code(2);
}

/// CLI-001 | A2: Wrong password → exit code 4.
#[test]
fn info_wrong_password_exit_code() {
	cmd().args(["info", &fixture("encrypted_v3.sfdl"), "-p", "wrong"]).assert().failure().code(4);
}

/// CLI-001 | A1: Encrypted without password → exit code 3.
#[test]
fn info_password_required_exit_code() {
	cmd().args(["info", &fixture("encrypted_v3.sfdl")]).assert().failure().code(3);
}

/// CLI-001 | BR-CLI-001-001: --json produces valid JSON.
#[test]
fn info_json_output() {
	let output = cmd().args(["info", &fixture("unencrypted_v3.sfdl"), "--json"]).assert().success().get_output().stdout.clone();

	let json: serde_json::Value = serde_json::from_slice(&output).expect("stdout should be valid JSON");
	assert_eq!(json["description"], "Test.Release.2026.1080p");
	assert_eq!(json["host"], "ftp.example.com");
	assert_eq!(json["port"], 21);
	assert_eq!(json["protocol"], "FTP");
	assert_eq!(json["encrypted"], false);
	assert_eq!(json["packages"], 1);
	assert_eq!(json["total_files"], 2);
}

/// CLI-001 | Main Success: --json with encrypted container.
#[test]
fn info_json_encrypted() {
	let output = cmd()
		.args(["info", &fixture("encrypted_v3.sfdl"), "-p", "test", "--json"])
		.assert()
		.success()
		.get_output()
		.stdout
		.clone();

	let json: serde_json::Value = serde_json::from_slice(&output).expect("stdout should be valid JSON");
	assert_eq!(json["encrypted"], true);
	assert_eq!(json["host"], "ftp.example.com");
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

/// CLI-002 | BR-CLI-002-002: Summary shows excluded count.
#[test]
fn list_shows_excluded_count() {
	cmd().args(["list", &fixture("unencrypted_v3.sfdl")]).assert().success().stdout(predicate::str::contains("0 excluded"));
}

/// CLI-002 | A3: --exclude adds patterns, excluded files hidden by default.
#[test]
fn list_exclude_hides_files() {
	cmd()
		.args(["list", &fixture("unencrypted_v3.sfdl"), "--exclude", "*.part2.*"])
		.assert()
		.success()
		.stdout(predicate::str::contains("movie.part1.rar"))
		.stdout(predicate::str::contains("1 files"))
		.stdout(predicate::str::contains("1 excluded"));
}

/// CLI-002 | BR-CLI-002-001: --show-excluded marks excluded files.
#[test]
fn list_show_excluded() {
	cmd()
		.args(["list", &fixture("unencrypted_v3.sfdl"), "--exclude", "*.part2.*", "--show-excluded"])
		.assert()
		.success()
		.stdout(predicate::str::contains("movie.part2.rar"))
		.stdout(predicate::str::contains("[excluded]"));
}

/// CLI-002 | A3: --no-exclude disables all patterns.
#[test]
fn list_no_exclude() {
	// Even with --exclude, --no-exclude disables everything
	cmd()
		.args(["list", &fixture("unencrypted_v3.sfdl"), "--exclude", "*.rar", "--no-exclude"])
		.assert()
		.success()
		.stdout(predicate::str::contains("2 files"))
		.stdout(predicate::str::contains("0 excluded"));
}

/// CLI-002 | BR-CLI-001-001: --json produces valid JSON with summary.
#[test]
fn list_json_output() {
	let output = cmd().args(["list", &fixture("unencrypted_v3.sfdl"), "--json"]).assert().success().get_output().stdout.clone();

	let json: serde_json::Value = serde_json::from_slice(&output).expect("stdout should be valid JSON");
	assert_eq!(json["packages"][0]["name"], "Package1");
	assert_eq!(json["packages"][0]["files"][0]["filename"], "movie.part1.rar");
	assert_eq!(json["summary"]["total_files"], 2);
	assert_eq!(json["summary"]["excluded_files"], 0);
}

/// CLI-002 | --json with --exclude shows excluded flag.
#[test]
fn list_json_with_exclude() {
	let output = cmd()
		.args(["list", &fixture("unencrypted_v3.sfdl"), "--json", "--exclude", "*.part2.*"])
		.assert()
		.success()
		.get_output()
		.stdout
		.clone();

	let json: serde_json::Value = serde_json::from_slice(&output).expect("stdout should be valid JSON");
	assert_eq!(json["summary"]["total_files"], 2);
	assert_eq!(json["summary"]["selected_files"], 1);
	assert_eq!(json["summary"]["excluded_files"], 1);

	// Check excluded file has pattern
	let files = json["packages"][0]["files"].as_array().unwrap();
	let excluded_file = files.iter().find(|f| f["excluded"] == true).unwrap();
	assert!(excluded_file["exclude_pattern"].is_string());
}

/// CLI-002 | list --help shows all new flags.
#[test]
fn list_help_shows_flags() {
	cmd()
		.args(["list", "--help"])
		.assert()
		.success()
		.stdout(predicate::str::contains("--json"))
		.stdout(predicate::str::contains("--exclude"))
		.stdout(predicate::str::contains("--no-exclude"))
		.stdout(predicate::str::contains("--show-excluded"));
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

/// CLI-003 | A6: download --help shows all flags.
#[test]
fn download_help_shows_all_flags() {
	let output = cmd().args(["download", "--help"]).assert().success();
	output
		.stdout(predicate::str::contains("--threads"))
		.stdout(predicate::str::contains("--max-speed"))
		.stdout(predicate::str::contains("--retries"))
		.stdout(predicate::str::contains("--retry-delay"))
		.stdout(predicate::str::contains("--strict-disk-check"))
		.stdout(predicate::str::contains("--exclude"))
		.stdout(predicate::str::contains("--no-exclude"))
		.stdout(predicate::str::contains("--quiet"));
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
		r#"
download_directory = "/tmp/downloads"
max_threads = 5
max_speed_kbps = 0
max_retries = 1
retry_delay_seconds = 1
auto_extract = false
delete_archives_after_extract = false
strict_disk_check = false
exclusion_patterns = []
auto_passwords = []
speedreport_template = ""
"#,
	);

	cmd()
		.args(["config", "show"])
		.env("RSFDL_CONFIG", &path)
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
	let path = dir.path().join("nonexistent/settings.toml");

	cmd()
		.args(["config", "show"])
		.env("RSFDL_CONFIG", path.to_string_lossy().to_string())
		.assert()
		.success()
		.stdout(predicate::str::contains("max_threads"))
		.stdout(predicate::str::contains("3"));
}

// --- AT-41: config edit creates file if missing ---

#[test]
fn config_edit_creates_default_file() {
	let dir = tempfile::tempdir().unwrap();
	let path = dir.path().join("rsfdl/settings.toml");

	// Use `true` as editor — it exits immediately with success
	cmd()
		.args(["config", "edit"])
		.env("RSFDL_CONFIG", path.to_string_lossy().to_string())
		.env("EDITOR", "true")
		.assert()
		.success();

	// File should now exist with valid TOML defaults
	assert!(path.exists(), "Settings file should have been created");
	let content = std::fs::read_to_string(&path).unwrap();
	assert!(content.contains("max_threads = 3"));
}

// --- AT-42: Download override does not modify settings file ---

#[test]
fn download_override_does_not_modify_settings_file() {
	let toml = r#"
download_directory = "/tmp/original"
max_threads = 3
max_speed_kbps = 0
max_retries = 1
retry_delay_seconds = 1
auto_extract = false
delete_archives_after_extract = false
strict_disk_check = false
exclusion_patterns = []
auto_passwords = []
speedreport_template = ""
"#;
	let (_dir, path) = create_settings_file(toml);
	let content_before = std::fs::read_to_string(&path).unwrap();

	// Download will fail (no FTP server), but that's OK —
	// we only care that the settings file is unchanged after the attempt.
	let _ = cmd()
		.args(["download", &fixture("unencrypted_v3.sfdl"), "--threads", "7", "--dest", "/tmp/override_dest"])
		.env("RSFDL_CONFIG", &path)
		.assert();

	let content_after = std::fs::read_to_string(&path).unwrap();
	assert_eq!(content_before, content_after, "Settings file must not be modified by CLI overrides");
}

// --- Info/List/Download use auto_passwords from settings ---

#[test]
fn info_uses_auto_passwords_from_settings() {
	let toml = r#"
download_directory = "/tmp/downloads"
max_threads = 3
max_speed_kbps = 0
max_retries = 1
retry_delay_seconds = 1
auto_extract = false
delete_archives_after_extract = false
strict_disk_check = false
exclusion_patterns = []
auto_passwords = ["wrong1", "test", "wrong2"]
speedreport_template = ""
"#;
	let (_dir, path) = create_settings_file(toml);

	cmd()
		.args(["info", &fixture("encrypted_v3.sfdl")])
		.env("RSFDL_CONFIG", &path)
		.assert()
		.success()
		.stderr(predicate::str::contains("Auto-decrypted with password from list"))
		.stdout(predicate::str::contains("ftp.example.com"));
}

#[test]
fn list_uses_auto_passwords_from_settings() {
	let toml = r#"
download_directory = "/tmp/downloads"
max_threads = 3
max_speed_kbps = 0
max_retries = 1
retry_delay_seconds = 1
auto_extract = false
delete_archives_after_extract = false
strict_disk_check = false
exclusion_patterns = []
auto_passwords = ["wrong1", "test", "wrong2"]
speedreport_template = ""
"#;
	let (_dir, path) = create_settings_file(toml);

	cmd()
		.args(["list", &fixture("encrypted_v3.sfdl")])
		.env("RSFDL_CONFIG", &path)
		.assert()
		.success()
		.stderr(predicate::str::contains("Auto-decrypted with password from list"))
		.stdout(predicate::str::contains("movie.part1.rar"));
}

// --- Download uses auto_passwords from settings ---

#[test]
fn download_uses_auto_passwords_from_settings() {
	let toml = r#"
download_directory = "/tmp/downloads"
max_threads = 3
max_speed_kbps = 0
max_retries = 1
retry_delay_seconds = 1
auto_extract = false
delete_archives_after_extract = false
strict_disk_check = false
exclusion_patterns = []
auto_passwords = ["wrong1", "test", "wrong2"]
speedreport_template = ""
"#;
	let (_dir, path) = create_settings_file(toml);

	// Download will fail at FTP connection, but decryption should succeed
	// using "test" from auto_passwords — no -p flag needed
	cmd()
		.args(["download", &fixture("encrypted_v3.sfdl")])
		.env("RSFDL_CONFIG", &path)
		.assert()
		.stderr(predicate::str::contains("Auto-decrypted with password from list"));
}

// --- AT-43: config show with corrupt settings file shows defaults ---

#[test]
fn config_show_with_corrupt_file_shows_defaults() {
	let (_dir, path) = create_settings_file("not valid toml {{{");

	cmd()
		.args(["config", "show"])
		.env("RSFDL_CONFIG", &path)
		.assert()
		.success()
		.stdout(predicate::str::contains("max_threads"))
		.stdout(predicate::str::contains("3"));
}

/// CLI-005 | config path: prints the default settings path.
#[test]
fn config_path_prints_path() {
	cmd()
		.args(["config", "path"])
		.assert()
		.success()
		.stdout(predicate::str::contains("rsfdl"))
		.stdout(predicate::str::contains("settings.toml"));
}

/// CLI-005 | config --help shows all subcommands.
#[test]
fn config_help_shows_subcommands() {
	cmd()
		.args(["config", "--help"])
		.assert()
		.success()
		.stdout(predicate::str::contains("show"))
		.stdout(predicate::str::contains("edit"))
		.stdout(predicate::str::contains("path"));
}
