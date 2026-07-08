//! GitCode Pull Request 提供者实现。
//!
//! 通过 `gitcode` CLI 实现 [`PrProvider`] trait，支持 Pull Request 的创建、列表、查看、
//! 关闭、合并、检出、草稿状态切换和分支同步。
//! 所有方法通过 `tokio::process::Command` 调用 `gc`，捕获 stdout 并解析 JSON。

use async_trait::async_trait;
use gitflow_cli_core::{
    CoreError, Result,
    pr::{CreatePrArgs, ListPrArgs, PrData, PrProvider},
    types::{CommentData, MergeResult, MergeStrategy, State},
};
use tracing::debug;

use crate::error::parse_gitcode_error;

/// `gc pr` 请求的 JSON 字段列表。
const PR_FIELDS: &str =
    "number,title,body,state,draft,author,baseBranch,headBranch,createdAt,updatedAt,url";

/// GitCode Pull Request 提供者，通过 `gitcode` CLI 操作。
///
/// 该结构体通过调用 `gitcode` CLI 实现 [`PrProvider`] trait 的所有方法，
/// 使上层命令能够以统一的方式操作 GitCode Pull Request。
///
/// # Examples
///
/// ```no_run
/// use gitflow_cli_gitcode::GitCodePrProvider;
///
/// let provider = GitCodePrProvider::new("octocat/hello-world");
/// ```
#[derive(Debug, Clone)]
pub struct GitCodePrProvider {
    /// GitCode `owner/repo`，如 `"byx-darwin/gitflow-cli"`。
    repo: String,
}

impl GitCodePrProvider {
    /// 创建新的 GitCode Pull Request 提供者。
    ///
    /// `repo` 格式为 `owner/repo`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> Self {
        Self { repo: repo.into() }
    }
}

#[async_trait]
impl PrProvider for GitCodePrProvider {
    async fn create(&self, args: CreatePrArgs) -> Result<PrData> {
        let mut cmd = tokio::process::Command::new(crate::gitcode_binary());
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
            "spawning `gc pr create`"
        );

