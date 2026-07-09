//! GitLab Merge Request 提供者实现。
//!
//! 通过 `glab mr` CLI 实现 [`PrProvider`] trait，支持 MR 的创建、列表、查看、
//! 关闭、合并、检出、草稿状态切换和分支同步。
//! 所有方法通过 [`CommandRunner`] 抽象调用 `glab`，捕获 stdout 并解析 JSON。
//!
//! `glab` 的 `JSON` 输出使用 `snake_case` 字段名和 `GitLab` 特有的字段名（如 `iid`、
//! `source_branch`、`target_branch`、`web_url`），因此使用中间类型 [`MrApiResponse`]
//! 进行反序列化，然后转换为核心类型 [`PrData`]。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use gitflow_cli_core::{
    CoreError, Result,
    pr::{CreatePrArgs, ListPrArgs, PrData, PrProvider},
    types::{CommentData, MergeResult, MergeStrategy, State, UserSummary},
};
use serde::Deserialize;
use tracing::debug;

use crate::{
    error::parse_glab_error,
    runner::{CommandRunner, RealCommandRunner},
};

/// GitLab Merge Request 提供者，通过 `glab` CLI 操作。
///
/// 命令执行通过 [`CommandRunner`] 抽象，生产环境默认使用
/// [`RealCommandRunner`]，测试可注入自定义 runner 以模拟成功或失败场景。
///
/// # Examples
///
/// ```no_run
/// use gitflow_cli_gitlab::GitLabMrProvider;
///
/// let provider = GitLabMrProvider::new("gitlab-org/gitlab");
/// ```
#[derive(Debug, Clone)]
pub struct GitLabMrProvider<R: CommandRunner = RealCommandRunner> {
    /// GitLab `namespace/project`。
    repo: String,
    /// 用于执行 `glab` CLI 命令的 runner。
    runner: R,
}

impl GitLabMrProvider<RealCommandRunner> {
    /// 创建新的 GitLab MR 提供者，使用真实的进程执行器。
    ///
    /// `repo` 格式为 `namespace/project`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> Self {
        Self {
            repo: repo.into(),
            runner: RealCommandRunner,
        }
    }
}

impl<R: CommandRunner> GitLabMrProvider<R> {
    /// 使用自定义 [`CommandRunner`] 创建提供者。
    ///
    /// 主要用于测试，可注入模拟 runner 以控制 `glab` CLI 的输出。
    /// `repo` 格式为 `namespace/project`。
    #[must_use]
    pub fn with_runner(repo: impl Into<String>, runner: R) -> Self {
        Self {
            repo: repo.into(),
            runner,
        }
    }
}

// ── 中间 API 响应类型 ──────────────────────────────────────────────

/// `glab mr` JSON 输出中的用户信息。
#[derive(Debug, Clone, Deserialize)]
struct ApiUser {
    username: String,
    #[serde(default)]
    id: u64,
}

impl From<&ApiUser> for UserSummary {
    fn from(u: &ApiUser) -> Self {
        Self {
            login: u.username.clone(),
            id: u.id.to_string(),
        }
    }
}

/// `glab mr --output json` 返回的 JSON 结构。
#[derive(Debug, Clone, Deserialize)]
struct MrApiResponse {
    iid: u64,
    title: String,
    #[serde(default)]
    description: Option<String>,
    state: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    author: Option<ApiUser>,
    #[serde(default)]
    source_branch: String,
    #[serde(default)]
    target_branch: String,
    #[serde(default)]
    created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    updated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    web_url: Option<String>,
}

impl From<MrApiResponse> for PrData {
    fn from(api: MrApiResponse) -> Self {
        let now = Utc::now();
        let state = if api.state == "closed" || api.state == "merged" {
            State::Closed
        } else {
            State::Open
        };
        let author = api.author.as_ref().map_or_else(
            || UserSummary {
                login: "unknown".into(),
                id: "0".to_string(),
            },
            UserSummary::from,
        );

        Self {
            number: api.iid,
            title: api.title,
            body: api.description,
            state,
            draft: api.draft,
            author,
            base_branch: api.target_branch,
            head_branch: api.source_branch,
            created_at: api.created_at.unwrap_or(now),
            updated_at: api.updated_at.unwrap_or(now),
            url: api.web_url.unwrap_or_default(),
        }
    }
}

/// `glab mr note --output json` 返回的 JSON 结构。
#[derive(Debug, Clone, Deserialize)]
struct CommentApiResponse {
    id: u64,
    #[serde(default)]
    body: String,
    #[serde(default)]
    author: Option<ApiUser>,
    #[serde(default)]
    created_at: Option<DateTime<Utc>>,
}

