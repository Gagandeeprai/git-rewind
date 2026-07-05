use ratatui::style::{Color, Modifier, Style};

/// Centralized presentation theme.
/// Defines visual styles for various UI widgets and text elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    /// Border of main block panels.
    pub border: Style,
    /// Title headers.
    pub title: Style,
    /// Selected items/indices.
    pub selected: Style,
    /// General regular list items.
    pub normal: Style,
    /// Action category text (e.g., `[commit]`, `[checkout]`).
    pub action: Style,
    /// Dates or epoch values.
    pub timestamp: Style,
    /// High-level presentation errors.
    pub error: Style,
    /// Placeholder texts for empty states.
    pub empty_message: Style,
}

/// The default premium dark theme configuration.
pub static DEFAULT_THEME: Theme = Theme {
    border: Style::new().fg(Color::Rgb(128, 128, 128)),
    title: Style::new().fg(Color::Rgb(255, 255, 255)).add_modifier(Modifier::BOLD),
    selected: Style::new().fg(Color::Rgb(255, 187, 0)).add_modifier(Modifier::BOLD),
    normal: Style::new().fg(Color::Rgb(200, 200, 200)),
    action: Style::new().fg(Color::Rgb(120, 150, 200)),
    timestamp: Style::new().fg(Color::Rgb(100, 100, 100)),
    error: Style::new().fg(Color::Rgb(220, 80, 80)),
    empty_message: Style::new().fg(Color::Rgb(140, 140, 140)),
};
