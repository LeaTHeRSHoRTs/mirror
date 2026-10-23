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

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn is_dot_hidden(file: &Path) -> bool {
    file.file_name()
        .is_some_and(|name| name.to_string_lossy().starts_with('.'))
}

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

#[cfg(target_os = "linux")]
pub fn is_hidden(file: &Path) -> IOResult<bool> {
    Ok(is_dot_hidden(file))
}

#[cfg(target_os = "linux")]
pub fn is_system(_file: &Path) -> IOResult<bool> {
    Ok(false)
}

#[cfg(target_os = "macos")]
pub fn is_hidden(file: &Path) -> IOResult<bool> {
    use std::os::macos::fs::MetadataExt;
    const UF_HIDDEN: u32 = 0x8000;
    let metadata = fs::metadata(file)?;
    Ok(is_dot_hidden(file) || metadata.st_flags() & UF_HIDDEN != 0)
}

#[cfg(target_os = "macos")]
pub fn is_system(_file: &Path) -> IOResult<bool> {
    Ok(false)
}