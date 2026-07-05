use git_rewind_core::timeline::TimelineItem;
use git_rewind_git::commit::CommitDetails;
use git_rewind_git::diff::CommitDiff;

/// Represents user intent triggered by input events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Request to exit the application.
    Quit,
    /// Request to select the next item.
    SelectNext,
    /// Request to select the previous item.
    SelectPrevious,
    /// Request to select the first item.
    SelectFirst,
    /// Request to select the last item.
    SelectLast,
    /// Action to update selected commit details in state.
    UpdateCommitDetails { details: CommitDetails, diff: CommitDiff },
    /// Action triggered when user initiates a reset (presses Enter/r).
    TriggerReset,
    /// Action to show the reset confirmation dialog.
    ShowConfirmReset { is_dirty: bool },
    /// User selects hard reset in dialog.
    ConfirmResetSelectHard,
    /// User selects mixed reset in dialog.
    ConfirmResetSelectMixed,
    /// User cancels the current dialog popup.
    CancelDialog,
    /// Action to set the timeline items directly.
    SetTimelineItems(Vec<TimelineItem>),
    /// Action to display an error state.
    ShowError { title: String, message: String },
}
