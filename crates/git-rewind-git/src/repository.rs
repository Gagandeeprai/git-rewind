use crate::error::GitError;
use git2::Repository;
use std::path::Path;

use std::fmt;

/// A handle to a Git repository, wrapping the underlying `git2::Repository`.
///
/// This encapsulates all interaction with the Git backend, preventing
/// direct dependency on `git2` types across other parts of the application.
pub struct RepositoryHandle {
    #[allow(dead_code)]
    inner: Repository,
}

impl fmt::Debug for RepositoryHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RepositoryHandle").finish_non_exhaustive()
    }
}

impl RepositoryHandle {
    pub(crate) fn repository(&self) -> &git2::Repository {
        &self.inner
    }

    /// Resets the repository HEAD to the specified commit.
    pub fn reset(
        &self,
        commit_id: &git_rewind_core::reflog::CommitId,
        hard: bool,
    ) -> Result<(), GitError> {
        let repo = &self.inner;
        let oid = git2::Oid::from_str(&commit_id.0).map_err(GitError::Git)?;
        let object = repo.find_object(oid, None).map_err(GitError::Git)?;
        let kind = if hard { git2::ResetType::Hard } else { git2::ResetType::Mixed };
        repo.reset(&object, kind, None).map_err(GitError::Git)?;
        Ok(())
    }

    /// Checks if the repository working tree or index contains uncommitted changes.
    pub fn is_dirty(&self) -> Result<bool, GitError> {
        let repo = &self.inner;
        let mut options = git2::StatusOptions::new();
        options.include_untracked(true);
        let statuses = repo.statuses(Some(&mut options)).map_err(GitError::Git)?;
        Ok(!statuses.is_empty())
    }
}

/// Discover the nearest Git repository starting from the current working directory.
///
/// This is a thin wrapper around `std::env::current_dir` and `discover_from`.
pub fn discover() -> Result<RepositoryHandle, GitError> {
    let current_dir = std::env::current_dir()
        .map_err(|e| GitError::CurrentDirectoryUnavailable(e.to_string()))?;
    discover_from(current_dir)
}

/// Discover the nearest Git repository starting from the specified path.
///
/// Traverses parent directories upwards until a Git repository is found.
pub fn discover_from(path: impl AsRef<Path>) -> Result<RepositoryHandle, GitError> {
    let path = path.as_ref();
    let inner = Repository::discover(path).map_err(|e| {
        if e.code() == git2::ErrorCode::NotFound {
            GitError::NotGitRepository(path.to_path_buf())
        } else {
            GitError::Git(e)
        }
    })?;
    Ok(RepositoryHandle { inner })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_discover_from_non_existent_or_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        // A clean temporary directory has no .git folder, so discovery must fail.
        let result = discover_from(temp_dir.path());
        assert!(result.is_err());
        match result.unwrap_err() {
            GitError::NotGitRepository(p) => assert_eq!(p, temp_dir.path()),
            other => panic!("Expected NotGitRepository error, got {:?}", other),
        }
    }

    #[test]
    fn test_discover_from_repository_root() {
        let temp_dir = TempDir::new().unwrap();
        // Initialize a Git repository in the temp directory.
        Repository::init(temp_dir.path()).unwrap();

        let result = discover_from(temp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_discover_from_nested_directories() {
        let temp_dir = TempDir::new().unwrap();
        // Initialize a Git repository.
        Repository::init(temp_dir.path()).unwrap();

        // Create a deep hierarchy: repo/src/feature/nested
        let nested_path = temp_dir.path().join("src").join("feature").join("nested");
        std::fs::create_dir_all(&nested_path).unwrap();

        // Discovery from the nested path should succeed and find the root repository.
        let result = discover_from(&nested_path);
        assert!(result.is_ok());
    }
}
