//! PR cleanup domain types and service.
//!
//! Provides argument and result types for the `gf pr cleanup` command,
//! plus the [`CleanupService`] that coordinates branch and worktree cleanup.

use serde::{Deserialize, Serialize};

use crate::{pr::PrData, types::State};

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

/// Check if a branch name matches common protected branch patterns.
///
/// Protected branches include: `main`, `master`, `develop`, and `release/*`.
///
/// This is a local check — Phase 2 may add remote branch protection queries.
#[must_use]
pub fn is_protected_branch(branch: &str) -> bool {
    matches!(branch, "main" | "master" | "develop") || branch.starts_with("release/")
}

/// Perform safety checks before cleaning up a PR.
///
/// Checks:
/// 1. PR status (must be merged or closed, unless `force` is true)
/// 2. Branch protection (hard reject — cannot be overridden)
/// 3. Current branch (cannot delete currently checked-out branch)
///
/// # Errors
///
/// Returns an error if any safety check fails.
pub fn check_safety(pr: &PrData, current_branch: &str, force: bool) -> crate::Result<()> {
    // 1. Check PR status
    if !force && pr.state != State::Closed {
        return Err(crate::CoreError::App(format!(
            "PR #{} 尚未合并或关闭。使用 --force 强制清理。",
            pr.number
        )));
    }

    // 2. Check protected branch (hard reject)
    if is_protected_branch(&pr.head_branch) {
        return Err(crate::CoreError::App(format!(
            "分支 '{}' 受保护，拒绝删除",
            pr.head_branch
        )));
    }

    // 3. Check current branch (hard reject)
    if pr.head_branch == current_branch {
        return Err(crate::CoreError::App(format!(
            "无法删除当前检出的分支 '{}'",
            pr.head_branch
        )));
    }

    Ok(())
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

    #[test]
    fn test_should_identify_main_as_protected() {
        assert!(is_protected_branch("main"));
    }

    #[test]
    fn test_should_identify_master_as_protected() {
        assert!(is_protected_branch("master"));
    }

    #[test]
    fn test_should_identify_develop_as_protected() {
        assert!(is_protected_branch("develop"));
    }

    #[test]
    fn test_should_identify_release_branches_as_protected() {
        assert!(is_protected_branch("release/1.0"));
        assert!(is_protected_branch("release/v2.0.0"));
    }

    #[test]
    fn test_should_not_identify_feature_branch_as_protected() {
        assert!(!is_protected_branch("feature/x"));
        assert!(!is_protected_branch("bugfix/123"));
    }

    #[test]
    fn test_should_allow_cleanup_of_merged_pr() {
        let pr = PrData {
            number: 172,
            title: "Add feature".to_string(),
            body: None,
            state: State::Closed,
            draft: false,
            author: crate::types::UserSummary {
                login: "alice".to_string(),
                id: "1".to_string(),
            },
            base_branch: "main".to_string(),
            head_branch: "feature/x".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            url: "https://github.com/test/repo/pull/172".to_string(),
        };
        let result = check_safety(&pr, "main", false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_should_refuse_to_delete_protected_branch() {
        let pr = PrData {
            number: 172,
            title: "Update main".to_string(),
            body: None,
            state: State::Closed,
            draft: false,
            author: crate::types::UserSummary {
                login: "alice".to_string(),
                id: "1".to_string(),
            },
            base_branch: "main".to_string(),
            head_branch: "main".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            url: "https://github.com/test/repo/pull/172".to_string(),
        };
        let result = check_safety(&pr, "develop", false);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("受保护"));
    }

    #[test]
    fn test_should_refuse_to_delete_current_branch() {
        let pr = PrData {
            number: 172,
            title: "Add feature".to_string(),
            body: None,
            state: State::Closed,
            draft: false,
            author: crate::types::UserSummary {
                login: "alice".to_string(),
                id: "1".to_string(),
            },
            base_branch: "main".to_string(),
            head_branch: "feature/x".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            url: "https://github.com/test/repo/pull/172".to_string(),
        };
        let result = check_safety(&pr, "feature/x", false);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("当前检出"));
    }

    #[test]
    fn test_should_require_merged_or_closed_state() {
        let pr = PrData {
            number: 172,
            title: "Add feature".to_string(),
            body: None,
            state: State::Open,
            draft: false,
            author: crate::types::UserSummary {
                login: "alice".to_string(),
                id: "1".to_string(),
            },
            base_branch: "main".to_string(),
            head_branch: "feature/x".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            url: "https://github.com/test/repo/pull/172".to_string(),
        };
        let result = check_safety(&pr, "main", false);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("尚未合并或关闭"));
    }

    #[test]
    fn test_should_allow_unmerged_pr_with_force() {
        let pr = PrData {
            number: 172,
            title: "Add feature".to_string(),
            body: None,
            state: State::Open,
            draft: false,
            author: crate::types::UserSummary {
                login: "alice".to_string(),
                id: "1".to_string(),
            },
            base_branch: "main".to_string(),
            head_branch: "feature/x".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            url: "https://github.com/test/repo/pull/172".to_string(),
        };
        let result = check_safety(&pr, "main", true);
        assert!(result.is_ok());
    }
}
