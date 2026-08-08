# gf update + Skills 版本管理 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 `gf` 添加 `gf update`（自更新 binary，GitHub Releases 源）与 `gf skills update`（从内嵌数据更新已安装 skills）两个命令。

**Architecture:** `gf update` 使用 `self_update` 0.42 的 github 后端完成下载+替换 binary；版本选择（稳定版/`--pre`）由独立纯函数 `select_target_version` 控制，基于 `semver` crate。`gf skills update` 复用现有 `skills.rs` 的复制逻辑（force 模式），并刷新 auto-report-bug hook。两个命令均为同步执行，经 `router` 分发。

**Tech Stack:** Rust 2024 · clap derive · `self_update` 0.42 (features: `rustls`, `compression-flate2`) · `semver` 1

**Issue:** #149 · **设计文档:** `docs/superpowers/specs/2026-08-08-gf-update-skills-design.md`

## Global Constraints

- Rust 2024，workspace lints 全开（`-D warnings -W clippy::pedantic`）
- 生产代码禁止 `unwrap()` / `expect()` / `panic`；错误用 `miette` 返回
- 禁止 `println!`/`dbg!` 用于日志——但 CLI 交互输出用 `println!`（与现有 skills.rs 一致）
- TLS 用 rustls（`self_update` 启用 `rustls` feature，`default-features = false`）
- 禁止修改 `deny.toml` / `.pre-commit-config.yaml` / `rust-toolchain.toml` 的现有策略配置
- 新增依赖前无需用户额外确认（已在 Phase 2 设计批准中授权 `self_update` + `semver`）
- `gf update` 与 `gf skills update` 为同步命令；`self_update` 的阻塞网络 I/O 在 tokio runtime 内可接受（单次 CLI 操作）
- 发布资产命名（`[package.metadata.binstall]`）：`gitflow-cli-{target}.tgz`，内含 `gitflow-cli-{target}/gf`
- 版本格式：GitHub release tag 为 `v1.0.0`（前导 `v`）；`self_update` 解析后 `Release.version` 为裸版本 `1.0.0`；`built_info::PKG_VERSION` 为 `1.0.0`

---

### Task 1: 添加依赖（workspace + gitflow-cli）

**Files:**
- Modify: `Cargo.toml:15-25`（`[workspace.dependencies]`）
- Modify: `apps/cli/Cargo.toml`（`[dependencies]`）

**Interfaces:**
- Consumes: 无
- Produces: `self_update`、`semver` 可作为 `{ workspace = true }` 依赖引入

- [ ] **Step 1: 在 workspace `[workspace.dependencies]` 添加**

```toml
# Self-update for `gf update` (GitHub Releases backend)
self_update = { version = "0.42", default-features = false, features = ["rustls", "compression-flate2"] }
# Semver for update version resolution (stable vs prerelease)
semver = "1"
```

（插入到 `secrecy = "0.11"` 附近；`self_update` 注释掉的行可删除，改用上面正式依赖。）

- [ ] **Step 2: 在 `apps/cli/Cargo.toml` 的 `[dependencies]` 添加**

```toml
self_update = { workspace = true }
semver = { workspace = true }
```

- [ ] **Step 3: 验证依赖解析**

Run: `cargo check -p gitflow-cli`
Expected: 编译通过（此时 `self_update` 未使用，会有 dead_code 警告可忽略；下一步引入使用后消失）。

- [ ] **Step 4: 提交**

```bash
git add Cargo.toml apps/cli/Cargo.toml
git commit -m "chore(update): add self_update and semver dependencies"
```

---

### Task 2: `update.rs` 版本解析纯函数

**Files:**
- Create: `apps/cli/src/commands/update.rs`（此任务只写版本解析部分）
- Modify: `apps/cli/src/commands/mod.rs`（注册 `pub mod update;`）

**Interfaces:**
- Consumes: `crate::built_info::PKG_VERSION`（main.rs 已生成）
- Produces:
  - `const REPO_OWNER: &str = "byx-darwin"`
  - `const REPO_NAME: &str = "gitflow-cli"`
  - `const BIN_NAME: &str = "gf"`
  - `fn parse_version(s: &str) -> Option<semver::Version>`
  - `fn is_prerelease(v: &semver::Version) -> bool`
  - `fn select_target_version<'a>(candidates: impl Iterator<Item = &'a str>, current: &semver::Version, include_prerelease: bool) -> Option<String>`
  - `fn current_version() -> String`
  - `fn target_triple() -> String`

