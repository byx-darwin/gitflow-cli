//! `gitflow skills` 子命令实现。
//!
//! 管理 gitflow Skills 的安装、列出和卸载。
//! Skills 从仓库的 `skills/` 目录复制到目标目录。
//!
//! 支持多 Agent 平台（Claude Code / Gemini CLI / Codex / Copilot CLI），
//! 支持用户级 / 项目级 / 自定义路径安装。
//!
//! Note: the install/uninstall helpers use `std::fs` for synchronous
//! file operations. This module is invoked before the `tokio` runtime is
//! constructed (see `main()`), so `tokio::fs` is not available here.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "Skills command runs synchronously before the tokio runtime is constructed"
)]

use clap::{ArgAction, Args, Subcommand, ValueEnum};
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Agent platform
// ---------------------------------------------------------------------------

/// 支持的 AI Agent 平台。
///
/// 每种平台有不同的 Skills 安装目录约定（依据 Superpowers 和各平台官方文档）。
/// Codex / Gemini / Copilot 还共享跨平台路径 `~/.agents/skills/`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum AgentPlatform {
    /// Claude Code / Superpowers — `~/.claude/skills/`
    Claude,
    /// Codex (`OpenAI`) — `~/.codex/skills/`（也支持 `~/.agents/skills/`）
    Codex,
    /// `OpenCode` — `~/.opencode/skills/`
    OpenCode,
    /// Gemini CLI — `~/.gemini/skills/`
    Gemini,
    /// GitHub Copilot CLI — `~/.copilot/skills/`
    Copilot,
}

impl AgentPlatform {
    /// 返回该 Agent 的用户级 skills 子目录名（相对于 home）。
    #[must_use]
    pub fn skills_dir_name(self) -> &'static str {
        match self {
            AgentPlatform::Claude => ".claude/skills",
            AgentPlatform::Codex => ".codex/skills",
            AgentPlatform::OpenCode => ".opencode/skills",
            AgentPlatform::Gemini => ".gemini/skills",
            AgentPlatform::Copilot => ".copilot/skills",
        }
    }

    /// 自动检测当前环境中的 Agent 平台。
    ///
    /// 检测策略：按优先级检查各平台的配置目录是否存在。
    /// 默认返回 `Claude`。
    #[must_use]
    pub fn detect() -> Self {
        let Some(home) = dirs::home_dir() else {
            return AgentPlatform::Claude;
        };
        // 按优先级检测
        for platform in &[
            AgentPlatform::Claude,
            AgentPlatform::Codex,
            AgentPlatform::OpenCode,
            AgentPlatform::Gemini,
            AgentPlatform::Copilot,
        ] {
            if home.join(platform.skills_dir_name()).exists() {
                return *platform;
            }
        }
        AgentPlatform::Claude
    }
}

// ---------------------------------------------------------------------------
// Install target
// ---------------------------------------------------------------------------

// （不再需要 InstallTarget enum — 用 bool global flag 表达）

// ---------------------------------------------------------------------------
// CLI args
// ---------------------------------------------------------------------------

/// Skills 管理命令集合。
#[derive(Debug, Subcommand)]
pub enum SkillsCommand {
    /// 安装 skills（默认项目级 `.claude/skills/`，-g 切换全局）
    Install(InstallArgs),
    /// 列出已安装的 skills
    List(ListArgs),
    /// 卸载已安装的 skills
    Uninstall(UninstallArgs),
}

/// `skills install` 参数。
#[derive(Debug, Args)]
pub struct InstallArgs {
    /// 安装到全局用户目录（~/.claude/skills/ 或其他 Agent 目录）
    #[arg(short = 'g', long, action = ArgAction::SetTrue)]
    pub global: bool,

    /// 目标 Agent 平台（仅 -g 时有效，默认自动检测）
    #[arg(long, value_enum, requires = "global")]
    pub agent: Option<AgentPlatform>,

    /// 自定义安装路径（最高优先级）
    #[arg(long = "path")]
    pub custom_path: Option<String>,

    /// 强制覆盖已存在的 skills
    #[arg(short = 'f', long, action = ArgAction::SetTrue)]
    pub force: bool,
}

/// `skills list` 参数。
#[derive(Debug, Args)]
pub struct ListArgs {
    /// 列出全局用户目录下的 skills
    #[arg(short = 'g', long, action = ArgAction::SetTrue)]
    pub global: bool,

