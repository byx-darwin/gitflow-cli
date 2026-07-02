//! GitCode Review 提供者实现。
//!
//! 通过 `gc` CLI 实现 [`ReviewProvider`] trait，支持 PR 审查的评论、
//! 批准、要求修改及提交审查。
//! 所有方法通过 `tokio::process::Command` 调用 `gc`，捕获 stdout 并解析 JSON。

use async_trait::async_trait;
use gitflow_cli_core::{
    CoreError, Result,
    review::{ReviewData, ReviewProvider, ReviewState},
};
use tracing::debug;

use crate::error::parse_gc_error;

/// `gc pr review` 请求的 JSON 字段列表。
const REVIEW_FIELDS: &str = "id,state,body,author,submittedAt";

/// GitCode Review 提供者，通过 `gc` CLI 操作。
///
/// 该结构体通过调用 `gc` CLI 实现 [`ReviewProvider`] trait 的所有方法，
/// 使上层命令能够以统一的方式操作 GitCode PR 审查。
///
/// # Examples
///
/// ```no_run
/// use gitflow_cli_gitcode::GitCodeReviewProvider;
///
/// let provider = GitCodeReviewProvider::new("octocat/hello-world");
/// ```
#[derive(Debug, Clone)]
pub struct GitCodeReviewProvider {
    /// GitCode `owner/repo`，如 `"byx-darwin/gitflow-cli"`。
    repo: String,
}

impl GitCodeReviewProvider {
    /// 创建新的 GitCode Review 提供者。
    ///
    /// `repo` 格式为 `owner/repo`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> Self {
        Self { repo: repo.into() }
    }
}

#[async_trait]
impl ReviewProvider for GitCodeReviewProvider {
    async fn comment(&self, pr_number: u64, body: &str) -> Result<ReviewData> {
        debug!(repo = %self.repo, number = pr_number, "spawning `gc pr review --comment`");

        let output = tokio::process::Command::new("gc")
            .args(["pr", "review"])
            .arg(pr_number.to_string())
            .arg("--comment")
            .arg("--body")
            .arg(body)
            .arg("--repo")
            .arg(&self.repo)
            .arg("--json")
            .arg(REVIEW_FIELDS)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gc: {e}")))?;

        if !output.status.success() {
            let gc_err = parse_gc_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gc_err}")));
        }

