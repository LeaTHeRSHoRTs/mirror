mod common;

#[test]
fn cli_rejects_missing_source() {
    let output = common::run_command(&[]);
    assert!(!output.success, "Expected command to fail");
}

#[test]
fn cli_rejects_nonexistent_source() {
    let (_source, destination) = common::create_temps(|_, _| {});

    let output = common::run_command(&[
        "/this/path/should/not/exist",
        destination.path().to_str().unwrap(),
    ]);

    assert!(!output.success, "Expected command to fail");
}

#[test]
fn cli_help_succeeds() {
    let output = common::run_command(&["--help"]);

    common::test_command_success(&output);
    assert!(output.stdout.contains("--recursive"));
    assert!(output.stdout.contains("--compare"));
}

#[test]
fn cli_rejects_unknown_flag() {
    let (source, destination) = common::create_temps(|_, _| {});

    let output = common::run_command(&[
        source.path().to_str().unwrap(),
        destination.path().to_str().unwrap(),
        "--not-a-real-flag",
    ]);

    assert!(!output.success, "Expected command to fail");
}

#[test]
fn cli_rejects_file_path_as_dir_path() {
    let (source, _) = file_tree! { "a.txt" => "File" };
    let output = common::run_command(&[source.path().join("a.txt").to_str().unwrap(),]);
    assert!(!output.success, "Expected command to fail");
}