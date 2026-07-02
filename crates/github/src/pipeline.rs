//! GitHub Pipeline 提供者实现。
//!
//! 通过 `gh run list` / `gh run view` CLI 实现 [`PipelineProvider`] trait。

use async_trait::async_trait;
use gitflow_cli_core::{
    CoreError, Result,
    pipeline::{JobData, PipelineProvider, PipelineReport, PipelineStatus, PipelineStatusEnum},
};
use serde::Deserialize;
use tracing::debug;

use crate::error::parse_gh_error;

/// `gh run list` 请求的 JSON 字段列表。
const PIPELINE_FIELDS: &str =
    "databaseId,headBranch,status,conclusion,createdAt,updatedAt,url";

/// 将 GitHub `gh run list` 返回的 status 字符串映射为 [`PipelineStatusEnum`]。
///
/// GitHub 返回小写状态：`queued`、`in_progress`、`completed`、`waiting`、
/// `requested`、`pending` 等。其中 `completed` 需要结合 `conclusion`
/// 判断最终结果。
fn gh_status_to_enum(status: &str, conclusion: Option<&str>) -> PipelineStatusEnum {
    match status {
        "in_progress" | "action_required" | "cancelled" => PipelineStatusEnum::Running,
        "completed" => match conclusion {
            Some("success") => PipelineStatusEnum::Success,
            Some("failure") | Some("startup_failure") | Some("timed_out") => {
                PipelineStatusEnum::Failed
            }
            Some("cancelled") => PipelineStatusEnum::Cancelled,
            Some("skipped") | Some("neutral") => PipelineStatusEnum::Pending,
            _ => PipelineStatusEnum::Running,
        },
        "queued" | "waiting" | "requested" | "pending" => PipelineStatusEnum::Pending,
        _ => PipelineStatusEnum::Running,
    }
}

/// GitHub 单次 run 的原始响应，用于反序列化。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GhRun {
    database_id: u64,
    head_branch: String,
    status: String,
    conclusion: Option<String>,
    created_at: String,
    updated_at: String,
    url: String,
}

impl GhRun {
    fn into_status(self) -> PipelineStatus {
        let created_at = chrono::DateTime::parse_from_rfc3339(&self.created_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now());
        let updated_at = chrono::DateTime::parse_from_rfc3339(&self.updated_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now());

        PipelineStatus {
            id: self.database_id,
            ref_name: self.head_branch,
            status: gh_status_to_enum(&self.status, self.conclusion.as_deref()),
            conclusion: self.conclusion,
            created_at,
            updated_at,
            url: self.url,
        }
    }
}

/// `gh run view --json jobs` 的包裹结构体。
///
/// GitHub 返回 `{"jobs": [...]}` 而非直接数组。
#[derive(Debug, Deserialize)]
struct JobsResponse {
    jobs: Vec<GhJob>,
}

/// GitHub 单次 job 的原始响应。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GhJob {
    database_id: u64,
    name: String,
    status: String,
    conclusion: Option<String>,
    started_at: Option<String>,
    completed_at: Option<String>,
    url: String,
}

impl GhJob {
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

/// GitHub Pipeline 提供者，通过 `gh` CLI 操作 CI/CD 流水线。
///
/// # Examples
///
/// ```no_run
/// use gitflow_cli_github::GitHubPipelineProvider;
///
/// let provider = GitHubPipelineProvider::new("octocat/hello-world");
/// ```
#[derive(Debug, Clone)]
pub struct GitHubPipelineProvider {
    /// GitHub `owner/repo`，如 `"byx-darwin/gitflow-cli"`。
    repo: String,
}

impl GitHubPipelineProvider {
    /// 创建新的 GitHub Pipeline 提供者。
    ///
    /// `repo` 格式为 `owner/repo`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> Self {
        Self { repo: repo.into() }
    }
}

