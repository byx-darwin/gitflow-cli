//! `gitflow commit` 子命令实现。
//!
//! 提供 Commit 的查看、Diff/Patch 获取、评论等功能，
//! 支持通过 clap 解析参数后调用对应平台的 [`CommitProvider`] 实现。
//! Phase 1 仅支持 JSON 输出。

use clap::Subcommand;
use gitflow_cli_core::{CliOutput, commit::CommitProvider};
use gitflow_cli_github::GitHubCommitProvider;

use crate::OutputFormat;

/// Commit 查看和操作子命令集合。
///
/// 支持 `view`、`diff`、`patch`、`comment` 操作。
#[derive(Debug, Subcommand)]
pub enum CommitCommand {
    /// 查看 Commit 详情。
    View {
        /// Commit SHA 哈希值。
        sha: String,
    },

    /// 获取 Commit 的 unified diff 输出。
    Diff {
        /// Commit SHA 哈希值。
        sha: String,
    },

    /// 获取 Commit 的原始 patch 内容。
    Patch {
        /// Commit SHA 哈希值。
        sha: String,
    },

    /// 评论 Commit 中的特定文件行。
    Comment {
        /// Commit SHA 哈希值。
        sha: String,

        /// 评论内容（必填，与 `--body-file` 二选一）。
        #[arg(long)]
        body: Option<String>,

        /// 从文件读取评论内容（可选）。
        #[arg(long = "body-file")]
        body_file: Option<String>,

        /// 文件路径（相对于仓库根目录，必填）。
        #[arg(long)]
        path: Option<String>,

        /// 评论的行号（1-based，必填）。
        #[arg(long)]
        line: Option<u64>,
    },
}

/// 处理 `gitflow commit` 子命令。
///
/// 根据 `platform` 选择对应的 Commit 提供者，然后执行具体命令并输出结果。
/// Phase 1 仅支持 `github` 平台与 JSON 输出格式。
///
/// # Errors
///
/// 返回错误当：
/// - 平台暂不支持。
/// - 底层 provider 调用失败。
/// - `comment` 命令未提供评论内容或必需的 `--path`/`--line`。
/// - `--body` 与 `--body-file` 同时提供。
/// - `--body-file` 文件读取失败。
/// - JSON 序列化失败。
#[allow(
    clippy::too_many_lines,
    reason = "Command dispatch: each match arm maps to one operation"
)]
pub async fn handle(
    command: CommitCommand,
    platform: &str,
    repo: &str,
    output_format: OutputFormat,
) -> miette::Result<()> {
    let provider: Box<dyn CommitProvider> = match platform {
        "github" => Box::new(GitHubCommitProvider::new(repo)),
        other => {
            return Err(miette::miette!(
                "Platform '{other}' not yet supported for commit commands"
            ));
        }
    };

    match command {
        CommitCommand::View { sha } => {
            let detail = provider
                .view(&sha)
                .await
                .map_err(|e| miette::miette!("Failed to view commit {sha}: {e}"))?;
            let output = CliOutput::success(detail, platform, "commit view");
            print_output(&output, &output_format)?;
        }
        CommitCommand::Diff { sha } => {
            let diff = provider
                .diff(&sha)
                .await
                .map_err(|e| miette::miette!("Failed to get diff for commit {sha}: {e}"))?;
            let output = CliOutput::success(
                serde_json::json!({ "sha": sha, "diff": diff }),
                platform,
                "commit diff",
            );
            print_output(&output, &output_format)?;
        }
        CommitCommand::Patch { sha } => {
            let patch = provider
                .patch(&sha)
                .await
                .map_err(|e| miette::miette!("Failed to get patch for commit {sha}: {e}"))?;
            let output = CliOutput::success(
                serde_json::json!({ "sha": sha, "patch": patch }),
                platform,
                "commit patch",
            );
            print_output(&output, &output_format)?;
        }
        CommitCommand::Comment {
            sha,
            body,
            body_file,
            path,
            line,
        } => {
            let resolved_body = resolve_comment_body(body, body_file)?;
            let path = path.ok_or_else(|| {
                miette::miette!("--path is required for commit comment. Specify the file path.")
            })?;
            let line = line.ok_or_else(|| {
                miette::miette!("--line is required for commit comment. Specify the line number.")
            })?;

            provider
                .comment(&sha, &resolved_body, &path, line)
                .await
                .map_err(|e| {
                    miette::miette!("Failed to comment on commit {sha} at {path}:{line}: {e}")
                })?;

            let result = serde_json::json!({
                "sha": sha,
                "path": path,
                "line": line,
                "commented": true,
            });
            let output = CliOutput::success(result, platform, "commit comment");
            print_output(&output, &output_format)?;
        }
    }

    Ok(())
}

