//! GitCode Pipeline 提供者实现。
//!
//! **注意**: GitCode CLI v0.6.1 不支持 `run` 命令，GitCode API 也没有 pipeline 端点。
//! 所有方法返回友好错误消息。保留结构体供未来使用。

use async_trait::async_trait;
use gitflow_cli_core::{
    CoreError, Result,
    pipeline::{JobData, PipelineProvider, PipelineReport, PipelineStatus, PipelineStatusEnum},
};
use serde::Deserialize;

/// 将 GitCode `gc run list` 返回的 status 字符串映射为 [`PipelineStatusEnum`]。
///
/// GitCode 返回小写状态：`queued`、`in_progress`、`completed`、`waiting`、
/// `requested`、`pending` 等。其中 `completed` 需要结合 `conclusion`
/// 判断最终结果。
#[allow(dead_code, reason = "Kept for future GitCode pipeline support")]
fn gc_status_to_enum(status: &str, conclusion: Option<&str>) -> PipelineStatusEnum {
    match status {
        "completed" => match conclusion {
            Some("success") => PipelineStatusEnum::Success,
            Some("failure" | "startup_failure" | "timed_out") => PipelineStatusEnum::Failed,
            Some("cancelled") => PipelineStatusEnum::Cancelled,
            Some("skipped" | "neutral") => PipelineStatusEnum::Pending,
            _ => PipelineStatusEnum::Running,
        },
        "queued" | "waiting" | "requested" | "pending" => PipelineStatusEnum::Pending,
        _ => PipelineStatusEnum::Running,
    }
}

/// GitCode 单次 run 的原始响应，用于反序列化。
///
/// Note: Currently unused because GitCode CLI v0.6.1 does not support `run` command.
/// Kept for future use when pipeline support is added.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code, reason = "Kept for future GitCode pipeline support")]
struct GcRun {
    database_id: u64,
    head_branch: String,
    status: String,
    conclusion: Option<String>,
    created_at: String,
    updated_at: String,
    url: String,
}

impl GcRun {
    #[allow(dead_code, reason = "Kept for future GitCode pipeline support")]
    fn into_status(self) -> PipelineStatus {
        let created_at = chrono::DateTime::parse_from_rfc3339(&self.created_at)
            .map_or_else(|_| chrono::Utc::now(), |dt| dt.with_timezone(&chrono::Utc));
        let updated_at = chrono::DateTime::parse_from_rfc3339(&self.updated_at)
            .map_or_else(|_| chrono::Utc::now(), |dt| dt.with_timezone(&chrono::Utc));

        PipelineStatus {
            id: self.database_id,
            ref_name: self.head_branch,
            status: gc_status_to_enum(&self.status, self.conclusion.as_deref()),
            conclusion: self.conclusion,
            created_at,
            updated_at,
            url: self.url,
        }
    }
}

/// GitCode Pipeline 提供者，通过 `gitcode` CLI 操作 CI/CD 流水线。
///
/// # Examples
///
/// ```no_run
/// use gitflow_cli_gitcode::GitCodePipelineProvider;
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

    #[test]
    fn test_should_map_gc_status_completed_success() {
        assert_eq!(
            gc_status_to_enum("completed", Some("success")),
            PipelineStatusEnum::Success
        );
    }

    #[test]
    fn test_should_map_gc_status_completed_failure() {
        assert_eq!(
            gc_status_to_enum("completed", Some("failure")),
            PipelineStatusEnum::Failed
        );
        assert_eq!(
            gc_status_to_enum("completed", Some("startup_failure")),
            PipelineStatusEnum::Failed
        );
        assert_eq!(
            gc_status_to_enum("completed", Some("timed_out")),
            PipelineStatusEnum::Failed
        );
    }

    #[test]
    fn test_should_map_gc_status_completed_cancelled() {
        assert_eq!(
            gc_status_to_enum("completed", Some("cancelled")),
            PipelineStatusEnum::Cancelled
        );
    }

    #[test]
    fn test_should_map_gc_status_completed_skipped() {
        assert_eq!(
            gc_status_to_enum("completed", Some("skipped")),
            PipelineStatusEnum::Pending
        );
        assert_eq!(
            gc_status_to_enum("completed", Some("neutral")),
            PipelineStatusEnum::Pending
        );
    }

    #[test]
    fn test_should_map_gc_status_in_progress() {
        assert_eq!(
            gc_status_to_enum("in_progress", None),
            PipelineStatusEnum::Running
        );
        assert_eq!(
            gc_status_to_enum("action_required", None),
            PipelineStatusEnum::Running
        );
    }

    #[test]
    fn test_should_map_gc_status_queued() {
        assert_eq!(
            gc_status_to_enum("queued", None),
            PipelineStatusEnum::Pending
        );
        assert_eq!(
            gc_status_to_enum("waiting", None),
            PipelineStatusEnum::Pending
        );
        assert_eq!(
            gc_status_to_enum("requested", None),
            PipelineStatusEnum::Pending
        );
    }

    #[test]
    fn test_should_map_gc_status_unknown() {
        assert_eq!(
            gc_status_to_enum("some_unknown_status", None),
            PipelineStatusEnum::Running
        );
    }

    #[test]
    fn test_should_map_completed_with_unknown_conclusion() {
        assert_eq!(
            gc_status_to_enum("completed", Some("weird")),
            PipelineStatusEnum::Running
        );
    }

    #[test]
    fn test_should_map_completed_with_none_conclusion() {
        assert_eq!(
            gc_status_to_enum("completed", None),
            PipelineStatusEnum::Running
        );
    }

    #[test]
    fn test_should_deserialize_gc_run_from_json() {
        let json = br#"{
            "databaseId": 12345,
            "headBranch": "main",
            "status": "completed",
            "conclusion": "success",
            "createdAt": "2026-07-01T10:00:00Z",
            "updatedAt": "2026-07-01T10:05:30Z",
            "url": "https://gitcode.com/example/repo/actions/runs/12345"
        }"#;

        let run: GcRun = serde_json::from_slice(json).expect("valid GcRun JSON");
        assert_eq!(run.database_id, 12345);
        assert_eq!(run.head_branch, "main");
        assert_eq!(run.status, "completed");
        assert_eq!(run.conclusion.as_deref(), Some("success"));
        assert_eq!(
            run.url,
            "https://gitcode.com/example/repo/actions/runs/12345"
        );
    }

    #[test]
    fn test_should_convert_gc_run_to_pipeline_status() {
        let run = GcRun {
            database_id: 42,
            head_branch: "main".into(),
            status: "completed".into(),
            conclusion: Some("success".into()),
            created_at: "2026-07-01T10:00:00Z".into(),
            updated_at: "2026-07-01T10:05:30Z".into(),
            url: "https://example.com/42".into(),
        };

        let status = run.into_status();
        assert_eq!(status.id, 42);
        assert_eq!(status.ref_name, "main");
        assert_eq!(status.status, PipelineStatusEnum::Success);
        assert_eq!(status.conclusion.as_deref(), Some("success"));
    }
}
