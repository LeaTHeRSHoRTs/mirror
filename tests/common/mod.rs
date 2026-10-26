use std::path::Path;
use std::process::Command;
use tempfile::{tempdir, TempDir};

#[allow(dead_code)]
pub struct TestOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

pub fn run_command(args: &[&str]) -> TestOutput {
    run_command_in(args, TempDir::new().unwrap().path())
}
pub fn run_command_in(args: &[&str], working_dir: &Path) -> TestOutput {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mirror"));
    command.args(args);
    command.current_dir(working_dir);

    let output = command.output().expect("Failed to run command");
    
    TestOutput {
        success: output.status.success(),
        stdout: String::from_utf8(output.stdout).unwrap(),
        stderr: String::from_utf8(output.stderr).unwrap(),
    }
}

pub fn test_command_success(output: &TestOutput) {
    assert!(output.success,
        "Command failed!\n\n\
        --- stderr ---\n{}\n\n\
        --- stdout ---\n{}",
        output.stderr,
        output.stdout
    );
}

pub fn create_temps<F : FnOnce(&Path, &Path)>(setup: F) -> (TempDir, TempDir) {
    let (source, destination) = (tempdir().unwrap(), tempdir().unwrap());
    setup(source.path(), destination.path());
    (source, destination)
}

#[macro_export]
macro_rules! file_tree {
    ($($path:literal => $contents:literal),*$(,)?) => {{
        let (source, dest) = create_temps(|source, _| {
            $(
                let path = source.join($path);

                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent).unwrap();
                }

                std::fs::write(path, $contents).unwrap();
            )*
        });

        (source, dest)
    }};

    (
        source: { $($source_path:literal => $source_contents:literal),* $(,)? },
        destination: { $($dest_path:literal => $dest_contents:literal),* $(,)? } $(,)?
    ) => {{
        let (source, destination) = create_temps(|source, destination| {
            $(
                let path = source.join($source_path);

                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent).unwrap();
                }

                std::fs::write(path, $source_contents).unwrap();
            )*

            $(
                let path = destination.join($dest_path);

                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent).unwrap();
                }

                std::fs::write(path, $dest_contents).unwrap();
            )*
        });

        (source, destination)
    }};
}