//! `gitflow label` 和 `gitflow milestone` 子命令实现。
//!
//! 提供仓库标签（Label）和里程碑（Milestone）的创建、列表、编辑、删除等操作，
//! 支持通过 clap 解析参数后调用对应平台的 [`LabelProvider`] 和 [`MilestoneProvider`] 实现。
//! Phase 1 仅支持 JSON 输出。

use clap::Subcommand;
use gitflow_cli_core::{
    CliOutput,
    label::{CreateLabelArgs, CreateMilestoneArgs, LabelProvider, MilestoneProvider},
};
use gitflow_cli_github::{GitHubLabelProvider, GitHubMilestoneProvider};

use crate::OutputFormat;

/// 标签（Label）管理子命令集合。
///
/// 支持 `create`、`list`、`edit`、`delete` 操作。
#[derive(Debug, Subcommand)]
pub enum LabelCommand {
    /// 创建一个新的标签。
    Create {
        /// 标签名称（必填）。
        name: String,

        /// 标签颜色（必填，十六进制格式，如 `d73a4a`）。
        #[arg(long)]
        color: String,

        /// 标签描述（可选）。
        #[arg(long)]
        description: Option<String>,
    },

    /// 列出仓库中的所有标签。
    List,

    /// 编辑一个已有的标签。
    Edit {
        /// 要编辑的标签名称。
        name: String,

        /// 新的标签颜色（可选）。
        #[arg(long)]
        color: Option<String>,

        /// 新的标签描述（可选）。
        #[arg(long)]
        description: Option<String>,
    },

    /// 删除一个标签。
    Delete {
        /// 要删除的标签名称。
        name: String,

        /// 跳过确认提示（默认跳过确认直接删除）。
        #[arg(short, long)]
        yes: bool,
    },
}

/// 里程碑（Milestone）管理子命令集合。
///
/// 支持 `create`、`list`、`edit`、`close`、`reopen` 操作。
#[derive(Debug, Subcommand)]
pub enum MilestoneCommand {
    /// 创建一个新的里程碑。
    Create {
        /// 里程碑标题（必填）。
        #[arg(long)]
        title: String,

        /// 里程碑描述（可选）。
        #[arg(long)]
        description: Option<String>,

        /// 截止日期（可选，RFC 3339 格式）。
        #[arg(long)]
        due_on: Option<String>,
    },

    /// 列出仓库中的所有里程碑。
    List,

    /// 编辑一个已有的里程碑。
    Edit {
        /// 里程碑编号。
        number: u64,

        /// 新的标题（可选）。
        #[arg(long)]
        title: Option<String>,

        /// 新的描述（可选）。
        #[arg(long)]
        description: Option<String>,

        /// 新的截止日期（可选，RFC 3339 格式）。
        #[arg(long)]
        due_on: Option<String>,
    },

    /// 关闭一个里程碑。
    Close {
        /// 里程碑编号。
        number: u64,
    },

    /// 重新打开一个已关闭的里程碑。
    Reopen {
        /// 里程碑编号。
        number: u64,
    },
}

