//! `gitflow release` 子命令实现。
//!
//! 提供 Release 的创建、列表、查看、编辑、上传/下载资源、删除等功能，
//! 支持通过 clap 解析参数后调用对应平台的 [`ReleaseProvider`] 实现。
//! Phase 1 仅支持 JSON 输出。

use clap::Subcommand;
use gitflow_cli_core::{
    CliOutput,
    release::{CreateReleaseArgs, ReleaseProvider},
};
use gitflow_cli_gitcode::GitCodeReleaseProvider;
use gitflow_cli_github::GitHubReleaseProvider;
use gitflow_cli_gitlab::GitLabReleaseProvider;

use crate::OutputFormat;

/// Release 子命令集合。
///
/// 支持 `create`、`list`、`view`、`edit`、`upload`、
/// `download`、`delete` 操作，每种操作对应不同的 clap 参数。
#[derive(Debug, Subcommand)]
pub enum ReleaseCommand {
    /// 创建一个新的 Release。
    Create {
        /// Git tag 名称（必填）。
        #[arg(long)]
        tag_name: String,

        /// Release 标题（可选）。
        #[arg(long)]
        name: Option<String>,

        /// Release 正文（可选，与 `--body-file` 二选一）。
        #[arg(long)]
        body: Option<String>,

        /// 从文件读取 Release 正文（可选）。
        #[arg(long = "body-file")]
        body_file: Option<String>,

        /// 以草稿方式创建。
        #[arg(long, default_value_t = false)]
        draft: bool,

        /// 以预发布方式创建。
        #[arg(long, default_value_t = false)]
        prerelease: bool,

        /// 目标 commitish（可选，默认为当前分支 HEAD）。
        #[arg(long)]
        target_commitish: Option<String>,
    },

    /// 列出 Release。
    List {
        /// 返回数量上限（可选）。
        #[arg(long)]
        limit: Option<u32>,
    },

    /// 查看指定 tag 的 Release 详情。
    View {
        /// Git tag 名称。
        tag: String,
    },

    /// 编辑指定 Release 的元数据。
    Edit {
        /// Git tag 名称。
        tag: String,

        /// Release 标题（可选）。
        #[arg(long)]
        name: Option<String>,

        /// Release 正文（可选，与 `--body-file` 二选一）。
        #[arg(long)]
        body: Option<String>,

        /// 从文件读取 Release 正文（可选）。
        #[arg(long = "body-file")]
        body_file: Option<String>,
    },

    /// 上传资源文件到 Release。
    Upload {
        /// Git tag 名称。
        tag: String,

        /// 本地文件路径。
        #[arg(long)]
        file: String,
    },

    /// 下载 Release 的资源文件。
    Download {
        /// Git tag 名称。
        tag: String,

        /// 文件名匹配模式（可选）。
        #[arg(long)]
        pattern: Option<String>,

        /// 下载目录（可选）。
        #[arg(long)]
        dir: Option<String>,
    },

    /// 删除指定 Release。
    Delete {
        /// Git tag 名称。
        tag: String,

        /// 跳过确认提示。
        #[arg(long, short, default_value_t = false)]
        yes: bool,
    },
}

