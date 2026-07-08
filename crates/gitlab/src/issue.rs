//! GitLab Issue 提供者实现。
//!
//! 通过 `glab` CLI 实现 [`IssueProvider`] trait，支持 Issue 的创建、列表、查看、
//! 关闭、重新打开、评论及标签管理。
//! 所有方法通过 `tokio::process::Command` 调用 `glab`，捕获 stdout 并解析 JSON。
//!
//! `glab` 的 `JSON` 输出使用 `snake_case` 字段名和 `GitLab` 特有的字段名（如 `iid`、
//! `description`、`web_url`），因此本模块使用中间类型 [`IssueApiResponse`] 进行
//! 反序列化，然后通过 `From` 实现转换为核心类型 [`IssueData`]。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use gitflow_cli_core::{
    CoreError, Result,
    issue::{CreateIssueArgs, IssueData, IssueProvider, ListIssueArgs},
    types::{CommentData, Label, State, UserSummary},
};
use serde::Deserialize;
use tracing::debug;

use crate::error::parse_glab_error;

/// GitLab Issue 提供者，通过 `glab` CLI 操作。
///
/// # Examples
///
/// ```no_run
/// use gitflow_cli_gitlab::GitLabIssueProvider;
///
/// let provider = GitLabIssueProvider::new("gitlab-org/gitlab");
/// ```
#[derive(Debug, Clone)]
pub struct GitLabIssueProvider {
    /// GitLab `namespace/project`，如 `"gitlab-org/gitlab"`。
    repo: String,
}

impl GitLabIssueProvider {
    /// 创建新的 GitLab Issue 提供者。
    ///
    /// `repo` 格式为 `namespace/project`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> Self {
        Self { repo: repo.into() }
    }
}

// ── 中间 API 响应类型 ──────────────────────────────────────────────

/// `glab issue` JSON 输出中的用户信息。
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

/// `glab issue --output json` 返回的 JSON 结构。
#[derive(Debug, Clone, Deserialize)]
struct IssueApiResponse {
    iid: u64,
    title: String,
    #[serde(default)]
    description: Option<String>,
    state: String,
    #[serde(default)]
    labels: Vec<String>,
    #[serde(default)]
    author: Option<ApiUser>,
    #[serde(default)]
    assignees: Vec<ApiUser>,
    #[serde(default)]
    created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    updated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    web_url: Option<String>,
}

impl From<IssueApiResponse> for IssueData {
    fn from(api: IssueApiResponse) -> Self {
        let now = Utc::now();
        let labels: Vec<Label> = api
            .labels
            .into_iter()
            .map(|name| Label {
                name,
                color: None,
                description: None,
            })
            .collect();
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
            state: if api.state == "closed" {
                State::Closed
            } else {
                State::Open
            },
            labels,
            author,
            assignees: api.assignees.iter().map(UserSummary::from).collect(),
            created_at: api.created_at.unwrap_or(now),
            updated_at: api.updated_at.unwrap_or(now),
            url: api.web_url.unwrap_or_default(),
        }
    }
}

/// `glab issue comment --output json` 返回的 JSON 结构。
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
impl IssueProvider for GitLabIssueProvider {
    async fn create(&self, args: CreateIssueArgs) -> Result<IssueData> {
        let mut cmd = tokio::process::Command::new("glab");
        cmd.args(["issue", "create"])
            .arg("--repo")
            .arg(&self.repo)
            .arg("--title")
            .arg(&args.title);

        if let Some(body) = &args.body {
            cmd.arg("--description").arg(body);
        }

        if !args.labels.is_empty() {
            cmd.arg("--label").arg(args.labels.join(","));
        }

        if !args.assignees.is_empty() {
            cmd.arg("--assignee").arg(args.assignees.join(","));
        }

        debug!(repo = %self.repo, title = %args.title, "spawning `glab issue create`");

        let output = cmd
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        // Parse the issue URL from stdout (format: https://gitlab.com/.../-/issues/123)
        let stdout = String::from_utf8_lossy(&output.stdout);
        let issue_iid = parse_issue_iid_from_url(&stdout).ok_or_else(|| {
            CoreError::Platform(format!("Failed to parse issue URL from output: {stdout}"))
        })?;

        // Fetch full issue details via view
        self.view(issue_iid).await
    }

