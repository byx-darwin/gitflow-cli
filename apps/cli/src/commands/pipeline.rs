//! `gitflow pipeline` 子命令实现。
//!
//! 提供流水线状态查询、日志获取、任务列表、健康报告等功能，
//! 支持通过 clap 解析参数后调用对应平台的 [`PipelineProvider`] 实现。
//! Phase 1 仅支持 JSON 输出。

use clap::Subcommand;
use gitflow_cli_core::{CliOutput, pipeline::PipelineProvider};
use gitflow_cli_gitcode::GitCodePipelineProvider;
use gitflow_cli_github::GitHubPipelineProvider;
use gitflow_cli_gitlab::GitLabPipelineProvider;

use crate::OutputFormat;

/// Pipeline 子命令集合。
///
/// 支持 `status`、`logs`、`jobs`、`report` 操作。
#[derive(Debug, Subcommand)]
pub enum PipelineCommand {
    /// 列出指定分支的流水线运行状态。
    Status {
        /// 分支名称（默认 `main`）。
        #[arg(long, default_value = "main")]
        branch: String,
    },

    /// 查看指定流水线的日志。
    Logs {
        /// 流水线 ID。
        #[arg(long)]
        pipeline_id: u64,
    },

    /// 列出指定流水线包含的任务。
    Jobs {
        /// 流水线 ID。
        #[arg(long)]
        pipeline_id: u64,
    },

    /// 生成流水线健康报告。
    Report {
        /// 分支名称（默认 `main`）。
        #[arg(long, default_value = "main")]
        branch: String,

        /// 统计天数（默认 30）。
        #[arg(long, default_value = "30")]
        days: u32,
    },
}

/// 处理 `gitflow pipeline` 子命令。
///
/// 根据 `platform` 选择对应的 Pipeline 提供者，然后执行具体命令并输出结果。
/// 支持 `github`、`gitlab`、`gitcode` 三个平台，Phase 1 仅支持 JSON 输出格式。
///
/// # Errors
///
/// 返回错误当：
/// - 平台暂不支持。
/// - 底层 provider 调用失败。
/// - JSON 序列化失败。
pub async fn handle(
    command: PipelineCommand,
    platform: &str,
    repo: &str,
    output_format: OutputFormat,
) -> miette::Result<()> {
    let provider: Box<dyn PipelineProvider> = match platform {
        "github" => Box::new(GitHubPipelineProvider::new(repo)),
        "gitlab" => Box::new(GitLabPipelineProvider::new(repo)),
        "gitcode" => Box::new(GitCodePipelineProvider::new(repo)),
        other => {
            return Err(miette::miette!(
                "Platform '{other}' not yet supported for pipeline commands"
            ));
        }
    };

    match command {
        PipelineCommand::Status { branch } => {
            let result = provider
                .status(&branch)
                .await
                .map_err(|e| miette::miette!("Failed to get pipeline status for '{branch}': {e}"))?;
            let output = CliOutput::success(result, platform, "pipeline status");
            print_output(&output, &output_format)?;
        }
        PipelineCommand::Logs { pipeline_id } => {
            let result = provider
                .logs(pipeline_id)
                .await
                .map_err(|e| miette::miette!("Failed to get logs for pipeline {pipeline_id}: {e}"))?;
            let result_value = serde_json::json!({
                "pipeline_id": pipeline_id,
                "logs": result,
            });
            let output = CliOutput::success(result_value, platform, "pipeline logs");
            print_output(&output, &output_format)?;
        }
        PipelineCommand::Jobs { pipeline_id } => {
            let result = provider
                .jobs(pipeline_id)
                .await
                .map_err(|e| miette::miette!("Failed to get jobs for pipeline {pipeline_id}: {e}"))?;
            let output = CliOutput::success(result, platform, "pipeline jobs");
            print_output(&output, &output_format)?;
        }
        PipelineCommand::Report { branch, days } => {
            let result = provider
                .report(&branch, days)
                .await
                .map_err(|e| miette::miette!("Failed to generate pipeline report for '{branch}': {e}"))?;
            let output = CliOutput::success(result, platform, "pipeline report");
            print_output(&output, &output_format)?;
        }
    }

    Ok(())
}

