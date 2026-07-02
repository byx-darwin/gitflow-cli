//! GitHub Release 提供者实现。
//!
//! 通过 `gh` CLI 实现 [`ReleaseProvider`] trait，支持 Release 的创建、列表、
//! 查看、编辑、资源上传/下载及删除。
//! 所有方法通过 `tokio::process::Command` 调用 `gh`，捕获 stdout 并解析 JSON。

use async_trait::async_trait;
use gitflow_cli_core::{
    CoreError, Result,
    release::{CreateReleaseArgs, ReleaseData, ReleaseProvider},
};
use tracing::debug;

use crate::error::parse_gh_error;

/// `gh release` 请求的 JSON 字段列表。
const RELEASE_FIELDS: &str =
    "id,tagName,name,body,draft,prerelease,author,createdAt,publishedAt,url";

/// GitHub Release 提供者，通过 `gh` CLI 操作。
///
/// 该结构体通过调用 `gh` CLI 实现 [`ReleaseProvider`] trait 的所有方法，
/// 使上层命令能够以统一的方式操作 GitHub Release。
///
/// # Examples
///
/// ```no_run
/// use gitflow_cli_github::GitHubReleaseProvider;
///
/// let provider = GitHubReleaseProvider::new("octocat/hello-world");
/// ```
#[derive(Debug, Clone)]
pub struct GitHubReleaseProvider {
    /// GitHub `owner/repo`，如 `"byx-darwin/gitflow-cli"`。
    repo: String,
}

impl GitHubReleaseProvider {
    /// 创建新的 GitHub Release 提供者。
    ///
    /// `repo` 格式为 `owner/repo`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> Self {
        Self { repo: repo.into() }
    }
}

#[async_trait]
impl ReleaseProvider for GitHubReleaseProvider {
    async fn create(&self, args: CreateReleaseArgs) -> Result<ReleaseData> {
        let mut cmd = tokio::process::Command::new("gh");
        cmd.args(["release", "create"])
            .arg(&args.tag_name)
            .arg("--repo")
            .arg(&self.repo)
            .arg("--json")
            .arg(RELEASE_FIELDS);

        if let Some(ref name) = args.name {
            cmd.arg("--title").arg(name);
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
            cmd.arg("--target").arg(commitish);
        }

        debug!(
            repo = %self.repo,
            tag = %args.tag_name,
            "spawning `gh release create`"
        );

        let output = cmd
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        let release: ReleaseData =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(release)
    }

    async fn list(&self) -> Result<Vec<ReleaseData>> {
        debug!(repo = %self.repo, "spawning `gh release list`");

        let output = tokio::process::Command::new("gh")
            .args(["release", "list"])
            .arg("--repo")
            .arg(&self.repo)
            .arg("--json")
            .arg(RELEASE_FIELDS)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        let releases: Vec<ReleaseData> =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(releases)
    }

    async fn view(&self, tag_name: &str) -> Result<ReleaseData> {
        debug!(repo = %self.repo, tag = %tag_name, "spawning `gh release view`");

        let output = tokio::process::Command::new("gh")
            .args(["release", "view"])
            .arg(tag_name)
            .arg("--repo")
            .arg(&self.repo)
            .arg("--json")
            .arg(RELEASE_FIELDS)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        let release: ReleaseData =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(release)
    }

    async fn edit(&self, tag_name: &str, args: CreateReleaseArgs) -> Result<ReleaseData> {
        let mut cmd = tokio::process::Command::new("gh");
        cmd.args(["release", "edit"])
            .arg(tag_name)
            .arg("--repo")
            .arg(&self.repo)
            .arg("--json")
            .arg(RELEASE_FIELDS);

        if let Some(ref name) = args.name {
            cmd.arg("--title").arg(name);
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
            cmd.arg("--target").arg(commitish);
        }

        debug!(
            repo = %self.repo,
            tag = %tag_name,
            "spawning `gh release edit`"
        );

        let output = cmd
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        let release: ReleaseData =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(release)
    }

    async fn upload_asset(&self, tag_name: &str, file_path: &str, _asset_name: &str) -> Result<()> {
        debug!(
            repo = %self.repo,
            tag = %tag_name,
            file = %file_path,
            "spawning `gh release upload`"
        );

        let output = tokio::process::Command::new("gh")
            .args(["release", "upload"])
            .arg(tag_name)
            .arg(file_path)
            .arg("--repo")
            .arg(&self.repo)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        Ok(())
    }

