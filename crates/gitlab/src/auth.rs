//! GitLab 认证提供者实现。
//!
//! 通过 `glab auth` CLI 命令实现 [`AuthProvider`] trait，支持登录、
//! 登出、状态查询及 Token 管理。
//! 所有方法通过 `tokio::process::Command` 调用 `glab`。

use async_trait::async_trait;
use gitflow_cli_core::{
    CoreError, Result,
    auth::{AuthProvider, AuthStatus},
};
use tracing::debug;

use crate::error::parse_glab_error;

/// GitLab 认证提供者，通过 `glab` CLI 操作认证。
///
/// # Examples
///
/// ```no_run
/// use gitflow_cli_gitlab::GitLabAuthProvider;
///
/// let provider = GitLabAuthProvider::new();
/// ```
#[derive(Debug, Clone)]
pub struct GitLabAuthProvider;

impl GitLabAuthProvider {
    /// 创建新的 GitLab 认证提供者。
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for GitLabAuthProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuthProvider for GitLabAuthProvider {
    async fn login(&self, token: Option<&str>) -> Result<()> {
        debug!("spawning `glab auth login`");

        let mut cmd = tokio::process::Command::new("glab");
        cmd.arg("auth").arg("login");

        // If token is provided, use non-interactive mode via stdin
        if let Some(token) = token {
            cmd.arg("--stdin");
            cmd.stdin(std::process::Stdio::piped());
            let mut child = cmd.spawn().map_err(|e| {
                CoreError::Platform(format!("Failed to spawn glab auth login: {e}"))
            })?;

            // Write token to stdin
            if let Some(mut stdin) = child.stdin.take() {
                use tokio::io::AsyncWriteExt;
                stdin.write_all(token.as_bytes()).await.map_err(|e| {
                    CoreError::Platform(format!("Failed to write token to stdin: {e}"))
                })?;
                // Drop stdin to close the pipe
                drop(stdin);
            }

            let status = child.wait().await.map_err(|e| {
                CoreError::Platform(format!("Failed to wait for glab auth login: {e}"))
            })?;

            if !status.success() {
                return Err(CoreError::Platform("glab auth login failed".into()));
            }
        } else {
            // Interactive mode
            let status = cmd.status().await.map_err(|e| {
                CoreError::Platform(format!("Failed to spawn glab auth login: {e}"))
            })?;

            if !status.success() {
                return Err(CoreError::Platform("glab auth login failed".into()));
            }
        }

        Ok(())
    }

    async fn logout(&self) -> Result<()> {
        debug!("spawning `glab auth logout`");

        let output = tokio::process::Command::new("glab")
            .args(["auth", "logout"])
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab auth logout: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        Ok(())
    }

    async fn status(&self) -> Result<AuthStatus> {
        debug!("spawning `glab auth status`");

        let output = tokio::process::Command::new("glab")
            .args(["auth", "status"])
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab auth status: {e}")))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
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

            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        // glab auth status outputs to stderr, not stdout
        let combined = format!("{stdout}{stderr}");
        let user = parse_user_from_status(&combined);

        Ok(AuthStatus {
            logged_in: user.is_some(),
            user,
            scopes: vec![],
        })
    }

    async fn token(&self) -> Result<String> {
        debug!("spawning `glab auth token`");

        let output = tokio::process::Command::new("glab")
            .args(["auth", "token"])
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab auth token: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        let token = String::from_utf8_lossy(&output.stdout);
        Ok(token.trim().to_string())
    }
}

fn parse_user_from_status(output: &str) -> Option<String> {
    for line in output.lines() {
        if let Some(pos) = line.find(" as ") {
            let after_as = &line[pos + 4..];
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
    fn test_should_construct_gitlab_auth_provider() {
        let provider = GitLabAuthProvider::new();
        let _ = format!("{provider:?}");
    }

    #[test]
    fn test_should_default_gitlab_auth_provider() {
        let provider = GitLabAuthProvider;
        let _ = format!("{provider:?}");
    }

    #[test]
    fn test_should_clone_gitlab_auth_provider() {
        let original = GitLabAuthProvider::new();
        let cloned = original.clone();
        let _ = format!("{cloned:?}");
    }

    #[test]
    fn test_should_parse_user_from_status_output() {
        let status = r"gitlab.com
  ✓ Logged in to gitlab.com as root (keyring)
  ✓ Git operations for gitlab.com configured to use ssh protocol.
";
        assert_eq!(parse_user_from_status(status), Some("root".to_string()));
    }

    #[test]
    fn test_should_parse_user_from_status_with_different_host() {
        let status = r"gitlab.com
  ✓ Logged in to gitlab.com as alice (oauth_token)
  ✓ Token: glpat-****
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
        let status = "Logged in to gitlab.com as bob";
        assert_eq!(parse_user_from_status(status), Some("bob".to_string()));
    }
}
