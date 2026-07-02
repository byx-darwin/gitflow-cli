//! GitLab Pipeline (CI/CD) 提供者实现。
//!
//! 通过 `glab ci` CLI 命令实现 [`PipelineProvider`] trait，支持 Pipeline
//! 状态查看、日志获取、任务列表和报告生成。
//!
//! glab 命令映射：
//! - `status` → `glab ci list`
//! - `logs` → `glab ci trace`
//! - `jobs` → `glab ci list --output json`
//! - `report` → 调用 `status` 后计算统计

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use gitflow_cli_core::{
    CoreError, Result,
    pipeline::{JobData, PipelineProvider, PipelineReport, PipelineStatus, PipelineStatusEnum},
};
use serde::Deserialize;
use tracing::debug;

use crate::error::parse_glab_error;

/// GitLab Pipeline 提供者，通过 `glab ci` 操作 CI/CD 管线。
///
/// # Examples
///
/// ```no_run
/// use gitflow_cli_gitlab::GitLabPipelineProvider;
///
/// let provider = GitLabPipelineProvider::new("gitlab-org/gitlab");
/// ```
#[derive(Debug, Clone)]
pub struct GitLabPipelineProvider {
    /// GitLab `namespace/project`。
    repo: String,
}

impl GitLabPipelineProvider {
    /// 创建新的 GitLab Pipeline 提供者。
    ///
    /// `repo` 格式为 `namespace/project`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> Self {
        Self { repo: repo.into() }
    }
}

// ── 中间 API 响应类型 ──────────────────────────────────────────────

/// `glab ci list --output json` 返回的 Pipeline JSON 结构。
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code, reason = "Used for deserialization; not all fields are read")]
struct PipelineApiResponse {
    id: u64,
    #[serde(default)]
    ref_name: Option<String>,
    /// glab may use `ref` instead of `ref_name`
    #[serde(alias = "ref")]
    #[serde(default)]
    git_ref: Option<String>,
    #[serde(default)]
    status: String,
    #[serde(default)]
    created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    updated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    web_url: Option<String>,
    /// Some glab versions include a `sha` field
    #[serde(default)]
    sha: Option<String>,
}

impl PipelineApiResponse {
    fn effective_ref(&self) -> String {
        self.ref_name
            .clone()
            .or_else(|| self.git_ref.clone())
            .unwrap_or_default()
    }
}

fn parse_pipeline_status(status: &str) -> PipelineStatusEnum {
    match status {
        "running" => PipelineStatusEnum::Running,
        "success" => PipelineStatusEnum::Success,
        "failed" => PipelineStatusEnum::Failed,
        "canceled" | "cancelled" => PipelineStatusEnum::Cancelled,
        _ => PipelineStatusEnum::Pending,
    }
}

impl From<PipelineApiResponse> for PipelineStatus {
    fn from(api: PipelineApiResponse) -> Self {
        let status_enum = parse_pipeline_status(&api.status);
        let conclusion = if api.status == "success" {
            Some("success".into())
        } else if api.status == "failed" {
            Some("failure".into())
        } else if api.status == "canceled" || api.status == "cancelled" {
            Some("cancelled".into())
        } else {
            None
        };

        Self {
            id: api.id,
            ref_name: api.effective_ref(),
            status: status_enum,
            conclusion,
            created_at: api.created_at.unwrap_or_else(Utc::now),
            updated_at: api.updated_at.unwrap_or_else(Utc::now),
            url: api.web_url.unwrap_or_default(),
        }
    }
}

/// `glab ci view --output json` 返回的 Job JSON 结构。
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code, reason = "Used for deserialization; not all fields are read")]
struct JobApiResponse {
    id: u64,
    #[serde(default)]
    name: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    stage: Option<String>,
    #[serde(default)]
    started_at: Option<DateTime<Utc>>,
    #[serde(default)]
    finished_at: Option<DateTime<Utc>>,
    #[serde(default)]
    web_url: Option<String>,
}

impl From<JobApiResponse> for JobData {
    fn from(api: JobApiResponse) -> Self {
        let conclusion = if api.status == "success" {
            Some("success".into())
        } else if api.status == "failed" {
            Some("failure".into())
        } else if api.status == "canceled" || api.status == "cancelled" {
            Some("cancelled".into())
        } else {
            None
        };

        Self {
            id: api.id,
            name: api.name,
            status: api.status,
            conclusion,
            started_at: api.started_at,
            completed_at: api.finished_at,
            url: api.web_url.unwrap_or_default(),
        }
    }
}

/// Helper struct for parsing `glab ci view` output that may contain
/// a pipeline object with an embedded `jobs` field.
#[derive(Debug, Deserialize)]
struct PipelineWithJobs {
    #[serde(default)]
    jobs: Vec<JobApiResponse>,
}

// ── trait 实现 ──────────────────────────────────────────────────────