/// 处理 `gitflow release` 子命令。
///
/// 根据 `platform` 选择对应的 Release 提供者，然后执行具体命令并输出结果。
/// 支持 `github`、`gitlab`、`gitcode` 三个平台，Phase 1 仅支持 JSON 输出格式。
///
/// # Errors
///
/// 返回错误当：
/// - 平台暂不支持。
/// - 底层 provider 调用失败。
/// - `--body` 与 `--body-file` 同时提供。
/// - `--body-file` 文件读取失败。
/// - JSON 序列化失败。
#[allow(
    clippy::too_many_lines,
    reason = "Command dispatch: each match arm maps to one operation"
)]
pub async fn handle(
    command: ReleaseCommand,
    platform: &str,
    repo: &str,
    output_format: OutputFormat,
) -> miette::Result<()> {
    let provider: Box<dyn ReleaseProvider> = match platform {
        "github" => Box::new(GitHubReleaseProvider::new(repo)),
        "gitlab" => Box::new(GitLabReleaseProvider::new(repo)),
        "gitcode" => Box::new(GitCodeReleaseProvider::new(repo)),
        other => {
            return Err(miette::miette!(
                "Platform '{other}' not yet supported for release commands"
            ));
        }
    };

    match command {
        ReleaseCommand::Create {
            tag_name,
            name,
            body,
            body_file,
            draft,
            prerelease,
            target_commitish,
        } => {
            let resolved_body = resolve_body(body, body_file)?;
            let args = CreateReleaseArgs {
                tag_name,
                name,
                body: resolved_body,
                draft,
                prerelease,
                target_commitish,
            };
            let release = provider
                .create(args)
                .await
                .map_err(|e| miette::miette!("Failed to create release: {e}"))?;
            let output = CliOutput::success(release, platform, "release create");
            print_output(&output, &output_format)?;
        }
        ReleaseCommand::List { .. } => {
            let releases = provider
                .list()
                .await
                .map_err(|e| miette::miette!("Failed to list releases: {e}"))?;
            let output = CliOutput::success(releases, platform, "release list");
            print_output(&output, &output_format)?;
        }
        ReleaseCommand::View { tag } => {
            let release = provider
                .view(&tag)
                .await
                .map_err(|e| miette::miette!("Failed to view release '{tag}': {e}"))?;
            let output = CliOutput::success(release, platform, "release view");
            print_output(&output, &output_format)?;
        }
        ReleaseCommand::Edit {
            tag,
            name,
            body,
            body_file,
        } => {
            let resolved_body = resolve_body(body, body_file)?;
            let args = CreateReleaseArgs {
                tag_name: tag.clone(),
                name,
                body: resolved_body,
                draft: false,
                prerelease: false,
                target_commitish: None,
            };
            let release = provider
                .edit(&tag, args)
                .await
                .map_err(|e| miette::miette!("Failed to edit release '{tag}': {e}"))?;
            let output = CliOutput::success(release, platform, "release edit");
            print_output(&output, &output_format)?;
        }
        ReleaseCommand::Upload { tag, file } => {
            let asset_name = std::path::Path::new(&file)
                .file_name()
                .map_or_else(|| file.clone(), |n| n.to_string_lossy().to_string());
            provider
                .upload_asset(&tag, &file, &asset_name)
                .await
                .map_err(|e| miette::miette!("Failed to upload asset to release '{tag}': {e}"))?;
            let result = serde_json::json!({
                "tag_name": tag,
                "asset_file": asset_name,
            });
            let output = CliOutput::success(result, platform, "release upload");
            print_output(&output, &output_format)?;
        }
        ReleaseCommand::Download { tag, pattern, dir } => {
            let asset_name = pattern.unwrap_or_else(|| "*".into());
            let dest = dir.unwrap_or_else(|| ".".into());
            provider
                .download_asset(&tag, &asset_name, &dest)
                .await
                .map_err(|e| {
                    miette::miette!("Failed to download asset from release '{tag}': {e}")
                })?;
            let result = serde_json::json!({
                "tag_name": tag,
                "pattern": asset_name,
                "dest": dest,
            });
            let output = CliOutput::success(result, platform, "release download");
            print_output(&output, &output_format)?;
        }
        ReleaseCommand::Delete { tag, .. } => {
            provider
                .delete(&tag)
                .await
                .map_err(|e| miette::miette!("Failed to delete release '{tag}': {e}"))?;
            let result = serde_json::json!({
                "tag_name": tag,
                "deleted": true,
            });
            let output = CliOutput::success(result, platform, "release delete");
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
        let result = resolve_body(Some("release notes".into()), None);
        assert!(result.is_ok());
        assert_eq!(
            result.expect("already checked"),
            Some("release notes".into())
        );
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
        let path = dir.join("gitflow_release_body.md");
        std::fs::write(&path, "release body content").expect("write temp file");
        let result = resolve_body(None, Some(path.to_string_lossy().into_owned()));
        let _ = std::fs::remove_file(&path);
        assert!(result.is_ok());
        assert_eq!(
            result.expect("already checked"),
            Some("release body content".into())
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
    fn test_should_print_json_output() {
        let value = serde_json::json!({"tag_name": "v1.0.0", "name": "v1.0.0"});
        let result = print_output(&value, &OutputFormat::Json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_should_reject_text_output_in_phase1() {
        let value = serde_json::json!({"tag_name": "v1.0.0"});
        let result = print_output(&value, &OutputFormat::Text);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not yet supported"));
    }

    // --- ReleaseCommand 解析测试 ---

    #[test]
    fn test_should_parse_release_create() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "gitflow",
            "release",
            "create",
            "--tag-name",
            "v1.0.0",
            "--name",
            "v1.0.0",
            "--body",
            "Initial release",
        ])
        .expect("parse");
        match cli.command {
            crate::Commands::Release(ReleaseCommand::Create {
                tag_name,
                name,
                body,
                body_file,
                draft,
                prerelease,
                target_commitish,
            }) => {
                assert_eq!(tag_name, "v1.0.0");
                assert_eq!(name, Some("v1.0.0".into()));
                assert_eq!(body, Some("Initial release".into()));
                assert!(body_file.is_none());
                assert!(!draft);
                assert!(!prerelease);
                assert!(target_commitish.is_none());
            }
            _ => panic!("Expected ReleaseCommand::Create"),
        }
    }

    #[test]
    fn test_should_parse_release_list() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from(["gitflow", "release", "list", "--limit", "10"])
            .expect("parse");
        match cli.command {
            crate::Commands::Release(ReleaseCommand::List { limit }) => {
                assert_eq!(limit, Some(10));
            }
            _ => panic!("Expected ReleaseCommand::List"),
        }
    }

    #[test]
    fn test_should_parse_release_view() {
        use clap::Parser;
        let cli =
            crate::Cli::try_parse_from(["gitflow", "release", "view", "v1.0.0"]).expect("parse");
        match cli.command {
            crate::Commands::Release(ReleaseCommand::View { tag }) => {
                assert_eq!(tag, "v1.0.0");
            }
            _ => panic!("Expected ReleaseCommand::View"),
        }
    }

    #[test]
    fn test_should_parse_release_edit() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "gitflow",
            "release",
            "edit",
            "v1.0.0",
            "--name",
            "v1.0.1",
            "--body",
            "Updated notes",
        ])
        .expect("parse");
        match cli.command {
            crate::Commands::Release(ReleaseCommand::Edit {
                tag,
                name,
                body,
                body_file,
            }) => {
                assert_eq!(tag, "v1.0.0");
                assert_eq!(name, Some("v1.0.1".into()));
                assert_eq!(body, Some("Updated notes".into()));
                assert!(body_file.is_none());
            }
            _ => panic!("Expected ReleaseCommand::Edit"),
        }
    }

    #[test]
    fn test_should_parse_release_upload() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "gitflow",
            "release",
            "upload",
            "v1.0.0",
            "--file",
            "./artifact.tar.gz",
        ])
        .expect("parse");
        match cli.command {
            crate::Commands::Release(ReleaseCommand::Upload { tag, file }) => {
                assert_eq!(tag, "v1.0.0");
                assert_eq!(file, "./artifact.tar.gz");
            }
            _ => panic!("Expected ReleaseCommand::Upload"),
        }
    }

    #[test]
    fn test_should_parse_release_download() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "gitflow",
            "release",
            "download",
            "v1.0.0",
            "--pattern",
            "*.tar.gz",
            "--dir",
            "./downloads",
        ])
        .expect("parse");
        match cli.command {
            crate::Commands::Release(ReleaseCommand::Download { tag, pattern, dir }) => {
                assert_eq!(tag, "v1.0.0");
                assert_eq!(pattern, Some("*.tar.gz".into()));
                assert_eq!(dir, Some("./downloads".into()));
            }
            _ => panic!("Expected ReleaseCommand::Download"),
        }
    }

    #[test]
    fn test_should_parse_release_delete() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from(["gitflow", "release", "delete", "v1.0.0", "--yes"])
            .expect("parse");
        match cli.command {
            crate::Commands::Release(ReleaseCommand::Delete { tag, yes }) => {
                assert_eq!(tag, "v1.0.0");
                assert!(yes);
            }
            _ => panic!("Expected ReleaseCommand::Delete"),
        }
    }

    #[test]
    fn test_should_parse_release_create_with_draft_and_prerelease() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "gitflow",
            "release",
            "create",
            "--tag-name",
            "v2.0.0-beta",
            "--draft",
            "--prerelease",
            "--target-commitish",
            "abc123",
        ])
        .expect("parse");
        match cli.command {
            crate::Commands::Release(ReleaseCommand::Create {
                tag_name,
                draft,
                prerelease,
                target_commitish,
                ..
            }) => {
                assert_eq!(tag_name, "v2.0.0-beta");
                assert!(draft);
                assert!(prerelease);
                assert_eq!(target_commitish, Some("abc123".into()));
            }
            _ => panic!("Expected ReleaseCommand::Create"),
        }
    }
}
