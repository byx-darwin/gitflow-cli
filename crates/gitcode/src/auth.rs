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

use crate::{
    error::parse_gitcode_error,
    runner::{CommandRunner, RealCommandRunner},
};

/// GitCode 认证提供者，通过 `gitcode` CLI 操作认证。
///
/// 命令执行通过 [`CommandRunner`] 抽象，生产环境默认使用
/// [`RealCommandRunner`]，测试可注入自定义 runner 以模拟成功或失败场景。
///
/// # Examples
///
/// ```no_run
/// use gitflow_cli_gitcode::GitCodeAuthProvider;
///
/// let provider = GitCodeAuthProvider::new();
/// ```
#[derive(Debug, Clone)]
pub struct GitCodeAuthProvider<R: CommandRunner = RealCommandRunner> {
    /// 用于执行 `gitcode` CLI 命令的 runner。
    runner: R,
}

impl GitCodeAuthProvider<RealCommandRunner> {
    /// 创建新的 GitCode 认证提供者。
    #[must_use]
    pub fn new() -> Self {
        Self {
            runner: RealCommandRunner,
        }
    }
}

impl<R: CommandRunner> GitCodeAuthProvider<R> {
    /// 使用自定义 [`CommandRunner`] 创建提供者。
    ///
    /// 主要用于测试，可注入模拟 runner 以控制 `gitcode` CLI 的输出。
    #[must_use]
    pub fn with_runner(runner: R) -> Self {
        Self { runner }
    }
}

