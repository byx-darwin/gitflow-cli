//! `gitflow pr` 子命令实现。
//!
//! 提供 Pull Request 的创建、列表、查看、关闭、重新打开、评论、
//! 合并、检出、标记就绪/草稿、同步等功能。
//! Phase 1 仅支持 JSON 输出。

use clap::Subcommand;
use gitflow_cli_core::{
    CliOutput,
    pr::{CreatePrArgs, ListPrArgs, PrProvider},
    types::{MergeStrategy, State},
};
use gitflow_cli_gitcode::GitCodePrProvider;
use gitflow_cli_github::GitHubPrProvider;
use gitflow_cli_gitlab::GitLabMrProvider;

use crate::OutputFormat;

/// PR 子命令集合。
///
/// 支持 `create`、`list`、`view`、`close`、`reopen`、`comment`、
/// `merge`、`checkout`、`ready`、`wip`、`sync` 操作。
#[derive(Debug, Subcommand)]
pub enum PrCommand {
    /// 创建一条新的 Pull Request。
    Create {
        /// PR 标题（必填）。
        #[arg(long)]
        title: String,

        /// 来源分支（可选，默认为当前 git 分支）。
        #[arg(long)]
        head: Option<String>,

        /// 目标分支（可选，默认为 `main`）。
        #[arg(long)]
        base: Option<String>,

        /// PR 正文（可选，与 `--body-file` 二选一）。
        #[arg(long)]
        body: Option<String>,

        /// 从文件读取 PR 正文（可选）。
        #[arg(long = "body-file")]
        body_file: Option<String>,

        /// 是否以草稿方式创建。
        #[arg(long)]
        draft: bool,

        /// 目标仓库（`owner/name` 格式，可选，默认为当前仓库）。
        #[arg(long)]
        repo: Option<String>,
    },

    /// 列出 Pull Request。
    List {
        /// 按状态过滤（`open` 或 `closed`）。
        #[arg(long)]
        state: Option<String>,

        /// 返回数量上限。
        #[arg(long)]
        limit: Option<u32>,
    },

    /// 查看单个 Pull Request 详情。
    View {
        /// PR 编号。
        number: u64,
    },

    /// 关闭 Pull Request。
    Close {
        /// PR 编号。
        number: u64,
    },

    /// 重新打开 Pull Request。
    Reopen {
        /// PR 编号。
        number: u64,
    },

    /// 评论 Pull Request。
    Comment {
        /// PR 编号。
        number: u64,

        /// 评论正文（可选，与 `--body-file` 二选一）。
        #[arg(long)]
        body: Option<String>,

        /// 从文件读取评论正文（可选）。
        #[arg(long = "body-file")]
        body_file: Option<String>,
    },

    /// 合并 Pull Request。
    Merge {
        /// PR 编号。
        number: u64,

        /// 合并策略（`merge`、`squash` 或 `rebase`，默认为 `merge`）。
        #[arg(long)]
        strategy: Option<String>,
    },

    /// 在本地检出 Pull Request 的分支。
    Checkout {
        /// PR 编号。
        number: u64,
    },

    /// 将草稿 PR 标记为可审查状态。
    Ready {
        /// PR 编号。
        number: u64,
    },

    /// 将 PR 标记为草稿状态。
    Wip {
        /// PR 编号。
        number: u64,
    },

    /// 同步 PR 的分支（将 base 分支的最新变更合入 head 分支）。
    Sync {
        /// PR 编号。
        number: u64,
    },
}

