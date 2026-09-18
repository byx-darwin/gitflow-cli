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
    /// Skip the confirmation prompt for `--merged` / `--closed` cleanups.
    ///
    /// Has no effect on the explicit-PR-numbers path (`gf pr cleanup 172 173`),
    /// which never prompts: the user already enumerated exactly what to delete.
    pub yes: bool,
}

/// The set of PRs a batch cleanup would act on, computed **before** any
/// deletion happens.
///
/// Splitting "work out what would be deleted" from "delete it" gives the CLI
/// a place to insert a confirmation prompt between the two. Nothing in
/// [`CleanupService::plan_merged`] or [`CleanupService::plan_closed`] may call
/// [`CleanupService::cleanup_single_pr`] or any function under
/// [`crate::git_ops`] — that is the safety property the whole split exists
/// to guarantee.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct CleanupPlan {
    /// PRs that would be cleaned up.
    pub targets: Vec<PrData>,
    /// Whether the underlying PR list was truncated at the platform limit.
    ///
    /// When `true`, more closed PRs exist beyond what this plan covers —
    /// the repository has more than [`crate::paging::DEFAULT_LIST_LIMIT`]
    /// closed PRs and only the first page was scanned.
    pub truncated: bool,
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
    /// Implemented as [`Self::plan_merged`] immediately followed by
    /// [`Self::execute_plan`]. Kept as a single call for callers that don't
    /// need to insert a confirmation step between planning and execution.
    ///
    /// # Errors
    ///
    /// Returns an error if listing PRs fails.
    pub async fn cleanup_merged(
        provider: &dyn crate::pr::PrProvider,
        args: &CleanupArgs,
    ) -> crate::Result<Vec<CleanupResult>> {
        let plan = Self::plan_merged(provider, args).await?;
        Self::execute_plan(provider, args, &plan).await
    }

    /// Clean up all closed PRs.
    ///
    /// Implemented as [`Self::plan_closed`] immediately followed by
    /// [`Self::execute_plan`]. Kept as a single call for callers that don't
    /// need to insert a confirmation step between planning and execution.
    ///
    /// # Errors
    ///
    /// Returns an error if listing PRs fails.
    pub async fn cleanup_closed(
        provider: &dyn crate::pr::PrProvider,
        args: &CleanupArgs,
    ) -> crate::Result<Vec<CleanupResult>> {
        let plan = Self::plan_closed(provider, args).await?;
        Self::execute_plan(provider, args, &plan).await
    }

    /// Work out which merged PRs `--merged` would clean up, without deleting
    /// anything.
    ///
    /// # Errors
    ///
    /// Returns an error if listing PRs fails.
    pub async fn plan_merged(
        provider: &dyn crate::pr::PrProvider,
        _args: &CleanupArgs,
    ) -> crate::Result<CleanupPlan> {
        Self::plan_filtered(provider, true).await
    }

    /// Work out which closed PRs `--closed` would clean up, without deleting
    /// anything.
    ///
    /// # Errors
    ///
    /// Returns an error if listing PRs fails.
    pub async fn plan_closed(
        provider: &dyn crate::pr::PrProvider,
        _args: &CleanupArgs,
    ) -> crate::Result<CleanupPlan> {
        Self::plan_filtered(provider, false).await
    }

    /// List closed PRs and select those matching `require_merged`, without
    /// deleting anything.
    ///
    /// `require_merged = false` is the `--closed` path and must keep matching PRs
    /// that were closed **without** merging; that is precisely what it exists to
    /// clean up.
    async fn plan_filtered(
        provider: &dyn crate::pr::PrProvider,
        require_merged: bool,
    ) -> crate::Result<CleanupPlan> {
        let paged = provider
            .list(crate::pr::ListPrArgs {
                state: Some(State::Closed),
                limit: None,
            })
            .await?;

        let targets: Vec<PrData> = if require_merged {
            select_merge_candidates(&paged.items)
                .into_iter()
                .cloned()
                .collect()
        } else {
            paged
                .items
                .iter()
                .filter(|pr| pr.state == State::Closed)
                .cloned()
                .collect()
        };

        Ok(CleanupPlan {
            targets,
            truncated: paged.truncated,
        })
    }

    /// Execute an already-computed [`CleanupPlan`].
    ///
    /// # Errors
    ///
    /// Individual PR failures are captured in the results rather than
    /// aborting the batch; this only returns an error if it cannot continue.
    pub async fn execute_plan(
        provider: &dyn crate::pr::PrProvider,
        args: &CleanupArgs,
        plan: &CleanupPlan,
    ) -> crate::Result<Vec<CleanupResult>> {
        let mut results = Vec::new();
        for pr in &plan.targets {
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

    /// A `PrProvider` stub that records every method invocation so tests can
    /// assert planning never reaches deletion-adjacent calls (`view`, which
    /// `cleanup_single_pr` calls as its very first step).
    #[derive(Debug)]
    struct RecordingProvider {
        prs: Vec<PrData>,
        truncated: bool,
        calls: std::sync::Mutex<Vec<&'static str>>,
    }

    impl RecordingProvider {
        fn new(prs: Vec<PrData>, truncated: bool) -> Self {
            Self {
                prs,
                truncated,
                calls: std::sync::Mutex::new(Vec::new()),
            }
        }

        fn record(&self, name: &'static str) {
            self.calls.lock().expect("lock calls").push(name);
        }

        fn call_count(&self, name: &str) -> usize {
            self.calls
                .lock()
                .expect("lock calls")
                .iter()
                .filter(|call| **call == name)
                .count()
        }
    }

    fn unsupported<T>(op: &str) -> crate::Result<T> {
        Err(crate::CoreError::App(format!(
            "RecordingProvider does not support '{op}' in tests"
        )))
    }

    #[async_trait::async_trait]
    impl crate::pr::PrProvider for RecordingProvider {
        async fn create(&self, _args: crate::pr::CreatePrArgs) -> crate::Result<PrData> {
            self.record("create");
            unsupported("create")
        }

        async fn list(
            &self,
            _args: crate::pr::ListPrArgs,
        ) -> crate::Result<crate::paging::Paged<PrData>> {
            self.record("list");
            Ok(crate::paging::Paged {
                items: self.prs.clone(),
                truncated: self.truncated,
                limit: crate::paging::DEFAULT_LIST_LIMIT,
                total_count: None,
            })
        }

        async fn view(&self, number: u64) -> crate::Result<PrData> {
            self.record("view");
            self.prs
                .iter()
                .find(|pr| pr.number == number)
                .cloned()
                .ok_or_else(|| crate::CoreError::App(format!("pr #{number} not found")))
        }

        async fn close(&self, _number: u64) -> crate::Result<PrData> {
            self.record("close");
            unsupported("close")
        }

        async fn reopen(&self, _number: u64) -> crate::Result<PrData> {
            self.record("reopen");
            unsupported("reopen")
        }

        async fn comment(
            &self,
            _number: u64,
            _body: &str,
        ) -> crate::Result<crate::types::CommentData> {
            self.record("comment");
            unsupported("comment")
        }

        async fn merge(
            &self,
            _number: u64,
            _strategy: Option<crate::types::MergeStrategy>,
            _auto: bool,
        ) -> crate::Result<crate::types::MergeResult> {
            self.record("merge");
            unsupported("merge")
        }

        async fn checkout(&self, _number: u64) -> crate::Result<()> {
            self.record("checkout");
            unsupported("checkout")
        }

        async fn mark_ready(&self, _number: u64) -> crate::Result<PrData> {
            self.record("mark_ready");
            unsupported("mark_ready")
        }

        async fn mark_wip(&self, _number: u64) -> crate::Result<PrData> {
            self.record("mark_wip");
            unsupported("mark_wip")
        }

        async fn sync_branch(&self, _number: u64) -> crate::Result<()> {
            self.record("sync_branch");
            unsupported("sync_branch")
        }

        async fn diff(&self, _number: u64) -> crate::Result<String> {
            self.record("diff");
            unsupported("diff")
        }

        async fn patch(&self, _number: u64) -> crate::Result<String> {
            self.record("patch");
            unsupported("patch")
        }

        async fn default_branch(&self) -> crate::Result<String> {
            self.record("default_branch");
            unsupported("default_branch")
        }
    }

    fn cleanup_args_for_plan() -> CleanupArgs {
        CleanupArgs {
            numbers: vec![],
            merged: true,
            closed: false,
            worktree: None,
            remote: false,
            local: false,
            force: false,
            dry_run: true,
            yes: false,
        }
    }

    #[tokio::test]
    async fn test_should_carry_truncated_flag_into_cleanup_plan() {
        let provider = RecordingProvider::new(
            vec![pr_fixture(500, State::Closed, true)],
            true, // Paged { truncated: true, .. }
        );
        let args = cleanup_args_for_plan();

        let plan = CleanupService::plan_merged(&provider, &args)
            .await
            .expect("plan_merged should succeed");

        assert!(plan.truncated);
    }

    #[tokio::test]
    async fn test_should_not_delete_anything_while_planning() {
        // Mixed merged/closed-without-merge PRs; only the shape of `list` matters
        // here since the assertion is about what planning *calls*, not its result.
        let provider = RecordingProvider::new(
            vec![
                pr_fixture(600, State::Closed, true),
                pr_fixture(601, State::Closed, false),
            ],
            false,
        );
        let args = cleanup_args_for_plan();

        let _plan = CleanupService::plan_merged(&provider, &args)
            .await
            .expect("plan_merged should succeed");

        // `cleanup_single_pr` — the only path that can reach a deletion call —
        // always starts by calling `view`. If planning never invoked `view`,
        // it cannot have deleted anything either.
        assert_eq!(
            provider.call_count("view"),
            0,
            "planning must not touch any per-PR cleanup path"
        );
        assert_eq!(provider.call_count("list"), 1);
    }

    #[tokio::test]
    async fn test_should_select_only_merged_prs_in_merged_plan() {
        let provider = RecordingProvider::new(
            vec![
                pr_fixture(700, State::Closed, true),
                pr_fixture(701, State::Closed, false),
            ],
            false,
        );
        let args = cleanup_args_for_plan();

        let plan = CleanupService::plan_merged(&provider, &args)
            .await
            .expect("plan_merged should succeed");

        let numbers: Vec<u64> = plan.targets.iter().map(|pr| pr.number).collect();
        assert_eq!(numbers, vec![700]);
    }

    #[tokio::test]
    async fn test_should_produce_same_results_through_plan_and_execute() {
        // Non-protected, non-checked-out branch names so `check_safety` passes;
        // `dry_run: true` guarantees no real git mutation regardless of
        // `remote`/`local`, so this is safe to run against the real repo.
        let prs = vec![
            pr_fixture(800, State::Closed, true),
            pr_fixture(801, State::Closed, true),
        ];
        let args = cleanup_args_for_plan();

        let provider_a = RecordingProvider::new(prs.clone(), false);
        let combined = CleanupService::cleanup_merged(&provider_a, &args)
            .await
            .expect("cleanup_merged should succeed");

        let provider_b = RecordingProvider::new(prs, false);
        let plan = CleanupService::plan_merged(&provider_b, &args)
            .await
            .expect("plan_merged should succeed");
        let staged = CleanupService::execute_plan(&provider_b, &args, &plan)
            .await
            .expect("execute_plan should succeed");

        let combined_numbers: Vec<u64> = combined.iter().map(|r| r.pr_number).collect();
        let staged_numbers: Vec<u64> = staged.iter().map(|r| r.pr_number).collect();
        assert_eq!(combined_numbers, staged_numbers);

        // Compare the fuller result shape, not just `pr_number`/`dry_run` (the
        // latter is `args.dry_run` on both paths and proves nothing). This is
        // the evidence that the plan/execute split preserved behaviour.
        let result_tuple = |r: &CleanupResult| {
            (
                r.pr_number,
                r.error.clone(),
                r.remote_deleted,
                r.local_deleted,
                r.worktree_removed,
            )
        };
        let combined_tuples: Vec<_> = combined.iter().map(result_tuple).collect();
        let staged_tuples: Vec<_> = staged.iter().map(result_tuple).collect();
        assert_eq!(combined_tuples, staged_tuples);
    }
}
