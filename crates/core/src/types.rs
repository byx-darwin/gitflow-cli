//! Shared domain types for gitflow-cli.
//!
//! These are pure data structures used across platform implementations
//! for representing users, states, labels, comments, and merge results.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Deserialize a `u64` that may be a number or string in JSON.
///
/// GitHub/GitLab CLIs return numeric fields as integers, but `GitCode` CLI
/// returns them as strings (e.g. `"number": "3"`).
///
/// # Errors
///
/// Returns an error if the value cannot be parsed as a `u64`.
pub fn deserialize_u64_or_string<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de;

    struct U64OrString;
    impl de::Visitor<'_> for U64OrString {
        type Value = u64;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("a u64 integer or string")
        }

        fn visit_u64<E: de::Error>(self, v: u64) -> Result<u64, E> {
            Ok(v)
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<u64, E> {
            v.parse::<u64>()
                .map_err(|_| de::Error::invalid_value(de::Unexpected::Str(v), &self))
        }
    }

    deserializer.deserialize_any(U64OrString)
}

/// Deserialize a `u64` or string in JSON, returning a `String`.
///
/// GitHub's `gh` CLI returns GraphQL node IDs as integers in some APIs,
/// but `UserSummary.id` is a `String`. This helper handles both cases.
///
/// # Errors
///
/// Returns an error if the value cannot be parsed as a `u64` or `String`.
pub fn deserialize_u64_or_string_to_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de;

    struct U64OrStringToString;
    impl de::Visitor<'_> for U64OrStringToString {
        type Value = String;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("a u64 integer or string")
        }

        fn visit_u64<E: de::Error>(self, v: u64) -> Result<String, E> {
            Ok(v.to_string())
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<String, E> {
            Ok(v.to_string())
        }
    }

    deserializer.deserialize_any(U64OrStringToString)
}

/// A summary of a platform user.
///
/// Contains the minimal identifying information needed to reference
/// a user across API responses.
///
/// Note: `id` is a `String` because GitHub's `gh` CLI returns GraphQL
/// node IDs (e.g. `"U_kgDOCfuwhg"`), while GitLab/GitCode use numeric IDs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSummary {
    /// The user's login name.
    pub login: String,
    /// The user's platform ID (GraphQL node ID on GitHub, numeric on GitLab/GitCode).
    pub id: String,
}

/// The state of an Issue or Pull Request.
///
/// Uses `snake_case` for serialization (`"open"`/`"closed"`), with
/// uppercase aliases for GitHub's `gh` CLI output (`"OPEN"`/`"CLOSED"`/`"MERGED"`)
/// and past-tense alias for `GitCode` (`"opened"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    /// Open and active.
    #[serde(alias = "OPEN", alias = "opened")]
    Open,
    /// Closed or merged.
    #[serde(alias = "CLOSED", alias = "MERGED", alias = "merged")]
    Closed,
}

/// A label attached to an Issue or Pull Request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    /// The label name.
    pub name: String,
    /// The label color as a hex string (e.g. `"ff0000"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// A human-readable description of the label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// A comment on an Issue or Pull Request.
///
/// Returned by `comment` operations on both [`IssueProvider`] and [`PrProvider`].
///
/// [`IssueProvider`]: crate::issue::IssueProvider
/// [`PrProvider`]: crate::pr::PrProvider
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentData {
    /// The comment's unique numeric identifier.
    pub id: u64,
    /// The comment body (Markdown).
    pub body: String,
    /// The comment author.
    pub author: UserSummary,
    /// When the comment was created (UTC).
    pub created_at: DateTime<Utc>,
}