#[async_trait]
impl PipelineProvider for GitHubPipelineProvider {
    async fn status(&self, branch: &str) -> Result<Vec<PipelineStatus>> {
        debug!(repo = %self.repo, branch = %branch, "spawning `gh run list`");

        let output = tokio::process::Command::new("gh")
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
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        let runs: Vec<GhRun> =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(runs.into_iter().map(GhRun::into_status).collect())
    }

    async fn logs(&self, pipeline_id: u64) -> Result<String> {
        debug!(repo = %self.repo, pipeline_id, "spawning `gh run view --log`");

        let output = tokio::process::Command::new("gh")
            .args(["run", "view"])
            .arg(pipeline_id.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .arg("--log")
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        String::from_utf8(output.stdout).map_err(|e| {
            CoreError::Platform(format!("Failed to decode log output as UTF-8: {e}"))
        })
    }

    async fn jobs(&self, pipeline_id: u64) -> Result<Vec<JobData>> {
        debug!(repo = %self.repo, pipeline_id, "spawning `gh run view --json jobs`");

        let output = tokio::process::Command::new("gh")
            .args(["run", "view"])
            .arg(pipeline_id.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .arg("--json")
            .arg("jobs")
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        let resp: JobsResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(resp.jobs.into_iter().map(GhJob::into_job_data).collect())
    }

    async fn report(&self, branch: &str, days: u32) -> Result<PipelineReport> {
        debug!(
            repo = %self.repo,
            branch = %branch,
            days,
            "spawning `gh run list` for report"
        );

        let output = tokio::process::Command::new("gh")
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
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gh_err}")));
        }

        // 使用最小结构体反序列化 report 所需字段
        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct ReportRun {
            conclusion: Option<String>,
            created_at: String,
            updated_at: String,
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
                } else if conclusion == "failure" {
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
                    total_duration_secs += duration as f64;
                    has_duration += 1;
                }
            }
        }

        let success_rate = if total_runs > 0 {
            success_count as f64 / total_runs as f64
        } else {
            0.0
        };

        let avg_duration_secs = if has_duration > 0 {
            total_duration_secs / has_duration as f64
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
    fn test_should_construct_github_pipeline_provider() {
        let provider = GitHubPipelineProvider::new("octocat/hello-world");
        assert_eq!(provider.repo, "octocat/hello-world");
    }

    #[test]
    fn test_should_construct_github_pipeline_provider_from_string() {
        let repo = String::from("octocat/hello-world");
        let provider = GitHubPipelineProvider::new(repo);
        assert_eq!(provider.repo, "octocat/hello-world");
    }

    #[test]
    fn test_should_debug_format_provider() {
        let provider = GitHubPipelineProvider::new("octocat/hello-world");
        let debug = format!("{provider:?}");
        assert!(debug.contains("GitHubPipelineProvider"));
        assert!(debug.contains("octocat/hello-world"));
    }

    #[test]
    fn test_should_clone_github_pipeline_provider() {
        let original = GitHubPipelineProvider::new("owner/repo");
        let cloned = original.clone();
        assert_eq!(original.repo, cloned.repo);
    }

    #[test]
    fn test_should_deserialize_gh_run_from_json() {
        let json = br#"{
            "databaseId": 12345,
            "headBranch": "main",
            "status": "completed",
            "conclusion": "success",
            "createdAt": "2026-07-01T10:00:00Z",
            "updatedAt": "2026-07-01T10:05:30Z",
            "url": "https://github.com/example/repo/actions/runs/12345"
        }"#;

        let run: GhRun = serde_json::from_slice(json).expect("valid GhRun JSON");
        assert_eq!(run.database_id, 12345);
        assert_eq!(run.head_branch, "main");
        assert_eq!(run.status, "completed");
        assert_eq!(run.conclusion.as_deref(), Some("success"));
        assert_eq!(
            run.url,
            "https://github.com/example/repo/actions/runs/12345"
        );
    }

    #[test]
    fn test_should_deserialize_gh_run_list_from_json_array() {
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

        let runs: Vec<GhRun> = serde_json::from_slice(json).expect("valid GhRun list");
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].database_id, 1);
        assert_eq!(runs[1].database_id, 2);
    }

    #[test]
    fn test_should_deserialize_empty_run_list() {
        let json = b"[]";
        let runs: Vec<GhRun> = serde_json::from_slice(json).expect("empty list");
        assert!(runs.is_empty());
    }

    #[test]
    fn test_should_convert_gh_run_to_pipeline_status() {
        let run = GhRun {
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
    fn test_should_map_gh_status_completed_success() {
        assert_eq!(
            gh_status_to_enum("completed", Some("success")),
            PipelineStatusEnum::Success
        );
    }

    #[test]
    fn test_should_map_gh_status_completed_failure() {
        assert_eq!(
            gh_status_to_enum("completed", Some("failure")),
            PipelineStatusEnum::Failed
        );
        assert_eq!(
            gh_status_to_enum("completed", Some("startup_failure")),
            PipelineStatusEnum::Failed
        );
        assert_eq!(
            gh_status_to_enum("completed", Some("timed_out")),
            PipelineStatusEnum::Failed
        );
    }

    #[test]
    fn test_should_map_gh_status_completed_cancelled() {
        assert_eq!(
            gh_status_to_enum("completed", Some("cancelled")),
            PipelineStatusEnum::Cancelled
        );
    }

    #[test]
    fn test_should_map_gh_status_completed_skipped() {
        assert_eq!(
            gh_status_to_enum("completed", Some("skipped")),
            PipelineStatusEnum::Pending
        );
        assert_eq!(
            gh_status_to_enum("completed", Some("neutral")),
            PipelineStatusEnum::Pending
        );
    }

    #[test]
    fn test_should_map_gh_status_in_progress() {
        assert_eq!(
            gh_status_to_enum("in_progress", None),
            PipelineStatusEnum::Running
        );
        assert_eq!(
            gh_status_to_enum("action_required", None),
            PipelineStatusEnum::Running
        );
    }

    #[test]
    fn test_should_map_gh_status_queued() {
        assert_eq!(
            gh_status_to_enum("queued", None),
            PipelineStatusEnum::Pending
        );
        assert_eq!(
            gh_status_to_enum("waiting", None),
            PipelineStatusEnum::Pending
        );
        assert_eq!(
            gh_status_to_enum("requested", None),
            PipelineStatusEnum::Pending
        );
    }

    #[test]
    fn test_should_map_gh_status_unknown() {
        assert_eq!(
            gh_status_to_enum("some_unknown_status", None),
            PipelineStatusEnum::Running
        );
    }

    #[test]
    fn test_should_map_completed_with_unknown_conclusion() {
        // completed + unknown conclusion -> Running
        assert_eq!(
            gh_status_to_enum("completed", Some("weird")),
            PipelineStatusEnum::Running
        );
    }

    #[test]
    fn test_should_map_completed_with_none_conclusion() {
        // completed + None conclusion -> Running
        assert_eq!(
            gh_status_to_enum("completed", None),
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

        let resp: JobsResponse =
            serde_json::from_slice(json).expect("valid JobsResponse");
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
    fn test_should_convert_gh_job_to_job_data() {
        let job = GhJob {
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
    fn test_should_convert_gh_job_with_null_timestamps() {
        let job = GhJob {
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

        let run: GhRun = serde_json::from_slice(json).expect("deserialize");
        assert!(run.conclusion.is_none());
        assert_eq!(run.status, "in_progress");
    }

    #[test]
    fn test_should_compute_report_from_runs() {
        // 模拟 report 使用的最小结构体
        #[derive(Debug, Deserialize)]
        #[allow(dead_code)]
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

        let runs: Vec<TestReportRun> =
            serde_json::from_slice(json).expect("valid");
        assert_eq!(runs.len(), 4);

        let total = runs.len() as u64;
        let success: u64 = runs
            .iter()
            .filter(|r| r.conclusion.as_deref() == Some("success"))
            .count() as u64;
        let rate = success as f64 / total as f64;

        assert_eq!(total, 4);
        assert_eq!(success, 2);
        assert!((rate - 0.5).abs() < f64::EPSILON);
    }
}