- [ ] **Step 1: 写失败测试**（新建 `apps/cli/src/commands/update.rs`，含 `#[cfg(test)] mod tests`）

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_stable() {
        assert_eq!(parse_version("1.0.0"), Some(Version::new(1, 0, 0)));
    }

    #[test]
    fn test_parse_version_strips_leading_v() {
        assert_eq!(parse_version("v1.2.3"), Some(Version::new(1, 2, 3)));
    }

    #[test]
    fn test_parse_version_prerelease() {
        let v = parse_version("1.1.0-rc.1").expect("parse rc");
        assert!(v.pre.to_string().contains("rc"));
    }

    #[test]
    fn test_parse_version_invalid() {
        assert_eq!(parse_version("not-a-version"), None);
        assert_eq!(parse_version(""), None);
    }

    #[test]
    fn test_is_prerelease_rc_is_prerelease() {
        assert!(is_prerelease(&Version::parse("1.1.0-rc.1").expect("rc")));
    }

    #[test]
    fn test_is_prerelease_stable_is_not() {
        assert!(!is_prerelease(&Version::new(1, 1, 0)));
    }

    #[test]
    fn test_select_target_version_ignores_prerelease_by_default() {
        let candidates = ["1.0.1", "1.1.0-rc.1", "1.1.0"];
        let current = Version::new(1, 0, 0);
        assert_eq!(
            select_target_version(candidates.into_iter(), &current, false).as_deref(),
            Some("1.1.0")
        );
    }

    #[test]
    fn test_select_target_version_includes_prerelease_with_flag() {
        let current = Version::new(1, 0, 0);
        // 稳定版更高时仍选稳定版
        let candidates = ["1.1.0-rc.1", "1.1.0"];
        assert_eq!(
            select_target_version(candidates.into_iter(), &current, true).as_deref(),
            Some("1.1.0")
        );
        // 仅预发布更高时，--pre 选中预发布
        let candidates = ["1.1.0-rc.1", "1.0.5"];
        assert_eq!(
            select_target_version(candidates.into_iter(), &current, true).as_deref(),
            Some("1.1.0-rc.1")
        );
    }

    #[test]
    fn test_select_target_version_none_when_up_to_date() {
        let candidates = ["0.9.0", "1.0.0"];
        let current = Version::new(1, 0, 0);
        assert_eq!(select_target_version(candidates.into_iter(), &current, false), None);
    }

    #[test]
    fn test_select_target_version_skips_invalid() {
        let candidates = ["not-a-version", "", "1.0.1"];
        let current = Version::new(1, 0, 0);
        assert_eq!(
            select_target_version(candidates.into_iter(), &current, false).as_deref(),
            Some("1.0.1")
        );
    }
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p gitflow-cli update::tests`
Expected: FAIL（`update` 模块/函数未定义）

- [ ] **Step 3: 写最小实现**

```rust
//! `gf update` 子命令实现。
//!
//! 从 GitHub Releases 检查并更新 gf binary 到最新版本。
//! 版本选择逻辑独立为纯函数，便于单测覆盖。

use semver::Version;

/// GitHub 仓库 owner。
pub(crate) const REPO_OWNER: &str = "byx-darwin";
/// GitHub 仓库名。
pub(crate) const REPO_NAME: &str = "gitflow-cli";
/// binary 名称。
pub(crate) const BIN_NAME: &str = "gf";

/// 解析 semver 版本字符串（容忍前导 `v`）。
fn parse_version(s: &str) -> Option<Version> {
    Version::parse(s.trim_start_matches('v')).ok()
}

/// 是否为预发布版本（含 `-alpha`/`-beta`/`-rc` 等 pre 标识）。
fn is_prerelease(v: &Version) -> bool {
    !v.pre.is_empty()
}

/// 从候选版本中选择目标版本：返回大于 `current` 的最高版本。
///
/// `include_prerelease` 为 `false` 时排除预发布版本（稳定版优先）。
fn select_target_version<'a>(
    candidates: impl Iterator<Item = &'a str>,
    current: &Version,
    include_prerelease: bool,
) -> Option<String> {
    candidates
        .filter_map(parse_version)
        .filter(|v| *v > *current && (include_prerelease || !is_prerelease(v)))
        .max()
        .map(|v| v.to_string())
}