impl From<CommentApiResponse> for CommentData {
    fn from(api: CommentApiResponse) -> Self {
        let author = api.author.as_ref().map_or_else(
            || UserSummary {
                login: "unknown".into(),
                id: "0".to_string(),
            },
            UserSummary::from,
        );
        Self {
            id: api.id,
            body: api.body,
            author,
            created_at: api.created_at.unwrap_or_else(Utc::now),
        }
    }
}

// ── trait 实现 ──────────────────────────────────────────────────────

#[async_trait]
impl<R: CommandRunner + 'static> PrProvider for GitLabMrProvider<R> {
    async fn create(&self, args: CreatePrArgs) -> Result<PrData> {
        let repo = args.repo.as_deref().unwrap_or(&self.repo);
        let mut cmd_args: Vec<&str> = vec![
            "mr",
            "create",
            "--repo",
            repo,
            "--title",
            &args.title,
            "--source-branch",
            &args.head,
            "--target-branch",
            &args.base,
            "--output",
            "json",
        ];

        if let Some(body) = &args.body {
            cmd_args.push("--description");
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
            "spawning `glab mr create`"
        );

        let output = self
            .runner
            .run("glab", &cmd_args)
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        let api_response: MrApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_response.into())
    }

    async fn list(&self, args: ListPrArgs) -> Result<Vec<PrData>> {
        let mut cmd_args: Vec<&str> = vec!["mr", "list", "--repo", &self.repo, "--output", "json"];

        // glab uses --closed for closed MRs
        // Default (no flag) shows open MRs
        if let Some(state) = &args.state
            && matches!(state, State::Closed)
        {
            cmd_args.push("--closed");
        }

        let limit_str = args.limit.map(|limit| limit.to_string());
        if let Some(ref limit) = limit_str {
            cmd_args.push("--per-page");
            cmd_args.push(limit);
        }

        debug!(repo = %self.repo, "spawning `glab mr list`");

        let output = self
            .runner
            .run("glab", &cmd_args)
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        let api_responses: Vec<MrApiResponse> =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_responses.into_iter().map(PrData::from).collect())
    }

    async fn view(&self, number: u64) -> Result<PrData> {
        debug!(repo = %self.repo, number, "spawning `glab mr view`");

        let number_str = number.to_string();
        let output = self
            .runner
            .run(
                "glab",
                &[
                    "mr",
                    "view",
                    &number_str,
                    "--repo",
                    &self.repo,
                    "--output",
                    "json",
                ],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        let api_response: MrApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_response.into())
    }

    async fn close(&self, number: u64) -> Result<PrData> {
        debug!(repo = %self.repo, number, "spawning `glab mr close`");

        let number_str = number.to_string();
        let output = self
            .runner
            .run(
                "glab",
                &[
                    "mr",
                    "close",
                    &number_str,
                    "--repo",
                    &self.repo,
                    "--output",
                    "json",
                ],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        let api_response: MrApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_response.into())
    }

    async fn reopen(&self, number: u64) -> Result<PrData> {
        debug!(repo = %self.repo, number, "spawning `glab mr reopen`");

        let number_str = number.to_string();
        let output = self
            .runner
            .run(
                "glab",
                &[
                    "mr",
                    "reopen",
                    &number_str,
                    "--repo",
                    &self.repo,
                    "--output",
                    "json",
                ],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        let api_response: MrApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_response.into())
    }

    async fn comment(&self, number: u64, body: &str) -> Result<CommentData> {
        debug!(repo = %self.repo, number, "spawning `glab mr note`");

        let number_str = number.to_string();
        let output = self
            .runner
            .run(
                "glab",
                &[
                    "mr",
                    "note",
                    &number_str,
                    "--repo",
                    &self.repo,
                    "--body",
                    body,
                    "--output",
                    "json",
                ],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        let api_response: CommentApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_response.into())
    }

    async fn merge(&self, number: u64, strategy: Option<MergeStrategy>) -> Result<MergeResult> {
        debug!(repo = %self.repo, number, ?strategy, "spawning `glab mr merge`");

        let number_str = number.to_string();
        let mut cmd_args: Vec<&str> = vec!["mr", "merge", &number_str, "--repo", &self.repo];

        match strategy {
            Some(MergeStrategy::Squash) => cmd_args.push("--squash"),
            Some(MergeStrategy::Rebase) => cmd_args.push("--rebase"),
            Some(MergeStrategy::Merge) | None => cmd_args.push("--merge"),
        }

        let output = self
            .runner
            .run("glab", &cmd_args)
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        let message = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(MergeResult {
            merged: true,
            sha: None,
            message: Some(message),
        })
    }

    async fn checkout(&self, number: u64) -> Result<()> {
        debug!(repo = %self.repo, number, "spawning `glab mr checkout`");

        let number_str = number.to_string();
        let output = self
            .runner
            .run(
                "glab",
                &["mr", "checkout", &number_str, "--repo", &self.repo],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        Ok(())
    }

    async fn mark_ready(&self, number: u64) -> Result<PrData> {
        debug!(repo = %self.repo, number, "spawning `glab mr ready`");

        let number_str = number.to_string();
        let output = self
            .runner
            .run("glab", &["mr", "ready", &number_str, "--repo", &self.repo])
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        self.view(number).await
    }

    async fn mark_wip(&self, number: u64) -> Result<PrData> {
        debug!(repo = %self.repo, number, "spawning `glab mr draft`");

        let number_str = number.to_string();
        let output = self
            .runner
            .run("glab", &["mr", "draft", &number_str, "--repo", &self.repo])
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        self.view(number).await
    }

    async fn sync_branch(&self, number: u64) -> Result<()> {
        debug!(repo = %self.repo, number, "spawning `glab mr rebase`");

        let number_str = number.to_string();
        let output = self
            .runner
            .run("glab", &["mr", "rebase", &number_str, "--repo", &self.repo])
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::MockCommandRunner;

    #[test]
    fn test_should_construct_gitlab_mr_provider() {
        let provider = GitLabMrProvider::new("gitlab-org/gitlab");
        assert_eq!(provider.repo, "gitlab-org/gitlab");
    }

    #[test]
    fn test_should_construct_gitlab_mr_provider_from_string() {
        let repo = String::from("gitlab-org/gitlab");
        let provider = GitLabMrProvider::new(repo);
        assert_eq!(provider.repo, "gitlab-org/gitlab");
    }

    #[test]
    fn test_should_deserialize_mr_api_response() {
        let json = br#"{
            "iid": 123,
            "title": "Add new feature",
            "description": "This MR adds a new feature",
            "state": "opened",
            "draft": false,
            "author": {"username": "alice", "id": 2},
            "source_branch": "feature/new-thing",
            "target_branch": "main",
            "created_at": "2026-02-20T14:00:00Z",
            "updated_at": "2026-02-21T10:30:00Z",
            "web_url": "https://gitlab.com/gitlab-org/gitlab/-/merge_requests/123"
        }"#;

        let api: MrApiResponse = serde_json::from_slice(json).expect("valid MrApiResponse");
        let pr: PrData = api.into();

        assert_eq!(pr.number, 123);
        assert_eq!(pr.title, "Add new feature");
        assert_eq!(pr.state, State::Open);
        assert!(!pr.draft);
        assert_eq!(pr.author.login, "alice");
        assert_eq!(pr.base_branch, "main");
        assert_eq!(pr.head_branch, "feature/new-thing");
        assert_eq!(
            pr.url,
            "https://gitlab.com/gitlab-org/gitlab/-/merge_requests/123"
        );
    }

    #[test]
    fn test_should_deserialize_draft_mr() {
        let json = br#"{
            "iid": 456,
            "title": "WIP: experiment",
            "description": null,
            "state": "opened",
            "draft": true,
            "author": {"username": "bob", "id": 3},
            "source_branch": "wip/experiment",
            "target_branch": "main",
            "created_at": "2026-03-10T09:00:00Z",
            "updated_at": "2026-03-10T09:00:00Z",
            "web_url": "https://gitlab.com/org/project/-/merge_requests/456"
        }"#;

        let api: MrApiResponse = serde_json::from_slice(json).expect("valid MrApiResponse");
        let pr: PrData = api.into();
        assert!(pr.draft);
        assert!(pr.body.is_none());
    }

    #[test]
    fn test_should_deserialize_merged_mr_as_closed() {
        let json = br#"{
            "iid": 789,
            "title": "Merged feature",
            "description": null,
            "state": "merged",
            "draft": false,
            "author": {"username": "dev", "id": 1},
            "source_branch": "feature/done",
            "target_branch": "main",
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-02T00:00:00Z",
            "web_url": "https://gitlab.com/org/project/-/merge_requests/789"
        }"#;

        let api: MrApiResponse = serde_json::from_slice(json).expect("valid MrApiResponse");
        let pr: PrData = api.into();
        assert_eq!(pr.state, State::Closed);
    }

    #[test]
    fn test_should_deserialize_empty_mr_list() {
        let json = b"[]";
        let list: Vec<MrApiResponse> = serde_json::from_slice(json).expect("valid empty list");
        assert!(list.is_empty());
    }

    #[test]
    fn test_should_deserialize_comment_api_response() {
        let json = br#"{
            "id": 2002,
            "body": "Approved, merging now.",
            "author": {"username": "reviewer", "id": 88},
            "created_at": "2026-06-20T16:00:00Z"
        }"#;

        let api: CommentApiResponse =
            serde_json::from_slice(json).expect("valid CommentApiResponse");
        let comment: CommentData = api.into();
        assert_eq!(comment.id, 2002);
        assert_eq!(comment.body, "Approved, merging now.");
        assert_eq!(comment.author.login, "reviewer");
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
    }

    #[test]
    fn test_should_debug_format_provider() {
        let provider = GitLabMrProvider::new("gitlab-org/gitlab");
        let debug = format!("{provider:?}");
        assert!(debug.contains("GitLabMrProvider"));
        assert!(debug.contains("gitlab-org/gitlab"));
    }

    #[test]
    fn test_should_clone_gitlab_mr_provider() {
        let original = GitLabMrProvider::new("owner/repo");
        let cloned = original.clone();
        assert_eq!(original.repo, cloned.repo);
    }

    #[test]
    fn test_should_handle_missing_author_with_fallback() {
        let json = br#"{
            "iid": 1,
            "title": "No author",
            "description": null,
            "state": "opened",
            "draft": false,
            "author": null,
            "source_branch": "dev",
            "target_branch": "main",
            "created_at": null,
            "updated_at": null,
            "web_url": null
        }"#;

        let api: MrApiResponse = serde_json::from_slice(json).expect("valid MrApiResponse");
        let pr: PrData = api.into();
        assert_eq!(pr.author.login, "unknown");
    }

    // --- Failure-path tests using an injected MockCommandRunner ---

    fn sample_create_args() -> CreatePrArgs {
        CreatePrArgs {
            title: "Add feature".to_string(),
            body: Some("Description".to_string()),
            head: "feature/x".to_string(),
            base: "main".to_string(),
            draft: false,
            repo: None,
        }
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_glab_fails_for_create() {
        let runner = MockCommandRunner::failure(r#"{"message": "Validation failed"}"#, 256);
        let provider = GitLabMrProvider::with_runner("owner/repo", runner);

        let result = provider.create(sample_create_args()).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_create() {
        let runner = MockCommandRunner::success("not valid json");
        let provider = GitLabMrProvider::with_runner("owner/repo", runner);

        let result = provider.create(sample_create_args()).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_glab_fails_for_list() {
        let runner = MockCommandRunner::failure(r#"{"message": "Forbidden"}"#, 256);
        let provider = GitLabMrProvider::with_runner("owner/repo", runner);

        let result = provider.list(ListPrArgs::default()).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_list() {
        let runner = MockCommandRunner::success("invalid");
        let provider = GitLabMrProvider::with_runner("owner/repo", runner);

        let result = provider.list(ListPrArgs::default()).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_glab_fails_for_view() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not found"}"#, 256);
        let provider = GitLabMrProvider::with_runner("owner/repo", runner);

        let result = provider.view(999).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_view() {
        let runner = MockCommandRunner::success("not valid json");
        let provider = GitLabMrProvider::with_runner("owner/repo", runner);

        let result = provider.view(1).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_glab_fails_for_close() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not found"}"#, 256);
        let provider = GitLabMrProvider::with_runner("owner/repo", runner);

        let result = provider.close(42).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_glab_fails_for_reopen() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not found"}"#, 256);
        let provider = GitLabMrProvider::with_runner("owner/repo", runner);

        let result = provider.reopen(42).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_glab_fails_for_comment() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not found"}"#, 256);
        let provider = GitLabMrProvider::with_runner("owner/repo", runner);

        let result = provider.comment(42, "a comment").await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_glab_fails_for_merge() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not mergeable"}"#, 256);
        let provider = GitLabMrProvider::with_runner("owner/repo", runner);

        let result = provider.merge(42, None).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_glab_fails_for_checkout() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not found"}"#, 256);
        let provider = GitLabMrProvider::with_runner("owner/repo", runner);

        let result = provider.checkout(42).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_glab_fails_for_sync_branch() {
        let runner = MockCommandRunner::failure(r#"{"message": "Rebase failed"}"#, 256);
        let provider = GitLabMrProvider::with_runner("owner/repo", runner);

        let result = provider.sync_branch(42).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_glab_fails_for_mark_ready() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not found"}"#, 256);
        let provider = GitLabMrProvider::with_runner("owner/repo", runner);

        let result = provider.mark_ready(42).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_glab_fails_for_mark_wip() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not found"}"#, 256);
        let provider = GitLabMrProvider::with_runner("owner/repo", runner);

        let result = provider.mark_wip(42).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }
}
