//! GitLab 认证提供者实现。
//!
//! 通过 `glab auth` CLI 命令实现 [`AuthProvider`] trait，支持登录、
//! 登出、状态查询及 Token 管理。
//! 异步方法通过 [`CommandRunner`] 抽象调用 `glab`，测试可注入自定义 runner
//! 以模拟成功或失败场景。

use async_trait::async_trait;
use gitflow_core::{
    CoreError, Result,
    auth::{AuthProvider, AuthStatus},
};
use tracing::debug;

use crate::{
    error::parse_glab_error,
    runner::{CommandRunner, RealCommandRunner},
};

/// GitLab 认证提供者，通过 `glab` CLI 操作认证。
///
/// 命令执行通过 [`CommandRunner`] 抽象，生产环境默认使用
/// [`RealCommandRunner`]，测试可注入自定义 runner 以模拟成功或失败场景。
///
/// # Examples
///
/// ```no_run
/// use gitflow_gitlab::GitLabAuthProvider;
///
/// let provider = GitLabAuthProvider::new();
/// ```
#[derive(Debug, Clone)]
pub struct GitLabAuthProvider<R: CommandRunner = RealCommandRunner> {
    /// 用于执行 `glab` CLI 命令的 runner。
    runner: R,
}

impl GitLabAuthProvider<RealCommandRunner> {
    /// 创建新的 GitLab 认证提供者，使用真实的进程执行器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            runner: RealCommandRunner,
        }
    }

    /// Create a new provider from a shared [`Session`].
    ///
    /// `AuthProvider` doesn't use repository context, so session.repo is ignored.
    /// This enables state reuse across multiple operations in workflow chains.
    #[must_use]
    pub fn with_session(_session: &gitflow_core::Session) -> Self {
        Self {
            runner: RealCommandRunner,
        }
    }
}

impl<R: CommandRunner> GitLabAuthProvider<R> {
    /// 使用自定义 [`CommandRunner`] 创建提供者。
    ///
    /// 主要用于测试，可注入模拟 runner 以控制 `glab` CLI 的输出。
    #[must_use]
    pub fn with_runner(runner: R) -> Self {
        Self { runner }
    }
}

impl Default for GitLabAuthProvider<RealCommandRunner> {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<R: CommandRunner + 'static> AuthProvider for GitLabAuthProvider<R> {
    async fn login(&self, token: Option<&str>) -> Result<()> {
        debug!("spawning `glab auth login`");

        // If token is provided, pass it via stdin to avoid exposing it in
        // process arguments (visible to other users via `ps`).
        let output = if let Some(token) = token {
            self.runner
                .run_with_stdin("glab", &["auth", "login", "--stdin"], token.as_bytes())
                .await
                .map_err(|e| CoreError::Platform(format!("Failed to spawn glab auth login: {e}")))?
        } else {
            self.runner
                .run("glab", &["auth", "login"])
                .await
                .map_err(|e| CoreError::Platform(format!("Failed to spawn glab auth login: {e}")))?
        };

        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
        }

        Ok(())
    }

    async fn logout(&self) -> Result<()> {
        debug!("spawning `glab auth logout`");

        let output = self
            .runner
            .run("glab", &["auth", "logout"])
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab auth logout: {e}")))?;

        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
        }

        Ok(())
    }

    async fn status(&self) -> Result<AuthStatus> {
        debug!("spawning `glab auth status`");

        let output = self
            .runner
            .run("glab", &["auth", "status"])
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab auth status: {e}")))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
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

            return Err(parse_glab_error(&output.stderr).into());
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
        // 环境变量优先（与 AuthChecker::is_authenticated 一致）
        if let Ok(tok) = std::env::var("GL_TOKEN") {
            return Ok(tok);
        }

        debug!("spawning `glab auth status --show-token`");

        let host = std::env::var("GITLAB_HOST").ok();
        let mut args: Vec<&str> = vec!["auth", "status", "--show-token"];
        if let Some(ref h) = host {
            args.push("--hostname");
            args.push(h);
        }

        let output =
            self.runner.run("glab", &args).await.map_err(|e| {
                CoreError::Platform(format!("Failed to spawn glab auth status: {e}"))
            })?;

        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        // glab writes the status block (including the `Token found ...` line) to
        // stderr even on success; parse both streams like `status()` does.
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{stdout}{stderr}");
        combined
            .lines()
            .find_map(|line| {
                if line.contains("Token found") {
                    line.rsplit_once(": ").map(|(_, t)| t.trim().to_string())
                } else {
                    None
                }
            })
            .filter(|t| !t.is_empty())
            .ok_or_else(|| {
                CoreError::Platform("No GitLab token found (run `glab auth login`)".into())
            })
    }
}

