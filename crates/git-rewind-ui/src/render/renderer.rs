use crate::render::layout;
use crate::render::theme::DEFAULT_THEME;
use crate::render::timeline;
use crate::state::{AppState, Dialog};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout as RataLayout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};

/// Stateless layout renderer.
/// Translates root application state into widgets on the terminal frame.
pub struct Renderer;

impl Renderer {
    /// Renders the complete screen layout based on the current AppState.
    pub fn render(frame: &mut Frame, state: &AppState) {
        let area = frame.area();
        let layout_partitions = layout::compute(area);
        let theme = &DEFAULT_THEME;

        // 1. Render Header Panel
        let header_block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme.border)
            .title(" Git Rewind — Safe State Travel ")
            .title_style(theme.title);
        let header_line = Line::from(vec![Span::styled(
            " Browse reflog timeline, check commit diffs, and rewind your repository state safely.",
            theme.normal,
        )]);
        let header_paragraph = Paragraph::new(header_line).block(header_block);
        frame.render_widget(header_paragraph, layout_partitions.header);

        // 2. Render Timeline Panel (Left)
        timeline::render_timeline(frame, &state.timeline, theme, layout_partitions.timeline);

        // 3. Render Commit Details Panel (Right Top)
        let details_block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme.border)
            .title(" Commit Details ")
            .title_style(theme.title);

        if let Some(ref details) = state.timeline.selected_commit_details {
            let message_summary = details.message.trim().replace('\r', "");
            let lines = vec![
                Line::from(vec![
                    Span::styled("Commit: ", theme.timestamp),
                    Span::styled(
                        &details.id.0[..std::cmp::min(8, details.id.0.len())],
                        theme.selected,
                    ),
                    Span::styled(format!(" [{}]", details.id.0), theme.timestamp),
                ]),
                Line::from(vec![
                    Span::styled("Author: ", theme.timestamp),
                    Span::styled(
                        format!("{} <{}>", details.author.name, details.author.email),
                        theme.normal,
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Date:   ", theme.timestamp),
                    Span::styled(details.timestamp.to_string(), theme.normal),
                ]),
                Line::raw(""),
                Line::from(vec![Span::styled(message_summary, theme.normal)]),
            ];
            let paragraph = Paragraph::new(lines).block(details_block).wrap(Wrap { trim: false });
            frame.render_widget(paragraph, layout_partitions.details);
        } else {
            let placeholder =
                Paragraph::new(Span::styled("No commit selected.", theme.empty_message))
                    .block(details_block);
            frame.render_widget(placeholder, layout_partitions.details);
        }

        // 4. Render Changed Files Panel (Right Bottom)
        let files_block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme.border)
            .title(" Changed Files ")
            .title_style(theme.title);

        if let Some(ref diff) = state.timeline.selected_commit_diff {
            if diff.files.is_empty() {
                let placeholder = Paragraph::new(Span::styled(
                    "No file changes in this commit.",
                    theme.empty_message,
                ))
                .block(files_block);
                frame.render_widget(placeholder, layout_partitions.files);
            } else {
                let list_items: Vec<ListItem> = diff
                    .files
                    .iter()
                    .map(|file| {
                        let change_symbol = match file.change {
                            git_rewind_git::diff::FileChangeType::Added => "[A]",
                            git_rewind_git::diff::FileChangeType::Modified => "[M]",
                            git_rewind_git::diff::FileChangeType::Deleted => "[D]",
                            git_rewind_git::diff::FileChangeType::Renamed => "[R]",
                            git_rewind_git::diff::FileChangeType::Copied => "[C]",
                            git_rewind_git::diff::FileChangeType::TypeChanged => "[T]",
                        };

                        let style = match file.change {
                            git_rewind_git::diff::FileChangeType::Added => {
                                ratatui::style::Style::default().fg(ratatui::style::Color::Green)
                            }
                            git_rewind_git::diff::FileChangeType::Deleted => {
                                ratatui::style::Style::default().fg(ratatui::style::Color::Red)
                            }
                            git_rewind_git::diff::FileChangeType::Modified => {
                                ratatui::style::Style::default().fg(ratatui::style::Color::Yellow)
                            }
                            _ => ratatui::style::Style::default().fg(ratatui::style::Color::Blue),
                        };

                        let line = Line::from(vec![
                            Span::styled(format!("{} ", change_symbol), style),
                            Span::styled(file.path.to_string_lossy().to_string(), theme.normal),
                        ]);
                        ListItem::new(line)
                    })
                    .collect();

                let list = List::new(list_items).block(files_block);
                frame.render_widget(list, layout_partitions.files);
            }
        } else {
            let placeholder =
                Paragraph::new(Span::styled("No file diff available.", theme.empty_message))
                    .block(files_block);
            frame.render_widget(placeholder, layout_partitions.files);
        }

        // 5. Render Footer Panel
        let footer_block = Block::default().borders(Borders::ALL).border_style(theme.border);
        let footer_text = Line::from(vec![
            Span::styled("j/k/Up/Down/Home/End", theme.selected),
            Span::raw(" Navigate Timeline | "),
            Span::styled("r/Enter", theme.selected),
            Span::raw(" Travel to Selected State | "),
            Span::styled("q/Esc", theme.selected),
            Span::raw(" Exit"),
        ]);
        let footer_paragraph = Paragraph::new(footer_text)
            .block(footer_block)
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(footer_paragraph, layout_partitions.footer);

        // 6. Render Popup Confirmation dialog overlays
        if let Dialog::ConfirmReset { commit_index, is_dirty } = state.dialog {
            let popup_area = centered_rect(70, 45, area);

            let target_commit_summary = state
                .timeline
                .items
                .get(commit_index)
                .map(|item| item.summary.as_str())
                .unwrap_or("");

            let target_commit_id = state
                .timeline
                .items
                .get(commit_index)
                .map(|item| {
                    let len = item.commit.0.len();
                    &item.commit.0[..std::cmp::min(8, len)]
                })
                .unwrap_or("");

            let dialog_block = Block::default()
                .title(" TRAVEL DESTINATION CONFIRMATION ")
                .borders(Borders::ALL)
                .border_style(theme.error);

            let dialog_text = if is_dirty {
                vec![
                    Line::from(vec![Span::styled(
                        " WARNING: You have uncommitted changes in your repository! ",
                        theme.error,
                    )]),
                    Line::from(vec![Span::styled(
                        " travel/rewind will discard uncommitted work unless you bypass. ",
                        theme.error,
                    )]),
                    Line::raw(""),
                    Line::from(vec![
                        Span::raw("Do you want to proceed and discard changes? "),
                        Span::styled("[y] Yes, proceed / [n] Cancel", theme.selected),
                    ]),
                ]
            } else {
                vec![
                    Line::from(vec![
                        Span::raw("Travel repository state to commit: "),
                        Span::styled(
                            format!("{} - {}", target_commit_id, target_commit_summary),
                            theme.selected,
                        ),
                    ]),
                    Line::raw(""),
                    Line::from(vec![
                        Span::styled(" [h] Hard Reset ", theme.error),
                        Span::raw("➔ Travel and discard all uncommitted changes"),
                    ]),
                    Line::from(vec![
                        Span::styled(" [m] Mixed Reset ", theme.selected),
                        Span::raw("➔ Travel and preserve changes in files"),
                    ]),
                    Line::from(vec![
                        Span::styled(" [c/Esc] Cancel ", theme.normal),
                        Span::raw("➔ Stay here"),
                    ]),
                ]
            };

            let dialog_paragraph = Paragraph::new(dialog_text)
                .block(dialog_block)
                .alignment(ratatui::layout::Alignment::Center)
                .wrap(Wrap { trim: false });

            frame.render_widget(Clear, popup_area);
            frame.render_widget(dialog_paragraph, popup_area);
        }
    }
}

/// Helper function to calculate popup overlays centered on the screen.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = RataLayout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    RataLayout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