    /// 目标 Agent 平台（仅 -g 时有效）
    #[arg(long, value_enum, requires = "global")]
    pub agent: Option<AgentPlatform>,

    /// 自定义查找路径
    #[arg(long = "path")]
    pub custom_path: Option<String>,
}

/// `skills uninstall` 参数。
#[derive(Debug, Args)]
pub struct UninstallArgs {
    /// 从全局用户目录卸载
    #[arg(short = 'g', long, action = ArgAction::SetTrue)]
    pub global: bool,

    /// 目标 Agent 平台（仅 -g 时有效）
    #[arg(long, value_enum, requires = "global")]
    pub agent: Option<AgentPlatform>,

    /// 自定义卸载路径
    #[arg(long = "path")]
    pub custom_path: Option<String>,
}

// ---------------------------------------------------------------------------
// Path resolution
// ---------------------------------------------------------------------------

/// 解析目标目录。
///
/// 优先级：`custom_path` > `-g` 全局 > 项目级（默认）
fn resolve_target_dir(
    global: bool,
    agent: Option<AgentPlatform>,
    custom_path: Option<&str>,
) -> miette::Result<PathBuf> {
    // 自定义路径优先
    if let Some(p) = custom_path {
        return Ok(PathBuf::from(p));
    }

    if global {
        let platform = agent.unwrap_or_else(AgentPlatform::detect);
        let home =
            dirs::home_dir().ok_or_else(|| miette::miette!("无法确定 HOME 目录"))?;
        Ok(home.join(platform.skills_dir_name()))
    } else {
        // 默认：项目级
        let output = std::process::Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .map_err(|e| miette::miette!("无法执行 git rev-parse: {e}"))?;
        if !output.status.success() {
            return Err(miette::miette!(
                "当前目录不在 Git 仓库中，项目级安装需要 Git 仓库。使用 -g 安装到全局目录。"
            ));
        }
        let repo_root = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(PathBuf::from(repo_root).join(".claude").join("skills"))
    }
}

/// Skills 源目录（仓库内的 skills/）。
fn skills_source_dir() -> PathBuf {
    PathBuf::from("skills")
}

// ---------------------------------------------------------------------------
// Command handlers
// ---------------------------------------------------------------------------

/// 处理 `gitflow skills` 命令。
pub fn handle(command: &SkillsCommand) -> miette::Result<()> {
    match command {
        SkillsCommand::Install(args) => install_skills(args),
        SkillsCommand::List(args) => list_skills(args),
        SkillsCommand::Uninstall(args) => uninstall_skills(args),
    }
}

/// 安装 skills。
fn install_skills(args: &InstallArgs) -> miette::Result<()> {
    let target = resolve_target_dir(args.global, args.agent, args.custom_path.as_deref())?;
    let source = skills_source_dir();

    if !source.exists() {
        return Err(miette::miette!(
            "Skills 源目录不存在: {}",
            source.display()
        ));
    }

    std::fs::create_dir_all(&target)
        .map_err(|e| miette::miette!("无法创建目标目录 {}: {e}", target.display()))?;

    let level = if args.global { "全局" } else { "项目级" };
    println!("目标: {} ({level})", target.display());

    let mut installed = 0u32;
    let mut skipped = 0u32;
    let mut overwritten = 0u32;

    for entry in std::fs::read_dir(&source)
        .map_err(|e| miette::miette!("无法读取 skills 源目录 {}: {e}", source.display()))?
    {
        let entry = entry.map_err(|e| miette::miette!("读取目录项失败: {e}"))?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if !name_str.starts_with("gitflow-") {
            continue;
        }

        let dest = target.join(&name);
        if dest.exists() {
            if args.force {
                std::fs::remove_dir_all(&dest).map_err(|e| {
                    miette::miette!("无法删除旧版本 {}: {e}", dest.display())
                })?;
                copy_dir_all(&entry.path(), &dest)
                    .map_err(|e| miette::miette!("复制 {} 失败: {e}", name_str))?;
                println!("♻ 已覆盖: {name_str}");
                overwritten += 1;
            } else {
                eprintln!("⚠ 跳过已存在: {name_str}");
                skipped += 1;
            }
            continue;
        }

        copy_dir_all(&entry.path(), &dest)
            .map_err(|e| miette::miette!("复制 {} 失败: {e}", name_str))?;
        println!("✅ 已安装: {name_str}");
        installed += 1;
    }

    println!();
    println!(
        "安装完成: 新增 {installed} 个，覆盖 {overwritten} 个，跳过 {skipped} 个"
    );
    Ok(())
}

