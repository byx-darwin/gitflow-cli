//! Release 领域类型与平台抽象。
//!
//! 定义了 Release（版本发布）的数据表示、创建参数，以及跨平台
//! 实现所需的 [`ReleaseProvider`] trait。GitHub、GitLab、GitCode
//! 等平台实现都需实现该 trait，使上层命令层可统一消费。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{Result, types::UserSummary};

/// Release 数据。
///
/// 由平台实现填充并返回给上层命令。字段命名与 `gh release`
/// CLI 输出的 JSON 字段对齐（camelCase）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseData {
    /// Release 的 numeric ID（list 命令可能不包含此字段）。
    #[serde(default, alias = "databaseId")]
    pub id: u64,
    /// 关联的 Git tag 名。
    pub tag_name: String,
    /// Release 标题（可选）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Release 正文（Markdown，list 命令可能不包含此字段）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// 是否为草稿 Release。
    #[serde(alias = "isDraft")]
    pub draft: bool,
    /// 是否为预发布版本。
    #[serde(alias = "isPrerelease")]
    pub prerelease: bool,
    /// Release 作者（list 命令可能不包含此字段）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<UserSummary>,
    /// 创建时间（UTC）。
    pub created_at: DateTime<Utc>,
    /// 发布时间（UTC），草稿 Release 为 None。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<DateTime<Utc>>,
    /// Release 的 Web URL（list 命令可能不包含此字段）。
    #[serde(default)]
    pub url: String,
}

/// 创建 Release 所需参数。
#[derive(Debug, Clone)]
pub struct CreateReleaseArgs {
    /// 关联的 Git tag 名。
    pub tag_name: String,
    /// Release 标题（可选）。
    pub name: Option<String>,
    /// Release 正文（Markdown）。
    pub body: Option<String>,
    /// 是否以草稿方式创建。
    pub draft: bool,
    /// 是否为预发布版本。
    pub prerelease: bool,
    /// 目标 commitish（可选，默认当前分支 HEAD）。
    pub target_commitish: Option<String>,
}

/// Release 操作的平台抽象。
///
/// 所有平台实现（GitHub/GitLab/GitCode）都必须实现此 trait，
/// 以提供统一的 Release 创建、列表、查看、编辑、资源上传/下载
/// 及删除能力。
///
/// # Errors
///
/// 所有方法在平台调用失败、反序列化失败或鉴权失败时返回
/// [`CoreError`](crate::CoreError)。
///
/// [`CoreError`]: crate::CoreError
#[async_trait]
pub trait ReleaseProvider: std::fmt::Debug + Send + Sync {
    /// 创建一条新 Release，返回平台生成的完整数据。
    ///
    /// # Errors
    ///
    /// 当平台 API 调用失败或参数非法时返回错误。
    async fn create(&self, args: CreateReleaseArgs) -> Result<ReleaseData>;

    /// 列出仓库的 Release 列表。
    ///
    /// # Errors
    ///
    /// 当平台 API 调用失败时返回错误。
    async fn list(&self) -> Result<Vec<ReleaseData>>;

    /// 查看指定 tag 的 Release 详情。
    ///
    /// # Errors
    ///
    /// 当 Release 不存在或平台 API 调用失败时返回错误。
    async fn view(&self, tag_name: &str) -> Result<ReleaseData>;

    /// 编辑指定 Release 的元数据，返回更新后的数据。
    ///
    /// # Errors
    ///
    /// 当 Release 不存在或平台 API 调用失败时返回错误。
    async fn edit(&self, tag_name: &str, args: CreateReleaseArgs) -> Result<ReleaseData>;

    /// 上传资源文件到指定 Release。
    ///
    /// `file_path` 为本地文件路径，`asset_name` 为在 Release
    /// 中显示的资源名。
    ///
    /// # Errors
    ///
    /// 当 Release 不存在、文件读取失败或平台 API 调用失败时返回错误。
    async fn upload_asset(&self, tag_name: &str, file_path: &str, asset_name: &str) -> Result<()>;

    /// 下载指定 Release 的资源文件到本地路径。
    ///
    /// # Errors
    ///
    /// 当 Release 不存在、资源不存在或写入失败时返回错误。
    async fn download_asset(&self, tag_name: &str, asset_name: &str, dest_path: &str)
    -> Result<()>;

