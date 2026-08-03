//! GitHub Issue 提供者实现。
//!
//! 通过 `gh` CLI 实现 [`IssueProvider`] trait，支持 Issue 的创建、列表、查看、
//! 关闭、重新打开、评论及标签管理。
//! 所有方法通过 `tokio::process::Command` 调用 `gh`，捕获 stdout 并解析 JSON。

use async_trait::async_trait;
use gitflow_cli_core::{
    CoreError, Result,
    issue::{CreateIssueArgs, IssueData, IssueProvider, ListIssueArgs},
    types::{CommentData, Label, State, UserSummary},
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
            let mut gh_err = parse_gh_error(&output.stderr);
            gh_err.user_message = format!("自动创建标签 '{name}' 失败：{}", gh_err.user_message);
            return Err(gh_err.into());
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
                    return Err(parse_gh_error(&retry_output.stderr).into());
                }

                let stdout = String::from_utf8_lossy(&retry_output.stdout);
                let issue_number = parse_issue_number_from_url(&stdout).ok_or_else(|| {
                    CoreError::Platform(format!("Failed to parse issue URL from output: {stdout}"))
                })?;
                return self.view(issue_number).await;
            }

            return Err(parse_gh_error(&output.stderr).into());
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
            return Err(parse_gh_error(&output.stderr).into());
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
            return Err(parse_gh_error(&output.stderr).into());
        }

        let issue: IssueData =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(issue)
    }

    /// 关闭指定编号的 Issue。
    ///
    /// 通过单次 `gh api repos/{owner}/{repo}/issues/{number} -X PATCH -f state=closed`
    /// 调用更新状态，并直接解析 REST 响应获得更新后的完整 Issue 数据。
    /// 该调用幂等：对已关闭的 Issue 再次关闭将成功返回其当前数据。
    ///
    /// # 设计说明（issue #117）
    ///
    /// 早期实现先调用 `gh issue close`，再用 `gh issue view --json` 重新获取详情。
    /// 该双调用设计有两个缺陷：
    ///
    /// 1. `--json` 输出来自 `gh` 的 GraphQL 导出层，字段形状随 `gh` 版本漂移 （如 gh 2.94+ 的 bot
    ///    author 省略 `id`），导致反序列化失败；
    /// 2. 关闭操作已经成功时，二次获取的解析失败仍被误报为"关闭失败"。
    ///
    /// 改为单次 REST 调用后，变更与数据在同一响应中返回，两个缺陷同时消除。
    /// REST 响应形状是 GitHub API 的稳定契约，bot 用户同样携带数字 `id`。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在、`gh` CLI 调用失败或响应解析失败时返回错误。
    async fn close(&self, number: u64) -> Result<IssueData> {
        debug!(repo = %self.repo, number, "spawning `gh api issues PATCH state=closed`");

        let api_path = format!("repos/{repo}/issues/{number}", repo = self.repo);
        let output = self
            .runner
            .run(
                "gh",
                &["api", &api_path, "-X", "PATCH", "-f", "state=closed"],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh api issue close: {e}")))?;

        if !output.status.success() {
            return Err(parse_gh_error(&output.stderr).into());
        }

        let api_response: GitHubIssueApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_response.into())
    }

    /// 重新打开指定编号的 Issue。
    ///
    /// 通过单次 `gh api repos/{owner}/{repo}/issues/{number} -X PATCH -f state=open`
    /// 调用更新状态，并直接解析 REST 响应获得更新后的完整 Issue 数据。
    /// 该调用幂等：对未关闭的 Issue 再次打开将成功返回其当前数据。
    ///
    /// # 设计说明（issue #117）
    ///
    /// 与 [`close`](Self::close) 相同，改为单次 REST 调用以消除双调用的
    /// 反序列化漂移与误报失败问题。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在、`gh` CLI 调用失败或响应解析失败时返回错误。
    async fn reopen(&self, number: u64) -> Result<IssueData> {
        debug!(repo = %self.repo, number, "spawning `gh api issues PATCH state=open`");

        let api_path = format!("repos/{repo}/issues/{number}", repo = self.repo);
        let output = self
            .runner
            .run("gh", &["api", &api_path, "-X", "PATCH", "-f", "state=open"])
            .await
            .map_err(|e| {
                CoreError::Platform(format!("Failed to spawn gh api issue reopen: {e}"))
            })?;

        if !output.status.success() {
            return Err(parse_gh_error(&output.stderr).into());
        }

        let api_response: GitHubIssueApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_response.into())
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
            return Err(parse_gh_error(&output.stderr).into());
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
            return Err(parse_gh_error(&output.stderr).into());
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
            return Err(parse_gh_error(&retry_output.stderr).into());
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
            return Err(parse_gh_error(&output.stderr).into());
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

