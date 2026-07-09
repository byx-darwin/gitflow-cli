//! GitHub 认证提供者实现。
//!
//! 通过 `gh auth` CLI 命令实现 [`AuthProvider`] trait，支持登录、
//! 登出、状态查询及 Token 管理。
//! 命令执行通过 [`CommandRunner`] 抽象，生产环境默认使用
//! [`RealCommandRunner`]，测试可注入自定义 runner 以模拟成功或失败场景。

use async_trait::async_trait;
use gitflow_cli_core::{
    CoreError, Result,
    auth::{AuthProvider, AuthStatus},
};
use tracing::debug;

use crate::{
    error::parse_gh_error,
    runner::{CommandRunner, RealCommandRunner},
};

/// GitHub 认证提供者，通过 `gh` CLI 操作认证。
///
/// 命令执行通过 [`CommandRunner`] 抽象，生产环境默认使用
/// [`RealCommandRunner`]，测试可注入自定义 runner 以模拟成功或失败场景。
///
/// # Examples
///
/// ```no_run
/// use gitflow_cli_github::GitHubAuthProvider;
///
/// let provider = GitHubAuthProvider::new();
/// ```
#[derive(Debug, Clone)]
pub struct GitHubAuthProvider<R: CommandRunner = RealCommandRunner> {
    /// 用于执行 `gh` CLI 命令的 runner。
    runner: R,
}

impl GitHubAuthProvider<RealCommandRunner> {
    /// 创建新的 GitHub 认证提供者，使用真实的进程执行器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            runner: RealCommandRunner,
        }
    }
}

impl<R: CommandRunner> GitHubAuthProvider<R> {
    /// 使用自定义 [`CommandRunner`] 创建提供者。
    ///
    /// 主要用于测试，可注入模拟 runner 以控制 `gh` CLI 的输出。
    #[must_use]
    pub fn with_runner(runner: R) -> Self {
        Self { runner }
    }
}

