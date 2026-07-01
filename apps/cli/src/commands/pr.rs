//! `gitflow pr` 子命令实现。
//!
//! 提供 Pull Request 的创建、列表和查看功能，支持通过 clap 解析参数后
//! 调用对应平台的 [`PrProvider`] 实现。Phase 1 仅支持 JSON 输出。

use clap::Subcommand;
use gitflow_cli_core::{
    pr::{CreatePrArgs, ListPrArgs, PrProvider},
    types::State,
};
use gitflow_cli_github::GitHubPrProvider;

use crate::OutputFormat;

/// PR 子命令集合。
///
/// 支持 `create`、`list`、`view` 三种操作，每种操作对应不同的 clap 参数。
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

        /// 从文件读取 PR 正文（可选，Phase 1 暂未实现）。
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
}

/// 处理 `gitflow pr` 子命令。
///
/// 根据 `platform` 选择对应的 PR 提供者，然后执行具体命令并输出结果。
/// Phase 1 仅支持 `github` 平台与 JSON 输出格式。
///
/// # Errors
///
/// 返回错误当：
/// - 平台暂不支持（如 `gitlab`）。
/// - 底层 provider 调用失败（如 `gh` CLI 执行失败）。
/// - `--body-file` 在 Phase 1 中被使用。
/// - `--head` 未提供且无法检测到当前 git 分支。
/// - JSON 序列化失败。
pub async fn handle(
    command: PrCommand,
    platform: &str,
    repo: &str,
    output_format: OutputFormat,
) -> miette::Result<()> {
    let provider: Box<dyn PrProvider> = match platform {
        "github" => Box::new(GitHubPrProvider::new(repo)),
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
            print_output(&pr, &output_format)?;
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
            print_output(&prs, &output_format)?;
        }
        PrCommand::View { number } => {
            let pr = provider
                .view(number)
                .await
                .map_err(|e| miette::miette!("Failed to view pr #{number}: {e}"))?;
            print_output(&pr, &output_format)?;
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
    fn test_should_reject_text_output_in_phase1() {
        let value = serde_json::json!({"number": 1});
        let result = print_output(&value, &OutputFormat::Text);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not yet supported"));
    }
}
