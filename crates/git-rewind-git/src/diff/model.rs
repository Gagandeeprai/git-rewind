use std::path::PathBuf;

/// Represents the type of changes made to a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileChangeType {
    /// File was created/added.
    Added,
    /// File was modified.
    Modified,
    /// File was deleted.
    Deleted,
    /// File was renamed.
    Renamed,
    /// File was copied.
    Copied,
    /// The type of file changed (e.g. symlink to regular file).
    TypeChanged,
}

/// Represents a single file changed in a commit diff.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChangedFile {
    /// The path of the changed file relative to repository root.
    pub path: PathBuf,
    /// The category of change.
    pub change: FileChangeType,
}

/// Represents the diff of a commit containing a list of changed files.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommitDiff {
    /// The list of changed files.
    pub files: Vec<ChangedFile>,
}
