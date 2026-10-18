use std::path::Path;
use std::process::{Command, Output};
use tempfile::{tempdir, TempDir};

#[allow(dead_code)]
pub struct TestOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

pub fn run_command(args: &[&str]) -> TestOutput {
    let output: Output = Command::new(env!("CARGO_BIN_EXE_mirror"))
        .args(args)
        .output()
        .expect("Failed to run command");

    TestOutput {
        success: output.status.success(),
        stdout: String::from_utf8(output.stdout).unwrap(),
        stderr: String::from_utf8(output.stderr).unwrap()
    }
}

pub fn create_temps<F : FnOnce(&Path, &Path)>(setup: F) -> (TempDir, TempDir) {
    let (source, destination) = (tempdir().unwrap(), tempdir().unwrap());
    setup(source.path(), destination.path());
    (source, destination)
}