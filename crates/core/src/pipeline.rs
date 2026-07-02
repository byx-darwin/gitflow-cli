//! Pipeline (CI/CD) 领域类型与平台抽象。
//!
//! 定义了流水线状态、任务数据、分析报告的数据表示，以及跨平台
//! 实现所需的 [`PipelineProvider`] trait。GitHub Actions、GitLab CI、
//! `GitCode` CI 等平台实现都需实现该 trait，使上层命令层可统一消费。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::Result;

/// 流水线状态枚举。
///
/// 表示 CI/CD 流水线的当前运行状态。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PipelineStatusEnum {
    /// 流水线正在运行。
    Running,
    /// 流水线成功完成。
    Success,
    /// 流水线失败。
    Failed,
    /// 流水线已取消。
    Cancelled,
    /// 流水线等待中。
    Pending,
}

/// 流水线状态数据。
///
/// 由平台实现填充并返回给上层命令。字段命名与平台 API 输出
/// 的 JSON 字段对齐（camelCase）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineStatus {
    /// 流水线编号（平台内唯一）。
    pub id: u64,
    /// 触发流水线的分支或标签名。
    pub ref_name: String,
    /// 流水线当前状态。
    pub status: PipelineStatusEnum,
    /// 流水线结论（成功/失败原因等，可选）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conclusion: Option<String>,
    /// 创建时间（UTC）。
    pub created_at: DateTime<Utc>,
    /// 最近更新时间（UTC）。
    pub updated_at: DateTime<Utc>,
    /// 流水线的 Web URL。
    pub url: String,
}

/// 流水线任务（Job）数据。
///
/// 表示流水线中的单个任务/步骤的执行信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobData {
    /// 任务编号（平台内唯一）。
    pub id: u64,
    /// 任务名称。
    pub name: String,
    /// 任务当前状态（字符串，由平台定义）。
    pub status: String,
    /// 任务结论（成功/失败原因等，可选）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conclusion: Option<String>,
    /// 任务开始时间（UTC）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<DateTime<Utc>>,
    /// 任务完成时间（UTC）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
    /// 任务日志的 Web URL。
    pub url: String,
}

/// 流水线分析报告。
///
/// 汇总指定分支在一段时间内的流水线运行统计数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineReport {
    /// 总运行次数。
    pub total_runs: u64,
    /// 成功率（0.0 ~ 1.0）。
    pub success_rate: f64,
    /// 平均运行时长（秒）。
    pub avg_duration_secs: f64,
    /// 最常见的失败原因列表。
    #[serde(default)]
    pub top_failures: Vec<String>,
}

/// 流水线（CI/CD）操作的平台抽象。
///
/// 所有平台实现（GitHub/GitLab/GitCode）都必须实现此 trait，
/// 以提供统一的流水线状态查询、日志获取、任务列表和报告生成能力。
///
/// # Errors
///
/// 所有方法在平台 API 调用失败、反序列化失败或鉴权失败时返回
/// [`CoreError`](crate::CoreError)。
///
/// [`CoreError`]: crate::CoreError
#[async_trait]
pub trait PipelineProvider: std::fmt::Debug + Send + Sync {
    /// 获取指定分支的流水线状态列表。
    ///
    /// # Errors
    ///
    /// 当平台 API 调用失败或分支不存在时返回错误。
    async fn status(&self, branch: &str) -> Result<Vec<PipelineStatus>>;

    /// 获取指定流水号的日志文本。
    ///
    /// # Errors
    ///
    /// 当流水线不存在、日志不可访问或平台 API 调用失败时返回错误。
    async fn logs(&self, pipeline_id: u64) -> Result<String>;

    /// 获取指定流水线包含的任务列表。
    ///
    /// # Errors
    ///
    /// 当流水线不存在或平台 API 调用失败时返回错误。
    async fn jobs(&self, pipeline_id: u64) -> Result<Vec<JobData>>;

