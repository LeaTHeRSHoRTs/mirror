use std::{fs, io::Result as IOResult};
use std::path::{Path, PathBuf};

pub fn collect_files(origin: &Path, dir: &Path, hidden: bool, system: bool) -> IOResult<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let full_path = entry.path();
        let relative_path = full_path.strip_prefix(origin).unwrap().to_path_buf();

        if (!system && is_system(&full_path)?) ||
            (!hidden && is_hidden(&full_path)?) {
            continue;
        }

        if full_path.is_dir() {
            files.extend(collect_files(origin, &full_path, hidden, system)?);
        } else if full_path.is_file() {
            files.push(relative_path);
        }
    }

    Ok(files)
}

pub fn collect_top(dir: &Path, hidden: bool, system: bool) -> IOResult<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if  (!hidden && is_hidden(&path)?) ||
            (!system && is_system(&path)?) {
            continue;
        }

        if path.is_file() {
            files.push(path
                .strip_prefix(dir)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
                .to_path_buf());
        }
    }

    Ok(files)
}

// Windows helpers
#[cfg(target_os = "windows")]
pub fn is_hidden(file: &Path) -> IOResult<bool> {
    use std::os::windows::fs::MetadataExt;
    let attributes = fs::metadata(file)?.file_attributes();
    // Hidden code is 0x2
    Ok(attributes & 0x2 != 0)
}

#[cfg(target_os = "windows")]
pub fn is_system(file: &Path) -> IOResult<bool> {
    use std::os::windows::fs::MetadataExt;
    let attributes = fs::metadata(file)?.file_attributes();
    // System code is 0x4
    Ok(attributes & 0x4 != 0)
}

// Unix helpers (Both linux and macos)
#[cfg(unix)]
fn is_dot_hidden(file: &Path) -> bool {
    file.file_name()
        .is_some_and(|name| name.to_string_lossy().starts_with('.'))
}

#[cfg(unix)]
pub fn is_system(_file: &Path) -> IOResult<bool> {
    Ok(false)
}

// Linux helpers
#[cfg(target_os = "linux")]
pub fn is_hidden(file: &Path) -> IOResult<bool> {
    Ok(is_dot_hidden(file))
}

// MacOS helpers
#[cfg(target_os = "macos")]
pub fn is_hidden(file: &Path) -> IOResult<bool> {
    use std::os::macos::fs::MetadataExt;
    const UF_HIDDEN: u32 = 0x8000;
    let metadata = fs::metadata(file)?;
    Ok(is_dot_hidden(file) || metadata.st_flags() & UF_HIDDEN != 0)
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
            is_hidden(&file).expect("Failed to find out if the file was hidden"),
            "File {} is not hidden", file.display()
        );
        assert!(
            is_hidden(&nested_file).expect("Failed to find out if the file was hidden"),
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
            !is_system(&file).expect("Failed to check system status"),
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
            !is_hidden(&file).expect("Failed to find out if the file was hidden"),
            "File {} is hidden", file.display()
        );
        assert!(
            !is_hidden(&nested_file).expect("Failed to find out if the file was hidden"),
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
            is_hidden(&file).expect("Failed to find out if the file was hidden"),
            "File {} is not hidden", file.display()
        );
        assert!(
            is_hidden(&nested_file).expect("Failed to find out if the file was hidden"),
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
                is_hidden(path).expect("Failed to check hidden attribute"),
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
            is_hidden(&file).expect("Failed to check hidden attribute"),
            "File {} is not hidden", file.display()
        );
        assert!(
            is_hidden(&nested_file).expect("Failed to check hidden attribute"),
            "File {} is not hidden", nested_file.display()
        );
    }
}