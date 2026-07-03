//! GitCode Pipeline 提供者实现。
//!
//! 通过 `gc run list` / `gc run view` CLI 实现 [`PipelineProvider`] trait。

use async_trait::async_trait;
use gitflow_cli_core::{
    CoreError, Result,
    pipeline::{JobData, PipelineProvider, PipelineReport, PipelineStatus, PipelineStatusEnum},
};
use serde::Deserialize;
use tracing::debug;

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
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
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
        debug!(repo = %self.repo, branch = %branch, "spawning `gc run list`");

        let output = tokio::process::Command::new(crate::gitcode_binary())
            .args(["run", "list"])
            .arg("--branch")
            .arg(branch)
            .arg("--repo")
            .arg(&self.repo)
            .arg("--json")
            .arg(PIPELINE_FIELDS)
            .arg("--limit")
            .arg("30")
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
        }

        let runs: Vec<GcRun> =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(runs.into_iter().map(GcRun::into_status).collect())
    }

    async fn logs(&self, pipeline_id: u64) -> Result<String> {
        debug!(repo = %self.repo, pipeline_id, "spawning `gc run view --log`");

        let output = tokio::process::Command::new(crate::gitcode_binary())
            .args(["run", "view"])
            .arg(pipeline_id.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .arg("--log")
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
        }

        String::from_utf8(output.stdout)
            .map_err(|e| CoreError::Platform(format!("Failed to decode log output as UTF-8: {e}")))
    }

    async fn jobs(&self, pipeline_id: u64) -> Result<Vec<JobData>> {
        debug!(repo = %self.repo, pipeline_id, "spawning `gc run view --json jobs`");

        let output = tokio::process::Command::new(crate::gitcode_binary())
            .args(["run", "view"])
            .arg(pipeline_id.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .arg("--json")
            .arg("jobs")
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
        }

        let resp: JobsResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(resp.jobs.into_iter().map(GcJob::into_job_data).collect())
    }

    async fn report(&self, branch: &str, days: u32) -> Result<PipelineReport> {
        // 使用最小结构体反序列化 report 所需字段
        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct ReportRun {
            conclusion: Option<String>,
            created_at: String,
            updated_at: String,
        }

        debug!(
            repo = %self.repo,
            branch = %branch,
            days,
            "spawning `gc run list` for report"
        );

        let output = tokio::process::Command::new(crate::gitcode_binary())
            .args(["run", "list"])
            .arg("--branch")
            .arg(branch)
            .arg("--repo")
            .arg(&self.repo)
            .arg("--json")
            .arg("conclusion,createdAt,updatedAt")
            .arg("--limit")
            .arg("100")
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
        }

        let runs: Vec<ReportRun> =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        let total_runs = runs.len() as u64;

        let mut success_count: u64 = 0;
        let mut total_duration_secs: f64 = 0.0;
        let mut failure_counts: std::collections::HashMap<String, u64> =
            std::collections::HashMap::new();
        let mut has_duration: u64 = 0;

        for run in &runs {
            if let Some(ref conclusion) = run.conclusion {
                if conclusion == "success" {
                    success_count += 1;
                } else if !matches!(conclusion.as_str(), "cancelled" | "skipped" | "neutral") {
                    *failure_counts.entry(conclusion.clone()).or_insert(0) += 1;
                }
            }

            if let (Ok(created), Ok(updated)) = (
                chrono::DateTime::parse_from_rfc3339(&run.created_at),
                chrono::DateTime::parse_from_rfc3339(&run.updated_at),
            ) {
                let duration = (updated.with_timezone(&chrono::Utc)
                    - created.with_timezone(&chrono::Utc))
                .num_seconds();
                if duration > 0 {
                    #[allow(
                        clippy::cast_precision_loss,
                        reason = "Duration values are small enough to fit in f64 without loss"
                    )]
                    let duration_f64 = duration as f64;
                    total_duration_secs += duration_f64;
                    has_duration += 1;
                }
            }
        }

        #[allow(
            clippy::cast_precision_loss,
            reason = "Run counts are small enough to fit in f64 without loss"
        )]
        let success_rate = if total_runs > 0 {
            success_count as f64 / total_runs as f64
        } else {
            0.0
        };

        #[allow(
            clippy::cast_precision_loss,
            reason = "Duration count is small enough to fit in f64 without loss"
        )]
        let avg_duration_secs = if has_duration > 0 {
            total_duration_secs / (has_duration as f64)
        } else {
            0.0
        };

        // 按失败次数降序取 top 失败结论
        let mut failures: Vec<_> = failure_counts.into_iter().collect();
        failures.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let top_failures: Vec<String> = failures.into_iter().map(|(k, _)| k).collect();

        Ok(PipelineReport {
            total_runs,
            success_rate,
            avg_duration_secs,
            top_failures,
        })
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
    fn test_should_deserialize_gc_run_list_from_json_array() {
        let json = br#"[
            {
                "databaseId": 1,
                "headBranch": "main",
                "status": "completed",
                "conclusion": "success",
                "createdAt": "2026-07-01T10:00:00Z",
                "updatedAt": "2026-07-01T10:05:00Z",
                "url": "https://example.com/1"
            },
            {
                "databaseId": 2,
                "headBranch": "main",
                "status": "in_progress",
                "conclusion": null,
                "createdAt": "2026-07-01T11:00:00Z",
                "updatedAt": "2026-07-01T11:01:00Z",
                "url": "https://example.com/2"
            }
        ]"#;

        let runs: Vec<GcRun> = serde_json::from_slice(json).expect("valid GcRun list");
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].database_id, 1);
        assert_eq!(runs[1].database_id, 2);
    }

    #[test]
    fn test_should_deserialize_empty_run_list() {
        let json = b"[]";
        let runs: Vec<GcRun> = serde_json::from_slice(json).expect("empty list");
        assert!(runs.is_empty());
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
    fn test_should_deserialize_jobs_response_from_json() {
        let json = br#"{
            "jobs": [
                {
                    "databaseId": 98765,
                    "name": "build",
                    "status": "completed",
                    "conclusion": "success",
                    "startedAt": "2026-07-01T10:00:00Z",
                    "completedAt": "2026-07-01T10:03:00Z",
                    "url": "https://example.com/job/98765"
                }
            ]
        }"#;

        let resp: JobsResponse = serde_json::from_slice(json).expect("valid JobsResponse");
        assert_eq!(resp.jobs.len(), 1);
        assert_eq!(resp.jobs[0].database_id, 98765);
        assert_eq!(resp.jobs[0].name, "build");
    }

    #[test]
    fn test_should_deserialize_empty_jobs_response() {
        let json = br#"{"jobs": []}"#;
        let resp: JobsResponse = serde_json::from_slice(json).expect("valid");
        assert!(resp.jobs.is_empty());
    }

    #[test]
    fn test_should_convert_gc_job_to_job_data() {
        let job = GcJob {
            database_id: 100,
            name: "test".into(),
            status: "completed".into(),
            conclusion: Some("success".into()),
            started_at: Some("2026-07-01T10:00:00Z".into()),
            completed_at: Some("2026-07-01T10:03:00Z".into()),
            url: "https://example.com/job/100".into(),
        };

        let data = job.into_job_data();
        assert_eq!(data.id, 100);
        assert_eq!(data.name, "test");
        assert_eq!(data.status, "completed");
        assert_eq!(data.conclusion.as_deref(), Some("success"));
        assert!(data.started_at.is_some());
        assert!(data.completed_at.is_some());
    }

    #[test]
    fn test_should_convert_gc_job_with_null_timestamps() {
        let job = GcJob {
            database_id: 1,
            name: "queued-job".into(),
            status: "queued".into(),
            conclusion: None,
            started_at: None,
            completed_at: None,
            url: "https://example.com/job/1".into(),
        };

        let data = job.into_job_data();
        assert!(data.started_at.is_none());
        assert!(data.completed_at.is_none());
    }

    #[test]
    fn test_should_deserialize_run_with_null_conclusion() {
        let json = br#"{
            "databaseId": 1,
            "headBranch": "develop",
            "status": "in_progress",
            "conclusion": null,
            "createdAt": "2026-07-02T08:00:00Z",
            "updatedAt": "2026-07-02T08:00:00Z",
            "url": "https://example.com/1"
        }"#;

        let run: GcRun = serde_json::from_slice(json).expect("deserialize");
        assert!(run.conclusion.is_none());
        assert_eq!(run.status, "in_progress");
    }

    #[test]
    fn test_should_compute_report_from_runs() {
        #[derive(Debug, Deserialize)]
        #[allow(
            dead_code,
            reason = "Test fixture struct fields are deserialized but not all read"
        )]
        #[serde(rename_all = "camelCase")]
        struct TestReportRun {
            conclusion: Option<String>,
            created_at: String,
            updated_at: String,
        }

        let json = br#"[
            {"conclusion": "success", "createdAt": "2026-07-01T10:00:00Z", "updatedAt": "2026-07-01T10:05:00Z"},
            {"conclusion": "success", "createdAt": "2026-07-01T11:00:00Z", "updatedAt": "2026-07-01T11:03:00Z"},
            {"conclusion": "failure", "createdAt": "2026-07-01T12:00:00Z", "updatedAt": "2026-07-01T12:02:00Z"},
            {"conclusion": null, "createdAt": "2026-07-01T13:00:00Z", "updatedAt": "2026-07-01T13:01:00Z"}
        ]"#;

        let runs: Vec<TestReportRun> = serde_json::from_slice(json).expect("valid");
        assert_eq!(runs.len(), 4);

        let total = runs.len() as u64;
        let success: u64 = runs
            .iter()
            .filter(|r| r.conclusion.as_deref() == Some("success"))
            .count() as u64;
        #[allow(
            clippy::cast_precision_loss,
            reason = "Test values are small enough to fit in f64 without loss"
        )]
        let rate = success as f64 / total as f64;

        assert_eq!(total, 4);
        assert_eq!(success, 2);
        assert!((rate - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_should_count_all_failure_types_in_report_logic() {
        use std::collections::HashMap;

        let conclusions = [
            "success",
            "success",
            "failure",
            "failure",
            "startup_failure",
            "timed_out",
            "cancelled",
            "skipped",
            "neutral",
        ];

        let mut success_count: u64 = 0;
        let mut failure_counts: HashMap<String, u64> = HashMap::new();

        for conclusion in &conclusions {
            if *conclusion == "success" {
                success_count += 1;
            } else if !matches!(*conclusion, "cancelled" | "skipped" | "neutral") {
                *failure_counts.entry(conclusion.to_string()).or_insert(0) += 1;
            }
        }

        assert_eq!(success_count, 2);

        assert_eq!(failure_counts.get("failure"), Some(&2));
        assert_eq!(failure_counts.get("startup_failure"), Some(&1));
        assert_eq!(failure_counts.get("timed_out"), Some(&1));
        assert_eq!(failure_counts.get("cancelled"), None);
        assert_eq!(failure_counts.get("skipped"), None);
        assert_eq!(failure_counts.get("neutral"), None);

        let mut failures: Vec<_> = failure_counts.into_iter().collect();
        failures.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let top_failures: Vec<String> = failures.into_iter().map(|(k, _)| k).collect();

        assert_eq!(top_failures[0], "failure");
        assert!(top_failures.contains(&"startup_failure".to_string()));
        assert!(top_failures.contains(&"timed_out".to_string()));
        assert_eq!(top_failures.len(), 3);
    }
}