        let review: ReviewData =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(review)
    }

    async fn approve(&self, pr_number: u64, body: Option<&str>) -> Result<ReviewData> {
        debug!(repo = %self.repo, number = pr_number, "spawning `gc pr review --approve`");

        let mut cmd = tokio::process::Command::new("gc");
        cmd.args(["pr", "review"])
            .arg(pr_number.to_string())
            .arg("--approve")
            .arg("--repo")
            .arg(&self.repo)
            .arg("--json")
            .arg(REVIEW_FIELDS);

        if let Some(b) = body {
            cmd.arg("--body").arg(b);
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gc: {e}")))?;

        if !output.status.success() {
            let gc_err = parse_gc_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gc_err}")));
        }

        let review: ReviewData =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(review)
    }

    async fn request_changes(&self, pr_number: u64, body: &str) -> Result<ReviewData> {
        debug!(repo = %self.repo, number = pr_number, "spawning `gc pr review --request-changes`");

        let output = tokio::process::Command::new("gc")
            .args(["pr", "review"])
            .arg(pr_number.to_string())
            .arg("--request-changes")
            .arg("--body")
            .arg(body)
            .arg("--repo")
            .arg(&self.repo)
            .arg("--json")
            .arg(REVIEW_FIELDS)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gc: {e}")))?;

        if !output.status.success() {
            let gc_err = parse_gc_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gc_err}")));
        }

        let review: ReviewData =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(review)
    }

    async fn submit_review(
        &self,
        pr_number: u64,
        event: ReviewState,
        body: Option<&str>,
    ) -> Result<ReviewData> {
        debug!(repo = %self.repo, number = pr_number, ?event, "spawning `gc pr review`");

        let mut cmd = tokio::process::Command::new("gc");
        cmd.args(["pr", "review"])
            .arg(pr_number.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .arg("--json")
            .arg(REVIEW_FIELDS);

        match event {
            ReviewState::Approved => {
                cmd.arg("--approve");
            }
            ReviewState::ChangesRequested => {
                cmd.arg("--request-changes");
            }
            ReviewState::Commented => {
                cmd.arg("--comment");
            }
        }

        if let Some(b) = body {
            cmd.arg("--body").arg(b);
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gc: {e}")))?;

        if !output.status.success() {
            let gc_err = parse_gc_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gc_err}")));
        }

        let review: ReviewData =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(review)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_construct_gitcode_review_provider() {
        let provider = GitCodeReviewProvider::new("octocat/hello-world");
        assert_eq!(provider.repo, "octocat/hello-world");
    }

    #[test]
    fn test_should_construct_gitcode_review_provider_from_string() {
        let repo = String::from("octocat/hello-world");
        let provider = GitCodeReviewProvider::new(repo);
        assert_eq!(provider.repo, "octocat/hello-world");
    }

    #[test]
    fn test_should_deserialize_review_data_from_gc_output() {
        let gc_json = br#"{
            "id": 2001,
            "state": "approved",
            "body": "Looks great, LGTM!",
            "author": {"login": "reviewer", "id": 42},
            "submittedAt": "2026-05-20T14:30:00Z"
        }"#;

        let review: ReviewData = serde_json::from_slice(gc_json).expect("valid ReviewData JSON");
        assert_eq!(review.id, 2001);
        assert_eq!(review.state, ReviewState::Approved);
        assert_eq!(review.body.as_deref(), Some("Looks great, LGTM!"));
        assert_eq!(review.author.login, "reviewer");
        assert_eq!(review.author.id, 42);
    }

    #[test]
    fn test_should_deserialize_changes_requested_review_from_gc_output() {
        let gc_json = br#"{
            "id": 2002,
            "state": "changes_requested",
            "body": "Please fix the error handling",
            "author": {"login": "senior-dev", "id": 7},
            "submittedAt": "2026-05-21T09:00:00Z"
        }"#;

        let review: ReviewData = serde_json::from_slice(gc_json).expect("valid ReviewData");
        assert_eq!(review.state, ReviewState::ChangesRequested);
        assert_eq!(
            review.body.as_deref(),
            Some("Please fix the error handling")
        );
    }

    #[test]
    fn test_should_deserialize_commented_review_from_gc_output() {
        let gc_json = br#"{
            "id": 2003,
            "state": "commented",
            "body": null,
            "author": {"login": "observer", "id": 15},
            "submittedAt": "2026-05-22T11:00:00Z"
        }"#;

        let review: ReviewData = serde_json::from_slice(gc_json).expect("valid ReviewData");
        assert_eq!(review.state, ReviewState::Commented);
        assert!(review.body.is_none());
    }

    #[test]
    fn test_should_debug_format_provider() {
        let provider = GitCodeReviewProvider::new("octocat/hello-world");
        let debug = format!("{provider:?}");
        assert!(debug.contains("GitCodeReviewProvider"));
        assert!(debug.contains("octocat/hello-world"));
    }

    #[test]
    fn test_should_create_provider_with_different_repos() {
        let r1 = GitCodeReviewProvider::new("org/repo-a");
        let r2 = GitCodeReviewProvider::new("org/repo-b");
        assert_eq!(r1.repo, "org/repo-a");
        assert_eq!(r2.repo, "org/repo-b");
    }

    #[test]
    fn test_should_clone_gitcode_review_provider() {
        let original = GitCodeReviewProvider::new("owner/repo");
        let cloned = original.clone();
        assert_eq!(original.repo, cloned.repo);
    }
}
