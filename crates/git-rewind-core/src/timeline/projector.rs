use super::item::TimelineItem;
use crate::reflog::ReflogEntry;

/// Projects domain reflog entries into presentation timeline items.
///
/// This transformation is deterministic, side-effect free, and completely
/// presentation-independent (does not add terminal formatting or layout logic).
pub fn project(entries: &[ReflogEntry]) -> Vec<TimelineItem> {
    entries
        .iter()
        .map(|entry| TimelineItem {
            index: entry.index,
            commit: entry.commit.clone(),
            action: entry.action.clone(),
            summary: entry.message.trim().to_string(),
            timestamp: entry.timestamp,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reflog::{CommitId, ReflogAction, ReflogIndex, ReflogTimestamp};
    use std::time::SystemTime;

    #[test]
    fn test_project_empty() {
        let entries = vec![];
        let items = project(&entries);
        assert!(items.is_empty());
        assert_eq!(items.len(), entries.len()); // Invariant: output length equals input length
    }

    #[test]
    fn test_project_single() {
        let time = SystemTime::UNIX_EPOCH;
        let entries = vec![ReflogEntry {
            index: ReflogIndex(0),
            commit: CommitId("a1b2c3d".to_string()),
            previous_commit: None,
            action: ReflogAction::Commit,
            message: "Add parser".to_string(),
            timestamp: Some(ReflogTimestamp(time)),
        }];

        let items = project(&entries);
        assert_eq!(items.len(), 1);
        assert_eq!(items.len(), entries.len()); // Invariant: output length equals input length

        let item = &items[0];
        assert_eq!(item.index, ReflogIndex(0));
        assert_eq!(item.commit.to_string(), "a1b2c3d");
        assert_eq!(item.action, ReflogAction::Commit);
        assert_eq!(item.summary, "Add parser");
        assert_eq!(item.timestamp, Some(ReflogTimestamp(time)));
    }

    #[test]
    fn test_project_multiple_order_and_sequential_indices() {
        let time = SystemTime::UNIX_EPOCH;
        let entries = vec![
            ReflogEntry {
                index: ReflogIndex(0),
                commit: CommitId("commit1".to_string()),
                previous_commit: None,
                action: ReflogAction::Commit,
                message: "first commit message".to_string(),
                timestamp: Some(ReflogTimestamp(time)),
            },
            ReflogEntry {
                index: ReflogIndex(1),
                commit: CommitId("commit2".to_string()),
                previous_commit: Some(CommitId("commit1".to_string())),
                action: ReflogAction::Checkout,
                message: "moving from main to feature".to_string(),
                timestamp: Some(ReflogTimestamp(time)),
            },
        ];

        let items = project(&entries);
        assert_eq!(items.len(), 2);
        assert_eq!(items.len(), entries.len()); // Invariant: output length equals input length

        // Assert order is preserved and indices are sequential
        assert_eq!(items[0].index, ReflogIndex(0));
        assert_eq!(items[0].summary, "first commit message");

        assert_eq!(items[1].index, ReflogIndex(1));
        assert_eq!(items[1].summary, "moving from main to feature");
    }

    #[test]
    fn test_summary_extraction() {
        let entries = vec![
            ReflogEntry {
                index: ReflogIndex(0),
                commit: CommitId("c1".to_string()),
                previous_commit: None,
                action: ReflogAction::Commit,
                message: "  Add parser  ".to_string(), // message with padding
                timestamp: None,
            },
            ReflogEntry {
                index: ReflogIndex(1),
                commit: CommitId("c2".to_string()),
                previous_commit: None,
                action: ReflogAction::Reset,
                message: "moving to HEAD~1".to_string(),
                timestamp: None,
            },
        ];

        let items = project(&entries);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].summary, "Add parser"); // trimmed
        assert_eq!(items[1].summary, "moving to HEAD~1");
    }

    #[test]
    fn test_unknown_and_custom_actions() {
        let entries = vec![ReflogEntry {
            index: ReflogIndex(0),
            commit: CommitId("c1".to_string()),
            previous_commit: None,
            action: ReflogAction::Unknown("custom_action".to_string()),
            message: "some message".to_string(),
            timestamp: None,
        }];

        let items = project(&entries);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].action, ReflogAction::Unknown("custom_action".to_string()));
        assert_eq!(items[0].summary, "some message");
    }
}
