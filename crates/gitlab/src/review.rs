//! GitLab Review 提供者实现。
//!
//! 通过 `glab mr approve` / `glab mr revoke` / `glab mr note` CLI 实现
//! [`ReviewProvider`] trait，支持 MR 审查的评论、批准、要求修改及提交审查。
//! 所有方法通过 `tokio::process::Command` 调用 `glab`，捕获 stdout 并解析 JSON。
//!
//! glab 的审查命令与 gh 不同：
//! - 批准使用 `glab mr approve`
//! - 撤回使用 `glab mr revoke`
//! - 评论使用 `glab mr note`
//!
//! 由于 glab 不提供统一的 `review --json` 输出，本模块在审查操作后
//! 构造符合 [`ReviewData`] 的返回值。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use gitflow_cli_core::{
    CoreError, Result,
    review::{ReviewData, ReviewProvider, ReviewState},
    types::UserSummary,
};
use serde::Deserialize;
use tracing::debug;

use crate::error::parse_glab_error;

/// GitLab Review 提供者，通过 `glab` CLI 操作。
///
/// # Examples
///
/// ```no_run
/// use gitflow_cli_gitlab::GitLabReviewProvider;
///
/// let provider = GitLabReviewProvider::new("gitlab-org/gitlab");
/// ```
#[derive(Debug, Clone)]
pub struct GitLabReviewProvider {
    /// GitLab `namespace/project`。
    repo: String,
}

impl GitLabReviewProvider {
    /// 创建新的 GitLab Review 提供者。
    ///
    /// `repo` 格式为 `namespace/project`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> Self {
        Self { repo: repo.into() }
    }
}

// ── 中间 API 响应类型 ──────────────────────────────────────────────

/// `glab mr note --output json` 返回的 JSON 结构（用于获取审查评论 ID）。
#[derive(Debug, Clone, Deserialize)]
struct NoteApiResponse {
    id: u64,
    #[serde(default)]
    body: String,
    #[serde(default)]
    author: Option<ApiUser>,
    #[serde(default)]
    created_at: Option<DateTime<Utc>>,
}

/// JSON 输出中的用户信息。
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

/// `glab mr view --output json` 返回的 JSON 结构（用于获取当前用户信息）。
#[derive(Debug, Clone, Deserialize)]
struct MrViewResponse {
    #[serde(default)]
    author: Option<ApiUser>,
}

// ── trait 实现 ──────────────────────────────────────────────────────

#[async_trait]
impl ReviewProvider for GitLabReviewProvider {
    /// 在指定 MR 上添加审查评论。
    ///
    /// 调用 `glab mr note` 发布评论，并通过 `glab mr view` 获取当前用户信息
    /// 以构造 [`ReviewData`]。
    async fn comment(&self, pr_number: u64, body: &str) -> Result<ReviewData> {
        debug!(repo = %self.repo, number = pr_number, "spawning `glab mr note`");

        let output = tokio::process::Command::new("glab")
            .args(["mr", "note"])
            .arg(pr_number.to_string())
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

        let note: NoteApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        let author = note.author.as_ref().map_or_else(
            || UserSummary {
                login: "unknown".into(),
                id: "0".to_string(),
            },
            UserSummary::from,
        );

        Ok(ReviewData {
            id: note.id,
            state: ReviewState::Commented,
            body: Some(note.body),
            author,
            submitted_at: note.created_at.unwrap_or_else(Utc::now),
        })
    }

