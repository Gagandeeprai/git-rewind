use ratatui::layout::{Constraint, Direction, Layout as RataLayout, Rect};

/// Represents screen partition areas for rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Layout {
    /// Title/header area.
    pub header: Rect,
    /// Timeline list area.
    pub timeline: Rect,
    /// Selected commit details metadata area.
    pub details: Rect,
    /// Selected commit changed files list area.
    pub files: Rect,
    /// Footer shortcut guide area.
    pub footer: Rect,
}

/// Computes the layout grid from the raw screen terminal viewport size.
pub fn compute(area: Rect) -> Layout {
    // Vertical split: Header (3), Main (Min 1), Footer (3)
    let outer = RataLayout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(3), Constraint::Min(10), Constraint::Length(3)])
        .split(area);

    let header = outer[0];
    let main_area = outer[1];
    let footer = outer[2];

    // Horizontal split in Main Area: Left (Timeline - 55%), Right (45%)
    let main_split = RataLayout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(main_area);

    let timeline = main_split[0];
    let right_area = main_split[1];

    // Vertical split on Right Area: Top (Commit details - 50%), Bottom (Files - 50%)
    let right_split = RataLayout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(right_area);

    let details = right_split[0];
    let files = right_split[1];

    Layout { header, timeline, details, files, footer }
}
