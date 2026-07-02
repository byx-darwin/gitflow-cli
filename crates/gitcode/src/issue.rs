//! GitCode Issue 提供者实现。
//!
//! 通过 `gc` CLI 实现 [`IssueProvider`] trait，支持 Issue 的创建、列表、查看、
//! 关闭、重新打开、评论及标签管理。
//! 所有方法通过 `tokio::process::Command` 调用 `gc`，捕获 stdout 并解析 JSON。

use async_trait::async_trait;
use gitflow_cli_core::{
    CoreError, Result,
    issue::{CreateIssueArgs, IssueData, IssueProvider, ListIssueArgs},
    types::{CommentData, State},
};
use tracing::debug;

use crate::error::parse_gc_error;

/// `gc issue` 请求的 JSON 字段列表。
const ISSUE_FIELDS: &str =
    "number,title,body,state,labels,author,assignees,createdAt,updatedAt,url";

/// GitCode Issue 提供者，通过 `gc` CLI 操作。
///
/// 该结构体通过调用 `gc` CLI 实现 [`IssueProvider`] trait 的所有方法，
/// 使上层命令能够以统一的方式操作 GitCode Issue。
///
/// # Examples
///
/// ```no_run
/// use gitflow_cli_gitcode::GitCodeIssueProvider;
///
/// let provider = GitCodeIssueProvider::new("octocat/hello-world");
/// ```
#[derive(Debug, Clone)]
pub struct GitCodeIssueProvider {
    /// GitCode `owner/repo`，如 `"byx-darwin/gitflow-cli"`。
    repo: String,
}

impl GitCodeIssueProvider {
    /// 创建新的 GitCode Issue 提供者。
    ///
    /// `repo` 格式为 `owner/repo`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> Self {
        Self { repo: repo.into() }
    }
}

#[async_trait]
impl IssueProvider for GitCodeIssueProvider {
    async fn create(&self, args: CreateIssueArgs) -> Result<IssueData> {
        let mut cmd = tokio::process::Command::new("gc");
        cmd.args(["issue", "create"])
            .arg("--repo")
            .arg(&self.repo)
            .arg("--title")
            .arg(&args.title)
            .arg("--json")
            .arg(ISSUE_FIELDS);

        if let Some(body) = &args.body {
            cmd.arg("--body").arg(body);
        }

        if !args.labels.is_empty() {
            cmd.arg("--label").arg(args.labels.join(","));
        }

        if !args.assignees.is_empty() {
            cmd.arg("--assignee").arg(args.assignees.join(","));
        }

        debug!(repo = %self.repo, title = %args.title, "spawning `gc issue create`");

        let output = cmd
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gc: {e}")))?;

        if !output.status.success() {
            let gc_err = parse_gc_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gc_err}")));
        }

        let issue: IssueData =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(issue)
    }

    async fn list(&self, args: ListIssueArgs) -> Result<Vec<IssueData>> {
        let mut cmd = tokio::process::Command::new("gc");
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

        debug!(repo = %self.repo, "spawning `gc issue list`");

        let output = cmd
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gc: {e}")))?;

        if !output.status.success() {
            let gc_err = parse_gc_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gc_err}")));
        }

        let issues: Vec<IssueData> =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(issues)
    }

    async fn view(&self, number: u64) -> Result<IssueData> {
        debug!(repo = %self.repo, number, "spawning `gc issue view`");

        let output = tokio::process::Command::new("gc")
            .args(["issue", "view"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .arg("--json")
            .arg(ISSUE_FIELDS)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gc: {e}")))?;

        if !output.status.success() {
            let gc_err = parse_gc_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gc_err}")));
        }

        let issue: IssueData =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(issue)
    }

    /// 关闭指定编号的 Issue。
    ///
    /// 调用 `gc issue close <number> --repo <repo> --json <fields>` 关闭 Issue，
    /// 并返回更新后的完整 Issue 数据。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在、已关闭或 `gc` CLI 调用失败时返回错误。
    async fn close(&self, number: u64) -> Result<IssueData> {
        debug!(repo = %self.repo, number, "spawning `gc issue close`");

        let output = tokio::process::Command::new("gc")
            .args(["issue", "close"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .arg("--json")
            .arg(ISSUE_FIELDS)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gc: {e}")))?;

        if !output.status.success() {
            let gc_err = parse_gc_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gc_err}")));
        }

        let issue: IssueData =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(issue)
    }

    /// 重新打开指定编号的 Issue。
    ///
    /// 调用 `gc issue reopen <number> --repo <repo> --json <fields>` 重新打开已关闭的 Issue，
    /// 并返回更新后的完整 Issue 数据。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在、未关闭或 `gc` CLI 调用失败时返回错误。
    async fn reopen(&self, number: u64) -> Result<IssueData> {
        debug!(repo = %self.repo, number, "spawning `gc issue reopen`");

        let output = tokio::process::Command::new("gc")
            .args(["issue", "reopen"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .arg("--json")
            .arg(ISSUE_FIELDS)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gc: {e}")))?;

        if !output.status.success() {
            let gc_err = parse_gc_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gc_err}")));
        }

        let issue: IssueData =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(issue)
    }

    /// 在指定 Issue 上添加评论。
    ///
    /// 调用 `gc issue comment <number> --repo <repo> --body "<body>" --json
    /// id,body,author,createdAt` 发布评论，并返回新建评论的数据。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在、`body` 为空或 `gc` CLI 调用失败时返回错误。
    async fn comment(&self, number: u64, body: &str) -> Result<CommentData> {
        debug!(repo = %self.repo, number, "spawning `gc issue comment`");

        let output = tokio::process::Command::new("gc")
            .args(["issue", "comment"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .arg("--body")
            .arg(body)
            .arg("--json")
            .arg("id,body,author,createdAt")
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gc: {e}")))?;

        if !output.status.success() {
            let gc_err = parse_gc_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gc_err}")));
        }

        let comment: CommentData =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(comment)
    }

    /// 为指定 Issue 添加一个或多个标签。
    ///
    /// 调用 `gc issue edit <number> --repo <repo> --add-label <label>` 逐个添加标签。
    /// 如果 `labels` 为空，不进行任何调用并返回成功。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在、标签名无效或 `gc` CLI 调用失败时返回错误。
    async fn add_labels(&self, number: u64, labels: &[String]) -> Result<()> {
        debug!(
            repo = %self.repo,
            number,
            label_count = labels.len(),
            "spawning `gc issue edit --add-label`"
        );

        let mut cmd = tokio::process::Command::new("gc");
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
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gc: {e}")))?;

        if !output.status.success() {
            let gc_err = parse_gc_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gc_err}")));
        }

        Ok(())
    }

    /// 从指定 Issue 移除一个标签。
    ///
    /// 调用 `gc issue edit <number> --repo <repo> --remove-label <label>` 移除标签。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在、标签未附加到该 Issue 或 `gc` CLI 调用失败时返回错误。
    async fn remove_label(&self, number: u64, label: &str) -> Result<()> {
        debug!(repo = %self.repo, number, label, "spawning `gc issue edit --remove-label`");

        let output = tokio::process::Command::new("gc")
            .args(["issue", "edit"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .arg("--remove-label")
            .arg(label)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gc: {e}")))?;

        if !output.status.success() {
            let gc_err = parse_gc_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gc_err}")));
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
            "id": 1001,
            "body": "Thanks for reporting, looking into it.",
            "author": {"login": "maintainer", "id": "42"},
            "createdAt": "2026-06-15T14:00:00Z"
        }"#;

        let comment: CommentData = serde_json::from_slice(gc_json).expect("valid CommentData");
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