/// GitHub API 标签响应结构（REST 形状）。
///
/// 用于解析 `gh api` 返回的 Issue 对象中的标签。REST 标签的 `description`
/// 在无描述时为 `null`，`color` 为不带 `#` 前缀的十六进制字符串。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct GitHubLabelApiResponse {
    pub name: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

/// GitHub API Issue 响应结构（REST 形状）。
///
/// 用于解析 `gh api repos/{owner}/{repo}/issues/{number} -X PATCH` 的返回数据。
///
/// 与 `gh issue view --json` 的 GraphQL 导出格式不同，REST 形状是 GitHub API
/// 的稳定契约，不随 `gh` CLI 版本漂移：bot 用户同样携带数字 `id`，
/// 也不存在 GraphQL 导出层新增/省略字段的问题（issue #117）。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct GitHubIssueApiResponse {
    pub number: u64,
    pub title: String,
    #[serde(default)]
    pub body: Option<String>,
    pub state: State,
    #[serde(default)]
    pub labels: Vec<GitHubLabelApiResponse>,
    pub user: GitHubUser,
    #[serde(default)]
    pub assignees: Vec<GitHubUser>,
    pub created_at: String,
    pub updated_at: String,
    pub html_url: String,
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
            created_at: parse_api_datetime(&api.created_at),
        }
    }
}

impl From<GitHubLabelApiResponse> for Label {
    fn from(api: GitHubLabelApiResponse) -> Self {
        Self {
            name: api.name,
            color: api.color,
            description: api.description,
        }
    }
}

impl From<GitHubUser> for UserSummary {
    fn from(user: GitHubUser) -> Self {
        Self {
            login: user.login,
            id: user.id.to_string(),
        }
    }
}

impl From<GitHubIssueApiResponse> for IssueData {
    fn from(api: GitHubIssueApiResponse) -> Self {
        Self {
            number: api.number,
            title: api.title,
            body: api.body,
            state: api.state,
            labels: api.labels.into_iter().map(Label::from).collect(),
            author: api.user.into(),
            assignees: api.assignees.into_iter().map(UserSummary::from).collect(),
            created_at: parse_api_datetime(&api.created_at),
            updated_at: parse_api_datetime(&api.updated_at),
            url: api.html_url,
        }
    }
}

