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

        debug!(repo = %self.repo, title = %args.title, "spawning `gh issue create`");

        let output = cmd
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

    async fn comment(&self, number: u64, body: &str) -> Result<CommentData> {
        debug!(repo = %self.repo, number, "spawning `gh issue comment`");

        let output = tokio::process::Command::new("gh")
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
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        let comment: CommentData =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(comment)
    }

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

#[cfg(test)]
mod tests {
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
            "author": {"login": "octocat", "id": 1},
            "assignees": [{"login": "alice", "id": 7}],
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
}
