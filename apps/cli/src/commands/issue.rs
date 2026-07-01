//! `gitflow issue` 子命令实现。
//!
//! 提供 Issue 的创建、列表和查看功能，支持通过 clap 解析参数后
//! 调用对应平台的 [`IssueProvider`] 实现。Phase 1 仅支持 JSON 输出。

use clap::Subcommand;
use gitflow_cli_core::{
    issue::{CreateIssueArgs, IssueProvider, ListIssueArgs},
    types::State,
    CliOutput,
};
use gitflow_cli_github::GitHubIssueProvider;

use crate::OutputFormat;

/// Issue 子命令集合。
///
/// 支持 `create`、`list`、`view` 三种操作，每种操作对应不同的 clap 参数。
#[derive(Debug, Subcommand)]
pub enum IssueCommand {
    /// 创建一个新的 Issue。
    Create {
        /// Issue 标题（必填）。
        #[arg(long)]
        title: String,

        /// Issue 正文（可选，与 `--body-file` 二选一）。
        #[arg(long)]
        body: Option<String>,

        /// 从文件读取 Issue 正文（可选，Phase 1 暂未实现）。
        #[arg(long = "body-file")]
        body_file: Option<String>,

        /// 标签列表（可多次指定）。
        #[arg(long = "label")]
        label: Vec<String>,

        /// 指派人列表（可多次指定）。
        #[arg(long = "assignee")]
        assignee: Vec<String>,
    },

    /// 列出 Issue。
    List {
        /// 按状态过滤（`open` 或 `closed`）。
        #[arg(long)]
        state: Option<String>,

        /// 搜索关键词。
        #[arg(long)]
        search: Option<String>,

        /// 按标签过滤（可多次指定）。
        #[arg(long = "label")]
        label: Vec<String>,

        /// 返回数量上限。
        #[arg(long)]
        limit: Option<u32>,
    },

    /// 查看单个 Issue 详情。
    View {
        /// Issue 编号。
        number: u64,
    },
}

/// 处理 `gitflow issue` 子命令。
///
/// 根据 `platform` 选择对应的 Issue 提供者，然后执行具体命令并输出结果。
/// Phase 1 仅支持 `github` 平台与 JSON 输出格式。
///
/// # Errors
///
/// 返回错误当：
/// - 平台暂不支持（如 `gitlab`）。
/// - 底层 provider 调用失败（如 `gh` CLI 执行失败）。
/// - `--body-file` 在 Phase 1 中被使用。
/// - JSON 序列化失败。
pub async fn handle(
    command: IssueCommand,
    platform: &str,
    repo: &str,
    output_format: OutputFormat,
) -> miette::Result<()> {
    let provider: Box<dyn IssueProvider> = match platform {
        "github" => Box::new(GitHubIssueProvider::new(repo)),
        other => {
            return Err(miette::miette!(
                "Platform '{other}' not yet supported for issue commands"
            ));
        }
    };

    match command {
        IssueCommand::Create {
            title,
            body,
            body_file,
            label,
            assignee,
        } => {
            let resolved_body = resolve_body(body, body_file)?;
            let args = CreateIssueArgs {
                title,
                body: resolved_body,
                labels: label,
                assignees: assignee,
            };
            let issue = provider
                .create(args)
                .await
                .map_err(|e| miette::miette!("Failed to create issue: {e}"))?;
            let output = CliOutput::success(issue, platform, "issue create");
            print_output(&output, &output_format)?;
        }
        IssueCommand::List {
            state,
            search,
            label,
            limit,
        } => {
            let parsed_state = state
                .as_deref()
                .map(|s| match s {
                    "open" => Ok(State::Open),
                    "closed" => Ok(State::Closed),
                    other => Err(miette::miette!(
                        "Invalid state '{other}'. Expected 'open' or 'closed'."
                    )),
                })
                .transpose()?;

            let args = ListIssueArgs {
                state: parsed_state,
                labels: label,
                assignee: None,
                search,
                limit,
            };
            let issues = provider
                .list(args)
                .await
                .map_err(|e| miette::miette!("Failed to list issues: {e}"))?;
            let output = CliOutput::success(issues, platform, "issue list");
            print_output(&output, &output_format)?;
        }
        IssueCommand::View { number } => {
            let issue = provider
                .view(number)
                .await
                .map_err(|e| miette::miette!("Failed to view issue #{number}: {e}"))?;
            let output = CliOutput::success(issue, platform, "issue view");
            print_output(&output, &output_format)?;
        }
    }

    Ok(())
}

/// 解析 `--body` 与 `--body-file` 参数。
///
/// Phase 1 仅支持 `--body`，若提供 `--body-file` 则返回错误。
///
/// # Errors
///
/// - 当同时提供 `--body` 与 `--body-file` 时返回错误。
/// - 当提供 `--body-file` 时返回 "not yet supported" 错误。
fn resolve_body(body: Option<String>, body_file: Option<String>) -> miette::Result<Option<String>> {
    if body.is_some() && body_file.is_some() {
        return Err(miette::miette!(
            "Cannot specify both --body and --body-file"
        ));
    }
    if let Some(path) = body_file {
        return Err(miette::miette!(
            "--body-file '{path}' not yet supported in Phase 1"
        ));
    }
    Ok(body)
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
mod tests {
    use super::*;

    #[test]
    fn test_should_resolve_body_with_body_only() {
        let result = resolve_body(Some("hello".into()), None);
        assert!(result.is_ok());
        assert_eq!(result.expect("already checked"), Some("hello".into()));
    }

    #[test]
    fn test_should_resolve_body_with_none() {
        let result = resolve_body(None, None);
        assert!(result.is_ok());
        assert_eq!(result.expect("already checked"), None);
    }

    #[test]
    fn test_should_reject_body_file_in_phase1() {
        let result = resolve_body(None, Some("/tmp/body.md".into()));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not yet supported"));
    }

    #[test]
    fn test_should_reject_both_body_and_body_file() {
        let result = resolve_body(Some("hello".into()), Some("/tmp/body.md".into()));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Cannot specify both"));
    }

    #[test]
    fn test_should_print_json_output() {
        let value = serde_json::json!({"number": 1, "title": "test"});
        let result = print_output(&value, &OutputFormat::Json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_should_reject_text_output_in_phase1() {
        let value = serde_json::json!({"number": 1});
        let result = print_output(&value, &OutputFormat::Text);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not yet supported"));
    }
}
