//! GitHub Review 提供者实现。
//!
//! 通过 `gh` CLI 实现 [`ReviewProvider`] trait，支持 PR 审查的评论、
//! 批准、要求修改及提交审查。
//! 所有方法通过 `tokio::process::Command` 调用 `gh`，捕获 stdout 并解析 JSON。

use async_trait::async_trait;
use gitflow_cli_core::{
    CoreError, Result,
    review::{ReviewData, ReviewProvider, ReviewState},
};
use tracing::debug;

use crate::{error::parse_gh_error, issue::GitHubUser};

/// GitHub Review 提供者，通过 `gh` CLI 操作。
///
/// 该结构体通过调用 `gh` CLI 实现 [`ReviewProvider`] trait 的所有方法，
/// 使上层命令能够以统一的方式操作 GitHub PR 审查。
///
/// # Examples
///
/// ```no_run
/// use gitflow_cli_github::GitHubReviewProvider;
///
/// let provider = GitHubReviewProvider::new("octocat/hello-world");
/// ```
#[derive(Debug, Clone)]
pub struct GitHubReviewProvider {
    /// GitHub `owner/repo`，如 `"byx-darwin/gitflow-cli"`。
    repo: String,
}

impl GitHubReviewProvider {
    /// 创建新的 GitHub Review 提供者。
    ///
    /// `repo` 格式为 `owner/repo`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> Self {
        Self { repo: repo.into() }
    }
}

#[async_trait]
impl ReviewProvider for GitHubReviewProvider {
    async fn comment(&self, pr_number: u64, body: &str) -> Result<ReviewData> {
        debug!(repo = %self.repo, number = pr_number, "spawning `gh pr review --comment`");

        // 1. 执行 gh pr review --comment（不返回 JSON）
        let number_str = pr_number.to_string();
        let output = tokio::process::Command::new("gh")
            .args(["pr", "review"])
            .arg(&number_str)
            .arg("--comment")
            .arg("--body")
            .arg(body)
            .arg("--repo")
            .arg(&self.repo)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            return Err(parse_gh_error(&output.stderr).into());
        }

        // 2. 使用 gh api 获取该 PR 的最新 review
        self.fetch_latest_review(pr_number).await
    }

    async fn approve(&self, pr_number: u64, body: Option<&str>) -> Result<ReviewData> {
        debug!(repo = %self.repo, number = pr_number, "spawning `gh pr review --approve`");

        let mut cmd = tokio::process::Command::new("gh");
        cmd.args(["pr", "review"])
            .arg(pr_number.to_string())
            .arg("--approve")
            .arg("--repo")
            .arg(&self.repo);

        if let Some(b) = body {
            cmd.arg("--body").arg(b);
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            // 检测 "approve your own pull request" 错误
            if gh_err
                .user_message
                .contains("approve your own pull request")
            {
                return Err(CoreError::Platform(
                    "GitHub 不允许批准自己的 PR。可以请求其他维护者审查。".to_string(),
                ));
            }
            return Err(gh_err.into());
        }

        // 使用 gh api 获取该 PR 的最新 review
        self.fetch_latest_review(pr_number).await
    }

    async fn request_changes(&self, pr_number: u64, body: &str) -> Result<ReviewData> {
        debug!(repo = %self.repo, number = pr_number, "spawning `gh pr review --request-changes`");

        let output = tokio::process::Command::new("gh")
            .args(["pr", "review"])
            .arg(pr_number.to_string())
            .arg("--request-changes")
            .arg("--body")
            .arg(body)
            .arg("--repo")
            .arg(&self.repo)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            return Err(parse_gh_error(&output.stderr).into());
        }

        // 使用 gh api 获取该 PR 的最新 review
        self.fetch_latest_review(pr_number).await
    }

    async fn submit_review(
        &self,
        pr_number: u64,
        event: ReviewState,
        body: Option<&str>,
    ) -> Result<ReviewData> {
        debug!(repo = %self.repo, number = pr_number, ?event, "spawning `gh pr review`");

        let mut cmd = tokio::process::Command::new("gh");
        cmd.args(["pr", "review"])
            .arg(pr_number.to_string())
            .arg("--repo")
            .arg(&self.repo);

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
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            if gh_err
                .user_message
                .contains("approve your own pull request")
            {
                return Err(CoreError::Platform(
                    "GitHub 不允许批准自己的 PR。可以请求其他维护者审查。".to_string(),
                ));
            }
            return Err(gh_err.into());
        }

        // 使用 gh api 获取该 PR 的最新 review
        self.fetch_latest_review(pr_number).await
    }
}

