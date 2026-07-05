use clap::Parser;
use git_rewind_cli::app::service::AppService;
use git_rewind_cli::cli::{Cli, Commands};
use git_rewind_ui::runtime::{TerminalGuard, run};
use git_rewind_ui::state::AppState;
use std::io;

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    if let Some(command) = cli.command {
        match command {
            Commands::Version => {
                git_rewind_cli::commands::version::run()
                    .map_err(|e| io::Error::other(e.to_string()))?;
            }
            Commands::Doctor => {
                git_rewind_cli::commands::doctor::run()
                    .map_err(|e| io::Error::other(e.to_string()))?;
            }
        }
    } else {
        // Discover nearest Git repository
        let repo = match git_rewind_git::repository::discover() {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        };

        let mut state = AppState::new();
        let service = AppService::new();

        // Load active reflog timeline items from the Git repository
        match service.load_timeline(&repo) {
            Ok(items) => {
                state.timeline.set_items(items);
            }
            Err(err) => {
                state.timeline.set_error(git_rewind_ui::state::ErrorState {
                    title: "Repository Error".to_string(),
                    message: format!("Failed to load timeline: {}", err),
                });
            }
        }

        // Initialize terminal lifecycle management and run TUI loop
        let mut guard = TerminalGuard::new()?;
        run(guard.terminal(), &mut state, &service, &repo)?;
    }

    Ok(())
}
