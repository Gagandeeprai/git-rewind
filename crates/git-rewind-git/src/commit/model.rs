use git_rewind_core::reflog::{CommitId, ReflogTimestamp};

/// Represents the author details of a commit.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommitAuthor {
    /// The author's name.
    pub name: String,
    /// The author's email address.
    pub email: String,
}

/// Represents the presentation details of a commit.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommitDetails {
    /// The commit ID.
    pub id: CommitId,
    /// The full commit message.
    pub message: String,
    /// The author of the commit.
    pub author: CommitAuthor,
    /// The commit timestamp.
    pub timestamp: ReflogTimestamp,
    /// The parent commit IDs.
    pub parents: Vec<CommitId>,
}
