//! GitCode Pipeline 提供者实现。
//!
//! **注意**: GitCode CLI v0.6.1 不支持 `run` 命令，GitCode API 也没有 pipeline 端点。
//! 所有方法返回友好错误消息。

use async_trait::async_trait;
use gitflow_core::{
    CoreError, Result, Session,
    pipeline::{JobData, PipelineProvider, PipelineReport, PipelineStatus},
};

/// GitCode Pipeline 提供者，通过 `gitcode` CLI 操作 CI/CD 流水线。
///
/// # Examples
///
/// ```no_run
/// use gitflow_gitcode::GitCodePipelineProvider;
///
/// let provider = GitCodePipelineProvider::new("octocat/hello-world");
/// ```
#[derive(Debug, Clone)]
pub struct GitCodePipelineProvider {
    #[allow(dead_code, reason = "Stored for future pipeline API calls")]
    repo: String,
}

impl GitCodePipelineProvider {
    /// 创建新的 GitCode Pipeline 提供者。
    ///
    /// `repo` 格式为 `owner/repo`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> Self {
        Self { repo: repo.into() }
    }

    /// Create a new provider from a shared [`Session`].
    ///
    /// This enables state reuse across multiple operations in workflow chains.
    #[must_use]
    pub fn with_session(session: &Session) -> Self {
        Self {
            repo: session.repo.clone(),
        }
    }
}

#[async_trait]
impl PipelineProvider for GitCodePipelineProvider {
    async fn status(&self, _branch: &str) -> Result<Vec<PipelineStatus>> {
        Err(CoreError::Platform(
            "GitCode does not support pipeline management. GitCode CLI v0.6.1 does not have 'run' \
             command."
                .into(),
        ))
    }

    async fn logs(&self, _pipeline_id: u64) -> Result<String> {
        Err(CoreError::Platform(
            "GitCode does not support pipeline management. GitCode CLI v0.6.1 does not have 'run' \
             command."
                .into(),
        ))
    }

    async fn jobs(&self, _pipeline_id: u64) -> Result<Vec<JobData>> {
        Err(CoreError::Platform(
            "GitCode does not support pipeline management. GitCode CLI v0.6.1 does not have 'run' \
             command."
                .into(),
        ))
    }

    async fn report(&self, _branch: &str, _days: u32) -> Result<PipelineReport> {
        Err(CoreError::Platform(
            "GitCode does not support pipeline management. GitCode CLI v0.6.1 does not have 'run' \
             command."
                .into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_construct_gitcode_pipeline_provider() {
        let provider = GitCodePipelineProvider::new("octocat/hello-world");
        assert_eq!(provider.repo, "octocat/hello-world");
    }

    #[test]
    fn test_should_construct_gitcode_pipeline_provider_from_string() {
        let repo = String::from("octocat/hello-world");
        let provider = GitCodePipelineProvider::new(repo);
        assert_eq!(provider.repo, "octocat/hello-world");
    }

    #[test]
    fn test_should_debug_format_provider() {
        let provider = GitCodePipelineProvider::new("octocat/hello-world");
        let debug = format!("{provider:?}");
        assert!(debug.contains("GitCodePipelineProvider"));
        assert!(debug.contains("octocat/hello-world"));
    }

    #[test]
    fn test_should_clone_gitcode_pipeline_provider() {
        let original = GitCodePipelineProvider::new("owner/repo");
        let cloned = original.clone();
        assert_eq!(original.repo, cloned.repo);
    }

    // --- Stub behavior: all pipeline operations are unsupported on GitCode ---

    #[tokio::test]
    async fn test_should_return_platform_error_for_status() {
        let provider = GitCodePipelineProvider::new("owner/repo");
        let result = provider.status("main").await;
        assert!(matches!(result.unwrap_err(), CoreError::Platform(_)));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_for_logs() {
        let provider = GitCodePipelineProvider::new("owner/repo");
        let result = provider.logs(123).await;
        assert!(matches!(result.unwrap_err(), CoreError::Platform(_)));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_for_jobs() {
        let provider = GitCodePipelineProvider::new("owner/repo");
        let result = provider.jobs(123).await;
        assert!(matches!(result.unwrap_err(), CoreError::Platform(_)));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_for_report() {
        let provider = GitCodePipelineProvider::new("owner/repo");
        let result = provider.report("main", 7).await;
        assert!(matches!(result.unwrap_err(), CoreError::Platform(_)));
    }
}
