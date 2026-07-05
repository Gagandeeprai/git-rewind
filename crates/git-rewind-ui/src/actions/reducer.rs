use super::action::Action;
use crate::state::AppState;

/// Represents the outcome of dispatching an action.
/// Allows communicating lifecycle control instructions (like quitting or resetting) back to the runner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReduceResult {
    /// Continue running the application loop.
    Continue,
    /// Stop execution and exit the application loop.
    Quit,
    /// Initiate a Git repository reset operation.
    ResetRepository {
        /// The commit target to reset HEAD to.
        commit_id: git_rewind_core::reflog::CommitId,
        /// If true, performs a hard reset, discarding uncommitted modifications.
        hard: bool,
    },
}

/// Applies an action's mutations to the mutable AppState, returning a ReduceResult.
pub fn reduce(state: &mut AppState, action: Action) -> ReduceResult {
    match action {
        Action::Quit => ReduceResult::Quit,
        Action::SelectNext => {
            state.timeline.select_next();
            ReduceResult::Continue
        }
        Action::SelectPrevious => {
            state.timeline.select_previous();
            ReduceResult::Continue
        }
        Action::SelectFirst => {
            state.timeline.select_first();
            ReduceResult::Continue
        }
        Action::SelectLast => {
            state.timeline.select_last();
            ReduceResult::Continue
        }
        Action::UpdateCommitDetails { details, diff } => {
            state.timeline.selected_commit_details = Some(details);
            state.timeline.selected_commit_diff = Some(diff);
            ReduceResult::Continue
        }
        Action::TriggerReset => {
            // Triggering reset checks is performed by the runner (e.g. checking dirty status first)
            ReduceResult::Continue
        }
        Action::ShowConfirmReset { is_dirty } => {
            if !state.timeline.items.is_empty() {
                state.dialog = crate::state::Dialog::ConfirmReset {
                    commit_index: state.timeline.selection.selected(),
                    is_dirty,
                };
            }
            ReduceResult::Continue
        }
        Action::ConfirmResetSelectHard => {
            let result =
                if let crate::state::Dialog::ConfirmReset { commit_index, .. } = state.dialog {
                    if let Some(item) = state.timeline.items.get(commit_index) {
                        ReduceResult::ResetRepository { commit_id: item.commit.clone(), hard: true }
                    } else {
                        ReduceResult::Continue
                    }
                } else {
                    ReduceResult::Continue
                };
            state.dialog = crate::state::Dialog::None;
            result
        }
        Action::ConfirmResetSelectMixed => {
            let result = if let crate::state::Dialog::ConfirmReset { commit_index, .. } =
                state.dialog
            {
                if let Some(item) = state.timeline.items.get(commit_index) {
                    ReduceResult::ResetRepository { commit_id: item.commit.clone(), hard: false }
                } else {
                    ReduceResult::Continue
                }
            } else {
                ReduceResult::Continue
            };
            state.dialog = crate::state::Dialog::None;
            result
        }
        Action::CancelDialog => {
            state.dialog = crate::state::Dialog::None;
            state.timeline.clear_error();
            ReduceResult::Continue
        }
        Action::SetTimelineItems(items) => {
            state.timeline.set_items(items);
            state.timeline.selected_commit_details = None;
            state.timeline.selected_commit_diff = None;
            ReduceResult::Continue
        }
        Action::ShowError { title, message } => {
            state.timeline.set_error(crate::state::app::ErrorState { title, message });
            ReduceResult::Continue
        }
    }
}
