//! 测试配置模块
//!
//! 从环境变量读取测试配置。

use thiserror::Error;

/// 配置错误
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Missing required environment variable
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),
}

/// 测试模式(由凭据可用性派生)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestMode {
    /// 已认证:可运行真实平台实测场景
    Authenticated,
    /// 未认证:仅运行错误路径与 harness 自测
    Unauthenticated,
}

/// 测试配置
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// 测试仓库（格式：owner/repo）
    pub test_repo: String,
    /// GitHub 令牌
    pub github_token: Option<String>,
    /// `GitCode` 令牌
    pub gitcode_token: Option<String>,
    /// GitLab 令牌
    pub gitlab_token: Option<String>,
    /// GitLab 测试仓库（格式：group/project），用于 `e2e-gitlab` 的 issue/pr 实测
    pub gitlab_test_repo: Option<String>,
    /// `GitCode` 测试仓库（格式：group/project），用于 `e2e-gitcode` 的 issue/pr 实测
    pub gitcode_test_repo: Option<String>,
}

impl TestConfig {
    /// 从环境变量加载配置
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::MissingEnvVar` if `E2E_TEST_REPO` is not set.
    pub fn from_env() -> Result<Self, ConfigError> {
        let test_repo = std::env::var("E2E_TEST_REPO")
            .map_err(|_| ConfigError::MissingEnvVar("E2E_TEST_REPO".to_string()))?;

        Ok(Self {
            test_repo,
            github_token: std::env::var("E2E_GITHUB_TOKEN").ok(),
            gitcode_token: std::env::var("E2E_GITCODE_TOKEN").ok(),
            gitlab_token: std::env::var("E2E_GITLAB_TOKEN").ok(),
            gitlab_test_repo: std::env::var("E2E_TEST_REPO_GITLAB").ok(),
            gitcode_test_repo: std::env::var("E2E_TEST_REPO_GITCODE").ok(),
        })
    }

    /// 从环境变量加载配置(宽松版:`E2E_TEST_REPO` 可缺省)
    ///
    /// 用于不依赖测试仓库的实测(如 `auth status`),fork PR 中
    /// secrets 为空时也能构造配置。
    #[must_use]
    pub fn from_env_lenient() -> Self {
        Self {
            test_repo: std::env::var("E2E_TEST_REPO").unwrap_or_default(),
            github_token: std::env::var("E2E_GITHUB_TOKEN").ok(),
            gitcode_token: std::env::var("E2E_GITCODE_TOKEN").ok(),
            gitlab_token: std::env::var("E2E_GITLAB_TOKEN").ok(),
            gitlab_test_repo: std::env::var("E2E_TEST_REPO_GITLAB").ok(),
            gitcode_test_repo: std::env::var("E2E_TEST_REPO_GITCODE").ok(),
        }
    }

    /// 派生测试模式:有 GitHub 令牌即 `Authenticated`
    #[must_use]
    pub fn mode(&self) -> TestMode {
        if self.has_github_auth() {
            TestMode::Authenticated
        } else {
            TestMode::Unauthenticated
        }
    }

    /// 派生 GitLab 测试模式:有 GitLab 令牌即 `Authenticated`
    #[must_use]
    pub fn gitlab_mode(&self) -> TestMode {
        if self.has_gitlab_auth() {
            TestMode::Authenticated
        } else {
            TestMode::Unauthenticated
        }
    }

    /// 派生 `GitCode` 测试模式:有 `GitCode` 令牌即 `Authenticated`
    #[must_use]
    pub fn gitcode_mode(&self) -> TestMode {
        if self.has_gitcode_auth() {
            TestMode::Authenticated
        } else {
            TestMode::Unauthenticated
        }
    }

    /// 是否具备 GitHub 凭据
    #[must_use]
    pub fn has_github_auth(&self) -> bool {
        self.github_token.is_some()
    }

    /// 是否具备 GitLab 凭据
    #[must_use]
    pub fn has_gitlab_auth(&self) -> bool {
        self.gitlab_token.is_some()
    }

    /// 是否具备 `GitCode` 凭据
    #[must_use]
    pub fn has_gitcode_auth(&self) -> bool {
        self.gitcode_token.is_some()
    }

    /// 需要注入 `gh` 子进程的环境变量;未认证时为空
    ///
    /// 修复凭据从未传递给底层 `gh` 子进程的问题——调用方应将
    /// 返回值逐个传入 `TtyRunner::env`。
    #[must_use]
    pub fn gh_env(&self) -> Vec<(String, String)> {
        self.github_token.as_ref().map_or_else(Vec::new, |token| {
            vec![("GH_TOKEN".to_string(), token.clone())]
        })
    }

    /// 需要注入 `glab` 子进程的环境变量;未认证时为空
    #[must_use]
    pub fn gl_env(&self) -> Vec<(String, String)> {
        self.gitlab_token.as_ref().map_or_else(Vec::new, |token| {
            vec![("GL_TOKEN".to_string(), token.clone())]
        })
    }

    /// 需要注入 `gc`/`gitcode` 子进程的环境变量;未认证时为空
    #[must_use]
    pub fn gitcode_env(&self) -> Vec<(String, String)> {
        self.gitcode_token.as_ref().map_or_else(Vec::new, |token| {
            vec![("GITCODE_TOKEN".to_string(), token.clone())]
        })
    }
}

#[cfg(test)]
mod tests {
    // Note: Environment variable tests are skipped because `std::env::set_var`
    // and `std::env::remove_var` are unsafe in Rust 2024, and this crate
    // forbids unsafe code with `#![forbid(unsafe_code)]`.
    // All logic is tested via directly constructed `TestConfig` values instead.

