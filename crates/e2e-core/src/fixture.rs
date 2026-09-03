//! 测试数据管理模块
//!
//! 提供测试资源的创建和清理。

use thiserror::Error;

use crate::{TtyMode, TtyRunner};

/// 固件错误
#[derive(Debug, Error)]
pub enum FixtureError {
    /// TTY error
    #[error("TTY error: {0}")]
    Tty(#[from] crate::tty::TtyError),
    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    /// Config error
    #[error("Config error: {0}")]
    Config(#[from] crate::config::ConfigError),
}

/// 测试资源类型
#[derive(Debug, Clone)]
pub enum TestResource {
    /// Issue 资源
    Issue {
        /// Issue number
        number: u64,
    },
    /// PR 资源
    Pr {
        /// PR number
        number: u64,
    },
    /// Label 资源
    Label {
        /// Label name
        name: String,
    },
}

/// 测试固件管理器
#[derive(Debug)]
pub struct TestFixture {
    repo: String,
    created_resources: Vec<TestResource>,
}

impl TestFixture {
    /// 从显式配置创建测试固件(不访问环境变量,便于 hermetic 测试)
    #[must_use]
    pub fn with_config(config: &crate::TestConfig) -> Self {
        Self {
            repo: config.test_repo.clone(),
            created_resources: Vec::new(),
        }
    }

    /// 创建新的测试固件
    ///
    /// # Errors
    ///
    /// Returns `FixtureError::Config` if `E2E_TEST_REPO` is not set.
    pub fn new() -> Result<Self, FixtureError> {
        let config = crate::TestConfig::from_env()?;
        Ok(Self::with_config(&config))
    }

    /// 清理所有创建的资源
    ///
    /// # Errors
    ///
    /// Returns `FixtureError::Tty` if cleanup commands fail.
    pub async fn cleanup(&mut self) -> Result<(), FixtureError> {
        let runner = TtyRunner::new(TtyMode::NonInteractive);

        for resource in self.created_resources.drain(..) {
            match resource {
                TestResource::Issue { number } => {
                    let _ = runner
                        .run(&["issue", "close", &number.to_string(), "--repo", &self.repo])
                        .await;
                }
                TestResource::Label { name } => {
                    let _ = runner
                        .run(&["label", "delete", "--name", &name, "--repo", &self.repo])
                        .await;
                }
                TestResource::Pr { number } => {
                    let _ = runner
                        .run(&["pr", "close", &number.to_string(), "--repo", &self.repo])
                        .await;
                }
            }
        }
        Ok(())
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        if !self.created_resources.is_empty() {
            tracing::warn!(
                "TestFixture dropped with {} resources not cleaned up",
                self.created_resources.len()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TestConfig;

    fn offline_config() -> TestConfig {
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
    fn test_should_build_fixture_from_config_without_env_access() {
        let fixture = TestFixture::with_config(&offline_config());
        assert_eq!(fixture.repo, "owner/repo");
        assert!(fixture.created_resources.is_empty());
    }

    #[tokio::test]
    async fn test_should_cleanup_empty_fixture_without_side_effects() {
        let mut fixture = TestFixture::with_config(&offline_config());
        assert!(fixture.cleanup().await.is_ok());
    }

    #[test]
    fn test_should_not_panic_when_dropping_empty_fixture() {
        let fixture = TestFixture::with_config(&offline_config());
        drop(fixture);
    }
}
