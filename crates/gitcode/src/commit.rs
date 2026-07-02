//! GitCode Commit 提供者实现。
//!
//! 通过 `gc api` CLI 命令实现 [`CommitProvider`] trait，
//! 支持 Commit 查看、Diff/Patch 获取及评论功能。

use async_trait::async_trait;
use gitflow_cli_core::{
    CoreError, Result,
    commit::{CommitDetail, CommitFile, CommitProvider},
    types::UserSummary,
};
use serde::Deserialize;
use tracing::debug;

use crate::error::parse_gc_error;

/// GitCode Commit 提供者，通过 `gc api` 查看提交信息。
///
/// # Examples
///
/// ```no_run
/// use gitflow_cli_gitcode::GitCodeCommitProvider;
///
/// let provider = GitCodeCommitProvider::new("octocat/hello-world");
/// ```
#[derive(Debug, Clone)]
pub struct GitCodeCommitProvider {
    /// GitCode `owner/repo`。
    repo: String,
}

impl GitCodeCommitProvider {
    /// 创建新的 GitCode Commit 提供者。
    ///
    /// `repo` 格式为 `owner/repo`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> Self {
        Self { repo: repo.into() }
    }
}

/// `gc api repos/<repo>/commits/<sha>` 返回的 JSON 结构。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(
    dead_code,
    reason = "Fields are deserialized but not all used directly"
)]
struct CommitApiResponse {
    sha: String,
    commit: CommitInner,
    author: Option<ApiUser>,
    committer: Option<ApiUser>,
    #[serde(default)]
    stats: Option<CommitStats>,
    #[serde(default)]
    files: Vec<CommitFileResponse>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(
    dead_code,
    reason = "Fields are deserialized but not all used directly"
)]
struct CommitInner {
    message: String,
    author: Option<CommitAuthorInner>,
    committer: Option<CommitAuthorInner>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(
    dead_code,
    reason = "Fields are deserialized but not all used directly"
)]
struct CommitAuthorInner {
    #[serde(default)]
    date: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(
    dead_code,
    reason = "Fields are deserialized but not all used directly"
)]
struct CommitStats {
    #[serde(default)]
    additions: u64,
    #[serde(default)]
    deletions: u64,
    #[serde(default)]
    total: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommitFileResponse {
    filename: String,
    #[serde(default)]
    additions: u64,
    #[serde(default)]
    deletions: u64,
    #[serde(default)]
    status: String,
}

/// GitCode API 返回的用户信息。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiUser {
    login: String,
    id: u64,
}

impl From<&ApiUser> for UserSummary {
    fn from(u: &ApiUser) -> Self {
        Self {
            login: u.login.clone(),
            id: u.id,
        }
    }
}

/// 将 API 响应转换为 `CommitDetail`。
impl From<CommitApiResponse> for CommitDetail {
    fn from(api: CommitApiResponse) -> Self {
        let author = api.author.as_ref().map_or_else(
            || fallback_user(api.commit.author.as_ref()),
            UserSummary::from,
        );
        let committer = api.committer.as_ref().map_or_else(
            || fallback_user(api.commit.committer.as_ref()),
            UserSummary::from,
        );

        let files: Vec<CommitFile> = api
            .files
            .into_iter()
            .map(|f| CommitFile {
                filename: f.filename,
                additions: f.additions,
                deletions: f.deletions,
                status: f.status,
            })
            .collect();

        Self {
            sha: api.sha,
            message: api.commit.message,
            author,
            committer,
            diff: None,
            files,
        }
    }
}

/// 为 `CommitDetail` 构造后备用户（当 API 未返回用户信息时）。
fn fallback_user(_inner: Option<&CommitAuthorInner>) -> UserSummary {
    UserSummary {
        login: "unknown".into(),
        id: 0,
    }
}

#[async_trait]
impl CommitProvider for GitCodeCommitProvider {
    async fn view(&self, sha: &str) -> Result<CommitDetail> {
        debug!(repo = %self.repo, sha, "spawning `gc api commit view`");

        let api_path = format!("repos/{repo}/commits/{sha}", repo = self.repo);

        let output = tokio::process::Command::new("gc")
            .args(["api", &api_path])
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gc api commit view: {e}")))?;

        if !output.status.success() {
            let gc_err = parse_gc_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gc_err}")));
        }

