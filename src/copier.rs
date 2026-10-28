use std::{fs, io};
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::time::Duration;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

pub fn copy_files(
    origin_root: &Path,
    dest_root: &Path,
    origin_files: &[PathBuf],
    verbose: bool
) -> (u64, HashMap<PathBuf, io::Error>) {
    let (mp, file_slots, mut file_history) = create_multi_progress(verbose);
    let progress_style = ProgressStyle::with_template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} files ({eta})")
        .unwrap_or(ProgressStyle::default_bar())
        .progress_chars("=>-");
    let progress_bar = mp.add(ProgressBar::new(origin_files.len() as u64).with_style(progress_style));

    let mut failures: HashMap<PathBuf, io::Error> = HashMap::new();
    let mut count: u64 = 0;
    for relative_path in origin_files {
        let dest_file = dest_root.join(relative_path);
        if let Some(parent) = dest_file.parent() {
            fs::create_dir_all(parent).expect("Failed to create directory");
        }

        if verbose {
            let current_file_msg = format!("Copying {}", relative_path.display());

            if file_history.len() == 3 {
                file_history.pop_front();
            }
            file_history.push_back(current_file_msg);

            for (idx, slot) in file_slots.iter().enumerate() {
                if let Some(msg) = file_history.get(idx) {
                    slot.set_message(msg.clone());
                } else {
                    slot.set_message("");
                }
            }
        }

        fs::copy(origin_root.join(relative_path), &dest_file)
            .map(|_| count += 1)
            .unwrap_or_else(|error| { failures.insert(dest_file.to_path_buf(), error); });
        progress_bar.inc(1)
    }

    // Clean up slots
    if verbose {
        for slot in file_slots {
            slot.finish_and_clear();
        }
    }

    progress_bar.finish();

    (count, failures)
}

// Returns (multi progress, the slots for the files being processed, the current files being processed
fn create_multi_progress(verbose: bool) -> (MultiProgress, Vec<ProgressBar>, VecDeque<String>) {
    let mp = MultiProgress::new();

    let mut file_slots = Vec::new();
    let file_history = VecDeque::with_capacity(4);

    if verbose {
        let spinner = ProgressStyle::default_spinner()
            .template("   {spinner} {msg}")
            .unwrap_or(ProgressStyle::default_spinner());

        for _ in 0..4 {
            let pb = mp.add(ProgressBar::new_spinner().with_style(spinner.clone()));
            pb.enable_steady_tick(Duration::from_millis(100));
            file_slots.push(pb);
        }
    }

    (mp, file_slots, file_history)
}