impl Default for GitCodeAuthProvider<RealCommandRunner> {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<R: CommandRunner + 'static> AuthProvider for GitCodeAuthProvider<R> {
    /// 执行登录。
    ///
    /// 调用 `gc auth login`。如果提供了 token，则通过 `--with-token` 参数并将
    /// token 写入子进程 stdin 进行非交互式登录，避免 token 出现在进程参数中。
    ///
    /// # Errors
    ///
    /// 当认证失败或 `gc` 调用失败时返回错误。
    async fn login(&self, token: Option<&str>) -> Result<()> {
        debug!("spawning `gc auth login`");

        let binary = crate::gitcode_binary();
        let output = if let Some(token) = token {
            self.runner
                .run_with_stdin(
                    &binary,
                    &["auth", "login", "--with-token"],
                    token.as_bytes(),
                )
                .await
                .map_err(|e| {
                    CoreError::Platform(format!("Failed to spawn gitcode auth login: {e}"))
                })?
        } else {
            self.runner
                .run(&binary, &["auth", "login"])
                .await
                .map_err(|e| {
                    CoreError::Platform(format!("Failed to spawn gitcode auth login: {e}"))
                })?
        };

        if !output.status.success() {
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
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

        let binary = crate::gitcode_binary();
        let output = self
            .runner
            .run(&binary, &["auth", "logout"])
            .await
            .map_err(|e| {
                CoreError::Platform(format!("Failed to spawn gitcode auth logout: {e}"))
            })?;

        if !output.status.success() {
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
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

        let binary = crate::gitcode_binary();
        let output = self
            .runner
            .run(&binary, &["auth", "status"])
            .await
            .map_err(|e| {
                CoreError::Platform(format!("Failed to spawn gitcode auth status: {e}"))
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // gitcode auth status 在未登录时会返回非零退出码
        if !output.status.success() {
            // 检查是否为未登录状态
            let text = format!("{stdout}{stderr}").to_lowercase();
            if text.contains("not logged in")
                || text.contains("no active account")
                || text.contains("not authenticated")
            {
                return Ok(AuthStatus {
                    logged_in: false,
                    user: None,
                    scopes: vec![],
                });
            }

            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
        }

        // 解析登录用户：查找 "Logged in to gitcode.com as <username>" 行
        let user = parse_user_from_status(&stdout);

        Ok(AuthStatus {
            logged_in: user.is_some(),
            user,
            scopes: vec![], // gitcode auth status 不直接返回 scopes 列表
        })
    }

    /// 获取当前有效的访问 Token。
    ///
    /// # Errors
    ///
    /// 当用户未登录或 Token 获取失败时返回错误。
    async fn token(&self) -> Result<String> {
        debug!("spawning `gc auth token`");

        let binary = crate::gitcode_binary();
        let output = self
            .runner
            .run(&binary, &["auth", "token"])
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode auth token: {e}")))?;

        if !output.status.success() {
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
        }

        let token = String::from_utf8_lossy(&output.stdout);
        Ok(token.trim().to_string())
    }
}

// AuthChecker 是同步 trait，必须使用 std::process::Command
#[allow(clippy::disallowed_types, reason = "AuthChecker is synchronous")]
impl<R: CommandRunner> gitflow_cli_core::AuthChecker for GitCodeAuthProvider<R> {
    fn is_authenticated(&self) -> bool {
        // 1. 优先检查环境变量
        if std::env::var("GITCODE_TOKEN").is_ok() {
            return true;
        }

        // 2. 检查 gitcode CLI 是否可用
        let binary = crate::gitcode_binary();
        debug!(binary = %binary, "checking gitcode authentication");

        let output = std::process::Command::new(&binary)
            .args(["auth", "status"])
            .output();

        match output {
            Ok(out) => {
                debug!(
                    exit_code = %out.status,
                    stdout = %String::from_utf8_lossy(&out.stdout),
                    stderr = %String::from_utf8_lossy(&out.stderr),
                    "gitcode auth status result"
                );
                out.status.success()
            }
            Err(e) => {
                debug!(error = %e, "failed to execute gitcode auth status");
                false
            }
        }
    }

    fn check_status(&self) -> gitflow_cli_core::AuthCheckResult {
        // 1. 检查环境变量
        if std::env::var("GITCODE_TOKEN").is_ok() {
            return gitflow_cli_core::AuthCheckResult {
                authenticated: true,
                user: None,
                reason: None,
                hint: None,
            };
        }

        // 2. 执行 gitcode auth status
        let binary = crate::gitcode_binary();
        let output = match std::process::Command::new(&binary)
            .args(["auth", "status"])
            .output()
        {
            Ok(out) => out,
            Err(e) => {
                return gitflow_cli_core::AuthCheckResult {
                    authenticated: false,
                    user: None,
                    reason: Some(format!("Failed to execute gitcode: {e}")),
                    hint: Some("Ensure gitcode CLI is installed: pip install gitcode-cli".into()),
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
                hint: Some("Run `gitcode auth login` to authenticate".into()),
            }
        }
    }
}

/// 从 `gc auth status` 的输出中解析用户名。
///
/// 支持两种格式：
/// - 旧格式: `"Logged in to gitcode.com as <username>"`
/// - 新格式: `"Logged in as <username>"`
fn parse_user_from_status(output: &str) -> Option<String> {
    // 匹配模式：查找 " as " 后跟用户名（两种格式都适用）
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
    use crate::runner::MockCommandRunner;

    #[test]
    fn test_should_construct_gitcode_auth_provider() {
        let provider = GitCodeAuthProvider::new();
        let _ = format!("{provider:?}");
    }

    #[test]
    fn test_should_default_gitcode_auth_provider() {
        let provider = GitCodeAuthProvider::default();
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

    #[test]
    fn test_should_parse_user_from_status_new_format() {
        let status = "Logged in as alice";
        assert_eq!(parse_user_from_status(status), Some("alice".to_string()));
    }

    #[test]
    fn test_should_parse_user_from_status_new_format_with_suffix() {
        let status = "Logged in as bob (oauth_token)";
        assert_eq!(parse_user_from_status(status), Some("bob".to_string()));
    }

    #[test]
    fn test_auth_checker_is_authenticated_with_env_var() {
        use gitflow_cli_core::AuthChecker;
        temp_env::with_var("GITCODE_TOKEN", Some("test_token"), || {
            let provider = GitCodeAuthProvider::new();
            assert!(provider.is_authenticated());
        });
    }

    #[test]
    fn test_auth_checker_check_status_with_env_var() {
        use gitflow_cli_core::AuthChecker;
        temp_env::with_var("GITCODE_TOKEN", Some("test_token"), || {
            let provider = GitCodeAuthProvider::new();
            let result = provider.check_status();
            assert!(result.authenticated);
            assert!(result.reason.is_none());
        });
    }

    // --- Failure-path tests using an injected MockCommandRunner ---

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_login() {
        let runner = MockCommandRunner::failure("authentication failed", 256);
        let provider = GitCodeAuthProvider::with_runner(runner);

        let result = provider.login(Some("gco_token")).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_logout() {
        let runner = MockCommandRunner::failure("not logged in", 256);
        let provider = GitCodeAuthProvider::with_runner(runner);

        let result = provider.logout().await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_logged_out_status_when_gc_reports_not_logged_in() {
        let runner = MockCommandRunner::failure("not logged in to any GitCode hosts", 256);
        let provider = GitCodeAuthProvider::with_runner(runner);

        let status = provider.status().await.expect("graceful logged-out status");

        assert!(!status.logged_in);
        assert!(status.user.is_none());
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_status_fails_unexpectedly() {
        let runner = MockCommandRunner::failure("internal server error", 256);
        let provider = GitCodeAuthProvider::with_runner(runner);

        let result = provider.status().await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_token() {
        let runner = MockCommandRunner::failure("no token found", 256);
        let provider = GitCodeAuthProvider::with_runner(runner);

        let result = provider.token().await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_empty_token_when_stdout_is_empty() {
        let runner = MockCommandRunner::success("");
        let provider = GitCodeAuthProvider::with_runner(runner);

        let token = provider
            .token()
            .await
            .expect("empty stdout yields empty token");

        assert!(token.is_empty());
    }
}
