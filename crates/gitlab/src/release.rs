//! GitLab Release 提供者实现。
//!
//! 通过 `glab release` CLI 实现 [`ReleaseProvider`] trait，支持 Release 的创建、
//! 列表、查看、编辑、资源上传/下载及删除。
//! 所有方法通过 `tokio::process::Command` 调用 `glab`，捕获 stdout 并解析 JSON。
//!
//! `glab` 的 `JSON` 输出使用 `snake_case` 字段名（如 `tag_name`、`created_at`），
//! 因此使用中间类型 [`ReleaseApiResponse`] 进行反序列化，然后转换为核心类型
//! [`ReleaseData`]。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use gitflow_cli_core::{
    CoreError, Result,
    release::{CreateReleaseArgs, ReleaseData, ReleaseProvider},
    types::UserSummary,
};
use serde::Deserialize;
use tracing::debug;

use crate::error::parse_glab_error;

/// GitLab Release 提供者，通过 `glab` CLI 操作。
///
/// # Examples
///
/// ```no_run
/// use gitflow_cli_gitlab::GitLabReleaseProvider;
///
/// let provider = GitLabReleaseProvider::new("gitlab-org/gitlab");
/// ```
#[derive(Debug, Clone)]
pub struct GitLabReleaseProvider {
    /// GitLab `namespace/project`。
    repo: String,
}

impl GitLabReleaseProvider {
    /// 创建新的 GitLab Release 提供者。
    ///
    /// `repo` 格式为 `namespace/project`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> Self {
        Self { repo: repo.into() }
    }
}

// ── 中间 API 响应类型 ──────────────────────────────────────────────

/// `glab release` JSON 输出中的用户信息。
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

/// `glab release --output json` 返回的 JSON 结构。
#[derive(Debug, Clone, Deserialize)]
struct ReleaseApiResponse {
    #[serde(default)]
    tag_name: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    author: Option<ApiUser>,
    #[serde(default)]
    created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    released_at: Option<DateTime<Utc>>,
    #[serde(default)]
    url: Option<String>,
    /// GitLab uses `upcoming_release` instead of a numeric id in some versions.
    #[serde(default)]
    id: Option<u64>,
}

impl From<ReleaseApiResponse> for ReleaseData {
    fn from(api: ReleaseApiResponse) -> Self {
        let now = Utc::now();
        let author = api.author.as_ref().map(UserSummary::from);

        Self {
            id: api.id.unwrap_or(0),
            tag_name: api.tag_name,
            name: api.name,
            body: api.description,
            draft: api.draft,
            prerelease: api.prerelease,
            author,
            created_at: api.created_at.unwrap_or(now),
            published_at: api.released_at,
            url: api.url.unwrap_or_default(),
        }
    }
}

// ── trait 实现 ──────────────────────────────────────────────────────

