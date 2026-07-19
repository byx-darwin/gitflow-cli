//! GitHub Issue 提供者实现。
//!
//! 通过 `gh` CLI 实现 [`IssueProvider`] trait，支持 Issue 的创建、列表、查看、
//! 关闭、重新打开、评论及标签管理。
//! 所有方法通过 `tokio::process::Command` 调用 `gh`，捕获 stdout 并解析 JSON。

use async_trait::async_trait;
use gitflow_cli_core::{
    CoreError, Result,
    issue::{CreateIssueArgs, IssueData, IssueProvider, ListIssueArgs},
    types::{CommentData, State},
};
use tracing::debug;

use crate::{
    error::parse_gh_error,
    runner::{CommandRunner, RealCommandRunner},
};

/// `gh issue` 请求的 JSON 字段列表。
const ISSUE_FIELDS: &str =
    "number,title,body,state,labels,author,assignees,createdAt,updatedAt,url";

/// GitHub Issue 提供者，通过 `gh` CLI 操作。
///
/// 该结构体通过调用 `gh` CLI 实现 [`IssueProvider`] trait 的所有方法，
/// 使上层命令能够以统一的方式操作 GitHub Issue。
///
/// 命令执行通过 [`CommandRunner`] 抽象，生产环境默认使用
/// [`RealCommandRunner`]，测试可注入自定义 runner 以模拟成功或失败场景。
///
/// # Examples
///
/// ```no_run
/// use gitflow_cli_github::GitHubIssueProvider;
///
/// let provider = GitHubIssueProvider::new("octocat/hello-world");
/// ```
#[derive(Debug, Clone)]
pub struct GitHubIssueProvider<R: CommandRunner = RealCommandRunner> {
    /// GitHub `owner/repo`，如 `"byx-darwin/gitflow-cli"`。
    repo: String,
    /// 用于执行 `gh` CLI 命令的 runner。
    runner: R,
}

impl GitHubIssueProvider<RealCommandRunner> {
    /// 创建新的 GitHub Issue 提供者，使用真实的进程执行器。
    ///
    /// `repo` 格式为 `owner/repo`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> Self {
        Self {
            repo: repo.into(),
            runner: RealCommandRunner,
        }
    }
}

impl<R: CommandRunner> GitHubIssueProvider<R> {
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

    /// 创建缺失的标签（如已存在则忽略"已存在"错误）。
    ///
    /// 调用 `gh label create <name> --color ededed --repo <repo> --force`。
    /// `--force` 确保在竞态条件（标签被并发创建）下仍保持幂等。
    ///
    /// # Errors
    ///
    /// 当 `gh label create` 调用失败时返回错误。
    async fn ensure_label_exists(&self, name: &str) -> Result<()> {
        debug!(repo = %self.repo, name, "auto-creating missing label via `gh label create`");

        let output = self
            .runner
            .run(
                "gh",
                &[
                    "label", "create", name, "--color", "ededed", "--repo", &self.repo, "--force",
                ],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh label create: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!(
                "Failed to auto-create label '{name}': {gh_err}"
            )));
        }

        Ok(())
    }
}

