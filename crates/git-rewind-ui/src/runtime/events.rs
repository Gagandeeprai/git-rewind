use crossterm::event::{self, Event as CrosstermEvent, KeyCode};
use std::io;
use std::time::Duration;

/// Presentation-independent representation of keyboard inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    /// Character key press.
    Char(char),
    /// Up arrow key.
    Up,
    /// Down arrow key.
    Down,
    /// Home key.
    Home,
    /// End key.
    End,
    /// Escape key.
    Esc,
    /// Enter key.
    Enter,
}

/// Presentation event model.
/// Isolates the application state and loops from crossterm/backend-specific types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// A keyboard event.
    Key(Key),
    /// Resize the layout grid.
    Resize(u16, u16),
    /// Periodic tick event.
    Tick,
}

/// Translates a raw crossterm event into the presentation Event.
pub fn translate_event(raw: CrosstermEvent) -> Option<Event> {
    match raw {
        CrosstermEvent::Key(key_event) => {
            let key = match key_event.code {
                KeyCode::Char(c) => Key::Char(c),
                KeyCode::Up => Key::Up,
                KeyCode::Down => Key::Down,
                KeyCode::Home => Key::Home,
                KeyCode::End => Key::End,
                KeyCode::Esc => Key::Esc,
                KeyCode::Enter => Key::Enter,
                _ => return None,
            };
            Some(Event::Key(key))
        }
        CrosstermEvent::Resize(w, h) => Some(Event::Resize(w, h)),
        _ => None,
    }
}

/// Polls for next input event with the specified timeout.
pub fn poll_event(timeout: Duration) -> io::Result<Option<Event>> {
    if event::poll(timeout)? {
        let raw = event::read()?;
        Ok(translate_event(raw))
    } else {
        Ok(None)
    }
}