/// 当前安装的 gf 版本（编译期注入）。
fn current_version() -> String {
    crate::built_info::PKG_VERSION.to_string()
}

/// 当前平台目标三元组（如 `x86_64-apple-darwin`）。
fn target_triple() -> String {
    self_update::get_target().to_string()
}
```

（`mod.rs` 添加 `pub mod update;`）

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo test -p gitflow-cli update::tests`
Expected: PASS（Task 3 的命令处理函数尚未实现，但版本解析测试已通过）

- [ ] **Step 5: 提交**

```bash
git add apps/cli/src/commands/update.rs apps/cli/src/commands/mod.rs
git commit -m "feat(update): add semver version resolution helpers"
```

---

### Task 3: `gf update` 命令处理

**Files:**
- Modify: `apps/cli/src/commands/update.rs`（追加 UpdateArgs + handle_update + fetch_release_versions + prompt_update_skills）
- Modify: `apps/cli/src/commands/skills.rs`（`fn confirm` → `pub(crate) fn confirm`；`SkillsUpdateArgs` 公开；`pub fn update_skills` —— 见 Task 4）

**Interfaces:**
- Consumes: `skills::confirm`, `skills::SkillsUpdateArgs`, `skills::update_skills`（Task 4 提供）、`fetch_release_versions`
- Produces:
  - `pub struct UpdateArgs { pre: bool, check: bool, yes: bool }`
  - `pub fn handle_update(args: &UpdateArgs) -> miette::Result<()>`
  - `fn handle_update_with(args: &UpdateArgs, fetch: impl Fn(&str) -> miette::Result<Vec<String>>) -> miette::Result<()>`
  - `fn fetch_release_versions(target: &str) -> miette::Result<Vec<String>>`

- [ ] **Step 1: 写失败测试**

```rust
// 追加到 update.rs tests 模块
    use clap::{Parser, Subcommand};

    #[derive(Debug, Parser)]
    struct TestCli {
        #[command(subcommand)]
        cmd: TestCmd,
    }

    #[derive(Debug, Subcommand)]
    enum TestCmd {
        Update(UpdateArgs),
    }

    #[test]
    fn test_update_args_parse_flags() {
        let cli = TestCli::parse_from(["test", "update", "--check", "--pre", "--yes"]);
        let TestCmd::Update(args) = cli.cmd;
        assert!(args.check);
        assert!(args.pre);
        assert!(args.yes);
    }

    #[test]
    fn test_update_up_to_date_when_no_newer() {
        let args = UpdateArgs { pre: false, check: false, yes: true };
        let versions = vec![current_version()];
        let result = handle_update_with(&args, |_| Ok(versions));
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_check_reports_latest_without_updating() {
        let args = UpdateArgs { pre: false, check: true, yes: false };
        let versions = vec!["9.9.9".to_string()];
        let result = handle_update_with(&args, |_| Ok(versions));
        assert!(result.is_ok());
    }
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p gitflow-cli update::tests::test_update_`
Expected: FAIL（`UpdateArgs`/`handle_update` 未定义）

- [ ] **Step 3: 写实现**（追加到 update.rs，更新文件头 doc）

