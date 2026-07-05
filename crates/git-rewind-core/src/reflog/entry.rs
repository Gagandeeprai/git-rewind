use super::action::ReflogAction;
use std::fmt;
use std::time::SystemTime;

/// Represents a Git commit identifier wrapper.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommitId(pub String);

impl fmt::Display for CommitId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a Git branch name wrapper.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BranchName(pub String);

impl fmt::Display for BranchName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a Git reference name wrapper.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReferenceName(pub String);

impl fmt::Display for ReferenceName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a domain wrapper around a Git reflog index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReflogIndex(pub usize);

impl fmt::Display for ReflogIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HEAD@{{{}}}", self.0)
    }
}

/// Represents a domain wrapper around the timestamp of a reflog entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReflogTimestamp(pub SystemTime);

impl fmt::Display for ReflogTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.duration_since(std::time::UNIX_EPOCH) {
            Ok(d) => write!(f, "{}", d.as_secs()),
            Err(_) => write!(f, "0"),
        }
    }
}

/// Represents the domain model of a single Git reflog entry.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReflogEntry {
    /// The index/position in the reflog chain.
    pub index: ReflogIndex,
    /// The current commit ID of this entry.
    pub commit: CommitId,
    /// The previous commit ID of this entry, if it exists.
    pub previous_commit: Option<CommitId>,
    /// The action associated with this entry.
    pub action: ReflogAction,
    /// The reflog message payload.
    pub message: String,
    /// The timestamp of when the entry was created.
    pub timestamp: Option<ReflogTimestamp>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_wrappers_display() {
        assert_eq!(CommitId("abc1234".to_string()).to_string(), "abc1234");
        assert_eq!(BranchName("main".to_string()).to_string(), "main");
        assert_eq!(ReferenceName("refs/heads/main".to_string()).to_string(), "refs/heads/main");
        assert_eq!(ReflogIndex(5).to_string(), "HEAD@{5}");

        let time = SystemTime::UNIX_EPOCH + Duration::from_secs(1620000000);
        assert_eq!(ReflogTimestamp(time).to_string(), "1620000000");
    }

    #[test]
    fn test_reflog_entry_equality() {
        let time = SystemTime::UNIX_EPOCH + Duration::from_secs(1620000000);
        let entry1 = ReflogEntry {
            index: ReflogIndex(0),
            commit: CommitId("a1b2c3d".to_string()),
            previous_commit: Some(CommitId("e5f6g7h".to_string())),
            action: ReflogAction::Commit,
            message: "feat: add feature".to_string(),
            timestamp: Some(ReflogTimestamp(time)),
        };

        let entry2 = entry1.clone();
        assert_eq!(entry1, entry2);
    }
}
