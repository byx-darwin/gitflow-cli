//! `gitflow review` 子命令实现。
//!
//! 提供 PR Review 的评论、批准、要求修改、提交审查等功能，
//! 支持通过 clap 解析参数后调用对应平台的 [`ReviewProvider`] 实现。
//! Phase 1 仅支持 JSON 输出。

use clap::Subcommand;
use gitflow_cli_core::{
    CliOutput,
    review::{ReviewProvider, ReviewState},
};
use gitflow_cli_gitcode::GitCodeReviewProvider;
use gitflow_cli_github::GitHubReviewProvider;
use gitflow_cli_gitlab::GitLabReviewProvider;

use crate::OutputFormat;

/// Review 子命令集合。
///
/// 支持 `comment`、`approve`、`request-changes`、`submit` 操作，
/// 每种操作对应不同的 clap 参数。
#[derive(Debug, Subcommand)]
pub enum ReviewCommand {
    /// 在 PR 上发表评论。
    Comment {
        /// PR 编号。
        pr_number: u64,

        /// 评论正文（可选，与 `--body-file` 二选一）。
        #[arg(long)]
        body: Option<String>,

        /// 从文件读取评论正文（可选）。
        #[arg(long = "body-file")]
        body_file: Option<String>,
    },

    /// 批准 PR。
    Approve {
        /// PR 编号。
        pr_number: u64,

        /// 批准说明（可选，与 `--body-file` 二选一）。
        #[arg(long)]
        body: Option<String>,

        /// 从文件读取批准说明（可选）。
        #[arg(long = "body-file")]
        body_file: Option<String>,
    },

    /// 要求对 PR 进行修改。
    RequestChanges {
        /// PR 编号。
        pr_number: u64,

        /// 修改要求说明（可选，与 `--body-file` 二选一）。
        #[arg(long)]
        body: Option<String>,

        /// 从文件读取修改要求（可选）。
        #[arg(long = "body-file")]
        body_file: Option<String>,
    },

    /// 提交一次完整的 Review。
    Submit {
        /// PR 编号。
        pr_number: u64,

        /// Review 总结说明（可选，与 `--body-file` 二选一）。
        #[arg(long)]
        body: Option<String>,

        /// 从文件读取 Review 总结（可选）。
        #[arg(long = "body-file")]
        body_file: Option<String>,

        /// Review 结论（`approved`、`changes_requested`、`commented`）。
        #[arg(long)]
        event: ReviewEventArg,
    },
}

/// Review 事件结论参数。
///
/// 用作 `clap::ValueEnum`，映射到 [`ReviewState`]。
#[derive(Debug, Clone, clap::ValueEnum)]
pub(crate) enum ReviewEventArg {
    /// 审查通过，可以合并。
    #[value(name = "approved")]
    Approved,
    /// 要求修改后才能合并。
    #[value(name = "changes_requested")]
    ChangesRequested,
    /// 仅发表评论，不表态。
    #[value(name = "commented")]
    Commented,
}

impl From<ReviewEventArg> for ReviewState {
    fn from(arg: ReviewEventArg) -> Self {
        match arg {
            ReviewEventArg::Approved => ReviewState::Approved,
            ReviewEventArg::ChangesRequested => ReviewState::ChangesRequested,
            ReviewEventArg::Commented => ReviewState::Commented,
        }
    }
}