        let output = cmd
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
        }

        let pr: PrData =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(pr)
    }

    async fn list(&self, args: ListPrArgs) -> Result<Vec<PrData>> {
        let mut cmd = tokio::process::Command::new(crate::gitcode_binary());
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

        debug!(repo = %self.repo, "spawning `gc pr list`");

        let output = cmd
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
        }

        let prs: Vec<PrData> =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(prs)
    }

    async fn view(&self, number: u64) -> Result<PrData> {
        debug!(repo = %self.repo, number, "spawning `gc pr view`");

        let output = tokio::process::Command::new(crate::gitcode_binary())
            .args(["pr", "view"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .arg("--json")
            .arg(PR_FIELDS)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
        }

        let pr: PrData =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(pr)
    }

    /// 关闭指定编号的 PR。
    ///
    /// 调用 `gc pr close <number> --repo <repo> --json <fields>` 关闭 PR，
    /// 并返回更新后的完整 PR 数据。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在、已关闭或 `gitcode` CLI 调用失败时返回错误。
    async fn close(&self, number: u64) -> Result<PrData> {
        debug!(repo = %self.repo, number, "spawning `gc pr close`");

        let output = tokio::process::Command::new(crate::gitcode_binary())
            .args(["pr", "close"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .arg("--json")
            .arg(PR_FIELDS)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
        }

        let pr: PrData =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(pr)
    }

    /// 重新打开指定编号的 PR。
    ///
    /// 调用 `gc pr reopen <number> --repo <repo> --json <fields>` 重新打开已关闭的 PR，
    /// 并返回更新后的完整 PR 数据。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在、未关闭或 `gitcode` CLI 调用失败时返回错误。
    async fn reopen(&self, number: u64) -> Result<PrData> {
        debug!(repo = %self.repo, number, "spawning `gc pr reopen`");

        let output = tokio::process::Command::new(crate::gitcode_binary())
            .args(["pr", "reopen"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .arg("--json")
            .arg(PR_FIELDS)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
        }

        let pr: PrData =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(pr)
    }

    /// 在指定 PR 上添加评论。
    ///
    /// 调用 `gc pr comment <number> --repo <repo> --body "<body>" --json id,body,author,createdAt`
    /// 发布评论，并返回新建评论的数据。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在、`body` 为空或 `gitcode` CLI 调用失败时返回错误。
    async fn comment(&self, number: u64, body: &str) -> Result<CommentData> {
        debug!(repo = %self.repo, number, "spawning `gc pr comment`");

        let output = tokio::process::Command::new(crate::gitcode_binary())
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
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
        }

        let comment: CommentData =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(comment)
    }

    /// 合并指定编号的 PR。
    ///
    /// 调用 `gc pr merge <number> --repo <repo>` 合并 PR。
    /// 注意：GitCode CLI 当前不支持通过命令行参数指定合并策略（squash/rebase/merge），
    /// 因此 `strategy` 参数会被忽略，并使用 GitCode 平台的默认合并策略。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在、存在冲突无法合并或 `gitcode` CLI 调用失败时返回错误。
    async fn merge(&self, number: u64, strategy: Option<MergeStrategy>) -> Result<MergeResult> {
        if strategy.is_some() {
            tracing::warn!(
                ?strategy,
                "Merge strategies are not yet supported on GitCode platform; using default merge behavior"
            );
        }

        debug!(repo = %self.repo, number, ?strategy, "spawning `gc pr merge`");

        let mut cmd = tokio::process::Command::new(crate::gitcode_binary());
        cmd.args(["pr", "merge"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo);

        let output = cmd
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
        }

        // `gc pr merge` outputs a human-readable message, not JSON.
        let message = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(MergeResult {
            merged: true,
            sha: None,
            message: Some(message),
        })
    }

    /// 在本地检出指定 PR 的分支。
    ///
    /// 调用 `gc pr checkout <number> --repo <repo>` 在本地工作区创建并切换到
    /// PR 的来源分支。如果本地已存在该分支，则尝试更新它。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在、本地 git 操作失败或 `gitcode` CLI 调用失败时返回错误。
    async fn checkout(&self, number: u64) -> Result<()> {
        debug!(repo = %self.repo, number, "spawning `gc pr checkout`");

        let output = tokio::process::Command::new(crate::gitcode_binary())
            .args(["pr", "checkout"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
        }

        Ok(())
    }

    /// 将草稿 PR 标记为可审查状态（ready for review）。
    ///
    /// 调用 `gc pr ready <number> --repo <repo>` 将草稿 PR 转为可审查状态，
    /// 并通过 `gc pr view` 重新获取更新后的 PR 数据。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在、不是草稿状态或 `gitcode` CLI 调用失败时返回错误。
    async fn mark_ready(&self, number: u64) -> Result<PrData> {
        debug!(repo = %self.repo, number, "spawning `gc pr ready`");

        let output = tokio::process::Command::new(crate::gitcode_binary())
            .args(["pr", "ready"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
        }

        // `gc pr ready` does not return JSON; re-fetch the PR to get updated data.
        self.view(number).await
    }

    /// 将 PR 标记为草稿状态（work in progress）。
    ///
    /// 调用 `gc pr convert-to-draft <number> --repo <repo>` 将可审查的 PR 转为草稿，
    /// 并通过 `gc pr view` 重新获取更新后的 PR 数据。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在、已是草稿状态或 `gitcode` CLI 调用失败时返回错误。
    async fn mark_wip(&self, number: u64) -> Result<PrData> {
        debug!(repo = %self.repo, number, "spawning `gc pr convert-to-draft`");

        let output = tokio::process::Command::new(crate::gitcode_binary())
            .args(["pr", "convert-to-draft"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
        }

        // `gc pr convert-to-draft` does not return JSON; re-fetch the PR.
        self.view(number).await
    }

    /// 同步 PR 的分支（将 base 分支的最新变更合入 head 分支）。
    ///
    /// 调用 `gc pr update-branch <number> --repo <repo>` 将 PR 的来源分支
    /// 更新到与目标分支的最新状态同步，解决分支过时问题。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在、同步存在冲突或 `gitcode` CLI 调用失败时返回错误。
    async fn sync_branch(&self, number: u64) -> Result<()> {
        debug!(repo = %self.repo, number, "spawning `gc pr update-branch`");

        let output = tokio::process::Command::new(crate::gitcode_binary())
            .args(["pr", "update-branch"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo)
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
    use super::*;

    #[test]
    fn test_should_construct_gitcode_pr_provider() {
        let provider = GitCodePrProvider::new("octocat/hello-world");
        assert_eq!(provider.repo, "octocat/hello-world");
    }

    #[test]
    fn test_should_construct_gitcode_pr_provider_from_string() {
        let repo = String::from("octocat/hello-world");
        let provider = GitCodePrProvider::new(repo);
        assert_eq!(provider.repo, "octocat/hello-world");
    }

    #[test]
    fn test_should_deserialize_pr_data_from_gc_output() {
        let gc_json = br#"{
            "number": 123,
            "title": "Add new feature",
            "body": "This PR adds a new feature",
            "state": "open",
            "draft": false,
            "author": {"login": "alice", "id": "2"},
            "baseBranch": "main",
            "headBranch": "feature/new-thing",
            "createdAt": "2026-02-20T14:00:00Z",
            "updatedAt": "2026-02-21T10:30:00Z",
            "url": "https://gitcode.com/octocat/hello-world/pull/123"
        }"#;

        let pr: PrData = serde_json::from_slice(gc_json).expect("valid PrData JSON");
        assert_eq!(pr.number, 123);
        assert_eq!(pr.title, "Add new feature");
        assert_eq!(pr.state, State::Open);
        assert!(!pr.draft);
        assert_eq!(pr.author.login, "alice");
        assert_eq!(pr.base_branch, "main");
        assert_eq!(pr.head_branch, "feature/new-thing");
        assert_eq!(pr.url, "https://gitcode.com/octocat/hello-world/pull/123");
    }

    #[test]
    fn test_should_deserialize_empty_pr_list_from_gc_output() {
        let gc_json = b"[]";
        let prs: Vec<PrData> = serde_json::from_slice(gc_json).expect("valid PrData list");
        assert!(prs.is_empty());
    }

    #[test]
    fn test_should_deserialize_draft_pr_from_gc_output() {
        let gc_json = br#"{
            "number": 456,
            "title": "WIP: experiment",
            "body": null,
            "state": "open",
            "draft": true,
            "author": {"login": "bob", "id": "3"},
            "baseBranch": "main",
            "headBranch": "wip/experiment",
            "createdAt": "2026-03-10T09:00:00Z",
            "updatedAt": "2026-03-10T09:00:00Z",
            "url": "https://gitcode.com/octocat/hello-world/pull/456"
        }"#;

        let pr: PrData = serde_json::from_slice(gc_json).expect("valid PrData JSON");
        assert!(pr.draft);
        assert!(pr.body.is_none());
        assert_eq!(pr.title, "WIP: experiment");
    }

    #[test]
    fn test_should_debug_format_provider() {
        let provider = GitCodePrProvider::new("octocat/hello-world");
        let debug = format!("{provider:?}");
        assert!(debug.contains("GitCodePrProvider"));
        assert!(debug.contains("octocat/hello-world"));
    }

    #[test]
    fn test_should_deserialize_closed_pr_from_gc_close_output() {
        let gc_json = br#"{
            "number": 50,
            "title": "Obsolete change",
            "body": "Superseded by #55",
            "state": "closed",
            "draft": false,
            "author": {"login": "dev", "id": "10"},
            "baseBranch": "main",
            "headBranch": "feature/obsolete",
            "createdAt": "2026-05-01T08:00:00Z",
            "updatedAt": "2026-05-02T12:00:00Z",
            "url": "https://gitcode.com/octocat/hello-world/pull/50"
        }"#;

        let pr: PrData = serde_json::from_slice(gc_json).expect("valid closed PrData");
        assert_eq!(pr.number, 50);
        assert_eq!(pr.state, State::Closed);
    }

    #[test]
    fn test_should_deserialize_reopened_pr_from_gc_reopen_output() {
        let gc_json = br#"{
            "number": 50,
            "title": "Obsolete change",
            "body": "Actually still needed",
            "state": "open",
            "draft": false,
            "author": {"login": "dev", "id": "10"},
            "baseBranch": "main",
            "headBranch": "feature/obsolete",
            "createdAt": "2026-05-01T08:00:00Z",
            "updatedAt": "2026-05-03T09:00:00Z",
            "url": "https://gitcode.com/octocat/hello-world/pull/50"
        }"#;

        let pr: PrData = serde_json::from_slice(gc_json).expect("valid reopened PrData");
        assert_eq!(pr.number, 50);
        assert_eq!(pr.state, State::Open);
    }

    #[test]
    fn test_should_deserialize_comment_data_from_gc_pr_comment_output() {
        let gc_json = br#"{
            "id": 2002,
            "body": "Approved, merging now.",
            "author": {"login": "reviewer", "id": "88"},
            "createdAt": "2026-06-20T16:00:00Z"
        }"#;

        let comment: CommentData = serde_json::from_slice(gc_json).expect("valid CommentData");
        assert_eq!(comment.id, 2002);
        assert_eq!(comment.body, "Approved, merging now.");
        assert_eq!(comment.author.login, "reviewer");
        assert_eq!(comment.author.id, "88");
    }

    #[test]
    fn test_should_deserialize_merge_result_from_gc_merge_output() {
        let gc_text = b"Pull request #123 was successfully merged.\n";
        let message = String::from_utf8_lossy(gc_text).trim().to_string();
        let result = MergeResult {
            merged: true,
            sha: None,
            message: Some(message),
        };

        assert!(result.merged);
        assert!(result.message.as_deref().is_some());
        assert_eq!(
            result.message.as_deref(),
            Some("Pull request #123 was successfully merged.")
        );
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
        assert_eq!(round_tripped.message, result.message);
    }

    #[test]
    fn test_should_serialize_merge_result_skips_null_fields() {
        let result = MergeResult {
            merged: false,
            sha: None,
            message: None,
        };
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("null"));
        assert_eq!(json, r#"{"merged":false}"#);
    }

    #[test]
    fn test_should_create_provider_with_different_repos() {
        let r1 = GitCodePrProvider::new("org/repo-a");
        let r2 = GitCodePrProvider::new("org/repo-b");
        assert_eq!(r1.repo, "org/repo-a");
        assert_eq!(r2.repo, "org/repo-b");
    }

    #[test]
    fn test_should_clone_gitcode_pr_provider() {
        let original = GitCodePrProvider::new("owner/repo");
        let cloned = original.clone();
        assert_eq!(original.repo, cloned.repo);
    }
}
