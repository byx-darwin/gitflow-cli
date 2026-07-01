//! Issue 子命令。
//!
//! 提供 `gitflow issue <action>` 的子命令枚举。
//! 具体实现在 Task 10 中完成。

use clap::Subcommand;

/// Issue 子命令。
#[derive(Debug, Subcommand)]
pub enum IssueCommand {
    /// 创建 Issue（将在 Task 10 实现完整参数）
    Create,
    /// 列出 Issue（将在 Task 10 实现完整参数）
    List,
    /// 查看 Issue（将在 Task 10 实现完整参数）
    View,
}