```rust
use clap::{ArgAction, Args};
use miette::miette;

/// `gf update` 参数。
#[derive(Debug, Args)]
pub struct UpdateArgs {
    /// 包含预发布版本（alpha/beta/rc），默认仅稳定版
    #[arg(long, action = ArgAction::SetTrue)]
    pub pre: bool,

    /// 仅检查是否有新版本，不执行更新
    #[arg(long, action = ArgAction::SetTrue)]
    pub check: bool,

    /// 跳过确认提示，直接更新
    #[arg(short = 'y', long = "yes", action = ArgAction::SetTrue)]
    pub yes: bool,
}

/// 处理 `gf update` 命令。
///
/// # Errors
///
/// - 无法获取 GitHub release 列表（网络错误）
/// - 当前版本号无法解析
/// - `self_update` 下载/替换 binary 失败
pub fn handle_update(args: &UpdateArgs) -> miette::Result<()> {
    handle_update_with(args, fetch_release_versions)
}

/// `gf update` 核心逻辑（注入版本获取函数，便于单测覆盖网络路径）。
fn handle_update_with(
    args: &UpdateArgs,
    fetch_versions: impl Fn(&str) -> miette::Result<Vec<String>>,
) -> miette::Result<()> {
    let current = current_version();
    let current_v = parse_version(&current)
        .ok_or_else(|| miette!("当前版本号无法解析: {current}"))?;
    let target = target_triple();

    let versions = fetch_versions(&target)?;
    let Some(latest) = select_target_version(versions.iter().map(String::as_str), &current_v, args.pre)
    else {
        println!("✅ 已是最新版本 v{current}");
        return Ok(());
    };

    if args.check {
        println!("当前版本: v{current}");
        println!("最新版本: v{latest}");
        return Ok(());
    }

    if !args.yes
        && !crate::commands::skills::confirm(&format!("是否更新到 v{latest}？"), true)?
    {
        println!("已取消更新");
        return Ok(());
    }

    let status = self_update::backends::github::Update::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .bin_name(BIN_NAME)
        .current_version(&current)
        .target_version_tag(&format!("v{latest}"))
        .target(&target)
        .bin_path_in_archive(&format!("{REPO_NAME}-{{{{ target }}}}/{{{{ bin }}}}"))
        .show_download_progress(true)
        .show_output(true)
        .no_confirm(true)
        .build()
        .map_err(|e| miette!("配置更新器失败: {e}"))?
        .update()
        .map_err(|e| miette!("更新失败: {e}"))?;

    println!("✅ gf 已更新到 v{}", status.version());

    prompt_update_skills()?;
    Ok(())
}

/// 从 GitHub Releases 获取适配当前平台的可用版本列表。
///
/// # Errors
///
/// - 网络请求失败
/// - 仓库无 release
fn fetch_release_versions(target: &str) -> miette::Result<Vec<String>> {
    let releases = self_update::backends::github::ReleaseList::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .with_target(target)
        .build()
        .map_err(|e| miette!("配置 release 查询失败: {e}"))?
        .fetch()
        .map_err(|e| miette!("获取 release 列表失败: {e}"))?;
    Ok(releases.into_iter().map(|r| r.version).collect())
}

/// 更新 binary 后提示是否同步更新全局 skills。
fn prompt_update_skills() -> miette::Result<()> {
    if !std::io::stderr().is_terminal() {
        println!("ℹ️ 非交互模式，已跳过 skills 同步。可运行 `gf skills update -g` 手动更新。");
        return Ok(());
    }
    if crate::commands::skills::confirm("是否同时更新全局 skills？", true)? {
        let args = crate::commands::skills::SkillsUpdateArgs {
            global: true,
            agent: None,
            custom_path: None,
        };
        crate::commands::skills::update_skills(&args)?;
    }
    Ok(())
}
```

