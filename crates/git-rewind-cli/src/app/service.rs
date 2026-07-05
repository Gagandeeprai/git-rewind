use crate::app::model::AppError;
use git_rewind_core::reflog::CommitId;
use git_rewind_core::timeline::{self, TimelineItem};
use git_rewind_git::commit::{self, CommitDetails};
use git_rewind_git::diff::{self, CommitDiff};
use git_rewind_git::repository::RepositoryHandle;

/// The thin application service layer that orchestrates backend workflows.
///
/// Frontends (TUI, GUI, etc.) should use this service to perform operations
/// rather than coordinating multiple backend modules directly.
#[derive(Debug, Default, Clone, Copy)]
pub struct AppService;

impl AppService {
    /// Creates a new instance of the application service.
    pub fn new() -> Self {
        Self
    }

    /// Loads the reflog timeline for the specified repository.
    pub fn load_timeline(
        &self,
        repository: &RepositoryHandle,
    ) -> Result<Vec<TimelineItem>, AppError> {
        let entries = git_rewind_git::reflog::read_reflog(repository)?;
        let items = timeline::project(&entries);
        Ok(items)
    }

    /// Retrieves detailed metadata for a specific commit.
    pub fn inspect_commit(
        &self,
        repository: &RepositoryHandle,
        id: CommitId,
    ) -> Result<CommitDetails, AppError> {
        let details = commit::inspect(repository, id)?;
        Ok(details)
    }

    /// Retrieves the list of changed files for a specific commit.
    pub fn inspect_diff(
        &self,
        repository: &RepositoryHandle,
        id: CommitId,
    ) -> Result<CommitDiff, AppError> {
        let diff = diff::inspect(repository, id)?;
        Ok(diff)
    }

    /// Resets the repository HEAD to the specified commit.
    pub fn reset_repository(
        &self,
        repository: &RepositoryHandle,
        id: &CommitId,
        hard: bool,
    ) -> Result<(), AppError> {
        repository.reset(id, hard)?;
        Ok(())
    }

    /// Checks if the repository contains uncommitted changes.
    pub fn is_dirty(&self, repository: &RepositoryHandle) -> Result<bool, AppError> {
        let dirty = repository.is_dirty()?;
        Ok(dirty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{Oid, Signature};
    use std::fs;
    use tempfile::TempDir;

    fn test_signature() -> Signature<'static> {
        Signature::now("App Service User", "app@example.com").unwrap()
    }

    fn make_commit(repo: &git2::Repository, message: &str) -> Oid {
        let mut index = repo.index().unwrap();
        let file_path = repo.path().parent().unwrap().join("test.txt");

        let content = format!("content for {}", message);
        fs::write(&file_path, content).unwrap();

        index.add_path(std::path::Path::new("test.txt")).unwrap();
        index.write().unwrap();

        let oid = index.write_tree().unwrap();
        let tree = repo.find_tree(oid).unwrap();

        let sig = test_signature();

        let parent = match repo.head() {
            Ok(head_ref) => vec![head_ref.peel_to_commit().unwrap()],
            Err(_) => Vec::new(),
        };

        let parents_ref: Vec<&git2::Commit> = parent.iter().collect();

        repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents_ref).unwrap()
    }

    #[test]
    fn test_service_load_timeline_success() {
        let temp_dir = TempDir::new().unwrap();
        let repo = git2::Repository::init(temp_dir.path()).unwrap();
        make_commit(&repo, "feat: test timeline service delegation");

        let handle = git_rewind_git::repository::discover_from(temp_dir.path()).unwrap();

        let service = AppService::new();
        let items = service.load_timeline(&handle).unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].summary, "feat: test timeline service delegation");
    }

    #[test]
    fn test_service_inspect_commit_success() {
        let temp_dir = TempDir::new().unwrap();
        let repo = git2::Repository::init(temp_dir.path()).unwrap();
        let oid = make_commit(&repo, "feat: inspect commit delegation");

        let handle = git_rewind_git::repository::discover_from(temp_dir.path()).unwrap();
        let commit_id = CommitId(oid.to_string());

        let service = AppService::new();
        let details = service.inspect_commit(&handle, commit_id.clone()).unwrap();

        assert_eq!(details.id, commit_id);
        assert_eq!(details.author.name, "App Service User");
    }

    #[test]
    fn test_service_inspect_diff_success() {
        let temp_dir = TempDir::new().unwrap();
        let repo = git2::Repository::init(temp_dir.path()).unwrap();
        let oid = make_commit(&repo, "feat: inspect diff delegation");

        let handle = git_rewind_git::repository::discover_from(temp_dir.path()).unwrap();
        let commit_id = CommitId(oid.to_string());

        let service = AppService::new();
        let diff = service.inspect_diff(&handle, commit_id).unwrap();

        assert_eq!(diff.files.len(), 1);
        assert_eq!(diff.files[0].path, std::path::Path::new("test.txt"));
    }

    #[test]
    fn test_service_maps_errors_correctly() {
        let temp_dir = TempDir::new().unwrap();
        let _repo = git2::Repository::init(temp_dir.path()).unwrap();

        let handle = git_rewind_git::repository::discover_from(temp_dir.path()).unwrap();
        let invalid_id = CommitId("0123456789abcdef0123456789abcdef01234567".to_string());

        let service = AppService::new();
        let result = service.inspect_commit(&handle, invalid_id);

        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Git(_) => {} // OK: wrapped git error
            _ => panic!("Expected Git backend error wrapped in AppError"),
        }
    }
}