    /// 生成指定分支在指定天数内的流水线分析报告。
    ///
    /// # Errors
    ///
    /// 当分支不存在、平台 API 调用失败或无法生成报告时返回错误。
    async fn report(&self, branch: &str, days: u32) -> Result<PipelineReport>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_serialize_pipeline_status_enum_to_snake_case() {
        assert_eq!(
            serde_json::to_string(&PipelineStatusEnum::Running).expect("serialize"),
            "\"running\""
        );
        assert_eq!(
            serde_json::to_string(&PipelineStatusEnum::Success).expect("serialize"),
            "\"success\""
        );
        assert_eq!(
            serde_json::to_string(&PipelineStatusEnum::Failed).expect("serialize"),
            "\"failed\""
        );
        assert_eq!(
            serde_json::to_string(&PipelineStatusEnum::Cancelled).expect("serialize"),
            "\"cancelled\""
        );
        assert_eq!(
            serde_json::to_string(&PipelineStatusEnum::Pending).expect("serialize"),
            "\"pending\""
        );
    }

    #[test]
    fn test_should_deserialize_pipeline_status_enum_from_snake_case() {
        assert_eq!(
            serde_json::from_str::<PipelineStatusEnum>("\"running\"").expect("deserialize"),
            PipelineStatusEnum::Running
        );
        assert_eq!(
            serde_json::from_str::<PipelineStatusEnum>("\"success\"").expect("deserialize"),
            PipelineStatusEnum::Success
        );
        assert_eq!(
            serde_json::from_str::<PipelineStatusEnum>("\"failed\"").expect("deserialize"),
            PipelineStatusEnum::Failed
        );
        assert_eq!(
            serde_json::from_str::<PipelineStatusEnum>("\"cancelled\"").expect("deserialize"),
            PipelineStatusEnum::Cancelled
        );
        assert_eq!(
            serde_json::from_str::<PipelineStatusEnum>("\"pending\"").expect("deserialize"),
            PipelineStatusEnum::Pending
        );
    }

