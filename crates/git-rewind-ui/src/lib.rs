pub mod actions;
pub mod render;
pub mod runtime;
pub mod state;

#[cfg(test)]
mod tests {
    use super::state::*;
    use git_rewind_core::reflog::{CommitId, ReflogAction, ReflogIndex};
    use git_rewind_core::timeline::TimelineItem;

    fn mock_item(index_val: usize, summary: &str) -> TimelineItem {
        TimelineItem {
            index: ReflogIndex(index_val),
            commit: CommitId("0123456789abcdef0123456789abcdef01234567".to_string()),
            action: ReflogAction::Commit,
            summary: summary.to_string(),
            timestamp: None,
        }
    }

    #[test]
    fn test_selection_initialization() {
        let selection = Selection::new(5);
        assert_eq!(selection.selected(), 5);

        let default_selection = Selection::default();
        assert_eq!(default_selection.selected(), 0);
    }

    #[test]
    fn test_selection_empty_movement() {
        let mut selection = Selection::default();
        assert_eq!(selection.selected(), 0);

        // Movement with empty list should keep index 0
        selection.next(0);
        assert_eq!(selection.selected(), 0);

        selection.previous();
        assert_eq!(selection.selected(), 0);

        selection.first(0);
        assert_eq!(selection.selected(), 0);

        selection.last(0);
        assert_eq!(selection.selected(), 0);

        selection.clamp(0);
        assert_eq!(selection.selected(), 0);
    }

    #[test]
    fn test_selection_bounds_movement() {
        let mut selection = Selection::default();
        let len = 3;

        // selection: 0
        assert_eq!(selection.selected(), 0);

        // lower bound protection
        selection.previous();
        assert_eq!(selection.selected(), 0);

        // next moves forward
        selection.next(len); // 1
        assert_eq!(selection.selected(), 1);

        selection.next(len); // 2
        assert_eq!(selection.selected(), 2);

        // upper bound protection
        selection.next(len); // remains 2
        assert_eq!(selection.selected(), 2);

        // previous moves backward
        selection.previous(); // 1
        assert_eq!(selection.selected(), 1);

        // select first
        selection.first(len);
        assert_eq!(selection.selected(), 0);

        // select last
        selection.last(len);
        assert_eq!(selection.selected(), 2);
    }

    #[test]
    fn test_selection_clamp() {
        let mut selection = Selection::new(5);

        // Clamping to smaller size
        selection.clamp(3);
        assert_eq!(selection.selected(), 2);

        // Clamping to empty
        selection.clamp(0);
        assert_eq!(selection.selected(), 0);
    }

    #[test]
    fn test_timeline_state_initialization() {
        let state = TimelineState::new();
        assert!(state.items.is_empty());
        assert_eq!(state.selection.selected(), 0);
        assert_eq!(state.status, LoadingStatus::Idle);
        assert!(state.error.is_none());
        assert!(!state.has_items());
        assert!(state.selected_item().is_none());
    }

    #[test]
    fn test_timeline_state_loading_items() {
        let mut state = TimelineState::new();
        state.set_status(LoadingStatus::Loading);
        assert_eq!(state.status, LoadingStatus::Loading);

        let items = vec![mock_item(0, "first commit"), mock_item(1, "second commit")];
        state.set_items(items);
        state.set_status(LoadingStatus::Ready);

        assert!(state.has_items());
        assert_eq!(state.status, LoadingStatus::Ready);
        assert_eq!(state.items.len(), 2);
        assert_eq!(state.selection.selected(), 0);

        // Verify selected item mapping
        let selected = state.selected_item().unwrap();
        assert_eq!(selected.summary, "first commit");

        // Navigate
        state.select_next();
        assert_eq!(state.selection.selected(), 1);
        assert_eq!(state.selected_item().unwrap().summary, "second commit");

        // Try navigating past end
        state.select_next();
        assert_eq!(state.selection.selected(), 1);

        // Navigate back
        state.select_previous();
        assert_eq!(state.selection.selected(), 0);

        // Go to last
        state.select_last();
        assert_eq!(state.selection.selected(), 1);

        // Go to first
        state.select_first();
        assert_eq!(state.selection.selected(), 0);
    }

    #[test]
    fn test_timeline_state_error_handling() {
        let mut state = TimelineState::new();

        let err = ErrorState {
            title: "Network Failure".to_string(),
            message: "Failed to connect to git daemon".to_string(),
        };
        state.set_error(err.clone());

        assert_eq!(state.error, Some(err));

        state.clear_error();
        assert!(state.error.is_none());
    }

