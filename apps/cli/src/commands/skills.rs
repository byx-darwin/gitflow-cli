//! `gitflow skills` 子命令实现。
//!
//! 管理 gitflow Skills 的安装、列出和卸载。
//! Skills 从仓库的 `skills/` 目录复制到 `~/.claude/skills/`。
//!
//! Note: the install/uninstall helpers use `std::fs` for synchronous
//! file operations. This module is invoked before the `tokio` runtime is
//! constructed (see `main()`), so `tokio::fs` is not available here.

#![allow(
    clippy::disallowed_methods,
    reason = "Skills command runs synchronously before the tokio runtime is constructed"
)]

use clap::Subcommand;
use std::path::PathBuf;

/// Skills 管理命令集合。
#[derive(Debug, Subcommand)]
pub enum SkillsCommand {
    /// 安装 skills 到 ~/.claude/skills/
    Install,
    /// 列出已安装的 skills
    List,
    /// 卸载已安装的 skills
    Uninstall,
}

/// Skills 安装目标目录。
fn skills_target_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude").join("skills"))
}

/// Skills 源目录（仓库内的 skills/）。
fn skills_source_dir() -> PathBuf {
    PathBuf::from("skills")
}

/// 处理 `gitflow skills` 命令。
///
/// # Errors
///
/// 返回错误当：
/// - 无法确定用户 home 目录。
/// - 文件操作失败。
pub fn handle(command: &SkillsCommand) -> miette::Result<()> {
    match command {
        SkillsCommand::Install => install_skills(),
        SkillsCommand::List => list_skills(),
        SkillsCommand::Uninstall => uninstall_skills(),
    }
}

/// 安装 skills：将仓库 `skills/` 目录下的 gitflow-* 文件复制到 `~/.claude/skills/`。
fn install_skills() -> miette::Result<()> {
    let target = skills_target_dir()
        .ok_or_else(|| miette::miette!("无法确定 HOME 目录，无法安装 skills"))?;
    let source = skills_source_dir();

    if !source.exists() {
        return Err(miette::miette!(
            "Skills 源目录不存在: {}",
            source.display()
        ));
    }

    std::fs::create_dir_all(&target)
        .map_err(|e| miette::miette!("无法创建目标目录 {}: {e}", target.display()))?;

    let mut installed = 0u32;
    let mut skipped = 0u32;

    for entry in std::fs::read_dir(&source)
        .map_err(|e| miette::miette!("无法读取 skills 源目录 {}: {e}", source.display()))?
    {
        let entry = entry.map_err(|e| miette::miette!("读取目录项失败: {e}"))?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // 仅处理 gitflow- 开头的目录
        if !name_str.starts_with("gitflow-") {
            continue;
        }

        let dest = target.join(&name);
        if dest.exists() {
            eprintln!("⚠ 跳过已存在: {name_str}");
            skipped += 1;
            continue;
        }

        copy_dir_all(&entry.path(), &dest)
            .map_err(|e| miette::miette!("复制 {} 失败: {e}", name_str))?;
        println!("✅ 已安装: {name_str}");
        installed += 1;
    }

    println!();
    println!("安装完成: 新增 {installed} 个，跳过 {skipped} 个");
    Ok(())
}

/// 列出 `~/.claude/skills/` 下的 gitflow-* skills。
fn list_skills() -> miette::Result<()> {
    let Some(target) = skills_target_dir() else {
        return Err(miette::miette!("无法确定 HOME 目录"));
    };

    if !target.exists() {
        println!("(未安装任何 skills)");
        return Ok(());
    }

    let mut found = 0u32;
    for entry in
        std::fs::read_dir(&target).map_err(|e| miette::miette!("读取目录失败 {}: {e}", target.display()))?
    {
        let entry = entry.map_err(|e| miette::miette!("读取目录项失败: {e}"))?;
        let name_str = entry.file_name().to_string_lossy().into_owned();
        if name_str.starts_with("gitflow-") {
            println!("  {name_str}");
            found += 1;
        }
    }

    if found == 0 {
        println!("(未安装任何 skills)");
    } else {
        println!();
        println!("共 {found} 个 skills");
    }
    Ok(())
}

/// 卸载 `~/.claude/skills/` 下的 gitflow-* skills。
fn uninstall_skills() -> miette::Result<()> {
    let Some(target) = skills_target_dir() else {
        return Err(miette::miette!("无法确定 HOME 目录"));
    };

    if !target.exists() {
        println!("(未安装任何 skills)");
        return Ok(());
    }

    let mut removed = 0u32;
    for entry in
        std::fs::read_dir(&target).map_err(|e| miette::miette!("读取目录失败 {}: {e}", target.display()))?
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
        println!("(未安装任何 skills)");
    } else {
        println!();
        println!("已卸载 {removed} 个 skills");
    }
    Ok(())
}

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
    fn test_skills_target_dir_returns_some() {
        assert!(skills_target_dir().is_some());
    }

    #[test]
    fn test_skills_source_dir_is_valid_path() {
        let dir = skills_source_dir();
        // 路径应以 "skills" 结尾
        assert!(dir.ends_with("skills"));
    }
}
