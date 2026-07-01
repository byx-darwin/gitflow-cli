//! Commit resource types with platform abstraction.
//!
//! Defines data types for representing commit summaries, detailed views
//! with diffs and file-level changes, along with the [`CommitProvider`]
//! trait for cross-platform implementations (GitHub, GitLab, `GitCode`, etc.).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{Result, types::UserSummary};

/// A summary of a single commit.
///
/// Contains the essential metadata needed for commit listings and
/// timeline views.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitData {
    /// The full SHA hash of the commit.
    pub sha: String,
    /// The commit message (first line + body).
    pub message: String,
    /// The commit author (the person who wrote the code).
    pub author: UserSummary,
    /// The committer (the person who applied the commit).
    pub committer: UserSummary,
    /// The number of lines added in this commit.
    pub additions: u64,
    /// The number of lines deleted in this commit.
    pub deletions: u64,
    /// The number of files changed in this commit.
    pub files_changed: u64,
}

/// A detailed view of a commit, including diff and per-file changes.
///
/// Returned by [`CommitProvider::view`] when the caller needs the full
/// diff payload and file-level breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitDetail {
    /// The full SHA hash of the commit.
    pub sha: String,
    /// The commit message (first line + body).
    pub message: String,
    /// The commit author.
    pub author: UserSummary,
    /// The committer.
    pub committer: UserSummary,
    /// The full unified diff output (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff: Option<String>,
    /// The list of changed files.
    #[serde(default)]
    pub files: Vec<CommitFile>,
}

/// A single file change within a commit.
///
/// Tracks the path, line counts, and status (added, modified, deleted, etc.)
/// for each file touched by a commit.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitFile {
    /// The file path relative to the repository root.
    pub filename: String,
    /// The number of lines added in this file.
    pub additions: u64,
    /// The number of lines deleted in this file.
    pub deletions: u64,
    /// The file status (e.g. `"added"`, `"modified"`, `"deleted"`, `"renamed"`).
    pub status: String,
}

/// Platform abstraction for commit inspection and commenting.
///
/// All platform implementations (GitHub/GitLab/GitCode) must implement
/// this trait to provide unified commit viewing, diffing, patch retrieval,
/// and commenting capabilities.
///
/// # Errors
///
/// All methods return [`CoreError`](crate::CoreError) on platform API
/// failure, deserialization errors, or authentication failures.
#[async_trait]
pub trait CommitProvider: std::fmt::Debug + Send + Sync {
    /// View the details of a commit by SHA.
    ///
    /// # Errors
    ///
    /// Returns an error if the commit does not exist or the platform API call fails.
    async fn view(&self, sha: &str) -> Result<CommitDetail>;

    /// Retrieve the unified diff for a commit by SHA.
    ///
    /// # Errors
    ///
    /// Returns an error if the commit does not exist or the platform API call fails.
    async fn diff(&self, sha: &str) -> Result<String>;

    /// Retrieve the raw patch content for a commit by SHA.
    ///
    /// # Errors
    ///
    /// Returns an error if the commit does not exist or the platform API call fails.
    async fn patch(&self, sha: &str) -> Result<String>;

    /// Comment on a specific file line within a commit.
    ///
    /// # Arguments
    ///
    /// * `sha` - The commit SHA to comment on.
    /// * `body` - The comment body (Markdown).
    /// * `path` - The file path within the commit.
    /// * `line` - The line number to comment on (1-based).
    ///
    /// # Errors
    ///
    /// Returns an error if the commit or file does not exist, the line number
    /// is out of range, or the platform API call fails.
    async fn comment(&self, sha: &str, body: &str, path: &str, line: u64) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- CommitData tests ---

