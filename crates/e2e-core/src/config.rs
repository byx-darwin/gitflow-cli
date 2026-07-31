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

    /// 是否具备 GitHub 凭据
    #[must_use]
    pub fn has_github_auth(&self) -> bool {
        self.github_token.is_some()
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
        }
    }

    fn config_without_token() -> TestConfig {
        TestConfig {
            test_repo: "owner/repo".to_string(),
            github_token: None,
            gitcode_token: None,
            gitlab_token: None,
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
}
