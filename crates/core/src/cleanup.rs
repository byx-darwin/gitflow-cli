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

/// Select the PRs that `--merged` is allowed to clean up.
///
/// [`State`] collapses `MERGED` into `Closed`, so `state == Closed` cannot tell a
/// merged PR from one closed **without** merging; filtering on it made `--merged`
/// delete branches still holding unmerged work. `merged_at` is the real signal, but
/// a `None` is ambiguous — platforms that never populate the field also report
/// `None` for genuinely merged PRs. So the field is trusted only once at least one
/// row carries it, otherwise this falls back to the previous looser behaviour
/// instead of silently matching nothing.
#[must_use]
pub fn select_merge_candidates(prs: &[PrData]) -> Vec<&PrData> {
    if prs.iter().any(|pr| pr.merged_at.is_some()) {
        prs.iter().filter(|pr| pr.merged_at.is_some()).collect()
    } else {
        prs.iter().filter(|pr| pr.state == State::Closed).collect()
    }
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

/// Service for coordinating PR cleanup operations.
///
/// Orchestrates safety checks, git operations, and worktree handling.
#[derive(Debug)]
pub struct CleanupService;

impl CleanupService {
    /// Clean up a single PR's branches and optionally its worktree.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Safety checks fail
    /// - Git operations fail
    /// - Worktree operations fail
    pub async fn cleanup_single_pr(
        provider: &dyn crate::pr::PrProvider,
        pr_number: u64,
        args: &CleanupArgs,
    ) -> crate::Result<CleanupResult> {
        // 1. Fetch PR data
        let pr = provider.view(pr_number).await?;

        // 2. Get current branch
        let current_branch = get_current_branch().await?;

        // 3. Safety checks
        check_safety(&pr, &current_branch, args.force)?;

        // 4. Delete remote branch (if requested and not dry-run)
        let mut remote_deleted = false;
        if args.remote && !args.dry_run {
            if let Err(e) = crate::git_ops::delete_remote_branch(&pr.head_branch).await {
                // Log error but continue (branch might not exist)
                tracing::warn!("Failed to delete remote branch: {}", e);
            } else {
                remote_deleted = true;
            }
        }

        // 5. Delete local branch (if requested and not dry-run)
        let mut local_deleted = false;
        if args.local && !args.dry_run {
            crate::git_ops::delete_local_branch(&pr.head_branch, args.force).await?;
            local_deleted = true;
        }

        // 6. Handle worktree
        let mut worktree_exited = false;
        let mut worktree_removed = false;

        if crate::git_ops::is_in_worktree().await? && !args.dry_run {
            crate::git_ops::exit_worktree().await?;
            worktree_exited = true;

            if let Some(ref worktree_path) = args.worktree {
                crate::git_ops::remove_worktree(worktree_path).await?;
                worktree_removed = true;
            }
        }

        Ok(CleanupResult {
            pr_number: pr.number,
            pr_title: pr.title,
            branch: pr.head_branch,
            remote_deleted,
            local_deleted,
            worktree_exited,
            worktree_removed,
            dry_run: args.dry_run,
            error: None,
        })
    }

    /// Clean up multiple PRs by number.
    ///
    /// Continues on individual failures and collects all results.
    ///
    /// # Errors
    ///
    /// Returns an error only if the provider call itself fails.
    /// Individual PR cleanup failures are captured in the results.
    pub async fn cleanup(
        provider: &dyn crate::pr::PrProvider,
        args: &CleanupArgs,
    ) -> crate::Result<Vec<CleanupResult>> {
        let mut results = Vec::new();

        for &pr_number in &args.numbers {
            match Self::cleanup_single_pr(provider, pr_number, args).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    // Capture error in result instead of failing the whole batch
                    results.push(CleanupResult {
                        pr_number,
                        pr_title: String::new(),
                        branch: String::new(),
                        remote_deleted: false,
                        local_deleted: false,
                        worktree_exited: false,
                        worktree_removed: false,
                        dry_run: args.dry_run,
                        error: Some(e.to_string()),
                    });
                }
            }
        }

        Ok(results)
    }

    /// Clean up all merged PRs.
    ///
    /// # Errors
    ///
    /// Returns an error if listing PRs fails.
    pub async fn cleanup_merged(
        provider: &dyn crate::pr::PrProvider,
        args: &CleanupArgs,
    ) -> crate::Result<Vec<CleanupResult>> {
        Self::cleanup_filtered(provider, args, true).await
    }

    /// List closed PRs and clean up those matching `require_merged`.
    ///
    /// `require_merged = false` is the `--closed` path and must keep matching PRs
    /// that were closed **without** merging; that is precisely what it exists to
    /// clean up.
    async fn cleanup_filtered(
        provider: &dyn crate::pr::PrProvider,
        args: &CleanupArgs,
        require_merged: bool,
    ) -> crate::Result<Vec<CleanupResult>> {
        let prs = provider
            .list(crate::pr::ListPrArgs {
                state: Some(State::Closed),
                limit: None,
            })
            .await?;

        let targets: Vec<&PrData> = if require_merged {
            select_merge_candidates(&prs)
        } else {
            prs.iter().filter(|pr| pr.state == State::Closed).collect()
        };

        let mut results = Vec::new();
        for pr in targets {
            let mut pr_args = args.clone();
            pr_args.numbers = vec![pr.number];

            match Self::cleanup_single_pr(provider, pr.number, &pr_args).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    results.push(CleanupResult {
                        pr_number: pr.number,
                        pr_title: pr.title.clone(),
                        branch: pr.head_branch.clone(),
                        remote_deleted: false,
                        local_deleted: false,
                        worktree_exited: false,
                        worktree_removed: false,
                        dry_run: args.dry_run,
                        error: Some(e.to_string()),
                    });
                }
            }
        }

        Ok(results)
    }

    /// Clean up all closed PRs.
    ///
    /// # Errors
    ///
    /// Returns an error if listing PRs fails.
    pub async fn cleanup_closed(
        provider: &dyn crate::pr::PrProvider,
        args: &CleanupArgs,
    ) -> crate::Result<Vec<CleanupResult>> {
        Self::cleanup_filtered(provider, args, false).await
    }
}

