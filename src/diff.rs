use std::collections::HashSet;
use std::{fs, io};
use std::path::{Path, PathBuf};
use console::{Color, Style};

#[derive(PartialEq)]
pub enum ChangeState {
    Changed,
    Removed,
    Added,
    None
}

pub struct Change<'a> {
    path: &'a Path,
    change: ChangeState
}

pub fn get_diffs<'a>(
    origin_root: &Path,
    dest_root: &Path,
    origin_files: &'a [PathBuf],
    dest_files: &'a [PathBuf],
    names_only: bool,
) -> Result<Vec<Change<'a>>, io::Error> {
    let origin_set: HashSet<&Path> =
        origin_files.iter().map(PathBuf::as_path).collect();
    let dest_set: HashSet<&Path> =
        dest_files.iter().map(PathBuf::as_path).collect();

    let mut accumulated: Vec<Change> = dest_files
        .iter()
        .map(|path| {
        // Does the origin have the file in the destination
        if origin_set.contains(path.as_path()) {

            // If we are to not only check the name
            if !names_only {
                let origin_size = fs::metadata(origin_root.join(path))?.len();
                let dest_size = fs::metadata(dest_root.join(path))?.len();

                // Check the size first for difference
                if origin_size == dest_size {
                    let origin_bytes = fs::read(origin_root.join(path))?;
                    let dest_bytes = fs::read(dest_root.join(path))?;

                    // Everything matches, so ignore it
                    if origin_bytes == dest_bytes {
                        return Ok(Change { path, change: ChangeState::None });
                    }
                }
            }
        } else {
            // Item wasn't in source, so it was added
            return Ok(Change { path, change: ChangeState::Added });
        }

        return Ok(Change { path, change: ChangeState::None });
    }).collect::<Result<Vec<_>, io::Error>>()?;

    // Check the source for files that are missing from dest
    accumulated.extend(
        origin_files
            .iter()
            .filter(|path| !dest_set.contains(path.as_path()))
            .map(|path| {
            // Does the destination not have the file?
            if !dest_set.contains(path.as_path()) {
                return Change { path, change: ChangeState::Removed, };
            }

            Change { path, change: ChangeState::None }
        }).collect::<Vec<_>>()
    );

    Ok(accumulated)
}

pub fn print_diff(diff: &[Change<'_>]) {
    let mut has_changes = false;
    
    for change in diff {
        has_changes = true;
        
        let status = match change.change {
            ChangeState::Changed => "~",
            ChangeState::Removed => "-",
            ChangeState::Added => "+",
            ChangeState::None => continue,
        };

        let status_color = match change.change {
            ChangeState::Changed => Color::Yellow,
            ChangeState::Removed => Color::Red,
            ChangeState::Added => Color::Green,
            ChangeState::None => continue,
        };

        let styling = Style::new().fg(status_color);
        println!("{}", styling.apply_to(format!(" [{}] {}", status, change.path.display())));
    }

    if !has_changes {
        println!("Directories already match.");
    }
}

// Tests

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{tempdir, TempDir};

    fn create_temps<F : FnOnce(&Path, &Path)>(setup: F) -> (TempDir, TempDir) {
        let (source, destination) = (tempdir().unwrap(), tempdir().unwrap());
        setup(source.path(), destination.path());
        (source, destination)
    }

    fn create_temps_simple() -> (TempDir, TempDir) {
        create_temps(|_, _| {})
    }

    #[test]
    fn directory_diffs_returns_empty_vec_for_nothing() {
        let (source, destination) = create_temps_simple();
        let files: Vec<PathBuf> = Vec::new();

        let res = get_diffs(source.path(), destination.path(), &files, &files, false)
            .expect("Failed to calculate differences in directories");
        assert!(res.is_empty(), "directory_diffs returned a difference, instead got: {}", res.len());
    }

    #[test]
    fn directory_diffs_returns_vec_with_none_indicators() {
        let (source, destination) = create_temps(|source, dest| {
            fs::write(source.join("test.txt"), "hello").unwrap();
            fs::write(dest.join("test.txt"), "hello").unwrap();
        });
        let files = vec![PathBuf::from("test.txt")];

        let res = get_diffs(source.path(), destination.path(), &files, &files, false)
            .expect("Failed to calculate differences in directories");
        let len = res.len();
        assert_eq!(len, 1, "directory_diffs returned a difference, instead got: {}", len);
    }

    #[test]
    fn directory_diffs_returns_filled_vec_for_difference() {
        let (source, destination) = create_temps(|source, dest| {
            fs::write(source.join("test.txt"), "hello").unwrap();
            fs::write(dest.join("test2.txt"), "goodbye").unwrap();
        });
        let source_files = vec![PathBuf::from("test.txt")];
        let dest_files = vec![PathBuf::from("test2.txt")];

        let res = get_diffs(source.path(), destination.path(), &source_files, &dest_files, false)
            .expect("Failed to calculate differences in directories");
        assert_eq!(res.len(), 2, "directory_diffs returned a difference different to 2, instead got: {}", res.len());
        assert!(res.iter().any(|change| {
            change.path == Path::new("test.txt") && matches!(change.change, ChangeState::Removed)
        }), "directory_diffs did not mark test.txt as removed");
        assert!(res.iter().any(|change| {
            change.path == Path::new("test2.txt") && matches!(change.change, ChangeState::Added)
        }), "directory_diffs did not mark test2.txt as added");
    }
}