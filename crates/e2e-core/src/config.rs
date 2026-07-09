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

/// 测试配置
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// 测试仓库（格式：owner/repo）
    pub test_repo: String,
    /// GitHub 令牌
    pub github_token: Option<String>,
    /// GitCode 令牌
    pub gitcode_token: Option<String>,
    /// GitLab 令牌
    pub gitlab_token: Option<String>,
}

impl TestConfig {
    /// 从环境变量加载配置
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_missing_env_var() {
        std::env::remove_var("E2E_TEST_REPO");
        let result = TestConfig::from_env();
        assert!(result.is_err());
    }
}
