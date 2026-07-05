use super::model::{ChangedFile, CommitDiff, FileChangeType};
use crate::error::GitError;
use crate::repository::RepositoryHandle;
use git_rewind_core::reflog::CommitId;

/// Computes the diff of a commit against its parent (or empty tree if root).
pub fn inspect(repository: &RepositoryHandle, commit_id: CommitId) -> Result<CommitDiff, GitError> {
    let repo = repository.repository();

    let oid = git2::Oid::from_str(&commit_id.0).map_err(GitError::Git)?;

    let commit = repo.find_commit(oid)?;
    let current_tree = commit.tree()?;

    // Determine the parent tree (if any)
    let parent_tree = if commit.parent_count() > 0 {
        let parent = commit.parent(0)?;
        Some(parent.tree()?)
    } else {
        None
    };

    // Compute diff
    let mut diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&current_tree), None)?;
    diff.find_similar(None)?;

    let mut files = Vec::new();
    for delta in diff.deltas() {
        let old_file = delta.old_file();
        let new_file = delta.new_file();

        let change = match delta.status() {
            git2::Delta::Added => FileChangeType::Added,
            git2::Delta::Deleted => FileChangeType::Deleted,
            git2::Delta::Modified => FileChangeType::Modified,
            git2::Delta::Renamed => FileChangeType::Renamed,
            git2::Delta::Copied => FileChangeType::Copied,
            git2::Delta::Typechange => FileChangeType::TypeChanged,
            _ => continue,
        };

        let path = match change {
            FileChangeType::Deleted => old_file.path().map(|p| p.to_path_buf()),
            _ => new_file.path().map(|p| p.to_path_buf()),
        };

        if let Some(p) = path {
            files.push(ChangedFile { path: p, change });
        }
    }

    Ok(CommitDiff { files })
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{Oid, Signature};
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    fn test_signature() -> Signature<'static> {
        Signature::now("Diff User", "diff@example.com").unwrap()
    }

    fn make_commit(
        repo: &git2::Repository,
        message: &str,
        files_to_add: &[(&str, Option<&str>)], // (path, content_option: None = delete)
    ) -> Oid {
        let mut index = repo.index().unwrap();
        let repo_root = repo.path().parent().unwrap();

        for &(file_path, content_opt) in files_to_add {
            let full_path = repo_root.join(file_path);
            if let Some(content) = content_opt {
                if let Some(parent) = full_path.parent() {
                    fs::create_dir_all(parent).unwrap();
                }
                fs::write(&full_path, content).unwrap();
                index.add_path(Path::new(file_path)).unwrap();
            } else {
                if full_path.exists() {
                    fs::remove_file(&full_path).unwrap();
                }
                index.remove_path(Path::new(file_path)).unwrap();
            }
        }
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
    fn test_inspect_diff_root_commit() {
        let temp_dir = TempDir::new().unwrap();
        let repo = git2::Repository::init(temp_dir.path()).unwrap();

        // Root commit adds a file
        let oid = make_commit(&repo, "initial", &[("a.txt", Some("file a"))]);

        let handle = crate::repository::discover_from(temp_dir.path()).unwrap();
        let diff = inspect(&handle, CommitId(oid.to_string())).unwrap();

        assert_eq!(diff.files.len(), 1);
        assert_eq!(diff.files[0].path, Path::new("a.txt"));
        assert_eq!(diff.files[0].change, FileChangeType::Added);
    }

    #[test]
    fn test_inspect_diff_normal_changes() {
        let temp_dir = TempDir::new().unwrap();
        let repo = git2::Repository::init(temp_dir.path()).unwrap();

        // Commit 1: Root commit
        let _oid1 =
            make_commit(&repo, "initial", &[("a.txt", Some("file a")), ("b.txt", Some("file b"))]);

        // Commit 2: Modify a.txt, delete b.txt, add c.txt
        let oid2 = make_commit(
            &repo,
            "modifications",
            &[("a.txt", Some("file a modified")), ("b.txt", None), ("c.txt", Some("file c"))],
        );

        let handle = crate::repository::discover_from(temp_dir.path()).unwrap();
        let diff = inspect(&handle, CommitId(oid2.to_string())).unwrap();

        assert_eq!(diff.files.len(), 3);

        let mut modified = false;
        let mut deleted = false;
        let mut added = false;

        for f in &diff.files {
            if f.path == Path::new("a.txt") {
                assert_eq!(f.change, FileChangeType::Modified);
                modified = true;
            } else if f.path == Path::new("b.txt") {
                assert_eq!(f.change, FileChangeType::Deleted);
                deleted = true;
            } else if f.path == Path::new("c.txt") {
                assert_eq!(f.change, FileChangeType::Added);
                added = true;
            }
        }

        assert!(modified);
        assert!(deleted);
        assert!(added);
    }

    #[test]
    fn test_inspect_diff_rename_detection() {
        let temp_dir = TempDir::new().unwrap();
        let repo = git2::Repository::init(temp_dir.path()).unwrap();

        // Commit 1: add a.txt
        let _oid1 = make_commit(&repo, "initial", &[("a.txt", Some("common file content"))]);

        // Commit 2: rename a.txt to b.txt
        let oid2 = make_commit(
            &repo,
            "rename",
            &[("a.txt", None), ("b.txt", Some("common file content"))],
        );

        let handle = crate::repository::discover_from(temp_dir.path()).unwrap();
        let diff = inspect(&handle, CommitId(oid2.to_string())).unwrap();

        assert_eq!(diff.files.len(), 1);
        assert_eq!(diff.files[0].path, Path::new("b.txt"));
        assert_eq!(diff.files[0].change, FileChangeType::Renamed);
    }

    #[test]
    fn test_inspect_diff_invalid_commit_id() {
        let temp_dir = TempDir::new().unwrap();
        let _repo = git2::Repository::init(temp_dir.path()).unwrap();

        let handle = crate::repository::discover_from(temp_dir.path()).unwrap();
        let invalid_id = CommitId("0123456789abcdef0123456789abcdef01234567".to_string());

        let result = inspect(&handle, invalid_id);
        assert!(result.is_err());
    }
}