impl GitHubReviewProvider {
    /// 使用 gh api 获取指定 PR 的最新 review。
    ///
    /// 在提交 review 后调用，以获取完整的 review 数据（ID、状态、时间戳等）。
    async fn fetch_latest_review(&self, pr_number: u64) -> Result<ReviewData> {
        let api_path = format!(
            "repos/{repo}/pulls/{number}/reviews?per_page=1",
            repo = self.repo,
            number = pr_number
        );

        let output = tokio::process::Command::new("gh")
            .args(["api", &api_path])
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh api: {e}")))?;

        if !output.status.success() {
            let gh_err = String::from_utf8_lossy(&output.stderr);
            return Err(CoreError::Platform(format!(
                "Failed to fetch review via gh api: {gh_err}"
            )));
        }

        let reviews: Vec<GitHubReviewApiResponse> =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        let review = reviews
            .into_iter()
            .next()
            .ok_or_else(|| CoreError::Platform("No review returned from gh api".to_string()))?;

        Ok(review.into())
    }
}

/// GitHub API Review 响应结构。
///
/// 用于解析 `gh api repos/{owner}/{repo}/pulls/{number}/reviews` 的返回数据。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct GitHubReviewApiResponse {
    /// Review 的 numeric ID。
    pub id: u64,
    /// Review 的状态（APPROVED, CHANGES_REQUESTED, COMMENTED 等）。
    pub state: String,
    /// Review 正文（Markdown，可选）。
    #[serde(default)]
    pub body: Option<String>,
    /// 审查人。
    pub user: GitHubUser,
    /// 提交时间（UTC，ISO 8601 格式）。
    pub submitted_at: String,
}

impl From<GitHubReviewApiResponse> for ReviewData {
    fn from(api: GitHubReviewApiResponse) -> Self {
        Self {
            id: api.id,
            state: match api.state.as_str() {
                "APPROVED" => ReviewState::Approved,
                "CHANGES_REQUESTED" => ReviewState::ChangesRequested,
                _ => ReviewState::Commented,
            },
            body: api.body,
            author: gitflow_cli_core::types::UserSummary {
                login: api.user.login,
                id: api.user.id.to_string(),
            },
            submitted_at: api
                .submitted_at
                .parse()
                .unwrap_or_else(|_| chrono::Utc::now()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_construct_github_review_provider() {
        let provider = GitHubReviewProvider::new("octocat/hello-world");
        assert_eq!(provider.repo, "octocat/hello-world");
    }

    #[test]
    fn test_should_construct_github_review_provider_from_string() {
        let repo = String::from("octocat/hello-world");
        let provider = GitHubReviewProvider::new(repo);
        assert_eq!(provider.repo, "octocat/hello-world");
    }

    #[test]
    fn test_should_deserialize_review_data_from_gh_output() {
        // 模拟 `gh pr review --json ...` 的实际输出
        let gh_json = br#"{
            "id": 2001,
            "state": "approved",
            "body": "Looks great, LGTM!",
            "author": {"login": "reviewer", "id": "42"},
            "submittedAt": "2026-05-20T14:30:00Z"
        }"#;

        let review: ReviewData = serde_json::from_slice(gh_json).expect("valid ReviewData JSON");
        assert_eq!(review.id, 2001);
        assert_eq!(review.state, ReviewState::Approved);
        assert_eq!(review.body.as_deref(), Some("Looks great, LGTM!"));
        assert_eq!(review.author.login, "reviewer");
        assert_eq!(review.author.id, "42");
    }

    #[test]
    fn test_should_deserialize_changes_requested_review_from_gh_output() {
        let gh_json = br#"{
            "id": 2002,
            "state": "changes_requested",
            "body": "Please fix the error handling",
            "author": {"login": "senior-dev", "id": "7"},
            "submittedAt": "2026-05-21T09:00:00Z"
        }"#;

        let review: ReviewData = serde_json::from_slice(gh_json).expect("valid ReviewData");
        assert_eq!(review.state, ReviewState::ChangesRequested);
        assert_eq!(
            review.body.as_deref(),
            Some("Please fix the error handling")
        );
    }

    #[test]
    fn test_should_deserialize_commented_review_from_gh_output() {
        let gh_json = br#"{
            "id": 2003,
            "state": "commented",
            "body": null,
            "author": {"login": "observer", "id": "15"},
            "submittedAt": "2026-05-22T11:00:00Z"
        }"#;

        let review: ReviewData = serde_json::from_slice(gh_json).expect("valid ReviewData");
        assert_eq!(review.state, ReviewState::Commented);
        assert!(review.body.is_none());
    }

