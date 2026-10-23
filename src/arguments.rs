use clap::Parser;

/// Mirrors a source directory into the destination directory
#[derive(Parser, Debug)]
pub struct Arguments {
    /// The source to copy from
    pub source: String,

    /// The destination to copy into. Defaults to the current directory
    #[arg(default_value = ".")]
    pub destination: String,

    /// Compares the directories instead of mirroring
    #[arg(short, long)]
    pub compare: bool,

    /// Logs verbosely
    #[arg(short, long)]
    pub verbose: bool,

    /// Forces certain operations
    #[arg(long)]
    pub force: bool,

    /// Copies over system files as well
    #[arg(short, long)]
    pub system: bool,

    /// Copies over hidden files as well
    #[arg(short = 'H', long)] // Set to H because -h is for --help
    pub hidden: bool,

    /// Checks or copies over only the top level files
    #[arg(short, long)]
    pub top_level: bool,

    /// Whether to compare files by name only, disregarding the contents. Improves performance
    #[arg(short, long)]
    pub name: bool
}