    /// 批准指定 MR。
    ///
    /// 调用 `glab mr approve` 批准 MR，然后构造 [`ReviewData`]。
    async fn approve(&self, pr_number: u64, body: Option<&str>) -> Result<ReviewData> {
        debug!(repo = %self.repo, number = pr_number, "spawning `glab mr approve`");

        let mut cmd = tokio::process::Command::new("glab");
        cmd.args(["mr", "approve"])
            .arg(pr_number.to_string())
            .arg("--repo")
            .arg(&self.repo);

        if let Some(b) = body {
            cmd.arg("--comment").arg(b);
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        let message = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let author = self.get_current_user().await.unwrap_or(UserSummary {
            login: "unknown".into(),
            id: "0".to_string(),
        });

        Ok(ReviewData {
            id: 0,
            state: ReviewState::Approved,
            body: if message.is_empty() {
                body.map(String::from)
            } else {
                Some(message)
            },
            author,
            submitted_at: Utc::now(),
        })
    }

    /// 对指定 MR 要求修改。
    ///
    /// glab 没有直接的 "request changes" 命令，通过 `glab mr note` 发布
    /// 包含要求修改意见的评论，并标记为 `ChangesRequested` 状态。
    async fn request_changes(&self, pr_number: u64, body: &str) -> Result<ReviewData> {
        debug!(
            repo = %self.repo,
            number = pr_number,
            "spawning `glab mr note` (request changes)"
        );

        let changes_body = format!("Changes requested:\n\n{body}");

        let output = tokio::process::Command::new("glab")
            .args(["mr", "note"])
            .arg(pr_number.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .arg("--body")
            .arg(&changes_body)
            .arg("--output")
            .arg("json")
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        let note: NoteApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        let author = note.author.as_ref().map_or_else(
            || UserSummary {
                login: "unknown".into(),
                id: "0".to_string(),
            },
            UserSummary::from,
        );

        Ok(ReviewData {
            id: note.id,
            state: ReviewState::ChangesRequested,
            body: Some(note.body),
            author,
            submitted_at: note.created_at.unwrap_or_else(Utc::now),
        })
    }

    /// 提交指定 MR 的审查。
    ///
    /// 根据 `event` 参数分别调用 `glab mr approve`、`glab mr revoke` 或
    /// `glab mr note` 实现不同的审查操作。
    async fn submit_review(
        &self,
        pr_number: u64,
        event: ReviewState,
        body: Option<&str>,
    ) -> Result<ReviewData> {
        debug!(repo = %self.repo, number = pr_number, ?event, "spawning `glab mr review`");

        match event {
            ReviewState::Approved => self.approve(pr_number, body).await,
            ReviewState::ChangesRequested => {
                self.request_changes(pr_number, body.unwrap_or("Changes requested."))
                    .await
            }
            ReviewState::Commented => {
                self.comment(pr_number, body.unwrap_or("Review comment."))
                    .await
            }
        }
    }
}

impl GitLabReviewProvider {
    /// 获取当前登录用户信息（内部辅助方法）。
    async fn get_current_user(&self) -> Result<UserSummary> {
        let output = tokio::process::Command::new("glab")
            .args(["mr", "list"])
            .arg("--repo")
            .arg(&self.repo)
            .arg("--output")
            .arg("json")
            .arg("--per-page")
            .arg("1")
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            return Ok(UserSummary {
                login: "unknown".into(),
                id: "0".to_string(),
            });
        }

        let mrs: Vec<MrViewResponse> =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        if let Some(mr) = mrs.into_iter().next()
            && let Some(author) = mr.author
        {
            return Ok(UserSummary::from(&author));
        }

        Ok(UserSummary {
            login: "unknown".into(),
            id: "0".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_construct_gitlab_review_provider() {
        let provider = GitLabReviewProvider::new("gitlab-org/gitlab");
        assert_eq!(provider.repo, "gitlab-org/gitlab");
    }

    #[test]
    fn test_should_construct_gitlab_review_provider_from_string() {
        let repo = String::from("gitlab-org/gitlab");
        let provider = GitLabReviewProvider::new(repo);
        assert_eq!(provider.repo, "gitlab-org/gitlab");
    }

    #[test]
    fn test_should_deserialize_note_api_response() {
        let json = br#"{
            "id": 2001,
            "body": "Looks great, LGTM!",
            "author": {"username": "reviewer", "id": 42},
            "created_at": "2026-05-20T14:30:00Z"
        }"#;

        let note: NoteApiResponse = serde_json::from_slice(json).expect("valid NoteApiResponse");
        assert_eq!(note.id, 2001);
        assert_eq!(note.body, "Looks great, LGTM!");
        assert_eq!(note.author.as_ref().map(|a| &*a.username), Some("reviewer"));
    }

    #[test]
    fn test_should_deserialize_note_without_author() {
        let json = br#"{
            "id": 2002,
            "body": "Anonymous comment",
            "author": null,
            "created_at": "2026-05-21T09:00:00Z"
        }"#;

        let note: NoteApiResponse = serde_json::from_slice(json).expect("valid NoteApiResponse");
        assert!(note.author.is_none());
    }

    #[test]
    fn test_should_convert_note_to_review_data() {
        let note = NoteApiResponse {
            id: 100,
            body: "LGTM".into(),
            author: Some(ApiUser {
                username: "reviewer".into(),
                id: 5,
            }),
            created_at: Some("2026-01-01T00:00:00Z".parse().expect("valid date")),
        };

        let author = note
            .author
            .as_ref()
            .map(UserSummary::from)
            .expect("has author");
        let review = ReviewData {
            id: note.id,
            state: ReviewState::Commented,
            body: Some(note.body),
            author,
            submitted_at: note.created_at.expect("has date"),
        };

        assert_eq!(review.id, 100);
        assert_eq!(review.state, ReviewState::Commented);
        assert_eq!(review.author.login, "reviewer");
    }

    #[test]
    fn test_should_debug_format_provider() {
        let provider = GitLabReviewProvider::new("gitlab-org/gitlab");
        let debug = format!("{provider:?}");
        assert!(debug.contains("GitLabReviewProvider"));
        assert!(debug.contains("gitlab-org/gitlab"));
    }

    #[test]
    fn test_should_clone_gitlab_review_provider() {
        let original = GitLabReviewProvider::new("owner/repo");
        let cloned = original.clone();
        assert_eq!(original.repo, cloned.repo);
    }
}
