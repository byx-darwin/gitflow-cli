//! GitHub Pipeline 提供者实现。
//!
//! 通过 `gh run list` / `gh run view` CLI 实现 [`PipelineProvider`] trait。

use async_trait::async_trait;
use gitflow_core::{
    CoreError, Result,
    pipeline::{JobData, PipelineProvider, PipelineReport, PipelineStatus, PipelineStatusEnum},
};
use serde::Deserialize;
use tracing::debug;

use crate::{
    error::parse_gh_error,
    runner::{CommandRunner, RealCommandRunner},
};

/// `gh run list` 请求的 JSON 字段列表。
const PIPELINE_FIELDS: &str = "databaseId,headBranch,status,conclusion,createdAt,updatedAt,url";

/// 将 GitHub `gh run list` 返回的 status 字符串映射为 [`PipelineStatusEnum`]。
///
/// GitHub 返回小写状态：`queued`、`in_progress`、`completed`、`waiting`、
/// `requested`、`pending` 等。其中 `completed` 需要结合 `conclusion`
/// 判断最终结果。
fn gh_status_to_enum(status: &str, conclusion: Option<&str>) -> PipelineStatusEnum {
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
            .ok()
            .map_or_else(chrono::Utc::now, |dt| dt.with_timezone(&chrono::Utc));
        let updated_at = chrono::DateTime::parse_from_rfc3339(&self.updated_at)
            .ok()
            .map_or_else(chrono::Utc::now, |dt| dt.with_timezone(&chrono::Utc));

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
/// 该结构体通过调用 `gh` CLI 实现 [`PipelineProvider`] trait 的所有方法，
/// 使上层命令能够以统一的方式查看 GitHub Actions 流水线状态。
///
/// 命令执行通过 [`CommandRunner`] 抽象，生产环境默认使用
/// [`RealCommandRunner`]，测试可注入自定义 runner 以模拟成功或失败场景。
///
/// # Examples
///
/// ```no_run
/// use gitflow_github::GitHubPipelineProvider;
///
/// let provider = GitHubPipelineProvider::new("octocat/hello-world");
/// ```
#[derive(Debug, Clone)]
pub struct GitHubPipelineProvider<R: CommandRunner = RealCommandRunner> {
    /// GitHub `owner/repo`，如 `"byx-darwin/gitflow-cli"`。
    repo: String,
    /// 用于执行 `gh` CLI 命令的 runner。
    runner: R,
}

impl GitHubPipelineProvider<RealCommandRunner> {
    /// 创建新的 GitHub Pipeline 提供者，使用真实的进程执行器。
    ///
    /// `repo` 格式为 `owner/repo`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> Self {
        Self {
            repo: repo.into(),
            runner: RealCommandRunner,
        }
    }

    /// Create a new provider from a shared [`Session`].
    ///
    /// This enables state reuse across multiple operations in workflow chains.
    #[must_use]
    pub fn with_session(session: &gitflow_core::Session) -> Self {
        Self {
            repo: session.repo.clone(),
            runner: RealCommandRunner,
        }
    }
}

impl<R: CommandRunner> GitHubPipelineProvider<R> {
    /// 使用自定义 [`CommandRunner`] 创建提供者。
    ///
    /// 主要用于测试，可注入模拟 runner 以控制 `gh` CLI 的输出。
    /// `repo` 格式为 `owner/repo`。
    #[must_use]
    pub fn with_runner(repo: impl Into<String>, runner: R) -> Self {
        Self {
            repo: repo.into(),
            runner,
        }
    }
}

impl<R: CommandRunner + 'static> GitHubPipelineProvider<R> {
    /// 为一批失败类 run 归因到具体失败 job 名称，用于 [`PipelineReport::top_failures`]。
    ///
    /// 只对结论落在失败类（见 [`is_failure_conclusion`]）的 run 发起 `jobs` 查询，
    /// 成功和非失败终态（`cancelled`/`skipped`/`neutral`）的 run 不消耗额外 API
    /// 调用。若某次 run 的 job 级数据无法获取或其中没有失败类 job，则回退为该
    /// run 的通用 `conclusion` 字符串（例如 `"failure"`），确保该样本仍计入
    /// 统计而不是被静默丢弃。
    async fn attribute_top_failures(&self, runs: &[ReportRun]) -> Vec<String> {
        let mut failure_counts: std::collections::HashMap<String, u64> =
            std::collections::HashMap::new();

        for run in runs {
            let Some(conclusion) = run.conclusion.as_deref() else {
                continue;
            };
            if !is_failure_conclusion(conclusion) {
                continue;
            }

            let label = match self.jobs(run.database_id).await {
                Ok(jobs) => jobs
                    .iter()
                    .find(|job| job.conclusion.as_deref().is_some_and(is_failure_conclusion))
                    .map_or_else(|| conclusion.to_owned(), |job| job.name.clone()),
                Err(err) => {
                    debug!(
                        repo = %self.repo,
                        pipeline_id = run.database_id,
                        error = %err,
                        "failed to fetch jobs for failure attribution, falling back to generic conclusion"
                    );
                    conclusion.to_owned()
                }
            };

            *failure_counts.entry(label).or_insert(0) += 1;
        }

        // 按失败次数降序排列；次数相同时按标签字母序，保证输出稳定。
        let mut failures: Vec<_> = failure_counts.into_iter().collect();
        failures.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        failures.into_iter().map(|(label, _)| label).collect()
    }
}

