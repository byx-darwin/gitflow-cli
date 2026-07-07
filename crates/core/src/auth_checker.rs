//! 认证检查器 trait 和类型定义。
//!
//! 提供同步版本的认证检查接口，用于 CLI 前置检查。

/// 认证检查器 trait（同步版本，用于前置检查）。
///
/// # Examples
///
/// ```ignore
/// use gitflow_cli_core::AuthChecker;
///
/// fn check_auth(checker: &dyn AuthChecker) {
///     if checker.is_authenticated() {
///         println!("User is authenticated");
///     }
/// }
/// ```
pub trait AuthChecker: Send + Sync {
    /// 快速检查是否已认证（不查询 API，仅检查本地凭据）。
    ///
    /// # Returns
    ///
    /// 如果已认证返回 `true`，否则返回 `false`。
    fn is_authenticated(&self) -> bool;

    /// 获取认证检查的详细状态。
    ///
    /// # Returns
    ///
    /// 包含认证状态、用户名、失败原因和修复建议的 [`AuthCheckResult`]。
    fn check_status(&self) -> AuthCheckResult;
}

/// 认证检查结果。
///
/// 包含认证状态的详细信息，用于生成用户友好的错误消息。
#[derive(Debug, Clone)]
pub struct AuthCheckResult {
    /// 是否已认证。
    pub authenticated: bool,
    /// 用户名（如果已认证）。
    pub user: Option<String>,
    /// 失败原因（如果未认证）。
    pub reason: Option<String>,
    /// 修复建议。
    pub hint: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_create_authenticated_result() {
        let result = AuthCheckResult {
            authenticated: true,
            user: Some("testuser".to_string()),
            reason: None,
            hint: None,
        };
        assert!(result.authenticated);
        assert_eq!(result.user, Some("testuser".to_string()));
    }

    #[test]
    fn test_should_create_not_authenticated_result() {
        let result = AuthCheckResult {
            authenticated: false,
            user: None,
            reason: Some("Not logged in".to_string()),
            hint: Some("Run `gitcode auth login`".to_string()),
        };
        assert!(!result.authenticated);
        assert!(result.reason.is_some());
        assert!(result.hint.is_some());
    }

    #[test]
    fn test_auth_checker_trait_is_object_safe() {
        // 验证 trait 是 object-safe 的
        fn _takes_checker(_checker: &dyn AuthChecker) {}
    }
}
