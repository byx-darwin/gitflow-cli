//! GitCode Release 提供者实现。
//!
//! 通过 `gitcode` CLI 实现 [`ReleaseProvider`] trait，支持 Release 的创建、列表、
//! 查看、编辑、资源上传/下载及删除。
//! 所有方法通过 `tokio::process::Command` 调用 `gc`，捕获 stdout 并解析 JSON。

use async_trait::async_trait;
use gitflow_cli_core::{
    CoreError, Result,
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
/// use gitflow_cli_gitcode::GitCodeReleaseProvider;
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
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
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
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
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
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
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
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
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
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
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
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
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
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
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
            gitflow_cli_core::CoreError::Platform(_)
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
            gitflow_cli_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_list() {
        let runner = MockCommandRunner::failure("forbidden", 256);
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let result = provider.list().await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_list() {
        let runner = MockCommandRunner::success("invalid");
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let result = provider.list().await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_view() {
        let runner = MockCommandRunner::failure("release not found", 256);
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let result = provider.view("v1.0.0").await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_view() {
        let runner = MockCommandRunner::success("not valid json");
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let result = provider.view("v1.0.0").await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_edit() {
        let runner = MockCommandRunner::failure("release not found", 256);
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let result = provider.edit("v1.0.0", sample_release_args()).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
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
            gitflow_cli_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_delete() {
        let runner = MockCommandRunner::failure("release not found", 256);
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let result = provider.delete("v1.0.0").await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
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
            gitflow_cli_core::CoreError::Platform(_)
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
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }
}