/// 解析评论正文，要求必须提供 `--body` 或 `--body-file` 之一。
///
/// # Errors
///
/// - 当两者都未提供时返回错误。
/// - 当同时提供两者时返回错误。
/// - 当文件读取失败时返回错误。
#[allow(
    clippy::disallowed_methods,
    reason = "Sync file read; called from async handler but file body input is small and local"
)]
fn resolve_comment_body(body: Option<String>, body_file: Option<String>) -> miette::Result<String> {
    if body.is_some() && body_file.is_some() {
        return Err(miette::miette!(
            "Cannot specify both --body and --body-file"
        ));
    }
    if let Some(path) = body_file {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| miette::miette!("Failed to read body file '{path}': {e}"))?;
        return Ok(content);
    }
    body.ok_or_else(|| miette::miette!("Comment body is required. Use --body or --body-file."))
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

    // --- resolve_comment_body 测试 ---

    #[test]
    fn test_should_resolve_comment_body_with_body() {
        let result = resolve_comment_body(Some("LGTM".into()), None);
        assert!(result.is_ok());
        assert_eq!(result.expect("already checked"), "LGTM");
    }

    #[test]
    fn test_should_require_comment_body() {
        let result = resolve_comment_body(None, None);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Comment body is required"));
    }

    #[test]
    fn test_should_reject_both_body_and_body_file() {
        let result = resolve_comment_body(Some("hello".into()), Some("/tmp/body.md".into()));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Cannot specify both"));
    }

    #[test]
    fn test_should_resolve_comment_body_from_file() {
        let dir = std::env::temp_dir();
        let path = dir.join("gitflow_test_commit_comment.md");
        std::fs::write(&path, "commit comment from file").expect("write temp file");
        let result = resolve_comment_body(None, Some(path.to_string_lossy().into_owned()));
        let _ = std::fs::remove_file(&path);
        assert!(result.is_ok());
        assert_eq!(result.expect("already checked"), "commit comment from file");
    }

    #[test]
    fn test_should_error_on_missing_body_file() {
        let result = resolve_comment_body(None, Some("/nonexistent/path/comment.md".into()));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Failed to read body file"));
    }

    // --- print_output 测试 ---

    #[test]
    fn test_should_print_json_output() {
        let value = serde_json::json!({"sha": "abc123", "message": "test"});
        let result = print_output(&value, &OutputFormat::Json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_should_reject_text_output_in_phase1() {
        let value = serde_json::json!({"sha": "abc"});
        let result = print_output(&value, &OutputFormat::Text);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not yet supported"));
    }

    // --- CommitCommand 解析测试 ---

    #[test]
    fn test_should_parse_commit_view() {
        use clap::Parser;
        let cli =
            crate::Cli::try_parse_from(["gitflow", "commit", "view", "abc123def"]).expect("parse");
        match cli.command {
            crate::Commands::Commit(CommitCommand::View { sha }) => {
                assert_eq!(sha, "abc123def");
            }
            _ => panic!("Expected CommitCommand::View"),
        }
    }

    #[test]
    fn test_should_parse_commit_diff() {
        use clap::Parser;
        let cli =
            crate::Cli::try_parse_from(["gitflow", "commit", "diff", "deadbeef"]).expect("parse");
        match cli.command {
            crate::Commands::Commit(CommitCommand::Diff { sha }) => {
                assert_eq!(sha, "deadbeef");
            }
            _ => panic!("Expected CommitCommand::Diff"),
        }
    }

    #[test]
    fn test_should_parse_commit_patch() {
        use clap::Parser;
        let cli =
            crate::Cli::try_parse_from(["gitflow", "commit", "patch", "f00ba42"]).expect("parse");
        match cli.command {
            crate::Commands::Commit(CommitCommand::Patch { sha }) => {
                assert_eq!(sha, "f00ba42");
            }
            _ => panic!("Expected CommitCommand::Patch"),
        }
    }

    #[test]
    fn test_should_parse_commit_comment() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "gitflow",
            "commit",
            "comment",
            "abc123",
            "--body",
            "This looks suspicious",
            "--path",
            "src/auth.rs",
            "--line",
            "42",
        ])
        .expect("parse");
        match cli.command {
            crate::Commands::Commit(CommitCommand::Comment {
                sha,
                body,
                body_file,
                path,
                line,
            }) => {
                assert_eq!(sha, "abc123");
                assert_eq!(body, Some("This looks suspicious".into()));
                assert!(body_file.is_none());
                assert_eq!(path, Some("src/auth.rs".into()));
                assert_eq!(line, Some(42));
            }
            _ => panic!("Expected CommitCommand::Comment"),
        }
    }

    #[test]
    fn test_should_parse_commit_comment_with_body_file() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "gitflow",
            "commit",
            "comment",
            "abc123",
            "--body-file",
            "/tmp/comment.md",
            "--path",
            "lib.rs",
            "--line",
            "10",
        ])
        .expect("parse");
        match cli.command {
            crate::Commands::Commit(CommitCommand::Comment {
                sha,
                body,
                body_file,
                path,
                line,
            }) => {
                assert_eq!(sha, "abc123");
                assert!(body.is_none());
                assert_eq!(body_file, Some("/tmp/comment.md".into()));
                assert_eq!(path, Some("lib.rs".into()));
                assert_eq!(line, Some(10));
            }
            _ => panic!("Expected CommitCommand::Comment"),
        }
    }
}