    #[allow(clippy::disallowed_methods)]
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
            "spawning `gh release download`"
        );

        // 下载到目标文件的父目录，然后重命名到目标路径
        let dest = std::path::PathBuf::from(dest_path);
        let parent = dest.parent().unwrap_or_else(|| std::path::Path::new("."));

        let output = tokio::process::Command::new("gh")
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
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        // gh 使用资源的实际文件名下载，尝试找到并移动
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
        debug!(repo = %self.repo, tag = %tag_name, "spawning `gh release delete`");

        let output = tokio::process::Command::new("gh")
            .args(["release", "delete"])
            .arg(tag_name)
            .arg("--repo")
            .arg(&self.repo)
            .arg("--yes")
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_construct_github_release_provider() {
        let provider = GitHubReleaseProvider::new("octocat/hello-world");
        assert_eq!(provider.repo, "octocat/hello-world");
    }

    #[test]
    fn test_should_construct_github_release_provider_from_string() {
        let repo = String::from("octocat/hello-world");
        let provider = GitHubReleaseProvider::new(repo);
        assert_eq!(provider.repo, "octocat/hello-world");
    }

    #[test]
    fn test_should_deserialize_release_data_from_gh_output() {
        // 模拟 `gh release view --json ...` 的实际输出
        let gh_json = br#"{
            "id": 1001,
            "tagName": "v1.0.0",
            "name": "Version 1.0.0",
            "body": "Initial stable release",
            "draft": false,
            "prerelease": false,
            "author": {"login": "octocat", "id": "1"},
            "createdAt": "2026-01-01T00:00:00Z",
            "publishedAt": "2026-01-15T12:00:00Z",
            "url": "https://github.com/octocat/hello-world/releases/tag/v1.0.0"
        }"#;

        let release: ReleaseData = serde_json::from_slice(gh_json).expect("valid ReleaseData JSON");
        assert_eq!(release.id, 1001);
        assert_eq!(release.tag_name, "v1.0.0");
        assert_eq!(release.name.as_deref(), Some("Version 1.0.0"));
        assert_eq!(release.body.as_deref(), Some("Initial stable release"));
        assert!(!release.draft);
        assert!(!release.prerelease);
        assert_eq!(release.author.login, "octocat");
        assert_eq!(release.author.id, "1");
        assert_eq!(
            release.url,
            "https://github.com/octocat/hello-world/releases/tag/v1.0.0"
        );
    }

    #[test]
    fn test_should_deserialize_empty_release_list_from_gh_output() {
        let gh_json = b"[]";
        let releases: Vec<ReleaseData> =
            serde_json::from_slice(gh_json).expect("valid ReleaseData list");
        assert!(releases.is_empty());
    }

    #[test]
    fn test_should_deserialize_draft_release_from_gh_output() {
        let gh_json = br#"{
            "id": 5,
            "tagName": "v0.1.0-draft",
            "name": null,
            "body": null,
            "draft": true,
            "prerelease": true,
            "author": {"login": "dev", "id": "99"},
            "createdAt": "2026-03-01T00:00:00Z",
            "publishedAt": null,
            "url": "https://example.com/releases/5"
        }"#;

        let release: ReleaseData =
            serde_json::from_slice(gh_json).expect("valid draft ReleaseData");
        assert!(release.draft);
        assert!(release.prerelease);
        assert!(release.name.is_none());
        assert!(release.body.is_none());
        assert!(release.published_at.is_none());
    }

    #[test]
    fn test_should_debug_format_provider() {
        let provider = GitHubReleaseProvider::new("octocat/hello-world");
        let debug = format!("{provider:?}");
        assert!(debug.contains("GitHubReleaseProvider"));
        assert!(debug.contains("octocat/hello-world"));
    }

    #[test]
    fn test_should_create_provider_with_different_repos() {
        let r1 = GitHubReleaseProvider::new("org/repo-a");
        let r2 = GitHubReleaseProvider::new("org/repo-b");
        assert_eq!(r1.repo, "org/repo-a");
        assert_eq!(r2.repo, "org/repo-b");
    }

    #[test]
    fn test_should_clone_github_release_provider() {
        let original = GitHubReleaseProvider::new("owner/repo");
        let cloned = original.clone();
        assert_eq!(original.repo, cloned.repo);
    }
}
