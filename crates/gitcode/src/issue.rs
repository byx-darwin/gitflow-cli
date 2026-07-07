//! GitCode Issue 提供者实现。
//!
//! 通过 `gitcode` CLI 实现 [`IssueProvider`] trait。GitCode CLI
//! 使用 `-R` 指定仓库、`--json` 为布尔标志、`version` 子命令检测版本。
//! JSON 响应字段名与 GitHub/GitLab CLI 不同（`user` 而非 `author` 等），
//! 通过 [`IssueApiResponse`] 做字段映射后转换为 core 类型。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use gitflow_cli_core::{
    CoreError, Result,
    issue::{CreateIssueArgs, IssueData, IssueProvider, ListIssueArgs},
    types::{CommentData, Label, State, UserSummary},
};
use serde::Deserialize;
use tracing::debug;

use crate::error::parse_gitcode_error;

/// gitcode CLI `issue list --json` 的响应类型。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
struct IssueApiResponse {
    number: String,
    title: String,
    body: Option<String>,
    state: String,
    #[serde(default)]
    labels: Vec<LabelApi>,
    user: Option<UserApi>,
    #[serde(default)]
    assignees: Vec<UserApi>,
    created_at: Option<String>,
    updated_at: Option<String>,
    html_url: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
struct LabelApi {
    name: String,
    #[serde(default)]
    color: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct UserApi {
    login: String,
    #[serde(default)]
    id: Option<String>,
}

impl From<IssueApiResponse> for IssueData {
    fn from(api: IssueApiResponse) -> Self {
        Self {
            number: api.number.parse().unwrap_or(0),
            title: api.title,
            body: api.body,
            state: match api.state.as_str() {
                "closed" => State::Closed,
                _ => State::Open,
            },
            labels: api.labels.into_iter().map(Label::from).collect(),
            author: api.user.map_or(
                UserSummary {
                    login: "unknown".into(),
                    id: String::new(),
                },
                UserSummary::from,
            ),
            assignees: api.assignees.into_iter().map(UserSummary::from).collect(),
            created_at: api
                .created_at
                .and_then(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .ok()
                        .map(|d| d.with_timezone(&Utc))
                })
                .unwrap_or_else(Utc::now),
            updated_at: api
                .updated_at
                .and_then(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .ok()
                        .map(|d| d.with_timezone(&Utc))
                })
                .unwrap_or_else(Utc::now),
            url: api.html_url,
        }
    }
}

impl From<LabelApi> for Label {
    fn from(api: LabelApi) -> Self {
        Self {
            name: api.name,
            color: api.color,
            description: api.description,
        }
    }
}

impl From<UserApi> for UserSummary {
    fn from(api: UserApi) -> Self {
        Self {
            login: api.login,
            id: api.id.unwrap_or_default(),
        }
    }
}

/// gitcode CLI `issue comment --json` 的响应类型。
///
/// GitCode API 返回格式与 GitHub/GitLab 不同：
/// - `id` 为 JSON 字符串（如 `"178838115"`）
/// - `author` 为纯字符串（用户名），不是对象
/// - `created_at` 格式为 `"2026-07-07 10:40:20"`，不是 RFC3339
#[derive(Debug, Clone, Deserialize)]
struct CommentApiResponse {
    id: String,
    body: String,
    author: String,
    created_at: String,
}

impl From<CommentApiResponse> for CommentData {
    fn from(api: CommentApiResponse) -> Self {
        Self {
            id: api.id.parse().unwrap_or(0),
            body: api.body,
            author: UserSummary {
                login: api.author,
                id: String::new(),
            },
            created_at: chrono::NaiveDateTime::parse_from_str(&api.created_at, "%Y-%m-%d %H:%M:%S")
                .map_or_else(|_| Utc::now(), |ndt| ndt.and_utc()),
        }
    }
}