/// 列出已安装的 skills。
fn list_skills(args: &ListArgs) -> miette::Result<()> {
    let target = resolve_target_dir(args.global, args.agent, args.custom_path.as_deref())?;

    if !target.exists() {
        println!("(未安装任何 skills)");
        println!("目录: {}", target.display());
        return Ok(());
    }

    println!("目录: {}", target.display());
    println!();

    let mut found = 0u32;
    for entry in std::fs::read_dir(&target)
        .map_err(|e| miette::miette!("读取目录失败 {}: {e}", target.display()))?
    {
        let entry = entry.map_err(|e| miette::miette!("读取目录项失败: {e}"))?;
        let name_str = entry.file_name().to_string_lossy().into_owned();
        if name_str.starts_with("gitflow-") {
            println!("  {name_str}");
            found += 1;
        }
    }

    if found == 0 {
        println!("(未安装任何 gitflow skills)");
    } else {
        println!();
        println!("共 {found} 个 skills");
    }
    Ok(())
}

/// 卸载 skills。
fn uninstall_skills(args: &UninstallArgs) -> miette::Result<()> {
    let target = resolve_target_dir(args.global, args.agent, args.custom_path.as_deref())?;

    if !target.exists() {
        println!("(未安装任何 skills)");
        println!("目录: {}", target.display());
        return Ok(());
    }

    println!("目录: {}", target.display());
    println!();

    let mut removed = 0u32;
    for entry in std::fs::read_dir(&target)
        .map_err(|e| miette::miette!("读取目录失败 {}: {e}", target.display()))?
    {
        let entry = entry.map_err(|e| miette::miette!("读取目录项失败: {e}"))?;
        let name_str = entry.file_name().to_string_lossy().into_owned();
        if name_str.starts_with("gitflow-") {
            let path = entry.path();
            if path.is_dir() {
                std::fs::remove_dir_all(&path)
                    .map_err(|e| miette::miette!("删除 {} 失败: {e}", path.display()))?;
            } else {
                std::fs::remove_file(&path)
                    .map_err(|e| miette::miette!("删除 {} 失败: {e}", path.display()))?;
            }
            println!("✅ 已卸载: {name_str}");
            removed += 1;
        }
    }

    if removed == 0 {
        println!("(未安装任何 gitflow skills)");
    } else {
        println!();
        println!("已卸载 {removed} 个 skills");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// 递归复制目录。
fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            std::fs::copy(entry.path(), &dest_path)?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    reason = "允许在测试中使用 expect/unwrap/panic"
)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_platform_claude_dir() {
        assert_eq!(
            AgentPlatform::Claude.skills_dir_name(),
            ".claude/skills"
        );
    }

    #[test]
    fn test_agent_platform_codex_dir() {
        assert_eq!(
            AgentPlatform::Codex.skills_dir_name(),
            ".codex/skills"
        );
    }

    #[test]
    fn test_agent_platform_opencode_dir() {
        assert_eq!(
            AgentPlatform::OpenCode.skills_dir_name(),
            ".opencode/skills"
        );
    }

    #[test]
    fn test_agent_detect_returns_some() {
        let platform = AgentPlatform::detect();
        assert!(matches!(
            platform,
            AgentPlatform::Claude
                | AgentPlatform::Codex
                | AgentPlatform::OpenCode
                | AgentPlatform::Gemini
                | AgentPlatform::Copilot
        ));
    }

    #[test]
    fn test_resolve_global_target() {
        let dir = resolve_target_dir(true, Some(AgentPlatform::Claude), None).expect("resolve");
        assert!(dir.ends_with(".claude/skills"));
    }

    #[test]
    fn test_resolve_custom_path_overrides_all() {
        let dir = resolve_target_dir(false, Some(AgentPlatform::Claude), Some("/tmp/my-skills"))
            .expect("resolve");
        assert_eq!(dir, PathBuf::from("/tmp/my-skills"));
    }

    #[test]
    fn test_skills_source_dir_is_valid_path() {
        let dir = skills_source_dir();
        assert!(dir.ends_with("skills"));
    }
}
