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
use gitflow_core::{
    CoreError, Result,
    pr::{CreatePrArgs, ListPrArgs, PrData, PrProvider},
    types::{CommentData, MergeResult, MergeStrategy, State, UserSummary},
};
use serde::Deserialize;
use tracing::debug;

use crate::{
    commit::encode_project_path,
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
/// use gitflow_gitlab::GitLabMrProvider;
///
/// let provider = GitLabMrProvider::new("gitlab-org/gitlab");
/// ```
#[derive(Debug, Clone)]
pub struct GitLabMrProvider<R: CommandRunner = RealCommandRunner> {
    /// GitLab `namespace/project`。供 REST notes API 路径编码使用，始终是裸
    /// `owner/repo` 形式，不受 [`repo_target`](Self::repo_target) 影响。
    repo: String,
    /// 传给 `glab mr ...` 子命令 `--repo` 参数的目标字符串。默认等于 `repo`；
    /// 通过 [`with_remote_url`](GitLabMrProvider::with_remote_url) 构造时为完整
    /// git remote URL，用于在自建 GitLab 实例上显式锁定 host。
    repo_target: String,
    /// 用于执行 `glab` CLI 命令的 runner。
    runner: R,
}

impl GitLabMrProvider<RealCommandRunner> {
    /// 创建新的 GitLab MR 提供者，使用真实的进程执行器。
    ///
    /// `repo` 格式为 `namespace/project`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> Self {
        let repo = repo.into();
        Self {
            repo_target: repo.clone(),
            repo,
            runner: RealCommandRunner,
        }
    }

    /// Create a new provider from a shared [`Session`].
    ///
    /// This enables state reuse across multiple operations in workflow chains.
    #[must_use]
    pub fn with_session(session: &gitflow_core::Session) -> Self {
        let repo = session.repo.clone();
        Self {
            repo_target: repo.clone(),
            repo,
            runner: RealCommandRunner,
        }
    }

    /// 使用完整 git remote URL 作为 `glab mr ...` 的 `--repo` 目标创建提供者。
    ///
    /// `repo` 仍为裸 `namespace/project`（供 REST notes API 路径编码使用），
    /// `remote_url` 为完整 git remote URL，`glab` 官方文档确认 `--repo` 接受该形式。
    #[must_use]
    pub fn with_remote_url(repo: impl Into<String>, remote_url: impl Into<String>) -> Self {
        Self {
            repo: repo.into(),
            repo_target: remote_url.into(),
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
        let repo = repo.into();
        Self {
            repo_target: repo.clone(),
            repo,
            runner,
        }
    }

    /// 使用自定义 [`CommandRunner`] 并显式指定 `--repo` 目标创建提供者。
    ///
    /// 主要用于测试，验证 `repo_target`（如完整 remote URL）被正确传给 `glab`。
    #[must_use]
    pub fn with_runner_and_repo_target(
        repo: impl Into<String>,
        repo_target: impl Into<String>,
        runner: R,
    ) -> Self {
        Self {
            repo: repo.into(),
            repo_target: repo_target.into(),
            runner,
        }
    }

    /// 切换 MR 的草稿状态。
    ///
    /// 调用 `glab mr update <number> --draft=false/true` 更新草稿标记。
    ///
    /// # Errors
    ///
    /// 当 MR 不存在或 `glab` CLI 调用失败时返回错误。
    async fn run_mr_update(&self, number: u64, draft: bool) -> Result<()> {
        let number_str = number.to_string();
        let draft_flag = if draft {
            "--draft=true"
        } else {
            "--draft=false"
        };
        let output = self
            .runner
            .run(
                "glab",
                &[
                    "mr",
                    "update",
                    &number_str,
                    "--repo",
                    &self.repo_target,
                    draft_flag,
                ],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab mr update: {e}")))?;
        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
        }
        Ok(())
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

/// `glab repo view --output json` 的响应类型（仅取需要的字段）。
#[derive(Debug, Deserialize)]
struct RepoViewResponse {
    default_branch: String,
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
    /// GitLab 独有：`state` 为 `merged` 时有值。`state` 本身不足以区分
    /// "已合并"与"关闭未合并"，故一并取回。
    #[serde(default)]
    merged_at: Option<DateTime<Utc>>,
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
            merged_at: api.merged_at,
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
        let repo = args.repo.as_deref().unwrap_or(&self.repo_target);
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
        ];

        let final_body =
            gitflow_core::pr::format_closing_body(&args.body, &args.closes_issues, "Closes");

        if let Some(body) = &final_body {
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
            return Err(parse_glab_error(&output.stderr).into());
        }

        // Parse the MR IID from stdout (format: https://gitlab.com/.../-/merge_requests/123)
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mr_iid = parse_mr_iid_from_url(&stdout).ok_or_else(|| {
            CoreError::Platform(format!("Failed to parse MR URL from output: {stdout}"))
        })?;

        // Fetch full MR details via view
        self.view(mr_iid).await
    }

    async fn list(&self, args: ListPrArgs) -> Result<Vec<PrData>> {
        let mut cmd_args: Vec<&str> = vec![
            "mr",
            "list",
            "--repo",
            &self.repo_target,
            "--output",
            "json",
        ];

        // glab uses --closed for closed MRs, --all for all MRs
        // Default (no flag) shows open MRs
        if let Some(state) = &args.state {
            match state {
                State::Closed => cmd_args.push("--closed"),
                State::All => cmd_args.push("--all"),
                State::Open => {}
            }
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
            return Err(parse_glab_error(&output.stderr).into());
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
                    &self.repo_target,
                    "--output",
                    "json",
                ],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
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
                &["mr", "close", &number_str, "--repo", &self.repo_target],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
        }

        self.view(number).await
    }

    async fn reopen(&self, number: u64) -> Result<PrData> {
        debug!(repo = %self.repo, number, "spawning `glab mr reopen`");

        let number_str = number.to_string();
        let output = self
            .runner
            .run(
                "glab",
                &["mr", "reopen", &number_str, "--repo", &self.repo_target],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
        }

        self.view(number).await
    }

    async fn comment(&self, number: u64, body: &str) -> Result<CommentData> {
        debug!(repo = %self.repo, number, "spawning `glab api` POST mr note");

        let encoded_path = encode_project_path(&self.repo);
        let api_path = format!("/projects/{encoded_path}/merge_requests/{number}/notes");
        let body_arg = format!("body={body}");

        let output = self
            .runner
            .run(
                "glab",
                &["api", "--method", "POST", &api_path, "-f", &body_arg],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab api: {e}")))?;

        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
        }

        let api_response: CommentApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_response.into())
    }

    /// `auto` 为 `true` 时显式传 `--auto-merge`，由 GitLab 在 pipeline 通过后
    /// 自动合并，调用立即返回；此时 `merged` 为 `false`。
    /// 显式传参而非依赖 glab 的隐式默认，以免 glab 版本变更时行为漂移。
    async fn merge(
        &self,
        number: u64,
        strategy: Option<MergeStrategy>,
        auto: bool,
    ) -> Result<MergeResult> {
        debug!(repo = %self.repo, number, ?strategy, auto, "spawning `glab mr merge`");

        let number_str = number.to_string();
        let mut cmd_args: Vec<&str> = vec!["mr", "merge", &number_str, "--repo", &self.repo_target];

        match strategy {
            Some(MergeStrategy::Squash) => cmd_args.push("--squash"),
            Some(MergeStrategy::Rebase) => cmd_args.push("--rebase"),
            Some(MergeStrategy::Merge) | None => {}
        }

        if auto {
            cmd_args.push("--auto-merge");
        }

        let output = self
            .runner
            .run("glab", &cmd_args)
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
        }

        let message = String::from_utf8_lossy(&output.stdout).trim().to_string();
        // glab prints nothing for a scheduled merge; without a message the caller
        // cannot tell "scheduled" apart from "failed", since both report false.
        let message = if message.is_empty() && auto {
            "已排队合并：GitLab 将在 pipeline 通过后自动合并".to_string()
        } else {
            message
        };
        Ok(MergeResult {
            merged: !auto,
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
                &["mr", "checkout", &number_str, "--repo", &self.repo_target],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
        }

        Ok(())
    }

    async fn mark_ready(&self, number: u64) -> Result<PrData> {
        debug!(repo = %self.repo, number, "spawning `glab mr update --draft=false`");
        self.run_mr_update(number, false).await?;
        self.view(number).await
    }

    async fn mark_wip(&self, number: u64) -> Result<PrData> {
        debug!(repo = %self.repo, number, "spawning `glab mr update --draft=true`");
        self.run_mr_update(number, true).await?;
        self.view(number).await
    }

    async fn sync_branch(&self, number: u64) -> Result<()> {
        debug!(repo = %self.repo, number, "spawning `glab mr rebase`");

        let number_str = number.to_string();
        let output = self
            .runner
            .run(
                "glab",
                &["mr", "rebase", &number_str, "--repo", &self.repo_target],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
        }

        Ok(())
    }

    /// 获取指定 MR 的统一差异格式（unified diff）文本。
    ///
    /// 调用 `glab mr diff <iid>` 获取平台原生的 formatted diff 输出。
    ///
    /// # Errors
    ///
    /// 当 MR 不存在或 `glab` CLI 调用失败时返回错误。
    async fn diff(&self, number: u64) -> Result<String> {
        debug!(repo = %self.repo, number, "spawning `glab mr diff`");

        let number_str = number.to_string();
        let output = self
            .runner
            .run("glab", &["mr", "diff", &number_str])
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab mr diff: {e}")))?;

        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
        }

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    /// 获取指定 MR 的 patch 格式文本（含邮件头信息）。
    ///
    /// 调用 `glab api projects/{encoded}/merge_requests/{iid}.patch` 获取
    /// 包含 commit 元数据的 patch 格式输出，可用于 `git am`。
    ///
    /// # Errors
    ///
    /// 当 MR 不存在或 `glab` CLI 调用失败时返回错误。
    async fn patch(&self, number: u64) -> Result<String> {
        debug!(repo = %self.repo, number, "spawning `glab api mr patch`");

        let encoded_path = encode_project_path(&self.repo);
        let api_path = format!("projects/{encoded_path}/merge_requests/{number}.patch");

        let output = self
            .runner
            .run("glab", &["api", &api_path])
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab api mr patch: {e}")))?;

        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
        }

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    /// 查询仓库配置的默认分支（如 `main`、`dev`）。
    ///
    /// 调用 `glab repo view --output json` 并解析 `default_branch` 字段。
    ///
    /// # Errors
    ///
    /// 当 `glab` CLI 调用失败或响应无法解析时返回错误。
    async fn default_branch(&self) -> Result<String> {
        debug!(repo = %self.repo, "spawning `glab repo view`");

        let output = self
            .runner
            .run(
                "glab",
                &[
                    "repo",
                    "view",
                    "--repo",
                    &self.repo_target,
                    "--output",
                    "json",
                ],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
        }

        let resp: RepoViewResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(resp.default_branch)
    }
}