#[async_trait]
impl<R: CommandRunner + 'static> IssueProvider for GitHubIssueProvider<R> {
    async fn create(&self, args: CreateIssueArgs) -> Result<IssueData> {
        let labels_joined = args.labels.join(",");
        let assignees_joined = args.assignees.join(",");

        let mut cmd_args: Vec<&str> = vec![
            "issue",
            "create",
            "--repo",
            &self.repo,
            "--title",
            &args.title,
        ];

        if let Some(body) = &args.body {
            cmd_args.push("--body");
            cmd_args.push(body);
        }

        if !args.labels.is_empty() {
            cmd_args.push("--label");
            cmd_args.push(&labels_joined);
        }

        if !args.assignees.is_empty() {
            cmd_args.push("--assignee");
            cmd_args.push(&assignees_joined);
        }

        debug!(repo = %self.repo, title = %args.title, "spawning `gh issue create`");

        let output = self
            .runner
            .run("gh", &cmd_args)
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            // `gh issue create` fails when a requested label doesn't exist.
            // Auto-create missing labels and retry once.
            let missing = extract_missing_labels_from_error(&output.stderr);
            if !missing.is_empty() {
                debug!(
                    repo = %self.repo,
                    missing_count = missing.len(),
                    "auto-creating missing label(s) before retrying issue create"
                );
                for label in &missing {
                    self.ensure_label_exists(label).await?;
                }

                let retry_output = self.runner.run("gh", &cmd_args).await.map_err(|e| {
                    CoreError::Platform(format!("Failed to spawn gh on retry: {e}"))
                })?;

                if !retry_output.status.success() {
                    let gh_err = parse_gh_error(&retry_output.stderr);
                    return Err(CoreError::Platform(format!("{gh_err}")));
                }

                let stdout = String::from_utf8_lossy(&retry_output.stdout);
                let issue_number = parse_issue_number_from_url(&stdout).ok_or_else(|| {
                    CoreError::Platform(format!("Failed to parse issue URL from output: {stdout}"))
                })?;
                return self.view(issue_number).await;
            }

            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        // Parse the issue URL from stdout (format: https://github.com/owner/repo/issues/123)
        let stdout = String::from_utf8_lossy(&output.stdout);
        let issue_number = parse_issue_number_from_url(&stdout).ok_or_else(|| {
            CoreError::Platform(format!("Failed to parse issue URL from output: {stdout}"))
        })?;

        // Fetch full issue details via view
        self.view(issue_number).await
    }

    async fn list(&self, args: ListIssueArgs) -> Result<Vec<IssueData>> {
        let mut cmd_args: Vec<&str> = vec![
            "issue",
            "list",
            "--repo",
            &self.repo,
            "--json",
            ISSUE_FIELDS,
        ];

        if let Some(state) = &args.state {
            cmd_args.push("--state");
            cmd_args.push(match state {
                State::Open => "open",
                State::Closed => "closed",
            });
        }

        if let Some(ref search) = args.search {
            cmd_args.push("--search");
            cmd_args.push(search);
        }

        let limit_str = args.limit.map(|limit| limit.to_string());
        if let Some(ref limit) = limit_str {
            cmd_args.push("--limit");
            cmd_args.push(limit);
        }

        debug!(repo = %self.repo, "spawning `gh issue list`");

        let output = self
            .runner
            .run("gh", &cmd_args)
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        let issues: Vec<IssueData> =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(issues)
    }

    async fn view(&self, number: u64) -> Result<IssueData> {
        debug!(repo = %self.repo, number, "spawning `gh issue view`");

        let number_str = number.to_string();
        let output = self
            .runner
            .run(
                "gh",
                &[
                    "issue",
                    "view",
                    &number_str,
                    "--repo",
                    &self.repo,
                    "--json",
                    ISSUE_FIELDS,
                ],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        let issue: IssueData =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(issue)
    }

    /// 关闭指定编号的 Issue。
    ///
    /// 调用 `gh issue close <number> --repo <repo>` 关闭 Issue，
    /// 并返回更新后的完整 Issue 数据。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在、已关闭或 `gh` CLI 调用失败时返回错误。
    async fn close(&self, number: u64) -> Result<IssueData> {
        debug!(repo = %self.repo, number, "spawning `gh issue close`");

        let number_str = number.to_string();
        let output = self
            .runner
            .run("gh", &["issue", "close", &number_str, "--repo", &self.repo])
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        // Fetch updated issue details
        self.view(number).await
    }

    /// 重新打开指定编号的 Issue。
    ///
    /// 调用 `gh issue reopen <number> --repo <repo>` 重新打开已关闭的 Issue，
    /// 并返回更新后的完整 Issue 数据。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在、未关闭或 `gh` CLI 调用失败时返回错误。
    async fn reopen(&self, number: u64) -> Result<IssueData> {
        debug!(repo = %self.repo, number, "spawning `gh issue reopen`");

        let number_str = number.to_string();
        let output = self
            .runner
            .run(
                "gh",
                &["issue", "reopen", &number_str, "--repo", &self.repo],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        // Fetch updated issue details
        self.view(number).await
    }

    /// 在指定 Issue 上添加评论。
    ///
    /// 调用 `gh issue comment <number> --repo <repo> --body "<body>"` 发布评论，
    /// 然后通过 `gh api` 获取最新评论数据。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在、`body` 为空或 `gh` CLI 调用失败时返回错误。
    async fn comment(&self, number: u64, body: &str) -> Result<CommentData> {
        debug!(repo = %self.repo, number, "spawning `gh issue comment`");

        // 1. 执行 gh issue comment 发布评论（不返回 JSON）
        let number_str = number.to_string();
        let output = self
            .runner
            .run(
                "gh",
                &[
                    "issue",
                    "comment",
                    &number_str,
                    "--repo",
                    &self.repo,
                    "--body",
                    body,
                ],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        // 2. 使用 gh api 获取该 issue 的最新评论
        let api_path = format!(
            "repos/{repo}/issues/{number}/comments?per_page=1",
            repo = self.repo,
            number = number
        );
        let api_output = self
            .runner
            .run("gh", &["api", &api_path])
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh api: {e}")))?;

        if !api_output.status.success() {
            let gh_err = String::from_utf8_lossy(&api_output.stderr);
            return Err(CoreError::Platform(format!(
                "Failed to fetch comment via gh api: {gh_err}"
            )));
        }

        // 3. 解析 API 响应（返回的是数组，取最后一个）
        let comments: Vec<GitHubCommentApiResponse> =
            serde_json::from_slice(&api_output.stdout).map_err(CoreError::Serialization)?;

        let comment = comments
            .into_iter()
            .next()
            .ok_or_else(|| CoreError::Platform("No comment returned from gh api".to_string()))?;

        Ok(comment.into())
    }

    /// 为指定 Issue 添加一个或多个标签。
    ///
    /// 调用 `gh issue edit <number> --repo <repo> --add-label <label>` 逐个添加标签。
    /// 如果 `labels` 为空，不进行任何调用并返回成功。
    ///
    /// # 自动创建缺失标签
    ///
    /// 当 `gh issue edit --add-label` 因标签不存在而失败时，本方法会自动调用
    /// `gh label create` 创建缺失的标签（使用默认颜色 `ededed`），然后重试原操作。
    /// 这避免了手动同步仓库标签列表的繁琐流程。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在、标签创建失败或 `gh` CLI 调用失败时返回错误。
    async fn add_labels(&self, number: u64, labels: &[String]) -> Result<()> {
        debug!(
            repo = %self.repo,
            number,
            label_count = labels.len(),
            "spawning `gh issue edit --add-label`"
        );

        let number_str = number.to_string();
        let mut cmd_args: Vec<&str> = vec!["issue", "edit", &number_str, "--repo", &self.repo];

        for label in labels {
            cmd_args.push("--add-label");
            cmd_args.push(label);
        }

        let output = self
            .runner
            .run("gh", &cmd_args)
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if output.status.success() {
            return Ok(());
        }

        // gh issue edit --add-label fails when a label doesn't exist in the repo.
        // Auto-create the missing label(s) and retry once.
        let missing = extract_missing_labels_from_error(&output.stderr);
        if missing.is_empty() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        debug!(
            repo = %self.repo,
            missing_count = missing.len(),
            "auto-creating missing label(s) before retry"
        );

        for label in &missing {
            self.ensure_label_exists(label).await?;
        }

        // Retry the original add-label command.
        let retry_output = self
            .runner
            .run("gh", &cmd_args)
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh on retry: {e}")))?;

        if !retry_output.status.success() {
            let gh_err = parse_gh_error(&retry_output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        Ok(())
    }

    /// 从指定 Issue 移除一个标签。
    ///
    /// 调用 `gh issue edit <number> --repo <repo> --remove-label <label>` 移除标签。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在、标签未附加到该 Issue 或 `gh` CLI 调用失败时返回错误。
    async fn remove_label(&self, number: u64, label: &str) -> Result<()> {
        debug!(repo = %self.repo, number, label, "spawning `gh issue edit --remove-label`");

        let number_str = number.to_string();
        let output = self
            .runner
            .run(
                "gh",
                &[
                    "issue",
                    "edit",
                    &number_str,
                    "--repo",
                    &self.repo,
                    "--remove-label",
                    label,
                ],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        Ok(())
    }
}

/// GitHub API 评论响应结构。
///
/// 用于解析 `gh api repos/{owner}/{repo}/issues/{number}/comments` 的返回数据。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct GitHubCommentApiResponse {
    pub id: u64,
    pub body: String,
    pub user: GitHubUser,
    pub created_at: String,
}

/// GitHub API 用户结构。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub id: u64,
}

impl From<GitHubCommentApiResponse> for CommentData {
    fn from(api: GitHubCommentApiResponse) -> Self {
        Self {
            id: api.id,
            body: api.body,
            author: gitflow_cli_core::types::UserSummary {
                login: api.user.login,
                id: api.user.id.to_string(),
            },
            created_at: api.created_at.parse().unwrap_or_else(|_| {
                tracing::warn!(created_at = %api.created_at, "Failed to parse comment created_at, using epoch");
                chrono::DateTime::UNIX_EPOCH
            }),
        }
    }
}

/// Parse issue number from GitHub URL.
///
/// Extracts the numeric issue number from URLs like:
/// - `https://github.com/owner/repo/issues/123`
/// - `https://github.enterprise.com/org/project/issues/456`
fn parse_issue_number_from_url(url: &str) -> Option<u64> {
    url.lines().find_map(|line| {
        let line = line.trim();
        if line.contains("/issues/") {
            line.rsplit("/issues/")
                .next()
                .and_then(|s| s.split('/').next())
                .and_then(|s| s.parse().ok())
        } else {
            None
        }
    })
}

/// 从 `gh issue edit --add-label` 的 stderr 中提取缺失的标签名。
///
/// `gh` 对缺失标签的错误格式为：
/// ```text
/// failed to update https://github.com/owner/repo/issues/18: 'type:enhancement' not found
/// ```
/// 或多个标签缺失时：
/// ```text
/// failed to update ...: 'bug' not found, 'priority:high' not found
/// ```
///
/// 本函数扫描所有 `'<label>' not found` 模式并返回标签名列表。
/// 若 stderr 不含该模式（例如鉴权错误），返回空列表，调用方据此判断
/// 是否为"标签缺失"错误并决定是否重试。
fn extract_missing_labels_from_error(stderr: &[u8]) -> Vec<String> {
    let text = String::from_utf8_lossy(stderr);
    let mut labels = Vec::new();
    let mut search_from: usize = 0;

    while let Some(rel_open) = text[search_from..].find('\'') {
        let open_pos = search_from + rel_open + 1; // position after opening quote
        let Some(rel_close) = text[open_pos..].find('\'') else {
            break; // no matching close quote
        };
        let close_pos = open_pos + rel_close;
        let after_close = &text[close_pos + 1..];

        if after_close.starts_with(" not found") {
            let label = text[open_pos..close_pos].to_string();
            if !label.is_empty() {
                labels.push(label);
            }
        }
        search_from = close_pos + 1;
    }

    labels
}

#[cfg(test)]
mod tests {
    use gitflow_cli_core::types::UserSummary;

    use super::*;
    use crate::runner::{MockCommandRunner, SequencedMockCommandRunner};

    #[test]
    fn test_should_construct_github_issue_provider() {
        let provider = GitHubIssueProvider::new("octocat/hello-world");
        assert_eq!(provider.repo, "octocat/hello-world");
    }

    #[test]
    fn test_should_construct_github_issue_provider_from_string() {
        let repo = String::from("octocat/hello-world");
        let provider = GitHubIssueProvider::new(repo);
        assert_eq!(provider.repo, "octocat/hello-world");
    }

    #[test]
    fn test_should_deserialize_issue_data_from_gh_output() {
        // 模拟 `gh issue view --json ...` 的实际输出
        let gh_json = br#"{
            "number": 42,
            "title": "Fix login bug",
            "body": "Reproduced on v1.2.3",
            "state": "open",
            "labels": [
                {"name": "bug", "color": "d73a4a", "description": "Something isn't working"}
            ],
            "author": {"login": "octocat", "id": "1"},
            "assignees": [{"login": "alice", "id": "7"}],
            "createdAt": "2026-01-15T09:30:00Z",
            "updatedAt": "2026-01-16T11:00:00Z",
            "url": "https://github.com/octocat/hello-world/issues/42"
        }"#;

        let issue: IssueData = serde_json::from_slice(gh_json).expect("valid IssueData JSON");
        assert_eq!(issue.number, 42);
        assert_eq!(issue.title, "Fix login bug");
        assert_eq!(issue.state, State::Open);
        assert_eq!(issue.labels.len(), 1);
        assert_eq!(issue.author.login, "octocat");
        assert_eq!(issue.assignees.len(), 1);
        assert_eq!(
            issue.url,
            "https://github.com/octocat/hello-world/issues/42"
        );
    }

    #[test]
    fn test_should_deserialize_empty_issue_list_from_gh_output() {
        let gh_json = b"[]";
        let issues: Vec<IssueData> = serde_json::from_slice(gh_json).expect("valid IssueData list");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_should_debug_format_provider() {
        let provider = GitHubIssueProvider::new("octocat/hello-world");
        let debug = format!("{provider:?}");
        assert!(debug.contains("GitHubIssueProvider"));
        assert!(debug.contains("octocat/hello-world"));
    }

    // --- close/reopen: deserialized IssueData tests ---

    #[test]
    fn test_should_deserialize_closed_issue_from_gh_close_output() {
        // 模拟 `gh issue close --json ...` 的返回数据
        let gh_json = br#"{
            "number": 10,
            "title": "Fixed typo",
            "body": null,
            "state": "closed",
            "labels": [],
            "author": {"login": "dev", "id": "5"},
            "assignees": [],
            "createdAt": "2026-06-01T08:00:00Z",
            "updatedAt": "2026-06-02T12:00:00Z",
            "url": "https://github.com/octocat/hello-world/issues/10"
        }"#;

        let issue: IssueData = serde_json::from_slice(gh_json).expect("valid closed IssueData");
        assert_eq!(issue.number, 10);
        assert_eq!(issue.state, State::Closed);
        assert_eq!(issue.title, "Fixed typo");
    }

    #[test]
    fn test_should_deserialize_reopened_issue_from_gh_reopen_output() {
        let gh_json = br#"{
            "number": 10,
            "title": "Fixed typo",
            "body": null,
            "state": "open",
            "labels": [],
            "author": {"login": "dev", "id": "5"},
            "assignees": [],
            "createdAt": "2026-06-01T08:00:00Z",
            "updatedAt": "2026-06-03T09:00:00Z",
            "url": "https://github.com/octocat/hello-world/issues/10"
        }"#;

        let issue: IssueData = serde_json::from_slice(gh_json).expect("valid reopened IssueData");
        assert_eq!(issue.number, 10);
        assert_eq!(issue.state, State::Open);
    }

    // --- comment: CommentData deserialization tests ---

    #[test]
    fn test_should_deserialize_comment_data_from_gh_comment_output() {
        // 模拟 `gh issue comment --json id,body,author,createdAt` 的输出
        let gh_json = br#"{
            "id": 1001,
            "body": "Thanks for reporting, looking into it.",
            "author": {"login": "maintainer", "id": "42"},
            "createdAt": "2026-06-15T14:00:00Z"
        }"#;

        let comment: CommentData = serde_json::from_slice(gh_json).expect("valid CommentData");
        assert_eq!(comment.id, 1001);
        assert_eq!(comment.body, "Thanks for reporting, looking into it.");
        assert_eq!(comment.author.login, "maintainer");
        assert_eq!(comment.author.id, "42");
    }

    #[test]
    fn test_should_roundtrip_comment_data_via_serde() {
        let comment = CommentData {
            id: 77,
            body: "reviewed".into(),
            author: UserSummary {
                login: "alice".into(),
                id: "3".to_string(),
            },
            created_at: "2026-05-01T00:00:00Z".parse().expect("valid date"),
        };
        let json = serde_json::to_string(&comment).expect("serialize");
        let round_tripped: CommentData = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(round_tripped.id, comment.id);
        assert_eq!(round_tripped.body, comment.body);
        assert_eq!(round_tripped.author.login, comment.author.login);
    }

    // --- GitHubCommentApiResponse conversion tests ---

    #[test]
    fn test_should_convert_github_api_response_to_comment_data() {
        let api_response = GitHubCommentApiResponse {
            id: 12345,
            body: "Test comment body".to_string(),
            user: GitHubUser {
                login: "testuser".to_string(),
                id: 42,
            },
            created_at: "2026-07-08T10:30:00Z".to_string(),
        };

        let comment_data: CommentData = api_response.into();

        assert_eq!(comment_data.id, 12345);
        assert_eq!(comment_data.body, "Test comment body");
        assert_eq!(comment_data.author.login, "testuser");
        assert_eq!(comment_data.author.id, "42");
    }

    #[test]
    fn test_should_handle_invalid_date_in_api_response() {
        let api_response = GitHubCommentApiResponse {
            id: 1,
            body: "test".to_string(),
            user: GitHubUser {
                login: "user".to_string(),
                id: 1,
            },
            created_at: "invalid-date".to_string(),
        };

        let comment_data: CommentData = api_response.into();
        // Should fall back to UNIX_EPOCH
        assert_eq!(comment_data.created_at, chrono::DateTime::UNIX_EPOCH);
    }

    // --- add_labels / remove_label: unit tests for provider ---

    #[test]
    fn test_should_create_provider_with_different_repos() {
        let r1 = GitHubIssueProvider::new("org/repo-a");
        let r2 = GitHubIssueProvider::new("org/repo-b");
        assert_eq!(r1.repo, "org/repo-a");
        assert_eq!(r2.repo, "org/repo-b");
    }

    #[test]
    fn test_should_clone_github_issue_provider() {
        let original = GitHubIssueProvider::new("owner/repo");
        let cloned = original.clone();
        assert_eq!(original.repo, cloned.repo);
    }

    #[test]
    fn test_should_parse_issue_number_from_github_url() {
        assert_eq!(
            parse_issue_number_from_url("https://github.com/owner/repo/issues/123"),
            Some(123)
        );
    }

    #[test]
    fn test_should_parse_issue_number_from_enterprise_url() {
        assert_eq!(
            parse_issue_number_from_url("https://github.enterprise.com/org/project/issues/456"),
            Some(456)
        );
    }

    #[test]
    fn test_should_parse_issue_number_from_multiline_output() {
        let output = "Creating issue...\nhttps://github.com/owner/repo/issues/789\nDone.";
        assert_eq!(parse_issue_number_from_url(output), Some(789));
    }

    #[test]
    fn test_should_return_none_for_invalid_url() {
        assert_eq!(parse_issue_number_from_url("not a url"), None);
    }

    #[test]
    fn test_should_return_none_for_url_without_number() {
        assert_eq!(
            parse_issue_number_from_url("https://github.com/owner/repo/issues/"),
            None
        );
    }

    // --- Failure-path tests using an injected MockCommandRunner ---

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_fails_for_view() {
        let runner = MockCommandRunner::failure(r#"{"message": "Issue not found"}"#, 256);
        let provider = GitHubIssueProvider::with_runner("owner/repo", runner);

        let result = provider.view(999).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_view() {
        let runner = MockCommandRunner::success("not valid json");
        let provider = GitHubIssueProvider::with_runner("owner/repo", runner);

        let result = provider.view(1).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_fails_for_list() {
        let runner = MockCommandRunner::failure(r#"{"message": "Forbidden"}"#, 256);
        let provider = GitHubIssueProvider::with_runner("owner/repo", runner);

        let result = provider.list(ListIssueArgs::default()).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_list() {
        let runner = MockCommandRunner::success("invalid");
        let provider = GitHubIssueProvider::with_runner("owner/repo", runner);

        let result = provider.list(ListIssueArgs::default()).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Serialization(_)
        ));
    }

    fn sample_create_args() -> CreateIssueArgs {
        CreateIssueArgs {
            title: "Bug report".to_string(),
            body: Some("Steps to reproduce".to_string()),
            labels: vec!["bug".to_string()],
            assignees: vec!["octocat".to_string()],
        }
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_fails_for_create() {
        let runner = MockCommandRunner::failure(r#"{"message": "Validation failed"}"#, 256);
        let provider = GitHubIssueProvider::with_runner("owner/repo", runner);

        let result = provider.create(sample_create_args()).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_create() {
        // `create` succeeds and parses the issue number from this URL, then delegates
        // to `view`, which receives the same non-JSON stdout and fails to deserialize.
        let runner = MockCommandRunner::success("https://github.com/owner/repo/issues/7");
        let provider = GitHubIssueProvider::with_runner("owner/repo", runner);

        let result = provider.create(sample_create_args()).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_fails_for_close() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not found"}"#, 256);
        let provider = GitHubIssueProvider::with_runner("owner/repo", runner);

        let result = provider.close(42).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_close() {
        // `close` succeeds, then `view` fails to deserialize the non-JSON stdout.
        let runner = MockCommandRunner::success("invalid");
        let provider = GitHubIssueProvider::with_runner("owner/repo", runner);

        let result = provider.close(42).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_fails_for_reopen() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not found"}"#, 256);
        let provider = GitHubIssueProvider::with_runner("owner/repo", runner);

        let result = provider.reopen(42).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_reopen() {
        // `reopen` succeeds, then `view` fails to deserialize the non-JSON stdout.
        let runner = MockCommandRunner::success("invalid");
        let provider = GitHubIssueProvider::with_runner("owner/repo", runner);

        let result = provider.reopen(42).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_fails_for_comment() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not found"}"#, 256);
        let provider = GitHubIssueProvider::with_runner("owner/repo", runner);

        let result = provider.comment(42, "a comment").await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_comment() {
        // The `gh issue comment` call succeeds, then the `gh api` call returns the same
        // non-JSON stdout that fails to deserialize into the comment response array.
        let runner = MockCommandRunner::success("invalid");
        let provider = GitHubIssueProvider::with_runner("owner/repo", runner);

        let result = provider.comment(42, "a comment").await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_fails_for_add_labels() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not found"}"#, 256);
        let provider = GitHubIssueProvider::with_runner("owner/repo", runner);

        let result = provider.add_labels(42, &["bug".to_string()]).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_fails_for_remove_label() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not found"}"#, 256);
        let provider = GitHubIssueProvider::with_runner("owner/repo", runner);

        let result = provider.remove_label(42, "bug").await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    // --- extract_missing_labels_from_error: pure-function tests ---

    #[test]
    fn test_should_extract_single_missing_label_from_gh_stderr() {
        let stderr = b"failed to update https://github.com/owner/repo/issues/18: 'type:enhancement' not found\nfailed to update 1 issue";
        let missing = extract_missing_labels_from_error(stderr);
        assert_eq!(missing, vec!["type:enhancement".to_string()]);
    }

    #[test]
    fn test_should_extract_multiple_missing_labels_from_gh_stderr() {
        let stderr = b"failed to update https://github.com/owner/repo/issues/5: 'bug' not found, 'priority:high' not found\nfailed to update 1 issue";
        let missing = extract_missing_labels_from_error(stderr);
        assert_eq!(
            missing,
            vec!["bug".to_string(), "priority:high".to_string()]
        );
    }

    #[test]
    fn test_should_return_empty_when_no_label_not_found_in_stderr() {
        let stderr = b"gh: Not logged in. Please run `gh auth login`";
        let missing = extract_missing_labels_from_error(stderr);
        assert!(missing.is_empty());
    }

    #[test]
    fn test_should_return_empty_for_empty_stderr() {
        let missing = extract_missing_labels_from_error(b"");
        assert!(missing.is_empty());
    }

    #[test]
    fn test_should_handle_label_with_special_characters() {
        let stderr = b"failed to update https://github.com/o/r/issues/1: 'type: enhancement / bug' not found";
        let missing = extract_missing_labels_from_error(stderr);
        assert_eq!(missing, vec!["type: enhancement / bug".to_string()]);
    }

    // --- add_labels: auto-create missing labels (RED phase) ---

    #[tokio::test]
    async fn test_should_auto_create_label_and_retry_on_not_found() {
        // Sequence:
        // 1. `gh issue edit 18 --add-label type:enhancement` → fails (label not found)
        // 2. `gh label create type:enhancement --color ededed --repo owner/repo` → succeeds
        // 3. `gh issue edit 18 --add-label type:enhancement` → succeeds (retry)
        let runner = SequencedMockCommandRunner::from_results(&[
            (
                false,
                "failed to update https://github.com/owner/repo/issues/18: 'type:enhancement' not \
                 found\nfailed to update 1 issue",
            ),
            (true, "Created label type:enhancement"),
            (true, ""),
        ]);
        let provider = GitHubIssueProvider::with_runner("owner/repo", runner);

        let result = provider
            .add_labels(18, &["type:enhancement".to_string()])
            .await;

        assert!(result.is_ok(), "expected Ok, got {result:?}");
    }

    #[tokio::test]
    async fn test_should_auto_create_multiple_labels_and_retry() {
        // Sequence:
        // 1. `gh issue edit 5 --add-label bug --add-label priority:high` → fails (both missing)
        // 2. `gh label create bug` → succeeds
        // 3. `gh label create priority:high` → succeeds
        // 4. `gh issue edit 5 --add-label bug --add-label priority:high` → succeeds (retry)
        let runner = SequencedMockCommandRunner::from_results(&[
            (
                false,
                "failed to update https://github.com/o/r/issues/5: 'bug' not found, \
                 'priority:high' not found\nfailed to update 1 issue",
            ),
            (true, "Created label bug"),
            (true, "Created label priority:high"),
            (true, ""),
        ]);
        let provider = GitHubIssueProvider::with_runner("o/r", runner);

        let result = provider
            .add_labels(5, &["bug".to_string(), "priority:high".to_string()])
            .await;

        assert!(result.is_ok(), "expected Ok, got {result:?}");
    }

    #[tokio::test]
    async fn test_should_propagate_error_when_label_creation_fails() {
        // Sequence:
        // 1. `gh issue edit 1 --add-label ghost` → fails (label not found)
        // 2. `gh label create ghost` → also fails (permission denied)
        let runner = SequencedMockCommandRunner::from_results(&[
            (
                false,
                "failed to update https://github.com/o/r/issues/1: 'ghost' not found",
            ),
            (false, "gh: 403 Forbidden"),
        ]);
        let provider = GitHubIssueProvider::with_runner("o/r", runner);

        let result = provider.add_labels(1, &["ghost".to_string()]).await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("403") || err.contains("Forbidden"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test]
    async fn test_should_not_retry_on_non_label_not_found_error() {
        // An auth error should propagate directly without any label create calls.
        // Only one response in the sequence — if the runner tries a second call,
        // SequencedMockCommandRunner will return an error ("no more responses").
        let runner = SequencedMockCommandRunner::from_results(&[(
            false,
            "gh: Not logged in. Please run `gh auth login`",
        )]);
        let provider = GitHubIssueProvider::with_runner("o/r", runner);

        let result = provider.add_labels(1, &["bug".to_string()]).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_should_propagate_retry_error_when_second_add_also_fails() {
        // Sequence:
        // 1. `gh issue edit 1 --add-label bug` → fails (label not found)
        // 2. `gh label create bug` → succeeds
        // 3. `gh issue edit 1 --add-label bug` → fails again (unrelated error)
        let runner = SequencedMockCommandRunner::from_results(&[
            (
                false,
                "failed to update https://github.com/o/r/issues/1: 'bug' not found",
            ),
            (true, "Created label bug"),
            (false, "gh: 500 Internal Server Error"),
        ]);
        let provider = GitHubIssueProvider::with_runner("o/r", runner);

        let result = provider.add_labels(1, &["bug".to_string()]).await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("500") || err.contains("Internal"),
            "unexpected error: {err}"
        );
    }

    // --- create: auto-create missing labels (RED phase) ---

    fn create_args_with_labels(labels: Vec<String>) -> CreateIssueArgs {
        CreateIssueArgs {
            title: "New feature".to_string(),
            body: Some("Description".to_string()),
            labels,
            assignees: vec![],
        }
    }

    #[tokio::test]
    async fn test_should_auto_create_label_and_retry_on_create_issue() {
        // Sequence:
        // 1. `gh issue create` → fails (label not found)
        // 2. `gh label create type:enhancement` → succeeds
        // 3. `gh issue create` (retry) → succeeds, returns URL
        // 4. `gh issue view <number>` → returns issue JSON
        let runner = SequencedMockCommandRunner::from_results(&[
            (false, "could not add label: 'type:enhancement' not found"),
            (true, "Created label type:enhancement"),
            (true, "https://github.com/owner/repo/issues/42"),
            (
                true,
                r#"{"number":42,"title":"New feature","body":"Description","state":"open","labels":[{"name":"type:enhancement","color":"ededed"}],"author":{"login":"octocat","id":"1"},"assignees":[],"createdAt":"2026-01-01T00:00:00Z","updatedAt":"2026-01-01T00:00:00Z","url":"https://github.com/owner/repo/issues/42"}"#,
            ),
        ]);
        let provider = GitHubIssueProvider::with_runner("owner/repo", runner);

        let result = provider
            .create(create_args_with_labels(vec![
                "type:enhancement".to_string(),
            ]))
            .await;

        assert!(result.is_ok(), "expected Ok, got {result:?}");
        let issue = result.expect("already checked");
        assert_eq!(issue.number, 42);
    }

    #[tokio::test]
    async fn test_should_propagate_error_when_label_create_fails_on_create_issue() {
        // Sequence:
        // 1. `gh issue create` → fails (label not found)
        // 2. `gh label create ghost` → also fails (permission denied)
        let runner = SequencedMockCommandRunner::from_results(&[
            (false, "could not add label: 'ghost' not found"),
            (false, "gh: 403 Forbidden"),
        ]);
        let provider = GitHubIssueProvider::with_runner("o/r", runner);

        let result = provider
            .create(create_args_with_labels(vec!["ghost".to_string()]))
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_should_not_retry_create_on_non_label_error() {
        // Only one response — any extra call will error with "no more responses".
        let runner = SequencedMockCommandRunner::from_results(&[(false, "gh: Not logged in")]);
        let provider = GitHubIssueProvider::with_runner("o/r", runner);

        let result = provider
            .create(create_args_with_labels(vec!["bug".to_string()]))
            .await;

        assert!(result.is_err());
    }
}
