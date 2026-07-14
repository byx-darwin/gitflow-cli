//! Review 领域类型与平台抽象。
//!
//! 定义了 Code Review 的数据表示、状态枚举、评论数据，以及
//! 跨平台实现所需的 [`ReviewProvider`] trait。GitHub、GitLab、
//! `GitCode` 等平台实现都需实现该 trait，使上层命令层可统一消费。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{Result, types::UserSummary};

/// Review 状态枚举。
///
/// 表示一次代码审查的最终结论。
///
/// 使用 `snake_case` 进行序列化，并提供 UPPERCASE 别名
/// 以兼容 GitHub `gh` CLI 的输出格式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewState {
    /// 审查通过，可以合并。
    #[serde(alias = "APPROVED")]
    Approved,
    /// 要求修改后才能合并。
    #[serde(alias = "CHANGES_REQUESTED")]
    ChangesRequested,
    /// 仅发表评论，不表态。
    #[serde(alias = "COMMENTED")]
    Commented,
}

/// Review 数据。
///
/// 由平台实现填充并返回给上层命令。字段命名与
/// `gh pr review` CLI 输出的 JSON 字段对齐（camelCase）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewData {
    /// Review 的 numeric ID。
    pub id: u64,
    /// Review 的最终状态。
    pub state: ReviewState,
    /// Review 正文（Markdown，可选）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// 审查人。
    pub author: UserSummary,
    /// 提交时间（UTC）。
    pub submitted_at: DateTime<Utc>,
}

/// Review 评论数据。
///
/// 表示单条行内或文件级别的代码审查评论。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewCommentData {
    /// 评论的 numeric ID。
    pub id: u64,
    /// 被评论的文件路径（行内评论时有值）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// 评论正文。
    pub body: String,
    /// 被评论的行号（行内评论时有值）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    /// 被评论的代码块差异（hunk）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_hunk: Option<String>,
    /// 评论作者。
    pub author: UserSummary,
    /// 创建时间（UTC）。
    pub created_at: DateTime<Utc>,
}

/// Review 操作的平台抽象。
///
/// 所有平台实现（GitHub/GitLab/GitCode）都必须实现此 trait，
/// 以提供统一的代码审查评论、批准、要求修改及提交审查能力。
///
/// # Errors
///
/// 所有方法在平台调用失败、反序列化失败或鉴权失败时返回
/// [`CoreError`](crate::CoreError)。
///
/// [`CoreError`]: crate::CoreError
#[async_trait]
pub trait ReviewProvider: std::fmt::Debug + Send + Sync {
    /// 在指定 PR 上发表评论。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在、`body` 为空或平台 API 调用失败时返回错误。
    async fn comment(&self, pr_number: u64, body: &str) -> Result<ReviewData>;

    /// 批准指定 PR。
    ///
    /// `body` 为可选的批准说明。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在或平台 API 调用失败时返回错误。
    async fn approve(&self, pr_number: u64, body: Option<&str>) -> Result<ReviewData>;

    /// 要求对指定 PR 进行修改。
    ///
    /// `body` 为修改要求的说明。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在或平台 API 调用失败时返回错误。
    async fn request_changes(&self, pr_number: u64, body: &str) -> Result<ReviewData>;

