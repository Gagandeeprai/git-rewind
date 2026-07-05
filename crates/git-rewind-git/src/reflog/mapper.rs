use crate::error::GitError;
use git_rewind_core::reflog::{CommitId, ReflogAction, ReflogEntry, ReflogIndex, ReflogTimestamp};
use std::time::{Duration, SystemTime};

/// Translates a git2 reflog entry into a domain ReflogEntry.
pub(super) fn parse_entry(
    index: usize,
    entry: &git2::ReflogEntry,
) -> Result<ReflogEntry, GitError> {
    let commit = map_commit_id(entry.id_new());
    let previous_commit =
        if entry.id_old().is_zero() { None } else { Some(map_commit_id(entry.id_old())) };

    let timestamp = map_timestamp(entry.committer().when());

    let raw_msg = entry.message().unwrap_or(None).unwrap_or("");
    let (action, message) = parse_message(raw_msg);

    Ok(ReflogEntry {
        index: ReflogIndex(index),
        commit,
        previous_commit,
        action,
        message,
        timestamp,
    })
}

/// Helper to map commit ID.
fn map_commit_id(oid: git2::Oid) -> CommitId {
    CommitId(oid.to_string())
}

/// Helper to map timestamp.
fn map_timestamp(time: git2::Time) -> Option<ReflogTimestamp> {
    let time_secs = time.seconds();
    let system_time = if time_secs >= 0 {
        SystemTime::UNIX_EPOCH + Duration::from_secs(time_secs as u64)
    } else {
        SystemTime::UNIX_EPOCH
    };
    Some(ReflogTimestamp(system_time))
}

/// Helper to parse raw message into action and details.
fn parse_message(raw: &str) -> (ReflogAction, String) {
    if let Some((action_part, message_part)) = raw.split_once(':') {
        let action = parse_action(action_part.trim());
        let message = message_part.trim().to_string();
        (action, message)
    } else {
        let action = ReflogAction::Unknown("".to_string());
        let message = raw.trim().to_string();
        (action, message)
    }
}

/// Helper to parse raw action string.
fn parse_action(action_str: &str) -> ReflogAction {
    ReflogAction::from(action_str)
}
