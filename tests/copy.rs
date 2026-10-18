mod common;
use std::fs;
use tempfile::tempdir;

#[test]
fn copy_copies_file_contents() {
    let source = tempdir().unwrap();
    let destination = tempdir().unwrap();

    fs::write(source.path().join("hello.txt"), "Hello, world!").unwrap();

    let output = common::run_command(&[
        source.path().to_str().unwrap(),
        destination.path().to_str().unwrap(),
    ]);

    assert!(
        output.success,
        "Command failed:{}\n",
        output.stderr
    );

    let copied = fs::read_to_string(destination.path().join("hello.txt"))
        .expect("Copied file should exist");

    assert_eq!(copied, "Hello, world!");
}

#[test]
fn copy_copies_multiple_files() {
    let source = tempdir().unwrap();
    let destination = tempdir().unwrap();

    fs::write(source.path().join("a.txt"), "A").unwrap();
    fs::write(source.path().join("b.txt"), "B").unwrap();
    fs::write(source.path().join("c.txt"), "C").unwrap();

    let output = common::run_command(&[
        source.path().to_str().unwrap(),
        destination.path().to_str().unwrap(),
    ]);

    assert!(
        output.success,
        "Command failed:\n{}",
        output.stderr
    );

    for (name, expected) in [("a.txt", "A"), ("b.txt", "B"), ("c.txt", "C")] {
        let copied = fs::read_to_string(destination.path().join(name))
            .unwrap_or_else(|_| panic!("{name} should have been copied"));

        assert_eq!(copied, expected, "Incorrect contents for {name}");
    }
}

#[test]
fn copy_does_not_modify_source_files() {
    let source = tempdir().unwrap();
    let destination = tempdir().unwrap();

    fs::write(source.path().join("hello.txt"), "Original").unwrap();

    let output = common::run_command(&[
        source.path().to_str().unwrap(),
        destination.path().to_str().unwrap(),
    ]);

    assert!(
        output.success,
        "Command failed:\n{}",
        output.stderr
    );

    let original = fs::read_to_string(source.path().join("hello.txt")).unwrap();
    assert_eq!(original, "Original");
}

#[test]
fn copy_overwrites_existing_destination_file() {
    let source = tempdir().unwrap();
    let destination = tempdir().unwrap();

    fs::write(source.path().join("hello.txt"), "New contents").unwrap();
    fs::write(destination.path().join("hello.txt"), "Old contents").unwrap();

    let output = common::run_command(&[
        source.path().to_str().unwrap(),
        destination.path().to_str().unwrap(),
    ]);

    assert!(
        output.success,
        "Command failed:\n{}",
        output.stderr
    );

    let copied = fs::read_to_string(destination.path().join("hello.txt")).unwrap();
    assert_eq!(copied, "New contents");
}

#[test]
fn copy_recursive_flag_recursively_copies() {
    let source = tempdir().unwrap();
    let destination = tempdir().unwrap();

    fs::create_dir_all(source.path().join("subfolder/nested")).unwrap();
    fs::write(source.path().join("root.txt"), "Root").unwrap();
    fs::write(source.path().join("subfolder/child.txt"), "Child").unwrap();
    fs::write(source.path().join("subfolder/nested/deep.txt"), "Deep").unwrap();

    let output = common::run_command(&[
        source.path().to_str().unwrap(),
        destination.path().to_str().unwrap(),
        "--recursive",
    ]);

    assert!(output.success, "Command failed:\n{}", output.stderr);

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