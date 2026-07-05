use std::path::PathBuf;
use thiserror::Error;

/// Errors returned by the Git infrastructure layer.
#[derive(Debug, Error)]
pub enum GitError {
    /// Returned when a Git repository cannot be found starting at the given path.
    #[error("Not in a Git repository: {0}")]
    NotGitRepository(PathBuf),

    /// Returned when the current working directory cannot be accessed.
    #[error("Current directory is unavailable: {0}")]
    CurrentDirectoryUnavailable(String),

    /// Returned when an underlying git2 library error occurs.
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),
}
