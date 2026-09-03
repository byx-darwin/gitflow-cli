//! Pull Request 领域类型与平台抽象。
//!
//! 定义了 PR 的数据表示、创建/列表参数，以及跨平台实现所需的
//! [`PrProvider`] trait。GitHub、GitLab、GitCode 等平台实现都
//! 需实现该 trait，使上层命令层可统一消费。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    Result,
    types::{
        CommentData, MergeResult, MergeStrategy, State, UserSummary, deserialize_u64_or_string,
    },
};

/// Pull Request 数据。
///
/// 由平台实现填充并返回给上层命令。字段命名与 `gh pr`
/// CLI 输出的 JSON 字段对齐（camelCase）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrData {
    /// PR 编号（平台内唯一）。
    #[serde(deserialize_with = "deserialize_u64_or_string")]
    pub number: u64,
    /// PR 标题。
    pub title: String,
    /// PR 正文（Markdown）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// PR 当前状态。
    pub state: State,
    /// 是否为草稿 PR。
    #[serde(alias = "isDraft")]
    pub draft: bool,
    /// PR 作者。
    pub author: UserSummary,
    /// 目标分支。
    #[serde(alias = "baseRefName")]
    pub base_branch: String,
    /// 来源分支。
    #[serde(alias = "headRefName")]
    pub head_branch: String,
    /// 创建时间（UTC）。
    pub created_at: DateTime<Utc>,
    /// 最近更新时间（UTC）。
    pub updated_at: DateTime<Utc>,
    /// 合并时间（UTC）；未合并时为 `None`。
    ///
    /// 存在的必要：[`State`] 把 `MERGED` alias 进 `Closed`，仅凭 `state`
    /// **无法区分"已合并"与"关闭但未合并"**，而 Branch Finish 必须区分二者
    /// 才能安全删分支。`gh` 提供 `mergedAt`；GitLab/GitCode 若不返回则为 `None`，
    /// 调用方须把 `None` 当作"未知"而非"未合并"。
    pub merged_at: Option<DateTime<Utc>>,
    /// PR 的 Web URL。
    pub url: String,
}

/// 创建 PR 所需参数。
#[derive(Debug, Clone)]
pub struct CreatePrArgs {
    /// PR 标题。
    pub title: String,
    /// PR 正文（可选）。
    pub body: Option<String>,
    /// 来源分支。
    pub head: String,
    /// 目标分支。
    pub base: String,
    /// 是否以草稿方式创建。
    pub draft: bool,
    /// 可选的目标仓库（`owner/name` 格式），未设置时使用默认仓库。
    pub repo: Option<String>,
    /// 需要在合并时自动关闭的 Issue 编号列表。
    pub closes_issues: Vec<u64>,
}

/// 列出 PR 的过滤参数。
///
/// 所有字段均可选，未设置时使用平台默认值。
#[derive(Debug, Clone, Default)]
pub struct ListPrArgs {
    /// 按状态过滤。
    pub state: Option<State>,
    /// 返回数量上限。
    pub limit: Option<u32>,
}

/// PR 操作的平台抽象。
///
/// 所有平台实现（GitHub/GitLab/GitCode）都必须实现此 trait，
/// 以提供统一的 PR 创建、列表、查看、关闭、合并、检出等能力。
///
/// # Errors
///
/// 所有方法在平台调用失败、反序列化失败或鉴权失败时返回 [`CoreError`](crate::CoreError)。
///
/// [`CoreError`]: crate::CoreError
#[async_trait]
pub trait PrProvider: std::fmt::Debug + Send + Sync {
    /// 创建一条新 PR，返回平台生成的完整数据。
    ///
    /// # Errors
    ///
    /// 当平台 API 调用失败或参数非法时返回错误。
    async fn create(&self, args: CreatePrArgs) -> Result<PrData>;

    /// 根据过滤条件列出 PR 列表。
    ///
    /// # Errors
    ///
    /// 当平台 API 调用失败或过滤条件非法时返回错误。
    async fn list(&self, args: ListPrArgs) -> Result<Vec<PrData>>;

    /// 查看指定编号的 PR 详情。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在或平台 API 调用失败时返回错误。
    async fn view(&self, number: u64) -> Result<PrData>;

    /// 关闭指定编号的 PR，返回更新后的数据。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在或平台 API 调用失败时返回错误。
    async fn close(&self, number: u64) -> Result<PrData>;