    #[test]
    fn test_should_debug_format_provider() {
        let provider = GitHubReviewProvider::new("octocat/hello-world");
        let debug = format!("{provider:?}");
        assert!(debug.contains("GitHubReviewProvider"));
        assert!(debug.contains("octocat/hello-world"));
    }

    #[test]
    fn test_should_create_provider_with_different_repos() {
        let r1 = GitHubReviewProvider::new("org/repo-a");
        let r2 = GitHubReviewProvider::new("org/repo-b");
        assert_eq!(r1.repo, "org/repo-a");
        assert_eq!(r2.repo, "org/repo-b");
    }

    #[test]
    fn test_should_clone_github_review_provider() {
        let original = GitHubReviewProvider::new("owner/repo");
        let cloned = original.clone();
        assert_eq!(original.repo, cloned.repo);
    }

    #[test]
    fn test_should_deserialize_github_review_api_response() {
        let json = r#"{
            "id": 12345,
            "state": "APPROVED",
            "body": "LGTM",
            "user": {"login": "octocat", "id": 1},
            "submitted_at": "2026-08-03T10:00:00Z"
        }"#;

        let response: GitHubReviewApiResponse = serde_json::from_str(json).expect("valid response");

        assert_eq!(response.id, 12345);
        assert_eq!(response.state, "APPROVED");
        assert_eq!(response.body, Some("LGTM".to_string()));
        assert_eq!(response.user.login, "octocat");
        assert_eq!(response.user.id, 1);
        assert_eq!(response.submitted_at, "2026-08-03T10:00:00Z");
    }

    #[test]
    fn test_should_convert_github_review_api_response_to_review_data() {
        let api_response = GitHubReviewApiResponse {
            id: 12345,
            state: "APPROVED".to_string(),
            body: Some("LGTM".to_string()),
            user: GitHubUser {
                login: "octocat".to_string(),
                id: 1,
            },
            submitted_at: "2026-08-03T10:00:00Z".to_string(),
        };

        let review_data: ReviewData = api_response.into();

        assert_eq!(review_data.id, 12345);
        assert_eq!(review_data.state, ReviewState::Approved);
        assert_eq!(review_data.body, Some("LGTM".to_string()));
        assert_eq!(review_data.author.login, "octocat");
        assert_eq!(review_data.author.id, "1");
    }

    #[test]
    fn test_should_convert_all_review_states() {
        let states = vec![
            ("APPROVED", ReviewState::Approved),
            ("CHANGES_REQUESTED", ReviewState::ChangesRequested),
            ("COMMENTED", ReviewState::Commented),
        ];

        for (state_str, expected_state) in states {
            let api_response = GitHubReviewApiResponse {
                id: 1,
                state: state_str.to_string(),
                body: None,
                user: GitHubUser {
                    login: "test".to_string(),
                    id: 1,
                },
                submitted_at: "2026-08-03T10:00:00Z".to_string(),
            };

            let review_data: ReviewData = api_response.into();
            assert_eq!(review_data.state, expected_state);
        }
    }
}
