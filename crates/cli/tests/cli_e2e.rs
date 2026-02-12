use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("rsfdl-cli").unwrap()
}

fn fixture(name: &str) -> String {
    let base = env!("CARGO_MANIFEST_DIR");
    format!("{}/../../crates/core/tests/fixtures/{}", base, name)
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
    cmd()
        .args(["info", "/tmp/nonexistent_file_12345.sfdl"])
        .assert()
        .failure();
}

// --- Invalid SFDL ---

#[test]
fn info_invalid_file_fails() {
    cmd()
        .args(["info", &fixture("invalid.sfdl")])
        .assert()
        .failure();
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
    cmd()
        .args(["download", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--exclude"));
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
