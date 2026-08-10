//! PR cleanup domain types and service.
//!
//! Provides argument and result types for the `gf pr cleanup` command,
//! plus the [`CleanupService`] that coordinates branch and worktree cleanup.

use serde::{Deserialize, Serialize};

/// Arguments for the `gf pr cleanup` command.
///
/// Supports cleanup by PR numbers, by status (`--merged`/`--closed`),
/// or a combination. The `numbers` field is mutually exclusive with
/// `merged`/`closed` flags.
#[derive(Debug, Clone)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "Configuration type with boolean flags"
)]
pub struct CleanupArgs {
    /// PR numbers to clean up (mutually exclusive with `merged`/`closed`).
    pub numbers: Vec<u64>,
    /// Clean up all merged PRs.
    pub merged: bool,
    /// Clean up all closed PRs.
    pub closed: bool,
    /// Remove the specified worktree path after cleanup.
    pub worktree: Option<String>,
    /// Delete remote branches.
    pub remote: bool,
    /// Delete local branches.
    pub local: bool,
    /// Force cleanup of unmerged branches.
    pub force: bool,
    /// Show what would be done without actually doing it.
    pub dry_run: bool,
    /// Skip interactive confirmation.
    pub yes: bool,
}

/// Result of cleaning up a single PR.
///
/// Tracks which operations succeeded and any errors encountered.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(
    clippy::struct_excessive_bools,
    reason = "Result type tracking multiple operation outcomes"
)]
pub struct CleanupResult {
    /// PR number.
    pub pr_number: u64,
    /// PR title.
    pub pr_title: String,
    /// Branch name that was cleaned up.
    pub branch: String,
    /// Whether the remote branch was deleted.
    pub remote_deleted: bool,
    /// Whether the local branch was deleted.
    pub local_deleted: bool,
    /// Whether the worktree was exited.
    pub worktree_exited: bool,
    /// Whether the worktree directory was removed.
    pub worktree_removed: bool,
    /// Whether this was a dry-run (no actual deletions).
    pub dry_run: bool,
    /// Error message if cleanup failed for this PR.
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_create_cleanup_args_with_defaults() {
        let args = CleanupArgs {
            numbers: vec![172],
            merged: false,
            closed: false,
            worktree: None,
            remote: true,
            local: true,
            force: false,
            dry_run: false,
            yes: false,
        };
        assert_eq!(args.numbers, vec![172]);
        assert!(args.remote);
        assert!(args.local);
        assert!(!args.force);
    }

    #[test]
    fn test_should_create_cleanup_args_for_batch() {
        let args = CleanupArgs {
            numbers: vec![172, 173, 174],
            merged: false,
            closed: false,
            worktree: None,
            remote: true,
            local: true,
            force: false,
            dry_run: false,
            yes: false,
        };
        assert_eq!(args.numbers.len(), 3);
    }

    #[test]
    fn test_should_create_cleanup_args_for_merged() {
        let args = CleanupArgs {
            numbers: vec![],
            merged: true,
            closed: false,
            worktree: None,
            remote: true,
            local: true,
            force: false,
            dry_run: false,
            yes: false,
        };
        assert!(args.merged);
        assert!(!args.closed);
    }

    #[test]
    fn test_should_serialize_cleanup_result() {
        let result = CleanupResult {
            pr_number: 172,
            pr_title: "Add feature".to_string(),
            branch: "feature/x".to_string(),
            remote_deleted: true,
            local_deleted: true,
            worktree_exited: false,
            worktree_removed: false,
            dry_run: false,
            error: None,
        };
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(json.contains("\"prNumber\":172"));
        assert!(json.contains("\"remoteDeleted\":true"));
    }

    #[test]
    fn test_should_serialize_cleanup_result_with_error() {
        let result = CleanupResult {
            pr_number: 175,
            pr_title: "Protected branch".to_string(),
            branch: "main".to_string(),
            remote_deleted: false,
            local_deleted: false,
            worktree_exited: false,
            worktree_removed: false,
            dry_run: false,
            error: Some("Branch 'main' is protected".to_string()),
        };
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(json.contains("\"error\":"));
        assert!(json.contains("protected"));
    }
}