    /// 重新打开指定编号的 PR，返回更新后的数据。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在或平台 API 调用失败时返回错误。
    async fn reopen(&self, number: u64) -> Result<PrData>;

    /// 在指定 PR 上添加评论，返回新建评论的数据。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在、`body` 为空或平台 API 调用失败时返回错误。
    async fn comment(&self, number: u64, body: &str) -> Result<CommentData>;

    /// 合并指定编号的 PR，返回合并结果。
    ///
    /// `strategy` 指定合并策略（merge/squash/rebase）。
    /// 未指定时使用平台默认策略。
    ///
    /// `auto` 为 `true` 时**排队合并**：满足条件即由平台自动完成合并，
    /// 调用方不必等待 CI。GitHub 与 GitLab 原生支持；GitCode 不支持，
    /// 传 `true` 会返回 [`CoreError::Platform`]（与 `PipelineProvider` 的处理一致）。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在、无法合并、平台不支持 `auto` 或平台 API 调用失败时返回错误。
    async fn merge(
        &self,
        number: u64,
        strategy: Option<MergeStrategy>,
        auto: bool,
    ) -> Result<MergeResult>;

    /// 在本地检出指定 PR 的分支。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在或 git 操作失败时返回错误。
    async fn checkout(&self, number: u64) -> Result<()>;

    /// 将草稿 PR 标记为可审查状态（ready for review）。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在、不是草稿或平台 API 调用失败时返回错误。
    async fn mark_ready(&self, number: u64) -> Result<PrData>;

    /// 将 PR 标记为草稿状态（work in progress）。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在或平台 API 调用失败时返回错误。
    async fn mark_wip(&self, number: u64) -> Result<PrData>;

    /// 同步 PR 分支（将 base 分支的最新变更合入 head 分支）。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在或同步失败时返回错误。
    async fn sync_branch(&self, number: u64) -> Result<()>;

    /// 获取指定 PR 的统一差异格式（unified diff）文本。
    ///
    /// 返回平台原生的 unified diff 输出，可直接用于 `git apply`。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在或平台 API 调用失败时返回 [`CoreError`]。
    ///
    /// [`CoreError`]: crate::CoreError
    async fn diff(&self, number: u64) -> Result<String>;

    /// 获取指定 PR 的 patch 格式文本（含邮件头信息）。
    ///
    /// 返回包含 commit 元数据的 patch 格式输出，可用于 `git am`。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在或平台 API 调用失败时返回 [`CoreError`]。
    ///
    /// [`CoreError`]: crate::CoreError
    async fn patch(&self, number: u64) -> Result<String>;

    /// 查询仓库配置的默认分支（如 `main`、`dev`）。
    ///
    /// 用于 `pr create` 在未显式指定 `--base` 时探测目标分支，避免硬编码
    /// `"main"` 导致默认分支非 `main` 的仓库创建出目标错误的 PR/MR。
    ///
    /// # Errors
    ///
    /// 当平台 API 调用失败或平台不支持该查询（如 GitCode）时返回错误。
    async fn default_branch(&self) -> Result<String>;
}

