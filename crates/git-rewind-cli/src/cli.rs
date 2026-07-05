use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "git-rewind",
    version,
    about = "Interactive Git reflog explorer",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Print the version information
    Version,
    /// Run diagnostic checks
    Doctor,
}