/// `gh run list` 的 report 统计所需最小字段集。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReportRun {
    database_id: u64,
    conclusion: Option<String>,
    created_at: String,
    updated_at: String,
}

/// 判断 run/job 的 `conclusion` 字符串是否属于失败类结论。
///
/// 与成功（`success`）、非失败终态（`cancelled`/`skipped`/`neutral`）区分开；
/// 其余（`failure`/`startup_failure`/`timed_out` 及任何未知值）视为失败。
fn is_failure_conclusion(conclusion: &str) -> bool {
    !matches!(conclusion, "success" | "cancelled" | "skipped" | "neutral")
}

/// 为 [`GitHubPipelineProvider::report`] 聚合每次运行的成功数与耗时指标。
///
/// 失败归因（`top_failures`）由 [`GitHubPipelineProvider::attribute_top_failures`]
/// 单独处理，因为它需要按需发起额外的 `jobs` API 调用。
///
/// 返回 `(success_count, total_duration_secs, runs_with_duration)`。
fn aggregate_report_metrics(runs: &[ReportRun]) -> (u64, f64, u64) {
    let mut success_count: u64 = 0;
    let mut total_duration_secs: f64 = 0.0;
    let mut has_duration: u64 = 0;

    for run in runs {
        if run.conclusion.as_deref() == Some("success") {
            success_count += 1;
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

    (success_count, total_duration_secs, has_duration)
}

#[async_trait]
impl<R: CommandRunner + 'static> PipelineProvider for GitHubPipelineProvider<R> {
    async fn status(&self, branch: &str) -> Result<Vec<PipelineStatus>> {
        debug!(repo = %self.repo, branch = %branch, "spawning `gh run list`");

        let output = self
            .runner
            .run(
                "gh",
                &[
                    "run",
                    "list",
                    "--branch",
                    branch,
                    "--repo",
                    &self.repo,
                    "--json",
                    PIPELINE_FIELDS,
                    "--limit",
                    "30",
                ],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            return Err(parse_gh_error(&output.stderr).into());
        }

        let runs: Vec<GhRun> =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(runs.into_iter().map(GhRun::into_status).collect())
    }

    async fn logs(&self, pipeline_id: u64) -> Result<String> {
        debug!(repo = %self.repo, pipeline_id, "spawning `gh run view --log`");

        let run_id = pipeline_id.to_string();
        let output = self
            .runner
            .run(
                "gh",
                &["run", "view", &run_id, "--repo", &self.repo, "--log"],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            return Err(parse_gh_error(&output.stderr).into());
        }

        String::from_utf8(output.stdout)
            .map_err(|e| CoreError::Platform(format!("Failed to decode log output as UTF-8: {e}")))
    }

    async fn jobs(&self, pipeline_id: u64) -> Result<Vec<JobData>> {
        debug!(repo = %self.repo, pipeline_id, "spawning `gh run view --json jobs`");

        let run_id = pipeline_id.to_string();
        let output = self
            .runner
            .run(
                "gh",
                &[
                    "run", "view", &run_id, "--repo", &self.repo, "--json", "jobs",
                ],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            return Err(parse_gh_error(&output.stderr).into());
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

        let output = self
            .runner
            .run(
                "gh",
                &[
                    "run",
                    "list",
                    "--branch",
                    branch,
                    "--repo",
                    &self.repo,
                    "--json",
                    "databaseId,conclusion,createdAt,updatedAt",
                    "--limit",
                    "100",
                ],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            return Err(parse_gh_error(&output.stderr).into());
        }

        let all_runs: Vec<ReportRun> =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        // Filter to the requested time window. Unparsable timestamps are kept
        // (conservative: never drop runs because of a date format surprise).
        let cutoff = chrono::Utc::now() - chrono::Duration::days(i64::from(days));
        let runs: Vec<ReportRun> = all_runs
            .into_iter()
            .filter(|run| {
                chrono::DateTime::parse_from_rfc3339(&run.created_at)
                    .map_or(true, |dt| dt.with_timezone(&chrono::Utc) >= cutoff)
            })
            .collect();

        // Only runs that have reached a terminal state carry a `conclusion`
        // (GitHub sets it once `status == "completed"`). An in-progress run
        // serializes with `conclusion: null` and must not inflate the
        // denominator used for `success_rate`.
        let total_runs = runs.iter().filter(|r| r.conclusion.is_some()).count() as u64;

        let (success_count, total_duration_secs, has_duration) = aggregate_report_metrics(&runs);

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
            total_duration_secs / has_duration as f64
        } else {
            0.0
        };

        // 按失败次数降序取 top 失败归因标签（job 名称，若无法归因则回退为
        // 通用 conclusion 字符串）。仅对失败类 run 发起额外的 jobs 查询，
        // 避免对每个 run 都调用 API。
        let top_failures = self.attribute_top_failures(&runs).await;

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
    use crate::runner::MockCommandRunner;

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
        // 验证 report 方法中的失败计数逻辑：
        // "failure"、"startup_failure"、"timed_out" 都应被计入 top_failures，
        // 而 "cancelled"、"skipped"、"neutral" 不应被计入。
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

        // 验证 top_failures 排序（按数量降序）
        let mut failures: Vec<_> = failure_counts.into_iter().collect();
        failures.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let top_failures: Vec<String> = failures.into_iter().map(|(k, _)| k).collect();

        // "failure" 有 2 次排第一，其余各 1 次按字母序
        assert_eq!(top_failures[0], "failure");
        assert!(top_failures.contains(&"startup_failure".to_string()));
        assert!(top_failures.contains(&"timed_out".to_string()));
        assert_eq!(top_failures.len(), 3);
    }

    #[tokio::test]
    async fn test_should_exclude_in_progress_runs_from_report_total_runs() {
        // 4 runs in the report window: 2 success, 1 failure, 1 still in-progress
        // (GitHub only sets `conclusion` once a run is `completed`, so an
        // in-progress run serializes with `"conclusion": null`).
        let now = chrono::Utc::now();
        let ts = |offset_secs: i64| (now - chrono::Duration::seconds(offset_secs)).to_rfc3339();

        let json = format!(
            r#"[
                {{"databaseId": 1, "conclusion": "success", "createdAt": "{}", "updatedAt": "{}"}},
                {{"databaseId": 2, "conclusion": "success", "createdAt": "{}", "updatedAt": "{}"}},
                {{"databaseId": 3, "conclusion": "failure", "createdAt": "{}", "updatedAt": "{}"}},
                {{"databaseId": 4, "conclusion": null, "createdAt": "{}", "updatedAt": "{}"}}
            ]"#,
            ts(600),
            ts(300),
            ts(500),
            ts(200),
            ts(400),
            ts(100),
            ts(60),
            ts(30),
        );

        let runner = MockCommandRunner::success(&json);
        let provider = GitHubPipelineProvider::with_runner("owner/repo", runner);

        let report = provider
            .report("main", 7)
            .await
            .expect("report should succeed");

        // Only 3 of the 4 runs have reached a terminal state (conclusion is
        // Some); the in-progress run (conclusion: null) must be excluded
        // from total_runs, not just from success/failure counts.
        assert_eq!(report.total_runs, 3);
        assert!((report.success_rate - (2.0 / 3.0)).abs() < f64::EPSILON);
    }

    // --- Failure-path tests using an injected MockCommandRunner ---

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_fails_for_status() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not found"}"#, 256);
        let provider = GitHubPipelineProvider::with_runner("owner/repo", runner);

        let result = provider.status("main").await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_status() {
        let runner = MockCommandRunner::success("invalid json");
        let provider = GitHubPipelineProvider::with_runner("owner/repo", runner);

        let result = provider.status("main").await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_fails_for_logs() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not found"}"#, 256);
        let provider = GitHubPipelineProvider::with_runner("owner/repo", runner);

        let result = provider.logs(42).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_fails_for_jobs() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not found"}"#, 256);
        let provider = GitHubPipelineProvider::with_runner("owner/repo", runner);

        let result = provider.jobs(42).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_jobs() {
        let runner = MockCommandRunner::success("invalid json");
        let provider = GitHubPipelineProvider::with_runner("owner/repo", runner);

        let result = provider.jobs(42).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_fails_for_report() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not found"}"#, 256);
        let provider = GitHubPipelineProvider::with_runner("owner/repo", runner);

        let result = provider.report("main", 7).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_report() {
        let runner = MockCommandRunner::success("invalid json");
        let provider = GitHubPipelineProvider::with_runner("owner/repo", runner);

        let result = provider.report("main", 7).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Serialization(_)
        ));
    }

    // --- Job-level failure attribution (issue #289) ---

    #[tokio::test]
    async fn test_should_attribute_top_failures_to_job_names_not_generic_conclusion() {
        use crate::runner::SequencedMockCommandRunner;

        let now = chrono::Utc::now();
        let ts = |offset_secs: i64| (now - chrono::Duration::seconds(offset_secs)).to_rfc3339();

        // Two failed runs (databaseId 10 and 11) plus one success run.
        let run_list_json = format!(
            r#"[
                {{"databaseId": 10, "conclusion": "failure", "createdAt": "{}", "updatedAt": "{}"}},
                {{"databaseId": 11, "conclusion": "failure", "createdAt": "{}", "updatedAt": "{}"}},
                {{"databaseId": 12, "conclusion": "success", "createdAt": "{}", "updatedAt": "{}"}}
            ]"#,
            ts(600),
            ts(500),
            ts(400),
            ts(300),
            ts(200),
            ts(100),
        );

        // Both failed runs' `jobs` responses point at the same failing job name,
        // so it should be attributed by name rather than the generic "failure"
        // conclusion string.
        let jobs_json = r#"{
            "jobs": [
                {
                    "databaseId": 1,
                    "name": "Test (windows-latest)",
                    "status": "completed",
                    "conclusion": "success",
                    "startedAt": "2026-07-01T10:00:00Z",
                    "completedAt": "2026-07-01T10:01:00Z",
                    "url": "https://example.com/job/1"
                },
                {
                    "databaseId": 2,
                    "name": "Test (macos-latest)",
                    "status": "completed",
                    "conclusion": "failure",
                    "startedAt": "2026-07-01T10:00:00Z",
                    "completedAt": "2026-07-01T10:02:00Z",
                    "url": "https://example.com/job/2"
                }
            ]
        }"#;

        let runner = SequencedMockCommandRunner::from_results(&[
            (true, &run_list_json),
            (true, jobs_json),
            (true, jobs_json),
        ]);
        let provider = GitHubPipelineProvider::with_runner("owner/repo", runner);

        let report = provider
            .report("main", 7)
            .await
            .expect("report should succeed");

        assert_eq!(report.total_runs, 3);
        assert_eq!(report.top_failures, vec!["Test (macos-latest)".to_string()]);
        assert!(!report.top_failures.contains(&"failure".to_string()));
    }

    #[tokio::test]
    async fn test_should_fall_back_to_generic_conclusion_when_jobs_fetch_fails() {
        use crate::runner::SequencedMockCommandRunner;

        let now = chrono::Utc::now();
        let ts = |offset_secs: i64| (now - chrono::Duration::seconds(offset_secs)).to_rfc3339();

        let run_list_json = format!(
            r#"[{{"databaseId": 20, "conclusion": "failure", "createdAt": "{}", "updatedAt": "{}"}}]"#,
            ts(600),
            ts(500),
        );

        // The `jobs` call for the failed run fails (e.g. permission error or
        // transient API failure); attribution must degrade gracefully to the
        // run's generic conclusion instead of panicking or dropping the run.
        let runner = SequencedMockCommandRunner::from_results(&[
            (true, &run_list_json),
            (false, r#"{"message": "Not found"}"#),
        ]);
        let provider = GitHubPipelineProvider::with_runner("owner/repo", runner);

        let report = provider
            .report("main", 7)
            .await
            .expect("report should succeed despite jobs fetch failure");

        assert_eq!(report.total_runs, 1);
        assert_eq!(report.top_failures, vec!["failure".to_string()]);
    }

    #[tokio::test]
    async fn test_should_not_call_jobs_api_for_non_failure_runs() {
        use crate::runner::SequencedMockCommandRunner;

        let now = chrono::Utc::now();
        let ts = |offset_secs: i64| (now - chrono::Duration::seconds(offset_secs)).to_rfc3339();

        // success / cancelled / skipped / neutral runs must never trigger a
        // `jobs` API call — only one response (the run list) is queued, so
        // the test panics via SequencedMockCommandRunner if attribution tries
        // to fetch jobs for any of them.
        let run_list_json = format!(
            r#"[
                {{"databaseId": 30, "conclusion": "success", "createdAt": "{}", "updatedAt": "{}"}},
                {{"databaseId": 31, "conclusion": "cancelled", "createdAt": "{}", "updatedAt": "{}"}},
                {{"databaseId": 32, "conclusion": "skipped", "createdAt": "{}", "updatedAt": "{}"}},
                {{"databaseId": 33, "conclusion": "neutral", "createdAt": "{}", "updatedAt": "{}"}}
            ]"#,
            ts(800),
            ts(700),
            ts(600),
            ts(500),
            ts(400),
            ts(300),
            ts(200),
            ts(100),
        );

        let runner = SequencedMockCommandRunner::from_results(&[(true, &run_list_json)]);
        let provider = GitHubPipelineProvider::with_runner("owner/repo", runner);

        let report = provider
            .report("main", 7)
            .await
            .expect("report should succeed");

        assert!(report.top_failures.is_empty());
    }
}
