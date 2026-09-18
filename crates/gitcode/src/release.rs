//! GitCode Release 提供者实现。
//!
//! 通过 `gitcode` CLI 实现 [`ReleaseProvider`] trait，支持 Release 的创建、列表、
//! 查看、编辑、资源上传/下载及删除。
//! 所有方法通过 `tokio::process::Command` 调用 `gc`，捕获 stdout 并解析 JSON。
//!
//! 例外：`list` 走 `gitcode api`，其响应字段名是 snake_case（`tag_name` / `html_url`），
//! 与 [`ReleaseData`] 的 camelCase 线上命名不兼容，因此先反序列化为中间类型
//! [`ReleaseApiResponse`] 再转换为 core 类型——与本 crate 的 `IssueApiResponse`
//! 及 `gitflow-gitlab` 的同名类型同构。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use gitflow_core::{
    CoreError, DEFAULT_LIST_LIMIT, FetchStrategy, Paged, Result, Session, fetch_capped,
    release::{CreateReleaseArgs, ReleaseData, ReleaseProvider},
    types::{UserSummary, deserialize_u64_or_string_to_string},
};
use serde::Deserialize;
use tracing::debug;

use crate::{
    error::parse_gitcode_error,
    runner::{CommandRunner, RealCommandRunner},
};

/// `gc release` 请求的 JSON 字段列表。
const RELEASE_FIELDS: &str =
    "id,tagName,name,body,isDraft,isPrerelease,author,createdAt,publishedAt,url";

// ── 中间 API 响应类型 ──────────────────────────────────────────────

/// 反序列化「u64 / 字符串 / `null`」三种形态为 `Option<String>`。
///
/// 本地小助手，专用于本文件；不放进 `crates/core` 是因为它只多做一件
/// `gitflow_core::types::deserialize_u64_or_string_to_string` 没做的事——
/// 容忍 `null`（该函数没有 `visit_unit`，遇到 `null` 会硬报
/// `invalid type: null`）。Gitee 血统的 API 常用 `null` 表示「这个 id 此刻
/// 不存在」（例如作者账号已注销），而不是省略字段。
fn deserialize_u64_or_string_or_null_to_string<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de;

    struct U64OrStringOrNullToString;
    impl de::Visitor<'_> for U64OrStringOrNullToString {
        type Value = Option<String>;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("a u64 integer, a string, or null")
        }

        fn visit_u64<E: de::Error>(self, v: u64) -> std::result::Result<Option<String>, E> {
            Ok(Some(v.to_string()))
        }

        fn visit_str<E: de::Error>(self, v: &str) -> std::result::Result<Option<String>, E> {
            Ok(Some(v.to_string()))
        }

        fn visit_unit<E: de::Error>(self) -> std::result::Result<Option<String>, E> {
            Ok(None)
        }

        fn visit_none<E: de::Error>(self) -> std::result::Result<Option<String>, E> {
            Ok(None)
        }
    }

    deserializer.deserialize_any(U64OrStringOrNullToString)
}

/// `gitcode api` release 响应中的作者对象。
///
/// 独立于 [`UserSummary`] 的原因：gitcode api 的 `id` 可能是数字也可能是字符串，
/// 而 `UserSummary::id` 是 `String` 且未挂 `deserialize_u64_or_string_to_string`，
/// 直接反序列化数字 id 会硬报 `invalid type: integer`。`login` 为 `Option<String>`：
/// 账号已注销等场景下 gitcode 会把 `login` 置为 `null`（F1 第二轮修复）。
#[derive(Debug, Clone, Deserialize)]
struct ReleaseUserApi {
    #[serde(default)]
    login: Option<String>,
    #[serde(default, deserialize_with = "deserialize_u64_or_string_to_string")]
    id: String,
}

impl From<ReleaseUserApi> for UserSummary {
    fn from(u: ReleaseUserApi) -> Self {
        Self {
            login: u.login.unwrap_or_default(),
            id: u.id,
        }
    }
}

