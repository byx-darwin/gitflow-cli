//! GitHub Release 提供者实现。
//!
//! 通过 `gh` CLI 实现 [`ReleaseProvider`] trait，支持 Release 的创建、列表、
//! 查看、编辑、资源上传/下载及删除。
//! 命令执行通过 [`CommandRunner`] 抽象，捕获 stdout 并解析 JSON。

use async_trait::async_trait;
use gitflow_cli_core::{
    CoreError, Result,
    release::{CreateReleaseArgs, ReleaseData, ReleaseProvider},
};
use tracing::debug;

use crate::{
    error::parse_gh_error,
    runner::{CommandRunner, RealCommandRunner},
};

/// `gh release list` 请求的 JSON 字段列表。
const RELEASE_LIST_FIELDS: &str = "tagName,name,isDraft,isPrerelease,createdAt,publishedAt";

/// `gh release view` 请求的 JSON 字段列表。
const RELEASE_VIEW_FIELDS: &str =
    "databaseId,tagName,name,body,isDraft,isPrerelease,author,createdAt,publishedAt,url";

/// GitHub Release 提供者，通过 `gh` CLI 操作。
///
/// 该结构体通过调用 `gh` CLI 实现 [`ReleaseProvider`] trait 的所有方法，
/// 使上层命令能够以统一的方式操作 GitHub Release。
///
/// 命令执行通过 [`CommandRunner`] 抽象，生产环境默认使用
/// [`RealCommandRunner`]，测试可注入自定义 runner 以模拟成功或失败场景。
///
/// # Examples
///
/// ```no_run
/// use gitflow_cli_github::GitHubReleaseProvider;
///
/// let provider = GitHubReleaseProvider::new("octocat/hello-world");
/// ```
#[derive(Debug, Clone)]
pub struct GitHubReleaseProvider<R: CommandRunner = RealCommandRunner> {
    /// GitHub `owner/repo`，如 `"byx-darwin/gitflow-cli"`。
    repo: String,
    /// 用于执行 `gh` CLI 命令的 runner。
    runner: R,
}

impl GitHubReleaseProvider<RealCommandRunner> {
    /// 创建新的 GitHub Release 提供者，使用真实的进程执行器。
    ///
    /// `repo` 格式为 `owner/repo`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> Self {
        Self {
            repo: repo.into(),
            runner: RealCommandRunner,
        }
    }
}

impl<R: CommandRunner> GitHubReleaseProvider<R> {
    /// 使用自定义 [`CommandRunner`] 创建提供者。
    ///
    /// 主要用于测试，可注入模拟 runner 以控制 `gh` CLI 的输出。
    /// `repo` 格式为 `owner/repo`。
    #[must_use]
    pub fn with_runner(repo: impl Into<String>, runner: R) -> Self {
        Self {
            repo: repo.into(),
            runner,
        }
    }
}

