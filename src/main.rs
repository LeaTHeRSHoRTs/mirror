mod arguments;
mod copier;

use std::fs;
use std::io::Result as IOResult;
use std::path::{Path, PathBuf};
use std::process;

use clap::Parser;

use arguments::Arguments;

fn main() {
    let cli: Arguments = Arguments::parse();
    
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
        if cli.recursive {
            copier::collect_files(dir, dir, cli.hidden, cli.system)
        } else {
            copier::collect_top(dir, cli.hidden, cli.system)
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
            for origin_file in &origin_files {
                if !dest_files.contains(origin_file) {
                    println!("    {}", origin_file.display());
                }
            }
        }
    } else {
        if origin_files.is_empty() {
            println!("Source directory has nothing to copy over");
            return;
        }
        
        for relative_path in &origin_files {
            let dest_file = dest_path.join(relative_path);

            if let Some(parent) = dest_file.parent() {
                fs::create_dir_all(parent).expect("Failed to create directory");
            }

            fs::copy(origin_path.join(relative_path), &dest_file)
                .unwrap_or_else(|error| panic!("Failed to copy file {}, {error}", dest_file.display()));
        }
    }
}