/// Parse MR IID from GitLab URL.
///
/// Extracts the numeric IID from URLs like:
/// - `https://gitlab.com/owner/repo/-/merge_requests/123`
/// - `https://gitlab.example.com/group/project/-/merge_requests/456`
fn parse_mr_iid_from_url(url: &str) -> Option<u64> {
    url.lines().find_map(|line| {
        let line = line.trim();
        if line.contains("/-/merge_requests/") {
            line.rsplit("/-/merge_requests/")
                .next()
                .and_then(|s| s.split('/').next())
                .and_then(|s| s.parse().ok())
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::{MockCommandRunner, SequencedMockCommandRunner};

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
            closes_issues: vec![],
        }
    }

    #[test]
    fn test_should_format_gitlab_body_with_closing_issues() {
        use gitflow_core::pr::format_closing_body;

        let body = Some("MR description".to_string());
        let issues = vec![42u64];
        let result = format_closing_body(&body, &issues, "Closes");
        assert_eq!(result, Some("MR description\n\nCloses #42".to_string()));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_glab_fails_for_create() {
        let runner = MockCommandRunner::failure(r#"{"message": "Validation failed"}"#, 256);
        let provider = GitLabMrProvider::with_runner("owner/repo", runner);

        let result = provider.create(sample_create_args()).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_create() {
        let runner = SequencedMockCommandRunner::from_results(&[
            (true, "https://gitlab.com/owner/repo/-/merge_requests/7"),
            (true, "not valid json"),
        ]);
        let provider = GitLabMrProvider::with_runner("owner/repo", runner);

        let result = provider.create(sample_create_args()).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_create_mr_without_output_json_and_refetch_via_view() {
        let runner = SequencedMockCommandRunner::from_results(&[
            (true, "https://gitlab.com/owner/repo/-/merge_requests/12"),
            (
                true,
                r#"{"iid":12,"title":"Feat","state":"opened","source_branch":"feat/x","target_branch":"main"}"#,
            ),
        ]);
        let provider = GitLabMrProvider::with_runner("owner/repo", runner.clone());

        let pr = provider
            .create(sample_create_args())
            .await
            .expect("should create");
        assert_eq!(pr.number, 12);
        let calls = runner.recorded_calls();
        assert!(!calls[0].1.contains(&"--output".to_string()));
        // 第二次调用是 mr view（保留 --output json）
        assert!(calls[1].1.contains(&"--output".to_string()));
    }

    #[tokio::test]
    async fn test_should_close_mr_without_output_json_and_refetch_via_view() {
        let runner = MockCommandRunner::success(
            r#"{"iid":12,"title":"Feat","state":"closed","source_branch":"feat/x","target_branch":"main"}"#,
        );
        let provider = GitLabMrProvider::with_runner("owner/repo", runner.clone());

        let pr = provider.close(12).await.expect("should close");

        assert_eq!(pr.number, 12);
        assert_eq!(pr.state, State::Closed);
        let calls = runner.recorded_calls();
        assert_eq!(
            calls[0].1,
            vec!["mr", "close", "12", "--repo", "owner/repo"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );
        assert!(calls[1].1.contains(&"--output".to_string()));
    }

    #[tokio::test]
    async fn test_should_use_explicit_repo_target_for_close() {
        let runner = MockCommandRunner::success(
            r#"{"iid":42,"title":"Fix","state":"closed","source_branch":"a","target_branch":"main"}"#,
        );
        let provider = GitLabMrProvider::with_runner_and_repo_target(
            "owner/repo",
            "https://192.168.230.23/iproost/proxy/api-src.git",
            runner.clone(),
        );

        let pr = provider.close(42).await.expect("close should succeed");

        assert_eq!(pr.number, 42);
        assert_eq!(
            runner.recorded_calls()[0].1,
            vec![
                "mr",
                "close",
                "42",
                "--repo",
                "https://192.168.230.23/iproost/proxy/api-src.git",
            ]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn test_should_reopen_mr_without_output_json_and_refetch_via_view() {
        let runner = MockCommandRunner::success(
            r#"{"iid":12,"title":"Feat","state":"opened","source_branch":"feat/x","target_branch":"main"}"#,
        );
        let provider = GitLabMrProvider::with_runner("owner/repo", runner.clone());

        let pr = provider.reopen(12).await.expect("should reopen");

        assert_eq!(pr.number, 12);
        assert_eq!(pr.state, State::Open);
        let calls = runner.recorded_calls();
        assert_eq!(
            calls[0].1,
            vec!["mr", "reopen", "12", "--repo", "owner/repo"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );
        assert!(calls[1].1.contains(&"--output".to_string()));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_glab_fails_for_list() {
        let runner = MockCommandRunner::failure(r#"{"message": "Forbidden"}"#, 256);
        let provider = GitLabMrProvider::with_runner("owner/repo", runner);

        let result = provider.list(ListPrArgs::default()).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_list() {
        let runner = MockCommandRunner::success("invalid");
        let provider = GitLabMrProvider::with_runner("owner/repo", runner);

        let result = provider.list(ListPrArgs::default()).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_list_all_mrs_with_all_flag() {
        let runner = MockCommandRunner::success("[]");
        let provider = GitLabMrProvider::with_runner("owner/repo", runner.clone());

        let _ = provider
            .list(ListPrArgs {
                state: Some(State::All),
                ..Default::default()
            })
            .await;

        assert_eq!(
            runner.recorded_calls()[0].1,
            vec![
                "mr",
                "list",
                "--repo",
                "owner/repo",
                "--output",
                "json",
                "--all"
            ]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_glab_fails_for_view() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not found"}"#, 256);
        let provider = GitLabMrProvider::with_runner("owner/repo", runner);

        let result = provider.view(999).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_view() {
        let runner = MockCommandRunner::success("not valid json");
        let provider = GitLabMrProvider::with_runner("owner/repo", runner);

        let result = provider.view(1).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_glab_fails_for_close() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not found"}"#, 256);
        let provider = GitLabMrProvider::with_runner("owner/repo", runner);

        let result = provider.close(42).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_glab_fails_for_reopen() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not found"}"#, 256);
        let provider = GitLabMrProvider::with_runner("owner/repo", runner);

        let result = provider.reopen(42).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_glab_fails_for_comment() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not found"}"#, 256);
        let provider = GitLabMrProvider::with_runner("owner/repo", runner);

        let result = provider.comment(42, "a comment").await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_post_mr_note_via_glab_api() {
        let runner = MockCommandRunner::success(
            r#"{"id":88,"body":"lgtm","author":{"username":"bob","id":2},"created_at":"2026-08-18T00:00:00Z"}"#,
        );
        let provider = GitLabMrProvider::with_runner("owner/repo", runner.clone());

        let comment = provider.comment(7, "lgtm").await.expect("should post");

        assert_eq!(comment.id, 88);
        assert_eq!(
            runner.recorded_calls()[0].1,
            vec![
                "api",
                "--method",
                "POST",
                "/projects/owner%2Frepo/merge_requests/7/notes",
                "-f",
                "body=lgtm",
            ]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn test_should_encode_nested_group_repo_path_for_mr_note() {
        let runner = MockCommandRunner::success(
            r#"{"id":88,"body":"lgtm","author":{"username":"bob","id":2},"created_at":"2026-08-18T00:00:00Z"}"#,
        );
        let provider = GitLabMrProvider::with_runner("group/subgroup/project", runner.clone());

        let comment = provider.comment(7, "lgtm").await.expect("should post");

        assert_eq!(comment.id, 88);
        assert_eq!(
            runner.recorded_calls()[0].1,
            vec![
                "api",
                "--method",
                "POST",
                "/projects/group%2Fsubgroup%2Fproject/merge_requests/7/notes",
                "-f",
                "body=lgtm",
            ]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_glab_fails_for_merge() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not mergeable"}"#, 256);
        let provider = GitLabMrProvider::with_runner("owner/repo", runner);

        let result = provider.merge(42, None, false).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_merge_without_merge_flag() {
        let runner = MockCommandRunner::success("Merged!");
        let provider = GitLabMrProvider::with_runner("owner/repo", runner.clone());

        let _ = provider
            .merge(9, Some(MergeStrategy::Merge), false)
            .await
            .expect("should merge");

        assert_eq!(
            runner.recorded_calls()[0].1,
            vec!["mr", "merge", "9", "--repo", "owner/repo"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn test_should_forward_auto_merge_and_report_not_merged() {
        let runner = MockCommandRunner::success("");
        let provider = GitLabMrProvider::with_runner("owner/repo", runner.clone());

        let result = provider
            .merge(9, None, true)
            .await
            .expect("should schedule merge");

        assert_eq!(
            runner.recorded_calls()[0].1,
            vec!["mr", "merge", "9", "--repo", "owner/repo", "--auto-merge"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );
        assert!(
            !result.merged,
            "a scheduled merge has not landed, so merged must be false"
        );
        let message = result.message.unwrap_or_default();
        assert!(
            message.contains("排队"),
            "empty glab stdout must be replaced by a scheduled-merge message, got {message:?}"
        );
    }

    #[tokio::test]
    async fn test_should_merge_without_strategy_flag() {
        let runner = MockCommandRunner::success("Merged!");
        let provider = GitLabMrProvider::with_runner("owner/repo", runner.clone());

        let _ = provider.merge(9, None, false).await.expect("should merge");

        assert_eq!(
            runner.recorded_calls()[0].1,
            vec!["mr", "merge", "9", "--repo", "owner/repo"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_glab_fails_for_checkout() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not found"}"#, 256);
        let provider = GitLabMrProvider::with_runner("owner/repo", runner);

        let result = provider.checkout(42).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_glab_fails_for_sync_branch() {
        let runner = MockCommandRunner::failure(r#"{"message": "Rebase failed"}"#, 256);
        let provider = GitLabMrProvider::with_runner("owner/repo", runner);

        let result = provider.sync_branch(42).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_glab_fails_for_mark_ready() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not found"}"#, 256);
        let provider = GitLabMrProvider::with_runner("owner/repo", runner);

        let result = provider.mark_ready(42).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_glab_fails_for_mark_wip() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not found"}"#, 256);
        let provider = GitLabMrProvider::with_runner("owner/repo", runner);

        let result = provider.mark_wip(42).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_mark_ready_with_mr_update_draft_false() {
        let runner = MockCommandRunner::success("");
        let provider = GitLabMrProvider::with_runner("owner/repo", runner.clone());
        provider
            .run_mr_update(5, false)
            .await
            .expect("should succeed");
        assert_eq!(
            runner.recorded_calls()[0].1,
            vec!["mr", "update", "5", "--repo", "owner/repo", "--draft=false"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn test_should_mark_wip_with_mr_update_draft_true() {
        let runner = MockCommandRunner::success("");
        let provider = GitLabMrProvider::with_runner("owner/repo", runner.clone());
        provider
            .run_mr_update(5, true)
            .await
            .expect("should succeed");
        assert_eq!(
            runner.recorded_calls()[0].1,
            vec!["mr", "update", "5", "--repo", "owner/repo", "--draft=true"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn test_should_fail_when_mr_update_glab_fails() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not found"}"#, 256);
        let provider = GitLabMrProvider::with_runner("owner/repo", runner);
        let result = provider.run_mr_update(5, false).await;
        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    // --- diff() tests ---

    #[tokio::test]
    async fn test_should_fetch_mr_diff() {
        use gitflow_core::pr::PrProvider;

        let diff_output = "diff --git a/src/lib.rs b/src/lib.rs\nindex 1234567..abcdefg \
                           100644\n--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1 +1 @@\n-old\n+new\n";
        let runner = MockCommandRunner::success(diff_output);
        let provider = GitLabMrProvider::with_runner("group/project", runner);

        let result = provider.diff(42).await.expect("diff should succeed");
        assert_eq!(result, diff_output);
    }

    #[tokio::test]
    async fn test_should_return_error_when_mr_diff_fails() {
        use gitflow_core::pr::PrProvider;

        let runner = MockCommandRunner::failure(r#"{"message": "404 Not Found"}"#, 256);
        let provider = GitLabMrProvider::with_runner("group/project", runner);

        let result = provider.diff(999).await;
        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    // --- patch() tests ---

    #[tokio::test]
    async fn test_should_fetch_mr_patch() {
        use gitflow_core::pr::PrProvider;

        let patch_output =
            "From abc123\nSubject: [PATCH] Update file\n\ndiff --git a/src/lib.rs b/src/lib.rs\n";
        let runner = MockCommandRunner::success(patch_output);
        let provider = GitLabMrProvider::with_runner("group/project", runner);

        let result = provider.patch(42).await.expect("patch should succeed");
        assert_eq!(result, patch_output);
    }

    #[tokio::test]
    async fn test_should_return_error_when_mr_patch_fails() {
        use gitflow_core::pr::PrProvider;

        let runner = MockCommandRunner::failure(r#"{"message": "404 Not Found"}"#, 256);
        let provider = GitLabMrProvider::with_runner("group/project", runner);

        let result = provider.patch(999).await;
        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    // --- default_branch() tests ---

    #[tokio::test]
    async fn test_should_return_default_branch_on_success() {
        let runner = MockCommandRunner::success(r#"{"default_branch":"dev"}"#);
        let provider = GitLabMrProvider::with_runner("group/project", runner);

        let result = provider.default_branch().await;

        assert!(result.is_ok());
        assert_eq!(result.expect("already checked"), "dev");
    }

    #[tokio::test]
    async fn test_should_use_repo_target_for_default_branch() {
        let runner = MockCommandRunner::success(r#"{"default_branch":"dev"}"#);
        let provider = GitLabMrProvider::with_runner_and_repo_target(
            "group/project",
            "https://gitlab.example.com/group/project.git",
            runner.clone(),
        );

        let _ = provider.default_branch().await;

        let calls = runner.recorded_calls();
        assert_eq!(calls[0].0, "glab");
        assert_eq!(
            calls[0].1,
            vec![
                "repo",
                "view",
                "--repo",
                "https://gitlab.example.com/group/project.git",
                "--output",
                "json",
            ]
        );
    }

    #[tokio::test]
    async fn test_should_return_error_when_repo_view_fails() {
        let runner = MockCommandRunner::failure("glab: 404 Not Found", 1);
        let provider = GitLabMrProvider::with_runner("group/nonexistent", runner);

        let result = provider.default_branch().await;

        assert!(result.is_err());
    }
}
