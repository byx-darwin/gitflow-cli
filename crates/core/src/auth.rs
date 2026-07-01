//! 认证领域类型与平台抽象。
//!
//! 定义了认证状态数据，以及跨平台实现所需的 [`AuthProvider`] trait。
//! GitHub、GitLab、GitCode 等平台实现都需实现该 trait，使上层
//! 命令层可统一处理登录、登出、状态查询及 Token 管理。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::Result;

/// 当前认证状态。
///
/// 由 [`AuthProvider::status`] 返回，用于判断用户是否已登录、
/// 当前用户身份以及持有的权限范围。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthStatus {
    /// 用户是否已登录。
    pub logged_in: bool,
    /// 当前登录用户名（未登录时为 None）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// 已授权的权限范围列表。
    #[serde(default)]
    pub scopes: Vec<String>,
}

/// 认证操作的平台抽象。
///
/// 所有平台实现（GitHub/GitLab/GitCode）都必须实现此 trait，
/// 以提供统一的登录、登出、状态查询及 Token 获取能力。
///
/// # Errors
///
/// 所有方法在平台调用失败、反序列化失败或鉴权失败时返回
/// [`CoreError`](crate::CoreError)。
///
/// [`CoreError`]: crate::CoreError
#[async_trait]
pub trait AuthProvider: std::fmt::Debug + Send + Sync {
    /// 执行登录流程（交互式或非交互式）。
    ///
    /// 登录成功后凭据会被持久化到本地存储。
    ///
    /// # Errors
    ///
    /// 当认证失败或凭据存储失败时返回错误。
    async fn login(&self) -> Result<()>;

    /// 执行登出流程，清除本地凭据。
    ///
    /// # Errors
    ///
    /// 当凭据清除失败时返回错误。
    async fn logout(&self) -> Result<()>;

    /// 查询当前认证状态。
    ///
    /// # Errors
    ///
    /// 当无法读取本地凭据或平台 API 调用失败时返回错误。
    async fn status(&self) -> Result<AuthStatus>;

    /// 获取当前有效的访问 Token。
    ///
    /// 返回的字符串可用于后续的平台 API 调用。
    ///
    /// # Errors
    ///
    /// 当用户未登录或 Token 获取失败时返回错误。
    async fn token(&self) -> Result<String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_deserialize_auth_status_logged_in() {
        let json = r#"{
            "loggedIn": true,
            "user": "octocat",
            "scopes": ["repo", "read:org"]
        }"#;
        let status: AuthStatus = serde_json::from_str(json).expect("valid AuthStatus JSON");

        assert!(status.logged_in);
        assert_eq!(status.user.as_deref(), Some("octocat"));
        assert_eq!(
            status.scopes,
            vec!["repo".to_string(), "read:org".to_string()]
        );
    }

    #[test]
    fn test_should_deserialize_auth_status_not_logged_in() {
        let json = r#"{
            "loggedIn": false
        }"#;
        let status: AuthStatus = serde_json::from_str(json).expect("valid AuthStatus JSON");

        assert!(!status.logged_in);
        assert!(status.user.is_none());
        assert!(status.scopes.is_empty());
    }

    #[test]
    fn test_should_roundtrip_auth_status_via_serde() {
        let json = r#"{
            "loggedIn": true,
            "user": "alice",
            "scopes": ["repo"]
        }"#;
        let status: AuthStatus = serde_json::from_str(json).expect("deserialize");
        let re_serialized = serde_json::to_string(&status).expect("serialize");
        let round_tripped: AuthStatus =
            serde_json::from_str(&re_serialized).expect("re-deserialize");

        assert_eq!(round_tripped.logged_in, status.logged_in);
        assert_eq!(round_tripped.user, status.user);
        assert_eq!(round_tripped.scopes, status.scopes);
    }

    #[test]
    fn test_should_serialize_auth_status_camel_case() {
        let status = AuthStatus {
            logged_in: true,
            user: Some("bob".into()),
            scopes: vec!["gist".into()],
        };
        let json = serde_json::to_string(&status).expect("serialize");
        assert!(json.contains("\"loggedIn\":true"));
        assert!(json.contains("\"user\":\"bob\""));
        assert!(json.contains("\"scopes\""));
        assert!(!json.contains("\"logged_in\""));
    }

    #[test]
    fn test_should_omit_none_user_on_serialize() {
        let status = AuthStatus {
            logged_in: false,
            user: None,
            scopes: vec![],
        };
        let json = serde_json::to_string(&status).expect("serialize");
        assert!(!json.contains("null"));
        assert!(!json.contains("\"user\":"));
        assert!(json.contains("\"loggedIn\":false"));
    }

    #[test]
    fn test_should_default_empty_scopes() {
        let json = r#"{"loggedIn":true,"user":"u"}"#;
        let status: AuthStatus = serde_json::from_str(json).expect("deserialize");
        assert!(status.scopes.is_empty());
    }

    #[test]
    fn test_auth_status_debug_derive() {
        let status = AuthStatus {
            logged_in: true,
            user: Some("test".into()),
            scopes: vec!["repo".into()],
        };
        let debug = format!("{status:?}");
        assert!(debug.contains("AuthStatus"));
        assert!(debug.contains("test"));
    }
}