#[async_trait]
impl ReleaseProvider for GitLabReleaseProvider {
    async fn create(&self, args: CreateReleaseArgs) -> Result<ReleaseData> {
        let mut cmd = tokio::process::Command::new("glab");
        cmd.args(["release", "create"])
            .arg(&args.tag_name)
            .arg("--repo")
            .arg(&self.repo)
            .arg("--output")
            .arg("json");

        if let Some(ref name) = args.name {
            cmd.arg("--name").arg(name);
        }

        if let Some(ref body) = args.body {
            cmd.arg("--notes").arg(body);
        }

        if args.draft {
            cmd.arg("--draft");
        }

        if args.prerelease {
            cmd.arg("--prerelease");
        }

        if let Some(ref commitish) = args.target_commitish {
            cmd.arg("--ref").arg(commitish);
        }

        debug!(
            repo = %self.repo,
            tag = %args.tag_name,
            "spawning `glab release create`"
        );

        let output = cmd
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        let api_response: ReleaseApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_response.into())
    }

    async fn list(&self) -> Result<Vec<ReleaseData>> {
        debug!(repo = %self.repo, "spawning `glab release list`");

        let output = tokio::process::Command::new("glab")
            .args(["release", "list"])
            .arg("--repo")
            .arg(&self.repo)
            .arg("--output")
            .arg("json")
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        let api_responses: Vec<ReleaseApiResponse> =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_responses.into_iter().map(ReleaseData::from).collect())
    }

    async fn view(&self, tag_name: &str) -> Result<ReleaseData> {
        debug!(repo = %self.repo, tag = %tag_name, "spawning `glab release view`");

        let output = tokio::process::Command::new("glab")
            .args(["release", "view"])
            .arg(tag_name)
            .arg("--repo")
            .arg(&self.repo)
            .arg("--output")
            .arg("json")
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        let api_response: ReleaseApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_response.into())
    }

    async fn edit(&self, tag_name: &str, args: CreateReleaseArgs) -> Result<ReleaseData> {
        let mut cmd = tokio::process::Command::new("glab");
        cmd.args(["release", "edit"])
            .arg(tag_name)
            .arg("--repo")
            .arg(&self.repo)
            .arg("--output")
            .arg("json");

        if let Some(ref name) = args.name {
            cmd.arg("--name").arg(name);
        }

        if let Some(ref body) = args.body {
            cmd.arg("--notes").arg(body);
        }

        if args.draft {
            cmd.arg("--draft");
        }

        if args.prerelease {
            cmd.arg("--prerelease");
        }

        if let Some(ref commitish) = args.target_commitish {
            cmd.arg("--ref").arg(commitish);
        }

        debug!(
            repo = %self.repo,
            tag = %tag_name,
            "spawning `glab release edit`"
        );

        let output = cmd
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        let api_response: ReleaseApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_response.into())
    }

    async fn upload_asset(&self, tag_name: &str, file_path: &str, _asset_name: &str) -> Result<()> {
        debug!(
            repo = %self.repo,
            tag = %tag_name,
            file = %file_path,
            "spawning `glab release upload`"
        );

        let output = tokio::process::Command::new("glab")
            .args(["release", "upload"])
            .arg(tag_name)
            .arg(file_path)
            .arg("--repo")
            .arg(&self.repo)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        Ok(())
    }

    #[allow(
        clippy::disallowed_methods,
        reason = "Path parent extraction required for download"
    )]
    async fn download_asset(
        &self,
        tag_name: &str,
        asset_name: &str,
        dest_path: &str,
    ) -> Result<()> {
        debug!(
            repo = %self.repo,
            tag = %tag_name,
            asset = %asset_name,
            dest = %dest_path,
            "spawning `glab release download`"
        );

        let dest = std::path::PathBuf::from(dest_path);
        let parent = dest.parent().unwrap_or_else(|| std::path::Path::new("."));

        let output = tokio::process::Command::new("glab")
            .args(["release", "download"])
            .arg(tag_name)
            .arg("--repo")
            .arg(&self.repo)
            .arg("--pattern")
            .arg(asset_name)
            .arg("--dir")
            .arg(parent)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        let downloaded = parent.join(asset_name);
        if downloaded != dest && downloaded.exists() {
            std::fs::rename(&downloaded, &dest).map_err(|e| {
                CoreError::Platform(format!(
                    "Failed to move downloaded asset to {dest_path}: {e}"
                ))
            })?;
        }

        Ok(())
    }

    async fn delete(&self, tag_name: &str) -> Result<()> {
        debug!(repo = %self.repo, tag = %tag_name, "spawning `glab release delete`");

        let output = tokio::process::Command::new("glab")
            .args(["release", "delete"])
            .arg(tag_name)
            .arg("--repo")
            .arg(&self.repo)
            .arg("--yes")
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

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
    fn test_should_construct_gitlab_release_provider() {
        let provider = GitLabReleaseProvider::new("gitlab-org/gitlab");
        assert_eq!(provider.repo, "gitlab-org/gitlab");
    }

    #[test]
    fn test_should_construct_gitlab_release_provider_from_string() {
        let repo = String::from("gitlab-org/gitlab");
        let provider = GitLabReleaseProvider::new(repo);
        assert_eq!(provider.repo, "gitlab-org/gitlab");
    }

    #[test]
    fn test_should_deserialize_release_api_response() {
        let json = br#"{
            "id": 1001,
            "tag_name": "v1.0.0",
            "name": "Version 1.0.0",
            "description": "Initial stable release",
            "draft": false,
            "prerelease": false,
            "author": {"username": "admin", "id": 1},
            "created_at": "2026-01-01T00:00:00Z",
            "released_at": "2026-01-15T12:00:00Z",
            "url": "https://gitlab.com/gitlab-org/gitlab/-/releases/v1.0.0"
        }"#;

        let api: ReleaseApiResponse =
            serde_json::from_slice(json).expect("valid ReleaseApiResponse");
        let release: ReleaseData = api.into();

        assert_eq!(release.id, 1001);
        assert_eq!(release.tag_name, "v1.0.0");
        assert_eq!(release.name.as_deref(), Some("Version 1.0.0"));
        assert_eq!(release.body.as_deref(), Some("Initial stable release"));
        assert!(!release.draft);
        assert!(!release.prerelease);
        assert_eq!(release.author.as_ref().unwrap().login, "admin");
        assert_eq!(
            release.url,
            "https://gitlab.com/gitlab-org/gitlab/-/releases/v1.0.0"
        );
    }

    #[test]
    fn test_should_deserialize_draft_release() {
        let json = br#"{
            "tag_name": "v0.1.0-draft",
            "name": null,
            "description": null,
            "draft": true,
            "prerelease": true,
            "author": {"username": "dev", "id": 99},
            "created_at": "2026-03-01T00:00:00Z",
            "released_at": null,
            "url": null
        }"#;

        let api: ReleaseApiResponse =
            serde_json::from_slice(json).expect("valid ReleaseApiResponse");
        let release: ReleaseData = api.into();
        assert!(release.draft);
        assert!(release.prerelease);
        assert!(release.name.is_none());
        assert!(release.published_at.is_none());
    }

    #[test]
    fn test_should_deserialize_empty_release_list() {
        let json = b"[]";
        let list: Vec<ReleaseApiResponse> = serde_json::from_slice(json).expect("valid empty list");
        assert!(list.is_empty());
    }

    #[test]
    fn test_should_debug_format_provider() {
        let provider = GitLabReleaseProvider::new("gitlab-org/gitlab");
        let debug = format!("{provider:?}");
        assert!(debug.contains("GitLabReleaseProvider"));
        assert!(debug.contains("gitlab-org/gitlab"));
    }

    #[test]
    fn test_should_clone_gitlab_release_provider() {
        let original = GitLabReleaseProvider::new("owner/repo");
        let cloned = original.clone();
        assert_eq!(original.repo, cloned.repo);
    }
}