    /// 提交一次完整的 Review。
    ///
    /// `event` 指定提交的最终结论（批准/要求修改/仅评论），
    /// `body` 为可选的总结说明。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在或平台 API 调用失败时返回错误。
    async fn submit_review(
        &self,
        pr_number: u64,
        event: ReviewState,
        body: Option<&str>,
    ) -> Result<ReviewData>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_review_json() -> &'static str {
        r#"{
            "id": 2001,
            "state": "approved",
            "body": "Looks great, LGTM!",
            "author": {"login": "reviewer", "id": "42"},
            "submittedAt": "2026-05-20T14:30:00Z"
        }"#
    }

    fn sample_review_comment_json() -> &'static str {
        r#"{
            "id": 3001,
            "path": "src/main.rs",
            "body": "Consider using `Result` here",
            "line": 42,
            "diffHunk": "@@ -40,3 +40,5 @@ fn main()",
            "author": {"login": "alice", "id": "7"},
            "createdAt": "2026-05-19T10:00:00Z"
        }"#
    }

    #[test]
    fn test_should_deserialize_review_data_from_json() {
        let json = sample_review_json();
        let review: ReviewData = serde_json::from_str(json).expect("valid ReviewData JSON");

        assert_eq!(review.id, 2001);
        assert_eq!(review.state, ReviewState::Approved);
        assert_eq!(review.body.as_deref(), Some("Looks great, LGTM!"));
        assert_eq!(review.author.login, "reviewer");
        assert_eq!(review.author.id, "42");
    }

    #[test]
    fn test_should_roundtrip_review_data_via_serde() {
        let json = sample_review_json();
        let review: ReviewData = serde_json::from_str(json).expect("deserialize");
        let re_serialized = serde_json::to_string(&review).expect("serialize");
        let round_tripped: ReviewData =
            serde_json::from_str(&re_serialized).expect("re-deserialize");

        assert_eq!(round_tripped.id, review.id);
        assert_eq!(round_tripped.state, review.state);
        assert_eq!(round_tripped.body, review.body);
        assert_eq!(round_tripped.author.login, review.author.login);
        assert_eq!(round_tripped.submitted_at, review.submitted_at);
    }

    #[test]
    fn test_should_deserialize_review_comment_data_from_json() {
        let json = sample_review_comment_json();
        let comment: ReviewCommentData =
            serde_json::from_str(json).expect("valid ReviewCommentData JSON");

        assert_eq!(comment.id, 3001);
        assert_eq!(comment.path.as_deref(), Some("src/main.rs"));
        assert_eq!(comment.body, "Consider using `Result` here");
        assert_eq!(comment.line, Some(42));
        assert_eq!(
            comment.diff_hunk.as_deref(),
            Some("@@ -40,3 +40,5 @@ fn main()")
        );
        assert_eq!(comment.author.login, "alice");
        assert_eq!(comment.author.id, "7");
    }

    #[test]
    fn test_should_roundtrip_review_comment_data_via_serde() {
        let json = sample_review_comment_json();
        let comment: ReviewCommentData = serde_json::from_str(json).expect("deserialize");
        let re_serialized = serde_json::to_string(&comment).expect("serialize");
        let round_tripped: ReviewCommentData =
            serde_json::from_str(&re_serialized).expect("re-deserialize");

        assert_eq!(round_tripped.id, comment.id);
        assert_eq!(round_tripped.path, comment.path);
        assert_eq!(round_tripped.body, comment.body);
        assert_eq!(round_tripped.line, comment.line);
        assert_eq!(round_tripped.diff_hunk, comment.diff_hunk);
        assert_eq!(round_tripped.author.login, comment.author.login);
        assert_eq!(round_tripped.author.id, comment.author.id);
        assert_eq!(round_tripped.created_at, comment.created_at);
    }

    #[test]
    fn test_should_serialize_review_comment_skips_null_fields() {
        let comment = ReviewCommentData {
            id: 1,
            path: None,
            body: "file-level comment".into(),
            line: None,
            diff_hunk: None,
            author: UserSummary {
                login: "bob".into(),
                id: "3".to_string(),
            },
            created_at: "2026-01-01T00:00:00Z".parse().expect("valid date"),
        };
        let json = serde_json::to_string(&comment).expect("serialize");
        assert!(!json.contains("null"));
        assert!(!json.contains("\"path\":"));
        assert!(!json.contains("\"line\":"));
        assert!(!json.contains("\"diffHunk\":"));
        assert!(json.contains("\"body\":\"file-level comment\""));
    }

    #[test]
    fn test_should_serialize_review_state_to_snake_case() {
        let json = serde_json::to_string(&ReviewState::Approved).expect("serialize");
        assert_eq!(json, "\"approved\"");

        let json = serde_json::to_string(&ReviewState::ChangesRequested).expect("serialize");
        assert_eq!(json, "\"changes_requested\"");

        let json = serde_json::to_string(&ReviewState::Commented).expect("serialize");
        assert_eq!(json, "\"commented\"");
    }

    #[test]
    fn test_should_deserialize_review_state_from_snake_case() {
        let state: ReviewState = serde_json::from_str("\"approved\"").expect("deserialize");
        assert_eq!(state, ReviewState::Approved);

        let state: ReviewState =
            serde_json::from_str("\"changes_requested\"").expect("deserialize");
        assert_eq!(state, ReviewState::ChangesRequested);

        let state: ReviewState = serde_json::from_str("\"commented\"").expect("deserialize");
        assert_eq!(state, ReviewState::Commented);
    }

    #[test]
    fn test_should_deserialize_review_state_from_github_uppercase() {
        let state: ReviewState = serde_json::from_str("\"APPROVED\"").expect("deserialize");
        assert_eq!(state, ReviewState::Approved);

        let state: ReviewState =
            serde_json::from_str("\"CHANGES_REQUESTED\"").expect("deserialize");
        assert_eq!(state, ReviewState::ChangesRequested);

        let state: ReviewState = serde_json::from_str("\"COMMENTED\"").expect("deserialize");
        assert_eq!(state, ReviewState::Commented);
    }

    #[test]
    fn test_should_derive_clone_and_copy_for_review_state() {
        let state = ReviewState::Approved;
        let cloned = state;
        let copied = state;
        assert_eq!(state, cloned);
        assert_eq!(state, copied);
    }

    #[test]
    fn test_should_derive_debug_for_review_state() {
        let debug = format!("{:?}", ReviewState::ChangesRequested);
        assert_eq!(debug, "ChangesRequested");
    }

    #[test]
    fn test_should_derive_eq_for_review_state() {
        assert_eq!(ReviewState::Approved, ReviewState::Approved);
        assert_ne!(ReviewState::Approved, ReviewState::ChangesRequested);
    }

    #[test]
    fn test_should_deserialize_review_data_with_null_body() {
        let json = r#"{
            "id": 10,
            "state": "commented",
            "body": null,
            "author": {"login": "u", "id": "1"},
            "submittedAt": "2026-06-01T00:00:00Z"
        }"#;
        let review: ReviewData = serde_json::from_str(json).expect("deserialize");
        assert_eq!(review.state, ReviewState::Commented);
        assert!(review.body.is_none());
    }
}