#[async_trait]
impl<R: CommandRunner + 'static> ReleaseProvider for GitHubReleaseProvider<R> {
    async fn create(&self, args: CreateReleaseArgs) -> Result<ReleaseData> {
        let mut cmd_args: Vec<&str> =
            vec!["release", "create", &args.tag_name, "--repo", &self.repo];

        if let Some(name) = &args.name {
            cmd_args.push("--title");
            cmd_args.push(name);
        }

        if let Some(body) = &args.body {
            cmd_args.push("--notes");
            cmd_args.push(body);
        }

        if args.draft {
            cmd_args.push("--draft");
        }

        if args.prerelease {
            cmd_args.push("--prerelease");
        }

        if let Some(commitish) = &args.target_commitish {
            cmd_args.push("--target");
            cmd_args.push(commitish);
        }

        debug!(
            repo = %self.repo,
            tag = %args.tag_name,
            "spawning `gh release create`"
        );

        let output = self
            .runner
            .run("gh", &cmd_args)
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        // gh release create doesn't support --json, so we fetch the created release
        self.view(&args.tag_name).await
    }

    async fn list(&self) -> Result<Vec<ReleaseData>> {
        debug!(repo = %self.repo, "spawning `gh release list`");

        let output = self
            .runner
            .run(
                "gh",
                &[
                    "release",
                    "list",
                    "--repo",
                    &self.repo,
                    "--json",
                    RELEASE_LIST_FIELDS,
                ],
            )
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

        let output = self
            .runner
            .run(
                "gh",
                &[
                    "release",
                    "view",
                    tag_name,
                    "--repo",
                    &self.repo,
                    "--json",
                    RELEASE_VIEW_FIELDS,
                ],
            )
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
        let mut cmd_args: Vec<&str> = vec!["release", "edit", tag_name, "--repo", &self.repo];

        if let Some(name) = &args.name {
            cmd_args.push("--title");
            cmd_args.push(name);
        }

        if let Some(body) = &args.body {
            cmd_args.push("--notes");
            cmd_args.push(body);
        }

        if args.draft {
            cmd_args.push("--draft");
        }

        if args.prerelease {
            cmd_args.push("--prerelease");
        }

        if let Some(commitish) = &args.target_commitish {
            cmd_args.push("--target");
            cmd_args.push(commitish);
        }

        debug!(
            repo = %self.repo,
            tag = %tag_name,
            "spawning `gh release edit`"
        );

        let output = self
            .runner
            .run("gh", &cmd_args)
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        // gh release edit doesn't support --json, so we fetch the edited release
        self.view(tag_name).await
    }

    async fn upload_asset(&self, tag_name: &str, file_path: &str, _asset_name: &str) -> Result<()> {
        debug!(
            repo = %self.repo,
            tag = %tag_name,
            file = %file_path,
            "spawning `gh release upload`"
        );

        let output = self
            .runner
            .run(
                "gh",
                &[
                    "release", "upload", tag_name, file_path, "--repo", &self.repo,
                ],
            )
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
        let parent_str = parent.to_str().ok_or_else(|| {
            CoreError::Platform(format!(
                "Destination directory is not valid UTF-8: {dest_path}"
            ))
        })?;

        let output = self
            .runner
            .run(
                "gh",
                &[
                    "release",
                    "download",
                    tag_name,
                    "--repo",
                    &self.repo,
                    "--pattern",
                    asset_name,
                    "--dir",
                    parent_str,
                ],
            )
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

        let output = self
            .runner
            .run(
                "gh",
                &["release", "delete", tag_name, "--repo", &self.repo, "--yes"],
            )
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
    use crate::runner::MockCommandRunner;

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
        assert_eq!(release.author.as_ref().unwrap().login, "octocat");
        assert_eq!(release.author.as_ref().unwrap().id, "1");
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

    // --- Failure-path tests using an injected MockCommandRunner ---

    fn sample_create_args() -> CreateReleaseArgs {
        CreateReleaseArgs {
            tag_name: "v1.0.0".to_string(),
            name: Some("Version 1.0.0".to_string()),
            body: Some("Release notes".to_string()),
            draft: false,
            prerelease: false,
            target_commitish: None,
        }
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_fails_for_create() {
        let runner = MockCommandRunner::failure(r#"{"message": "Validation failed"}"#, 256);
        let provider = GitHubReleaseProvider::with_runner("owner/repo", runner);

        let result = provider.create(sample_create_args()).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_create() {
        // `create` succeeds, then re-fetches via `view`, whose invalid JSON fails parsing.
        let runner = MockCommandRunner::success("invalid json");
        let provider = GitHubReleaseProvider::with_runner("owner/repo", runner);

        let result = provider.create(sample_create_args()).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_fails_for_list() {
        let runner = MockCommandRunner::failure(r#"{"message": "Forbidden"}"#, 256);
        let provider = GitHubReleaseProvider::with_runner("owner/repo", runner);

        let result = provider.list().await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_list() {
        let runner = MockCommandRunner::success("invalid json");
        let provider = GitHubReleaseProvider::with_runner("owner/repo", runner);

        let result = provider.list().await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_fails_for_view() {
        let runner = MockCommandRunner::failure(r#"{"message": "Release not found"}"#, 256);
        let provider = GitHubReleaseProvider::with_runner("owner/repo", runner);

        let result = provider.view("v1.0.0").await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_view() {
        let runner = MockCommandRunner::success("invalid json");
        let provider = GitHubReleaseProvider::with_runner("owner/repo", runner);

        let result = provider.view("v1.0.0").await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_fails_for_edit() {
        let runner = MockCommandRunner::failure(r#"{"message": "Release not found"}"#, 256);
        let provider = GitHubReleaseProvider::with_runner("owner/repo", runner);

        let result = provider.edit("v1.0.0", sample_create_args()).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_edit() {
        // `edit` succeeds, then re-fetches via `view`, whose invalid JSON fails parsing.
        let runner = MockCommandRunner::success("invalid json");
        let provider = GitHubReleaseProvider::with_runner("owner/repo", runner);

        let result = provider.edit("v1.0.0", sample_create_args()).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_fails_for_delete() {
        let runner = MockCommandRunner::failure(r#"{"message": "Release not found"}"#, 256);
        let provider = GitHubReleaseProvider::with_runner("owner/repo", runner);

        let result = provider.delete("v1.0.0").await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_fails_for_upload_asset() {
        let runner = MockCommandRunner::failure(r#"{"message": "Upload failed"}"#, 256);
        let provider = GitHubReleaseProvider::with_runner("owner/repo", runner);

        let result = provider
            .upload_asset("v1.0.0", "/tmp/asset.tar.gz", "asset.tar.gz")
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_fails_for_download_asset() {
        let runner = MockCommandRunner::failure(r#"{"message": "Asset not found"}"#, 256);
        let provider = GitHubReleaseProvider::with_runner("owner/repo", runner);

        let result = provider
            .download_asset("v1.0.0", "asset.tar.gz", "/tmp/download/asset.tar.gz")
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }
}
