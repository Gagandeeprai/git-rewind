mod cli;
mod commands;

use anyhow::Result;
use clap::Parser;
use cli::Commands;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    if let Some(command) = cli.command {
        match command {
            Commands::Version => commands::version::run()?,
            Commands::Doctor => commands::doctor::run()?,
        }
    } else {
        println!("Git Rewind (Interactive UI not implemented yet)");
    }

    Ok(())
}