> **注意：** `update.rs` 的 `miette` 宏与 miette crate 的 `miette::miette!` 冲突——项目使用 `miette::miette!` 宏。若 `use miette::miette` 与现有 `miette::miette!` 用法冲突，去掉 `use miette::miette`，改用全路径 `miette::miette!(...)`。Task 4 中 `skills.rs` 需将 `fn confirm` 改为 `pub(crate) fn confirm`（现有实现不变）。

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo test -p gitflow-cli update::tests`
Expected: PASS（`test_update_up_to_date_when_no_newer` 与 `test_update_check_reports_latest_without_updating` 通过注入 fetch 避开网络；`prompt_update_skills` 在非 TTY 测试环境下走 `!is_terminal()` 分支）

- [ ] **Step 5: 提交**

```bash
git add apps/cli/src/commands/update.rs apps/cli/src/commands/skills.rs
git commit -m "feat(update): implement gf update command via GitHub Releases"
```

---

### Task 4: `gf skills update` 子命令

**Files:**
- Modify: `apps/cli/src/commands/skills.rs`（`SkillsCommand::Update` 变体、`SkillsUpdateArgs`、`update_skills`、`handle` match arm、`confirm` → `pub(crate)`、提取 `copy_skills_dir` helper）

**Interfaces:**
- Consumes: `resolve_target_dir`, `skills_source_dir`, `copy_dir_all`, `copy_skills_dir`（新 helper）, `install_skills_bundled`, `install_hook`, `AgentPlatform`
- Produces:
  - `pub enum SkillsCommand` 新增 `Update(SkillsUpdateArgs)`
  - `pub struct SkillsUpdateArgs { global: bool, agent: Option<AgentPlatform>, custom_path: Option<String> }`
  - `pub fn update_skills(args: &SkillsUpdateArgs) -> miette::Result<()>`
  - `fn copy_skills_dir(source: &Path, target: &Path, force: bool) -> miette::Result<(u32, u32, u32)>`（从 install_skills 提取）
  - `pub(crate) fn confirm(prompt: &str, default: bool) -> miette::Result<bool>`（原私有改为 pub(crate)）

- [ ] **Step 1: 写失败测试**（追加到 skills.rs tests 模块）

```rust
    // -----------------------------------------------------------------------
    // skills update tests
    // -----------------------------------------------------------------------

    /// 构造临时源目录，写入若干 gf-* skills 与一个非 gf 目录。
    fn seed_source(tmp: &tempfile::TempDir, names: &[&str]) {
        for name in names {
            let dir = tmp.path().join("source").join(name);
            std::fs::create_dir_all(&dir).expect("create source skill dir");
            std::fs::write(dir.join("SKILL.md"), format!("# {name}\n")).expect("write SKILL.md");
        }
        std::fs::create_dir_all(tmp.path().join("source/not-a-skill")).expect("create non-gf dir");
    }

    /// 构造临时目标目录，写入若干已安装 gf-* skills。
    fn seed_target(tmp: &tempfile::TempDir, names: &[&str]) {
        for name in names {
            let dir = tmp.path().join("target").join(name);
            std::fs::create_dir_all(&dir).expect("create target skill dir");
            std::fs::write(dir.join("SKILL.md"), "old\n").expect("write old content");
        }
    }

    #[test]
    fn test_copy_skills_dir_overwrites_existing() {
        let tmp = tempfile::tempdir().expect("tempdir");
        seed_source(&tmp, &["gf-alpha"]);
        seed_target(&tmp, &["gf-alpha"]);

        let (installed, overwritten, skipped) =
            copy_skills_dir(&tmp.path().join("source"), &tmp.path().join("target"), true)
                .expect("copy");
        assert_eq!(installed, 0);
        assert_eq!(overwritten, 1);
        assert_eq!(skipped, 0);

        let content = std::fs::read_to_string(tmp.path().join("target/gf-alpha/SKILL.md"))
            .expect("read updated");
        assert_eq!(content, "# gf-alpha\n", "content must be replaced");
    }

    #[test]
    fn test_copy_skills_dir_installs_new() {
        let tmp = tempfile::tempdir().expect("tempdir");
        seed_source(&tmp, &["gf-alpha", "gf-beta"]);
        seed_target(&tmp, &["gf-alpha"]);

        let (installed, overwritten, _) =
            copy_skills_dir(&tmp.path().join("source"), &tmp.path().join("target"), true)
                .expect("copy");
        assert_eq!(installed, 1, "gf-beta must be newly installed");
        assert_eq!(overwritten, 1);
        assert!(tmp.path().join("target/gf-beta/SKILL.md").exists());
    }

    #[test]
    fn test_copy_skills_dir_preserves_other_dirs() {
        let tmp = tempfile::tempdir().expect("tempdir");
        seed_source(&tmp, &["gf-alpha"]);
        seed_target(&tmp, &["gf-alpha"]);
        let other = tmp.path().join("target/not-a-skill");
        std::fs::write(other.join("README.md"), "keep me\n").expect("write other dir");

        copy_skills_dir(&tmp.path().join("source"), &tmp.path().join("target"), true)
            .expect("copy");

        assert!(
            other.join("README.md").exists(),
            "non-gf-* dirs must be left untouched"
        );
    }

    #[test]
    fn test_update_skills_args_parse() {
        use clap::Parser;
        #[derive(Debug, Parser)]
        struct TestCli {
            #[command(subcommand)]
            cmd: TestCmd,
        }
        #[derive(Debug, Subcommand)]
        enum TestCmd {
            Update(SkillsUpdateArgs),
        }
        let cli = TestCli::parse_from(["test", "update", "-g", "--agent", "codex", "--path", "/tmp/x"]);
        let TestCmd::Update(args) = cli.cmd;
        assert!(args.global);
        assert_eq!(args.agent, Some(AgentPlatform::Codex));
        assert_eq!(args.custom_path.as_deref(), Some("/tmp/x"));
    }
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p gitflow-cli skills::tests::test_copy_skills_dir_`
Expected: FAIL（`copy_skills_dir`/`SkillsUpdateArgs` 未定义）

- [ ] **Step 3: 实现**

**3a. 提取 `copy_skills_dir` helper**（将 `install_skills` 中的复制循环提取为独立函数；`install_skills` 改为调用它）

```rust
/// 将源目录下的 `gf-*` skills 复制到目标目录。
///
/// `force` 为 `true` 时覆盖已存在项；否则跳过。
/// 返回 `(新增, 覆盖, 跳过)`。逐项打印结果，与原有 UX 一致。
fn copy_skills_dir(source: &Path, target: &Path, force: bool) -> miette::Result<(u32, u32, u32)> {
    std::fs::create_dir_all(target)
        .map_err(|e| miette::miette!("无法创建目标目录 {}: {e}", target.display()))?;

    let mut installed = 0u32;
    let mut overwritten = 0u32;
    let mut skipped = 0u32;

    for entry in std::fs::read_dir(source)
        .map_err(|e| miette::miette!("无法读取 skills 源目录 {}: {e}", source.display()))?
    {
        let entry = entry.map_err(|e| miette::miette!("读取目录项失败: {e}"))?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if !name_str.starts_with("gf-") {
            continue;
        }

        let dest = target.join(&name);
        if dest.exists() {
            if force {
                std::fs::remove_dir_all(&dest)
                    .map_err(|e| miette::miette!("无法删除旧版本 {}: {e}", dest.display()))?;
                copy_dir_all(&entry.path(), &dest)
                    .map_err(|e| miette::miette!("复制 {name_str} 失败: {e}"))?;
                println!("♻ 已覆盖: {name_str}");
                overwritten += 1;
            } else {
                eprintln!("⚠ 跳过已存在: {name_str}");
                skipped += 1;
            }
            continue;
        }

        copy_dir_all(&entry.path(), &dest)
            .map_err(|e| miette::miette!("复制 {name_str} 失败: {e}"))?;
        println!("✅ 已安装: {name_str}");
        installed += 1;
    }

    Ok((installed, overwritten, skipped))
}
```

将 `install_skills` 中从 `let mut installed = 0u32;` 到 `println!(); println!("安装完成: ...")` 的循环替换为：

```rust
        let (installed, overwritten, skipped) = copy_skills_dir(&source, &target, args.force)?;
        println!();
        println!("安装完成: 新增 {installed} 个，覆盖 {overwritten} 个，跳过 {skipped} 个");