/// 根据输出格式打印结果。
///
/// Phase 1 仅支持 JSON（pretty-printed）。Text 格式暂未实现，返回错误。
///
/// # Errors
///
/// 返回错误当：
/// - JSON 序列化失败。
/// - 输出格式为 `Text`（Phase 1 不支持）。
fn print_output<T: serde::Serialize>(value: &T, format: &OutputFormat) -> miette::Result<()> {
    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(value)
                .map_err(|e| miette::miette!("Failed to serialize output to JSON: {e}"))?;
            println!("{json}");
            Ok(())
        }
        OutputFormat::Text => Err(miette::miette!(
            "Text output format is not yet supported in Phase 1. Use --output json."
        )),
    }
}

#[cfg(test)]
#[allow(
    clippy::panic,
    reason = "Test code: panic is acceptable for assertion failures"
)]
mod tests {
    use super::*;

    #[test]
    fn test_should_print_json_output() {
        let value = serde_json::json!({"pipeline_id": 1, "status": "success"});
        let result = print_output(&value, &OutputFormat::Json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_should_reject_text_output_in_phase1() {
        let value = serde_json::json!({"pipeline_id": 1});
        let result = print_output(&value, &OutputFormat::Text);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not yet supported"));
    }

    // --- PipelineCommand 解析测试 ---

    #[test]
    fn test_should_parse_pipeline_status_default_branch() {
        use clap::Parser;
        let cli =
            crate::Cli::try_parse_from(["gitflow", "pipeline", "status"]).expect("parse");
        match cli.command {
            crate::Commands::Pipeline(PipelineCommand::Status { branch }) => {
                assert_eq!(branch, "main");
            }
            _ => panic!("Expected PipelineCommand::Status"),
        }
    }

    #[test]
    fn test_should_parse_pipeline_status_with_branch() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "gitflow",
            "pipeline",
            "status",
            "--branch",
            "develop",
        ])
        .expect("parse");
        match cli.command {
            crate::Commands::Pipeline(PipelineCommand::Status { branch }) => {
                assert_eq!(branch, "develop");
            }
            _ => panic!("Expected PipelineCommand::Status"),
        }
    }

    #[test]
    fn test_should_parse_pipeline_logs() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "gitflow",
            "pipeline",
            "logs",
            "--pipeline-id",
            "12345",
        ])
        .expect("parse");
        match cli.command {
            crate::Commands::Pipeline(PipelineCommand::Logs { pipeline_id }) => {
                assert_eq!(pipeline_id, 12_345);
            }
            _ => panic!("Expected PipelineCommand::Logs"),
        }
    }

    #[test]
    fn test_should_parse_pipeline_jobs() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "gitflow",
            "pipeline",
            "jobs",
            "--pipeline-id",
            "99",
        ])
        .expect("parse");
        match cli.command {
            crate::Commands::Pipeline(PipelineCommand::Jobs { pipeline_id }) => {
                assert_eq!(pipeline_id, 99);
            }
            _ => panic!("Expected PipelineCommand::Jobs"),
        }
    }

    #[test]
    fn test_should_parse_pipeline_report_default() {
        use clap::Parser;
        let cli =
            crate::Cli::try_parse_from(["gitflow", "pipeline", "report"]).expect("parse");
        match cli.command {
            crate::Commands::Pipeline(PipelineCommand::Report { branch, days }) => {
                assert_eq!(branch, "main");
                assert_eq!(days, 30);
            }
            _ => panic!("Expected PipelineCommand::Report"),
        }
    }

    #[test]
    fn test_should_parse_pipeline_report_with_args() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "gitflow",
            "pipeline",
            "report",
            "--branch",
            "release/1.0",
            "--days",
            "14",
        ])
        .expect("parse");
        match cli.command {
            crate::Commands::Pipeline(PipelineCommand::Report { branch, days }) => {
                assert_eq!(branch, "release/1.0");
                assert_eq!(days, 14);
            }
            _ => panic!("Expected PipelineCommand::Report"),
        }
    }
}