impl Default for GitHubAuthProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<R: CommandRunner + 'static> AuthProvider for GitHubAuthProvider<R> {
    /// 执行交互式登录。
    ///
    /// 调用 `gh auth login`，将子进程的 stdout/stderr 透传给终端。
    /// 如果提供了 token，则通过 `--with-token` 参数进行非交互式登录，
    /// 使用 shell 管道将 token 传入 stdin。
    ///
    /// # Errors
    ///
    /// 当认证失败或 `gh` 调用失败时返回错误。
    async fn login(&self, token: Option<&str>) -> Result<()> {
        debug!("spawning `gh auth login`");

        if let Some(token) = token {
            // Non-interactive mode with token via shell pipe
            // Escape single quotes in token for shell safety
            let escaped = token.replace('\'', "'\\''");
            let shell_cmd = format!("printf '%s' '{escaped}' | gh auth login --with-token");
            let output = self
                .runner
                .run("sh", &["-c", &shell_cmd])
                .await
                .map_err(|e| CoreError::Platform(format!("Failed to spawn gh auth login: {e}")))?;

            if !output.status.success() {
                return Err(CoreError::Platform("gh auth login failed".into()));
            }
        } else {
            // Interactive mode
            let output = self
                .runner
                .run("gh", &["auth", "login"])
                .await
                .map_err(|e| CoreError::Platform(format!("Failed to spawn gh auth login: {e}")))?;

            if !output.status.success() {
                return Err(CoreError::Platform("gh auth login failed".into()));
            }
        }

        Ok(())
    }

    /// 执行登出，清除本地凭据。
    ///
    /// # Errors
    ///
    /// 当 `gh` 调用失败时返回错误。
    async fn logout(&self) -> Result<()> {
        debug!("spawning `gh auth logout`");

        let output = self
            .runner
            .run("gh", &["auth", "logout"])
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh auth logout: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        Ok(())
    }

    /// 查询当前认证状态。
    ///
    /// 调用 `gh auth status` 并解析输出来构造 [`AuthStatus`]。
    /// 当 `gh` 命令执行失败或用户未登录时，返回 `AuthStatus { logged_in: false }`
    /// 而非传播错误。
    ///
    /// # Errors
    ///
    /// 仅在 `gh` 返回非零退出码且无法识别为"未登录"模式时返回错误。
    async fn status(&self) -> Result<AuthStatus> {
        debug!("spawning `gh auth status`");

        let Ok(output) = self.runner.run("gh", &["auth", "status"]).await else {
            // CLI 执行失败（如 gh 未安装）→ 视为未登录
            return Ok(AuthStatus {
                logged_in: false,
                user: None,
                scopes: vec![],
            });
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // gh auth status 在未登录时会返回非零退出码
        if !output.status.success() {
            // 检查是否为未登录状态（某些版本会返回特定错误）
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

            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        // 解析登录用户：查找 "Logged in to github.com as <username>" 行
        let user = parse_user_from_status(&stdout);

        Ok(AuthStatus {
            logged_in: user.is_some(),
            user,
            scopes: vec![], // gh auth status 不直接返回 scopes 列表
        })
    }

    /// 获取当前有效的访问 Token。
    ///
    /// # Errors
    ///
    /// 当 `gh` 命令执行失败、用户未登录或 Token 为空时返回错误。
    async fn token(&self) -> Result<String> {
        debug!("spawning `gh auth token`");

        let output = self
            .runner
            .run("gh", &["auth", "token"])
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh auth token: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        let token = String::from_utf8_lossy(&output.stdout);
        let trimmed = token.trim().to_string();
        if trimmed.is_empty() {
            return Err(CoreError::Platform(
                "gh auth token returned empty output".into(),
            ));
        }
        Ok(trimmed)
    }
}

// AuthChecker 是同步 trait，必须使用 std::process::Command
#[allow(clippy::disallowed_types, reason = "AuthChecker is synchronous")]
impl<R: CommandRunner> gitflow_cli_core::AuthChecker for GitHubAuthProvider<R> {
    fn is_authenticated(&self) -> bool {
        if std::env::var("GH_TOKEN").is_ok() {
            return true;
        }

        let output = std::process::Command::new("gh")
            .args(["auth", "status"])
            .output();

        matches!(output, Ok(out) if out.status.success())
    }

    fn check_status(&self) -> gitflow_cli_core::AuthCheckResult {
        // 1. 检查环境变量
        if std::env::var("GH_TOKEN").is_ok() {
            return gitflow_cli_core::AuthCheckResult {
                authenticated: true,
                user: None,
                reason: None,
                hint: None,
            };
        }

        // 2. 执行 gh auth status
        let output = match std::process::Command::new("gh")
            .args(["auth", "status"])
            .output()
        {
            Ok(out) => out,
            Err(e) => {
                return gitflow_cli_core::AuthCheckResult {
                    authenticated: false,
                    user: None,
                    reason: Some(format!("Failed to execute gh: {e}")),
                    hint: Some("Install GitHub CLI: https://cli.github.com".into()),
                };
            }
        };

        // 3. 解析结果
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let user = parse_user_from_status(&stdout);

            gitflow_cli_core::AuthCheckResult {
                authenticated: true,
                user,
                reason: None,
                hint: None,
            }
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);

            gitflow_cli_core::AuthCheckResult {
                authenticated: false,
                user: None,
                reason: Some(stderr.to_string()),
                hint: Some("Run `gh auth login` to authenticate".into()),
            }
        }
    }
}

