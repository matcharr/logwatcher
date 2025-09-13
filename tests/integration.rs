use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::NamedTempFile;
use std::io::Write;

#[test]
fn test_help_output() {
    let mut cmd = Command::cargo_bin("logwatcher").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("monitoring log files"));
}

#[test]
fn test_version_output() {
    let mut cmd = Command::cargo_bin("logwatcher").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("logwatcher 0.1.0"));
}

#[test]
fn test_no_files_error() {
    let mut cmd = Command::cargo_bin("logwatcher").unwrap();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_dry_run_with_existing_file() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "This is an ERROR message").unwrap();
    writeln!(temp_file, "This is a normal message").unwrap();
    writeln!(temp_file, "Another ERROR message").unwrap();
    temp_file.flush().unwrap();

    let mut cmd = Command::cargo_bin("logwatcher").unwrap();
    cmd.args(&[
        "--file",
        temp_file.path().to_str().unwrap(),
        "--dry-run",
        "--pattern",
        "ERROR",
        "--no-color",
    ]);
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("ERROR message").count(2))
        .stdout(predicate::str::contains("[DRY-RUN]"));
}

#[test]
fn test_quiet_mode() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "This is an ERROR message").unwrap();
    writeln!(temp_file, "This is a normal message").unwrap();
    writeln!(temp_file, "Another ERROR message").unwrap();
    temp_file.flush().unwrap();

    let mut cmd = Command::cargo_bin("logwatcher").unwrap();
    cmd.args(&[
        "--file",
        temp_file.path().to_str().unwrap(),
        "--dry-run",
        "--pattern",
        "ERROR",
        "--quiet",
        "--no-color",
    ]);
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("normal message").not())
        .stdout(predicate::str::contains("ERROR message").count(2));
}

#[test]
fn test_case_insensitive_matching() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "This is an error message").unwrap();
    writeln!(temp_file, "This is an ERROR message").unwrap();
    temp_file.flush().unwrap();

    let mut cmd = Command::cargo_bin("logwatcher").unwrap();
    cmd.args(&[
        "--file",
        temp_file.path().to_str().unwrap(),
        "--dry-run",
        "--pattern",
        "ERROR",
        "--case-insensitive",
        "--no-color",
    ]);
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("error message"))
        .stdout(predicate::str::contains("ERROR message"));
}

#[test]
fn test_regex_matching() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "Login successful for user_id=12345").unwrap();
    writeln!(temp_file, "Login successful for user_id=abc").unwrap();
    writeln!(temp_file, "Order placed by user_id=67890").unwrap();
    temp_file.flush().unwrap();

    let mut cmd = Command::cargo_bin("logwatcher").unwrap();
    cmd.args(&[
        "--file",
        temp_file.path().to_str().unwrap(),
        "--dry-run",
        "--pattern",
        r"user_id=\d+",
        "--regex",
        "--no-color",
    ]);
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("user_id=12345"))
        .stdout(predicate::str::contains("user_id=67890"))
        .stdout(predicate::str::contains("user_id=abc").not());
}

#[test]
fn test_multiple_files() {
    let mut temp_file1 = NamedTempFile::new().unwrap();
    let mut temp_file2 = NamedTempFile::new().unwrap();
    
    writeln!(temp_file1, "ERROR in file1").unwrap();
    writeln!(temp_file2, "ERROR in file2").unwrap();
    temp_file1.flush().unwrap();
    temp_file2.flush().unwrap();

    let mut cmd = Command::cargo_bin("logwatcher").unwrap();
    cmd.args(&[
        "--file",
        temp_file1.path().to_str().unwrap(),
        "--file",
        temp_file2.path().to_str().unwrap(),
        "--dry-run",
        "--pattern",
        "ERROR",
        "--no-color",
    ]);
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("[DRY-RUN]").count(2))
        .stdout(predicate::str::contains("ERROR in file1"))
        .stdout(predicate::str::contains("ERROR in file2"));
}

#[test]
fn test_invalid_regex() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "Test message").unwrap();
    temp_file.flush().unwrap();

    let mut cmd = Command::cargo_bin("logwatcher").unwrap();
    cmd.args(&[
        "--file",
        temp_file.path().to_str().unwrap(),
        "--dry-run",
        "--pattern",
        "[invalid",
        "--regex",
        "--no-color",
    ]);
    
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Invalid regex pattern"));
}
