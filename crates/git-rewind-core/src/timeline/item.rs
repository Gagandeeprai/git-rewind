use crate::reflog::{CommitId, ReflogAction, ReflogIndex, ReflogTimestamp};

/// A presentation-independent representation of a timeline item.
///
/// This provides a clean interface for UI renderers (e.g., TUI)
/// without exposing Git-specific raw reflog parsing details.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TimelineItem {
    /// The reflog index.
    pub index: ReflogIndex,
    /// The commit ID associated with this item.
    pub commit: CommitId,
    /// The action parsed from the reflog.
    pub action: ReflogAction,
    /// A human-readable presentation-friendly summary of the action.
    pub summary: String,
    /// The timestamp when this action occurred.
    pub timestamp: Option<ReflogTimestamp>,
}
