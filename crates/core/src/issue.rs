//! Issue 领域类型与平台抽象。
//!
//! 定义了 Issue 的数据表示、创建/列表参数，以及跨平台实现所需的
//! [`IssueProvider`] trait。GitHub、GitLab、GitCode 等平台实现都
//! 需实现该 trait，使上层命令层可统一消费。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    Result,
    types::{CommentData, Label, State, UserSummary, deserialize_u64_or_string},
};

/// Issue 数据。
///
/// 由平台实现填充并返回给上层命令。字段命名与 GitHub CLI 输出的 JSON 字段对齐（camelCase）。
/// `number` 使用自定义反序列化兼容 GitCode CLI 返回的字符串格式。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueData {
    /// Issue 编号（平台内唯一）。
    #[serde(deserialize_with = "deserialize_u64_or_string")]
    pub number: u64,
    /// Issue 标题。
    pub title: String,
    /// Issue 正文（Markdown）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// Issue 当前状态。
    pub state: State,
    /// 附加的标签列表。
    #[serde(default)]
    pub labels: Vec<Label>,
    /// Issue 作者。
    pub author: UserSummary,
    /// 被指派的成员列表。
    #[serde(default)]
    pub assignees: Vec<UserSummary>,
    /// 创建时间（UTC）。
    pub created_at: DateTime<Utc>,
    /// 最近更新时间（UTC）。
    pub updated_at: DateTime<Utc>,
    /// Issue 的 Web URL。
    pub url: String,
}

/// 创建 Issue 所需参数。
#[derive(Debug, Clone)]
pub struct CreateIssueArgs {
    /// Issue 标题。
    pub title: String,
    /// Issue 正文（可选）。
    pub body: Option<String>,
    /// 附加的标签名列表。
    pub labels: Vec<String>,
    /// 指派的登录名列表。
    pub assignees: Vec<String>,
}

/// 列出 Issue 的过滤参数。
///
/// 所有字段均可选，未设置时使用平台默认值。
#[derive(Debug, Clone, Default)]
pub struct ListIssueArgs {
    /// 按状态过滤。
    pub state: Option<State>,
    /// 按标签名过滤。
    pub labels: Vec<String>,
    /// 按指派用户过滤。
    pub assignee: Option<String>,
    /// 关键字搜索条件。
    pub search: Option<String>,
    /// 返回数量上限。
    pub limit: Option<u32>,
}

/// Issue 操作的平台抽象。
///
/// 所有平台实现（GitHub/GitLab/GitCode）都必须实现此 trait，
/// 以提供统一的 Issue 创建、列表、查看、关闭、评论及标签管理能力。
///
/// # Errors
///
/// 所有方法在平台调用失败、反序列化失败或鉴权失败时返回 [`CoreError`](crate::CoreError)。
///
/// [`CoreError`]: crate::CoreError
#[async_trait]
pub trait IssueProvider: std::fmt::Debug + Send + Sync {
    /// 创建一条新 Issue，返回平台生成的完整数据。
    ///
    /// # Errors
    ///
    /// 当平台 API 调用失败或参数非法时返回错误。
    async fn create(&self, args: CreateIssueArgs) -> Result<IssueData>;

    /// 根据过滤条件列出 Issue 列表。
    ///
    /// # Errors
    ///
    /// 当平台 API 调用失败或过滤条件非法时返回错误。
    async fn list(&self, args: ListIssueArgs) -> Result<Vec<IssueData>>;

    /// 查看指定编号的 Issue 详情。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在或平台 API 调用失败时返回错误。
    async fn view(&self, number: u64) -> Result<IssueData>;

    /// 关闭指定编号的 Issue，返回更新后的数据。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在或平台 API 调用失败时返回错误。
    async fn close(&self, number: u64) -> Result<IssueData>;

    /// 重新打开指定编号的 Issue，返回更新后的数据。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在或平台 API 调用失败时返回错误。
    async fn reopen(&self, number: u64) -> Result<IssueData>;

    /// 在指定 Issue 上添加评论，返回新建评论的数据。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在、`body` 为空或平台 API 调用失败时返回错误。
    async fn comment(&self, number: u64, body: &str) -> Result<CommentData>;

    /// 为指定 Issue 添加标签。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在、标签列表为空或平台 API 调用失败时返回错误。
    async fn add_labels(&self, number: u64, labels: &[String]) -> Result<()>;

