use std::{fs, io::Result as IOResult, path};
use std::path::{Path, PathBuf};
use console::style;
use indicatif::{ProgressBar};

#[derive(Default)]
struct GatheredMetadata {
    hidden: bool,
    system: bool,
    is_file: bool,
    is_dir: bool
}

pub fn collect_files(
    origin: &Path,
    dir: &Path,
    hidden: bool,
    system: bool,
    spinner: &ProgressBar
) -> IOResult<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = Vec::new();
    spinner.set_message(format!("Scanning: {}", dir.display()));

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let full_path = entry.path();
        let relative_path = full_path.strip_prefix(origin).unwrap().to_path_buf();
        let attrs = gather_metadata(&full_path);

        if (!system && attrs.system) ||
            (!hidden && attrs.hidden) {
            continue;
        }

        if attrs.is_dir {
            files.extend(collect_files(origin, &full_path, hidden, system, spinner)?);
        } else if attrs.is_file {
            files.push(relative_path);
        }

        spinner.inc(1);
    }

    Ok(files)
}

pub fn collect_top(dir: &Path, hidden: bool, system: bool, spinner: &ProgressBar) -> IOResult<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = Vec::new();
    spinner.set_message(format!("Scanning: {}", dir.display()));

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        let attrs = gather_metadata(&path);

        if (!system && attrs.system) ||
            (!hidden && attrs.hidden) {
            continue;
        }

        if attrs.is_file {
            files.push(path
                .strip_prefix(dir)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
                .to_path_buf());
        }

        spinner.inc(1);
    }

    Ok(files)
}

pub fn get_absolute_path(path: &Path) -> String {
    path::absolute(path).map_or_else(
        |_| format!("./{}", path.display()),
        |val| val.display().to_string(),
    )
}

// Utility that unwraps the promise returned by the attribute helpers elegantly
fn gather_metadata(path: &Path) -> GatheredMetadata {
    match get_attributes(path) {
        Ok(attrs) => attrs,
        Err(error) => {
            println!(
                "{}",
                style(format!(
                    "Failed to identify attributes of {}: {}. Skipping entry",
                    path.display(), error
                )).yellow()
            );

            GatheredMetadata {
                system: true,
                hidden: true,
                is_file: false,
                is_dir: false
            }
        }
    }
}

// Windows helper
#[cfg(target_os = "windows")]
fn get_attributes(file: &Path) -> IOResult<GatheredMetadata> {
    use std::os::windows::fs::MetadataExt;
    let metadata = fs::metadata(&file)?;
    let attrs = metadata.file_attributes();

    // Hidden code is 0x2, System code is 0x4
    Ok(GatheredMetadata {
        hidden: attrs & 0x2 != 0,
        system: attrs & 0x4 != 0,
        is_dir: metadata.is_dir(),
        is_file: metadata.is_file()
    })
}

// Unix helpers (Both linux and macos)
#[cfg(unix)]
fn is_dot_hidden(file: &Path) -> bool {
    file.file_name()
        .is_some_and(|name| name.to_string_lossy().starts_with('.'))
}

// Linux helper
#[cfg(target_os = "linux")]
fn get_attributes(file: &Path) -> IOResult<GatheredMetadata> {
    let metadata = file.metadata()?;

    Ok(GatheredMetadata {
        hidden: is_dot_hidden(&file),
        system: false,
        is_dir: metadata.is_dir(),
        is_file: metadata.is_file()
    })
}

// MacOS helper
#[cfg(target_os = "macos")]
fn get_attributes(file: &Path) -> IOResult<GatheredMetadata> {
    use std::os::macos::fs::MetadataExt;
    const UF_HIDDEN: u32 = 0x8000;
    let metadata = fs::metadata(file)?;

    Ok(GatheredMetadata {
        hidden: is_dot_hidden(&file) || metadata.st_flags() & UF_HIDDEN != 0,
        system: false,
        is_dir: metadata.is_dir(),
        is_file: metadata.is_file()
    })
}


// Tests
#[cfg(test)]
mod tests {
    // Those tests use commands
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    use std::process::Command;
    use super::*;

    // Unix specific
    #[test]
    #[cfg(unix)]
    fn unix_hidden_dot_files_are_hidden_by_helper() {
        assert!(is_dot_hidden(Path::new(".config")));
        assert!(is_dot_hidden(Path::new("folder/.config")));
    }

    #[test]
    #[cfg(unix)]
    fn unix_hidden_ordinary_files_are_not_hidden_by_helper() {
        assert!(!is_dot_hidden(Path::new("file.txt")));
        assert!(!is_dot_hidden(Path::new("folder/file.txt")));
    }

