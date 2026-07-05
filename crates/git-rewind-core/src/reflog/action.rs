use std::fmt;

/// Represents a recognized Git reflog action.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ReflogAction {
    /// Commit action
    Commit,
    /// Checkout action
    Checkout,
    /// Reset action
    Reset,
    /// Merge action
    Merge,
    /// Rebase action
    Rebase,
    /// Cherry-pick action
    CherryPick,
    /// Branch creation action
    BranchCreate,
    /// Branch deletion action
    BranchDelete,
    /// Pull action
    Pull,
    /// Clone action
    Clone,
    /// An unrecognized reflog action, storing the original text.
    Unknown(String),
}

impl From<&str> for ReflogAction {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "commit" | "commit (initial)" => ReflogAction::Commit,
            "checkout" => ReflogAction::Checkout,
            "reset" => ReflogAction::Reset,
            "merge" => ReflogAction::Merge,
            "rebase" => ReflogAction::Rebase,
            "cherry-pick" | "cherrypick" => ReflogAction::CherryPick,
            "branch: created" | "branch: create" | "branch-create" => ReflogAction::BranchCreate,
            "branch: deleted" | "branch: delete" | "branch-delete" => ReflogAction::BranchDelete,
            "pull" => ReflogAction::Pull,
            "clone" => ReflogAction::Clone,
            _ => ReflogAction::Unknown(s.to_string()),
        }
    }
}

impl AsRef<str> for ReflogAction {
    fn as_ref(&self) -> &str {
        match self {
            ReflogAction::Commit => "commit",
            ReflogAction::Checkout => "checkout",
            ReflogAction::Reset => "reset",
            ReflogAction::Merge => "merge",
            ReflogAction::Rebase => "rebase",
            ReflogAction::CherryPick => "cherry-pick",
            ReflogAction::BranchCreate => "branch-create",
            ReflogAction::BranchDelete => "branch-delete",
            ReflogAction::Pull => "pull",
            ReflogAction::Clone => "clone",
            ReflogAction::Unknown(s) => s.as_str(),
        }
    }
}

impl fmt::Display for ReflogAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_parsing() {
        assert_eq!(ReflogAction::from("commit"), ReflogAction::Commit);
        assert_eq!(ReflogAction::from("COMMIT"), ReflogAction::Commit);
        assert_eq!(ReflogAction::from("commit (initial)"), ReflogAction::Commit);
        assert_eq!(ReflogAction::from("checkout"), ReflogAction::Checkout);
        assert_eq!(ReflogAction::from("reset"), ReflogAction::Reset);
        assert_eq!(ReflogAction::from("merge"), ReflogAction::Merge);
        assert_eq!(ReflogAction::from("rebase"), ReflogAction::Rebase);
        assert_eq!(ReflogAction::from("pull"), ReflogAction::Pull);
        assert_eq!(ReflogAction::from("clone"), ReflogAction::Clone);
    }

    #[test]
    fn test_unknown_action() {
        let raw = "custom-action-123";
        let parsed = ReflogAction::from(raw);
        assert_eq!(parsed, ReflogAction::Unknown("custom-action-123".to_string()));
        assert_eq!(parsed.to_string(), "custom-action-123");
    }

    #[test]
    fn test_display_formatting() {
        assert_eq!(ReflogAction::Commit.to_string(), "commit");
        assert_eq!(ReflogAction::Checkout.to_string(), "checkout");
        assert_eq!(ReflogAction::Reset.to_string(), "reset");
    }

    #[test]
    fn test_as_ref_str() {
        assert_eq!(ReflogAction::Commit.as_ref(), "commit");
        let unknown = ReflogAction::Unknown("my-action".to_string());
        assert_eq!(unknown.as_ref(), "my-action");
    }

    #[test]
    fn test_unknown_lossless_roundtrip() {
        let original = ReflogAction::Unknown("custom action".to_string());
        let display = original.to_string();
        let parsed = ReflogAction::from(display.as_str());
        assert_eq!(parsed, original);
    }
}
