use crate::common::create_temps;

mod common;

#[test]
fn compare_finds_missing_files() {
    let (source, destination) = file_tree! {
        "a.txt" => "A",
        "b.txt" => "B",
    };

    std::fs::write(destination.path().join("a.txt"), "A").unwrap();

    let output = common::run_command(
        source.path(),
        Some(destination.path()),
        &["compare"]
    );

    common::assert_command_success(&output);

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

    let output = common::run_command(
        source.path(),
        Some(destination.path()),
        &["compare"]
    );

    common::assert_command_success(&output);
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

    let output = common::run_command(
        source.path(),
        Some(destination.path()),
        &["compare"]
    );

    common::assert_command_success(&output);
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

    let output = common::run_command(
        source.path(),
        Some(destination.path()),
        &["compare"]
    );

    common::assert_command_success(&output);
    assert!(
        output.stdout.contains("extra.txt"),
        "Unexpected difference: {:?}",
        output.stdout
    );
}

#[test]
fn compare_recurses_by_default() {
    let (source, destination) = file_tree! {
        "subfolder/nested/deep.txt" => "Deep",
    };

    let output = common::run_command(
        source.path(),
        Some(destination.path()),
        &["compare"]
    );

    common::assert_command_success(&output);
    assert!(
        output.stdout.contains("subfolder/*"),
        "Unexpected output: {:?}",
        output.stdout
    );
}

#[test]
fn compare_handles_empty_source_directory() {
    let (source, destination) = file_tree!();

    let output = common::run_command(
        source.path(),
        Some(destination.path()),
        &["compare"]
    );

    common::assert_command_success(&output);
    assert!(
        output.stdout.contains("Source and destination directory has nothing to compare"),
        "Command did not tell the user that the directories already match. Actual: '{}'",
        output.stdout
    )
}

#[test]
fn compare_can_detect_file_states() {
    let (source, destination) = file_tree! {
        source: {
            "a.txt" => "A",
            "b.txt" => "B",
            "c.txt" => "X",
            "d.txt" => "Y",
            "e.txt" => "E",
        },
        destination: {
            "a.txt" => "X",
            "b.txt" => "Y",
            "c.txt" => "C",
            "d.txt" => "D",
            "f.txt" => "F",
        }
    };

    let output = common::run_command(
        source.path(),
        Some(destination.path()),
        &["compare"]
    );

    common::assert_command_success(&output);

    // Check if the indicators exist
    assert!(output.stdout.contains("[+]"), "Output did not detect any added files.");
    assert!(output.stdout.contains("[-]"), "Output did not detect any removed files.");
    assert!(output.stdout.contains("[~]"), "Output did not detect any modified files.");

    // Check if the added and removed files are logged
    assert!(output.stdout.contains("[+] f.txt"), "Output did not mark f.txt as added.");
    assert!(output.stdout.contains("[-] e.txt"), "Output did not mark e.txt as removed.");

    for file in ["a.txt", "b.txt", "c.txt", "d.txt"] {
        assert!(
            output.stdout.contains(&format!("[~] {file}")),
            "Output did not mark {file} as modified."
        );
    }
}

#[test]
fn compare_can_collapse_directories_correctly() {
    let (source, destination) = file_tree! {
        source: {
            "main/a.txt" => "A",
            "main/b.txt" => "B",
            "main/c.txt" => "C",
            "tests/a.txt" => "A",
            "tests/b.txt" => "B",
            "tests/c.txt" => "C"
        },
        destination: {
            "main/a.txt" => "X",
            "main/b.txt" => "Y",
            "main/c.txt" => "Z",
            "other/a.txt" => "A",
            "other/b.txt" => "B",
            "other/c.txt" => "C"
        }
    };

    let output = common::run_command(
        source.path(),
        Some(destination.path()),
        &["compare"]
    );

    assert!(output.stdout.contains("[+]"), "Output did not find added files");
    assert!(output.stdout.contains("[-]"), "Output did not find removed files");
    assert!(output.stdout.contains("[~]"), "Output did not find modified files");

    assert!(output.stdout.contains("[+] other/*"), "Output did not collapse added directory");
    assert!(output.stdout.contains("[-] tests/*"), "Output did not collapse removed directory");
    assert!(output.stdout.contains("[~] main/*"), "Output did not collapse modified directory");
}

// top-level
#[test]
fn compare_does_not_find_nested_missing_files_using_top_level_flag() {
    let (source, destination) = file_tree! {
        "root.txt" => "Root",
        "subfolder/nested/deep.txt" => "Deep",
    };

    let output = common::run_command(
        source.path(),
        Some(destination.path()),
        &["compare", "top-level"]
    );

    common::assert_command_success(&output);

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

