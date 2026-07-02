//! GitCode 认证提供者实现。
//!
//! 通过 `gc auth` CLI 命令实现 [`AuthProvider`] trait，支持登录、
//! 登出、状态查询及 Token 管理。
//! 所有方法通过 `tokio::process::Command` 调用 `gc`。

use async_trait::async_trait;
use gitflow_cli_core::{
    CoreError, Result,
    auth::{AuthProvider, AuthStatus},
};
use tracing::debug;

use crate::error::parse_gc_error;

/// GitCode 认证提供者，通过 `gc` CLI 操作认证。
///
/// # Examples
///
/// ```no_run
/// use gitflow_cli_gitcode::GitCodeAuthProvider;
///
/// let provider = GitCodeAuthProvider::new();
/// ```
#[derive(Debug, Clone)]
pub struct GitCodeAuthProvider;

impl GitCodeAuthProvider {
    /// 创建新的 GitCode 认证提供者。
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for GitCodeAuthProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuthProvider for GitCodeAuthProvider {
    /// 执行交互式登录。
    ///
    /// 调用 `gc auth login`，将子进程的 stdout/stderr 透传给终端。
    ///
    /// # Errors
    ///
    /// 当认证失败或 `gc` 调用失败时返回错误。
    async fn login(&self) -> Result<()> {
        debug!("spawning `gc auth login`");

        let status = tokio::process::Command::new(crate::gitcode_binary())
            .arg("auth")
            .arg("login")
            .status()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gc auth login: {e}")))?;

        if !status.success() {
            return Err(CoreError::Platform("gc auth login failed".into()));
        }

        Ok(())
    }

    /// 执行登出，清除本地凭据。
    ///
    /// # Errors
    ///
    /// 当 `gc` 调用失败时返回错误。
    async fn logout(&self) -> Result<()> {
        debug!("spawning `gc auth logout`");

        let output = tokio::process::Command::new(crate::gitcode_binary())
            .args(["auth", "logout"])
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gc auth logout: {e}")))?;

        if !output.status.success() {
            let gc_err = parse_gc_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gc_err}")));
        }

        Ok(())
    }

    /// 查询当前认证状态。
    ///
    /// 调用 `gc auth status` 并解析输出来构造 [`AuthStatus`]。
    ///
    /// # Errors
    ///
    /// 当 `gc` 调用失败或无法解析状态输出时返回错误。
    async fn status(&self) -> Result<AuthStatus> {
        debug!("spawning `gc auth status`");

        let output = tokio::process::Command::new(crate::gitcode_binary())
            .args(["auth", "status"])
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gc auth status: {e}")))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // gc auth status 在未登录时会返回非零退出码
        if !output.status.success() {
            // 检查是否为未登录状态
            let text = format!("{stdout}{stderr}");
            if text.to_lowercase().contains("not logged in")
                || text.to_lowercase().contains("no active account")
                || text.to_lowercase().contains("not authenticated")
            {
                return Ok(AuthStatus {
                    logged_in: false,
                    user: None,
                    scopes: vec![],
                });
            }

            let gc_err = parse_gc_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gc_err}")));
        }

        // 解析登录用户：查找 "Logged in to gitcode.com as <username>" 行
        let user = parse_user_from_status(&stdout);

        Ok(AuthStatus {
            logged_in: user.is_some(),
            user,
            scopes: vec![], // gc auth status 不直接返回 scopes 列表
        })
    }

    /// 获取当前有效的访问 Token。
    ///
    /// # Errors
    ///
    /// 当用户未登录或 Token 获取失败时返回错误。
    async fn token(&self) -> Result<String> {
        debug!("spawning `gc auth token`");

        let output = tokio::process::Command::new(crate::gitcode_binary())
            .args(["auth", "token"])
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gc auth token: {e}")))?;

        if !output.status.success() {
            let gc_err = parse_gc_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gc_err}")));
        }

        let token = String::from_utf8_lossy(&output.stdout);
        Ok(token.trim().to_string())
    }
}

/// 从 `gc auth status` 的输出中解析用户名。
fn parse_user_from_status(output: &str) -> Option<String> {
    // 匹配模式："Logged in to gitcode.com as <username>"
    for line in output.lines() {
        if let Some(pos) = line.find(" as ") {
            let after_as = &line[pos + 4..];
            // 用户名后面可能跟 " (" 或空格或其他
            if let Some(end) = after_as.find(' ') {
                let user = &after_as[..end];
                if !user.is_empty() {
                    return Some(user.to_string());
                }
            } else if !after_as.is_empty() {
                return Some(after_as.trim().to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_construct_gitcode_auth_provider() {
        let provider = GitCodeAuthProvider::new();
        let _ = format!("{provider:?}");
    }

    #[test]
    fn test_should_default_gitcode_auth_provider() {
        let provider = GitCodeAuthProvider;
        let _ = format!("{provider:?}");
    }

    #[test]
    fn test_should_clone_gitcode_auth_provider() {
        let original = GitCodeAuthProvider::new();
        let cloned = original.clone();
        let _ = format!("{cloned:?}");
    }

    #[test]
    fn test_should_parse_user_from_status_output() {
        let status = r"gitcode.com
  ✓ Logged in to gitcode.com as octocat (keyring)
  ✓ Git operations for gitcode.com configured to use ssh protocol.
  ✓ API calls for gitcode.com should use https protocol
";
        assert_eq!(parse_user_from_status(status), Some("octocat".to_string()));
    }

    #[test]
    fn test_should_parse_user_from_status_with_different_host() {
        let status = r"gitcode.com
  ✓ Logged in to gitcode.com as alice (oauth_token)
  ✓ Token: gco_****
";
        assert_eq!(parse_user_from_status(status), Some("alice".to_string()));
    }

    #[test]
    fn test_should_return_none_when_no_user_in_status() {
        let status = "No active account found";
        assert!(parse_user_from_status(status).is_none());
    }

    #[test]
    fn test_should_return_none_for_empty_status() {
        assert!(parse_user_from_status("").is_none());
    }

    #[test]
    fn test_should_parse_user_without_suffix() {
        let status = "Logged in to gitcode.com as bob";
        assert_eq!(parse_user_from_status(status), Some("bob".to_string()));
    }
}
