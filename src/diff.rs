use std::collections::{BTreeMap, HashSet};
use std::ffi::OsString;
use std::io::Error;
use std::path::{Path, PathBuf};
use console::style;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum State {
    Added,
    Removed,
    Unchanged,
    Mixed,
    Modified
}

#[derive(Default)]
struct Node {
    children: BTreeMap<OsString, Node>,
    file: Option<State>,
    state: Option<State>,
}

impl Node {
    fn insert(&mut self, path: &Path, state: State) {
        let mut node = self;

        for component in path.components() {
            node = node.children
                .entry(component.as_os_str().to_os_string())
                .or_default();
        }

        node.file = Some(state);
    }

    fn summarize(&mut self) -> State {
        let mut combined = self.file;

        for child in self.children.values_mut() {
            let next = child.summarize();

            combined = Some(match combined {
                None => next,
                Some(previous) if previous == next => previous,
                _ => State::Mixed,
            });
        }

        let state = combined.unwrap_or(State::Unchanged);
        self.state = Some(state);
        state
    }

    fn print(&self, path: &Path) {
        match self.state {
            Some(State::Unchanged) | None => {}
            Some(State::Added) | Some(State::Removed) => {
                let display = format!(
                    "{}{}",
                    path.display(),
                    if self.children.is_empty() { "" } else { "/*" },
                );

                if self.state == Some(State::Added) {
                    println!(" {}", style(format!(" [+] {display}")).green());
                } else {
                    println!(" {}" ,style(format!(" [-] {display}")).red());
                }
            }
            Some(State::Mixed) => {
                for (name, child) in &self.children {
                    child.print(&path.join(name));
                }
            }
            Some(State::Modified) => {
                let display = path.display();
                println!(" {}", style(format!(" [+] {display}")).yellow());
            }
        }
    }
}

fn files_differ(origin: &Path, dest: &Path) -> std::io::Result<bool> {
    if std::fs::metadata(origin)?.len() != std::fs::metadata(dest)?.len() {
        return Ok(true);
    }

    Ok(std::fs::read(origin)? != std::fs::read(dest)?)
}

pub fn show_diff(
    origin_root: &Path,
    dest_root: &Path,
    origin_files: &[PathBuf],
    dest_files: &[PathBuf],
    name_only: bool,
) -> Result<(), Error> {
    let origin: HashSet<&Path> =
        origin_files.iter().map(PathBuf::as_path).collect();

    let dest: HashSet<&Path> =
        dest_files.iter().map(PathBuf::as_path).collect();

    let mut root = Node::default();

    for path in origin_files {
        let state = if !dest.contains(path.as_path()) {
            State::Removed
        } else if !name_only && files_differ(
            &origin_root.join(path),
            &dest_root.join(path),
        )? {
            State::Modified
        } else {
            State::Unchanged
        };

        root.insert(path, state);
    }

    for path in dest_files {
        if !origin.contains(path.as_path()) {
            root.insert(path, State::Added);
        }
    }

    root.summarize();
    root.print(Path::new(""));

    Ok(())
}