#[async_trait]
impl PipelineProvider for GitLabPipelineProvider {
    /// 获取指定分支的 Pipeline 状态列表。
    ///
    /// 调用 `glab ci list --ref <branch> --output json`。
    async fn status(&self, branch: &str) -> Result<Vec<PipelineStatus>> {
        debug!(repo = %self.repo, branch, "spawning `glab ci list`");

        let output = tokio::process::Command::new("glab")
            .args(["ci", "list"])
            .arg("--repo")
            .arg(&self.repo)
            .arg("--ref")
            .arg(branch)
            .arg("--output")
            .arg("json")
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab ci list: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        let api_responses: Vec<PipelineApiResponse> =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_responses
            .into_iter()
            .map(PipelineStatus::from)
            .collect())
    }

    /// 获取指定 Pipeline 的日志。
    ///
    /// 调用 `glab ci trace <pipeline_id>` 或 `glab ci view <pipeline_id>`。
    async fn logs(&self, pipeline_id: u64) -> Result<String> {
        debug!(repo = %self.repo, pipeline_id, "spawning `glab ci trace`");

        let output = tokio::process::Command::new("glab")
            .args(["ci", "trace"])
            .arg(pipeline_id.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab ci trace: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    /// 获取指定 Pipeline 的 Job 列表。
    ///
    /// 调用 `glab ci view <pipeline_id> --output json`。
    async fn jobs(&self, pipeline_id: u64) -> Result<Vec<JobData>> {
        debug!(repo = %self.repo, pipeline_id, "spawning `glab ci view`");

        let output = tokio::process::Command::new("glab")
            .args(["ci", "view"])
            .arg(pipeline_id.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .arg("--output")
            .arg("json")
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab ci view: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        // glab ci view may return the pipeline object with embedded jobs,
        // or a jobs list depending on version. Try parsing as jobs first.
        if let Ok(jobs) = serde_json::from_slice::<Vec<JobApiResponse>>(&output.stdout) {
            return Ok(jobs.into_iter().map(JobData::from).collect());
        }

        // If that fails, try parsing as a pipeline with jobs field
        let pipeline: PipelineWithJobs =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;
        Ok(pipeline.jobs.into_iter().map(JobData::from).collect())
    }

    /// 生成指定分支最近 `days` 天的 Pipeline 报告。
    ///
    /// 调用 `status` 获取 Pipeline 列表，然后计算统计信息。
    async fn report(&self, branch: &str, days: u32) -> Result<PipelineReport> {
        debug!(
            repo = %self.repo,
            branch,
            days,
            "generating pipeline report"
        );

        let pipelines = self.status(branch).await?;

        // Filter by date range
        let cutoff = Utc::now() - chrono::Duration::days(i64::from(days));
        let recent: Vec<&PipelineStatus> = pipelines
            .iter()
            .filter(|p| p.created_at >= cutoff)
            .collect();

        let total_runs = recent.len() as u64;
        if total_runs == 0 {
            return Ok(PipelineReport {
                total_runs: 0,
                success_rate: 0.0,
                avg_duration_secs: 0.0,
                top_failures: vec![],
            });
        }

        let success_count = recent
            .iter()
            .filter(|p| matches!(p.status, PipelineStatusEnum::Success))
            .count() as u64;

        let success_rate = {
            #[allow(
                clippy::cast_precision_loss,
                reason = "Pipeline counts never exceed f64 precision"
            )]
            {
                success_count as f64 / total_runs as f64
            }
        };

        // Calculate average duration
        #[allow(
            clippy::cast_precision_loss,
            reason = "Duration values never exceed f64 precision"
        )]
        let durations: Vec<f64> = recent
            .iter()
            .map(|p| (p.updated_at - p.created_at).num_seconds().max(0) as f64)
            .filter(|d| *d > 0.0)
            .collect();

        let avg_duration_secs = if durations.is_empty() {
            0.0
        } else {
            #[allow(
                clippy::cast_precision_loss,
                reason = "Duration length never exceeds f64 precision"
            )]
            {
                durations.iter().sum::<f64>() / durations.len() as f64
            }
        };

        // Find top failures (by ref)
        let mut failure_refs: Vec<String> = recent
            .iter()
            .filter(|p| matches!(p.status, PipelineStatusEnum::Failed))
            .map(|p| p.ref_name.clone())
            .collect();
        failure_refs.sort();
        failure_refs.dedup();

        Ok(PipelineReport {
            total_runs,
            success_rate,
            avg_duration_secs,
            top_failures: failure_refs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_construct_gitlab_pipeline_provider() {
        let provider = GitLabPipelineProvider::new("gitlab-org/gitlab");
        assert_eq!(provider.repo, "gitlab-org/gitlab");
    }

    #[test]
    fn test_should_construct_gitlab_pipeline_provider_from_string() {
        let repo = String::from("gitlab-org/gitlab");
        let provider = GitLabPipelineProvider::new(repo);
        assert_eq!(provider.repo, "gitlab-org/gitlab");
    }

    #[test]
    fn test_should_debug_format_pipeline_provider() {
        let provider = GitLabPipelineProvider::new("owner/repo");
        let debug = format!("{provider:?}");
        assert!(debug.contains("GitLabPipelineProvider"));
        assert!(debug.contains("owner/repo"));
    }

    #[test]
    fn test_should_clone_gitlab_pipeline_provider() {
        let original = GitLabPipelineProvider::new("owner/repo");
        let cloned = original.clone();
        assert_eq!(original.repo, cloned.repo);
    }

    // --- API response deserialization tests ---

    #[test]
    fn test_should_deserialize_pipeline_api_response() {
        let json = br#"{
            "id": 12345,
            "ref_name": "main",
            "status": "success",
            "created_at": "2026-01-15T09:30:00Z",
            "updated_at": "2026-01-15T09:35:00Z",
            "web_url": "https://gitlab.com/gitlab-org/gitlab/-/pipelines/12345"
        }"#;

        let api: PipelineApiResponse =
            serde_json::from_slice(json).expect("valid PipelineApiResponse");
        assert_eq!(api.id, 12345);
        assert_eq!(api.status, "success");
        assert_eq!(api.effective_ref(), "main");
    }

    #[test]
    fn test_should_deserialize_pipeline_with_ref_alias() {
        let json = br#"{
            "id": 12345,
            "ref": "main",
            "status": "success",
            "created_at": "2026-01-15T09:30:00Z",
            "updated_at": "2026-01-15T09:35:00Z"
        }"#;

        let api: PipelineApiResponse =
            serde_json::from_slice(json).expect("valid PipelineApiResponse");
        assert_eq!(api.effective_ref(), "main");
    }

    #[test]
    fn test_should_convert_pipeline_to_status() {
        let api = PipelineApiResponse {
            id: 100,
            ref_name: Some("main".into()),
            git_ref: None,
            status: "success".into(),
            created_at: Some("2026-01-01T00:00:00Z".parse().expect("valid date")),
            updated_at: Some("2026-01-01T00:05:00Z".parse().expect("valid date")),
            web_url: Some("https://gitlab.com/org/project/-/pipelines/100".into()),
            sha: None,
        };

        let status: PipelineStatus = api.into();
        assert_eq!(status.id, 100);
        assert_eq!(status.ref_name, "main");
        assert_eq!(status.status, PipelineStatusEnum::Success);
        assert_eq!(status.conclusion.as_deref(), Some("success"));
    }

    #[test]
    fn test_should_parse_pipeline_status_enum() {
        assert_eq!(parse_pipeline_status("running"), PipelineStatusEnum::Running);
        assert_eq!(parse_pipeline_status("success"), PipelineStatusEnum::Success);
        assert_eq!(parse_pipeline_status("failed"), PipelineStatusEnum::Failed);
        assert_eq!(parse_pipeline_status("canceled"), PipelineStatusEnum::Cancelled);
        assert_eq!(parse_pipeline_status("cancelled"), PipelineStatusEnum::Cancelled);
        assert_eq!(parse_pipeline_status("pending"), PipelineStatusEnum::Pending);
        assert_eq!(parse_pipeline_status("created"), PipelineStatusEnum::Pending);
        assert_eq!(parse_pipeline_status("unknown"), PipelineStatusEnum::Pending);
    }

    #[test]
    fn test_should_deserialize_job_api_response() {
        let json = br#"{
            "id": 5001,
            "name": "test-unit",
            "status": "success",
            "stage": "test",
            "started_at": "2026-01-15T09:30:00Z",
            "finished_at": "2026-01-15T09:35:00Z",
            "web_url": "https://gitlab.com/org/project/-/jobs/5001"
        }"#;

        let api: JobApiResponse =
            serde_json::from_slice(json).expect("valid JobApiResponse");
        let job: JobData = api.into();

        assert_eq!(job.id, 5001);
        assert_eq!(job.name, "test-unit");
        assert_eq!(job.status, "success");
        assert_eq!(job.conclusion.as_deref(), Some("success"));
    }

    #[test]
    fn test_should_deserialize_failed_job() {
        let json = br#"{
            "id": 5002,
            "name": "test-integration",
            "status": "failed",
            "started_at": "2026-01-15T09:30:00Z",
            "finished_at": "2026-01-15T09:40:00Z",
            "web_url": "https://gitlab.com/org/project/-/jobs/5002"
        }"#;

        let api: JobApiResponse =
            serde_json::from_slice(json).expect("valid JobApiResponse");
        let job: JobData = api.into();
        assert_eq!(job.conclusion.as_deref(), Some("failure"));
    }

    #[test]
    fn test_should_deserialize_empty_pipeline_list() {
        let json = b"[]";
        let list: Vec<PipelineApiResponse> =
            serde_json::from_slice(json).expect("valid empty list");
        assert!(list.is_empty());
    }

    #[test]
    fn test_should_deserialize_empty_job_list() {
        let json = b"[]";
        let list: Vec<JobApiResponse> =
            serde_json::from_slice(json).expect("valid empty list");
        assert!(list.is_empty());
    }
}
