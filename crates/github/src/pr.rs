//! GitHub Pull Request 提供者实现。
//!
//! 通过 `gh` CLI 实现 [`PrProvider`] trait，支持 Pull Request 的创建、列表、查看、
//! 关闭、合并、检出、草稿状态切换和分支同步。
//! 命令执行通过 [`CommandRunner`] 抽象，捕获 stdout 并解析 JSON。

use async_trait::async_trait;
use gitflow_core::{
    CoreError, Result,
    pr::{CreatePrArgs, ListPrArgs, PrData, PrProvider},
    types::{CommentData, MergeResult, MergeStrategy, State},
};
use tracing::debug;

use crate::{
    error::parse_gh_error,
    runner::{CommandRunner, RealCommandRunner},
};

/// `gh pr` 请求的 JSON 字段列表。
const PR_FIELDS: &str =
    "number,title,body,state,isDraft,author,baseRefName,headRefName,createdAt,updatedAt,url";

/// GitHub Pull Request 提供者，通过 `gh` CLI 操作。
///
/// 该结构体通过调用 `gh` CLI 实现 [`PrProvider`] trait 的所有方法，
/// 使上层命令能够以统一的方式操作 GitHub Pull Request。
///
/// 命令执行通过 [`CommandRunner`] 抽象，生产环境默认使用
/// [`RealCommandRunner`]，测试可注入自定义 runner 以模拟成功或失败场景。
///
/// # Examples
///
/// ```no_run
/// use gitflow_github::GitHubPrProvider;
///
/// let provider = GitHubPrProvider::new("octocat/hello-world");
/// ```
#[derive(Debug, Clone)]
pub struct GitHubPrProvider<R: CommandRunner = RealCommandRunner> {
    /// GitHub `owner/repo`，如 `"byx-darwin/gitflow-cli"`。
    repo: String,
    /// 用于执行 `gh` CLI 命令的 runner。
    runner: R,
}

impl GitHubPrProvider<RealCommandRunner> {
    /// 创建新的 GitHub Pull Request 提供者，使用真实的进程执行器。
    ///
    /// `repo` 格式为 `owner/repo`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> Self {
        Self {
            repo: repo.into(),
            runner: RealCommandRunner,
        }
    }

    /// Create a new provider from a shared [`Session`].
    ///
    /// This enables state reuse across multiple operations in workflow chains.
    #[must_use]
    pub fn with_session(session: &gitflow_core::Session) -> Self {
        Self {
            repo: session.repo.clone(),
            runner: RealCommandRunner,
        }
    }
}

impl<R: CommandRunner> GitHubPrProvider<R> {
    /// 使用自定义 [`CommandRunner`] 创建提供者。
    ///
    /// 主要用于测试，可注入模拟 runner 以控制 `gh` CLI 的输出。
    /// `repo` 格式为 `owner/repo`。
    #[must_use]
    pub fn with_runner(repo: impl Into<String>, runner: R) -> Self {
        Self {
            repo: repo.into(),
            runner,
        }
    }
}

