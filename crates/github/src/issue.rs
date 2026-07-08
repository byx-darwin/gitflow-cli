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

use crate::error::parse_gh_error;

/// `gh issue` 请求的 JSON 字段列表。
const ISSUE_FIELDS: &str =
    "number,title,body,state,labels,author,assignees,createdAt,updatedAt,url";

/// GitHub Issue 提供者，通过 `gh` CLI 操作。
///
/// 该结构体通过调用 `gh` CLI 实现 [`IssueProvider`] trait 的所有方法，
/// 使上层命令能够以统一的方式操作 GitHub Issue。
///
/// # Examples
///
/// ```no_run
/// use gitflow_cli_github::GitHubIssueProvider;
///
/// let provider = GitHubIssueProvider::new("octocat/hello-world");
/// ```
#[derive(Debug, Clone)]
pub struct GitHubIssueProvider {
    /// GitHub `owner/repo`，如 `"byx-darwin/gitflow-cli"`。
    repo: String,
}

impl GitHubIssueProvider {
    /// 创建新的 GitHub Issue 提供者。
    ///
    /// `repo` 格式为 `owner/repo`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> Self {
        Self { repo: repo.into() }
    }
}

#[async_trait]
impl IssueProvider for GitHubIssueProvider {
    async fn create(&self, args: CreateIssueArgs) -> Result<IssueData> {
        let mut cmd = tokio::process::Command::new("gh");
        cmd.args(["issue", "create"])
            .arg("--repo")
            .arg(&self.repo)
            .arg("--title")
            .arg(&args.title);

        if let Some(body) = &args.body {
            cmd.arg("--body").arg(body);
        }

        if !args.labels.is_empty() {
            cmd.arg("--label").arg(args.labels.join(","));
        }

        if !args.assignees.is_empty() {
            cmd.arg("--assignee").arg(args.assignees.join(","));
        }

        debug!(repo = %self.repo, title = %args.title, "spawning `gh issue create`");

        let output = cmd
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
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
        let mut cmd = tokio::process::Command::new("gh");
        cmd.args(["issue", "list"])
            .arg("--repo")
            .arg(&self.repo)
            .arg("--json")
            .arg(ISSUE_FIELDS);

        if let Some(state) = &args.state {
            cmd.arg("--state").arg(match state {
                State::Open => "open",
                State::Closed => "closed",
            });
        }

        if let Some(ref search) = args.search {
            cmd.arg("--search").arg(search);
        }

        if let Some(limit) = args.limit {
            cmd.arg("--limit").arg(limit.to_string());
        }

        debug!(repo = %self.repo, "spawning `gh issue list`");

        let output = cmd
            .output()
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

        let output = tokio::process::Command::new("gh")
            .args(["issue", "view"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .arg("--json")
            .arg(ISSUE_FIELDS)
            .output()
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
    /// 调用 `gh issue close <number> --repo <repo> --json <fields>` 关闭 Issue，
    /// 并返回更新后的完整 Issue 数据。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在、已关闭或 `gh` CLI 调用失败时返回错误。
    async fn close(&self, number: u64) -> Result<IssueData> {
        debug!(repo = %self.repo, number, "spawning `gh issue close`");

        let output = tokio::process::Command::new("gh")
            .args(["issue", "close"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .arg("--json")
            .arg(ISSUE_FIELDS)
            .output()
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

    /// 重新打开指定编号的 Issue。
    ///
    /// 调用 `gh issue reopen <number> --repo <repo> --json <fields>` 重新打开已关闭的 Issue，
    /// 并返回更新后的完整 Issue 数据。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在、未关闭或 `gh` CLI 调用失败时返回错误。
    async fn reopen(&self, number: u64) -> Result<IssueData> {
        debug!(repo = %self.repo, number, "spawning `gh issue reopen`");

        let output = tokio::process::Command::new("gh")
            .args(["issue", "reopen"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .arg("--json")
            .arg(ISSUE_FIELDS)
            .output()
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
        let output = tokio::process::Command::new("gh")
            .args(["issue", "comment"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .arg("--body")
            .arg(body)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        // 2. 使用 gh api 获取该 issue 的最新评论
        let api_path = format!("repos/{repo}/issues/{number}/comments?per_page=1", repo = self.repo, number = number);
        let api_output = tokio::process::Command::new("gh")
            .args(["api", &api_path])
            .output()
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

        let comment = comments.into_iter().next().ok_or_else(|| {
            CoreError::Platform("No comment returned from gh api".to_string())
        })?;

        Ok(comment.into())
    }

    /// 为指定 Issue 添加一个或多个标签。
    ///
    /// 调用 `gh issue edit <number> --repo <repo> --add-label <label>` 逐个添加标签。
    /// 如果 `labels` 为空，不进行任何调用并返回成功。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在、标签名无效或 `gh` CLI 调用失败时返回错误。
    async fn add_labels(&self, number: u64, labels: &[String]) -> Result<()> {
        debug!(
            repo = %self.repo,
            number,
            label_count = labels.len(),
            "spawning `gh issue edit --add-label`"
        );

        let mut cmd = tokio::process::Command::new("gh");
        cmd.args(["issue", "edit"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo);

        for label in labels {
            cmd.arg("--add-label").arg(label);
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
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

        let output = tokio::process::Command::new("gh")
            .args(["issue", "edit"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .arg("--remove-label")
            .arg(label)
            .output()
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

#[cfg(test)]
mod tests {
    use gitflow_cli_core::types::UserSummary;

    use super::*;

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
}
