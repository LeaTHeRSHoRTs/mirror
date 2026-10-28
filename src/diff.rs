use std::collections::{BTreeMap, HashSet};
use std::{fs, io};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use console::{Color, Style};

#[derive(PartialEq, Copy, Clone)]
pub enum ChangeState {
    Changed,
    Removed,
    Added,
    Mixed,
    None
}

pub struct Change<'a> {
    path: &'a Path,
    change: ChangeState
}

#[derive(Default)]
struct Node {
    change: Option<ChangeState>,
    children: BTreeMap<OsString, Node>
}

impl Node {
    fn insert(&mut self, path: &Path, change: ChangeState) {
        let mut node = self;

        for component in path.components() {
            node = node.children
                .entry(component.as_os_str().to_os_string())
                .or_default();
        }

        node.change = Some(change);
    }
    fn summarize(&self) -> Option<ChangeState> {
        if self.children.is_empty() {
            return self.change;
        }

        let mut states = self.children.values().map(Node::summarize);

        let Some(Some(first)) = states.next() else {
            return Some(ChangeState::Mixed);
        };

        // If all files are same in the folder, collapse
        if states.all(|state| state == Some(first)) {
            Some(first)
        } else {
            Some(ChangeState::Mixed)
        }
    }
    fn print(&self, path: &mut PathBuf, verbose: bool) {
        let Some(state) = self.summarize() else {
            return;
        };

        // Collapse if the children have the same states
        if !verbose
            && !self.children.is_empty()
            && state != ChangeState::Mixed
        {
            if state != ChangeState::None || verbose {
                Node::print_entry(path, state, !self.children.is_empty());
            }
            return;
        }

        // If it's a file
        if self.children.is_empty() {
            // If it was modified in some way
            if state != ChangeState::None {
                Self::print_entry(path, state, false);
            }
            return;
        }

        for (name, child) in &self.children {
            path.push(name);
            child.print(path, verbose);
            path.pop();
        }
    }

    // Static methods
    fn print_entry(path: &Path, state: ChangeState, is_dir: bool) {
        let (status, color) = match state {
            ChangeState::Changed => ("~", Color::Yellow),
            ChangeState::Removed => ("-", Color::Red),
            ChangeState::Added => ("+", Color::Green),
            ChangeState::None | ChangeState::Mixed => return,
        };

        let suffix = if is_dir { "/*" } else { "" };

        println!(
            "{}",
            Style::new().fg(color).apply_to(format!(
                " [{}] {}{}",
                status,
                path.display().to_string().replace("\\", "/"),
                suffix
            ))
        );
    }
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
            return if origin_set.contains(path.as_path()) {
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

                Ok(Change { path, change: ChangeState::Changed })
            } else {
                // Item wasn't in source, so it was added
                Ok(Change { path, change: ChangeState::Added })
            }
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

pub fn print_diff(diff: &[Change<'_>], verbose: bool) {
    let mut root = Node::default();

    for change in diff {
        root.insert(change.path, change.change);
    }

    let mut has_changes = false;
    let mut not_changed = 0;


    for change in diff {
        match change.change {
            ChangeState::None => not_changed += 1,
            _ => has_changes = true,
        }
    }

    if has_changes {
        println!("There are differences in the two directories:");

        for (name, child) in &root.children {
            let mut path = PathBuf::from(name.as_os_str());
            child.print(&mut path, verbose);
        }

        println!("{} files are the same in both directories.", not_changed)
    } else {
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