/// 处理 `gitflow label` 子命令。
///
/// 根据 `platform` 选择对应的 Label 提供者，然后执行具体命令并输出结果。
/// Phase 1 仅支持 `github` 平台与 JSON 输出格式。
///
/// # Errors
///
/// 返回错误当：
/// - 平台暂不支持。
/// - 底层 provider 调用失败。
/// - 编辑标签时未提供任何可编辑的字段。
/// - JSON 序列化失败。
pub async fn handle_label(
    command: LabelCommand,
    platform: &str,
    repo: &str,
    output_format: OutputFormat,
) -> miette::Result<()> {
    let provider: Box<dyn LabelProvider> = match platform {
        "github" => Box::new(GitHubLabelProvider::new(repo)),
        other => {
            return Err(miette::miette!(
                "Platform '{other}' not yet supported for label commands"
            ));
        }
    };

    match command {
        LabelCommand::Create {
            name,
            color,
            description,
        } => {
            let args = CreateLabelArgs {
                name,
                color,
                description,
            };
            let label = provider
                .create(args)
                .await
                .map_err(|e| miette::miette!("Failed to create label: {e}"))?;
            let output = CliOutput::success(label, platform, "label create");
            print_output(&output, &output_format)?;
        }
        LabelCommand::List => {
            let labels = provider
                .list()
                .await
                .map_err(|e| miette::miette!("Failed to list labels: {e}"))?;
            let output = CliOutput::success(labels, platform, "label list");
            print_output(&output, &output_format)?;
        }
        LabelCommand::Edit {
            name,
            color,
            description,
        } => {
            // 编辑时需要提供 color，如果未提供则先 fetch 当前标签获取原有 color
            let (color_val, desc_val) = match (&color, &description) {
                (Some(c), d) => (c.clone(), d.clone()),
                (None, Some(d)) => {
                    // 仅更新描述，需要先获取当前标签的 color
                    let current = provider
                        .list()
                        .await
                        .map_err(|e| miette::miette!("Failed to list labels for edit: {e}"))?;
                    let existing = current
                        .iter()
                        .find(|l| l.name == name)
                        .ok_or_else(|| miette::miette!("Label '{name}' not found"))?;
                    let c = existing
                        .color
                        .clone()
                        .ok_or_else(|| miette::miette!("Label '{name}' has no color set"))?;
                    (c, Some(d.clone()))
                }
                (None, None) => {
                    return Err(miette::miette!(
                        "At least one of --color or --description is required for edit"
                    ));
                }
            };
            let args = CreateLabelArgs {
                name: name.clone(),
                color: color_val,
                description: desc_val,
            };
            let label = provider
                .edit(&name, args)
                .await
                .map_err(|e| miette::miette!("Failed to edit label '{name}': {e}"))?;
            let output = CliOutput::success(label, platform, "label edit");
            print_output(&output, &output_format)?;
        }
        LabelCommand::Delete { name, yes } => {
            if !yes {
                tracing::info!("Deleting label '{name}' (use --yes to skip confirmation)");
            }
            provider
                .delete(&name)
                .await
                .map_err(|e| miette::miette!("Failed to delete label '{name}': {e}"))?;
            let result = serde_json::json!({
                "label": name,
                "deleted": true,
            });
            let output = CliOutput::success(result, platform, "label delete");
            print_output(&output, &output_format)?;
        }
    }

    Ok(())
}

