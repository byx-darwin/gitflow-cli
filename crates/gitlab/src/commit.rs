//! GitLab Commit 提供者实现。
//!
//! 通过 `glab api` CLI 命令实现 [`CommitProvider`] trait，
//! 支持 Commit 查看、Diff/Patch 获取及评论功能。
//!
//! GitLab API 路径与 GitHub 不同：
//! - `projects/:id/repository/commits/:sha` — 查看提交
//! - `projects/:id/repository/commits/:sha/diff` — 获取 diff
//! - `projects/:id/repository/commits/:sha` + Accept header — 获取 patch
//! - `projects/:id/repository/commits/:sha/comments` — 评论

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use gitflow_cli_core::{
    CoreError, Result,
    commit::{CommitDetail, CommitProvider},
    types::UserSummary,
};
use serde::Deserialize;
use tracing::debug;

use crate::error::parse_glab_error;

/// GitLab Commit 提供者，通过 `glab api` 查看提交信息。
///
/// # Examples
///
/// ```no_run
/// use gitflow_cli_gitlab::GitLabCommitProvider;
///
/// let provider = GitLabCommitProvider::new("gitlab-org/gitlab");
/// ```
#[derive(Debug, Clone)]
pub struct GitLabCommitProvider {
    /// GitLab `namespace/project`。
    repo: String,
}

impl GitLabCommitProvider {
    /// 创建新的 GitLab Commit 提供者。
    ///
    /// `repo` 格式为 `namespace/project`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> Self {
        Self { repo: repo.into() }
    }
}

// ── 中间 API 响应类型 ──────────────────────────────────────────────

/// GitLab API 返回的提交结构。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(
    dead_code,
    reason = "Used for deserialization; not all fields are read"
)]
struct CommitApiResponse {
    id: String,
    #[serde(default)]
    short_id: String,
    title: String,
    #[serde(default)]
    message: String,
    #[serde(default)]
    author_name: String,
    #[serde(default)]
    author_email: String,
    #[serde(default)]
    authored_date: Option<String>,
    #[serde(default)]
    committer_name: String,
    #[serde(default)]
    committer_email: String,
    #[serde(default)]
    committed_date: Option<String>,
    #[serde(default)]
    stats: Option<CommitStats>,
    #[serde(default)]
    parent_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(
    dead_code,
    reason = "Used for deserialization; not all fields are read"
)]
struct CommitStats {
    #[serde(default)]
    additions: u64,
    #[serde(default)]
    deletions: u64,
    #[serde(default)]
    total: u64,
}

/// GitLab diff 条目。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(
    dead_code,
    reason = "Used for deserialization; not all fields are read"
)]
struct DiffResponse {
    #[serde(default)]
    old_path: String,
    #[serde(default)]
    new_path: String,
    #[serde(default)]
    diff: String,
    #[serde(default)]
    new_file: bool,
    #[serde(default)]
    renamed_file: bool,
    #[serde(default)]
    deleted_file: bool,
}

/// GitLab commit comment 结构。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(
    dead_code,
    reason = "Used for deserialization; not all fields are read"
)]
struct CommentApiResponse {
    #[serde(default)]
    note: String,
    #[serde(default)]
    author: Option<ApiUser>,
    #[serde(default)]
    created_at: Option<DateTime<Utc>>,
}

/// GitLab API 返回的用户信息。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
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

/// 将 GitLab API 响应转换为 `CommitDetail`。
impl From<CommitApiResponse> for CommitDetail {
    fn from(api: CommitApiResponse) -> Self {
        let author = UserSummary {
            login: if api.author_name.is_empty() {
                "unknown".into()
            } else {
                api.author_name.clone()
            },
            id: "0".to_string(),
        };
        let committer = UserSummary {
            login: if api.committer_name.is_empty() {
                "unknown".into()
            } else {
                api.committer_name.clone()
            },
            id: "0".to_string(),
        };

        Self {
            sha: api.id,
            message: api.message,
            author,
            committer,
            diff: None,
            files: vec![],
        }
    }
}

// ── 辅助函数 ──────────────────────────────────────────────────────

/// 将 `namespace/project` 转换为 URL 编码的项目路径。
///
/// GitLab API 需要 URL 编码的项目路径（如 `namespace%2Fproject`）。
fn encode_project_path(repo: &str) -> String {
    repo.replace('/', "%2F")
}