    fn sample_pipeline_status_json() -> &'static str {
        r#"{
            "id": 12345,
            "refName": "main",
            "status": "success",
            "conclusion": "all checks passed",
            "createdAt": "2026-07-01T10:00:00Z",
            "updatedAt": "2026-07-01T10:05:30Z",
            "url": "https://github.com/example/repo/actions/runs/12345"
        }"#
    }

    #[test]
    fn test_should_deserialize_pipeline_status_from_json() {
        let json = sample_pipeline_status_json();
        let status: PipelineStatus = serde_json::from_str(json).expect("valid PipelineStatus JSON");

        assert_eq!(status.id, 12345);
        assert_eq!(status.ref_name, "main");
        assert_eq!(status.status, PipelineStatusEnum::Success);
        assert_eq!(status.conclusion.as_deref(), Some("all checks passed"));
        assert_eq!(
            status.url,
            "https://github.com/example/repo/actions/runs/12345"
        );
    }

    #[test]
    fn test_should_deserialize_pipeline_status_with_null_conclusion() {
        let json = r#"{
            "id": 1,
            "refName": "develop",
            "status": "running",
            "conclusion": null,
            "createdAt": "2026-07-02T08:00:00Z",
            "updatedAt": "2026-07-02T08:00:00Z",
            "url": "https://example.com/1"
        }"#;
        let status: PipelineStatus = serde_json::from_str(json).expect("deserialize");
        assert!(status.conclusion.is_none());
        assert_eq!(status.status, PipelineStatusEnum::Running);
    }

    #[test]
    fn test_should_omit_none_conclusion_on_serialize() {
        let json = sample_pipeline_status_json();
        let mut status: PipelineStatus = serde_json::from_str(json).expect("deserialize");
        status.conclusion = None;
        let serialized = serde_json::to_string(&status).expect("serialize");
        assert!(!serialized.contains("\"conclusion\":null"));
        assert!(!serialized.contains("\"conclusion\": null"));
    }

    fn sample_job_json() -> &'static str {
        r#"{
            "id": 98765,
            "name": "build",
            "status": "completed",
            "conclusion": "success",
            "startedAt": "2026-07-01T10:00:00Z",
            "completedAt": "2026-07-01T10:03:00Z",
            "url": "https://github.com/example/repo/actions/runs/12345/job/98765"
        }"#
    }

    #[test]
    fn test_should_deserialize_job_data_from_json() {
        let json = sample_job_json();
        let job: JobData = serde_json::from_str(json).expect("valid JobData JSON");

        assert_eq!(job.id, 98765);
        assert_eq!(job.name, "build");
        assert_eq!(job.status, "completed");
        assert_eq!(job.conclusion.as_deref(), Some("success"));
        assert_eq!(
            job.url,
            "https://github.com/example/repo/actions/runs/12345/job/98765"
        );
    }

    #[test]
    fn test_should_deserialize_job_data_with_null_optional_fields() {
        let json = r#"{
            "id": 1,
            "name": "test",
            "status": "queued",
            "conclusion": null,
            "startedAt": null,
            "completedAt": null,
            "url": "https://example.com/job/1"
        }"#;
        let job: JobData = serde_json::from_str(json).expect("deserialize");
        assert!(job.conclusion.is_none());
        assert!(job.started_at.is_none());
        assert!(job.completed_at.is_none());
        assert_eq!(job.status, "queued");
    }

    #[test]
    fn test_should_deserialize_job_data_and_roundtrip() {
        let json = sample_job_json();
        let job: JobData = serde_json::from_str(json).expect("deserialize");
        let re_serialized = serde_json::to_string(&job).expect("serialize");
        let round_tripped: JobData = serde_json::from_str(&re_serialized).expect("re-deserialize");

        assert_eq!(round_tripped.id, job.id);
        assert_eq!(round_tripped.name, job.name);
        assert_eq!(round_tripped.status, job.status);
        assert_eq!(round_tripped.conclusion, job.conclusion);
        assert_eq!(round_tripped.url, job.url);
    }

    #[test]
    fn test_should_build_pipeline_report_and_access_fields() {
        let report = PipelineReport {
            total_runs: 100,
            success_rate: 0.95,
            avg_duration_secs: 180.5,
            top_failures: vec!["timeout".into(), "compile error".into()],
        };

        assert_eq!(report.total_runs, 100);
        assert!((report.success_rate - 0.95).abs() < f64::EPSILON);
        assert!((report.avg_duration_secs - 180.5).abs() < f64::EPSILON);
        assert_eq!(report.top_failures.len(), 2);
        assert_eq!(report.top_failures[0], "timeout");
        assert_eq!(report.top_failures[1], "compile error");
    }

    #[test]
    fn test_should_serialize_pipeline_report_to_camel_case() {
        let report = PipelineReport {
            total_runs: 42,
            success_rate: 0.8,
            avg_duration_secs: 120.0,
            top_failures: vec!["flaky test".into()],
        };
        let serialized = serde_json::to_string(&report).expect("serialize");

        // Verify camelCase keys
        assert!(serialized.contains("\"totalRuns\""));
        assert!(serialized.contains("\"successRate\""));
        assert!(serialized.contains("\"avgDurationSecs\""));
        assert!(serialized.contains("\"topFailures\""));
    }

    #[test]
    fn test_should_deserialize_pipeline_report_with_missing_top_failures() {
        let json = r#"{
            "totalRuns": 10,
            "successRate": 1.0,
            "avgDurationSecs": 60.0
        }"#;
        let report: PipelineReport = serde_json::from_str(json).expect("deserialize");
        assert_eq!(report.total_runs, 10);
        assert!(report.top_failures.is_empty());
    }
}