    #[test]
    #[cfg(unix)]
    fn unix_hidden_dot_files_are_hidden() {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let file = dir.path().join(".config");
        let nested_file = dir.path().join("folder/.config");

        fs::create_dir(dir.path().join("folder")).expect("Failed to create nested folder");
        fs::write(&file, "test").expect("Failed to make file");
        fs::write(&nested_file, "test second").expect("Failed to make nested file");

        assert!(
            get_attributes(&file).expect("Failed to find out if the file was hidden").hidden,
            "File {} is not hidden", file.display()
        );
        assert!(
            get_attributes(&nested_file).expect("Failed to find out if the file was hidden").hidden,
            "File {} is not hidden", nested_file.display()
        );
    }
    
    #[test]
    #[cfg(unix)]
    fn unix_system_always_returns_false() {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let file = dir.path().join("file.txt");

        fs::write(&file, "test").expect("Failed to create file");

        assert!(
            !get_attributes(&file).expect("Failed to check system status").system,
            "Unix files should not be marked as system files by this helper"
        );
    }

    #[test]
    #[cfg(unix)]
    fn unix_hidden_ordinary_files_are_not_hidden() {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let file = dir.path().join("file.txt");
        let nested_file = dir.path().join("folder/file.txt");

        fs::create_dir(dir.path().join("folder")).expect("Failed to create nested folder");
        fs::write(&file, "test").expect("Failed to make file");
        fs::write(&nested_file, "test second").expect("Failed to make nested file");

        assert!(
            !get_attributes(&file).expect("Failed to find out if the file was hidden").hidden,
            "File {} is hidden", file.display()
        );
        assert!(
            !get_attributes(&nested_file).expect("Failed to find out if the file was hidden").hidden,
            "File {} is hidden", nested_file.display()
        );
    }

    // Mac specific
    #[test]
    #[cfg(target_os = "macos")]
    fn macos_hidden_flagged_files_are_hidden() {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let file = dir.path().join("file.txt");
        let nested_file = dir.path().join("folder/file.txt");

        fs::create_dir(dir.path().join("folder")).expect("Failed to create nested folder");
        fs::write(&file, "test").expect("Failed to make file");
        fs::write(&nested_file, "test second").expect("Failed to make nested file");

        let status = std::process::Command::new("chflags")
            .arg("hidden")
            .arg(&file)
            .status()
            .expect("Failed to run command 'chflags' on macos");

        let nested_status = std::process::Command::new("chflags")
            .arg("hidden")
            .arg(&nested_file)
            .status()
            .expect("Failed to run command 'chflags' on macos");

        assert!(status.success(), "Failed to run command 'chflags' on macos for {}", file.display());
        assert!(nested_status.success(), "Failed to run command 'chflags' on macos for {}", nested_file.display());
        assert!(
            get_attributes(&file).expect("Failed to find out if the file was hidden").hidden,
            "File {} is not hidden", file.display()
        );
        assert!(
            get_attributes(&nested_file).expect("Failed to find out if the file was hidden").hidden,
            "File {} is not hidden", nested_file.display()
        );
    }

    // Windows specific
    #[test]
    #[cfg(target_os = "windows")]
    fn windows_hidden_flagged_files_are_hidden() {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let file = dir.path().join("file.txt");
        let nested_file = dir.path().join("folder/file.txt");

        fs::create_dir(dir.path().join("folder")).expect("Failed to create nested folder");
        fs::write(&file, "test").expect("Failed to create file");
        fs::write(&nested_file, "test second").expect("Failed to create nested file");

        for path in [&file, &nested_file] {
            let status = Command::new("attrib")
                .arg("+H")
                .arg(path)
                .status()
                .expect("Failed to run command 'attrib' on windows");

            assert!(status.success(), "Failed to hide {}", path.display());
            assert!(
                get_attributes(path).expect("Failed to check hidden attribute").hidden,
                "File {} is not hidden", path.display()
            );
        }
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn windows_hidden_normal_files_are_not_hidden() {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let file = dir.path().join("file.txt");
        let nested_file = dir.path().join("folder/file.txt");

        fs::create_dir(dir.path().join("folder")).expect("Failed to create nested folder");
        fs::write(&file, "test").expect("Failed to create file");
        fs::write(&nested_file, "test second").expect("Failed to create nested file");

        assert!(
            !get_attributes(&file).expect("Failed to check hidden attribute").hidden,
            "File {} is hidden", file.display()
        );
        assert!(
            !get_attributes(&nested_file).expect("Failed to check hidden attribute").hidden,
            "File {} is hidden", nested_file.display()
        );
    }
}