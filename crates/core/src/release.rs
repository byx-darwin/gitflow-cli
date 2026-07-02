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
    /// Release 的 numeric ID。
    pub id: u64,
    /// 关联的 Git tag 名。
    pub tag_name: String,
    /// Release 标题（可选）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Release 正文（Markdown）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// 是否为草稿 Release。
    pub draft: bool,
    /// 是否为预发布版本。
    pub prerelease: bool,
    /// Release 作者。
    pub author: UserSummary,
    /// 创建时间（UTC）。
    pub created_at: DateTime<Utc>,
    /// 发布时间（UTC），草稿 Release 为 None。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<DateTime<Utc>>,
    /// Release 的 Web URL。
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
        assert_eq!(release.author.login, "octocat");
        assert_eq!(release.author.id, "1");
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
}
