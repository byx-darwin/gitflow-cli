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
    types::{CommentData, MergeResult, MergeStrategy, State, UserSummary},
};

/// Pull Request 数据。
///
/// 由平台实现填充并返回给上层命令。字段命名与 `gh pr`
/// CLI 输出的 JSON 字段对齐（camelCase）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrData {
    /// PR 编号（平台内唯一）。
    pub number: u64,
    /// PR 标题。
    pub title: String,
    /// PR 正文（Markdown）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// PR 当前状态。
    pub state: State,
    /// 是否为草稿 PR。
    pub draft: bool,
    /// PR 作者。
    pub author: UserSummary,
    /// 目标分支。
    pub base_branch: String,
    /// 来源分支。
    pub head_branch: String,
    /// 创建时间（UTC）。
    pub created_at: DateTime<Utc>,
    /// 最近更新时间（UTC）。
    pub updated_at: DateTime<Utc>,
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
    /// # Errors
    ///
    /// 当 PR 不存在、无法合并或平台 API 调用失败时返回错误。
    async fn merge(&self, number: u64, strategy: Option<MergeStrategy>) -> Result<MergeResult>;

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
            "author": {"login": "alice", "id": 7},
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
        assert_eq!(pr.author.id, 7);
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
            "author": {"login": "bob", "id": 9},
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
}
