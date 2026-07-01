//! `gitflow issue` 子命令实现。
//!
//! 提供 Issue 的创建、列表、查看、关闭、重新打开、评论、标签管理等功能，
//! 支持通过 clap 解析参数后调用对应平台的 [`IssueProvider`] 实现。
//! Phase 1 仅支持 JSON 输出。

use clap::Subcommand;
use gitflow_cli_core::{
    CliOutput,
    issue::{CreateIssueArgs, IssueProvider, ListIssueArgs},
    types::State,
};
use gitflow_cli_github::GitHubIssueProvider;

use crate::OutputFormat;

/// Issue 子命令集合。
///
/// 支持 `create`、`list`、`view`、`close`、`reopen`、`comment`、
/// `add-label`、`remove-label` 操作，每种操作对应不同的 clap 参数。
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

        /// 从文件读取 Issue 正文（可选）。
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

    /// 关闭 Issue。
    Close {
        /// Issue 编号。
        number: u64,
    },

    /// 重新打开 Issue。
    Reopen {
        /// Issue 编号。
        number: u64,
    },

    /// 评论 Issue。
    Comment {
        /// Issue 编号。
        number: u64,

        /// 评论正文（可选，与 `--body-file` 二选一）。
        #[arg(long)]
        body: Option<String>,

        /// 从文件读取评论正文（可选）。
        #[arg(long = "body-file")]
        body_file: Option<String>,
    },

    /// 为 Issue 添加标签。
    AddLabel {
        /// Issue 编号。
        number: u64,

        /// 要添加的标签（至少一个）。
        #[arg(long = "label", num_args = 1.., required = true)]
        label: Vec<String>,
    },

    /// 从 Issue 移除标签。
    RemoveLabel {
        /// Issue 编号。
        number: u64,

        /// 要移除的标签名称。
        #[arg(long)]
        label: String,
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
/// - `--body` 与 `--body-file` 同时提供。
/// - `--body-file` 文件读取失败。
/// - `comment` 命令未提供评论正文。
/// - JSON 序列化失败。
#[allow(
    clippy::too_many_lines,
    reason = "Command dispatch: each match arm maps to one operation"
)]
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
        IssueCommand::Close { number } => {
            let issue = provider
                .close(number)
                .await
                .map_err(|e| miette::miette!("Failed to close issue #{number}: {e}"))?;
            let output = CliOutput::success(issue, platform, "issue close");
            print_output(&output, &output_format)?;
        }
        IssueCommand::Reopen { number } => {
            let issue = provider
                .reopen(number)
                .await
                .map_err(|e| miette::miette!("Failed to reopen issue #{number}: {e}"))?;
            let output = CliOutput::success(issue, platform, "issue reopen");
            print_output(&output, &output_format)?;
        }
        IssueCommand::Comment {
            number,
            body,
            body_file,
        } => {
            let resolved_body = resolve_comment_body(body, body_file)?;
            let comment = provider
                .comment(number, &resolved_body)
                .await
                .map_err(|e| miette::miette!("Failed to comment on issue #{number}: {e}"))?;
            let output = CliOutput::success(comment, platform, "issue comment");
            print_output(&output, &output_format)?;
        }
        IssueCommand::AddLabel { number, label } => {
            provider
                .add_labels(number, &label)
                .await
                .map_err(|e| miette::miette!("Failed to add labels to issue #{number}: {e}"))?;
            let result = serde_json::json!({
                "number": number,
                "labels_added": label,
            });
            let output = CliOutput::success(result, platform, "issue add-label");
            print_output(&output, &output_format)?;
        }
        IssueCommand::RemoveLabel { number, label } => {
            provider
                .remove_label(number, &label)
                .await
                .map_err(|e| miette::miette!("Failed to remove label from issue #{number}: {e}"))?;
            let result = serde_json::json!({
                "number": number,
                "label_removed": label,
            });
            let output = CliOutput::success(result, platform, "issue remove-label");
            print_output(&output, &output_format)?;
        }
    }

    Ok(())
}

/// 解析 `--body` 与 `--body-file` 参数。
///
/// 当提供 `--body-file` 时从文件读取内容。
///
/// # Errors
///
/// - 当同时提供 `--body` 与 `--body-file` 时返回错误。
/// - 当文件读取失败时返回错误。
#[allow(
    clippy::disallowed_methods,
    reason = "Sync file read; called from async handler but file body input is small and local"
)]
fn resolve_body(body: Option<String>, body_file: Option<String>) -> miette::Result<Option<String>> {
    if body.is_some() && body_file.is_some() {
        return Err(miette::miette!(
            "Cannot specify both --body and --body-file"
        ));
    }
    if let Some(path) = body_file {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| miette::miette!("Failed to read body file '{path}': {e}"))?;
        return Ok(Some(content));
    }
    Ok(body)
}

