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
use gitflow_core::{
    CoreError, Result,
    review::{ReviewData, ReviewProvider, ReviewState},
    types::UserSummary,
};
use serde::Deserialize;
use tracing::debug;

use crate::{
    commit::encode_project_path,
    error::parse_glab_error,
    runner::{CommandRunner, RealCommandRunner},
};

/// GitLab Review 提供者，通过 `glab` CLI 操作。
///
/// # Examples
///
/// ```no_run
/// use gitflow_gitlab::GitLabReviewProvider;
///
/// let provider = GitLabReviewProvider::new("gitlab-org/gitlab");
/// ```
#[derive(Debug, Clone)]
pub struct GitLabReviewProvider<R: CommandRunner = RealCommandRunner> {
    /// GitLab `namespace/project`。
    repo: String,
    /// 用于执行 `glab` CLI 命令的 runner。
    runner: R,
}

impl GitLabReviewProvider<RealCommandRunner> {
    /// 创建新的 GitLab Review 提供者。
    ///
    /// `repo` 格式为 `namespace/project`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> GitLabReviewProvider<RealCommandRunner> {
        GitLabReviewProvider {
            repo: repo.into(),
            runner: RealCommandRunner,
        }
    }

    /// Create a new provider from a shared [`Session`].
    ///
    /// This enables state reuse across multiple operations in workflow chains.
    #[must_use]
    pub fn with_session(session: &gitflow_core::Session) -> Self {
        Self {
            repo: session.repo.clone(),
            runner: RealCommandRunner,
        }
    }
}