/// `gitcode api /repos/{owner}/{repo}/releases` 的响应结构。
///
/// 线上字段名为 snake_case，与 [`ReleaseData`] 的 camelCase
/// （`tagName` / `createdAt` / `publishedAt`）不同，故必须经由本类型转换。
///
/// # 容错边界（务必精确，不要泛化这句话——本节已两次因过度概括而失实）
///
/// `#[serde(default)]` 只在字段**缺失**时生效；字段存在但类型不对，serde
/// 依然会报错并使整条记录、进而整个 `releases` 数组反序列化失败。
///
/// ## 实际容忍 `null` 的字段
///
/// [`ReleaseApiResponse`] 的每个字段类型都是 `Option<T>`（`id` 的字段类型是
/// `Option<String>`，见下），[`ReleaseUserApi::login`] 也是 `Option<String>`——
/// 因此这些字段本身缺失、或值显式为 `null`，都会退化为默认值，不会让 `list`
/// 失败：`id`（退化为 `0`）、`tag_name`（退化为空字符串）、`name`、`body`、
/// `draft`（退化为 `false`）、`prerelease`（退化为 `false`）、`author`
/// （退化为 `None`）、`author.login`（退化为空字符串，F1 第二轮修复——
/// Gitee 血统 API 常用「账号已注销」→ `login: null` 这一形状）、
/// `created_at`、`published_at`、`html_url`、`url`。
///
/// `id` 额外用本文件的 [`deserialize_u64_or_string_or_null_to_string`]（而非
/// `gitflow_core::types::deserialize_u64_or_string_to_string`，后者没有
/// `visit_unit`、遇 `null` 仍会报错）容忍数字、字符串、`null` 三种形态，
/// 与 [`ReleaseUserApi::id`] 的字符串/数字容忍同源，都是因为 gitcode 的 id
/// 观测到过两种线上编码。
///
/// ## 仍然不容忍、会使整个 `list` 失败的情况
///
/// - **字段存在但类型错误**（例如 `"tag_name": 5`、`"author": "dev"`）： `Option<T>`
///   的默认/自定义反序列化只特殊处理 `null`，其余类型不匹配 仍会报 `invalid type`。
/// - **`author.id` 为 `null`**：`ReleaseUserApi::id` 用的是
///   `gitflow_core::types::deserialize_u64_or_string_to_string`（核心库，
///   本次修复未改动），该函数没有 `visit_unit`，`null` 会报 `invalid type: null, expected a u64
///   integer or string`。本次只扩展了 `release.id`（顶层）与 `author.login` 的 `null`
///   容忍，`author.id` 未 纳入范围。
/// - **`created_at` / `published_at` 存在且不是合法 RFC3339**（例如 `"2026-01-01
///   00:00:00"`）：`null` 或字段缺失没问题，但格式错误的字符串 仍会被 chrono
///   拒绝并让整条记录失败——这不属于本次修复范围。
#[derive(Debug, Clone, Deserialize)]
struct ReleaseApiResponse {
    #[serde(
        default,
        deserialize_with = "deserialize_u64_or_string_or_null_to_string"
    )]
    id: Option<String>,
    #[serde(default)]
    tag_name: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    draft: Option<bool>,
    #[serde(default)]
    prerelease: Option<bool>,
    #[serde(default)]
    author: Option<ReleaseUserApi>,
    #[serde(default)]
    created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    published_at: Option<DateTime<Utc>>,
    /// 可浏览的网页地址。gitcode 沿用 GitHub 的切分：`url` 是 API self-link，
    /// `html_url` 才是网页地址。`release view`（CLI 路径）返回的是网页地址，
    /// 故此处优先取 `html_url`，避免同一字段在 `list` 与 `view` 下含义不同。
    #[serde(default)]
    html_url: Option<String>,
    #[serde(default)]
    url: Option<String>,
}