/// Get the current git branch name.
///
/// # Errors
///
/// Returns an error if the git command fails or HEAD is detached.
async fn get_current_branch() -> crate::Result<String> {
    let output = tokio::process::Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .await
        .map_err(|e| crate::CoreError::App(format!("Failed to get current branch: {e}")))?;

    if !output.status.success() {
        return Err(crate::CoreError::App(
            "Failed to get current branch".to_string(),
        ));
    }

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if branch.is_empty() {
        return Err(crate::CoreError::App("HEAD is detached".to_string()));
    }

    Ok(branch)
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
            merged_at: Some(chrono::Utc::now()),
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
            merged_at: None,
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
            merged_at: None,
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
            merged_at: None,
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
            merged_at: None,
            base_branch: "main".to_string(),
            head_branch: "feature/x".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            url: "https://github.com/test/repo/pull/172".to_string(),
        };
        let result = check_safety(&pr, "main", true);
        assert!(result.is_ok());
    }

    fn pr_fixture(number: u64, state: State, merged: bool) -> PrData {
        PrData {
            number,
            title: format!("PR {number}"),
            body: None,
            state,
            draft: false,
            author: crate::types::UserSummary {
                login: "alice".to_string(),
                id: "1".to_string(),
            },
            base_branch: "main".to_string(),
            head_branch: format!("feature/{number}"),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            merged_at: if merged {
                Some(chrono::Utc::now())
            } else {
                None
            },
            url: format!("https://github.com/test/repo/pull/{number}"),
        }
    }

    fn picked(prs: &[&PrData]) -> Vec<u64> {
        prs.iter().map(|pr| pr.number).collect()
    }

    #[test]
    fn test_should_exclude_closed_but_unmerged_from_merged_cleanup() {
        // #201 was closed without merging; once the platform reports merged_at for
        // any row, --merged must not reach into it and delete unmerged work.
        let prs = vec![
            pr_fixture(200, State::Closed, true),
            pr_fixture(201, State::Closed, false),
        ];
        assert_eq!(picked(&select_merge_candidates(&prs)), vec![200]);
    }

    #[test]
    fn test_should_fall_back_to_closed_state_when_merged_at_never_reported() {
        // A platform that omits merged_at yields None even for merged PRs, so
        // trusting None here would silently clean nothing.
        let prs = vec![
            pr_fixture(300, State::Closed, false),
            pr_fixture(301, State::Closed, false),
        ];
        assert_eq!(picked(&select_merge_candidates(&prs)), vec![300, 301]);
    }

    #[test]
    fn test_should_keep_open_pr_out_of_merge_candidates() {
        let prs = vec![
            pr_fixture(400, State::Closed, true),
            pr_fixture(401, State::Open, false),
        ];
        assert_eq!(picked(&select_merge_candidates(&prs)), vec![400]);
    }
}