/// 处理 `gitflow review` 子命令。
///
/// 根据 `platform` 选择对应的 Review 提供者，然后执行具体命令并输出结果。
/// 支持 `github`、`gitlab`、`gitcode` 三个平台，Phase 1 仅支持 JSON 输出格式。
///
/// # Errors
///
/// 返回错误当：
/// - 平台暂不支持。
/// - 底层 provider 调用失败。
/// - `--body` 与 `--body-file` 同时提供。
/// - `--body-file` 文件读取失败。
/// - `comment` 命令未提供评论正文。
/// - `request-changes` 命令未提供修改要求。
/// - JSON 序列化失败。
#[allow(
    clippy::too_many_lines,
    reason = "Command dispatch: each match arm maps to one operation"
)]
pub async fn handle(
    command: ReviewCommand,
    platform: &str,
    repo: &str,
    output_format: OutputFormat,
) -> miette::Result<()> {
    let provider: Box<dyn ReviewProvider> = match platform {
        "github" => Box::new(GitHubReviewProvider::new(repo)),
        "gitlab" => Box::new(GitLabReviewProvider::new(repo)),
        "gitcode" => Box::new(GitCodeReviewProvider::new(repo)),
        other => {
            return Err(miette::miette!(
                "Platform '{other}' not yet supported for review commands"
            ));
        }
    };

    match command {
        ReviewCommand::Comment {
            pr_number,
            body,
            body_file,
        } => {
            let resolved_body = resolve_comment_body(body, body_file)?;
            let review = provider
                .comment(pr_number, &resolved_body)
                .await
                .map_err(|e| miette::miette!("Failed to comment on PR #{pr_number}: {e}"))?;
            let output = CliOutput::success(review, platform, "review comment");
            print_output(&output, &output_format)?;
        }
        ReviewCommand::Approve {
            pr_number,
            body,
            body_file,
        } => {
            let resolved_body = resolve_body(body, body_file)?;
            let review = provider
                .approve(pr_number, resolved_body.as_deref())
                .await
                .map_err(|e| miette::miette!("Failed to approve PR #{pr_number}: {e}"))?;
            let output = CliOutput::success(review, platform, "review approve");
            print_output(&output, &output_format)?;
        }
        ReviewCommand::RequestChanges {
            pr_number,
            body,
            body_file,
        } => {
            let resolved_body = resolve_comment_body(body, body_file)?;
            let review = provider
                .request_changes(pr_number, &resolved_body)
                .await
                .map_err(|e| {
                    miette::miette!("Failed to request changes on PR #{pr_number}: {e}")
                })?;
            let output = CliOutput::success(review, platform, "review request-changes");
            print_output(&output, &output_format)?;
        }
        ReviewCommand::Submit {
            pr_number,
            body,
            body_file,
            event,
        } => {
            let resolved_body = resolve_body(body, body_file)?;
            let state: ReviewState = event.into();
            let review = provider
                .submit_review(pr_number, state, resolved_body.as_deref())
                .await
                .map_err(|e| miette::miette!("Failed to submit review for PR #{pr_number}: {e}"))?;
            let output = CliOutput::success(review, platform, "review submit");
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
    resolved.ok_or_else(|| miette::miette!("Review body is required. Use --body or --body-file."))
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
    crate::commands::output::print_output(value, format)
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
        let result = resolve_body(Some("looks good".into()), None);
        assert!(result.is_ok());
        assert_eq!(result.expect("already checked"), Some("looks good".into()));
    }

    #[test]
    fn test_should_resolve_body_with_none() {
        let result = resolve_body(None, None);
        assert!(result.is_ok());
        assert_eq!(result.expect("already checked"), None);
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
        assert!(err.contains("Review body is required"));
    }

    #[test]
    fn test_should_resolve_comment_body_with_body() {
        let result = resolve_comment_body(Some("needs work".into()), None);
        assert!(result.is_ok());
        assert_eq!(result.expect("already checked"), "needs work");
    }

    #[test]
    fn test_should_print_json_output() {
        let value = serde_json::json!({"id": 1, "state": "approved"});
        let result = print_output(&value, &OutputFormat::Json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_should_accept_text_output() {
        let value = serde_json::json!({"number": 1});
        let result = print_output(&value, &OutputFormat::Text);
        assert!(result.is_ok());
    }

    // --- ReviewCommand 解析测试 ---

    #[test]
    fn test_should_parse_review_comment() {
        use clap::Parser;
        let cli =
            crate::Cli::try_parse_from(["gitflow", "review", "comment", "42", "--body", "LGTM"])
                .expect("parse");
        match cli.command {
            crate::Commands::Review(ReviewCommand::Comment {
                pr_number,
                body,
                body_file,
            }) => {
                assert_eq!(pr_number, 42);
                assert_eq!(body, Some("LGTM".into()));
                assert!(body_file.is_none());
            }
            _ => panic!("Expected ReviewCommand::Comment"),
        }
    }

    #[test]
    fn test_should_parse_review_approve() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "gitflow",
            "review",
            "approve",
            "10",
            "--body",
            "Looks good to me",
        ])
        .expect("parse");
        match cli.command {
            crate::Commands::Review(ReviewCommand::Approve {
                pr_number,
                body,
                body_file,
            }) => {
                assert_eq!(pr_number, 10);
                assert_eq!(body, Some("Looks good to me".into()));
                assert!(body_file.is_none());
            }
            _ => panic!("Expected ReviewCommand::Approve"),
        }
    }

    #[test]
    fn test_should_parse_review_request_changes() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "gitflow",
            "review",
            "request-changes",
            "5",
            "--body",
            "Please fix the error handling",
        ])
        .expect("parse");
        match cli.command {
            crate::Commands::Review(ReviewCommand::RequestChanges {
                pr_number,
                body,
                body_file,
            }) => {
                assert_eq!(pr_number, 5);
                assert_eq!(body, Some("Please fix the error handling".into()));
                assert!(body_file.is_none());
            }
            _ => panic!("Expected ReviewCommand::RequestChanges"),
        }
    }

    #[test]
    fn test_should_parse_review_submit() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "gitflow",
            "review",
            "submit",
            "7",
            "--body",
            "Overall good",
            "--event",
            "approved",
        ])
        .expect("parse");
        match cli.command {
            crate::Commands::Review(ReviewCommand::Submit {
                pr_number,
                body,
                body_file,
                event,
            }) => {
                assert_eq!(pr_number, 7);
                assert_eq!(body, Some("Overall good".into()));
                assert!(body_file.is_none());
                assert!(matches!(event, ReviewEventArg::Approved));
            }
            _ => panic!("Expected ReviewCommand::Submit"),
        }
    }

    #[test]
    fn test_should_parse_review_submit_changes_requested() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "gitflow",
            "review",
            "submit",
            "3",
            "--event",
            "changes_requested",
        ])
        .expect("parse");
        match cli.command {
            crate::Commands::Review(ReviewCommand::Submit {
                pr_number, event, ..
            }) => {
                assert_eq!(pr_number, 3);
                assert!(matches!(event, ReviewEventArg::ChangesRequested));
            }
            _ => panic!("Expected ReviewCommand::Submit"),
        }
    }

    #[test]
    fn test_should_parse_review_submit_commented() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "gitflow",
            "review",
            "submit",
            "15",
            "--event",
            "commented",
        ])
        .expect("parse");
        match cli.command {
            crate::Commands::Review(ReviewCommand::Submit { event, .. }) => {
                assert!(matches!(event, ReviewEventArg::Commented));
            }
            _ => panic!("Expected ReviewCommand::Submit"),
        }
    }

    #[test]
    fn test_should_convert_review_event_arg() {
        assert!(matches!(
            ReviewState::from(ReviewEventArg::Approved),
            ReviewState::Approved
        ));
        assert!(matches!(
            ReviewState::from(ReviewEventArg::ChangesRequested),
            ReviewState::ChangesRequested
        ));
        assert!(matches!(
            ReviewState::from(ReviewEventArg::Commented),
            ReviewState::Commented
        ));
    }
}