/// The result of a merge operation on a Pull Request.
///
/// Returned by [`PrProvider::merge`] to indicate whether the merge
/// succeeded and, if so, the resulting commit SHA and message.
///
/// [`PrProvider::merge`]: crate::pr::PrProvider::merge
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeResult {
    /// Whether the merge was successful.
    pub merged: bool,
    /// The merge commit SHA (present only when `merged` is `true`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha: Option<String>,
    /// A human-readable merge message from the platform.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// The strategy to use when merging a Pull Request.
///
/// Controls how the platform combines commits from the head branch
/// into the base branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergeStrategy {
    /// A standard merge commit preserving full history.
    Merge,
    /// Squash all commits into a single commit.
    Squash,
    /// Rebase commits onto the base branch.
    Rebase,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_serialize_state_to_snake_case() {
        let open = State::Open;
        let json = serde_json::to_string(&open).expect("serialize Open");
        assert_eq!(json, "\"open\"");

        let closed = State::Closed;
        let json = serde_json::to_string(&closed).expect("serialize Closed");
        assert_eq!(json, "\"closed\"");
    }

    #[test]
    fn test_should_deserialize_state_from_snake_case() {
        let state: State = serde_json::from_str("\"open\"").expect("deserialize open");
        assert_eq!(state, State::Open);

        let state: State = serde_json::from_str("\"closed\"").expect("deserialize closed");
        assert_eq!(state, State::Closed);
    }

    #[test]
    fn test_should_deserialize_state_with_gitcode_opened_alias() {
        let state: State = serde_json::from_str("\"opened\"").expect("deserialize gitcode opened");
        assert_eq!(state, State::Open);
    }

    #[test]
    fn test_should_deserialize_user_summary_from_json() {
        let json = r#"{"login":"octocat","id":"12345"}"#;
        let user: UserSummary = serde_json::from_str(json).expect("valid UserSummary");
        assert_eq!(user.login, "octocat");
        assert_eq!(user.id, "12345");
    }

    #[test]
    fn test_should_serialize_user_summary_to_json() {
        let user = UserSummary {
            login: "octocat".into(),
            id: "12345".to_string(),
        };
        let json = serde_json::to_string(&user).expect("serialize UserSummary");
        assert!(json.contains("\"login\":\"octocat\""));
        assert!(json.contains("\"id\":\"12345\""));
    }

    #[test]
    fn test_should_deserialize_label_with_optional_fields() {
        // With all fields
        let json = r#"{"name":"bug","color":"d73a4a","description":"Something isn't working"}"#;
        let label: Label = serde_json::from_str(json).expect("valid Label");
        assert_eq!(label.name, "bug");
        assert_eq!(label.color, Some("d73a4a".into()));
        assert_eq!(label.description, Some("Something isn't working".into()));
    }

    #[test]
    fn test_should_deserialize_label_with_missing_optional_fields() {
        let json = r#"{"name":"wip"}"#;
        let label: Label = serde_json::from_str(json).expect("valid Label");
        assert_eq!(label.name, "wip");
        assert!(label.color.is_none());
        assert!(label.description.is_none());
    }

    #[test]
    fn test_should_serialize_label_skips_null_fields() {
        let label = Label {
            name: "wip".into(),
            color: None,
            description: None,
        };
        let json = serde_json::to_string(&label).expect("serialize Label");
        // Optional fields with None are omitted per CLAUDE.md serialization policy
        assert_eq!(json, r#"{"name":"wip"}"#);
        assert!(!json.contains("null"));
    }

    #[test]
    fn test_should_derive_debug_for_user_summary() {
        let user = UserSummary {
            login: "test".into(),
            id: "1".to_string(),
        };
        let debug = format!("{user:?}");
        assert!(debug.contains("UserSummary"));
        assert!(debug.contains("login"));
        assert!(debug.contains("test"));
    }

    #[test]
    fn test_should_derive_debug_for_label() {
        let label = Label {
            name: "bug".into(),
            color: Some("ff0000".into()),
            description: None,
        };
        let debug = format!("{label:?}");
        assert!(debug.contains("Label"));
        assert!(debug.contains("bug"));
    }

    #[test]
    fn test_should_derive_clone_for_state() {
        let state = State::Open;
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }

    #[test]
    fn test_should_derive_clone_for_user_summary() {
        let user = UserSummary {
            login: "test".into(),
            id: "42".to_string(),
        };
        let cloned = user.clone();
        assert_eq!(user.login, cloned.login);
        assert_eq!(user.id, cloned.id);
    }

    #[test]
    fn test_should_derive_clone_for_label() {
        let label = Label {
            name: "bug".into(),
            color: None,
            description: None,
        };
        let cloned = label.clone();
        assert_eq!(label.name, cloned.name);
    }

    // --- CommentData tests ---

    #[test]
    fn test_should_deserialize_comment_data_from_camel_case_json() {
        let json = r#"{
            "id": 99,
            "body": "Looks good!",
            "author": {"login": "reviewer", "id": "5"},
            "createdAt": "2026-05-20T14:30:00Z"
        }"#;
        let comment: CommentData = serde_json::from_str(json).expect("valid CommentData");
        assert_eq!(comment.id, 99);
        assert_eq!(comment.body, "Looks good!");
        assert_eq!(comment.author.login, "reviewer");
        assert_eq!(comment.author.id, "5");
    }

    #[test]
    fn test_should_serialize_comment_data_to_camel_case_json() {
        let comment = CommentData {
            id: 1,
            body: "test comment".into(),
            author: UserSummary {
                login: "alice".into(),
                id: "7".to_string(),
            },
            created_at: "2026-01-01T00:00:00Z".parse().expect("valid date"),
        };
        let json = serde_json::to_string(&comment).expect("serialize CommentData");
        assert!(json.contains("\"createdAt\""));
        assert!(json.contains("\"author\""));
        assert!(!json.contains("\"created_at\""));
    }

    #[test]
    fn test_should_roundtrip_comment_data_via_serde() {
        let comment = CommentData {
            id: 42,
            body: "hello".into(),
            author: UserSummary {
                login: "bob".into(),
                id: "3".to_string(),
            },
            created_at: "2026-03-15T10:00:00Z".parse().expect("valid date"),
        };
        let json = serde_json::to_string(&comment).expect("serialize");
        let round_tripped: CommentData = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(round_tripped.id, comment.id);
        assert_eq!(round_tripped.body, comment.body);
        assert_eq!(round_tripped.author.login, comment.author.login);
        assert_eq!(round_tripped.created_at, comment.created_at);
    }

    #[test]
    fn test_should_derive_debug_for_comment_data() {
        let comment = CommentData {
            id: 1,
            body: "hi".into(),
            author: UserSummary {
                login: "u".into(),
                id: "1".to_string(),
            },
            created_at: "2026-01-01T00:00:00Z".parse().expect("valid date"),
        };
        let debug = format!("{comment:?}");
        assert!(debug.contains("CommentData"));
        assert!(debug.contains("body"));
    }

    // --- MergeResult tests ---

    #[test]
    fn test_should_deserialize_merge_result_merged() {
        let json = r#"{"merged":true,"sha":"abc1234","message":"Merged via UI"}"#;
        let result: MergeResult = serde_json::from_str(json).expect("valid MergeResult");
        assert!(result.merged);
        assert_eq!(result.sha.as_deref(), Some("abc1234"));
        assert_eq!(result.message.as_deref(), Some("Merged via UI"));
    }

    #[test]
    fn test_should_deserialize_merge_result_not_merged() {
        let json = r#"{"merged":false}"#;
        let result: MergeResult = serde_json::from_str(json).expect("valid MergeResult");
        assert!(!result.merged);
        assert!(result.sha.is_none());
        assert!(result.message.is_none());
    }

    #[test]
    fn test_should_serialize_merge_result_skips_null_fields() {
        let result = MergeResult {
            merged: false,
            sha: None,
            message: None,
        };
        let json = serde_json::to_string(&result).expect("serialize MergeResult");
        assert!(!json.contains("null"));
        assert!(json.contains("\"merged\":false"));
    }

    #[test]
    fn test_should_serialize_merge_result_camel_case() {
        let result = MergeResult {
            merged: true,
            sha: Some("deadbeef".into()),
            message: Some("ok".into()),
        };
        let json = serde_json::to_string(&result).expect("serialize");
        // Fields are already camelCase (single-word), but struct-level is camelCase
        assert!(json.contains("\"merged\":true"));
        assert!(json.contains("\"sha\":\"deadbeef\""));
    }

    #[test]
    fn test_should_derive_debug_for_merge_result() {
        let result = MergeResult {
            merged: true,
            sha: Some("abc".into()),
            message: None,
        };
        let debug = format!("{result:?}");
        assert!(debug.contains("MergeResult"));
        assert!(debug.contains("merged"));
    }

    // --- MergeStrategy tests ---

    #[test]
    fn test_should_serialize_merge_strategy_to_snake_case() {
        let json = serde_json::to_string(&MergeStrategy::Merge).expect("serialize");
        assert_eq!(json, "\"merge\"");

        let json = serde_json::to_string(&MergeStrategy::Squash).expect("serialize");
        assert_eq!(json, "\"squash\"");

        let json = serde_json::to_string(&MergeStrategy::Rebase).expect("serialize");
        assert_eq!(json, "\"rebase\"");
    }

    #[test]
    fn test_should_deserialize_merge_strategy_from_snake_case() {
        let s: MergeStrategy = serde_json::from_str("\"merge\"").expect("deserialize");
        assert_eq!(s, MergeStrategy::Merge);

        let s: MergeStrategy = serde_json::from_str("\"squash\"").expect("deserialize");
        assert_eq!(s, MergeStrategy::Squash);

        let s: MergeStrategy = serde_json::from_str("\"rebase\"").expect("deserialize");
        assert_eq!(s, MergeStrategy::Rebase);
    }

    #[test]
    fn test_should_derive_clone_and_copy_for_merge_strategy() {
        let strategy = MergeStrategy::Squash;
        let cloned = strategy;
        let copied = strategy;
        assert_eq!(strategy, cloned);
        assert_eq!(strategy, copied);
    }

    #[test]
    fn test_should_derive_debug_for_merge_strategy() {
        let debug = format!("{:?}", MergeStrategy::Rebase);
        assert_eq!(debug, "Rebase");
    }

    #[test]
    fn test_should_derive_eq_for_merge_strategy() {
        assert_eq!(MergeStrategy::Merge, MergeStrategy::Merge);
        assert_ne!(MergeStrategy::Merge, MergeStrategy::Squash);
    }
}