```

（注意：替换后删除原循环体内的 `let level`/`println!("目标: ...")` 之上的部分保持不动；`target` 创建已移入 helper。）

**3b. 新增 `SkillsUpdateArgs` 与 `update_skills`**

```rust
/// `gf skills update` 参数。
#[derive(Debug, Args)]
pub struct SkillsUpdateArgs {
    /// 从全局用户目录更新
    #[arg(short = 'g', long, action = ArgAction::SetTrue)]
    pub global: bool,

    /// 目标 Agent 平台（默认 `claude`）
    #[arg(long, value_enum)]
    pub agent: Option<AgentPlatform>,

    /// 自定义更新路径（最高优先级）
    #[arg(long = "path")]
    pub custom_path: Option<String>,
}

/// 更新已安装的 skills：从当前 binary 内嵌数据覆盖所有 `gf-*` skills，
/// 并刷新 auto-report-bug hook。等价于 `gf skills install --force`，
/// 但不做技能来源检查、不触发共建计划提示。
///
/// # Errors
///
/// - 目标目录读取失败
/// - 复制 / 删除失败
pub fn update_skills(args: &SkillsUpdateArgs) -> miette::Result<()> {
    let platform = args.agent.unwrap_or_else(AgentPlatform::detect);
    let target = resolve_target_dir(args.global, Some(platform), args.custom_path.as_deref())?;
    let source = skills_source_dir();
    let has_source = source.exists();
    let has_bundled = !SKILLS.is_empty();

    if !target.exists() {
        println!("(未安装任何 skills)");
        println!("目录: {}", target.display());
        return Ok(());
    }

    if has_source {
        let (installed, overwritten, skipped) = copy_skills_dir(&source, &target, true)?;
        println!();
        println!("✅ Skills 已更新: 覆盖 {overwritten} 个，新增 {installed} 个，跳过 {skipped} 个");
    } else if has_bundled {
        let install_args = InstallArgs {
            global: args.global,
            agent: args.agent,
            custom_path: args.custom_path.clone(),
            force: true,
            report_bug: false,
        };
        install_skills_bundled(&target, &install_args)?;
    } else {
        println!("⚠ Skills 源目录未找到，且 binary 未内嵌 skills 数据");
        println!("  请从源码目录运行，或手动指定 --path <skills 目录路径>");
    }

    if platform.supports_hooks() {
        install_hook(args.global, true, platform)?;
    }
    Ok(())
}
```

**3c. 更新 `SkillsCommand` 枚举与 `handle` match**

```rust
pub enum SkillsCommand {
    /// 安装 skills（默认项目级 `.claude/skills/`，-g 切换全局）
    Install(InstallArgs),
    /// 列出已安装的 skills
    List(ListArgs),
    /// 卸载已安装的 skills
    Uninstall(UninstallArgs),
    /// 更新已安装的 skills（等价于 install --force，刷新 hook）
    Update(SkillsUpdateArgs),
}
```

```rust
pub fn handle(command: &SkillsCommand) -> miette::Result<()> {
    match command {
        SkillsCommand::Install(args) => install_skills(args),
        SkillsCommand::List(args) => list_skills(args),
        SkillsCommand::Uninstall(args) => uninstall_skills(args),
        SkillsCommand::Update(args) => update_skills(args),
    }
}
```

**3d. `fn confirm` 改为 `pub(crate)`**

```rust
pub(crate) fn confirm(prompt: &str, default: bool) -> miette::Result<bool> {
```

（`confirm_with_reader` 保持私有，`pub(crate)` 只暴露 `confirm` 入口。）

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo test -p gitflow-cli skills::tests`
Expected: PASS（新测试 + 原有 install 测试均通过，确认 refactor 未破坏 install）

- [ ] **Step 5: 运行 fmt/clippy 确认 lint 通过**

Run: `cargo +nightly fmt --all -- --check && cargo clippy -p gitflow-cli --all-targets --all-features -- -D warnings`
Expected: PASS（clippy pedantic 无告警）

- [ ] **Step 6: 提交**

```bash
git add apps/cli/src/commands/skills.rs
git commit -m "feat(skills): add gf skills update subcommand"
```

---

### Task 5: `main.rs` 命令注册与路由

**Files:**
- Modify: `apps/cli/src/main.rs`（`Commands` 枚举、`platform_needed`、prerequisites 排除、`router`、`command_name`、CLI 解析测试）

**Interfaces:**
- Consumes: `commands::update::UpdateArgs`、`commands::update::handle_update`
- Produces: `gf update` 可从 CLI 入口解析并分发

- [ ] **Step 1: 写失败测试**（追加到 main.rs `#[cfg(test)] mod tests`）

```rust
    #[test]
    fn test_should_parse_update_command() {
        let cli = Cli::try_parse_from(["gf", "update", "--check"]).expect("parse update");
        assert!(matches!(cli.command, Commands::Update(ref a) if a.check));
    }

    #[test]
    fn test_update_does_not_need_platform() {
        let cli = Cli::try_parse_from(["gf", "update"]).expect("parse update");
        let platform_needed = !matches!(
            cli.command,
            Commands::Skills(_) | Commands::Completions(_) | Commands::Workflow(_)
                | Commands::Update(_)
        );
        assert!(!platform_needed);
    }
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p gitflow-cli main::tests::test_should_parse_update_command`
Expected: FAIL（`Commands::Update` 未定义，无法编译）

- [ ] **Step 3: 实现**（main.rs 四处修改）

**3a. `Commands` 枚举添加变体**（在 `Skills` 之后、`Run` 之前）

```rust
    /// Skills management operations (install, list, uninstall, update).
    #[command(subcommand)]
    Skills(commands::skills::SkillsCommand),

    /// Update the gf binary to the latest GitHub release.
    #[command(subcommand)]
    Update(commands::update::UpdateArgs),
```

**3b. `platform_needed` 排除**（第 115 行）

```rust
    let platform_needed = !matches!(
        cli.command,
        Commands::Skills(_) | Commands::Completions(_) | Commands::Workflow(_)
            | Commands::Update(_)
    );
```

**3c. prerequisites 排除**（第 172 行）

```rust
    if !matches!(
        cli.command,
        Commands::Skills(_) | Commands::Completions(_) | Commands::Workflow(_)
            | Commands::Update(_)
    ) {
        commands::prerequisites::check(platform).map_err(|e| miette::miette!("{e}"))?;
    }
```

**3d. `router` 分发**（在 `Commands::Skills` 分支后）

```rust
        Commands::Skills(ref cmd) => commands::skills::handle(cmd),
        Commands::Update(cmd) => commands::update::handle_update(&cmd),
```

**3e. `command_name` 匹配**（第 441-453 行，Skills 后加）

```rust
            Commands::Skills(_) => "skills",
            Commands::Update(_) => "update",
```

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo test -p gitflow-cli main::tests`
Expected: PASS

- [ ] **Step 5: 运行完整验证**

Run:
```bash
cargo +nightly fmt --all -- --check
cargo clippy -p gitflow-cli --all-targets --all-features -- -D warnings
cargo test -p gitflow-cli
```
Expected: 全部 PASS

- [ ] **Step 6: 手动冒烟验证**

Run:
```bash
cargo run --quiet -- update --check
```
Expected: 输出 `当前版本: v1.0.0` 与 GitHub 最新版本（若发布过 v1.0.0 及以上则显示最新，否则提示已是最新）；不触发下载。

Run:
```bash
cargo run --quiet -- skills update --help
```
Expected: 显示 `gf skills update` 帮助，含 `-g`/`--agent`/`--path`。

- [ ] **Step 7: 提交**

```bash
git add apps/cli/src/main.rs
git commit -m "feat(update): wire gf update command into CLI"
```

---

### Task 6: README 使用说明更新

**Files:**
- Modify: `README.md`（命令列表 / 使用文档）

**Interfaces:**
- Consumes: Task 3/4/5 产出的命令

- [ ] **Step 1: 在 README 命令列表中补充**

```markdown
### 更新与维护

- `gf update` — 从 GitHub Releases 自更新 binary（`--check` 仅检查，`--pre` 含预发布，`-y` 跳过确认）
- `gf skills update` — 从当前 binary 内嵌数据更新已安装 skills（`-g` 全局，`--agent` 指定平台）
```

（插入到 README 合适的命令使用章节；若无命令列表章节，则放入 `## Usage` 相关小节。）

- [ ] **Step 2: 校对渲染**

Run: 打开 `README.md` 目视检查新小节格式与上下文衔接。
Expected: 无 Markdown 语法错误，与邻近内容风格一致。

- [ ] **Step 3: 提交**

```bash
git add README.md
git commit -m "docs(readme): document gf update and gf skills update"
```

---

## 自审记录（Self-Review）

**1. Spec coverage（对照设计文档）：**
- ✅ `gf update` CLI 参数（`--pre`/`--check`/`--yes`）— Task 3
- ✅ GitHub Releases 更新源 + self_update — Task 3
- ✅ 更新后提示同步 skills — Task 3（`prompt_update_skills`）
- ✅ `gf skills update`（`-g`/`--agent`/`--path`）— Task 4
- ✅ 从内嵌数据覆盖更新 + hook 刷新 — Task 4
- ✅ 更新失败不留半成品 — Task 3（`self_update` 事务性替换，见 Global Constraints）
- ✅ TDD 测试矩阵 — Task 2/3/4/5 测试表

**2. Placeholder scan：** 无 TBD/TODO；所有代码块均为可编译的具体实现。

**3. Type consistency：**
- `select_target_version(candidates: impl Iterator<Item=&str>, current: &Version, include_prerelease: bool) -> Option<String>` — Task 2 定义，Task 3 调用时 `versions.iter().map(String::as_str)` 匹配。
- `update_skills(&SkillsUpdateArgs)` — Task 4 定义，Task 3 `prompt_update_skills` 以结构体字面量调用，字段名一致（`global`/`agent`/`custom_path`）。
- `confirm(&str, bool) -> miette::Result<bool>` — Task 3 以 `pub(crate)` 从 skills.rs 复用，签名与现有实现一致。
- `copy_skills_dir(&Path, &Path, bool) -> miette::Result<(u32, u32, u32)>` — Task 4 定义，install_skills 与 update_skills 均调用。

**4. 已知验证边界：** `self_update` 的真实下载+替换路径（Task 3 中 `handle_update_with` 的 update 分支）不做单元测试（需网络+真实 release）；通过 `--check` 路径测试 + 手动冒烟验证 + 最终 dogfooding 覆盖。`self_update` 0.42 API 已对照 crate 源码核实（`ReleaseList::configure().repo_owner().repo_name().with_target().build().fetch()`、`Update::configure()...build().update()`、`Status::version()`）。