    /// 删除指定 Release。
    ///
    /// # Errors
    ///
    /// 当 Release 不存在或平台 API 调用失败时返回错误。
    async fn delete(&self, tag_name: &str) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_release_json() -> &'static str {
        r#"{
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
        }"#
    }

    #[test]
    fn test_should_deserialize_release_data_from_json() {
        let json = sample_release_json();
        let release: ReleaseData = serde_json::from_str(json).expect("valid ReleaseData JSON");

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
    fn test_should_roundtrip_release_data_via_serde() {
        let json = sample_release_json();
        let release: ReleaseData = serde_json::from_str(json).expect("deserialize");
        let re_serialized = serde_json::to_string(&release).expect("serialize");
        let round_tripped: ReleaseData =
            serde_json::from_str(&re_serialized).expect("re-deserialize");

        assert_eq!(round_tripped.id, release.id);
        assert_eq!(round_tripped.tag_name, release.tag_name);
        assert_eq!(round_tripped.name, release.name);
        assert_eq!(round_tripped.body, release.body);
        assert_eq!(round_tripped.draft, release.draft);
        assert_eq!(round_tripped.prerelease, release.prerelease);
        assert_eq!(round_tripped.created_at, release.created_at);
        assert_eq!(round_tripped.published_at, release.published_at);
        assert_eq!(round_tripped.url, release.url);
    }

    #[test]
    fn test_should_deserialize_draft_release_with_null_optional_fields() {
        let json = r#"{
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
        let release: ReleaseData = serde_json::from_str(json).expect("deserialize");
        assert!(release.draft);
        assert!(release.prerelease);
        assert!(release.name.is_none());
        assert!(release.body.is_none());
        assert!(release.published_at.is_none());
    }

    #[test]
    fn test_should_omit_none_fields_on_serialize() {
        let json = sample_release_json();
        let mut release: ReleaseData = serde_json::from_str(json).expect("deserialize");
        release.name = None;
        release.body = None;
        release.published_at = None;
        let serialized = serde_json::to_string(&release).expect("serialize");
        assert!(!serialized.contains("null"));
        assert!(!serialized.contains("\"name\":"));
        assert!(!serialized.contains("\"body\":"));
        assert!(!serialized.contains("\"publishedAt\":"));
    }

    #[test]
    fn test_should_serialize_camel_case_fields() {
        let json = sample_release_json();
        let release: ReleaseData = serde_json::from_str(json).expect("deserialize");
        let serialized = serde_json::to_string(&release).expect("serialize");
        assert!(serialized.contains("\"tagName\""));
        assert!(serialized.contains("\"createdAt\""));
        assert!(serialized.contains("\"publishedAt\""));
        assert!(!serialized.contains("\"tag_name\""));
        assert!(!serialized.contains("\"created_at\""));
        assert!(!serialized.contains("\"published_at\""));
    }

    #[test]
    fn test_create_release_args_debug_derive() {
        let args = CreateReleaseArgs {
            tag_name: "v1.0.0".into(),
            name: Some("v1".into()),
            body: None,
            draft: false,
            prerelease: false,
            target_commitish: Some("abc123".into()),
        };
        let debug = format!("{args:?}");
        assert!(debug.contains("CreateReleaseArgs"));
        assert!(debug.contains("v1.0.0"));
    }

    // --- Alias deserialization tests ---

    #[test]
    fn test_should_deserialize_is_draft_alias() {
        let json = r#"{
            "id": 1,
            "tagName": "v0.1.0",
            "isDraft": true,
            "prerelease": false,
            "createdAt": "2026-01-01T00:00:00Z"
        }"#;
        let release: ReleaseData = serde_json::from_str(json).expect("deserialize");
        assert!(release.draft);
        assert!(!release.prerelease);
    }

    #[test]
    fn test_should_deserialize_is_prerelease_alias() {
        let json = r#"{
            "id": 1,
            "tagName": "v0.1.0",
            "draft": false,
            "isPrerelease": true,
            "createdAt": "2026-01-01T00:00:00Z"
        }"#;
        let release: ReleaseData = serde_json::from_str(json).expect("deserialize");
        assert!(!release.draft);
        assert!(release.prerelease);
    }

    #[test]
    fn test_should_deserialize_database_id_alias() {
        let json = r#"{
            "databaseId": 42,
            "tagName": "v1.0.0",
            "draft": false,
            "prerelease": false,
            "createdAt": "2026-01-01T00:00:00Z"
        }"#;
        let release: ReleaseData = serde_json::from_str(json).expect("deserialize");
        assert_eq!(release.id, 42);
    }

    #[test]
    fn test_should_reject_duplicate_id_and_database_id() {
        let json = r#"{
            "id": 10,
            "databaseId": 99,
            "tagName": "v1.0.0",
            "draft": false,
            "prerelease": false,
            "createdAt": "2026-01-01T00:00:00Z"
        }"#;
        let result: std::result::Result<ReleaseData, _> = serde_json::from_str(json);
        // Serde rejects duplicate fields when both the primary name and an alias
        // are present in the same JSON object.
        assert!(result.is_err());
    }

    // --- Default / optional field tests ---

    #[test]
    fn test_should_default_url_to_empty_string() {
        let json = r#"{
            "id": 1,
            "tagName": "v0.1.0",
            "draft": false,
            "prerelease": false,
            "createdAt": "2026-01-01T00:00:00Z"
        }"#;
        let release: ReleaseData = serde_json::from_str(json).expect("deserialize");
        assert_eq!(release.url, "");
    }

    #[test]
    fn test_should_default_id_to_zero() {
        let json = r#"{
            "tagName": "v0.1.0",
            "draft": false,
            "prerelease": false,
            "createdAt": "2026-01-01T00:00:00Z"
        }"#;
        let release: ReleaseData = serde_json::from_str(json).expect("deserialize");
        assert_eq!(release.id, 0);
    }

    // --- Skip-serializing-if tests for optional fields ---

    #[test]
    fn test_should_skip_author_when_none_on_serialize() {
        let json = r#"{
            "id": 1,
            "tagName": "v1.0.0",
            "draft": false,
            "prerelease": false,
            "createdAt": "2026-01-01T00:00:00Z",
            "url": ""
        }"#;
        let release: ReleaseData = serde_json::from_str(json).expect("deserialize");
        assert!(release.author.is_none());
        let serialized = serde_json::to_string(&release).expect("serialize");
        assert!(!serialized.contains("\"author\""));
    }

    // --- Debug format tests ---

    #[test]
    fn test_release_data_should_have_debug_format() {
        let json = sample_release_json();
        let release: ReleaseData = serde_json::from_str(json).expect("deserialize");
        let debug = format!("{release:?}");
        assert!(debug.contains("ReleaseData"));
        assert!(debug.contains("v1.0.0"));
        assert!(debug.contains("Version 1.0.0"));
    }

    // --- CreateReleaseArgs additional tests ---

    #[test]
    fn test_should_create_release_args_with_minimal_fields() {
        let args = CreateReleaseArgs {
            tag_name: "v1.0.0".into(),
            name: None,
            body: None,
            draft: false,
            prerelease: false,
            target_commitish: None,
        };
        assert_eq!(args.tag_name, "v1.0.0");
        assert!(args.name.is_none());
        assert!(args.body.is_none());
        assert!(!args.draft);
        assert!(!args.prerelease);
        assert!(args.target_commitish.is_none());
    }

    #[test]
    fn test_should_create_release_args_with_all_fields() {
        let args = CreateReleaseArgs {
            tag_name: "v2.0.0".into(),
            name: Some("Major Release".into()),
            body: Some("## Breaking Changes\n- ...".into()),
            draft: true,
            prerelease: true,
            target_commitish: Some("main".into()),
        };
        assert_eq!(args.tag_name, "v2.0.0");
        assert_eq!(args.name.as_deref(), Some("Major Release"));
        assert_eq!(args.body.as_deref(), Some("## Breaking Changes\n- ..."));
        assert!(args.draft);
        assert!(args.prerelease);
        assert_eq!(args.target_commitish.as_deref(), Some("main"));
    }

    #[test]
    fn test_create_release_args_should_be_cloneable() {
        let args = CreateReleaseArgs {
            tag_name: "v1.0.0".into(),
            name: Some("v1".into()),
            body: Some("body".into()),
            draft: true,
            prerelease: false,
            target_commitish: Some("abc123".into()),
        };
        let cloned = args.clone();
        assert_eq!(cloned.tag_name, args.tag_name);
        assert_eq!(cloned.name, args.name);
        assert_eq!(cloned.body, args.body);
        assert_eq!(cloned.draft, args.draft);
        assert_eq!(cloned.prerelease, args.prerelease);
        assert_eq!(cloned.target_commitish, args.target_commitish);
    }
}
