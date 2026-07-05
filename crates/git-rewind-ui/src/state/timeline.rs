use super::app::ErrorState;
use super::selection::Selection;
use git_rewind_core::timeline::TimelineItem;
use git_rewind_git::commit::CommitDetails;
use git_rewind_git::diff::CommitDiff;

/// Representation of the loading state for UI modules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoadingStatus {
    #[default]
    Idle,
    Loading,
    Ready,
}

/// Presentation state for the Reflog Timeline panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimelineState {
    /// Loaded timeline items.
    pub items: Vec<TimelineItem>,
    /// Selected item navigation.
    pub selection: Selection,
    /// Loading status.
    pub status: LoadingStatus,
    /// Active error details, if any.
    pub error: Option<ErrorState>,
    /// Detailed metadata of the highlighted commit.
    pub selected_commit_details: Option<CommitDetails>,
    /// Changed files of the highlighted commit.
    pub selected_commit_diff: Option<CommitDiff>,
}

impl TimelineState {
    /// Creates a new `TimelineState` in default idle configuration.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selection: Selection::default(),
            status: LoadingStatus::Idle,
            error: None,
            selected_commit_details: None,
            selected_commit_diff: None,
        }
    }

    /// Checks if the timeline contains any items.
    pub fn has_items(&self) -> bool {
        !self.items.is_empty()
    }

    /// Sets the loaded timeline items, adjusts selection bounds, and clears status.
    pub fn set_items(&mut self, items: Vec<TimelineItem>) {
        self.items = items;
        self.selection.clamp(self.items.len());
    }

    /// Updates the loading status.
    pub fn set_status(&mut self, status: LoadingStatus) {
        self.status = status;
    }

    /// Sets the UI error state, transitioning to failure reporting.
    pub fn set_error(&mut self, error: ErrorState) {
        self.error = Some(error);
    }

    /// Clears any active error state.
    pub fn clear_error(&mut self) {
        self.error = None;
    }

    /// Navigates to the next item.
    pub fn select_next(&mut self) {
        self.selection.next(self.items.len());
    }

    /// Navigates to the previous item.
    pub fn select_previous(&mut self) {
        self.selection.previous();
    }

    /// Navigates to the first item.
    pub fn select_first(&mut self) {
        self.selection.first(self.items.len());
    }

    /// Navigates to the last item.
    pub fn select_last(&mut self) {
        self.selection.last(self.items.len());
    }

    /// Returns a reference to the currently selected timeline item, if available.
    pub fn selected_item(&self) -> Option<&TimelineItem> {
        if self.items.is_empty() { None } else { self.items.get(self.selection.selected()) }
    }
}

impl Default for TimelineState {
    fn default() -> Self {
        Self::new()
    }
}
