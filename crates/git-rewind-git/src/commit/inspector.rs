use super::model::{CommitAuthor, CommitDetails};
use crate::error::GitError;
use crate::repository::RepositoryHandle;
use git_rewind_core::reflog::{CommitId, ReflogTimestamp};
use std::time::{Duration, SystemTime};

/// Inspects the metadata of a specific commit by its ID.
///
/// Looks up the commit in the repository and returns an owned representation
/// of its details, shielding higher layers from `git2` structures and lifetimes.
pub fn inspect(repository: &RepositoryHandle, id: CommitId) -> Result<CommitDetails, GitError> {
    let repo = repository.repository();

    let oid = git2::Oid::from_str(&id.0).map_err(GitError::Git)?;

    let commit = repo.find_commit(oid)?;

    let author_sig = commit.author();
    let author = CommitAuthor {
        name: author_sig.name().unwrap_or("").to_string(),
        email: author_sig.email().unwrap_or("").to_string(),
    };

    let time_secs = commit.time().seconds();
    let system_time = if time_secs >= 0 {
        SystemTime::UNIX_EPOCH + Duration::from_secs(time_secs as u64)
    } else {
        SystemTime::UNIX_EPOCH
    };
    let timestamp = ReflogTimestamp(system_time);

    let message = commit.message().unwrap_or("").to_string();

    let mut parents = Vec::new();
    for parent in commit.parents() {
        parents.push(CommitId(parent.id().to_string()));
    }

    Ok(CommitDetails { id, message, author, timestamp, parents })
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{Oid, Signature};
    use std::fs;
    use tempfile::TempDir;

    fn test_signature() -> Signature<'static> {
        Signature::now("Inspector User", "inspector@example.com").unwrap()
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

    fn make_merge_commit(
        repo: &git2::Repository,
        message: &str,
        parent1: Oid,
        parent2: Oid,
    ) -> Oid {
        let sig = test_signature();
        let tree_id = repo.find_commit(parent1).unwrap().tree_id();
        let tree = repo.find_tree(tree_id).unwrap();

        let p1 = repo.find_commit(parent1).unwrap();
        let p2 = repo.find_commit(parent2).unwrap();

        repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[&p1, &p2]).unwrap()
    }

    #[test]
    fn test_inspect_successful_lookup_and_mapping() {
        let temp_dir = TempDir::new().unwrap();
        let repo = git2::Repository::init(temp_dir.path()).unwrap();
        let commit_msg = "feat: add inspection capability";
        let oid = make_commit(&repo, commit_msg);

        let handle = crate::repository::discover_from(temp_dir.path()).unwrap();
        let details = inspect(&handle, CommitId(oid.to_string())).unwrap();

        assert_eq!(details.id, CommitId(oid.to_string()));
        assert_eq!(details.message, commit_msg);
        assert_eq!(details.author.name, "Inspector User");
        assert_eq!(details.author.email, "inspector@example.com");
        assert!(details.timestamp.0 > SystemTime::UNIX_EPOCH);
    }

    #[test]
    fn test_inspect_root_commit_has_no_parents() {
        let temp_dir = TempDir::new().unwrap();
        let repo = git2::Repository::init(temp_dir.path()).unwrap();
        let oid = make_commit(&repo, "initial commit");

        let handle = crate::repository::discover_from(temp_dir.path()).unwrap();
        let details = inspect(&handle, CommitId(oid.to_string())).unwrap();

        assert!(details.parents.is_empty());
    }

    #[test]
    fn test_inspect_merge_commit_multiple_parents() {
        let temp_dir = TempDir::new().unwrap();
        let repo = git2::Repository::init(temp_dir.path()).unwrap();

        let c1 = make_commit(&repo, "commit 1");

        // Create branch b and commit on it
        let commit1 = repo.find_commit(c1).unwrap();
        let branch_ref = repo.branch("branch-b", &commit1, false).unwrap();
        let branch_name = branch_ref.get().name().unwrap();

        // Switch to branch-b
        repo.set_head(branch_name).unwrap();
        let c2 = make_commit(&repo, "commit 2 on branch-b");

        // Switch back to main
        repo.set_head("refs/heads/main").unwrap();
        let c3 = make_commit(&repo, "commit 3 on main");

        // Merge c2 and c3
        let merge_oid = make_merge_commit(&repo, "merge branch-b into main", c3, c2);

        let handle = crate::repository::discover_from(temp_dir.path()).unwrap();
        let details = inspect(&handle, CommitId(merge_oid.to_string())).unwrap();

        assert_eq!(details.parents.len(), 2);
        assert_eq!(details.parents[0], CommitId(c3.to_string()));
        assert_eq!(details.parents[1], CommitId(c2.to_string()));
    }

    #[test]
    fn test_inspect_invalid_commit_id() {
        let temp_dir = TempDir::new().unwrap();
        let _repo = git2::Repository::init(temp_dir.path()).unwrap();

        let handle = crate::repository::discover_from(temp_dir.path()).unwrap();
        // A random, valid hex hash that doesn't exist in the repository
        let invalid_id = CommitId("0123456789abcdef0123456789abcdef01234567".to_string());

        let result = inspect(&handle, invalid_id);
        assert!(result.is_err());

        // A completely malformed hash format
        let malformed_id = CommitId("invalid-hash".to_string());
        let malformed_result = inspect(&handle, malformed_id);
        assert!(malformed_result.is_err());
    }
}