/// 从 `gh auth status` 的输出中解析用户名。
///
/// 支持两种 `gh` CLI 输出格式：
/// - 旧版：`"Logged in to github.com as <username>"`
/// - 新版：`"Logged in to github.com account <username>"`
fn parse_user_from_status(output: &str) -> Option<String> {
    let separators = [" as ", " account "];
    for line in output.lines() {
        // Only parse lines that contain "Logged in to" to avoid false matches
        if !line.contains("Logged in to") {
            continue;
        }
        for sep in &separators {
            if let Some(pos) = line.find(sep) {
                let after = &line[pos + sep.len()..];
                // 用户名后面可能跟 " (" 或空格或其他
                if let Some(end) = after.find(' ') {
                    let user = &after[..end];
                    if !user.is_empty() {
                        return Some(user.to_string());
                    }
                } else if !after.is_empty() {
                    return Some(after.trim().to_string());
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::MockCommandRunner;

    #[test]
    fn test_should_construct_github_auth_provider() {
        let provider = GitHubAuthProvider::new();
        let _ = format!("{provider:?}");
    }

    #[test]
    fn test_should_default_github_auth_provider() {
        let provider = GitHubAuthProvider::default();
        let _ = format!("{provider:?}");
    }

    #[test]
    fn test_should_clone_github_auth_provider() {
        let original = GitHubAuthProvider::new();
        let cloned = original.clone();
        let _ = format!("{cloned:?}");
    }

    #[test]
    fn test_should_parse_user_from_status_output() {
        let status = r"github.com
  ✓ Logged in to github.com as octocat (keyring)
  ✓ Git operations for github.com configured to use ssh protocol.
  ✓ API calls for github.com should use https protocol
";
        assert_eq!(parse_user_from_status(status), Some("octocat".to_string()));
    }

    #[test]
    fn test_should_parse_user_from_status_with_different_host() {
        let status = r"github.com
  ✓ Logged in to github.com as alice (oauth_token)
  ✓ Token: gho_****
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
        let status = "Logged in to github.com as bob";
        assert_eq!(parse_user_from_status(status), Some("bob".to_string()));
    }

    #[test]
    fn test_should_parse_user_from_account_format() {
        let status = r"github.com
  ✓ Logged in to github.com account octocat (keyring)
  ✓ Git operations for github.com configured to use ssh protocol.
  ✓ API calls for github.com should use https protocol
";
        assert_eq!(parse_user_from_status(status), Some("octocat".to_string()));
    }

    #[test]
    fn test_should_parse_user_from_account_format_without_suffix() {
        let status = "Logged in to github.com account bob";
        assert_eq!(parse_user_from_status(status), Some("bob".to_string()));
    }

    #[test]
    fn test_auth_checker_is_authenticated_with_env_var() {
        use gitflow_cli_core::AuthChecker;
        temp_env::with_var("GH_TOKEN", Some("test_token"), || {
            let provider = GitHubAuthProvider::new();
            assert!(provider.is_authenticated());
        });
    }

    #[test]
    fn test_auth_checker_check_status_with_env_var() {
        use gitflow_cli_core::AuthChecker;
        temp_env::with_var("GH_TOKEN", Some("test_token"), || {
            let provider = GitHubAuthProvider::new();
            let result = provider.check_status();
            assert!(result.authenticated);
            assert!(result.reason.is_none());
        });
    }

    // --- Failure-path tests using an injected MockCommandRunner ---

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_login_fails() {
        let runner = MockCommandRunner::failure("login failed", 1);
        let provider = GitHubAuthProvider::with_runner(runner);

        let result = provider.login(None).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_logout_fails() {
        let runner = MockCommandRunner::failure("logout failed", 1);
        let provider = GitHubAuthProvider::with_runner(runner);

        let result = provider.logout().await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_not_logged_in_when_gh_status_spawn_fails() {
        let runner = MockCommandRunner::spawn_error();
        let provider = GitHubAuthProvider::with_runner(runner);

        let result = provider.status().await;

        assert!(result.is_ok());
        let status = result.unwrap();
        assert!(!status.logged_in);
        assert!(status.user.is_none());
        assert!(status.scopes.is_empty());
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_token_fails() {
        let runner = MockCommandRunner::failure("token failed", 1);
        let provider = GitHubAuthProvider::with_runner(runner);

        let result = provider.token().await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_error_when_gh_token_returns_empty() {
        let runner = MockCommandRunner::success("");
        let provider = GitHubAuthProvider::with_runner(runner);

        let result = provider.token().await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }
}