/// 解析评论正文，要求必须提供 `--body` 或 `--body-file` 之一。
///
/// # Errors
///
/// - 当两者都未提供时返回错误。
/// - 当同时提供两者时返回错误。
/// - 当文件读取失败时返回错误。
fn resolve_comment_body(body: Option<String>, body_file: Option<String>) -> miette::Result<String> {
    let resolved = resolve_body(body, body_file)?;
    resolved.ok_or_else(|| miette::miette!("Comment body is required. Use --body or --body-file."))
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
    fn test_should_resolve_body_from_file() {
        let dir = std::env::temp_dir();
        let path = dir.join("gitflow_test_body.md");
        std::fs::write(&path, "file content here").expect("write temp file");
        let result = resolve_body(None, Some(path.to_string_lossy().into_owned()));
        let _ = std::fs::remove_file(&path);
        assert!(result.is_ok());
        assert_eq!(
            result.expect("already checked"),
            Some("file content here".into())
        );
    }

    #[test]
    fn test_should_error_on_missing_body_file() {
        let result = resolve_body(None, Some("/nonexistent/path/body.md".into()));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Failed to read body file"));
    }

    #[test]
    fn test_should_reject_both_body_and_body_file() {
        let result = resolve_body(Some("hello".into()), Some("/tmp/body.md".into()));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Cannot specify both"));
    }

    #[test]
    fn test_should_require_comment_body() {
        let result = resolve_comment_body(None, None);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Comment body is required"));
    }

    #[test]
    fn test_should_resolve_comment_body_with_body() {
        let result = resolve_comment_body(Some("a comment".into()), None);
        assert!(result.is_ok());
        assert_eq!(result.expect("already checked"), "a comment");
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

    // --- IssueCommand 解析测试 ---

    #[test]
    fn test_should_parse_issue_close() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from(["gitflow", "issue", "close", "42"]).expect("parse");
        match cli.command {
            crate::Commands::Issue(IssueCommand::Close { number }) => {
                assert_eq!(number, 42);
            }
            _ => panic!("Expected IssueCommand::Close"),
        }
    }

    #[test]
    fn test_should_parse_issue_reopen() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from(["gitflow", "issue", "reopen", "7"]).expect("parse");
        match cli.command {
            crate::Commands::Issue(IssueCommand::Reopen { number }) => {
                assert_eq!(number, 7);
            }
            _ => panic!("Expected IssueCommand::Reopen"),
        }
    }

    #[test]
    fn test_should_parse_issue_comment_with_body() {
        use clap::Parser;
        let cli =
            crate::Cli::try_parse_from(["gitflow", "issue", "comment", "10", "--body", "LGTM"])
                .expect("parse");
        match cli.command {
            crate::Commands::Issue(IssueCommand::Comment {
                number,
                body,
                body_file,
            }) => {
                assert_eq!(number, 10);
                assert_eq!(body, Some("LGTM".into()));
                assert!(body_file.is_none());
            }
            _ => panic!("Expected IssueCommand::Comment"),
        }
    }

    #[test]
    fn test_should_parse_issue_add_label() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "gitflow",
            "issue",
            "add-label",
            "5",
            "--label",
            "bug",
            "urgent",
        ])
        .expect("parse");
        match cli.command {
            crate::Commands::Issue(IssueCommand::AddLabel { number, label }) => {
                assert_eq!(number, 5);
                assert_eq!(label, vec!["bug".to_string(), "urgent".to_string()]);
            }
            _ => panic!("Expected IssueCommand::AddLabel"),
        }
    }

    #[test]
    fn test_should_parse_issue_remove_label() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "gitflow",
            "issue",
            "remove-label",
            "3",
            "--label",
            "wontfix",
        ])
        .expect("parse");
        match cli.command {
            crate::Commands::Issue(IssueCommand::RemoveLabel { number, label }) => {
                assert_eq!(number, 3);
                assert_eq!(label, "wontfix");
            }
            _ => panic!("Expected IssueCommand::RemoveLabel"),
        }
    }
}
