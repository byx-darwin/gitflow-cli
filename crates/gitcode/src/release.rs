//! GitCode Release 提供者实现。
//!
//! 通过 `gitcode` CLI 实现 [`ReleaseProvider`] trait，支持 Release 的创建、列表、
//! 查看、编辑、资源上传/下载及删除。
//! 所有方法通过 `tokio::process::Command` 调用 `gc`，捕获 stdout 并解析 JSON。

use async_trait::async_trait;
use gitflow_core::{
    CoreError, Result, Session,
    release::{CreateReleaseArgs, ReleaseData, ReleaseProvider},
};
use tracing::debug;

use crate::{
    error::parse_gitcode_error,
    runner::{CommandRunner, RealCommandRunner},
};

/// `gc release` 请求的 JSON 字段列表。
const RELEASE_FIELDS: &str =
    "id,tagName,name,body,isDraft,isPrerelease,author,createdAt,publishedAt,url";

/// GitCode Release 提供者，通过 `gitcode` CLI 操作。
///
/// 该结构体通过调用 `gitcode` CLI 实现 [`ReleaseProvider`] trait 的所有方法，
/// 使上层命令能够以统一的方式操作 GitCode Release。
///
/// 命令执行通过 [`CommandRunner`] 抽象，生产环境默认使用
/// [`RealCommandRunner`]，测试可注入自定义 runner 以模拟成功或失败场景。
///
/// # Examples
///
/// ```no_run
/// use gitflow_gitcode::GitCodeReleaseProvider;
///
/// let provider = GitCodeReleaseProvider::new("octocat/hello-world");
/// ```
#[derive(Debug, Clone)]
pub struct GitCodeReleaseProvider<R: CommandRunner = RealCommandRunner> {
    /// GitCode `owner/repo`，如 `"byx-darwin/gitflow-cli"`。
    repo: String,
    /// 用于执行 `gitcode` CLI 命令的 runner。
    runner: R,
}

impl GitCodeReleaseProvider<RealCommandRunner> {
    /// 创建新的 GitCode Release 提供者。
    ///
    /// `repo` 格式为 `owner/repo`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> Self {
        Self {
            repo: repo.into(),
            runner: RealCommandRunner,
        }
    }

    /// Create a new provider from a shared [`Session`].
    ///
    /// This enables state reuse across multiple operations in workflow chains.
    #[must_use]
    pub fn with_session(session: &Session) -> Self {
        Self {
            repo: session.repo.clone(),
            runner: RealCommandRunner,
        }
    }
}

impl<R: CommandRunner> GitCodeReleaseProvider<R> {
    /// 使用自定义 [`CommandRunner`] 创建提供者。
    ///
    /// 主要用于测试，可注入模拟 runner 以控制 `gitcode` CLI 的输出。
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
impl<R: CommandRunner + 'static> ReleaseProvider for GitCodeReleaseProvider<R> {
    async fn create(&self, args: CreateReleaseArgs) -> Result<ReleaseData> {
        let binary = crate::gitcode_binary();
        let mut cmd_args: Vec<&str> = vec![
            "release",
            "create",
            &args.tag_name,
            "-R",
            &self.repo,
            "--json",
            RELEASE_FIELDS,
        ];

        if let Some(ref name) = args.name {
            cmd_args.push("--title");
            cmd_args.push(name);
        }

        if let Some(ref body) = args.body {
            cmd_args.push("--notes");
            cmd_args.push(body);
        }

        if args.draft {
            cmd_args.push("--draft");
        }

        if args.prerelease {
            cmd_args.push("--prerelease");
        }

        if let Some(ref commitish) = args.target_commitish {
            cmd_args.push("--target");
            cmd_args.push(commitish);
        }

        debug!(
            repo = %self.repo,
            tag = %args.tag_name,
            "spawning `gc release create`"
        );

        let output = self
            .runner
            .run(&binary, &cmd_args)
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            return Err(parse_gitcode_error(&output.stderr).into());
        }

