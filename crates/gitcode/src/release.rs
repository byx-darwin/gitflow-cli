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

use crate::error::parse_gitcode_error;

/// `gc release` 请求的 JSON 字段列表。
const RELEASE_FIELDS: &str =
    "id,tagName,name,body,isDraft,isPrerelease,author,createdAt,publishedAt,url";

/// GitCode Release 提供者，通过 `gitcode` CLI 操作。
///
/// 该结构体通过调用 `gitcode` CLI 实现 [`ReleaseProvider`] trait 的所有方法，
/// 使上层命令能够以统一的方式操作 GitCode Release。
///
/// # Examples
///
/// ```no_run
/// use gitflow_cli_gitcode::GitCodeReleaseProvider;
///
/// let provider = GitCodeReleaseProvider::new("octocat/hello-world");
/// ```
#[derive(Debug, Clone)]
pub struct GitCodeReleaseProvider {
    /// GitCode `owner/repo`，如 `"byx-darwin/gitflow-cli"`。
    repo: String,
}

impl GitCodeReleaseProvider {
    /// 创建新的 GitCode Release 提供者。
    ///
    /// `repo` 格式为 `owner/repo`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> Self {
        Self { repo: repo.into() }
    }
}

#[async_trait]
impl ReleaseProvider for GitCodeReleaseProvider {
    async fn create(&self, args: CreateReleaseArgs) -> Result<ReleaseData> {
        let mut cmd = tokio::process::Command::new(crate::gitcode_binary());
        cmd.args(["release", "create"])
            .arg(&args.tag_name)
            .arg("-R")
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
            "spawning `gc release create`"
        );

        let output = cmd
            .output()
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
        debug!(repo = %self.repo, "spawning `gc release list`");

        let output = tokio::process::Command::new(crate::gitcode_binary())
            .args(["release", "list"])
            .arg("-R")
            .arg(&self.repo)
            .arg("--json")
            .arg(RELEASE_FIELDS)
            .output()
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
        debug!(repo = %self.repo, tag = %tag_name, "spawning `gc release view`");

        let output = tokio::process::Command::new(crate::gitcode_binary())
            .args(["release", "view"])
            .arg(tag_name)
            .arg("-R")
            .arg(&self.repo)
            .arg("--json")
            .output()
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
        let mut cmd = tokio::process::Command::new(crate::gitcode_binary());
        cmd.args(["release", "edit"])
            .arg(tag_name)
            .arg("-R")
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

        debug!(
            repo = %self.repo,
            tag = %tag_name,
            "spawning `gc release edit`"
        );

        let output = cmd
            .output()
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
        debug!(repo = %self.repo, tag = %tag_name, "spawning `gc release delete`");

        let output = tokio::process::Command::new(crate::gitcode_binary())
            .args(["release", "delete"])
            .arg(tag_name)
            .arg("-R")
            .arg(&self.repo)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
        }

        Ok(())
    }

    async fn upload_asset(&self, tag_name: &str, file_path: &str, _asset_name: &str) -> Result<()> {
        debug!(
            repo = %self.repo,
            tag = %tag_name,
            file = %file_path,
            "spawning `gc release upload`"
        );

        let output = tokio::process::Command::new(crate::gitcode_binary())
            .args(["release", "upload"])
            .arg(tag_name)
            .arg(file_path)
            .arg("-R")
            .arg(&self.repo)
            .output()
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
        debug!(
            repo = %self.repo,
            tag = %tag_name,
            asset = %asset_name,
            output = %output_path,
            "spawning `gc release download`"
        );

        let output = tokio::process::Command::new(crate::gitcode_binary())
            .args(["release", "download"])
            .arg(tag_name)
            .arg("-R")
            .arg(&self.repo)
            .arg("--asset")
            .arg(asset_name)
            .arg("--output")
            .arg(output_path)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
        }

        Ok(())
    }
}
