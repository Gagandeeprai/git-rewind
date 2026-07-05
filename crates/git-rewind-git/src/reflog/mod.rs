mod mapper;

use crate::error::GitError;
use crate::repository::RepositoryHandle;
use git_rewind_core::reflog::ReflogEntry;

/// Reads the HEAD reflog and returns its entries in reflog order (newest first).
///
/// If there is no reflog for HEAD (e.g., in a newly initialized repository with no commits),
/// this function returns an empty vector.
pub fn read_reflog(repository: &RepositoryHandle) -> Result<Vec<ReflogEntry>, GitError> {
    let repo = repository.repository();

    let reflog = match repo.reflog("HEAD") {
        Ok(r) => r,
        Err(e) if e.code() == git2::ErrorCode::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(GitError::Git(e)),
    };

    let mut entries = Vec::new();
    let count = reflog.len();

    // Iterate through reflog entries in index order.
    // Index 0 represents the newest entry, matching Git newest-first reflog output.
    for i in 0..count {
        if let Some(entry) = reflog.get(i) {
            let parsed = mapper::parse_entry(i, &entry)?;
            entries.push(parsed);
        }
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use git_rewind_core::reflog::{ReflogAction, ReflogIndex};
    use git2::{Oid, Signature};
    use std::fs;
    use tempfile::TempDir;

    fn test_signature() -> Signature<'static> {
        Signature::now("Test User", "test@example.com").unwrap()
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
    fn test_read_reflog_empty_repo() {
        let temp_dir = TempDir::new().unwrap();
        let _repo = git2::Repository::init(temp_dir.path()).unwrap();
        let handle = crate::repository::discover_from(temp_dir.path()).unwrap();

        let reflog = read_reflog(&handle).unwrap();
        assert!(reflog.is_empty());
    }

    #[test]
    fn test_read_reflog_one_commit() {
        let temp_dir = TempDir::new().unwrap();
        let repo = git2::Repository::init(temp_dir.path()).unwrap();
        let oid = make_commit(&repo, "init repo");

        let handle = crate::repository::discover_from(temp_dir.path()).unwrap();
        let reflog = read_reflog(&handle).unwrap();

        assert_eq!(reflog.len(), 1);
        assert_eq!(reflog[0].index, ReflogIndex(0));
        assert_eq!(reflog[0].commit.to_string(), oid.to_string());
        assert_eq!(reflog[0].action, ReflogAction::Commit);
        assert_eq!(reflog[0].message, "init repo");
    }

    #[test]
    fn test_read_reflog_order_and_multiple_commits() {
        let temp_dir = TempDir::new().unwrap();
        let repo = git2::Repository::init(temp_dir.path()).unwrap();

        let oid1 = make_commit(&repo, "commit 1");
        let oid2 = make_commit(&repo, "commit 2");

        let handle = crate::repository::discover_from(temp_dir.path()).unwrap();
        let reflog = read_reflog(&handle).unwrap();

        assert_eq!(reflog.len(), 2);

        // Index 0 must be the newest commit (oid2, "commit 2")
        assert_eq!(reflog[0].index, ReflogIndex(0));
        assert_eq!(reflog[0].commit.to_string(), oid2.to_string());
        assert_eq!(reflog[0].message, "commit 2");

        // Index 1 must be the older commit (oid1, "commit 1")
        assert_eq!(reflog[1].index, ReflogIndex(1));
        assert_eq!(reflog[1].commit.to_string(), oid1.to_string());
        assert_eq!(reflog[1].message, "commit 1");
    }

    #[test]
    fn test_custom_actions_and_colon_splits() {
        let temp_dir = TempDir::new().unwrap();
        let repo = git2::Repository::init(temp_dir.path()).unwrap();
        let oid = make_commit(&repo, "commit: first commit");

        let sig = test_signature();

        {
            let mut reflog = repo.reflog("HEAD").unwrap();

            // Append checkout action
            reflog.append(oid, &sig, Some("checkout: moving from main to feature")).unwrap();

            // Append reset action
            reflog.append(oid, &sig, Some("reset: moving to HEAD~1")).unwrap();

            // Append unknown action
            reflog.append(oid, &sig, Some("custom_action: hello there")).unwrap();

            // Append multi-colon message
            reflog.append(oid, &sig, Some("commit: fix parser: handle edge case")).unwrap();

            reflog.write().unwrap();
        }

        let handle = crate::repository::discover_from(temp_dir.path()).unwrap();
        let reflog = read_reflog(&handle).unwrap();

        // Expected count: 1 (original commit) + 4 (appends) = 5
        assert_eq!(reflog.len(), 5);

        // Newest entry (index 0) is the multi-colon message
        assert_eq!(reflog[0].index, ReflogIndex(0));
        assert_eq!(reflog[0].action, ReflogAction::Commit);
        assert_eq!(reflog[0].message, "fix parser: handle edge case");

        // Index 1 is the custom action
        assert_eq!(reflog[1].index, ReflogIndex(1));
        assert_eq!(reflog[1].action, ReflogAction::Unknown("custom_action".to_string()));
        assert_eq!(reflog[1].message, "hello there");

        // Index 2 is reset
        assert_eq!(reflog[2].index, ReflogIndex(2));
        assert_eq!(reflog[2].action, ReflogAction::Reset);
        assert_eq!(reflog[2].message, "moving to HEAD~1");

        // Index 3 is checkout
        assert_eq!(reflog[3].index, ReflogIndex(3));
        assert_eq!(reflog[3].action, ReflogAction::Checkout);
        assert_eq!(reflog[3].message, "moving from main to feature");
    }
}