    #[test]
    fn test_app_state_initialization() {
        let app = AppState::new();
        assert!(app.timeline.items.is_empty());

        let default_app = AppState::default();
        assert!(default_app.timeline.items.is_empty());
    }

    #[test]
    fn test_layout_computation() {
        use ratatui::layout::Rect;
        let area = Rect::new(0, 0, 100, 50);
        let layout_partitions = super::render::layout::compute(area);
        assert_eq!(layout_partitions.timeline, area);
    }

    #[test]
    fn test_render_empty_timeline() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let state = AppState::new();

        terminal
            .draw(|f| {
                super::render::Renderer::render(f, &state);
            })
            .unwrap();

        // Verify that the title and empty message are present in the output
        let buffer = terminal.backend().buffer();
        let text: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(text.contains("Git Rewind"));
        assert!(text.contains("No reflog entries available."));
    }

    #[test]
    fn test_render_populated_timeline() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        let mut state = AppState::new();
        let items = vec![mock_item(0, "first commit"), mock_item(1, "second commit")];
        state.timeline.set_items(items);
        state.timeline.select_first();

        terminal
            .draw(|f| {
                super::render::Renderer::render(f, &state);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let text: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(text.contains("Git Rewind"));
        assert!(text.contains("first commit"));
        assert!(text.contains("second commit"));
        // Selected item indicator
        assert!(text.contains("▶"));
    }

    #[test]
    fn test_render_error_state() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        let mut state = AppState::new();
        state.timeline.set_error(ErrorState {
            title: "Load Failure".to_string(),
            message: "Missing git binary".to_string(),
        });

        terminal
            .draw(|f| {
                super::render::Renderer::render(f, &state);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let text: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(text.contains("Git Rewind"));
        assert!(text.contains("ERROR: Load Failure"));
        assert!(text.contains("Missing git binary"));
    }

    #[test]
    fn test_event_translation() {
        use super::runtime::{Event, Key, translate_event};
        use crossterm::event::{
            Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers,
        };

        let raw_q = CrosstermEvent::Key(KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
        assert_eq!(translate_event(raw_q), Some(Event::Key(Key::Char('q'))));

        let raw_esc = CrosstermEvent::Key(KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
        assert_eq!(translate_event(raw_esc), Some(Event::Key(Key::Esc)));

        let raw_resize = CrosstermEvent::Resize(120, 40);
        assert_eq!(translate_event(raw_resize), Some(Event::Resize(120, 40)));

        let raw_other = CrosstermEvent::Key(KeyEvent {
            code: KeyCode::Char('x'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
        assert_eq!(translate_event(raw_other), None);
    }

    #[test]
    fn test_runtime_loop_exits_on_quit() {
        use super::runtime::{Event, Key, run_with_events};
        use git_rewind_cli::app::service::AppService;
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        git2::Repository::init(temp_dir.path()).unwrap();
        let repo = git_rewind_git::repository::discover_from(temp_dir.path()).unwrap();
        let service = AppService::new();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut state = AppState::new();

        let mut event_count = 0;
        let next_event = || {
            event_count += 1;
            if event_count == 3 { Ok(Some(Event::Key(Key::Esc))) } else { Ok(Some(Event::Tick)) }
        };

        let result = run_with_events(&mut terminal, &mut state, &service, &repo, next_event);
        assert!(result.is_ok());
        assert_eq!(event_count, 3);
    }

    #[test]
    fn test_event_to_action_mapping() {
        use super::actions::{Action, map_event_to_action};
        use super::runtime::{Event, Key};

        let state = AppState::new();

        assert_eq!(map_event_to_action(Event::Key(Key::Char('q')), &state), Some(Action::Quit));
        assert_eq!(map_event_to_action(Event::Key(Key::Esc), &state), Some(Action::CancelDialog));
        assert_eq!(map_event_to_action(Event::Key(Key::Down), &state), Some(Action::SelectNext));
        assert_eq!(
            map_event_to_action(Event::Key(Key::Char('j')), &state),
            Some(Action::SelectNext)
        );
        assert_eq!(map_event_to_action(Event::Key(Key::Up), &state), Some(Action::SelectPrevious));
        assert_eq!(
            map_event_to_action(Event::Key(Key::Char('k')), &state),
            Some(Action::SelectPrevious)
        );
        assert_eq!(map_event_to_action(Event::Key(Key::Home), &state), Some(Action::SelectFirst));
        assert_eq!(
            map_event_to_action(Event::Key(Key::Char('g')), &state),
            Some(Action::SelectFirst)
        );
        assert_eq!(map_event_to_action(Event::Key(Key::End), &state), Some(Action::SelectLast));
        assert_eq!(
            map_event_to_action(Event::Key(Key::Char('G')), &state),
            Some(Action::SelectLast)
        );

        assert_eq!(map_event_to_action(Event::Key(Key::Char('x')), &state), None);
        assert_eq!(map_event_to_action(Event::Tick, &state), None);
    }

    #[test]
    fn test_reducer_actions() {
        use super::actions::{Action, ReduceResult, reduce};

        let mut state = AppState::new();
        let items = vec![mock_item(0, "first"), mock_item(1, "second"), mock_item(2, "third")];
        state.timeline.set_items(items);

        // Initial selection is 0
        assert_eq!(state.timeline.selection.selected(), 0);

        // SelectNext
        assert_eq!(reduce(&mut state, Action::SelectNext), ReduceResult::Continue);
        assert_eq!(state.timeline.selection.selected(), 1);

        // SelectNext again
        assert_eq!(reduce(&mut state, Action::SelectNext), ReduceResult::Continue);
        assert_eq!(state.timeline.selection.selected(), 2);

        // Idempotence/Bounds test: SelectNext at upper bound
        assert_eq!(reduce(&mut state, Action::SelectNext), ReduceResult::Continue);
        assert_eq!(state.timeline.selection.selected(), 2);

        // SelectPrevious
        assert_eq!(reduce(&mut state, Action::SelectPrevious), ReduceResult::Continue);
        assert_eq!(state.timeline.selection.selected(), 1);

        // SelectFirst
        assert_eq!(reduce(&mut state, Action::SelectFirst), ReduceResult::Continue);
        assert_eq!(state.timeline.selection.selected(), 0);

        // Idempotence/Bounds test: SelectPrevious at lower bound
        assert_eq!(reduce(&mut state, Action::SelectPrevious), ReduceResult::Continue);
        assert_eq!(state.timeline.selection.selected(), 0);

        // SelectLast
        assert_eq!(reduce(&mut state, Action::SelectLast), ReduceResult::Continue);
        assert_eq!(state.timeline.selection.selected(), 2);

        // Quit action
        assert_eq!(reduce(&mut state, Action::Quit), ReduceResult::Quit);
    }

    #[test]
    fn test_dialog_reducer_and_mapper() {
        use super::actions::{Action, ReduceResult, map_event_to_action, reduce};
        use super::runtime::{Event, Key};
        use super::state::Dialog;

        let mut state = AppState::new();
        let items = vec![mock_item(0, "first"), mock_item(1, "second")];
        state.timeline.set_items(items);

        // Dispatches ShowConfirmReset
        assert_eq!(
            reduce(&mut state, Action::ShowConfirmReset { is_dirty: true }),
            ReduceResult::Continue
        );
        assert_eq!(state.dialog, Dialog::ConfirmReset { commit_index: 0, is_dirty: true });

        // When dialog is open and is_dirty is true, mapper checks dialog keys:
        // 'y' maps to proceeding option (ShowConfirmReset with is_dirty: false)
        assert_eq!(
            map_event_to_action(Event::Key(Key::Char('y')), &state),
            Some(Action::ShowConfirmReset { is_dirty: false })
        );
        // 'n' maps to CancelDialog
        assert_eq!(
            map_event_to_action(Event::Key(Key::Char('n')), &state),
            Some(Action::CancelDialog)
        );

        // Proceed by selecting Yes (sets is_dirty: false)
        assert_eq!(
            reduce(&mut state, Action::ShowConfirmReset { is_dirty: false }),
            ReduceResult::Continue
        );
        assert_eq!(state.dialog, Dialog::ConfirmReset { commit_index: 0, is_dirty: false });

        // When is_dirty is false, dialog supports reset selection:
        // 'h' maps to ConfirmResetSelectHard
        assert_eq!(
            map_event_to_action(Event::Key(Key::Char('h')), &state),
            Some(Action::ConfirmResetSelectHard)
        );
        // 'm' maps to ConfirmResetSelectMixed
        assert_eq!(
            map_event_to_action(Event::Key(Key::Char('m')), &state),
            Some(Action::ConfirmResetSelectMixed)
        );

        // Reducer executes reset hard
        assert_eq!(
            reduce(&mut state, Action::ConfirmResetSelectHard),
            ReduceResult::ResetRepository {
                commit_id: git_rewind_core::reflog::CommitId(
                    "0123456789abcdef0123456789abcdef01234567".to_string()
                ),
                hard: true,
            }
        );
        assert_eq!(state.dialog, Dialog::None);
    }
}
