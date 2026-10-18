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
    #[arg(short = 'H', long)]
    pub hidden: bool,

    /// Recursively checks files and directories. Defaults to true when used without the --compare or -c flag
    #[arg(short, long)]
    pub recursive: bool
}