#[async_trait]
impl<R: CommandRunner + 'static> PrProvider for GitHubPrProvider<R> {
    async fn create(&self, args: CreatePrArgs) -> Result<PrData> {
        let repo = args.repo.as_deref().unwrap_or(&self.repo);

        let mut cmd_args: Vec<&str> = vec![
            "pr",
            "create",
            "--repo",
            repo,
            "--title",
            &args.title,
            "--head",
            &args.head,
            "--base",
            &args.base,
        ];

        let final_body =
            gitflow_core::pr::format_closing_body(&args.body, &args.closes_issues, "Closes");

        if let Some(body) = &final_body {
            cmd_args.push("--body");
            cmd_args.push(body);
        }

        if args.draft {
            cmd_args.push("--draft");
        }

        debug!(
            repo = %self.repo,
            title = %args.title,
            head = %args.head,
            base = %args.base,
            "spawning `gh pr create`"
        );

        let output = self
            .runner
            .run("gh", &cmd_args)
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            return Err(parse_gh_error(&output.stderr).into());
        }

        // Parse the PR URL from stdout (format: https://github.com/owner/repo/pull/123)
        let stdout = String::from_utf8_lossy(&output.stdout);
        let pr_number = parse_pr_number_from_url(&stdout).ok_or_else(|| {
            CoreError::Platform(format!("Failed to parse PR URL from output: {stdout}"))
        })?;

        // Fetch full PR details via view
        self.view(pr_number).await
    }

    async fn list(&self, args: ListPrArgs) -> Result<Vec<PrData>> {
        let mut cmd_args: Vec<&str> = vec!["pr", "list", "--repo", &self.repo, "--json", PR_FIELDS];

        if let Some(state) = &args.state {
            cmd_args.push("--state");
            cmd_args.push(match state {
                State::Open => "open",
                State::Closed => "closed",
                State::All => "all",
            });
        }

        let limit_str = args.limit.map(|limit| limit.to_string());
        if let Some(ref limit) = limit_str {
            cmd_args.push("--limit");
            cmd_args.push(limit);
        }

        debug!(repo = %self.repo, "spawning `gh pr list`");

        let output = self
            .runner
            .run("gh", &cmd_args)
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            return Err(parse_gh_error(&output.stderr).into());
        }

        let prs: Vec<PrData> =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(prs)
    }

    async fn view(&self, number: u64) -> Result<PrData> {
        debug!(repo = %self.repo, number, "spawning `gh pr view`");

        let number_str = number.to_string();
        let output = self
            .runner
            .run(
                "gh",
                &[
                    "pr",
                    "view",
                    &number_str,
                    "--repo",
                    &self.repo,
                    "--json",
                    PR_FIELDS,
                ],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            return Err(parse_gh_error(&output.stderr).into());
        }

        let pr: PrData =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(pr)
    }

    /// 关闭指定编号的 PR。
    ///
    /// 调用 `gh pr close <number> --repo <repo>` 关闭 PR，
    /// 并返回更新后的完整 PR 数据。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在、已关闭或 `gh` CLI 调用失败时返回错误。
    async fn close(&self, number: u64) -> Result<PrData> {
        debug!(repo = %self.repo, number, "spawning `gh pr close`");

        let number_str = number.to_string();
        let output = self
            .runner
            .run("gh", &["pr", "close", &number_str, "--repo", &self.repo])
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            return Err(parse_gh_error(&output.stderr).into());
        }

        // Fetch updated PR details
        self.view(number).await
    }

    /// 重新打开指定编号的 PR。
    ///
    /// 调用 `gh pr reopen <number> --repo <repo>` 重新打开已关闭的 PR，
    /// 并返回更新后的完整 PR 数据。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在、未关闭或 `gh` CLI 调用失败时返回错误。
    async fn reopen(&self, number: u64) -> Result<PrData> {
        debug!(repo = %self.repo, number, "spawning `gh pr reopen`");

        let number_str = number.to_string();
        let output = self
            .runner
            .run("gh", &["pr", "reopen", &number_str, "--repo", &self.repo])
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            return Err(parse_gh_error(&output.stderr).into());
        }

        // Fetch updated PR details
        self.view(number).await
    }

    /// 在指定 PR 上添加评论。
    ///
    /// 调用 `gh api repos/{repo}/issues/{number}/comments -X POST` 创建评论，
    /// 直接从 POST 响应中解析新创建的评论数据。
    /// （PR 评论使用与 Issue 评论相同的 API 端点。）
    ///
    /// # Errors
    ///
    /// 当 PR 不存在、`body` 为空或 `gh` CLI 调用失败时返回错误。
    async fn comment(&self, number: u64, body: &str) -> Result<CommentData> {
        debug!(repo = %self.repo, number, "spawning `gh api` POST to create PR comment");

        let api_path = format!(
            "repos/{repo}/issues/{number}/comments",
            repo = self.repo,
            number = number
        );

        let body_field = format!("body={body}");
        let output = self
            .runner
            .run("gh", &["api", &api_path, "-X", "POST", "-f", &body_field])
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh api: {e}")))?;

        if !output.status.success() {
            return Err(parse_gh_error(&output.stderr).into());
        }

        let comment: crate::issue::GitHubCommentApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(comment.into())
    }

    /// 合并指定编号的 PR。
    ///
    /// 调用 `gh pr merge <number> --repo <repo>` 并根据 `strategy` 参数
    /// 添加 `--squash`、`--rebase` 或 `--merge` 标志。
    /// 未指定策略时使用 `--merge`（标准合并）。
    /// `auto` 为 `true` 时追加 `--auto`，由 GitHub 在必需检查通过后自动合并，
    /// 调用立即返回；此时 `merged` 为 `false`（排队不等于已合并）。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在、存在冲突无法合并或 `gh` CLI 调用失败时返回错误。
    async fn merge(
        &self,
        number: u64,
        strategy: Option<MergeStrategy>,
        auto: bool,
    ) -> Result<MergeResult> {
        debug!(repo = %self.repo, number, ?strategy, auto, "spawning `gh pr merge`");

        let number_str = number.to_string();
        let mut cmd_args: Vec<&str> = vec!["pr", "merge", &number_str, "--repo", &self.repo];

        match strategy {
            Some(MergeStrategy::Squash) => cmd_args.push("--squash"),
            Some(MergeStrategy::Rebase) => cmd_args.push("--rebase"),
            Some(MergeStrategy::Merge) | None => cmd_args.push("--merge"),
        }

        if auto {
            cmd_args.push("--auto");
        }

        let output = self
            .runner
            .run("gh", &cmd_args)
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            return Err(parse_gh_error(&output.stderr).into());
        }

        // `gh pr merge` outputs a human-readable message, not JSON.
        let message = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(MergeResult {
            merged: !auto,
            sha: None,
            message: Some(message),
        })
    }

    /// 在本地检出指定 PR 的分支。
    ///
    /// 调用 `gh pr checkout <number> --repo <repo>` 在本地工作区创建并切换到
    /// PR 的来源分支。如果本地已存在该分支，则尝试更新它。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在、本地 git 操作失败或 `gh` CLI 调用失败时返回错误。
    async fn checkout(&self, number: u64) -> Result<()> {
        debug!(repo = %self.repo, number, "spawning `gh pr checkout`");

        let number_str = number.to_string();
        let output = self
            .runner
            .run("gh", &["pr", "checkout", &number_str, "--repo", &self.repo])
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            return Err(parse_gh_error(&output.stderr).into());
        }

        Ok(())
    }

    /// 将草稿 PR 标记为可审查状态（ready for review）。
    ///
    /// 调用 `gh pr ready <number> --repo <repo>` 将草稿 PR 转为可审查状态，
    /// 并通过 `gh pr view` 重新获取更新后的 PR 数据。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在、不是草稿状态或 `gh` CLI 调用失败时返回错误。
    async fn mark_ready(&self, number: u64) -> Result<PrData> {
        debug!(repo = %self.repo, number, "spawning `gh pr ready`");

        let number_str = number.to_string();
        let output = self
            .runner
            .run("gh", &["pr", "ready", &number_str, "--repo", &self.repo])
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            return Err(parse_gh_error(&output.stderr).into());
        }

        // `gh pr ready` does not return JSON; re-fetch the PR to get updated data.
        self.view(number).await
    }

    /// 将 PR 标记为草稿状态（work in progress）。
    ///
    /// 调用 `gh pr convert-to-draft <number> --repo <repo>` 将可审查的 PR 转为草稿，
    /// 并通过 `gh pr view` 重新获取更新后的 PR 数据。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在、已是草稿状态或 `gh` CLI 调用失败时返回错误。
    async fn mark_wip(&self, number: u64) -> Result<PrData> {
        debug!(repo = %self.repo, number, "spawning `gh pr convert-to-draft`");

        let number_str = number.to_string();
        let output = self
            .runner
            .run(
                "gh",
                &["pr", "convert-to-draft", &number_str, "--repo", &self.repo],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            return Err(parse_gh_error(&output.stderr).into());
        }

        // `gh pr convert-to-draft` does not return JSON; re-fetch the PR.
        self.view(number).await
    }

    /// 同步 PR 的分支（将 base 分支的最新变更合入 head 分支）。
    ///
    /// 调用 `gh pr update-branch <number> --repo <repo>` 将 PR 的来源分支
    /// 更新到与目标分支的最新状态同步，解决分支过时问题。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在、同步存在冲突或 `gh` CLI 调用失败时返回错误。
    async fn sync_branch(&self, number: u64) -> Result<()> {
        debug!(repo = %self.repo, number, "spawning `gh pr update-branch`");

        let number_str = number.to_string();
        let output = self
            .runner
            .run(
                "gh",
                &["pr", "update-branch", &number_str, "--repo", &self.repo],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            return Err(parse_gh_error(&output.stderr).into());
        }

        Ok(())
    }

    /// 获取指定 PR 的统一差异格式（unified diff）文本。
    ///
    /// 调用 `gh pr diff <number> --repo <repo>` 获取平台原生的 unified diff
    /// 输出，可直接用于 `git apply`。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在或 `gh` CLI 调用失败时返回错误。
    async fn diff(&self, number: u64) -> Result<String> {
        debug!(repo = %self.repo, number, "spawning `gh pr diff`");

        let number_str = number.to_string();
        let output = self
            .runner
            .run("gh", &["pr", "diff", &number_str, "--repo", &self.repo])
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh pr diff: {e}")))?;

        if !output.status.success() {
            return Err(parse_gh_error(&output.stderr).into());
        }

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    /// 获取指定 PR 的 patch 格式文本（含邮件头信息）。
    ///
    /// 调用 `gh api repos/{repo}/pulls/{number}` 并设置 `Accept` 头为
    /// `application/vnd.github.v3.patch`，返回包含 commit 元数据的 patch 格式输出，
    /// 可用于 `git am`。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在或 `gh` CLI 调用失败时返回错误。
    async fn patch(&self, number: u64) -> Result<String> {
        debug!(repo = %self.repo, number, "spawning `gh api pr patch`");

        let api_path = format!(
            "repos/{repo}/pulls/{number}",
            repo = self.repo,
            number = number
        );

        let output = self
            .runner
            .run(
                "gh",
                &[
                    "api",
                    &api_path,
                    "-H",
                    "Accept: application/vnd.github.v3.patch",
                ],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh api pr patch: {e}")))?;

        if !output.status.success() {
            return Err(parse_gh_error(&output.stderr).into());
        }

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}

/// Parse PR number from GitHub URL.
///
/// Extracts the numeric PR number from URLs like:
/// - `https://github.com/owner/repo/pull/123`
/// - `https://github.enterprise.com/org/project/pull/456`
fn parse_pr_number_from_url(url: &str) -> Option<u64> {
    url.lines().find_map(|line| {
        let line = line.trim();
        if line.contains("/pull/") {
            line.rsplit('/').next().and_then(|s| s.parse().ok())
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::MockCommandRunner;

    #[test]
    fn test_should_construct_github_pr_provider() {
        let provider = GitHubPrProvider::new("octocat/hello-world");
        assert_eq!(provider.repo, "octocat/hello-world");
    }

    #[test]
    fn test_should_construct_github_pr_provider_from_string() {
        let repo = String::from("octocat/hello-world");
        let provider = GitHubPrProvider::new(repo);
        assert_eq!(provider.repo, "octocat/hello-world");
    }

    #[test]
    fn test_should_deserialize_pr_data_from_gh_output() {
        // 模拟 `gh pr view --json ...` 的实际输出
        let gh_json = br#"{
            "number": 123,
            "title": "Add new feature",
            "body": "This PR adds a new feature",
            "state": "open",
            "draft": false,
            "author": {"login": "alice", "id": "2"},
            "baseBranch": "main",
            "headBranch": "feature/new-thing",
            "createdAt": "2026-02-20T14:00:00Z",
            "updatedAt": "2026-02-21T10:30:00Z",
            "url": "https://github.com/octocat/hello-world/pull/123"
        }"#;

        let pr: PrData = serde_json::from_slice(gh_json).expect("valid PrData JSON");
        assert_eq!(pr.number, 123);
        assert_eq!(pr.title, "Add new feature");
        assert_eq!(pr.state, State::Open);
        assert!(!pr.draft);
        assert_eq!(pr.author.login, "alice");
        assert_eq!(pr.base_branch, "main");
        assert_eq!(pr.head_branch, "feature/new-thing");
        assert_eq!(pr.url, "https://github.com/octocat/hello-world/pull/123");
    }

    #[test]
    fn test_should_deserialize_empty_pr_list_from_gh_output() {
        let gh_json = b"[]";
        let prs: Vec<PrData> = serde_json::from_slice(gh_json).expect("valid PrData list");
        assert!(prs.is_empty());
    }

    #[test]
    fn test_should_deserialize_draft_pr_from_gh_output() {
        let gh_json = br#"{
            "number": 456,
            "title": "WIP: experiment",
            "body": null,
            "state": "open",
            "draft": true,
            "author": {"login": "bob", "id": "3"},
            "baseBranch": "main",
            "headBranch": "wip/experiment",
            "createdAt": "2026-03-10T09:00:00Z",
            "updatedAt": "2026-03-10T09:00:00Z",
            "url": "https://github.com/octocat/hello-world/pull/456"
        }"#;

        let pr: PrData = serde_json::from_slice(gh_json).expect("valid PrData JSON");
        assert!(pr.draft);
        assert!(pr.body.is_none());
        assert_eq!(pr.title, "WIP: experiment");
    }

    #[test]
    fn test_should_debug_format_provider() {
        let provider = GitHubPrProvider::new("octocat/hello-world");
        let debug = format!("{provider:?}");
        assert!(debug.contains("GitHubPrProvider"));
        assert!(debug.contains("octocat/hello-world"));
    }

    // --- close/reopen: deserialized PrData tests ---

    #[test]
    fn test_should_deserialize_closed_pr_from_gh_close_output() {
        // 模拟 `gh pr close --json ...` 的返回数据
        let gh_json = br#"{
            "number": 50,
            "title": "Obsolete change",
            "body": "Superseded by #55",
            "state": "closed",
            "draft": false,
            "author": {"login": "dev", "id": "10"},
            "baseBranch": "main",
            "headBranch": "feature/obsolete",
            "createdAt": "2026-05-01T08:00:00Z",
            "updatedAt": "2026-05-02T12:00:00Z",
            "url": "https://github.com/octocat/hello-world/pull/50"
        }"#;

        let pr: PrData = serde_json::from_slice(gh_json).expect("valid closed PrData");
        assert_eq!(pr.number, 50);
        assert_eq!(pr.state, State::Closed);
    }

    #[test]
    fn test_should_deserialize_reopened_pr_from_gh_reopen_output() {
        let gh_json = br#"{
            "number": 50,
            "title": "Obsolete change",
            "body": "Actually still needed",
            "state": "open",
            "draft": false,
            "author": {"login": "dev", "id": "10"},
            "baseBranch": "main",
            "headBranch": "feature/obsolete",
            "createdAt": "2026-05-01T08:00:00Z",
            "updatedAt": "2026-05-03T09:00:00Z",
            "url": "https://github.com/octocat/hello-world/pull/50"
        }"#;

        let pr: PrData = serde_json::from_slice(gh_json).expect("valid reopened PrData");
        assert_eq!(pr.number, 50);
        assert_eq!(pr.state, State::Open);
    }

    // --- comment: CommentData deserialization tests ---

    #[test]
    fn test_should_deserialize_comment_data_from_gh_pr_comment_output() {
        // 模拟 `gh pr comment --json id,body,author,createdAt` 的输出
        let gh_json = br#"{
            "id": 2002,
            "body": "Approved, merging now.",
            "author": {"login": "reviewer", "id": "88"},
            "createdAt": "2026-06-20T16:00:00Z"
        }"#;

        let comment: CommentData = serde_json::from_slice(gh_json).expect("valid CommentData");
        assert_eq!(comment.id, 2002);
        assert_eq!(comment.body, "Approved, merging now.");
        assert_eq!(comment.author.login, "reviewer");
        assert_eq!(comment.author.id, "88");
    }

    #[tokio::test]
    async fn test_should_return_newly_created_pr_comment_not_stale() {
        use crate::runner::MockCommandRunner;

        // Mock `gh api POST` response — the comment just created
        let post_response_json = r#"{
            "id": 8888888888,
            "body": "NEW PR comment",
            "user": {"login": "reviewer", "id": 99},
            "created_at": "2026-08-04T13:00:00Z"
        }"#;

        let runner = MockCommandRunner::success(post_response_json);
        let provider = GitHubPrProvider::with_runner("owner/repo", runner);

        let result = provider.comment(7, "NEW PR comment").await.unwrap();

        // The returned id must be the NEW comment's id, not a stale one
        assert_eq!(result.id, 8_888_888_888);
        assert_eq!(result.body, "NEW PR comment");
        assert_eq!(result.author.login, "reviewer");
    }

    #[tokio::test]
    async fn test_should_return_error_when_pr_comment_api_fails() {
        use crate::runner::MockCommandRunner;

        let runner = MockCommandRunner::failure(
            r#"{"message": "Not Found", "documentation_url": "https://docs.github.com"}"#,
            1,
        );
        let provider = GitHubPrProvider::with_runner("owner/repo", runner);

        let result = provider.comment(999, "test").await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        // `parse_gh_error` returns `PlatformCliError`, which converts to
        // `CoreError::Cli` (matching the existing comment error-path test).
        assert!(
            matches!(err, CoreError::Cli(_)),
            "expected Cli error, got: {err:?}"
        );
    }

    // --- merge: MergeResult deserialization tests ---

    #[test]
    fn test_should_deserialize_merge_result_from_gh_merge_output() {
        // `gh pr merge` 返回人类可读文本，不是 JSON。
        // MergeResult 由代码构造，但需确保序列化/反序列化正确。
        let gh_text = b"Pull request #123 was successfully merged.\n";
        let message = String::from_utf8_lossy(gh_text).trim().to_string();
        let result = MergeResult {
            merged: true,
            sha: None,
            message: Some(message),
        };

        assert!(result.merged);
        assert!(result.message.as_deref().is_some());
        assert_eq!(
            result.message.as_deref(),
            Some("Pull request #123 was successfully merged.")
        );
    }

    #[test]
    fn test_should_roundtrip_merge_result_via_serde() {
        let result = MergeResult {
            merged: true,
            sha: Some("deadbeef1234".into()),
            message: Some("Squash merged".into()),
        };
        let json = serde_json::to_string(&result).expect("serialize");
        let round_tripped: MergeResult = serde_json::from_str(&json).expect("deserialize");
        assert!(round_tripped.merged);
        assert_eq!(round_tripped.sha, result.sha);
        assert_eq!(round_tripped.message, result.message);
    }

    #[test]
    fn test_should_serialize_merge_result_skips_null_fields() {
        let result = MergeResult {
            merged: false,
            sha: None,
            message: None,
        };
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("null"));
        assert_eq!(json, r#"{"merged":false}"#);
    }

    // --- provider construction / clone tests ---

    #[test]
    fn test_should_create_provider_with_different_repos() {
        let r1 = GitHubPrProvider::new("org/repo-a");
        let r2 = GitHubPrProvider::new("org/repo-b");
        assert_eq!(r1.repo, "org/repo-a");
        assert_eq!(r2.repo, "org/repo-b");
    }

    #[test]
    fn test_should_clone_github_pr_provider() {
        let original = GitHubPrProvider::new("owner/repo");
        let cloned = original.clone();
        assert_eq!(original.repo, cloned.repo);
    }

    #[test]
    fn test_should_parse_pr_number_from_url() {
        assert_eq!(
            parse_pr_number_from_url("https://github.com/owner/repo/pull/123"),
            Some(123)
        );
        assert_eq!(
            parse_pr_number_from_url("https://github.enterprise.com/org/project/pull/456"),
            Some(456)
        );
    }

    #[test]
    fn test_should_parse_pr_number_from_multiline_output() {
        let output = "some header\nhttps://github.com/owner/repo/pull/789\nsome footer";
        assert_eq!(parse_pr_number_from_url(output), Some(789));
    }

    #[test]
    fn test_should_return_none_when_no_pr_url() {
        assert_eq!(parse_pr_number_from_url("no pull url here"), None);
        assert_eq!(parse_pr_number_from_url(""), None);
        assert_eq!(
            parse_pr_number_from_url("https://github.com/owner/repo/issues/123"),
            None
        );
    }

    // --- Failure-path tests using an injected MockCommandRunner ---

    fn sample_create_args() -> CreatePrArgs {
        CreatePrArgs {
            title: "Add feature".to_string(),
            body: Some("Detailed description".to_string()),
            head: "feature/new".to_string(),
            base: "main".to_string(),
            draft: false,
            repo: None,
            closes_issues: vec![],
        }
    }

    #[test]
    fn test_should_format_github_body_with_closing_issues() {
        use gitflow_core::pr::format_closing_body;

        let body = Some("Feature description".to_string());
        let issues = vec![24u64, 23];
        let result = format_closing_body(&body, &issues, "Closes");
        assert_eq!(
            result,
            Some("Feature description\n\nCloses #24\nCloses #23".to_string())
        );
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_fails_for_create() {
        let runner = MockCommandRunner::failure(r#"{"message": "Validation failed"}"#, 256);
        let provider = GitHubPrProvider::with_runner("owner/repo", runner);

        let result = provider.create(sample_create_args()).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_on_url_parse_failure_for_create() {
        // `create` succeeds but stdout has no parseable PR URL, so URL parsing fails.
        let runner = MockCommandRunner::success("no pull url here");
        let provider = GitHubPrProvider::with_runner("owner/repo", runner);

        let result = provider.create(sample_create_args()).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_fails_for_list() {
        let runner = MockCommandRunner::failure(r#"{"message": "Forbidden"}"#, 256);
        let provider = GitHubPrProvider::with_runner("owner/repo", runner);

        let result = provider.list(ListPrArgs::default()).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_list() {
        let runner = MockCommandRunner::success("invalid json");
        let provider = GitHubPrProvider::with_runner("owner/repo", runner);

        let result = provider.list(ListPrArgs::default()).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_fails_for_view() {
        let runner = MockCommandRunner::failure(r#"{"message": "PR not found"}"#, 256);
        let provider = GitHubPrProvider::with_runner("owner/repo", runner);

        let result = provider.view(999).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_view() {
        let runner = MockCommandRunner::success("invalid json");
        let provider = GitHubPrProvider::with_runner("owner/repo", runner);

        let result = provider.view(1).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_fails_for_close() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not found"}"#, 256);
        let provider = GitHubPrProvider::with_runner("owner/repo", runner);

        let result = provider.close(42).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_fails_for_reopen() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not found"}"#, 256);
        let provider = GitHubPrProvider::with_runner("owner/repo", runner);

        let result = provider.reopen(42).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_fails_for_merge() {
        let runner = MockCommandRunner::failure(r#"{"message": "Merge conflict"}"#, 256);
        let provider = GitHubPrProvider::with_runner("owner/repo", runner);

        let result = provider.merge(42, None, false).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_forward_auto_and_report_not_merged_for_queued_merge() {
        let runner = MockCommandRunner::success("✓ Merge scheduled for pull request #42");
        let provider = GitHubPrProvider::with_runner("owner/repo", runner.clone());

        let result = provider
            .merge(42, Some(MergeStrategy::Squash), true)
            .await
            .expect("queued merge should succeed");

        let args = &runner.recorded_calls()[0].1;
        assert!(
            args.iter().any(|a| a == "--auto"),
            "`--auto` must be forwarded to gh, got {args:?}"
        );
        assert!(
            !result.merged,
            "a scheduled merge has not landed, so merged must be false"
        );
    }

    #[tokio::test]
    async fn test_should_return_platform_error_on_invalid_json_for_merge() {
        // `gh pr merge` returns human-readable text, so any successful output is a
        // valid message. A failing exit status is the merge failure path instead.
        let runner = MockCommandRunner::failure("invalid json", 256);
        let provider = GitHubPrProvider::with_runner("owner/repo", runner);

        let result = provider.merge(42, Some(MergeStrategy::Squash), false).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_fails_for_checkout() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not found"}"#, 256);
        let provider = GitHubPrProvider::with_runner("owner/repo", runner);

        let result = provider.checkout(42).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_fails_for_sync_branch() {
        let runner = MockCommandRunner::failure(r#"{"message": "Conflict"}"#, 256);
        let provider = GitHubPrProvider::with_runner("owner/repo", runner);

        let result = provider.sync_branch(42).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_fails_for_mark_ready() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not a draft"}"#, 256);
        let provider = GitHubPrProvider::with_runner("owner/repo", runner);

        let result = provider.mark_ready(42).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_fails_for_mark_wip() {
        let runner = MockCommandRunner::failure(r#"{"message": "Already a draft"}"#, 256);
        let provider = GitHubPrProvider::with_runner("owner/repo", runner);

        let result = provider.mark_wip(42).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_fails_for_comment() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not found"}"#, 256);
        let provider = GitHubPrProvider::with_runner("owner/repo", runner);

        let result = provider.comment(42, "a comment").await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    // --- diff() tests ---

    #[tokio::test]
    async fn test_should_fetch_pr_diff() {
        let diff_output = "diff --git a/file.txt b/file.txt\nindex 1234567..abcdefg 100644\n--- \
                           a/file.txt\n+++ b/file.txt\n@@ -1 +1 @@\n-old\n+new\n";
        let runner = MockCommandRunner::success(diff_output);
        let provider = GitHubPrProvider::with_runner("owner/repo", runner);

        let result = provider.diff(42).await.expect("diff should succeed");
        assert_eq!(result, diff_output);
    }

    #[tokio::test]
    async fn test_should_return_error_when_pr_diff_fails() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not found"}"#, 256);
        let provider = GitHubPrProvider::with_runner("owner/repo", runner);

        let result = provider.diff(999).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    // --- patch() tests ---

    #[tokio::test]
    async fn test_should_fetch_pr_patch() {
        let patch_output =
            "From abc123\nSubject: [PATCH] Update file\n\ndiff --git a/file.txt b/file.txt\n";
        let runner = MockCommandRunner::success(patch_output);
        let provider = GitHubPrProvider::with_runner("owner/repo", runner);

        let result = provider.patch(42).await.expect("patch should succeed");
        assert_eq!(result, patch_output);
    }

    #[tokio::test]
    async fn test_should_return_error_when_pr_patch_fails() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not found"}"#, 256);
        let provider = GitHubPrProvider::with_runner("owner/repo", runner);

        let result = provider.patch(999).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }
}
