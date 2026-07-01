//! PR 子命令。
//!
//! 提供 `gitflow pr <action>` 的子命令枚举。
//! 具体实现在 Task 11 中完成。

use clap::Subcommand;

/// PR 子命令。
#[derive(Debug, Subcommand)]
pub enum PrCommand {
    /// 创建 PR（将在 Task 11 实现完整参数）
    Create,
    /// 列出 PR（将在 Task 11 实现完整参数）
    List,
    /// 查看 PR（将在 Task 11 实现完整参数）
    View,
}
