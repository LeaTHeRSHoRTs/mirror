use crate::common::create_temps;

mod common;

#[test]
fn compare_finds_missing_files() {
    let (source, destination) = file_tree! {
        "a.txt" => "A",
        "b.txt" => "B",
    };

    std::fs::write(destination.path().join("a.txt"), "A").unwrap();

    let output = common::run_command(&[
        source.path().to_str().unwrap(),
        destination.path().to_str().unwrap(),
        "--compare",
    ]);

    common::test_command_success(&output);

    assert!(
        !output.stdout.contains("a.txt"),
        "Did not expect to find file a.txt in stdout. Instead got: {:?}",
        output.stdout
    );
    assert!(
        output.stdout.contains("b.txt"),
        "Did not find file b.txt in stdout. Instead got: {:?}",
        output.stdout
    );
}

#[test]
fn compare_returns_empty_when_directories_match() {
    let (source, destination) = file_tree! { 
        source: { "a.txt" => "A" },
        destination: { "a.txt" => "A" }
    };

    let output = common::run_command(&[
        source.path().to_str().unwrap(),
        destination.path().to_str().unwrap(),
        "--compare",
    ]);

    common::test_command_success(&output);
    assert!(
        !output.stdout.contains("a.txt"),
        "Expected no differences, but stdout was: {:?}",
        output.stdout
    );
}

#[test]
fn compare_finds_files_when_destination_is_empty() {
    let (source, destination) = file_tree! {
        "a.txt" => "A",
        "b.txt" => "B",
    };

    let output = common::run_command(&[
        source.path().to_str().unwrap(),
        destination.path().to_str().unwrap(),
        "--compare",
    ]);

    common::test_command_success(&output);
    assert!(output.stdout.contains("a.txt"));
    assert!(output.stdout.contains("b.txt"));
}

#[test]
fn compare_includes_extra_destination_files() {
    let (source, destination) = file_tree! {
        source: {
            "a.txt" => "A",
        },
        destination: {
            "a.txt" => "A",
            "extra.txt" => "Extra",
        },
    };

    let output = common::run_command(&[
        source.path().to_str().unwrap(),
        destination.path().to_str().unwrap(),
        "--compare",
    ]);

    common::test_command_success(&output);
    assert!(
        output.stdout.contains("extra.txt"),
        "Unexpected difference: {:?}",
        output.stdout
    );
}

#[test]
fn compare_does_not_find_nested_missing_files_using_top_level_flag() {
    let (source, destination) = file_tree! {
        "root.txt" => "Root",
        "subfolder/nested/deep.txt" => "Deep",
    };

    let output = common::run_command(&[
        source.path().to_str().unwrap(),
        destination.path().to_str().unwrap(),
        "--compare",
        "--top-level"
    ]);

    common::test_command_success(&output);

    assert!(
        !output.stdout.contains("deep.txt"),
        "Nested files should not be reported in top-level mode"
    );
    assert!(
        output.stdout.contains("root.txt"),
        "Missing top-level files should still be reported. Actual: {}",
        output.stdout
    );
}

#[test]
fn compare_recurses_by_default() {
    let (source, destination) = file_tree! {
        "subfolder/nested/deep.txt" => "Deep",
    };

    let output = common::run_command(&[
        source.path().to_str().unwrap(),
        destination.path().to_str().unwrap(),
        "--compare",
    ]);

    common::test_command_success(&output);
    assert!(
        output.stdout.contains("deep.txt"),
        "Unexpected output: {:?}",
        output.stdout
    );
}

#[test]
fn compare_handles_empty_source_directory() {
    let (source, destination) = file_tree! {};

    let output = common::run_command(&[
        source.path().to_str().unwrap(),
        destination.path().to_str().unwrap(),
        "--compare",
    ]);

    common::test_command_success(&output);
    assert!(
        output.stdout.contains("Source and destination directory has nothing to compare"),
        "Command did not tell the user that the directories already match. Actual: '{}'",
        output.stdout
    )
}