/// 处理 `gitflow pr` 子命令。
///
/// 根据 `platform` 选择对应的 PR 提供者，然后执行具体命令并输出结果。
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
/// - `merge` 的 `--strategy` 值非法。
/// - `--head` 未提供且无法检测到当前 git 分支。
/// - JSON 序列化失败。
#[allow(
    clippy::too_many_lines,
    reason = "Command dispatch: each match arm maps to one operation"
)]
pub async fn handle(
    command: PrCommand,
    platform: &str,
    repo: &str,
    output_format: OutputFormat,
) -> miette::Result<()> {
    let provider: Box<dyn PrProvider> = match platform {
        "github" => Box::new(GitHubPrProvider::new(repo)),
        "gitlab" => Box::new(GitLabMrProvider::new(repo)),
        "gitcode" => Box::new(GitCodePrProvider::new(repo)),
        other => {
            return Err(miette::miette!(
                "Platform '{other}' not yet supported for pr commands"
            ));
        }
    };

    match command {
        PrCommand::Create {
            title,
            head,
            base,
            body,
            body_file,
            draft,
            repo: target_repo,
        } => {
            let resolved_body = resolve_body(body, body_file)?;
            let resolved_head = resolve_head(head)?;
            let resolved_base = base.unwrap_or_else(|| "main".to_string());

            let args = CreatePrArgs {
                title,
                body: resolved_body,
                head: resolved_head,
                base: resolved_base,
                draft,
                repo: target_repo,
            };
            let pr = provider
                .create(args)
                .await
                .map_err(|e| miette::miette!("Failed to create pr: {e}"))?;
            let output = CliOutput::success(pr, platform, "pr create");
            print_output(&output, &output_format)?;
        }
        PrCommand::List { state, limit } => {
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

            let args = ListPrArgs {
                state: parsed_state,
                limit,
            };
            let prs = provider
                .list(args)
                .await
                .map_err(|e| miette::miette!("Failed to list prs: {e}"))?;
            let output = CliOutput::success(prs, platform, "pr list");
            print_output(&output, &output_format)?;
        }
        PrCommand::View { number } => {
            let pr = provider
                .view(number)
                .await
                .map_err(|e| miette::miette!("Failed to view pr #{number}: {e}"))?;
            let output = CliOutput::success(pr, platform, "pr view");
            print_output(&output, &output_format)?;
        }
        PrCommand::Close { number } => {
            let pr = provider
                .close(number)
                .await
                .map_err(|e| miette::miette!("Failed to close pr #{number}: {e}"))?;
            let output = CliOutput::success(pr, platform, "pr close");
            print_output(&output, &output_format)?;
        }
        PrCommand::Reopen { number } => {
            let pr = provider
                .reopen(number)
                .await
                .map_err(|e| miette::miette!("Failed to reopen pr #{number}: {e}"))?;
            let output = CliOutput::success(pr, platform, "pr reopen");
            print_output(&output, &output_format)?;
        }
        PrCommand::Comment {
            number,
            body,
            body_file,
        } => {
            let resolved_body = resolve_comment_body(body, body_file)?;
            let comment = provider
                .comment(number, &resolved_body)
                .await
                .map_err(|e| miette::miette!("Failed to comment on pr #{number}: {e}"))?;
            let output = CliOutput::success(comment, platform, "pr comment");
            print_output(&output, &output_format)?;
        }
        PrCommand::Merge { number, strategy } => {
            let parsed_strategy = match strategy.as_deref() {
                Some("merge") => Some(MergeStrategy::Merge),
                Some("squash") => Some(MergeStrategy::Squash),
                Some("rebase") => Some(MergeStrategy::Rebase),
                None => None,
                Some(other) => {
                    return Err(miette::miette!(
                        "Invalid merge strategy '{other}'. Expected 'merge', 'squash', or \
                         'rebase'."
                    ));
                }
            };
            let result = provider
                .merge(number, parsed_strategy)
                .await
                .map_err(|e| miette::miette!("Failed to merge pr #{number}: {e}"))?;
            let output = CliOutput::success(result, platform, "pr merge");
            print_output(&output, &output_format)?;
        }
        PrCommand::Checkout { number } => {
            provider
                .checkout(number)
                .await
                .map_err(|e| miette::miette!("Failed to checkout pr #{number}: {e}"))?;
            let result = serde_json::json!({
                "number": number,
                "checked_out": true,
            });
            let output = CliOutput::success(result, platform, "pr checkout");
            print_output(&output, &output_format)?;
        }
        PrCommand::Ready { number } => {
            let pr = provider
                .mark_ready(number)
                .await
                .map_err(|e| miette::miette!("Failed to mark pr #{number} as ready: {e}"))?;
            let output = CliOutput::success(pr, platform, "pr ready");
            print_output(&output, &output_format)?;
        }
        PrCommand::Wip { number } => {
            let pr = provider
                .mark_wip(number)
                .await
                .map_err(|e| miette::miette!("Failed to mark pr #{number} as draft: {e}"))?;
            let output = CliOutput::success(pr, platform, "pr wip");
            print_output(&output, &output_format)?;
        }
        PrCommand::Sync { number } => {
            provider
                .sync_branch(number)
                .await
                .map_err(|e| miette::miette!("Failed to sync pr #{number} branch: {e}"))?;
            let result = serde_json::json!({
                "number": number,
                "synced": true,
            });
            let output = CliOutput::success(result, platform, "pr sync");
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

/// 解析 `--head` 参数。
///
/// 当未提供时，通过 `git branch --show-current` 检测当前分支。
///
/// # Errors
///
/// - 当 `--head` 未提供且无法通过 git 检测到当前分支时返回错误。
#[allow(
    clippy::disallowed_types,
    reason = "Quick sync `git` call; converting to async adds overhead disproportionate to the \
              work"
)]
fn resolve_head(head: Option<String>) -> miette::Result<String> {
    if let Some(branch) = head {
        return Ok(branch);
    }

    let output = std::process::Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .map_err(|e| miette::miette!("Failed to detect current git branch: {e}"))?;

    if !output.status.success() {
        return Err(miette::miette!(
            "Could not detect current branch. Use --head to specify explicitly."
        ));
    }

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if branch.is_empty() {
        return Err(miette::miette!(
            "Current git HEAD is detached. Use --head to specify the source branch explicitly."
        ));
    }

    Ok(branch)
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
        let path = dir.join("gitflow_test_pr_body.md");
        std::fs::write(&path, "pr body from file").expect("write temp file");
        let result = resolve_body(None, Some(path.to_string_lossy().into_owned()));
        let _ = std::fs::remove_file(&path);
        assert!(result.is_ok());
        assert_eq!(
            result.expect("already checked"),
            Some("pr body from file".into())
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
        let result = resolve_comment_body(Some("LGTM".into()), None);
        assert!(result.is_ok());
        assert_eq!(result.expect("already checked"), "LGTM");
    }

    #[test]
    fn test_should_resolve_head_with_explicit_value() {
        let result = resolve_head(Some("feature/my-branch".into()));
        assert!(result.is_ok());
        assert_eq!(result.expect("already checked"), "feature/my-branch");
    }

    #[test]
    fn test_should_print_json_output() {
        let value = serde_json::json!({"number": 1, "title": "test"});
        let result = print_output(&value, &OutputFormat::Json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_should_accept_text_output() {
        let value = serde_json::json!({"number": 1});
        let result = print_output(&value, &OutputFormat::Text);
        assert!(result.is_ok());
    }

    // --- PrCommand 解析测试 ---

    #[test]
    fn test_should_parse_pr_close() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from(["gitflow", "pr", "close", "42"]).expect("parse");
        match cli.command {
            crate::Commands::Pr(PrCommand::Close { number }) => {
                assert_eq!(number, 42);
            }
            _ => panic!("Expected PrCommand::Close"),
        }
    }

    #[test]
    fn test_should_parse_pr_reopen() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from(["gitflow", "pr", "reopen", "7"]).expect("parse");
        match cli.command {
            crate::Commands::Pr(PrCommand::Reopen { number }) => {
                assert_eq!(number, 7);
            }
            _ => panic!("Expected PrCommand::Reopen"),
        }
    }

    #[test]
    fn test_should_parse_pr_comment_with_body() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from(["gitflow", "pr", "comment", "10", "--body", "LGTM"])
            .expect("parse");
        match cli.command {
            crate::Commands::Pr(PrCommand::Comment {
                number,
                body,
                body_file,
            }) => {
                assert_eq!(number, 10);
                assert_eq!(body, Some("LGTM".into()));
                assert!(body_file.is_none());
            }
            _ => panic!("Expected PrCommand::Comment"),
        }
    }

    #[test]
    fn test_should_parse_pr_merge_with_strategy() {
        use clap::Parser;
        let cli =
            crate::Cli::try_parse_from(["gitflow", "pr", "merge", "5", "--strategy", "squash"])
                .expect("parse");
        match cli.command {
            crate::Commands::Pr(PrCommand::Merge { number, strategy }) => {
                assert_eq!(number, 5);
                assert_eq!(strategy, Some("squash".into()));
            }
            _ => panic!("Expected PrCommand::Merge"),
        }
    }

    #[test]
    fn test_should_parse_pr_merge_without_strategy() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from(["gitflow", "pr", "merge", "3"]).expect("parse");
        match cli.command {
            crate::Commands::Pr(PrCommand::Merge { number, strategy }) => {
                assert_eq!(number, 3);
                assert!(strategy.is_none());
            }
            _ => panic!("Expected PrCommand::Merge"),
        }
    }

    #[test]
    fn test_should_parse_pr_checkout() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from(["gitflow", "pr", "checkout", "15"]).expect("parse");
        match cli.command {
            crate::Commands::Pr(PrCommand::Checkout { number }) => {
                assert_eq!(number, 15);
            }
            _ => panic!("Expected PrCommand::Checkout"),
        }
    }

    #[test]
    fn test_should_parse_pr_ready() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from(["gitflow", "pr", "ready", "8"]).expect("parse");
        match cli.command {
            crate::Commands::Pr(PrCommand::Ready { number }) => {
                assert_eq!(number, 8);
            }
            _ => panic!("Expected PrCommand::Ready"),
        }
    }

    #[test]
    fn test_should_parse_pr_wip() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from(["gitflow", "pr", "wip", "12"]).expect("parse");
        match cli.command {
            crate::Commands::Pr(PrCommand::Wip { number }) => {
                assert_eq!(number, 12);
            }
            _ => panic!("Expected PrCommand::Wip"),
        }
    }

    #[test]
    fn test_should_parse_pr_sync() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from(["gitflow", "pr", "sync", "20"]).expect("parse");
        match cli.command {
            crate::Commands::Pr(PrCommand::Sync { number }) => {
                assert_eq!(number, 20);
            }
            _ => panic!("Expected PrCommand::Sync"),
        }
    }
}
