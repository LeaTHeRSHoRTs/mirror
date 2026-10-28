mod arguments;
mod traverser;
mod diff;
mod copier;

use std::{fs, process};
use std::io::Result as IOResult;
use std::path::{Path, PathBuf};

use clap::Parser;
use console::style;

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
        if cli.top_level { traverser::collect_top(dir, cli.hidden, cli.system) }
        else { traverser::collect_files(dir, dir, cli.hidden, cli.system) }
    };

    let origin_files = collect(&origin_path).expect("Failed to collect source files");
    let dest_files = collect(&dest_path).expect("Failed to collect destination files");

    if cli.compare {
        if origin_files.is_empty() && dest_files.is_empty() {
            println!("Source and destination directory has nothing to compare");
            process::exit(0);
        }

        let differences = diff::get_diffs(
            &origin_path, &dest_path,
            &origin_files, &dest_files,
            cli.name_only
        ).expect("Failed to calculate differences between source and destination");
        diff::print_diff(&differences, cli.verbose);
    } else {
        if origin_files.is_empty() {
            println!("Source directory has nothing to copy over");
            process::exit(0);
        }

        let display_origin = traverser::get_absolute_path(&origin_path);
        let display_dest = traverser::get_absolute_path(&dest_path);

        println!("Moving {} files from \"{}\" to \"{}\"", style(origin_files.len()).bold(), style(&display_origin).italic(), style(&display_dest).italic());

        let (copied, failures) = copier::copy_files(
            &origin_path,
            &dest_path,
            &origin_files,
            cli.verbose
        );

        println!("Copied {} files to {}", style(copied).bold(), style(&display_dest).italic());

        if !failures.is_empty() {
            println!("{}", style(format!("Failed to copy {} files into the destination.", failures.len())).bold().red());

            for (key, error) in failures {
                println!("  {}: {}", key.display(), error);
            }
        }
    }
}