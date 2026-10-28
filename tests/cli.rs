mod common;

use std::path::Path;
use common::create_temps;

#[test]
fn cli_rejects_missing_source() {
    let output = common::run_command(Path::new(""), None, &[]);
    assert!(!output.success, "Expected command to fail");
}

#[test]
fn cli_rejects_nonexistent_source() {
    let (_source, destination) = file_tree!();

    let output = common::run_command(
        Path::new("/this/path/should/not/exist"),
        Some(destination.path()),
        &[]
    );

    assert!(!output.success, "Expected command to fail");
}

#[test]
fn cli_help_succeeds() {
    let output = common::run_command(Path::new(""), None, &["help"]);

    common::assert_command_success(&output);
    assert!(output.stdout.contains("--top-level"));
    assert!(output.stdout.contains("--compare"));
}

#[test]
fn cli_rejects_unknown_flag() {
    let (source, destination) = file_tree!();

    let output = common::run_command(
        source.path(),
        Some(destination.path()),
        &["not-a-real-flag"]
    );

    assert!(!output.success, "Expected command to fail");
}

#[test]
fn cli_rejects_file_path_as_dir_path() {
    let (source, _) = file_tree! { "a.txt" => "File" };
    let output = common::run_command(
        &source.path().join("a.txt"),
        None,
        &[]
    );
    assert!(!output.success, "Expected command to fail");
}