/// 解析 GitHub REST API 的 RFC 3339 时间戳。
///
/// 格式非法时记录警告并回退到 Unix 纪元，避免时间戳异常阻断主流程。
fn parse_api_datetime(value: &str) -> chrono::DateTime<chrono::Utc> {
    value.parse().unwrap_or_else(|_| {
        tracing::warn!(value, "Failed to parse GitHub API timestamp, using epoch");
        chrono::DateTime::UNIX_EPOCH
    })
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

    // --- GitHubIssueApiResponse conversion tests ---

    fn sample_rest_issue_response() -> GitHubIssueApiResponse {
        serde_json::from_str(REST_CLOSED_ISSUE_JSON).expect("fixture must deserialize")
    }

    #[test]
    fn test_should_convert_rest_issue_response_to_issue_data() {
        let issue: IssueData = sample_rest_issue_response().into();

        assert_eq!(issue.number, 107);
        assert_eq!(issue.title, "upstream CLI 新版本: glab 1.111.0");
        assert_eq!(issue.body, None);
        assert_eq!(issue.state, State::Closed);
        assert_eq!(issue.author.login, "github-actions[bot]");
        assert_eq!(issue.author.id, "41898282");
        assert_eq!(issue.labels.len(), 1);
        assert_eq!(issue.labels[0].name, "upstream-drift");
        assert_eq!(
            issue.labels[0].color.as_deref(),
            Some("ededed"),
            "REST label colors must keep the API-provided hex value"
        );
        assert_eq!(issue.labels[0].description, None);
        assert!(issue.assignees.is_empty());
        assert_eq!(issue.url, "https://github.com/o/r/issues/107");
    }

    #[test]
    fn test_should_fall_back_to_epoch_for_invalid_rest_issue_dates() {
        let mut api_response = sample_rest_issue_response();
        api_response.created_at = "invalid-date".to_string();
        api_response.updated_at = "also-invalid".to_string();

        let issue: IssueData = api_response.into();

        assert_eq!(issue.created_at, chrono::DateTime::UNIX_EPOCH);
        assert_eq!(issue.updated_at, chrono::DateTime::UNIX_EPOCH);
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
            gitflow_cli_core::CoreError::Cli(_)
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
            gitflow_cli_core::CoreError::Cli(_)
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
            gitflow_cli_core::CoreError::Cli(_)
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
            gitflow_cli_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_close() {
        // The single `gh api PATCH` call succeeds but its stdout is not JSON.
        let runner = MockCommandRunner::success("invalid");
        let provider = GitHubIssueProvider::with_runner("owner/repo", runner);

        let result = provider.close(42).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Serialization(_)
        ));
    }

    /// 模拟 `gh api repos/{owner}/{repo}/issues/{number} -X PATCH -f state=closed`
    /// 返回的 GitHub REST issue 对象（bot user、null body、含 null description 的 label）。
    const REST_CLOSED_ISSUE_JSON: &str = r#"{
        "id": 5123456789,
        "node_id": "I_kwDOXYZ",
        "number": 107,
        "title": "upstream CLI 新版本: glab 1.111.0",
        "body": null,
        "state": "closed",
        "labels": [
            {
                "id": 7301464236,
                "node_id": "LA_kwDOXYZ",
                "url": "https://api.github.com/repos/o/r/labels/upstream-drift",
                "name": "upstream-drift",
                "color": "ededed",
                "description": null,
                "default": false
            }
        ],
        "user": {"login": "github-actions[bot]", "id": 41898282, "type": "Bot"},
        "assignees": [],
        "created_at": "2026-07-31T02:00:00Z",
        "updated_at": "2026-08-03T09:31:29Z",
        "closed_at": "2026-08-03T09:31:29Z",
        "html_url": "https://github.com/o/r/issues/107"
    }"#;

    const REST_OPEN_ISSUE_JSON: &str = r#"{
        "id": 5123456790,
        "node_id": "I_kwDOXYZ2",
        "number": 108,
        "title": "upstream CLI 新版本: gitcode 0.8.0",
        "body": "patrol report",
        "state": "open",
        "labels": [],
        "user": {"login": "github-actions[bot]", "id": 41898282, "type": "Bot"},
        "assignees": [{"login": "octocat", "id": 583231}],
        "created_at": "2026-07-31T02:00:00Z",
        "updated_at": "2026-08-03T06:20:18Z",
        "closed_at": null,
        "html_url": "https://github.com/o/r/issues/108"
    }"#;

    // --- close/reopen: single-call PATCH redesign (issue #117) ---
    //
    // The old flow ran `gh issue close` and then re-fetched via `gh issue view`;
    // a parse failure on the re-fetch misreported an already-successful close as
    // a failure. The new flow performs ONE `gh api PATCH` call whose response is
    // the authoritative REST issue object — mutation and data in one round trip.

    #[tokio::test]
    async fn test_should_close_issue_via_single_api_patch_call() {
        // Exactly one response: if `close` makes a second call (the old
        // `gh issue view` re-fetch), the sequenced runner errors out.
        let runner = SequencedMockCommandRunner::from_results(&[(true, REST_CLOSED_ISSUE_JSON)]);
        let provider = GitHubIssueProvider::with_runner("o/r", runner);

        let issue = provider
            .close(107)
            .await
            .expect("close must succeed with a single PATCH call");

        assert_eq!(issue.number, 107);
        assert_eq!(issue.state, State::Closed);
        assert_eq!(issue.title, "upstream CLI 新版本: glab 1.111.0");
        assert_eq!(issue.body, None);
        assert_eq!(issue.author.login, "github-actions[bot]");
        assert_eq!(issue.author.id, "41898282");
        assert_eq!(issue.labels.len(), 1);
        assert_eq!(issue.labels[0].name, "upstream-drift");
        assert_eq!(issue.labels[0].description, None);
        assert_eq!(issue.url, "https://github.com/o/r/issues/107");
        assert_eq!(issue.updated_at.to_rfc3339(), "2026-08-03T09:31:29+00:00");
    }

    #[tokio::test]
    async fn test_should_reopen_issue_via_single_api_patch_call() {
        let runner = SequencedMockCommandRunner::from_results(&[(true, REST_OPEN_ISSUE_JSON)]);
        let provider = GitHubIssueProvider::with_runner("o/r", runner);

        let issue = provider
            .reopen(108)
            .await
            .expect("reopen must succeed with a single PATCH call");

        assert_eq!(issue.number, 108);
        assert_eq!(issue.state, State::Open);
        assert_eq!(issue.body.as_deref(), Some("patrol report"));
        assert_eq!(issue.assignees.len(), 1);
        assert_eq!(issue.assignees[0].login, "octocat");
        assert_eq!(issue.assignees[0].id, "583231");
        assert_eq!(issue.url, "https://github.com/o/r/issues/108");
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_fails_for_reopen() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not found"}"#, 256);
        let provider = GitHubIssueProvider::with_runner("owner/repo", runner);

        let result = provider.reopen(42).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_reopen() {
        // The single `gh api PATCH` call succeeds but its stdout is not JSON.
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
            gitflow_cli_core::CoreError::Cli(_)
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
            gitflow_cli_core::CoreError::Cli(_)
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
            gitflow_cli_core::CoreError::Cli(_)
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
        let err = result.unwrap_err();
        assert!(
            matches!(err, gitflow_cli_core::CoreError::Cli(_)),
            "expected CoreError::Cli, got: {err:?}"
        );
        if let gitflow_cli_core::CoreError::Cli(boxed) = err {
            assert!(
                boxed.raw_stderr.contains("403") || boxed.raw_stderr.contains("Forbidden"),
                "unexpected raw_stderr: {}",
                boxed.raw_stderr
            );
        }
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
        let err = result.unwrap_err();
        assert!(
            matches!(err, gitflow_cli_core::CoreError::Cli(_)),
            "expected CoreError::Cli, got: {err:?}"
        );
        if let gitflow_cli_core::CoreError::Cli(boxed) = err {
            assert!(
                boxed.raw_stderr.contains("500") || boxed.raw_stderr.contains("Internal"),
                "unexpected raw_stderr: {}",
                boxed.raw_stderr
            );
        }
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

#[cfg(test)]
mod contract_tests {
    use super::*;
    use crate::runner::MockCommandRunner;

    /// 契约测试：验证 gh v2.94 对 bot author 省略 `id` 字段时仍可反序列化。
    ///
    /// gh 2.94 对 bot-authored issue 返回 `{"is_bot": true, "login": "app/github-actions"}`
    /// 而非人类的 `{"id": "...", "login": "..."}`。`UserSummary::id` 使用 `#[serde(default)]`
    /// 兼容该差异,此处用真实 fixture 守护契约。
    #[tokio::test]
    async fn test_contract_issue_list_github_v294_bot_author() {
        let fixture = include_str!("../tests/fixtures/issue_list_github_v294_bot_author.json");
        let runner = MockCommandRunner::success(fixture);
        let provider = GitHubIssueProvider::with_runner("owner/repo", runner);

        let issues = provider
            .list(ListIssueArgs::default())
            .await
            .expect("gh v2.94 bot-author fixture must parse");

        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].number, 107);
        assert_eq!(issues[0].author.login, "app/github-actions");
        assert_eq!(issues[1].number, 42);
        assert_eq!(issues[1].author.login, "test-user");
    }

    /// 契约测试：验证 gh issue list JSON 输出与 `IssueData` 反序列化一致。
    ///
    /// 夹具来源：gh v2.x `--json` 输出格式。
    #[tokio::test]
    async fn test_contract_issue_list_github_v2() {
        let fixture = include_str!("../tests/fixtures/issue_list_github_v2.json");
        let runner = MockCommandRunner::success(fixture);
        let provider = GitHubIssueProvider::with_runner("owner/repo", runner);

        let issues = provider
            .list(ListIssueArgs::default())
            .await
            .expect("contract fixture must parse");

        assert_eq!(issues.len(), 1);
        let issue = &issues[0];
        assert_eq!(issue.number, 42);
        assert!(!issue.title.is_empty());
        assert_eq!(issue.state, gitflow_cli_core::types::State::Open);
        assert_eq!(issue.author.login, "test-user");
    }

    /// 契约测试：验证 gh v2.97 混合 author 形状（bot 无 `id` + human 新增 `name`）可反序列化。
    ///
    /// gh 2.97.0 对 bot author 返回 `{"is_bot": true, "login": "app/github-actions"}`
    /// （无 `id`），对 human author 返回
    /// `{"id": "...", "is_bot": false, "login": "...", "name": "..."}`（新增 `name` 字段）。
    /// `UserSummary::id` 的 `#[serde(default)]` 兜底 bot 缺 `id`，`name` 作为未知字段被忽略。
    /// 夹具为 2026-08-03 从 gh 2.97.0 真实捕获（issue #108/#117）并截断 body。
    /// 守护 issue #117：修复前该形状触发 `missing field 'id'` 反序列化错误。
    #[tokio::test]
    async fn test_contract_issue_list_github_v297_mixed_authors() {
        let fixture = include_str!("../tests/fixtures/issue_list_github_v297_mixed_authors.json");
        let runner = MockCommandRunner::success(fixture);
        let provider = GitHubIssueProvider::with_runner("owner/repo", runner);

        let issues = provider
            .list(ListIssueArgs::default())
            .await
            .expect("gh v2.97 mixed-author fixture must parse");

        assert_eq!(issues.len(), 2);
        // bot-authored issue: id omitted by gh → defaults to empty string
        assert_eq!(issues[0].number, 108);
        assert_eq!(issues[0].author.login, "app/github-actions");
        assert_eq!(issues[0].author.id, "");
        // human-authored issue: id present, extra `name` field ignored
        assert_eq!(issues[1].number, 117);
        assert_eq!(issues[1].author.login, "byx-darwin");
        assert!(!issues[1].author.id.is_empty());
    }

    /// 契约测试：验证 gh v2.97 `issue view` 对 bot-authored issue 的输出可反序列化。
    ///
    /// 夹具为 2026-08-03 从 gh 2.97.0 真实捕获（issue #108，author 为
    /// `app/github-actions`）并截断 body。守护 issue #117 的 `close`/`view`
    /// 路径：bot-authored issue 的 author 对象缺少 `id` 字段。
    #[tokio::test]
    async fn test_contract_issue_view_github_v297_bot_author() {
        let fixture = include_str!("../tests/fixtures/issue_view_github_v297_bot_author.json");
        let runner = MockCommandRunner::success(fixture);
        let provider = GitHubIssueProvider::with_runner("owner/repo", runner);

        let issue = provider
            .view(108)
            .await
            .expect("gh v2.97 bot-author view fixture must parse");

        assert_eq!(issue.number, 108);
        assert_eq!(issue.author.login, "app/github-actions");
        assert_eq!(issue.author.id, "");
        assert_eq!(issue.state, gitflow_cli_core::types::State::Open);
    }
}