// AuthChecker 是同步 trait，必须使用 std::process::Command
#[allow(clippy::disallowed_types, reason = "AuthChecker is synchronous")]
impl<R: CommandRunner> gitflow_core::AuthChecker for GitLabAuthProvider<R> {
    fn is_authenticated(&self) -> bool {
        if std::env::var("GL_TOKEN").is_ok() {
            return true;
        }

        let output = std::process::Command::new("glab")
            .args(["auth", "status"])
            .output();

        matches!(output, Ok(out) if out.status.success())
    }

    fn check_status(&self) -> gitflow_core::AuthCheckResult {
        // 1. 检查环境变量
        if std::env::var("GL_TOKEN").is_ok() {
            return gitflow_core::AuthCheckResult {
                authenticated: true,
                user: None,
                reason: None,
                hint: None,
            };
        }

        // 2. 执行 glab auth status
        let output = match std::process::Command::new("glab")
            .args(["auth", "status"])
            .output()
        {
            Ok(out) => out,
            Err(e) => {
                return gitflow_core::AuthCheckResult {
                    authenticated: false,
                    user: None,
                    reason: Some(format!("Failed to execute glab: {e}")),
                    hint: Some("Install GitLab CLI: brew install glab".into()),
                };
            }
        };

        // 3. 解析结果
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let user = parse_user_from_status(&stdout);

            gitflow_core::AuthCheckResult {
                authenticated: true,
                user,
                reason: None,
                hint: None,
            }
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);