impl From<ReleaseApiResponse> for ReleaseData {
    fn from(api: ReleaseApiResponse) -> Self {
        Self {
            id: api.id.and_then(|s| s.parse().ok()).unwrap_or(0),
            tag_name: api.tag_name.unwrap_or_default(),
            name: api.name,
            body: api.body,
            draft: api.draft.unwrap_or_default(),
            prerelease: api.prerelease.unwrap_or_default(),
            author: api.author.map(UserSummary::from),
            created_at: api.created_at.unwrap_or_else(Utc::now),
            published_at: api.published_at,
            url: api.html_url.or(api.url).unwrap_or_default(),
        }
    }
}

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

    /// 列出 Release，按页抓取至多 `limit` 条。
    ///
    /// 走 `gitcode api` 而非 `release list` 子命令：实测（gitcode-cli 0.12.0）
    /// `release list` **没有任何分页旗标**（只有 `-L/--limit`），而其 API 层的
    /// `per_page` 被静默封顶在 100，因此 CLI 路径无法诚实报告截断。api 路径与
    /// 本 crate 的 `issue comments`（`issue.rs`）同构。
    ///
    /// api 响应是 snake_case，经 [`ReleaseApiResponse`] 转换为 [`ReleaseData`]。
    ///
    /// `create` / `view` 等其余方法仍走 CLI 子命令，不受影响。
    ///
    /// # Errors
    ///
    /// 当 `gitcode` CLI 调用失败或响应无法反序列化时返回错误。
    async fn list(&self, limit: Option<u32>) -> Result<Paged<ReleaseData>> {
        let binary = crate::gitcode_binary();
        let binary = &binary;
        let cap = limit.unwrap_or(DEFAULT_LIST_LIMIT);
        let repo = &self.repo;
        let runner = &self.runner;
        let per_page = cap.saturating_add(1).min(crate::GITCODE_API_MAX_PER_PAGE);

        debug!(repo = %self.repo, cap, per_page, "spawning `gitcode api` GET releases");

        fetch_capped(
            FetchStrategy::Paged { per_page },
            cap,
            |page, per_page| async move {
                let api_path = format!("/repos/{repo}/releases?per_page={per_page}&page={page}");

                let output = runner.run(binary, &["api", &api_path]).await.map_err(|e| {
                    CoreError::Platform(format!("Failed to spawn gitcode api: {e}"))
                })?;

                if !output.status.success() {
                    return Err(parse_gitcode_error(&output.stderr).into());
                }

                let releases: Vec<ReleaseApiResponse> =
                    serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;
                Ok(releases.into_iter().map(ReleaseData::from).collect())
            },
        )
        .await
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
    use crate::runner::{MockCommandRunner, SequencedMockCommandRunner};

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

        let result = provider.list(None).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_list() {
        let runner = MockCommandRunner::success("invalid");
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let result = provider.list(None).await;

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

    /// `gitcode api /repos/{owner}/{repo}/releases` 的真实响应形状：snake_case
    /// 字段名、数字型 author id、`url` 与 `html_url` 并存。
    ///
    /// 这是 C1 的回归护栏——用 [`ReleaseData`] 直接反序列化这份 payload 会得到
    /// `missing field \`tagName\``。
    fn valid_release_api_json() -> &'static str {
        r#"{
            "id": 1,
            "tag_name": "v1.0.0",
            "name": "Release 1.0.0",
            "body": "First stable release",
            "draft": false,
            "prerelease": false,
            "author": {"login": "dev", "id": 1},
            "created_at": "2026-01-01T00:00:00Z",
            "published_at": "2026-01-01T00:00:00Z",
            "html_url": "https://gitcode.com/owner/repo/releases/tag/v1.0.0",
            "url": "https://api.gitcode.com/repos/owner/repo/releases/1"
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
        let json = format!("[{}]", valid_release_api_json());
        let runner = MockCommandRunner::success(&json);
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let paged = provider.list(None).await.expect("list should succeed");
        assert_eq!(paged.items.len(), 1);
        assert_eq!(paged.items[0].tag_name, "v1.0.0");
    }

    #[tokio::test]
    async fn test_should_fetch_gitcode_releases_via_api_with_pagination() {
        // cap=100 → per_page=100，want=101；首页满 100 条 ⇒ 必然发出第二页。
        // cap 必须 ≥ 100，否则 per_page 恰为 cap+1，循环只发一次调用，
        // 断言 calls[1] 会直接索引越界。
        let one = valid_release_api_json();
        let page = format!("[{}]", vec![one; 100].join(","));
        let runner = SequencedMockCommandRunner::from_results(&[(true, &page), (true, &page)]);
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner.clone());

        let paged = provider.list(Some(100)).await.expect("list should succeed");

        assert_eq!(paged.items.len(), 100);
        assert!(paged.truncated);

        let calls = runner.recorded_calls();
        assert_eq!(
            calls.len(),
            2,
            "必须真的翻到第二页，实际调用数: {}",
            calls.len()
        );
        // 整串相等，而非 `contains`：`"per_page=1001".contains("per_page=100")`
        // 与 `"per_page=100".contains("page=1")` 都为真，子串断言检测不出钳位失效。
        assert_eq!(
            calls[0].1,
            vec!["api", "/repos/owner/repo/releases?per_page=100&page=1"]
        );
        assert_eq!(
            calls[1].1,
            vec!["api", "/repos/owner/repo/releases?per_page=100&page=2"]
        );
    }

    #[tokio::test]
    async fn test_should_deserialize_release_from_gitcode_api_response() {
        // 钉住 api 响应（snake_case）→ ReleaseData 的字段映射（本机无带 release 的
        // 公开 gitcode 仓库可验，故以 fixture 覆盖）。
        let runner = MockCommandRunner::success(&format!("[{}]", valid_release_api_json()));
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let paged = provider.list(None).await.expect("list should succeed");

        assert_eq!(paged.items.len(), 1);
        let release = &paged.items[0];
        assert_eq!(release.id, 1);
        assert_eq!(release.tag_name, "v1.0.0");
        assert_eq!(release.name.as_deref(), Some("Release 1.0.0"));
        assert_eq!(release.body.as_deref(), Some("First stable release"));
        assert!(!release.draft);
        assert!(!release.prerelease);
        // 数字型 author id 必须被接受并转成字符串。
        let author = release.author.as_ref().expect("author");
        assert_eq!(author.login, "dev");
        assert_eq!(author.id, "1");
        assert_eq!(release.created_at.to_rfc3339(), "2026-01-01T00:00:00+00:00");
        assert!(release.published_at.is_some());
        // html_url 优先于 API self-link。
        assert_eq!(
            release.url,
            "https://gitcode.com/owner/repo/releases/tag/v1.0.0"
        );
    }

    #[test]
    fn test_should_reject_snake_case_api_payload_when_using_core_release_data() {
        // C1 的成因说明：ReleaseData 是 camelCase 线上命名，直接吃 api 响应会失败。
        // 这两条断言把「必须有中间类型」这一事实钉住，防止日后有人把它去掉。

        // 成因一：字段名不匹配。剥掉 author 以隔离出 tagName 这一项。
        let no_author = r#"{
            "tag_name": "v1.0.0",
            "draft": false,
            "prerelease": false,
            "created_at": "2026-01-01T00:00:00Z"
        }"#;
        let err = serde_json::from_str::<ReleaseData>(no_author)
            .expect_err("ReleaseData must not accept a snake_case api payload");
        assert!(err.to_string().contains("tagName"), "实际错误: {err}");

        // 成因二：UserSummary::id 是 String，api 的数字 id 会硬报类型错误。
        let err = serde_json::from_str::<ReleaseData>(valid_release_api_json())
            .expect_err("ReleaseData must not accept the full api payload either");
        assert!(
            err.to_string().contains("invalid type: integer"),
            "实际错误: {err}"
        );
    }

    #[tokio::test]
    async fn test_should_prefer_html_url_over_api_self_link_for_release_url() {
        let json = r#"[{
            "tag_name": "v2.0.0",
            "url": "https://api.gitcode.com/repos/owner/repo/releases/9",
            "html_url": "https://gitcode.com/owner/repo/releases/tag/v2.0.0"
        }]"#;
        let runner = MockCommandRunner::success(json);
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let paged = provider.list(None).await.expect("list should succeed");

        assert_eq!(
            paged.items[0].url,
            "https://gitcode.com/owner/repo/releases/tag/v2.0.0"
        );
    }

    #[tokio::test]
    async fn test_should_fall_back_to_url_when_api_omits_html_url() {
        let json = r#"[{
            "tag_name": "v2.0.0",
            "url": "https://gitcode.com/owner/repo/releases/tag/v2.0.0"
        }]"#;
        let runner = MockCommandRunner::success(json);
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let paged = provider.list(None).await.expect("list should succeed");

        assert_eq!(
            paged.items[0].url,
            "https://gitcode.com/owner/repo/releases/tag/v2.0.0"
        );
    }

    #[tokio::test]
    async fn test_should_degrade_predictably_for_minimal_api_release_object() {
        // 形状不匹配时必须整体退化而非半途失败：所有字段都有 default。
        let runner = MockCommandRunner::success(r#"[{"tag_name": "v0.0.1"}]"#);
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let paged = provider.list(None).await.expect("list should succeed");

        let release = &paged.items[0];
        assert_eq!(release.tag_name, "v0.0.1");
        assert_eq!(release.id, 0);
        assert!(!release.draft);
        assert!(!release.prerelease);
        assert!(release.author.is_none());
        assert!(release.published_at.is_none());
        assert!(release.url.is_empty());
    }

    #[tokio::test]
    async fn test_should_accept_string_author_id_from_api() {
        let json = r#"[{
            "tag_name": "v1.0.0",
            "author": {"login": "dev", "id": "42"}
        }]"#;
        let runner = MockCommandRunner::success(json);
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let paged = provider.list(None).await.expect("list should succeed");

        assert_eq!(paged.items[0].author.as_ref().expect("author").id, "42");
    }

    #[tokio::test]
    async fn test_should_accept_string_release_id_from_api() {
        // gitcode 的 release id 与 author id 同源，同样可能是字符串。F1: id
        // 之前是裸 u64，字符串形态会让整个 list 反序列化失败。
        let json = r#"[{
            "id": "12",
            "tag_name": "v1.0.0"
        }]"#;
        let runner = MockCommandRunner::success(json);
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let paged = provider.list(None).await.expect("list should succeed");

        assert_eq!(paged.items[0].id, 12);
    }

    #[tokio::test]
    async fn test_should_degrade_release_flags_when_null_in_api_response() {
        // Gitee 血统的 API 常用 null 表示「此概念在这里不存在」，而非省略字段。
        // F1: tag_name/draft/prerelease 之前是裸类型，null 会让整个 list 失败。
        let json = r#"[{
            "id": 1,
            "tag_name": null,
            "draft": null,
            "prerelease": null
        }]"#;
        let runner = MockCommandRunner::success(json);
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let paged = provider.list(None).await.expect("list should succeed");

        let release = &paged.items[0];
        assert_eq!(release.tag_name, "");
        assert!(!release.draft);
        assert!(!release.prerelease);
    }

    #[tokio::test]
    async fn test_should_degrade_release_id_to_zero_when_null_in_api_response() {
        // F1 第二轮：id 此前用 deserialize_u64_or_string_to_string，该函数没有
        // visit_unit，null 会硬报 `invalid type: null`。补上本地 null 容忍。
        let json = r#"[{
            "id": null,
            "tag_name": "v1.0.0"
        }]"#;
        let runner = MockCommandRunner::success(json);
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let paged = provider.list(None).await.expect("list should succeed");

        assert_eq!(paged.items[0].id, 0);
    }

    #[tokio::test]
    async fn test_should_degrade_author_login_to_empty_when_null_in_api_response() {
        // F1 第二轮：Gitee 血统 API 常用「作者已注销」→ login: null 这一形状，
        // ReleaseUserApi.login 此前是裸 String，null 会让整个 list 失败。
        let json = r#"[{
            "id": 1,
            "tag_name": "v1.0.0",
            "author": {"login": null, "id": 42}
        }]"#;
        let runner = MockCommandRunner::success(json);
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let paged = provider.list(None).await.expect("list should succeed");

        let author = paged.items[0].author.as_ref().expect("author");
        assert_eq!(author.login, "");
        assert_eq!(author.id, "42");
    }

    #[tokio::test]
    async fn test_should_fail_list_when_created_at_is_not_rfc3339() {
        // 钉住容错边界：null/缺失可以退化，但格式错误的字符串（非 RFC3339）
        // 仍然必须整体失败——这不是本次修复要处理的场景，文档不得声称已覆盖。
        let json = r#"[{
            "id": 1,
            "tag_name": "v1.0.0",
            "created_at": "2026-01-01 00:00:00"
        }]"#;
        let runner = MockCommandRunner::success(json);
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let err = provider
            .list(None)
            .await
            .expect_err("malformed created_at must still fail the whole list");
        // chrono 对非 RFC3339 时间戳给出的错误信息，经 serde_json 透传上来；
        // 钉住这条信息防止「null/缺失容错」被误扩展成「任意格式都容错」。
        assert!(
            err.to_string().contains("premature end of input"),
            "实际错误: {err}"
        );
    }

    #[tokio::test]
    async fn test_should_cap_gitcode_release_per_page_at_api_maximum() {
        let runner = MockCommandRunner::success("[]");
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner.clone());

        provider.list(None).await.expect("list should succeed");

        // 整串相等：`contains("per_page=100")` 在 per_page=1001 时同样为真，
        // 无法证明钳位生效。
        assert_eq!(
            runner.recorded_calls()[0].1,
            vec!["api", "/repos/owner/repo/releases?per_page=100&page=1"],
            "默认 cap={DEFAULT_LIST_LIMIT} 时 per_page 必须被 API 上限 100 钳住"
        );
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
