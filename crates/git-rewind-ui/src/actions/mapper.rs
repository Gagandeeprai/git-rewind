use super::action::Action;
use crate::runtime::{Event, Key};
use crate::state::{AppState, Dialog};

/// Maps a runtime input event to a presentation Action based on current state context.
pub fn map_event_to_action(event: Event, state: &AppState) -> Option<Action> {
    match event {
        Event::Key(key) => match state.dialog {
            Dialog::None => match key {
                Key::Char('q') => Some(Action::Quit),
                Key::Esc => Some(Action::CancelDialog), // Clears any active error state
                Key::Down | Key::Char('j') => Some(Action::SelectNext),
                Key::Up | Key::Char('k') => Some(Action::SelectPrevious),
                Key::Home | Key::Char('g') => Some(Action::SelectFirst),
                Key::End | Key::Char('G') => Some(Action::SelectLast),
                Key::Char('r') | Key::Enter => Some(Action::TriggerReset),
                _ => None,
            },
            Dialog::ConfirmReset { is_dirty, .. } => {
                if is_dirty {
                    match key {
                        Key::Char('y') | Key::Char('Y') => {
                            Some(Action::ShowConfirmReset { is_dirty: false })
                        }
                        Key::Char('n') | Key::Char('N') | Key::Esc | Key::Char('c') => {
                            Some(Action::CancelDialog)
                        }
                        _ => None,
                    }
                } else {
                    match key {
                        Key::Char('h') | Key::Char('H') => Some(Action::ConfirmResetSelectHard),
                        Key::Char('m') | Key::Char('M') => Some(Action::ConfirmResetSelectMixed),
                        Key::Esc | Key::Char('c') | Key::Char('C') => Some(Action::CancelDialog),
                        _ => None,
                    }
                }
            }
        },
        _ => None,
    }
}