/// Format closing keywords and append to body.
///
/// `keyword` is the platform closing verb (e.g. `"Closes"`, `"Fixes"`).
/// Returns `None` if both `body` is `None`/empty and `issues` is empty.
/// Returns the original body unchanged if `issues` is empty.
///
/// # Examples
///
/// ```
/// use gitflow_core::pr::format_closing_body;
///
/// let body = Some("Description".to_string());
/// let result = format_closing_body(&body, &[24, 23], "Closes");
/// assert_eq!(result, Some("Description\n\nCloses #24\nCloses #23".to_string()));
/// ```
#[must_use]
pub fn format_closing_body(body: &Option<String>, issues: &[u64], keyword: &str) -> Option<String> {
    if issues.is_empty() {
        return body.clone();
    }
    let closing = issues
        .iter()
        .map(|n| format!("{keyword} #{n}"))
        .collect::<Vec<_>>()
        .join("\n");
    Some(match body {
        Some(b) if !b.is_empty() => format!("{b}\n\n{closing}"),
        _ => closing,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 构造用于 serde 测试的样本 JSON，字段命名与 `gh pr view --json` 输出一致。
    fn sample_pr_json() -> &'static str {
        r#"{
            "number": 101,
            "title": "Add feature X",
            "body": "Implements X per spec.",
            "state": "open",
            "draft": false,
            "author": {"login": "alice", "id": "7"},
            "baseBranch": "main",
            "headBranch": "feature/x",
            "createdAt": "2026-02-10T08:00:00Z",
            "updatedAt": "2026-02-11T12:30:00Z",
            "url": "https://github.com/octocat/hello-world/pull/101"
        }"#
    }

    #[test]
    fn test_should_deserialize_pr_data_from_gh_cli_json() {
        let json = sample_pr_json();
        let pr: PrData = serde_json::from_str(json).expect("valid PrData JSON");

        assert_eq!(pr.number, 101);
        assert_eq!(pr.title, "Add feature X");
        assert_eq!(pr.body.as_deref(), Some("Implements X per spec."));
        assert_eq!(pr.state, State::Open);
        assert!(!pr.draft);
        assert_eq!(pr.author.login, "alice");
        assert_eq!(pr.author.id, "7");
        assert_eq!(pr.base_branch, "main");
        assert_eq!(pr.head_branch, "feature/x");
        assert_eq!(pr.url, "https://github.com/octocat/hello-world/pull/101");
    }

    #[test]
    fn test_should_roundtrip_pr_data_via_serde() {
        let json = sample_pr_json();
        let pr: PrData = serde_json::from_str(json).expect("deserialize");
        let re_serialized = serde_json::to_string(&pr).expect("serialize");
        let round_tripped: PrData = serde_json::from_str(&re_serialized).expect("re-deserialize");

        assert_eq!(round_tripped.number, pr.number);
        assert_eq!(round_tripped.title, pr.title);
        assert_eq!(round_tripped.body, pr.body);
        assert_eq!(round_tripped.state, pr.state);
        assert_eq!(round_tripped.draft, pr.draft);
        assert_eq!(round_tripped.base_branch, pr.base_branch);
        assert_eq!(round_tripped.head_branch, pr.head_branch);
        assert_eq!(round_tripped.url, pr.url);
        assert_eq!(round_tripped.created_at, pr.created_at);
        assert_eq!(round_tripped.updated_at, pr.updated_at);
    }

    #[test]
    fn test_should_deserialize_draft_pr_with_null_body() {
        let json = r#"{
            "number": 5,
            "title": "WIP experiment",
            "body": null,
            "state": "open",
            "draft": true,
            "author": {"login": "bob", "id": "9"},
            "baseBranch": "main",
            "headBranch": "wip",
            "createdAt": "2026-04-01T00:00:00Z",
            "updatedAt": "2026-04-02T00:00:00Z",
            "url": "https://example.com/pull/5"
        }"#;
        let pr: PrData = serde_json::from_str(json).expect("deserialize");
        assert!(pr.draft);
        assert!(pr.body.is_none());
        assert_eq!(pr.state, State::Open);
    }

    #[test]
    fn test_should_omit_none_body_on_serialize() {
        let json = sample_pr_json();
        let mut pr: PrData = serde_json::from_str(json).expect("deserialize");
        pr.body = None;
        let serialized = serde_json::to_string(&pr).expect("serialize");
        // `body: null` 不应出现在输出中
        assert!(!serialized.contains("\"body\":null"));
        assert!(!serialized.contains("\"body\": null"));
    }

    #[test]
    fn test_should_serialize_camel_case_fields() {
        let json = sample_pr_json();
        let pr: PrData = serde_json::from_str(json).expect("deserialize");
        let serialized = serde_json::to_string(&pr).expect("serialize");
        // camelCase 字段必须被保留
        assert!(serialized.contains("\"baseBranch\""));
        assert!(serialized.contains("\"headBranch\""));
        assert!(serialized.contains("\"createdAt\""));
        assert!(serialized.contains("\"updatedAt\""));
        // snake_case 字段不应出现
        assert!(!serialized.contains("\"base_branch\""));
        assert!(!serialized.contains("\"head_branch\""));
    }

    #[test]
    fn test_list_pr_args_default_is_empty() {
        let args = ListPrArgs::default();
        assert!(args.state.is_none());
        assert!(args.limit.is_none());
    }

    /// Compile-time check that `PrProvider` has `diff()` and `patch()` methods.
    ///
    /// This test passes if the code compiles; the runtime assertion is trivial.
    #[test]
    fn test_should_have_diff_and_patch_methods_on_trait() {
        use async_trait::async_trait;

        use crate::{Result, pr::PrProvider};

        #[derive(Debug)]
        struct Check;

        #[async_trait]
        impl PrProvider for Check {
            async fn create(&self, _args: crate::pr::CreatePrArgs) -> Result<crate::pr::PrData> {
                unimplemented!()
            }
            async fn list(&self, _args: crate::pr::ListPrArgs) -> Result<Vec<crate::pr::PrData>> {
                unimplemented!()
            }
            async fn view(&self, _number: u64) -> Result<crate::pr::PrData> {
                unimplemented!()
            }
            async fn close(&self, _number: u64) -> Result<crate::pr::PrData> {
                unimplemented!()
            }
            async fn reopen(&self, _number: u64) -> Result<crate::pr::PrData> {
                unimplemented!()
            }
            async fn comment(
                &self,
                _number: u64,
                _body: &str,
            ) -> Result<crate::types::CommentData> {
                unimplemented!()
            }
            async fn merge(
                &self,
                _number: u64,
                _strategy: Option<crate::types::MergeStrategy>,
                _auto: bool,
            ) -> Result<crate::types::MergeResult> {
                unimplemented!()
            }
            async fn checkout(&self, _number: u64) -> Result<()> {
                unimplemented!()
            }
            async fn mark_ready(&self, _number: u64) -> Result<crate::pr::PrData> {
                unimplemented!()
            }
            async fn mark_wip(&self, _number: u64) -> Result<crate::pr::PrData> {
                unimplemented!()
            }
            async fn sync_branch(&self, _number: u64) -> Result<()> {
                unimplemented!()
            }
            async fn diff(&self, _number: u64) -> Result<String> {
                unimplemented!()
            }
            async fn patch(&self, _number: u64) -> Result<String> {
                unimplemented!()
            }
            async fn default_branch(&self) -> Result<String> {
                unimplemented!()
            }
        }

        // Verify Check implements PrProvider with diff() and patch()
        let check = Check;
        let _ = format!("{check:?}");
    }

    // --- format_closing_body tests ---

    #[test]
    fn test_should_return_body_unchanged_when_no_issues() {
        let body = Some("Existing body".to_string());
        let result = crate::pr::format_closing_body(&body, &[], "Closes");
        assert_eq!(result, Some("Existing body".to_string()));
    }

    #[test]
    fn test_should_return_none_when_no_body_and_no_issues() {
        let result = crate::pr::format_closing_body(&None, &[], "Closes");
        assert!(result.is_none());
    }

    #[test]
    fn test_should_create_closing_body_when_no_existing_body() {
        let result = crate::pr::format_closing_body(&None, &[24], "Closes");
        assert_eq!(result, Some("Closes #24".to_string()));
    }

    #[test]
    fn test_should_append_closing_to_existing_body() {
        let body = Some("Feature description".to_string());
        let result = crate::pr::format_closing_body(&body, &[24], "Closes");
        assert_eq!(
            result,
            Some("Feature description\n\nCloses #24".to_string())
        );
    }

    #[test]
    fn test_should_handle_multiple_issues() {
        let body = Some("Description".to_string());
        let result = crate::pr::format_closing_body(&body, &[24, 23], "Closes");
        assert_eq!(
            result,
            Some("Description\n\nCloses #24\nCloses #23".to_string())
        );
    }

    #[test]
    fn test_should_use_custom_keyword() {
        let result = crate::pr::format_closing_body(&None, &[42], "Fixes");
        assert_eq!(result, Some("Fixes #42".to_string()));
    }

    #[test]
    fn test_should_treat_empty_body_as_no_body() {
        let body = Some(String::new());
        let result = crate::pr::format_closing_body(&body, &[10], "Closes");
        assert_eq!(result, Some("Closes #10".to_string()));
    }

    #[test]
    fn test_should_have_closes_issues_field() {
        let args = crate::pr::CreatePrArgs {
            title: "Test".to_string(),
            body: None,
            head: "feature".to_string(),
            base: "main".to_string(),
            draft: false,
            repo: None,
            closes_issues: vec![24, 23],
        };
        assert_eq!(args.closes_issues, vec![24, 23]);
    }
}