    async fn list(&self, args: ListIssueArgs) -> Result<Vec<IssueData>> {
        let mut cmd = tokio::process::Command::new("glab");
        cmd.args(["issue", "list"])
            .arg("--repo")
            .arg(&self.repo)
            .arg("--output")
            .arg("json");

        // glab uses --closed for closed issues, --all for all issues
        // Default (no flag) shows open issues
        if let Some(state) = &args.state {
            match state {
                State::Open => {
                    // Default behavior, no flag needed
                }
                State::Closed => {
                    cmd.arg("--closed");
                }
            }
        }

        if let Some(ref search) = args.search {
            cmd.arg("--search").arg(search);
        }

        if let Some(limit) = args.limit {
            cmd.arg("--per-page").arg(limit.to_string());
        }

        debug!(repo = %self.repo, "spawning `glab issue list`");

        let output = cmd
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        let api_responses: Vec<IssueApiResponse> =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_responses.into_iter().map(IssueData::from).collect())
    }

    async fn view(&self, number: u64) -> Result<IssueData> {
        debug!(repo = %self.repo, number, "spawning `glab issue view`");

        let output = tokio::process::Command::new("glab")
            .args(["issue", "view"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .arg("--output")
            .arg("json")
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        let api_response: IssueApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_response.into())
    }

    async fn close(&self, number: u64) -> Result<IssueData> {
        debug!(repo = %self.repo, number, "spawning `glab issue close`");

        let output = tokio::process::Command::new("glab")
            .args(["issue", "close"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .arg("--output")
            .arg("json")
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        let api_response: IssueApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_response.into())
    }

    async fn reopen(&self, number: u64) -> Result<IssueData> {
        debug!(repo = %self.repo, number, "spawning `glab issue reopen`");

        let output = tokio::process::Command::new("glab")
            .args(["issue", "reopen"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .arg("--output")
            .arg("json")
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        let api_response: IssueApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_response.into())
    }

    async fn comment(&self, number: u64, body: &str) -> Result<CommentData> {
        debug!(repo = %self.repo, number, "spawning `glab issue comment`");

        let output = tokio::process::Command::new("glab")
            .args(["issue", "note"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .arg("--body")
            .arg(body)
            .arg("--output")
            .arg("json")
            .output()
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

    async fn add_labels(&self, number: u64, labels: &[String]) -> Result<()> {
        debug!(
            repo = %self.repo,
            number,
            label_count = labels.len(),
            "spawning `glab issue edit --add-label`"
        );

        let output = tokio::process::Command::new("glab")
            .args(["issue", "edit"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .arg("--add-label")
            .arg(labels.join(","))
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        Ok(())
    }

    async fn remove_label(&self, number: u64, label: &str) -> Result<()> {
        debug!(repo = %self.repo, number, label, "spawning `glab issue edit --remove-label`");

        let output = tokio::process::Command::new("glab")
            .args(["issue", "edit"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .arg("--remove-label")
            .arg(label)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        Ok(())
    }
}

/// Parse issue IID from GitLab URL.
///
/// Extracts the numeric IID from URLs like:
/// - `https://gitlab.com/owner/repo/-/issues/123`
/// - `https://gitlab.example.com/group/project/-/issues/456`
fn parse_issue_iid_from_url(url: &str) -> Option<u64> {
    url.lines().find_map(|line| {
        let line = line.trim();
        if line.contains("/-/issues/") {
            line.rsplit("/-/issues/")
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

    #[test]
    fn test_should_construct_gitlab_issue_provider() {
        let provider = GitLabIssueProvider::new("gitlab-org/gitlab");
        assert_eq!(provider.repo, "gitlab-org/gitlab");
    }

    #[test]
    fn test_should_construct_gitlab_issue_provider_from_string() {
        let repo = String::from("gitlab-org/gitlab");
        let provider = GitLabIssueProvider::new(repo);
        assert_eq!(provider.repo, "gitlab-org/gitlab");
    }

    #[test]
    fn test_should_deserialize_issue_api_response() {
        let json = br#"{
            "iid": 42,
            "title": "Fix login bug",
            "description": "Reproduced on v1.2.3",
            "state": "opened",
            "labels": ["bug", "critical"],
            "author": {"username": "admin", "id": 1},
            "assignees": [{"username": "alice", "id": 7}],
            "created_at": "2026-01-15T09:30:00Z",
            "updated_at": "2026-01-16T11:00:00Z",
            "web_url": "https://gitlab.com/gitlab-org/gitlab/-/issues/42"
        }"#;

        let api: IssueApiResponse = serde_json::from_slice(json).expect("valid IssueApiResponse");
        let issue: IssueData = api.into();

        assert_eq!(issue.number, 42);
        assert_eq!(issue.title, "Fix login bug");
        assert_eq!(issue.body.as_deref(), Some("Reproduced on v1.2.3"));
        assert_eq!(issue.state, State::Open);
        assert_eq!(issue.labels.len(), 2);
        assert_eq!(issue.labels[0].name, "bug");
        assert_eq!(issue.author.login, "admin");
        assert_eq!(issue.assignees.len(), 1);
        assert_eq!(
            issue.url,
            "https://gitlab.com/gitlab-org/gitlab/-/issues/42"
        );
    }

    #[test]
    fn test_should_deserialize_closed_issue_api_response() {
        let json = br#"{
            "iid": 10,
            "title": "Fixed typo",
            "description": null,
            "state": "closed",
            "labels": [],
            "author": {"username": "dev", "id": 5},
            "assignees": [],
            "created_at": "2026-06-01T08:00:00Z",
            "updated_at": "2026-06-02T12:00:00Z",
            "web_url": "https://gitlab.com/org/project/-/issues/10"
        }"#;

        let api: IssueApiResponse = serde_json::from_slice(json).expect("valid IssueApiResponse");
        let issue: IssueData = api.into();
        assert_eq!(issue.state, State::Closed);
    }

    #[test]
    fn test_should_deserialize_comment_api_response() {
        let json = br#"{
            "id": 1001,
            "body": "Thanks for reporting.",
            "author": {"username": "maintainer", "id": 42},
            "created_at": "2026-06-15T14:00:00Z"
        }"#;

        let api: CommentApiResponse =
            serde_json::from_slice(json).expect("valid CommentApiResponse");
        let comment: CommentData = api.into();
        assert_eq!(comment.id, 1001);
        assert_eq!(comment.body, "Thanks for reporting.");
        assert_eq!(comment.author.login, "maintainer");
    }

    #[test]
    fn test_should_handle_missing_author_with_fallback() {
        let json = br#"{
            "iid": 1,
            "title": "No author",
            "description": null,
            "state": "opened",
            "labels": [],
            "author": null,
            "assignees": [],
            "created_at": null,
            "updated_at": null,
            "web_url": null
        }"#;

        let api: IssueApiResponse = serde_json::from_slice(json).expect("valid IssueApiResponse");
        let issue: IssueData = api.into();
        assert_eq!(issue.author.login, "unknown");
        assert_eq!(issue.author.id, "0");
    }

    #[test]
    fn test_should_convert_labels_from_strings() {
        let json = br#"{
            "iid": 1,
            "title": "Test",
            "description": null,
            "state": "opened",
            "labels": ["bug", "enhancement", "documentation"],
            "author": {"username": "admin", "id": 1},
            "assignees": [],
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z",
            "web_url": "https://gitlab.com/x/y/-/issues/1"
        }"#;

        let api: IssueApiResponse = serde_json::from_slice(json).expect("valid IssueApiResponse");
        let issue: IssueData = api.into();
        assert_eq!(issue.labels.len(), 3);
        assert_eq!(issue.labels[0].name, "bug");
        assert_eq!(issue.labels[1].name, "enhancement");
        assert!(issue.labels[0].color.is_none());
    }

    #[test]
    fn test_should_deserialize_empty_issue_list() {
        let json = b"[]";
        let list: Vec<IssueApiResponse> = serde_json::from_slice(json).expect("valid empty list");
        assert!(list.is_empty());
    }

    #[test]
    fn test_should_debug_format_provider() {
        let provider = GitLabIssueProvider::new("gitlab-org/gitlab");
        let debug = format!("{provider:?}");
        assert!(debug.contains("GitLabIssueProvider"));
        assert!(debug.contains("gitlab-org/gitlab"));
    }

    #[test]
    fn test_should_clone_gitlab_issue_provider() {
        let original = GitLabIssueProvider::new("owner/repo");
        let cloned = original.clone();
        assert_eq!(original.repo, cloned.repo);
    }

    #[test]
    fn test_should_parse_issue_iid_from_gitlab_url() {
        assert_eq!(
            parse_issue_iid_from_url("https://gitlab.com/owner/repo/-/issues/123"),
            Some(123)
        );
    }

    #[test]
    fn test_should_parse_issue_iid_from_self_hosted_url() {
        assert_eq!(
            parse_issue_iid_from_url("https://gitlab.example.com/group/project/-/issues/456"),
            Some(456)
        );
    }

    #[test]
    fn test_should_parse_issue_iid_from_multiline_output() {
        let output = "Creating issue...\nhttps://gitlab.com/owner/repo/-/issues/789\nDone.";
        assert_eq!(parse_issue_iid_from_url(output), Some(789));
    }

    #[test]
    fn test_should_return_none_for_invalid_url() {
        assert_eq!(parse_issue_iid_from_url("not a url"), None);
    }

    #[test]
    fn test_should_return_none_for_url_without_iid() {
        assert_eq!(
            parse_issue_iid_from_url("https://gitlab.com/owner/repo/-/issues/"),
            None
        );
    }
}