/// 从 diff 条目推断文件状态。
#[allow(
    dead_code,
    reason = "Used in tests; reserved for future diff formatting"
)]
fn diff_status(diff: &DiffResponse) -> String {
    if diff.new_file {
        "added".into()
    } else if diff.deleted_file {
        "removed".into()
    } else if diff.renamed_file {
        "renamed".into()
    } else {
        "modified".into()
    }
}

// ── trait 实现 ──────────────────────────────────────────────────────

#[async_trait]
impl CommitProvider for GitLabCommitProvider {
    async fn view(&self, sha: &str) -> Result<CommitDetail> {
        debug!(repo = %self.repo, sha, "spawning `glab api commit view`");

        let encoded_path = encode_project_path(&self.repo);
        let api_path = format!("projects/{encoded_path}/repository/commits/{sha}");

        let output = tokio::process::Command::new("glab")
            .args(["api", &api_path])
            .output()
            .await
            .map_err(|e| {
                CoreError::Platform(format!("Failed to spawn glab api commit view: {e}"))
            })?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        let api_response: CommitApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_response.into())
    }

    async fn diff(&self, sha: &str) -> Result<String> {
        debug!(repo = %self.repo, sha, "spawning `glab api commit diff`");

        let encoded_path = encode_project_path(&self.repo);
        let api_path = format!("projects/{encoded_path}/repository/commits/{sha}/diff");

        let output = tokio::process::Command::new("glab")
            .args(["api", &api_path])
            .output()
            .await
            .map_err(|e| {
                CoreError::Platform(format!("Failed to spawn glab api commit diff: {e}"))
            })?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        // GitLab returns a JSON array of diff objects
        let diffs: Vec<DiffResponse> =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        let mut result = String::new();
        for diff in &diffs {
            use std::fmt::Write as _;
            let _ = write!(
                result,
                "diff --git a/{} b/{}\n{}\n",
                diff.old_path, diff.new_path, diff.diff
            );
        }

        Ok(result)
    }

    async fn patch(&self, sha: &str) -> Result<String> {
        debug!(repo = %self.repo, sha, "spawning `glab api commit patch`");

        let encoded_path = encode_project_path(&self.repo);
        let api_path = format!("projects/{encoded_path}/repository/commits/{sha}.patch");

        let output = tokio::process::Command::new("glab")
            .args(["api", &api_path])
            .output()
            .await
            .map_err(|e| {
                CoreError::Platform(format!("Failed to spawn glab api commit patch: {e}"))
            })?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    async fn comment(&self, sha: &str, body: &str, path: &str, line: u64) -> Result<()> {
        debug!(repo = %self.repo, sha, "spawning `glab api commit comment`");

        let encoded_path = encode_project_path(&self.repo);
        let api_path = format!("projects/{encoded_path}/repository/commits/{sha}/comments");

        let full_note = format!("{body}\n\nFile: {path}, Line: {line}");

        let output = tokio::process::Command::new("glab")
            .args(["api", &api_path, "-X", "POST"])
            .arg("-f")
            .arg(format!("note={full_note}"))
            .output()
            .await
            .map_err(|e| {
                CoreError::Platform(format!("Failed to spawn glab api commit comment: {e}"))
            })?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_construct_gitlab_commit_provider() {
        let provider = GitLabCommitProvider::new("gitlab-org/gitlab");
        assert_eq!(provider.repo, "gitlab-org/gitlab");
    }

    #[test]
    fn test_should_construct_gitlab_commit_provider_from_string() {
        let repo = String::from("gitlab-org/gitlab");
        let provider = GitLabCommitProvider::new(repo);
        assert_eq!(provider.repo, "gitlab-org/gitlab");
    }

    #[test]
    fn test_should_debug_format_commit_provider() {
        let provider = GitLabCommitProvider::new("owner/repo");
        let debug = format!("{provider:?}");
        assert!(debug.contains("GitLabCommitProvider"));
        assert!(debug.contains("owner/repo"));
    }

    #[test]
    fn test_should_clone_gitlab_commit_provider() {
        let original = GitLabCommitProvider::new("owner/repo");
        let cloned = original.clone();
        assert_eq!(original.repo, cloned.repo);
    }

    // --- API response deserialization tests ---

    #[test]
    fn test_should_deserialize_commit_api_response() {
        let json = br#"{
            "id": "abc123def456",
            "short_id": "abc123d",
            "title": "Fix login bug",
            "message": "Fix login bug\n\nMore details here.",
            "author_name": "alice",
            "author_email": "alice@example.com",
            "authored_date": "2026-01-15T09:30:00Z",
            "committer_name": "bob",
            "committer_email": "bob@example.com",
            "committed_date": "2026-01-15T09:30:00Z",
            "parent_ids": ["parent1"]
        }"#;

        let api: CommitApiResponse = serde_json::from_slice(json).expect("valid CommitApiResponse");
        assert_eq!(api.id, "abc123def456");
        assert!(api.message.contains("Fix login bug"));
        assert_eq!(api.author_name, "alice");
        assert_eq!(api.committer_name, "bob");
    }

    #[test]
    fn test_should_convert_commit_api_to_detail() {
        let api = CommitApiResponse {
            id: "sha123".into(),
            short_id: "sha123".into(),
            title: "Test commit".into(),
            message: "Test commit".into(),
            author_name: "dev".into(),
            author_email: "dev@example.com".into(),
            authored_date: Some("2026-01-01T00:00:00Z".into()),
            committer_name: "dev".into(),
            committer_email: "dev@example.com".into(),
            committed_date: Some("2026-01-01T00:00:00Z".into()),
            stats: None,
            parent_ids: vec![],
        };

        let detail: CommitDetail = api.into();
        assert_eq!(detail.sha, "sha123");
        assert_eq!(detail.message, "Test commit");
        assert_eq!(detail.author.login, "dev");
        assert!(detail.diff.is_none());
        assert!(detail.files.is_empty());
    }

    #[test]
    fn test_should_handle_empty_author_name() {
        let api = CommitApiResponse {
            id: "sha456".into(),
            short_id: "sha456".into(),
            title: "Test".into(),
            message: "Test".into(),
            author_name: String::new(),
            author_email: String::new(),
            authored_date: None,
            committer_name: String::new(),
            committer_email: String::new(),
            committed_date: None,
            stats: None,
            parent_ids: vec![],
        };

        let detail: CommitDetail = api.into();
        assert_eq!(detail.author.login, "unknown");
        assert_eq!(detail.committer.login, "unknown");
    }

    #[test]
    fn test_should_deserialize_diff_response() {
        let json = br#"{
            "old_path": "src/main.rs",
            "new_path": "src/main.rs",
            "diff": "@@ -1,3 +1,4 @@\n-old line\n+new line",
            "new_file": false,
            "renamed_file": false,
            "deleted_file": false
        }"#;

        let diff: DiffResponse = serde_json::from_slice(json).expect("valid DiffResponse");
        assert_eq!(diff.old_path, "src/main.rs");
        assert_eq!(diff.new_path, "src/main.rs");
        assert!(!diff.new_file);
        assert!(!diff.deleted_file);
    }

    #[test]
    fn test_should_determine_diff_status() {
        let new_file = DiffResponse {
            old_path: String::new(),
            new_path: "new.rs".into(),
            diff: String::new(),
            new_file: true,
            renamed_file: false,
            deleted_file: false,
        };
        assert_eq!(diff_status(&new_file), "added");

        let deleted = DiffResponse {
            old_path: "old.rs".into(),
            new_path: String::new(),
            diff: String::new(),
            new_file: false,
            renamed_file: false,
            deleted_file: true,
        };
        assert_eq!(diff_status(&deleted), "removed");

        let modified = DiffResponse {
            old_path: "main.rs".into(),
            new_path: "main.rs".into(),
            diff: String::new(),
            new_file: false,
            renamed_file: false,
            deleted_file: false,
        };
        assert_eq!(diff_status(&modified), "modified");
    }

    #[test]
    fn test_should_encode_project_path() {
        assert_eq!(
            encode_project_path("gitlab-org/gitlab"),
            "gitlab-org%2Fgitlab"
        );
        assert_eq!(
            encode_project_path("group/subgroup/project"),
            "group%2Fsubgroup%2Fproject"
        );
    }
}
