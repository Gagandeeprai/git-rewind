use thiserror::Error;

/// High-level application errors that occur during orchestration.
#[derive(Debug, Error)]
pub enum AppError {
    /// Error wrapper for Git infrastructure failures.
    #[error("Git backend error: {0}")]
    Git(#[from] git_rewind_git::error::GitError),

    /// General application failure.
    #[error("Application error: {0}")]
    General(String),
}
