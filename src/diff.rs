use std::collections::HashSet;
use std::{fs, io};
use std::path::{Path, PathBuf};
use console::{Color, Style};

enum ChangeState {
    Changed,
    Removed,
    Added
}

struct Change<'a> {
    path: &'a Path,
    change: ChangeState
}

fn diff_paths<'a>(
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

    let mut accumulated: Vec<Change> = Vec::new();

    for path in dest_files.iter() {
        // Does the origin have the file in the destination
        if origin_set.contains(path) {

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
                        continue;
                    }
                }

                // Size differed, so changed
                accumulated.push(Change { path, change: ChangeState::Changed });
            }
        } else {
            // Item wasn't in source, so it was added
            accumulated.push(Change { path, change: ChangeState::Added });
        }
    }

    // Check the source for files that are missing from dest
    for path in origin_files {
        // Does the destination not have the file?
        if !dest_set.contains(path.as_path()) {
            accumulated.push(Change { path, change: ChangeState::Removed, });
        }
    }

    Ok(accumulated)
}

pub fn print_diff(diff: Box<[Change]>) {
    for change in diff {
        let status = match change.change {
            ChangeState::Changed => "~",
            ChangeState::Removed => "-",
            ChangeState::Added => "+"
        };

        let status_color = match change.change {
            ChangeState::Changed => Color::Yellow,
            ChangeState::Removed => Color::Red,
            ChangeState::Added => Color::Green,
        };

        let styling = Style::new().fg(status_color);

        println!("{}", styling.apply_to(format!(" [{}] {}", status, change.path.display())));
    }
}