    use super::*;

    fn config_with_token() -> TestConfig {
        TestConfig {
            test_repo: "owner/repo".to_string(),
            github_token: Some("test-token".to_string()),
            gitcode_token: None,
            gitlab_token: None,
            gitlab_test_repo: None,
            gitcode_test_repo: None,
        }
    }

    fn config_without_token() -> TestConfig {
        TestConfig {
            test_repo: "owner/repo".to_string(),
            github_token: None,
            gitcode_token: None,
            gitlab_token: None,
            gitlab_test_repo: None,
            gitcode_test_repo: None,
        }
    }

    #[test]
    fn test_should_derive_authenticated_mode_when_github_token_present() {
        assert_eq!(config_with_token().mode(), TestMode::Authenticated);
    }

    #[test]
    fn test_should_derive_unauthenticated_mode_when_no_github_token() {
        assert_eq!(config_without_token().mode(), TestMode::Unauthenticated);
    }

    #[test]
    fn test_should_report_github_auth_presence() {
        assert!(config_with_token().has_github_auth());
        assert!(!config_without_token().has_github_auth());
    }

    #[test]
    fn test_should_emit_gh_token_env_pair_when_authenticated() {
        assert_eq!(
            config_with_token().gh_env(),
            vec![("GH_TOKEN".to_string(), "test-token".to_string())]
        );
    }

    #[test]
    fn test_should_emit_empty_env_when_unauthenticated() {
        assert!(config_without_token().gh_env().is_empty());
    }

    fn config_with_gitlab_token() -> TestConfig {
        TestConfig {
            test_repo: "owner/repo".to_string(),
            github_token: None,
            gitcode_token: None,
            gitlab_token: Some("gl-token".to_string()),
            gitlab_test_repo: Some("group/project".to_string()),
            gitcode_test_repo: None,
        }
    }

    fn config_with_gitcode_token() -> TestConfig {
        TestConfig {
            test_repo: "owner/repo".to_string(),
            github_token: None,
            gitcode_token: Some("gc-token".to_string()),
            gitlab_token: None,
            gitlab_test_repo: None,
            gitcode_test_repo: Some("group/project".to_string()),
        }
    }

    #[test]
    fn test_should_derive_gitlab_authenticated_mode_when_token_present() {
        assert_eq!(
            config_with_gitlab_token().gitlab_mode(),
            TestMode::Authenticated
        );
    }

    #[test]
    fn test_should_derive_gitlab_unauthenticated_mode_when_no_token() {
        assert_eq!(
            config_without_token().gitlab_mode(),
            TestMode::Unauthenticated
        );
    }

    #[test]
    fn test_should_derive_gitcode_authenticated_mode_when_token_present() {
        assert_eq!(
            config_with_gitcode_token().gitcode_mode(),
            TestMode::Authenticated
        );
    }

    #[test]
    fn test_should_derive_gitcode_unauthenticated_mode_when_no_token() {
        assert_eq!(
            config_without_token().gitcode_mode(),
            TestMode::Unauthenticated
        );
    }

    #[test]
    fn test_should_report_gitlab_auth_presence() {
        assert!(config_with_gitlab_token().has_gitlab_auth());
        assert!(!config_without_token().has_gitlab_auth());
    }

    #[test]
    fn test_should_report_gitcode_auth_presence() {
        assert!(config_with_gitcode_token().has_gitcode_auth());
        assert!(!config_without_token().has_gitcode_auth());
    }

    #[test]
    fn test_should_emit_gl_token_env_pair_when_authenticated() {
        assert_eq!(
            config_with_gitlab_token().gl_env(),
            vec![("GL_TOKEN".to_string(), "gl-token".to_string())]
        );
    }

    #[test]
    fn test_should_emit_empty_gl_env_when_unauthenticated() {
        assert!(config_without_token().gl_env().is_empty());
    }

    #[test]
    fn test_should_emit_gitcode_token_env_pair_when_authenticated() {
        assert_eq!(
            config_with_gitcode_token().gitcode_env(),
            vec![("GITCODE_TOKEN".to_string(), "gc-token".to_string())]
        );
    }

    #[test]
    fn test_should_emit_empty_gitcode_env_when_unauthenticated() {
        assert!(config_without_token().gitcode_env().is_empty());
    }
}