        let api_response: CommitApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_response.into())
    }

    async fn diff(&self, sha: &str) -> Result<String> {
        debug!(repo = %self.repo, sha, "spawning `gc api commit diff`");

        let api_path = format!("repos/{repo}/commits/{sha}", repo = self.repo);

        let output = tokio::process::Command::new("gc")
            .args(["api", &api_path])
            .arg("-H")
            .arg("Accept: application/vnd.gitcode.v3.diff")
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gc api commit diff: {e}")))?;

        if !output.status.success() {
            let gc_err = parse_gc_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gc_err}")));
        }

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    async fn patch(&self, sha: &str) -> Result<String> {
        debug!(repo = %self.repo, sha, "spawning `gc api commit patch`");

        let api_path = format!("repos/{repo}/commits/{sha}", repo = self.repo);

        let output = tokio::process::Command::new("gc")
            .args(["api", &api_path])
            .arg("-H")
            .arg("Accept: application/vnd.gitcode.v3.patch")
            .output()
            .await
            .map_err(|e| {
                CoreError::Platform(format!("Failed to spawn gc api commit patch: {e}"))
            })?;

        if !output.status.success() {
            let gc_err = parse_gc_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gc_err}")));
        }

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    async fn comment(&self, sha: &str, body: &str, path: &str, line: u64) -> Result<()> {
        debug!(repo = %self.repo, sha, "spawning `gc api commit comment`");

        let api_path = format!("repos/{repo}/commits/{sha}/comments", repo = self.repo);

        let mut cmd = tokio::process::Command::new("gc");
        cmd.args(["api", &api_path, "-X", "POST"])
            .arg("-f")
            .arg(format!("body={body}"))
            .arg("-f")
            .arg(format!("path={path}"))
            .arg("-f")
            .arg(format!("line={line}"))
            .arg("-f")
            .arg("position=1");

        let output = cmd.output().await.map_err(|e| {
            CoreError::Platform(format!("Failed to spawn gc api commit comment: {e}"))
        })?;

        if !output.status.success() {
            let gc_err = parse_gc_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gc_err}")));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- GitCodeCommitProvider tests ---

    #[test]
    fn test_should_construct_gitcode_commit_provider() {
        let provider = GitCodeCommitProvider::new("octocat/hello-world");
        assert_eq!(provider.repo, "octocat/hello-world");
    }

    #[test]
    fn test_should_construct_gitcode_commit_provider_from_string() {
        let repo = String::from("octocat/hello-world");
        let provider = GitCodeCommitProvider::new(repo);
        assert_eq!(provider.repo, "octocat/hello-world");
    }

    #[test]
    fn test_should_debug_format_commit_provider() {
        let provider = GitCodeCommitProvider::new("owner/repo");
        let debug = format!("{provider:?}");
        assert!(debug.contains("GitCodeCommitProvider"));
        assert!(debug.contains("owner/repo"));
    }

    #[test]
    fn test_should_clone_gitcode_commit_provider() {
        let original = GitCodeCommitProvider::new("owner/repo");
        let cloned = original.clone();
        assert_eq!(original.repo, cloned.repo);
    }

    // --- API response deserialization tests ---

    #[test]
    fn test_should_deserialize_commit_api_response() {
        let json = br#"{
            "sha": "abc123def456",
            "commit": {
                "message": "Fix login bug\n\nMore details here.",
                "author": {"date": "2026-01-15T09:30:00Z"},
                "committer": {"date": "2026-01-15T09:30:00Z"}
            },
            "author": {"login": "alice", "id": 7},
            "committer": {"login": "bob", "id": 12},
            "stats": {"additions": 15, "deletions": 3, "total": 18},
            "files": [
                {"filename": "auth.rs", "additions": 10, "deletions": 0, "status": "modified"},
                {"filename": "tests/auth.rs", "additions": 5, "deletions": 3, "status": "modified"}
            ]
        }"#;

        let api: CommitApiResponse = serde_json::from_slice(json).expect("valid CommitApiResponse");
        assert_eq!(api.sha, "abc123def456");
        assert!(api.commit.message.contains("Fix login bug"));
        assert_eq!(api.author.as_ref().map(|u| &*u.login), Some("alice"));
        assert_eq!(api.files.len(), 2);
        assert_eq!(api.files[0].filename, "auth.rs");
        assert_eq!(api.stats.as_ref().map(|s| s.additions), Some(15));
    }

    #[test]
    fn test_should_deserialize_commit_detail_from_api() {
        let json = br#"{
            "sha": "deadbeef",
            "commit": {
                "message": "Add feature",
                "author": null,
                "committer": null
            },
            "author": null,
            "committer": null,
            "stats": null,
            "files": []
        }"#;

        let api: CommitApiResponse = serde_json::from_slice(json).expect("valid CommitApiResponse");
        let detail: CommitDetail = api.into();

        assert_eq!(detail.sha, "deadbeef");
        assert_eq!(detail.message, "Add feature");
        assert_eq!(detail.author.login, "unknown");
        assert!(detail.diff.is_none());
        assert!(detail.files.is_empty());
    }

    #[test]
    fn test_should_deserialize_commit_file_response() {
        let json = br#"{"filename":"src/main.rs","additions":5,"deletions":2,"status":"modified"}"#;

        let file: CommitFileResponse =
            serde_json::from_slice(json).expect("valid CommitFileResponse");
        assert_eq!(file.filename, "src/main.rs");
        assert_eq!(file.additions, 5);
        assert_eq!(file.deletions, 2);
        assert_eq!(file.status, "modified");
    }

    #[test]
    fn test_should_deserialize_api_user() {
        let json = br#"{"login":"octocat","id":583231}"#;
        let user: ApiUser = serde_json::from_slice(json).expect("valid ApiUser");
        assert_eq!(user.login, "octocat");
        assert_eq!(user.id, 583_231);
    }

    #[test]
    fn test_should_convert_api_user_to_user_summary() {
        let user = ApiUser {
            login: "alice".into(),
            id: 42,
        };
        let summary: UserSummary = (&user).into();
        assert_eq!(summary.login, "alice");
        assert_eq!(summary.id, 42);
    }

    #[test]
    fn test_should_convert_commit_api_to_detail() {
        let api = CommitApiResponse {
            sha: "sha123".into(),
            commit: CommitInner {
                message: "Test commit".into(),
                author: None,
                committer: None,
            },
            author: Some(ApiUser {
                login: "dev".into(),
                id: 1,
            }),
            committer: Some(ApiUser {
                login: "dev".into(),
                id: 1,
            }),
            stats: Some(CommitStats {
                additions: 10,
                deletions: 2,
                total: 12,
            }),
            files: vec![CommitFileResponse {
                filename: "lib.rs".into(),
                additions: 10,
                deletions: 2,
                status: "modified".into(),
            }],
        };

        let detail: CommitDetail = api.into();
        assert_eq!(detail.sha, "sha123");
        assert_eq!(detail.message, "Test commit");
        assert_eq!(detail.author.login, "dev");
        assert_eq!(detail.files.len(), 1);
        assert_eq!(detail.files[0].filename, "lib.rs");
    }
}
