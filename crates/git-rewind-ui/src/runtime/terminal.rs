use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::{self, Stdout};

/// RAII guard for managing the lifecycle of the terminal resources.
/// Enables raw mode and enters the alternate screen on initialization,
/// and guarantees automatic restoration of standard terminal mode when dropped.
pub struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalGuard {
    /// Initializes raw mode, enters alternate screen, and constructs a Terminal.
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    /// Accesses the underlying terminal.
    pub fn terminal(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>> {
        &mut self.terminal
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // Leave the alternate screen and disable raw mode to restore the user's terminal
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}