            gitflow_core::AuthCheckResult {
                authenticated: false,
                user: None,
                reason: Some(stderr.to_string()),
                hint: Some("Run `glab auth login` to authenticate".into()),
            }
        }
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
    use crate::runner::MockCommandRunner;

    #[test]
    fn test_should_construct_gitlab_auth_provider() {
        let provider = GitLabAuthProvider::new();
        let debug = format!("{provider:?}");
        assert!(debug.contains("GitLabAuthProvider"));
    }

    #[test]
    fn test_should_default_gitlab_auth_provider() {
        let provider = GitLabAuthProvider::default();
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

    #[test]
    fn test_auth_checker_is_authenticated_with_env_var() {
        use gitflow_core::AuthChecker;
        temp_env::with_var("GL_TOKEN", Some("test_token"), || {
            let provider = GitLabAuthProvider::new();
            assert!(provider.is_authenticated());
        });
    }

    #[test]
    fn test_auth_checker_check_status_with_env_var() {
        use gitflow_core::AuthChecker;
        temp_env::with_var("GL_TOKEN", Some("test_token"), || {
            let provider = GitLabAuthProvider::new();
            let result = provider.check_status();
            assert!(result.authenticated);
            assert!(result.reason.is_none());
        });
    }

    // --- Failure-path tests using an injected MockCommandRunner ---

    #[tokio::test]
    async fn test_should_return_platform_error_when_glab_fails_for_login() {
        let runner = MockCommandRunner::failure("authentication failed", 256);
        let provider = GitLabAuthProvider::with_runner(runner);

        let result = provider.login(Some("glpat-token")).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_glab_fails_for_logout() {
        let runner = MockCommandRunner::failure("not logged in", 256);
        let provider = GitLabAuthProvider::with_runner(runner);

        let result = provider.logout().await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_logged_out_status_when_glab_reports_not_logged_in() {
        let runner = MockCommandRunner::failure("not logged in to any GitLab hosts", 256);
        let provider = GitLabAuthProvider::with_runner(runner);

        let status = provider.status().await.expect("graceful logged-out status");

        assert!(!status.logged_in);
        assert!(status.user.is_none());
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_status_fails_unexpectedly() {
        let runner = MockCommandRunner::failure("internal server error", 256);
        let provider = GitLabAuthProvider::with_runner(runner);

        let result = provider.status().await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_glab_fails_for_token() {
        let runner = MockCommandRunner::failure("no token found", 256);
        let provider = GitLabAuthProvider::with_runner(runner);

        let result = provider.token().await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_error_when_stdout_has_no_token_line() {
        let runner = MockCommandRunner::success("");
        let provider = GitLabAuthProvider::with_runner(runner);

        let result = provider.token().await;

        assert!(result.is_err());
    }

    // --- Success-path tests using an injected MockCommandRunner ---

    #[test]
    fn test_should_construct_with_custom_runner() {
        let runner = MockCommandRunner::success("");
        let provider = GitLabAuthProvider::with_runner(runner);
        let debug = format!("{provider:?}");
        assert!(debug.contains("GitLabAuthProvider"));
    }

    #[tokio::test]
    async fn test_should_login_interactively() {
        let runner = MockCommandRunner::success("");
        let provider = GitLabAuthProvider::with_runner(runner);

        let result = provider.login(None).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_should_login_with_token_via_stdin() {
        let runner = MockCommandRunner::success("");
        let provider = GitLabAuthProvider::with_runner(runner);

        let result = provider.login(Some("glpat-secret-token")).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_should_logout_successfully() {
        let runner = MockCommandRunner::success("");
        let provider = GitLabAuthProvider::with_runner(runner);

        let result = provider.logout().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_should_return_logged_in_status_when_authenticated() {
        let stdout = r"gitlab.com
  ✓ Logged in to gitlab.com as testuser (keyring)
  ✓ Git operations for gitlab.com configured to use ssh protocol.
";
        let runner = MockCommandRunner::success(stdout);
        let provider = GitLabAuthProvider::with_runner(runner);

        let result = provider.status().await;

        assert!(result.is_ok());
        let status = result.unwrap();
        assert!(status.logged_in);
        assert_eq!(status.user, Some("testuser".to_string()));
        assert!(status.scopes.is_empty());
    }

    #[tokio::test]
    async fn test_should_return_token_successfully() {
        let stdout = "  ✓ Token found in operating system keyring: glpat-test12345\n";
        let runner = MockCommandRunner::success(stdout);
        let provider = GitLabAuthProvider::with_runner(runner);

        let result = provider.token().await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "glpat-test12345");
    }

    #[tokio::test]
    async fn test_should_trim_whitespace_from_token() {
        let stdout = "  ✓ Token found in keyring: glpat-test12345  \n\n";
        let runner = MockCommandRunner::success(stdout);
        let provider = GitLabAuthProvider::with_runner(runner);

        let result = provider.token().await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "glpat-test12345");
    }

    #[tokio::test]
    async fn test_should_extract_token_from_auth_status_show_token() {
        let stdout = "192.168.230.23\n  ✓ Logged in to 192.168.230.23 as baoyuexing (keyring)\n  \
                      ✓ Token found in operating system keyring: glpat-abcdef\n";
        let runner = MockCommandRunner::success(stdout);
        let provider = GitLabAuthProvider::with_runner(runner.clone());

        let token = provider.token().await.expect("should get token");

        assert_eq!(token, "glpat-abcdef");
        assert_eq!(
            runner.recorded_calls()[0].1,
            vec!["auth", "status", "--show-token"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn test_should_extract_token_from_stderr_like_real_glab() {
        // `glab auth status --show-token` writes the status block (including the
        // `Token found ...` line) to stderr while exiting 0. Regression test for
        // the real self-hosted GitLab smoke test (Issue #199).
        let stderr = "192.168.230.23\n  ✓ Logged in to 192.168.230.23 as baoyuexing (keyring)\n  \
                      ✓ Token found in operating system keyring: glpat-abcdef\n";
        let runner = MockCommandRunner::success_with_stderr("", stderr);
        let provider = GitLabAuthProvider::with_runner(runner);

        let token = provider
            .token()
            .await
            .expect("should get token from stderr");

        assert_eq!(token, "glpat-abcdef");
    }

    #[tokio::test]
    async fn test_should_error_when_no_token_found() {
        let stdout =
            "  ! No token found (checked config file, keyring, and environment variables).\n";
        let runner = MockCommandRunner::success(stdout);
        let provider = GitLabAuthProvider::with_runner(runner);

        let result = provider.token().await;

        assert!(result.is_err());
    }

    // --- Spawn-error tests using an injected MockCommandRunner ---

    #[tokio::test]
    async fn test_should_return_platform_error_when_login_spawn_fails() {
        let runner = MockCommandRunner::spawn_error();
        let provider = GitLabAuthProvider::with_runner(runner);

        let result = provider.login(None).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_login_with_token_spawn_fails() {
        let runner = MockCommandRunner::spawn_error();
        let provider = GitLabAuthProvider::with_runner(runner);

        let result = provider.login(Some("glpat-token")).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_logout_spawn_fails() {
        let runner = MockCommandRunner::spawn_error();
        let provider = GitLabAuthProvider::with_runner(runner);

        let result = provider.logout().await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_status_spawn_fails() {
        let runner = MockCommandRunner::spawn_error();
        let provider = GitLabAuthProvider::with_runner(runner);

        let result = provider.status().await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_token_spawn_fails() {
        let runner = MockCommandRunner::spawn_error();
        let provider = GitLabAuthProvider::with_runner(runner);

        let result = provider.token().await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Platform(_)
        ));
    }
}