/// 处理 `gitflow milestone` 子命令。
///
/// 根据 `platform` 选择对应的 Milestone 提供者，然后执行具体命令并输出结果。
/// Phase 1 仅支持 `github` 平台与 JSON 输出格式。
///
/// # Errors
///
/// 返回错误当：
/// - 平台暂不支持。
/// - 底层 provider 调用失败。
/// - 编辑里程碑时未提供任何可编辑的字段。
/// - `due_on` 日期解析失败。
/// - JSON 序列化失败。
#[allow(
    clippy::too_many_lines,
    reason = "Command dispatch: each match arm maps to one operation"
)]
pub async fn handle_milestone(
    command: MilestoneCommand,
    platform: &str,
    repo: &str,
    output_format: OutputFormat,
) -> miette::Result<()> {
    let provider: Box<dyn MilestoneProvider> = match platform {
        "github" => Box::new(GitHubMilestoneProvider::new(repo)),
        other => {
            return Err(miette::miette!(
                "Platform '{other}' not yet supported for milestone commands"
            ));
        }
    };

    match command {
        MilestoneCommand::Create {
            title,
            description,
            due_on,
        } => {
            let parsed_due_on = if let Some(ref s) = due_on {
                Some(
                    chrono::DateTime::parse_from_rfc3339(s)
                        .map_err(|e| miette::miette!("Invalid RFC 3339 date for --due-on: {e}"))?
                        .with_timezone(&chrono::Utc),
                )
            } else {
                None
            };

            let args = CreateMilestoneArgs {
                title,
                description,
                due_on: parsed_due_on,
            };
            let milestone = provider
                .create(args)
                .await
                .map_err(|e| miette::miette!("Failed to create milestone: {e}"))?;
            let output = CliOutput::success(milestone, platform, "milestone create");
            print_output(&output, &output_format)?;
        }
        MilestoneCommand::List => {
            let milestones = provider
                .list()
                .await
                .map_err(|e| miette::miette!("Failed to list milestones: {e}"))?;
            let output = CliOutput::success(milestones, platform, "milestone list");
            print_output(&output, &output_format)?;
        }
        MilestoneCommand::Edit {
            number,
            title,
            description,
            due_on,
        } => {
            if title.is_none() && description.is_none() && due_on.is_none() {
                return Err(miette::miette!(
                    "At least one of --title, --description, or --due-on is required for edit"
                ));
            }

            // 获取当前里程碑信息作为默认值
            let current_milestones = provider
                .list()
                .await
                .map_err(|e| miette::miette!("Failed to list milestones for edit: {e}"))?;
            let existing = current_milestones
                .iter()
                .find(|m| m.number == number)
                .ok_or_else(|| miette::miette!("Milestone #{number} not found"))?;

            let resolved_title = title.clone().unwrap_or_else(|| existing.title.clone());
            let resolved_description = description.clone().or(existing.description.clone());
            let resolved_due_on = if let Some(ref s) = due_on {
                Some(
                    chrono::DateTime::parse_from_rfc3339(s)
                        .map_err(|e| miette::miette!("Invalid RFC 3339 date for --due-on: {e}"))?
                        .with_timezone(&chrono::Utc),
                )
            } else {
                existing.due_on
            };

            let args = CreateMilestoneArgs {
                title: resolved_title,
                description: resolved_description,
                due_on: resolved_due_on,
            };
            let milestone = provider
                .edit(number, args)
                .await
                .map_err(|e| miette::miette!("Failed to edit milestone #{number}: {e}"))?;
            let output = CliOutput::success(milestone, platform, "milestone edit");
            print_output(&output, &output_format)?;
        }
        MilestoneCommand::Close { number } => {
            let milestone = provider
                .close(number)
                .await
                .map_err(|e| miette::miette!("Failed to close milestone #{number}: {e}"))?;
            let output = CliOutput::success(milestone, platform, "milestone close");
            print_output(&output, &output_format)?;
        }
        MilestoneCommand::Reopen { number } => {
            let milestone = provider
                .reopen(number)
                .await
                .map_err(|e| miette::miette!("Failed to reopen milestone #{number}: {e}"))?;
            let output = CliOutput::success(milestone, platform, "milestone reopen");
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

    // --- LabelCommand 解析测试 ---

    #[test]
    fn test_should_parse_label_create() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "gitflow",
            "label",
            "create",
            "bug",
            "--color",
            "d73a4a",
            "--description",
            "Something broken",
        ])
        .expect("parse");
        match cli.command {
            crate::Commands::Label(LabelCommand::Create {
                name,
                color,
                description,
            }) => {
                assert_eq!(name, "bug");
                assert_eq!(color, "d73a4a");
                assert_eq!(description, Some("Something broken".into()));
            }
            _ => panic!("Expected LabelCommand::Create"),
        }
    }

    #[test]
    fn test_should_parse_label_list() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from(["gitflow", "label", "list"]).expect("parse");
        match cli.command {
            crate::Commands::Label(LabelCommand::List) => {}
            _ => panic!("Expected LabelCommand::List"),
        }
    }

    #[test]
    fn test_should_parse_label_edit() {
        use clap::Parser;
        let cli =
            crate::Cli::try_parse_from(["gitflow", "label", "edit", "bug", "--color", "ff0000"])
                .expect("parse");
        match cli.command {
            crate::Commands::Label(LabelCommand::Edit {
                name,
                color,
                description,
            }) => {
                assert_eq!(name, "bug");
                assert_eq!(color, Some("ff0000".into()));
                assert!(description.is_none());
            }
            _ => panic!("Expected LabelCommand::Edit"),
        }
    }

    #[test]
    fn test_should_parse_label_delete() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from(["gitflow", "label", "delete", "wontfix", "--yes"])
            .expect("parse");
        match cli.command {
            crate::Commands::Label(LabelCommand::Delete { name, yes }) => {
                assert_eq!(name, "wontfix");
                assert!(yes);
            }
            _ => panic!("Expected LabelCommand::Delete"),
        }
    }

    // --- MilestoneCommand 解析测试 ---

    #[test]
    fn test_should_parse_milestone_create() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "gitflow",
            "milestone",
            "create",
            "--title",
            "v1.0",
            "--description",
            "First release",
        ])
        .expect("parse");
        match cli.command {
            crate::Commands::Milestone(MilestoneCommand::Create {
                title,
                description,
                due_on,
            }) => {
                assert_eq!(title, "v1.0");
                assert_eq!(description, Some("First release".into()));
                assert!(due_on.is_none());
            }
            _ => panic!("Expected MilestoneCommand::Create"),
        }
    }

    #[test]
    fn test_should_parse_milestone_list() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from(["gitflow", "milestone", "list"]).expect("parse");
        match cli.command {
            crate::Commands::Milestone(MilestoneCommand::List) => {}
            _ => panic!("Expected MilestoneCommand::List"),
        }
    }

    #[test]
    fn test_should_parse_milestone_edit() {
        use clap::Parser;
        let cli =
            crate::Cli::try_parse_from(["gitflow", "milestone", "edit", "5", "--title", "v1.1"])
                .expect("parse");
        match cli.command {
            crate::Commands::Milestone(MilestoneCommand::Edit {
                number,
                title,
                description,
                due_on,
            }) => {
                assert_eq!(number, 5);
                assert_eq!(title, Some("v1.1".into()));
                assert!(description.is_none());
                assert!(due_on.is_none());
            }
            _ => panic!("Expected MilestoneCommand::Edit"),
        }
    }

    #[test]
    fn test_should_parse_milestone_close() {
        use clap::Parser;
        let cli =
            crate::Cli::try_parse_from(["gitflow", "milestone", "close", "3"]).expect("parse");
        match cli.command {
            crate::Commands::Milestone(MilestoneCommand::Close { number }) => {
                assert_eq!(number, 3);
            }
            _ => panic!("Expected MilestoneCommand::Close"),
        }
    }

    #[test]
    fn test_should_parse_milestone_reopen() {
        use clap::Parser;
        let cli =
            crate::Cli::try_parse_from(["gitflow", "milestone", "reopen", "3"]).expect("parse");
        match cli.command {
            crate::Commands::Milestone(MilestoneCommand::Reopen { number }) => {
                assert_eq!(number, 3);
            }
            _ => panic!("Expected MilestoneCommand::Reopen"),
        }
    }

    #[test]
    fn test_should_parse_milestone_create_with_due_on() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "gitflow",
            "milestone",
            "create",
            "--title",
            "v2.0",
            "--due-on",
            "2026-12-01T00:00:00Z",
        ])
        .expect("parse");
        match cli.command {
            crate::Commands::Milestone(MilestoneCommand::Create { title, due_on, .. }) => {
                assert_eq!(title, "v2.0");
                assert_eq!(due_on, Some("2026-12-01T00:00:00Z".into()));
            }
            _ => panic!("Expected MilestoneCommand::Create"),
        }
    }

    // --- 辅助函数测试 ---

    #[test]
    fn test_should_print_json_output() {
        let value = serde_json::json!({"name": "test", "color": "ff0000"});
        let result = print_output(&value, &OutputFormat::Json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_should_reject_text_output_in_phase1() {
        let value = serde_json::json!({"name": "test"});
        let result = print_output(&value, &OutputFormat::Text);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not yet supported"));
    }
}