    fn sample_commit_json() -> &'static str {
        r#"{
            "sha": "abc1234567890def",
            "message": "Fix login redirect loop\n\nThe auth middleware was missing a redirect check.",
            "author": {"login": "alice", "id": 7},
            "committer": {"login": "bob", "id": 12},
            "additions": 15,
            "deletions": 3,
            "filesChanged": 2
        }"#
    }

    #[test]
    fn test_should_deserialize_commit_data_from_camel_case_json() {
        let json = sample_commit_json();
        let commit: CommitData = serde_json::from_str(json).expect("valid CommitData");
        assert_eq!(commit.sha, "abc1234567890def");
        assert!(commit.message.contains("Fix login redirect loop"));
        assert_eq!(commit.author.login, "alice");
        assert_eq!(commit.committer.login, "bob");
        assert_eq!(commit.additions, 15);
        assert_eq!(commit.deletions, 3);
        assert_eq!(commit.files_changed, 2);
    }

    #[test]
    fn test_should_serialize_commit_data_to_camel_case_json() {
        let json = sample_commit_json();
        let commit: CommitData = serde_json::from_str(json).expect("deserialize");
        let serialized = serde_json::to_string(&commit).expect("serialize");
        assert!(serialized.contains("\"filesChanged\""));
        assert!(serialized.contains("\"additions\""));
        assert!(!serialized.contains("\"files_changed\""));
    }

    #[test]
    fn test_should_roundtrip_commit_data_via_serde() {
        let json = sample_commit_json();
        let commit: CommitData = serde_json::from_str(json).expect("deserialize");
        let re_serialized = serde_json::to_string(&commit).expect("serialize");
        let round_tripped: CommitData =
            serde_json::from_str(&re_serialized).expect("re-deserialize");
        assert_eq!(round_tripped.sha, commit.sha);
        assert_eq!(round_tripped.message, commit.message);
        assert_eq!(round_tripped.author.login, commit.author.login);
        assert_eq!(round_tripped.committer.login, commit.committer.login);
        assert_eq!(round_tripped.additions, commit.additions);
        assert_eq!(round_tripped.deletions, commit.deletions);
        assert_eq!(round_tripped.files_changed, commit.files_changed);
    }

    #[test]
    fn test_should_derive_debug_for_commit_data() {
        let commit = CommitData {
            sha: "abc".into(),
            message: "test".into(),
            author: UserSummary {
                login: "u".into(),
                id: 1,
            },
            committer: UserSummary {
                login: "u".into(),
                id: 1,
            },
            additions: 0,
            deletions: 0,
            files_changed: 0,
        };
        let debug = format!("{commit:?}");
        assert!(debug.contains("CommitData"));
        assert!(debug.contains("abc"));
    }

    // --- CommitDetail tests ---

    fn sample_commit_detail_json() -> &'static str {
        r#"{
            "sha": "deadbeef1234",
            "message": "Add unit tests for auth module",
            "author": {"login": "tester", "id": 99},
            "committer": {"login": "tester", "id": 99},
            "diff": "--- a/auth.rs\n+++ b/auth.rs\n@@ -1,3 +1,5 @@\n+use chrono::Utc;\n",
            "files": [
                {"filename": "auth.rs", "additions": 10, "deletions": 0, "status": "modified"}
            ]
        }"#
    }

    #[test]
    fn test_should_deserialize_commit_detail_from_camel_case_json() {
        let json = sample_commit_detail_json();
        let detail: CommitDetail = serde_json::from_str(json).expect("valid CommitDetail");
        assert_eq!(detail.sha, "deadbeef1234");
        assert_eq!(detail.message, "Add unit tests for auth module");
        assert!(detail.diff.is_some());
        assert_eq!(detail.files.len(), 1);
        assert_eq!(detail.files[0].filename, "auth.rs");
        assert_eq!(detail.files[0].additions, 10);
        assert_eq!(detail.files[0].deletions, 0);
        assert_eq!(detail.files[0].status, "modified");
    }

    #[test]
    fn test_should_serialize_commit_detail_to_camel_case_json() {
        let json = sample_commit_detail_json();
        let detail: CommitDetail = serde_json::from_str(json).expect("deserialize");
        let serialized = serde_json::to_string(&detail).expect("serialize");
        assert!(serialized.contains("\"files\""));
        assert!(serialized.contains("\"filename\""));
        assert!(!serialized.contains("\"files\":[{\"filename\":null"));
    }

    #[test]
    fn test_should_serialize_commit_detail_skips_null_diff() {
        let detail = CommitDetail {
            sha: "abc".into(),
            message: "no diff".into(),
            author: UserSummary {
                login: "u".into(),
                id: 1,
            },
            committer: UserSummary {
                login: "u".into(),
                id: 1,
            },
            diff: None,
            files: vec![],
        };
        let json = serde_json::to_string(&detail).expect("serialize");
        assert!(!json.contains("null"));
        assert!(!json.contains("\"diff\":null"));
    }

    #[test]
    fn test_should_roundtrip_commit_detail_via_serde() {
        let json = sample_commit_detail_json();
        let detail: CommitDetail = serde_json::from_str(json).expect("deserialize");
        let re_serialized = serde_json::to_string(&detail).expect("serialize");
        let round_tripped: CommitDetail =
            serde_json::from_str(&re_serialized).expect("re-deserialize");
        assert_eq!(round_tripped.sha, detail.sha);
        assert_eq!(round_tripped.message, detail.message);
        assert_eq!(round_tripped.diff, detail.diff);
        assert_eq!(round_tripped.files.len(), detail.files.len());
    }

    #[test]
    fn test_should_derive_debug_for_commit_detail() {
        let detail = CommitDetail {
            sha: "abc".into(),
            message: "test".into(),
            author: UserSummary {
                login: "u".into(),
                id: 1,
            },
            committer: UserSummary {
                login: "u".into(),
                id: 1,
            },
            diff: None,
            files: vec![],
        };
        let debug = format!("{detail:?}");
        assert!(debug.contains("CommitDetail"));
        assert!(debug.contains("abc"));
    }

    // --- CommitFile tests ---

    #[test]
    fn test_should_deserialize_commit_file_from_camel_case_json() {
        let json = r#"{"filename":"src/main.rs","additions":5,"deletions":2,"status":"modified"}"#;
        let file: CommitFile = serde_json::from_str(json).expect("valid CommitFile");
        assert_eq!(file.filename, "src/main.rs");
        assert_eq!(file.additions, 5);
        assert_eq!(file.deletions, 2);
        assert_eq!(file.status, "modified");
    }

    #[test]
    fn test_should_serialize_commit_file_to_camel_case_json() {
        let file = CommitFile {
            filename: "Cargo.toml".into(),
            additions: 1,
            deletions: 0,
            status: "modified".into(),
        };
        let json = serde_json::to_string(&file).expect("serialize CommitFile");
        assert!(json.contains("\"filename\":\"Cargo.toml\""));
        assert!(json.contains("\"additions\":1"));
        assert!(json.contains("\"deletions\":0"));
        assert!(json.contains("\"status\":\"modified\""));
    }

    #[test]
    fn test_should_roundtrip_commit_file_via_serde() {
        let file = CommitFile {
            filename: "README.md".into(),
            additions: 20,
            deletions: 5,
            status: "added".into(),
        };
        let json = serde_json::to_string(&file).expect("serialize");
        let round_tripped: CommitFile = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(round_tripped.filename, file.filename);
        assert_eq!(round_tripped.additions, file.additions);
        assert_eq!(round_tripped.deletions, file.deletions);
        assert_eq!(round_tripped.status, file.status);
    }

    #[test]
    fn test_should_derive_debug_for_commit_file() {
        let file = CommitFile {
            filename: "test.rs".into(),
            additions: 0,
            deletions: 0,
            status: "added".into(),
        };
        let debug = format!("{file:?}");
        assert!(debug.contains("CommitFile"));
        assert!(debug.contains("test.rs"));
    }
}
