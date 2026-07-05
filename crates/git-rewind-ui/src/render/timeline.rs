use crate::render::theme::Theme;
use crate::state::TimelineState;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};

/// Renders the timeline panel into the specified area.
///
/// Precedence:
/// 1. If an error is present, renders the error block.
/// 2. If no items are present, renders a friendly empty state placeholder.
/// 3. Otherwise, renders the list of reflog timeline items.
pub fn render_timeline(frame: &mut Frame, state: &TimelineState, theme: &Theme, area: Rect) {
    let block = Block::default()
        .title("Git Rewind")
        .borders(Borders::ALL)
        .border_style(theme.border)
        .title_style(theme.title);

    // 1. Error Precedence
    if let Some(ref err) = state.error {
        let lines = vec![
            Line::from(vec![Span::styled(format!("ERROR: {}", err.title), theme.error)]),
            Line::raw(""),
            Line::from(vec![Span::styled(&err.message, theme.normal)]),
        ];
        let list = List::new(lines.into_iter().map(ListItem::new)).block(block);
        frame.render_widget(list, area);
        return;
    }

    // 2. Empty State Precedence
    if state.items.is_empty() {
        let line =
            Line::from(vec![Span::styled("No reflog entries available.", theme.empty_message)]);
        let list = List::new(vec![ListItem::new(line)]).block(block);
        frame.render_widget(list, area);
        return;
    }

    // 3. Populate List of Items
    let list_items: Vec<ListItem> = state
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_selected = i == state.selection.selected();

            let indicator =
                if is_selected { Span::styled("▶ ", theme.selected) } else { Span::raw("  ") };

            let summary_style = if is_selected { theme.selected } else { theme.normal };
            let summary_span = Span::styled(&item.summary, summary_style);

            let action_span = Span::styled(format!(" [{}]", item.action), theme.action);

            let time_str = item.timestamp.map(|t| format!(" ({})", t)).unwrap_or_default();
            let time_span = Span::styled(time_str, theme.timestamp);

            ListItem::new(Line::from(vec![indicator, summary_span, action_span, time_span]))
        })
        .collect();

    let list = List::new(list_items).block(block);
    frame.render_widget(list, area);
}
