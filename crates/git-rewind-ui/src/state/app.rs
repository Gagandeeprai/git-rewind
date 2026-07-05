use super::timeline::TimelineState;

/// Presentation-oriented error state.
/// Surfaces high-level error context for the UI (e.g. status bar, popup panels)
/// without coupling the state layer to low-level backend implementations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorState {
    pub title: String,
    pub message: String,
}

/// Represents active confirmation or warning dialogs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dialog {
    /// No dialog is currently active.
    None,
    /// Confirm rewinding HEAD to the selected reflog commit.
    ConfirmReset {
        /// The index of the target timeline commit.
        commit_index: usize,
        /// Whether the repository has uncommitted changes.
        is_dirty: bool,
    },
}

/// The root UI state representing the entire presentation layer state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
    pub timeline: TimelineState,
    pub dialog: Dialog,
}

impl AppState {
    /// Creates a new, idle `AppState` with empty sub-states.
    pub fn new() -> Self {
        Self { timeline: TimelineState::new(), dialog: Dialog::None }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