/// gitcode CLI `issue close/reopen --json` 的响应类型。
///
/// GitCode close/reopen 返回的字段比 list/view 少很多，
/// `number` 为 integer，没有 `title`/`body`/`labels` 等字段。
#[derive(Debug, Clone, Deserialize)]
struct CloseApiResponse {
    number: u64,
    state: String,
    url: String,
}

impl From<CloseApiResponse> for IssueData {
    fn from(api: CloseApiResponse) -> Self {
        Self {
            number: api.number,
            title: String::new(),
            body: None,
            state: match api.state.as_str() {
                "closed" => State::Closed,
                _ => State::Open,
            },
            labels: Vec::new(),
            author: UserSummary {
                login: "unknown".into(),
                id: String::new(),
            },
            assignees: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            url: api.url,
        }
    }
}

/// GitCode Issue 提供者，通过 `gitcode`/`gitcode` CLI 操作。
#[derive(Debug, Clone)]
pub struct GitCodeIssueProvider {
    /// GitCode `owner/repo`。
    repo: String,
}

impl GitCodeIssueProvider {
    /// 创建一个新的 `GitCodeIssueProvider`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> Self {
        Self { repo: repo.into() }
    }
}

#[async_trait]
impl IssueProvider for GitCodeIssueProvider {
    async fn create(&self, args: CreateIssueArgs) -> Result<IssueData> {
        let mut cmd = tokio::process::Command::new(crate::gitcode_binary());
        cmd.args(["issue", "create"])
            .arg("-R")
            .arg(&self.repo)
            .arg("--title")
            .arg(&args.title)
            .arg("--json");

        if let Some(body) = &args.body {
            cmd.arg("--body").arg(body);
        }
        for label in &args.labels {
            cmd.arg("--label").arg(label);
        }
        for assignee in &args.assignees {
            cmd.arg("--assignee").arg(assignee);
        }

        debug!(repo = %self.repo, title = %args.title, "spawning gitcode issue create");
        let output = cmd
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("{e}")))?;
        if !output.status.success() {
            return Err(CoreError::Platform(
                parse_gitcode_error(&output.stderr).to_string(),
            ));
        }
        serde_json::from_slice::<IssueApiResponse>(&output.stdout)
            .map(|api: IssueApiResponse| IssueData::from(api))
            .map_err(CoreError::Serialization)
    }

    async fn list(&self, args: ListIssueArgs) -> Result<Vec<IssueData>> {
        let mut cmd = tokio::process::Command::new(crate::gitcode_binary());
        cmd.args(["issue", "list"])
            .arg("-R")
            .arg(&self.repo)
            .arg("--json");

        if let Some(ref state) = args.state {
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
        for label in &args.labels {
            cmd.arg("--label").arg(label);
        }

        debug!(repo = %self.repo, "spawning gitcode issue list");
        let output = cmd
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("{e}")))?;
        if !output.status.success() {
            return Err(CoreError::Platform(
                parse_gitcode_error(&output.stderr).to_string(),
            ));
        }
        let issues: Vec<IssueApiResponse> =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;
        Ok(issues.into_iter().map(IssueData::from).collect())
    }

    async fn view(&self, number: u64) -> Result<IssueData> {
        debug!(repo = %self.repo, number, "spawning gitcode issue view");
        let output = tokio::process::Command::new(crate::gitcode_binary())
            .args(["issue", "view"])
            .arg(number.to_string())
            .arg("-R")
            .arg(&self.repo)
            .arg("--json")
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("{e}")))?;

        if !output.status.success() {
            return Err(CoreError::Platform(
                parse_gitcode_error(&output.stderr).to_string(),
            ));
        }
        serde_json::from_slice::<IssueApiResponse>(&output.stdout)
            .map(|api: IssueApiResponse| IssueData::from(api))
            .map_err(CoreError::Serialization)
    }

    async fn close(&self, number: u64) -> Result<IssueData> {
        debug!(repo = %self.repo, number, "spawning gitcode issue close");
        let output = tokio::process::Command::new(crate::gitcode_binary())
            .args(["issue", "close"])
            .arg(number.to_string())
            .arg("-R")
            .arg(&self.repo)
            .arg("--yes")
            .arg("--json")
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("{e}")))?;

        if !output.status.success() {
            return Err(CoreError::Platform(
                parse_gitcode_error(&output.stderr).to_string(),
            ));
        }
        serde_json::from_slice::<CloseApiResponse>(&output.stdout)
            .map(|api: CloseApiResponse| IssueData::from(api))
            .map_err(CoreError::Serialization)
    }

    async fn reopen(&self, number: u64) -> Result<IssueData> {
        debug!(repo = %self.repo, number, "spawning gitcode issue reopen");
        let output = tokio::process::Command::new(crate::gitcode_binary())
            .args(["issue", "reopen"])
            .arg(number.to_string())
            .arg("-R")
            .arg(&self.repo)
            .arg("--yes")
            .arg("--json")
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("{e}")))?;

        if !output.status.success() {
            return Err(CoreError::Platform(
                parse_gitcode_error(&output.stderr).to_string(),
            ));
        }
        serde_json::from_slice::<CloseApiResponse>(&output.stdout)
            .map(|api: CloseApiResponse| IssueData::from(api))
            .map_err(CoreError::Serialization)
    }

    /// 在指定 Issue 上添加评论。
    ///
    /// 调用 `gc issue comment <number> --repo <repo> --body "<body>" --json
    /// id,body,author,createdAt` 发布评论，并返回新建评论的数据。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在、`body` 为空或 `gitcode` CLI 调用失败时返回错误。
    async fn comment(&self, number: u64, body: &str) -> Result<CommentData> {
        debug!(repo = %self.repo, number, "spawning `gc issue comment`");

        let output = tokio::process::Command::new(crate::gitcode_binary())
            .args(["issue", "comment"])
            .arg(number.to_string())
            .arg("-R")
            .arg(&self.repo)
            .arg("--body")
            .arg(body)
            .arg("--json")
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
        }

        let api: CommentApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(CommentData::from(api))
    }

    /// 为指定 Issue 添加一个或多个标签。
    ///
    /// 调用 `gc issue edit <number> --repo <repo> --add-label <label>` 逐个添加标签。
    /// 如果 `labels` 为空，不进行任何调用并返回成功。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在、标签名无效或 `gitcode` CLI 调用失败时返回错误。
    async fn add_labels(&self, number: u64, labels: &[String]) -> Result<()> {
        debug!(
            repo = %self.repo,
            number,
            label_count = labels.len(),
            "spawning `gc issue edit --add-label`"
        );

        let mut cmd = tokio::process::Command::new(crate::gitcode_binary());
        cmd.args(["issue", "edit"])
            .arg(number.to_string())
            .arg("-R")
            .arg(&self.repo);

        for label in labels {
            cmd.arg("--add-label").arg(label);
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
        }

        Ok(())
    }

    /// 从指定 Issue 移除一个标签。
    ///
    /// 调用 `gc issue edit <number> --repo <repo> --remove-label <label>` 移除标签。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在、标签未附加到该 Issue 或 `gitcode` CLI 调用失败时返回错误。
    async fn remove_label(&self, number: u64, label: &str) -> Result<()> {
        debug!(repo = %self.repo, number, label, "spawning `gc issue edit --remove-label`");

        let output = tokio::process::Command::new(crate::gitcode_binary())
            .args(["issue", "edit"])
            .arg(number.to_string())
            .arg("-R")
            .arg(&self.repo)
            .arg("--remove-label")
            .arg(label)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use gitflow_cli_core::types::UserSummary;

    use super::*;

    #[test]
    fn test_should_construct_gitcode_issue_provider() {
        let provider = GitCodeIssueProvider::new("octocat/hello-world");
        assert_eq!(provider.repo, "octocat/hello-world");
    }

    #[test]
    fn test_should_construct_gitcode_issue_provider_from_string() {
        let repo = String::from("octocat/hello-world");
        let provider = GitCodeIssueProvider::new(repo);
        assert_eq!(provider.repo, "octocat/hello-world");
    }

    #[test]
    fn test_should_deserialize_issue_data_from_gc_output() {
        let gc_json = br#"{
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
            "url": "https://gitcode.com/octocat/hello-world/issues/42"
        }"#;

        let issue: IssueData = serde_json::from_slice(gc_json).expect("valid IssueData JSON");
        assert_eq!(issue.number, 42);
        assert_eq!(issue.title, "Fix login bug");
        assert_eq!(issue.state, State::Open);
        assert_eq!(issue.labels.len(), 1);
        assert_eq!(issue.author.login, "octocat");
        assert_eq!(issue.assignees.len(), 1);
        assert_eq!(
            issue.url,
            "https://gitcode.com/octocat/hello-world/issues/42"
        );
    }

    #[test]
    fn test_should_deserialize_empty_issue_list_from_gc_output() {
        let gc_json = b"[]";
        let issues: Vec<IssueData> = serde_json::from_slice(gc_json).expect("valid IssueData list");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_should_debug_format_provider() {
        let provider = GitCodeIssueProvider::new("octocat/hello-world");
        let debug = format!("{provider:?}");
        assert!(debug.contains("GitCodeIssueProvider"));
        assert!(debug.contains("octocat/hello-world"));
    }

    #[test]
    fn test_should_deserialize_closed_issue_from_gc_close_output() {
        let gc_json = br#"{
            "number": 10,
            "title": "Fixed typo",
            "body": null,
            "state": "closed",
            "labels": [],
            "author": {"login": "dev", "id": "5"},
            "assignees": [],
            "createdAt": "2026-06-01T08:00:00Z",
            "updatedAt": "2026-06-02T12:00:00Z",
            "url": "https://gitcode.com/octocat/hello-world/issues/10"
        }"#;

        let issue: IssueData = serde_json::from_slice(gc_json).expect("valid closed IssueData");
        assert_eq!(issue.number, 10);
        assert_eq!(issue.state, State::Closed);
    }

    #[test]
    fn test_should_deserialize_reopened_issue_from_gc_reopen_output() {
        let gc_json = br#"{
            "number": 10,
            "title": "Fixed typo",
            "body": null,
            "state": "open",
            "labels": [],
            "author": {"login": "dev", "id": "5"},
            "assignees": [],
            "createdAt": "2026-06-01T08:00:00Z",
            "updatedAt": "2026-06-03T09:00:00Z",
            "url": "https://gitcode.com/octocat/hello-world/issues/10"
        }"#;

        let issue: IssueData = serde_json::from_slice(gc_json).expect("valid reopened IssueData");
        assert_eq!(issue.number, 10);
        assert_eq!(issue.state, State::Open);
    }

    #[test]
    fn test_should_deserialize_comment_data_from_gc_comment_output() {
        let gc_json = br#"{
            "id": "1001",
            "body": "Thanks for reporting, looking into it.",
            "author": "maintainer",
            "created_at": "2026-06-15 14:00:00"
        }"#;

        let api: CommentApiResponse =
            serde_json::from_slice(gc_json).expect("valid CommentApiResponse");
        let comment = CommentData::from(api);
        assert_eq!(comment.id, 1001);
        assert_eq!(comment.body, "Thanks for reporting, looking into it.");
        assert_eq!(comment.author.login, "maintainer");
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

    #[test]
    fn test_should_create_provider_with_different_repos() {
        let r1 = GitCodeIssueProvider::new("org/repo-a");
        let r2 = GitCodeIssueProvider::new("org/repo-b");
        assert_eq!(r1.repo, "org/repo-a");
        assert_eq!(r2.repo, "org/repo-b");
    }

    #[test]
    fn test_should_clone_gitcode_issue_provider() {
        let original = GitCodeIssueProvider::new("owner/repo");
        let cloned = original.clone();
        assert_eq!(original.repo, cloned.repo);
    }
}