impl<R: CommandRunner> GitLabReviewProvider<R> {
    /// 使用自定义 [`CommandRunner`] 创建提供者。
    ///
    /// 主要用于测试，可注入模拟 runner 以控制 `glab` CLI 的输出。
    /// `repo` 格式为 `namespace/project`。
    #[must_use]
    pub fn with_runner(repo: impl Into<String>, runner: R) -> Self {
        Self {
            repo: repo.into(),
            runner,
        }
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
impl<R: CommandRunner + 'static> ReviewProvider for GitLabReviewProvider<R> {
    /// 在指定 MR 上添加审查评论。
    ///
    /// 调用 `glab api --method POST` 发布 note 评论并构造 [`ReviewData`]。
    async fn comment(&self, pr_number: u64, body: &str) -> Result<ReviewData> {
        let note = self.post_note(pr_number, body).await?;
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

        let pr_number_str = pr_number.to_string();
        let mut cmd_args: Vec<&str> = vec!["mr", "approve", &pr_number_str];

        if let Some(b) = body {
            cmd_args.push("--comment");
            cmd_args.push(b);
        }

        let output = self
            .runner
            .run("glab", &cmd_args)
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
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
    /// glab 没有直接的 "request changes" 命令，通过 `glab api --method POST`
    /// 发布包含要求修改意见的 note 评论，并标记为 `ChangesRequested` 状态。
    async fn request_changes(&self, pr_number: u64, body: &str) -> Result<ReviewData> {
        let changes_body = format!("Changes requested:\n\n{body}");
        let note = self.post_note(pr_number, &changes_body).await?;
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

impl<R: CommandRunner> GitLabReviewProvider<R> {
    /// 在指定 MR 上发布一条 note（内部辅助方法）。
    ///
    /// 调用 `glab api --method POST
    /// /projects/{repo-encoded}/merge_requests/{pr_number}/notes`，
    /// 其中 `{repo-encoded}` 为全量 URL 编码的项目路径
    /// （如 `group/subgroup/project` → `group%2Fsubgroup%2Fproject`）。
    ///
    /// # Errors
    ///
    /// 当 MR 不存在或 `glab` CLI 调用失败时返回错误。
    async fn post_note(&self, pr_number: u64, body: &str) -> Result<NoteApiResponse> {
        debug!(repo = %self.repo, number = pr_number, "spawning `glab api` POST mr note");

        let encoded_path = encode_project_path(&self.repo);
        let api_path = format!("/projects/{encoded_path}/merge_requests/{pr_number}/notes");
        let body_arg = format!("body={body}");

        let output = self
            .runner
            .run(
                "glab",
                &["api", "--method", "POST", &api_path, "-f", &body_arg],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab api: {e}")))?;

        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
        }

        serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)
    }

    /// 获取当前登录用户信息（内部辅助方法）。
    async fn get_current_user(&self) -> Result<UserSummary> {
        let output = self
            .runner
            .run(
                "glab",
                &["mr", "list", "--output", "json", "--per-page", "1"],
            )
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
    use crate::runner::{MockCommandRunner, SequencedMockCommandRunner};

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

    // --- Failure-path tests using an injected MockCommandRunner ---

    #[tokio::test]
    async fn test_should_fail_when_review_comment_glab_fails() {
        let runner = MockCommandRunner::failure(r#"{"message": "Forbidden"}"#, 256);
        let provider = GitLabReviewProvider::with_runner("owner/repo", runner);
        let result = provider.comment(7, "fix this").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_should_post_review_note_via_glab_api() {
        let runner = MockCommandRunner::success(
            r#"{"id":99,"body":"fix this","author":{"username":"alice","id":1},"created_at":"2026-08-18T00:00:00Z"}"#,
        );
        let provider = GitLabReviewProvider::with_runner("owner/repo", runner.clone());

        let review = provider.comment(7, "fix this").await.expect("should post");

        assert_eq!(review.id, 99);
        assert_eq!(review.author.login, "alice");
        assert_eq!(review.body.as_deref(), Some("fix this"));
        assert_eq!(
            runner.recorded_calls()[0].1,
            vec![
                "api",
                "--method",
                "POST",
                "/projects/owner%2Frepo/merge_requests/7/notes",
                "-f",
                "body=fix this",
            ]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn test_should_encode_nested_group_repo_path_for_review_note() {
        let runner = MockCommandRunner::success(
            r#"{"id":99,"body":"fix this","author":{"username":"alice","id":1},"created_at":"2026-08-18T00:00:00Z"}"#,
        );
        let provider = GitLabReviewProvider::with_runner("group/subgroup/project", runner.clone());

        let review = provider.comment(7, "fix this").await.expect("should post");

        assert_eq!(review.id, 99);
        assert_eq!(
            runner.recorded_calls()[0].1,
            vec![
                "api",
                "--method",
                "POST",
                "/projects/group%2Fsubgroup%2Fproject/merge_requests/7/notes",
                "-f",
                "body=fix this",
            ]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn test_should_request_changes_via_glab_api() {
        let runner = MockCommandRunner::success(
            r#"{"id":100,"body":"Changes requested:\n\nredo it","author":{"username":"alice","id":1},"created_at":"2026-08-18T00:00:00Z"}"#,
        );
        let provider = GitLabReviewProvider::with_runner("owner/repo", runner.clone());

        let review = provider
            .request_changes(7, "redo it")
            .await
            .expect("should post");

        assert_eq!(review.id, 100);
        assert_eq!(review.state, ReviewState::ChangesRequested);
        assert_eq!(
            review.body.as_deref(),
            Some("Changes requested:\n\nredo it")
        );
        assert_eq!(
            runner.recorded_calls()[0].1,
            vec![
                "api",
                "--method",
                "POST",
                "/projects/owner%2Frepo/merge_requests/7/notes",
                "-f",
                "body=Changes requested:\n\nredo it",
            ]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_should_deserialize_mr_view_response() {
        let json = br#"{
            "author": {"username": "maintainer", "id": 99}
        }"#;

        let mr: MrViewResponse = serde_json::from_slice(json).expect("valid MrViewResponse");
        assert!(mr.author.is_some());
        let author = mr.author.as_ref().expect("author present");
        assert_eq!(author.username, "maintainer");
        assert_eq!(author.id, 99);
    }

    #[test]
    fn test_should_deserialize_mr_view_response_without_author() {
        let json = br#"{"author": null}"#;

        let mr: MrViewResponse = serde_json::from_slice(json).expect("valid MrViewResponse");
        assert!(mr.author.is_none());
    }

    #[test]
    fn test_should_deserialize_note_without_created_at() {
        let json = br#"{
            "id": 3001,
            "body": "No timestamp",
            "author": null,
            "created_at": null
        }"#;

        let note: NoteApiResponse = serde_json::from_slice(json).expect("valid NoteApiResponse");
        assert!(note.created_at.is_none());
        assert!(note.author.is_none());
    }

    #[test]
    fn test_should_convert_api_user_to_user_summary() {
        let user = ApiUser {
            username: "reviewer".into(),
            id: 42,
        };
        let summary: UserSummary = (&user).into();
        assert_eq!(summary.login, "reviewer");
        assert_eq!(summary.id, "42");
    }

    #[test]
    fn test_should_convert_api_user_with_zero_id_to_user_summary() {
        let user = ApiUser {
            username: "bot".into(),
            id: 0,
        };
        let summary: UserSummary = (&user).into();
        assert_eq!(summary.login, "bot");
        assert_eq!(summary.id, "0");
    }

    // --- approve() tests: --repo dropped, glab auto-detects from git remote ---

    #[tokio::test]
    async fn test_should_approve_mr_without_repo_flag() {
        // Sequenced: first `glab mr approve` succeeds, then `glab mr list` (get_current_user).
        let runner = SequencedMockCommandRunner::from_results(&[
            (true, "Approved MR #7"),
            (true, r#"[{"author":{"username":"alice","id":1}}]"#),
        ]);
        let provider = GitLabReviewProvider::with_runner("owner/repo", runner.clone());

        let review = provider
            .approve(7, Some("LGTM"))
            .await
            .expect("should approve");

        assert_eq!(review.state, ReviewState::Approved);
        assert_eq!(review.author.login, "alice");

        let calls = runner.recorded_calls();
        // First call: `glab mr approve <num> --comment <body>` — NO --repo.
        assert_eq!(
            calls[0].1,
            vec!["mr", "approve", "7", "--comment", "LGTM"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );
        // Second call: `glab mr list --output json --per-page 1` — NO --repo.
        assert_eq!(
            calls[1].1,
            vec!["mr", "list", "--output", "json", "--per-page", "1"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn test_should_approve_nested_group_mr_without_repo_flag() {
        // Regression: 3-segment path (iproost/proxy/edge) must NOT pass --repo,
        // which glab cannot parse. Dropping --repo lets glab auto-detect.
        let runner = SequencedMockCommandRunner::from_results(&[
            (true, "Approved MR #26"),
            (true, r#"[{"author":{"username":"reviewer","id":42}}]"#),
        ]);
        let provider = GitLabReviewProvider::with_runner("iproost/proxy/edge", runner.clone());

        let review = provider
            .approve(26, Some("LGTM"))
            .await
            .expect("should approve");

        assert_eq!(review.state, ReviewState::Approved);
        assert_eq!(review.author.login, "reviewer");

        let calls = runner.recorded_calls();
        assert_eq!(
            calls[0].1,
            vec!["mr", "approve", "26", "--comment", "LGTM"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            calls[1].1,
            vec!["mr", "list", "--output", "json", "--per-page", "1"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );
    }
}
