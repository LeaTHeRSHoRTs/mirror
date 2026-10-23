mod arguments;
mod traverser;
mod diff;

use std::{fs, path, process};
use std::path::{Path, PathBuf};
use std::io::Result as IOResult;

use clap::Parser;
use indicatif::{ProgressIterator, ProgressStyle};
use console::style;

use arguments::Arguments;

fn main() {
    let cli: Arguments = Arguments::parse();

    println!();
    
    if !fs::exists(&cli.source).unwrap_or(false) {
        eprintln!("Source path does not exist: {}", cli.source);
        process::exit(1);
    }

    if !fs::exists(&cli.destination).unwrap_or(false) {
        eprintln!("Destination path does not exist: {}", cli.destination);
        process::exit(1);
    }

    let origin_path = PathBuf::from(&cli.source);
    let dest_path = PathBuf::from(&cli.destination);

    let collect = |dir: &Path| -> IOResult<Vec<PathBuf>> {
        if cli.top_level {
            traverser::collect_top(dir, cli.hidden, cli.system)
        } else {
            traverser::collect_files(dir, dir, cli.hidden, cli.system)
        }
    };

    let origin_files = collect(&origin_path).expect("Failed to collect source files");
    let dest_files = collect(&dest_path).expect("Failed to collect destination files");

    if cli.compare {
        if origin_files.is_empty() {
            println!("Source directory has nothing to compare");
            return;
        }

        let missing_files: Vec<_> = origin_files
            .iter()
            .filter(|file| !dest_files.contains(file))
            .collect();

        if missing_files.is_empty() {
            println!("Directories already match.");
        } else {
            diff::show_diff(
                &origin_path,
                &dest_path,
                &origin_files,
                &dest_files,
                cli.name
            ).expect("Could not generate diff");
        }
    } else {
        if origin_files.is_empty() {
            println!("Source directory has nothing to copy over");
            return;
        }

        let display_path = |path: &Path| {
            path::absolute(path).map_or_else(
                |_| format!("./{}", path.display()),
                |val| val.display().to_string(),
            )
        };

        let display_origin = display_path(&origin_path);
        let display_dest = display_path(&dest_path);

        println!("Moving {} files from \"{}\" to \"{}\"", dest_files.len(), style(&display_origin).italic(), style(&display_dest).italic());

        for relative_path in origin_files.iter().progress_with_style(
            ProgressStyle::with_template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} files ({eta})")
                .unwrap()
                .progress_chars("=>-")
        ) {
            let dest_file = dest_path.join(relative_path);

            if let Some(parent) = dest_file.parent() {
                fs::create_dir_all(parent).expect("Failed to create directory");
            }

            fs::copy(origin_path.join(relative_path), &dest_file)
                .unwrap_or_else(|error| panic!("Failed to copy file {}, {error}", dest_file.display()));
        }
    }
}