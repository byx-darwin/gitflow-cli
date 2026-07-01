//! GitHub Pull Request 提供者实现。
//!
//! 通过 `gh` CLI 实现 [`PrProvider`] trait，支持 Pull Request 的创建、列表、查看、
//! 关闭、合并、检出、草稿状态切换和分支同步。
//! 所有方法通过 `tokio::process::Command` 调用 `gh`，捕获 stdout 并解析 JSON。

use async_trait::async_trait;
use gitflow_cli_core::{
    CoreError, Result,
    pr::{CreatePrArgs, ListPrArgs, PrData, PrProvider},
    types::{CommentData, MergeResult, MergeStrategy, State},
};
use tracing::debug;

use crate::error::parse_gh_error;

/// `gh pr` 请求的 JSON 字段列表。
const PR_FIELDS: &str =
    "number,title,body,state,draft,author,baseBranch,headBranch,createdAt,updatedAt,url";

/// GitHub Pull Request 提供者，通过 `gh` CLI 操作。
///
/// 该结构体通过调用 `gh` CLI 实现 [`PrProvider`] trait 的所有方法，
/// 使上层命令能够以统一的方式操作 GitHub Pull Request。
///
/// # Examples
///
/// ```no_run
/// use gitflow_cli_github::GitHubPrProvider;
///
/// let provider = GitHubPrProvider::new("octocat/hello-world");
/// ```
#[derive(Debug, Clone)]
pub struct GitHubPrProvider {
    /// GitHub `owner/repo`，如 `"byx-darwin/gitflow-cli"`。
    repo: String,
}

impl GitHubPrProvider {
    /// 创建新的 GitHub Pull Request 提供者。
    ///
    /// `repo` 格式为 `owner/repo`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> Self {
        Self { repo: repo.into() }
    }
}

#[async_trait]
impl PrProvider for GitHubPrProvider {
    async fn create(&self, args: CreatePrArgs) -> Result<PrData> {
        let mut cmd = tokio::process::Command::new("gh");
        cmd.args(["pr", "create"])
            .arg("--repo")
            .arg(args.repo.as_deref().unwrap_or(&self.repo))
            .arg("--title")
            .arg(&args.title)
            .arg("--head")
            .arg(&args.head)
            .arg("--base")
            .arg(&args.base)
            .arg("--json")
            .arg(PR_FIELDS);

        if let Some(body) = &args.body {
            cmd.arg("--body").arg(body);
        }

        if args.draft {
            cmd.arg("--draft");
        }

        debug!(
            repo = %self.repo,
            title = %args.title,
            head = %args.head,
            base = %args.base,
            "spawning `gh pr create`"
        );

        let output = cmd
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        let pr: PrData =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(pr)
    }

    async fn list(&self, args: ListPrArgs) -> Result<Vec<PrData>> {
        let mut cmd = tokio::process::Command::new("gh");
        cmd.args(["pr", "list"])
            .arg("--repo")
            .arg(&self.repo)
            .arg("--json")
            .arg(PR_FIELDS);

        if let Some(state) = &args.state {
            cmd.arg("--state").arg(match state {
                State::Open => "open",
                State::Closed => "closed",
            });
        }

        if let Some(limit) = args.limit {
            cmd.arg("--limit").arg(limit.to_string());
        }

        debug!(repo = %self.repo, "spawning `gh pr list`");

        let output = cmd
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        let prs: Vec<PrData> =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(prs)
    }

    async fn view(&self, number: u64) -> Result<PrData> {
        debug!(repo = %self.repo, number, "spawning `gh pr view`");

        let output = tokio::process::Command::new("gh")
            .args(["pr", "view"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .arg("--json")
            .arg(PR_FIELDS)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        let pr: PrData =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(pr)
    }

    async fn close(&self, number: u64) -> Result<PrData> {
        debug!(repo = %self.repo, number, "spawning `gh pr close`");

        let output = tokio::process::Command::new("gh")
            .args(["pr", "close"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .arg("--json")
            .arg(PR_FIELDS)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        let pr: PrData =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(pr)
    }

    async fn reopen(&self, number: u64) -> Result<PrData> {
        debug!(repo = %self.repo, number, "spawning `gh pr reopen`");

        let output = tokio::process::Command::new("gh")
            .args(["pr", "reopen"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .arg("--json")
            .arg(PR_FIELDS)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        let pr: PrData =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(pr)
    }

    async fn comment(&self, number: u64, body: &str) -> Result<CommentData> {
        debug!(repo = %self.repo, number, "spawning `gh pr comment`");

        let output = tokio::process::Command::new("gh")
            .args(["pr", "comment"])
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

    async fn merge(&self, number: u64, strategy: Option<MergeStrategy>) -> Result<MergeResult> {
        debug!(repo = %self.repo, number, ?strategy, "spawning `gh pr merge`");

        let mut cmd = tokio::process::Command::new("gh");
        cmd.args(["pr", "merge"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo);

        match strategy {
            Some(MergeStrategy::Squash) => {
                cmd.arg("--squash");
            }
            Some(MergeStrategy::Rebase) => {
                cmd.arg("--rebase");
            }
            Some(MergeStrategy::Merge) | None => {
                cmd.arg("--merge");
            }
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        // `gh pr merge` outputs a human-readable message, not JSON.
        let message = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(MergeResult {
            merged: true,
            sha: None,
            message: Some(message),
        })
    }

    async fn checkout(&self, number: u64) -> Result<()> {
        debug!(repo = %self.repo, number, "spawning `gh pr checkout`");

        let output = tokio::process::Command::new("gh")
            .args(["pr", "checkout"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        Ok(())
    }

    async fn mark_ready(&self, number: u64) -> Result<PrData> {
        debug!(repo = %self.repo, number, "spawning `gh pr ready`");

        let output = tokio::process::Command::new("gh")
            .args(["pr", "ready"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        // `gh pr ready` does not return JSON; re-fetch the PR to get updated data.
        self.view(number).await
    }

    async fn mark_wip(&self, number: u64) -> Result<PrData> {
        debug!(repo = %self.repo, number, "spawning `gh pr convert-to-draft`");

        let output = tokio::process::Command::new("gh")
            .args(["pr", "convert-to-draft"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        // `gh pr convert-to-draft` does not return JSON; re-fetch the PR.
        self.view(number).await
    }

    async fn sync_branch(&self, number: u64) -> Result<()> {
        debug!(repo = %self.repo, number, "spawning `gh pr update-branch`");

        let output = tokio::process::Command::new("gh")
            .args(["pr", "update-branch"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo)
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
            "author": {"login": "alice", "id": 2},
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
            "author": {"login": "bob", "id": 3},
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
}
