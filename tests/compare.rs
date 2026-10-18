
mod common;
use std::fs;
use tempfile::tempdir;

#[test]
fn compare_finds_missing_files() {
    let source = tempdir().unwrap();
    let destination = tempdir().unwrap();

    fs::write(source.path().join("a.txt"), "A").unwrap();
    fs::write(source.path().join("b.txt"), "B").unwrap();
    fs::write(destination.path().join("a.txt"), "A").unwrap();

    let output = common::run_command(&[
        source.path().to_str().unwrap(),
        destination.path().to_str().unwrap(),
        "--compare"
    ]);

    assert!(output.success);
    assert!(!output.stdout.contains("a.txt"), "Did not find file a.txt in stdout. Instead got: {:?}", output.stdout);
    assert!(output.stdout.contains("b.txt"), "Did not find file b.txt in stdout. Instead got: {:?}", output.stdout);
}

#[test]
fn compare_returns_empty_when_directories_match() {
    let source = tempdir().unwrap();
    let destination = tempdir().unwrap();

    fs::write(source.path().join("a.txt"), "A").unwrap();
    fs::write(destination.path().join("a.txt"), "A").unwrap();

    let output = common::run_command(&[
        source.path().to_str().unwrap(),
        destination.path().to_str().unwrap(),
        "--compare"
    ]);

    assert!(output.success);
    assert!(!output.stdout.contains("a.txt"));
}
