mod common;
use crate::common::create_temps;
use std::fs;

#[test]
fn copy_copies_file_contents() {
    let (source, destination) = file_tree! { "hello.txt" => "Hello, world!" };

    let output = common::run_command(&[
        source.path().to_str().unwrap(),
        destination.path().to_str().unwrap(),
    ]);

    common::test_command_success(&output);

    let copied = fs::read_to_string(destination.path().join("hello.txt"))
        .expect("Copied file should exist");

    assert_eq!(copied, "Hello, world!");
}

#[test]
fn copy_copies_implicitly_to_current_dir() {
    let (source, destination) = file_tree! {
        "hello.txt" => "Hello, world!",
        "goodbye.txt" => "Goodbye, world!",
    };

    let output = common::run_command_in(
        &[source.path().to_str().unwrap()],
        destination.path()
    );

    common::test_command_success(&output);

    for (name, expected) in [
        ("hello.txt", "Hello, world!"),
        ("goodbye.txt", "Goodbye, world!"),
    ] {
        let copied = fs::read_to_string(destination.path().join(name))
            .unwrap_or_else(|_| panic!("{name} should have been copied"));

        assert_eq!(copied, expected);
    }
}

#[test]
fn copy_copies_multiple_files() {
    let (source, destination) = file_tree! {
        "a.txt" => "A",
        "b.txt" => "B",
        "c.txt" => "C",
    };

    let output = common::run_command(&[
        source.path().to_str().unwrap(),
        destination.path().to_str().unwrap(),
    ]);

    common::test_command_success(&output);

    for (name, expected) in [("a.txt", "A"), ("b.txt", "B"), ("c.txt", "C")] {
        let copied = fs::read_to_string(destination.path().join(name))
            .unwrap_or_else(|_| panic!("{name} should have been copied"));

        assert_eq!(copied, expected, "Incorrect contents for {name}");
    }
}

#[test]
fn copy_does_not_modify_source_files() {
    let (source, destination) = file_tree! {
        "hello.txt" => "Original",
    };

    let output = common::run_command(&[
        source.path().to_str().unwrap(),
        destination.path().to_str().unwrap(),
    ]);

    common::test_command_success(&output);

    let original = fs::read_to_string(source.path().join("hello.txt")).unwrap();
    assert_eq!(original, "Original");
}

#[test]
fn copy_overwrites_existing_destination_file() {
    let (source, destination) = file_tree! {
        source: { "hello.txt" => "New contents" },
        destination: { "hello.txt" => "Old contents" }
    };

    let output = common::run_command(&[
        source.path().to_str().unwrap(),
        destination.path().to_str().unwrap(),
    ]);

    common::test_command_success(&output);

    let copied = fs::read_to_string(destination.path().join("hello.txt")).unwrap();
    assert_eq!(copied, "New contents");
}

#[test]
fn copy_recursively_copies() {
    let (source, destination) = file_tree! {
        "root.txt" => "Root",
        "subfolder/child.txt" => "Child",
        "subfolder/nested/deep.txt" => "Deep",
    };

    let output = common::run_command(&[
        source.path().to_str().unwrap(),
        destination.path().to_str().unwrap(),
    ]);

    common::test_command_success(&output);

    for (path, expected) in [
        ("root.txt", "Root"),
        ("subfolder/child.txt", "Child"),
        ("subfolder/nested/deep.txt", "Deep"),
    ] {
        let copied = fs::read_to_string(destination.path().join(path))
            .unwrap_or_else(|_| panic!("{path} should have been copied"));

        assert_eq!(copied, expected, "Incorrect contents for {path}");
    }
}

#[test]
fn copy_does_not_copy_nested_files_with_top_level_flag() {
    let (source, destination) = file_tree! {
        "root.txt" => "Root",
        "subfolder/child.txt" => "Child",
    };

    let output = common::run_command(&[
        source.path().to_str().unwrap(),
        destination.path().to_str().unwrap(),
        "--top-level"
    ]);

    common::test_command_success(&output);
    assert!(destination.path().join("root.txt").is_file());
    assert!(
        !destination.path().join("subfolder/child.txt").exists(),
        "Nested file should not have been copied with top-level flag"
    );
}

#[test]
fn copy_creates_nested_destination_directories() {
    let (source, destination) = file_tree! {
        "subfolder/nested/deep.txt" => "Deep",
    };

    let output = common::run_command(&[
        source.path().to_str().unwrap(),
        destination.path().to_str().unwrap()
    ]);

    common::test_command_success(&output);
    let copied = fs
        ::read_to_string(destination.path().join("subfolder/nested/deep.txt"))
        .expect("Nested file should have been copied");
    assert_eq!(copied, "Deep");
}

#[test]
fn copy_handles_empty_source_directory() {
    let (source, destination) = file_tree! {};

    let output = common::run_command(&[
        source.path().to_str().unwrap(),
        destination.path().to_str().unwrap(),
    ]);

    common::test_command_success(&output);
    assert!(
        output.stdout.contains("Source directory has nothing to copy over"),
        "Command did not tell the user that nothing could be copied over. Actual: '{}'",
        output.stdout
    )
}

#[test]
fn copy_preserves_empty_source_file() {
    let (source, destination) = file_tree! {
        "empty.txt" => "",
    };

    let output = common::run_command(&[
        source.path().to_str().unwrap(),
        destination.path().to_str().unwrap(),
    ]);

    common::test_command_success(&output);
    let copied = fs
        ::read_to_string(destination.path().join("empty.txt"))
        .expect("Empty file should have been copied");
    
    assert_eq!(copied, "");
}