        // If JSON parsing fails, the release was still created successfully
        // Try to fetch it via view
        match serde_json::from_slice::<ReleaseData>(&output.stdout) {
            Ok(release) => Ok(release),
            Err(_) => self.view(&args.tag_name).await,
        }
    }

    async fn list(&self) -> Result<Vec<ReleaseData>> {
        let binary = crate::gitcode_binary();
        debug!(repo = %self.repo, "spawning `gc release list`");

        let output = self
            .runner
            .run(
                &binary,
                &[
                    "release",
                    "list",
                    "-R",
                    &self.repo,
                    "--json",
                    RELEASE_FIELDS,
                ],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            return Err(parse_gitcode_error(&output.stderr).into());
        }

        let releases: Vec<ReleaseData> =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(releases)
    }

    async fn view(&self, tag_name: &str) -> Result<ReleaseData> {
        let binary = crate::gitcode_binary();
        debug!(repo = %self.repo, tag = %tag_name, "spawning `gc release view`");

        let output = self
            .runner
            .run(
                &binary,
                &["release", "view", tag_name, "-R", &self.repo, "--json"],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            return Err(parse_gitcode_error(&output.stderr).into());
        }

        let release: ReleaseData =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(release)
    }

    async fn edit(&self, tag_name: &str, args: CreateReleaseArgs) -> Result<ReleaseData> {
        let binary = crate::gitcode_binary();
        let mut cmd_args: Vec<&str> = vec![
            "release",
            "edit",
            tag_name,
            "-R",
            &self.repo,
            "--json",
            RELEASE_FIELDS,
        ];

        if let Some(ref name) = args.name {
            cmd_args.push("--title");
            cmd_args.push(name);
        }

        if let Some(ref body) = args.body {
            cmd_args.push("--notes");
            cmd_args.push(body);
        }

        if args.draft {
            cmd_args.push("--draft");
        }

        if args.prerelease {
            cmd_args.push("--prerelease");
        }

        debug!(
            repo = %self.repo,
            tag = %tag_name,
            "spawning `gc release edit`"
        );

        let output = self
            .runner
            .run(&binary, &cmd_args)
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            return Err(parse_gitcode_error(&output.stderr).into());
        }

        // If JSON parsing fails, try to fetch the edited release
        match serde_json::from_slice::<ReleaseData>(&output.stdout) {
            Ok(release) => Ok(release),
            Err(_) => self.view(tag_name).await,
        }
    }

    async fn delete(&self, tag_name: &str) -> Result<()> {
        let binary = crate::gitcode_binary();
        debug!(repo = %self.repo, tag = %tag_name, "spawning `gc release delete`");

        let output = self
            .runner
            .run(&binary, &["release", "delete", tag_name, "-R", &self.repo])
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            return Err(parse_gitcode_error(&output.stderr).into());
        }

        Ok(())
    }

    async fn upload_asset(&self, tag_name: &str, file_path: &str, _asset_name: &str) -> Result<()> {
        let binary = crate::gitcode_binary();
        debug!(
            repo = %self.repo,
            tag = %tag_name,
            file = %file_path,
            "spawning `gc release upload`"
        );

        let output = self
            .runner
            .run(
                &binary,
                &["release", "upload", tag_name, file_path, "-R", &self.repo],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            return Err(parse_gitcode_error(&output.stderr).into());
        }

        Ok(())
    }

    async fn download_asset(
        &self,
        tag_name: &str,
        asset_name: &str,
        output_path: &str,
    ) -> Result<()> {
        let binary = crate::gitcode_binary();
        debug!(
            repo = %self.repo,
            tag = %tag_name,
            asset = %asset_name,
            output = %output_path,
            "spawning `gc release download`"
        );

        let output = self
            .runner
            .run(
                &binary,
                &[
                    "release",
                    "download",
                    tag_name,
                    "-R",
                    &self.repo,
                    "--asset",
                    asset_name,
                    "--output",
                    output_path,
                ],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            return Err(parse_gitcode_error(&output.stderr).into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::MockCommandRunner;

    #[test]
    fn test_should_construct_gitcode_release_provider() {
        let provider = GitCodeReleaseProvider::new("octocat/hello-world");
        assert_eq!(provider.repo, "octocat/hello-world");
    }

    #[test]
    fn test_should_construct_gitcode_release_provider_from_string() {
        let repo = String::from("octocat/hello-world");
        let provider = GitCodeReleaseProvider::new(repo);
        assert_eq!(provider.repo, "octocat/hello-world");
    }

    #[test]
    fn test_should_debug_format_provider() {
        let provider = GitCodeReleaseProvider::new("octocat/hello-world");
        let debug = format!("{provider:?}");
        assert!(debug.contains("GitCodeReleaseProvider"));
        assert!(debug.contains("octocat/hello-world"));
    }

    #[test]
    fn test_should_clone_gitcode_release_provider() {
        let original = GitCodeReleaseProvider::new("owner/repo");
        let cloned = original.clone();
        assert_eq!(original.repo, cloned.repo);
    }

    // --- Failure-path tests using an injected MockCommandRunner ---

    fn sample_release_args() -> CreateReleaseArgs {
        CreateReleaseArgs {
            tag_name: "v1.0.0".to_string(),
            name: Some("Release 1.0.0".to_string()),
            body: Some("First stable release".to_string()),
            draft: false,
            prerelease: false,
            target_commitish: None,
        }
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_create() {
        let runner = MockCommandRunner::failure("tag already exists", 256);
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let result = provider.create(sample_release_args()).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_create() {
        // create parses ReleaseData; on failure it falls back to view, which
        // receives the same non-JSON stdout and fails to deserialize.
        let runner = MockCommandRunner::success("not valid json");
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let result = provider.create(sample_release_args()).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_list() {
        let runner = MockCommandRunner::failure("forbidden", 256);
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let result = provider.list().await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_list() {
        let runner = MockCommandRunner::success("invalid");
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let result = provider.list().await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_view() {
        let runner = MockCommandRunner::failure("release not found", 256);
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let result = provider.view("v1.0.0").await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_view() {
        let runner = MockCommandRunner::success("not valid json");
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let result = provider.view("v1.0.0").await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_edit() {
        let runner = MockCommandRunner::failure("release not found", 256);
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let result = provider.edit("v1.0.0", sample_release_args()).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_edit() {
        // edit parses ReleaseData; on failure it falls back to view, which
        // receives the same non-JSON stdout and fails to deserialize.
        let runner = MockCommandRunner::success("not valid json");
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let result = provider.edit("v1.0.0", sample_release_args()).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_delete() {
        let runner = MockCommandRunner::failure("release not found", 256);
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let result = provider.delete("v1.0.0").await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_upload_asset() {
        let runner = MockCommandRunner::failure("upload failed", 256);
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let result = provider
            .upload_asset("v1.0.0", "/tmp/artifact.tar.gz", "artifact.tar.gz")
            .await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_download_asset() {
        let runner = MockCommandRunner::failure("asset not found", 256);
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let result = provider
            .download_asset("v1.0.0", "artifact.tar.gz", "/tmp/out.tar.gz")
            .await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    // --- Deserialization tests ---

    #[test]
    fn test_should_deserialize_release_data_from_gitcode_output() {
        let json = br#"{
            "id": 100,
            "tagName": "v2.0.0",
            "name": "GitCode Release",
            "body": "Changelog here",
            "draft": false,
            "prerelease": false,
            "author": {"login": "dev", "id": "42"},
            "createdAt": "2026-02-01T00:00:00Z",
            "publishedAt": "2026-02-01T12:00:00Z",
            "url": "https://gitcode.com/owner/repo/releases/tag/v2.0.0"
        }"#;

        let release: ReleaseData = serde_json::from_slice(json).expect("valid ReleaseData JSON");
        assert_eq!(release.id, 100);
        assert_eq!(release.tag_name, "v2.0.0");
        assert_eq!(release.name.as_deref(), Some("GitCode Release"));
        assert_eq!(release.body.as_deref(), Some("Changelog here"));
        assert!(!release.draft);
        assert!(!release.prerelease);
        assert_eq!(release.author.as_ref().expect("author").login, "dev");
        assert_eq!(release.author.as_ref().expect("author").id, "42");
    }

    #[test]
    fn test_should_deserialize_empty_release_list_from_gitcode_output() {
        let json = b"[]";
        let releases: Vec<ReleaseData> = serde_json::from_slice(json).expect("valid empty list");
        assert!(releases.is_empty());
    }

    #[test]
    fn test_should_deserialize_draft_release_from_gitcode_output() {
        let json = br#"{
            "id": 7,
            "tagName": "v0.1.0-rc1",
            "name": null,
            "body": null,
            "draft": true,
            "prerelease": true,
            "author": {"login": "bot", "id": "0"},
            "createdAt": "2026-03-01T00:00:00Z",
            "publishedAt": null,
            "url": "https://gitcode.com/owner/repo/releases/tag/v0.1.0-rc1"
        }"#;

        let release: ReleaseData = serde_json::from_slice(json).expect("valid draft ReleaseData");
        assert!(release.draft);
        assert!(release.prerelease);
        assert!(release.name.is_none());
        assert!(release.body.is_none());
        assert!(release.published_at.is_none());
    }

    // --- with_runner constructor ---

    #[test]
    fn test_should_create_provider_with_custom_runner() {
        let runner = MockCommandRunner::success("");
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);
        assert_eq!(provider.repo, "owner/repo");
    }

    // --- Success-path tests ---

    fn valid_release_json() -> &'static str {
        r#"{
            "id": 1,
            "tagName": "v1.0.0",
            "name": "Release 1.0.0",
            "body": "First stable release",
            "draft": false,
            "prerelease": false,
            "author": {"login": "dev", "id": "1"},
            "createdAt": "2026-01-01T00:00:00Z",
            "publishedAt": "2026-01-01T00:00:00Z",
            "url": "https://gitcode.com/owner/repo/releases/tag/v1.0.0"
        }"#
    }

    #[tokio::test]
    async fn test_should_view_release_successfully() {
        let runner = MockCommandRunner::success(valid_release_json());
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let release = provider.view("v1.0.0").await.expect("view should succeed");
        assert_eq!(release.tag_name, "v1.0.0");
        assert_eq!(release.name.as_deref(), Some("Release 1.0.0"));
        assert_eq!(release.id, 1);
    }

    #[tokio::test]
    async fn test_should_list_releases_successfully() {
        let json = format!("[{}]", valid_release_json());
        let runner = MockCommandRunner::success(&json);
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let releases = provider.list().await.expect("list should succeed");
        assert_eq!(releases.len(), 1);
        assert_eq!(releases[0].tag_name, "v1.0.0");
    }

    #[tokio::test]
    async fn test_should_create_release_successfully() {
        let runner = MockCommandRunner::success(valid_release_json());
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let release = provider
            .create(sample_release_args())
            .await
            .expect("create should succeed");
        assert_eq!(release.tag_name, "v1.0.0");
    }

    #[tokio::test]
    async fn test_should_edit_release_successfully() {
        let runner = MockCommandRunner::success(valid_release_json());
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let release = provider
            .edit("v1.0.0", sample_release_args())
            .await
            .expect("edit should succeed");
        assert_eq!(release.tag_name, "v1.0.0");
    }

    #[tokio::test]
    async fn test_should_delete_release_successfully() {
        let runner = MockCommandRunner::success("");
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        assert!(provider.delete("v1.0.0").await.is_ok());
    }

    #[tokio::test]
    async fn test_should_upload_asset_successfully() {
        let runner = MockCommandRunner::success("");
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        assert!(
            provider
                .upload_asset("v1.0.0", "/tmp/artifact.tar.gz", "artifact.tar.gz")
                .await
                .is_ok()
        );
    }
}