    /// 从指定 Issue 移除一个标签。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在、标签不存在或平台 API 调用失败时返回错误。
    async fn remove_label(&self, number: u64, label: &str) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 构造用于 serde 测试的样本 JSON，字段命名与 `gh issue view --json` 输出一致。
    fn sample_issue_json() -> &'static str {
        r#"{
            "number": 42,
            "title": "Fix login bug",
            "body": "Reproduced on v1.2.3",
            "state": "open",
            "labels": [
                {"name": "bug", "color": "d73a4a", "description": "Something isn't working"}
            ],
            "author": {"login": "octocat", "id": "1"},
            "assignees": [{"login": "alice", "id": "7"}],
            "createdAt": "2026-01-15T09:30:00Z",
            "updatedAt": "2026-01-16T11:00:00Z",
            "url": "https://github.com/octocat/hello-world/issues/42"
        }"#
    }

    #[test]
    fn test_should_deserialize_issue_data_from_gh_cli_json() {
        let json = sample_issue_json();
        let issue: IssueData = serde_json::from_str(json).expect("valid IssueData JSON");

        assert_eq!(issue.number, 42);
        assert_eq!(issue.title, "Fix login bug");
        assert_eq!(issue.body.as_deref(), Some("Reproduced on v1.2.3"));
        assert_eq!(issue.state, State::Open);
        assert_eq!(issue.labels.len(), 1);
        assert_eq!(issue.labels[0].name, "bug");
        assert_eq!(issue.labels[0].color.as_deref(), Some("d73a4a"));
        assert_eq!(issue.author.login, "octocat");
        assert_eq!(issue.author.id, "1");
        assert_eq!(issue.assignees.len(), 1);
        assert_eq!(issue.assignees[0].login, "alice");
        assert_eq!(
            issue.url,
            "https://github.com/octocat/hello-world/issues/42"
        );
    }

    #[test]
    fn test_should_roundtrip_issue_data_via_serde() {
        let json = sample_issue_json();
        let issue: IssueData = serde_json::from_str(json).expect("deserialize");
        let re_serialized = serde_json::to_string(&issue).expect("serialize");
        let round_tripped: IssueData =
            serde_json::from_str(&re_serialized).expect("re-deserialize");

        assert_eq!(round_tripped.number, issue.number);
        assert_eq!(round_tripped.title, issue.title);
        assert_eq!(round_tripped.body, issue.body);
        assert_eq!(round_tripped.state, issue.state);
        assert_eq!(round_tripped.url, issue.url);
        assert_eq!(round_tripped.created_at, issue.created_at);
        assert_eq!(round_tripped.updated_at, issue.updated_at);
    }

    #[test]
    fn test_should_deserialize_issue_with_null_optional_body() {
        let json = r#"{
            "number": 1,
            "title": "t",
            "body": null,
            "state": "closed",
            "labels": [],
            "author": {"login": "u", "id": "2"},
            "assignees": [],
            "createdAt": "2026-02-01T00:00:00Z",
            "updatedAt": "2026-02-02T00:00:00Z",
            "url": "https://example.com/1"
        }"#;
        let issue: IssueData = serde_json::from_str(json).expect("deserialize");
        assert!(issue.body.is_none());
        assert_eq!(issue.state, State::Closed);
        assert!(issue.labels.is_empty());
        assert!(issue.assignees.is_empty());
    }

    #[test]
    fn test_should_omit_none_body_on_serialize() {
        let json = sample_issue_json();
        let mut issue: IssueData = serde_json::from_str(json).expect("deserialize");
        issue.body = None;
        let serialized = serde_json::to_string(&issue).expect("serialize");
        // `body: null` 不应出现在输出中
        assert!(!serialized.contains("\"body\":null"));
        assert!(!serialized.contains("\"body\": null"));
    }

    #[test]
    fn test_should_default_empty_labels_and_assignees_when_missing() {
        let json = r#"{
            "number": 7,
            "title": "t",
            "state": "open",
            "author": {"login": "u", "id": "3"},
            "createdAt": "2026-03-01T00:00:00Z",
            "updatedAt": "2026-03-02T00:00:00Z",
            "url": "https://example.com/7"
        }"#;
        let issue: IssueData = serde_json::from_str(json).expect("deserialize");
        assert!(issue.labels.is_empty());
        assert!(issue.assignees.is_empty());
    }

    #[test]
    fn test_list_issue_args_default_is_empty() {
        let args = ListIssueArgs::default();
        assert!(args.state.is_none());
        assert!(args.labels.is_empty());
        assert!(args.assignee.is_none());
        assert!(args.search.is_none());
        assert!(args.limit.is_none());
    }
}
