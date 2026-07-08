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

use crate::error::parse_gitcode_error;

/// `gc run list` 请求的 JSON 字段列表。
const PIPELINE_FIELDS: &str = "databaseId,headBranch,status,conclusion,createdAt,updatedAt,url";

/// 将 GitCode `gc run list` 返回的 status 字符串映射为 [`PipelineStatusEnum`]。
///
/// GitCode 返回小写状态：`queued`、`in_progress`、`completed`、`waiting`、
/// `requested`、`pending` 等。其中 `completed` 需要结合 `conclusion`
/// 判断最终结果。
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
#[allow(dead_code)]
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
    #[allow(dead_code)]
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

/// `gc run view --json jobs` 的包裹结构体。
///
/// GitCode 返回 `{"jobs": [...]}` 而非直接数组。
#[derive(Debug, Deserialize)]
struct JobsResponse {
    jobs: Vec<GcJob>,
}

/// GitCode 单次 job 的原始响应。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GcJob {
    database_id: u64,
    name: String,
    status: String,
    conclusion: Option<String>,
    started_at: Option<String>,
    completed_at: Option<String>,
    url: String,
}

impl GcJob {
    fn into_job_data(self) -> JobData {
        let parse_ts = |s: Option<&str>| {
            s.and_then(|v| {
                chrono::DateTime::parse_from_rfc3339(v)
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            })
        };

        JobData {
            id: self.database_id,
            name: self.name,
            status: self.status,
            conclusion: self.conclusion,
            started_at: parse_ts(self.started_at.as_deref()),
            completed_at: parse_ts(self.completed_at.as_deref()),
            url: self.url,
        }
    }
}

/// GitCode Pipeline 提供者，通过 `gitcode`  CLI 操作 CI/CD 流水线。
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
    /// GitCode `owner/repo`，如 `"byx-darwin/gitflow-cli"`。
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
    async fn status(&self, branch: &str) -> Result<Vec<PipelineStatus>> {
        // GitCode CLI does not support `run` command (version 0.6.1)
        // and GitCode API does not have pipeline endpoints
        return Err(CoreError::Platform(
            "GitCode does not support pipeline management. GitCode CLI v0.6.1 does not have 'run' \
             command."
                .into(),
        ));
    }

    async fn logs(&self, pipeline_id: u64) -> Result<String> {
        // GitCode CLI does not support `run` command (version 0.6.1)
        // and GitCode API does not have pipeline endpoints
        return Err(CoreError::Platform(
            "GitCode does not support pipeline management. GitCode CLI v0.6.1 does not have 'run' \
             command."
                .into(),
        ));
    }

    async fn jobs(&self, pipeline_id: u64) -> Result<Vec<JobData>> {
        // GitCode CLI does not support `run` command (version 0.6.1)
        // and GitCode API does not have pipeline endpoints
        return Err(CoreError::Platform(
            "GitCode does not support pipeline management. GitCode CLI v0.6.1 does not have 'run' \
             command."
                .into(),
        ));
    }

    async fn report(&self, branch: &str, days: u32) -> Result<PipelineReport> {
        // GitCode CLI does not support `run` command (version 0.6.1)
        // and GitCode API does not have pipeline endpoints
        return Err(CoreError::Platform(
            "GitCode does not support pipeline management. GitCode CLI v0.6.1 does not have 'run' \
             command."
                .into(),
        ));
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
}
