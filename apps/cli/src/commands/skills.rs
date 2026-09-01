//! `gf skills` 子命令实现。
//!
//! 管理 gitflow Skills 的安装、列出和卸载。
//! Skills 可以从仓库的 `skills/` 目录复制，也可以从编译时嵌入的
//! 数据中提取（release 场景，binary 发布包不带 skills/ 目录）。
//!
//! 支持多 Agent 平台（Claude Code / Gemini CLI / Codex / Copilot CLI），
//! 支持用户级 / 项目级 / 自定义路径安装。
//!
//! Note: the install/uninstall helpers use `std::fs` for synchronous
//! file operations. This module is invoked before the `tokio` runtime is
//! constructed (see `main()`), so `tokio::fs` is not available here.
//! This is intentional — these operations are short-lived I/O that do
//! not benefit from async.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "Skills command runs synchronously before the tokio runtime is constructed"
)]

use std::{
    fmt,
    path::{Path, PathBuf},
};

// 编译时由 build.rs 生成的 skills 清单（release binary 内嵌）
include!(concat!(env!("OUT_DIR"), "/skills_manifest.rs"));

use clap::{ArgAction, Args, Subcommand, ValueEnum};

// ---------------------------------------------------------------------------
// Agent platform
// ---------------------------------------------------------------------------

/// 支持的 AI Agent 平台。
///
/// 每种平台有不同的 Skills 安装目录约定（依据 Superpowers 和各平台官方文档）。
/// 路径统一使用 `~/<.agent>/skills/` 形式，不与 `~/.agents/skills/` 混用。
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum AgentPlatform {
    /// Claude Code / Superpowers — `~/.claude/skills/`
    Claude,
    /// Codex (`OpenAI`) — `~/.codex/skills/`
    Codex,
    /// `OpenCode` — `~/.opencode/skills/`
    OpenCode,
    /// Gemini CLI — `~/.gemini/skills/`
    Gemini,
    /// GitHub Copilot CLI — `~/.copilot/skills/`
    Copilot,
    /// Qoder — `~/.qoder-cn/skills/`
    Qoder,
    /// Pi Code Agent — `~/.pi/agent/skills/`
    Pi,
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
            AgentPlatform::Qoder => ".qoder-cn/skills",
            AgentPlatform::Pi => ".pi/agent/skills",
        }
    }

    /// 返回该 Agent 的全局（用户级）skills 子目录名。
    ///
    /// 默认与 `skills_dir_name()` 相同；仅当 Agent 的全局路径与项目级路径不同
    /// 时才覆写（如 `OpenCode` 遵循 `XDG` 规范，全局配置在 `~/.config/opencode/`）。
    #[must_use]
    pub fn global_skills_dir_name(self) -> &'static str {
        match self {
            AgentPlatform::OpenCode => ".config/opencode/skills",
            other => other.skills_dir_name(),
        }
    }

    /// 返回默认 Agent 平台。
    ///
    /// 默认固定为 `Claude`；其他平台需通过 `--agent` 参数显式指定。
    /// 不扫描 `$HOME` 下各平台目录的存在性，避免隐式探测导致目标漂移。
    #[must_use]
    pub fn detect() -> Self {
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
    /// 更新已安装的 skills（等价于 install --force，刷新 hook）
    Update(SkillsUpdateArgs),
}

/// `skills install` 参数。
#[derive(Debug, Args)]
pub struct InstallArgs {
    /// 安装到全局用户目录（~/.claude/skills/ 或其他 Agent 目录）
    #[arg(short = 'g', long, action = ArgAction::SetTrue)]
    pub global: bool,

    /// 目标 Agent 平台（默认 `claude`）
    #[arg(long, value_enum)]
    pub agent: Option<AgentPlatform>,

    /// 自定义安装路径（最高优先级）
    #[arg(long = "path")]
    pub custom_path: Option<String>,

    /// 强制覆盖已存在的 skills
    #[arg(short = 'f', long, action = ArgAction::SetTrue)]
    pub force: bool,

    /// 外部技能集来源（superpowers 或 mattpocock）
    #[arg(long, value_enum)]
    pub source: Option<SkillSource>,
}

/// 外部技能集来源。
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum SkillSource {
    /// Superpowers skill set (by Anthropic)
    Superpowers,
    /// Matt Pocock's skills collection
    Mattpocock,
}

/// `skills list` 参数。
#[derive(Debug, Args)]
pub struct ListArgs {
    /// 列出全局用户目录下的 skills
    #[arg(short = 'g', long, action = ArgAction::SetTrue)]
    pub global: bool,

    /// 目标 Agent 平台（默认 `claude`）
    #[arg(long, value_enum)]
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

    /// 目标 Agent 平台（默认 `claude`）
    #[arg(long, value_enum)]
    pub agent: Option<AgentPlatform>,

    /// 自定义卸载路径
    #[arg(long = "path")]
    pub custom_path: Option<String>,
}

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

// ---------------------------------------------------------------------------
// Path resolution
// ---------------------------------------------------------------------------

/// 解析目标目录。
///
/// 优先级：`custom_path` > `-g` 全局（按 agent）> 项目级（按 agent，默认 `Claude`）
fn resolve_target_dir(
    global: bool,
    agent: Option<AgentPlatform>,
    custom_path: Option<&str>,
) -> miette::Result<PathBuf> {
    // 自定义路径优先（用户显式指定的目标目录，允许绝对路径，但仍拒绝
    // `..`/NUL 字节/危险字符等注入形态的输入）
    if let Some(p) = custom_path {
        let safe = gitflow_core::SafePath::new_allow_absolute(p)
            .map_err(|e| miette::miette!("无效的 --path 参数: {e}"))?;
        return Ok(safe.as_path().to_path_buf());
    }

    let platform = agent.unwrap_or_else(AgentPlatform::detect);

    if global {
        let home = dirs::home_dir().ok_or_else(|| miette::miette!("无法确定 HOME 目录"))?;
        Ok(home.join(platform.global_skills_dir_name()))
    } else {
        let repo_root = git_repo_root()?;
        Ok(resolve_project_target(&repo_root, platform))
    }
}

/// 解析项目级 skills 安装目录（尊重 agent 参数）。
///
/// 独立函数便于单测覆盖，避免在 `resolve_target_dir` 内部隐式调用
/// `git rev-parse`。参数 `repo_root` 已由调用方通过 `git_repo_root()` 解析，
/// 此函数仅做路径拼接（无失败分支），因此直接返回 `PathBuf`。
///
/// 注意：`agent` 接受 `AgentPlatform` 而非 `Option<AgentPlatform>`，
/// 因为调用方在调用前已确定目标平台（避免 `detect()` 重复触发）。
fn resolve_project_target(repo_root: &std::path::Path, agent: AgentPlatform) -> PathBuf {
    // `skills_dir_name` 返回 `.claude/skills` 这类相对路径，直接拼到 repo 根
    repo_root.join(agent.skills_dir_name())
}

/// Skills 源目录（仓库内的 skills/）。
fn skills_source_dir() -> PathBuf {
    // 1. 优先：binary 所在目录的上级目录（release 安装场景：binary 在 ./，skills 在 ./skills/）
    if let Ok(exe) = std::env::current_exe()
        && let Some(exe_dir) = exe.parent()
    {
        let candidate = exe_dir.join("skills");
        if candidate.exists() {
            return candidate;
        }
        // binary 在子目录（如 bin/）的场景
        if let Some(parent) = exe_dir.parent() {
            let candidate = parent.join("skills");
            if candidate.exists() {
                return candidate;
            }
        }
    }

    // 2. 回退：当前工作目录（开发场景：cargo run 在项目根目录）
    PathBuf::from("skills")
}

// ---------------------------------------------------------------------------
// Skill source detection (Issue #141)
// ---------------------------------------------------------------------------

/// 技能来源类型。
///
/// 与运行时检测（`skills/gf-workflow/references.md` 哨兵表）共享同一份权威定义；
/// 修改哨兵规则时两处必须同步（测试 `test_sentinel_rules_match_references` 守护）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillSourceKind {
    /// Superpowers（plugin `superpowers` 或裸名安装）。
    Superpowers,
    /// mattpocock/skills（plugin `mattpocock-skills` 或裸名安装）。
    Mattpocock,
}

impl fmt::Display for SkillSourceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Superpowers => write!(f, "superpowers"),
            Self::Mattpocock => write!(f, "mattpocock"),
        }
    }
}

/// plugin 形态的注册表键前缀（`installed_plugins.json` 键形如 `<plugin>@<marketplace>`）。
const SUPERPOWERS_PLUGIN_PREFIX: &str = "superpowers@";
/// mattpocock plugin 形态的注册表键前缀。
const MATTPOCOCK_PLUGIN_PREFIX: &str = "mattpocock-skills@";

/// 裸名形态哨兵（双哨兵同时命中才判定；裸名脆弱从严）。
const SUPERPOWERS_BARE_SENTINELS: &[&str] = &["brainstorming", "writing-plans"];
/// mattpocock 裸名形态哨兵（双哨兵同时命中才判定）。
const MATTPOCOCK_BARE_SENTINELS: &[&str] = &["to-spec", "grilling"];

/// plugin 形态探测：解析 `~/.claude/plugins/installed_plugins.json` 键前缀。
///
/// 注册表缺失或损坏返回 `false`（降级到裸名探测，不 panic）。
fn plugin_source_present(home: &Path, prefix: &str) -> bool {
    let path = home.join(".claude/plugins/installed_plugins.json");
    let Ok(content) = std::fs::read_to_string(&path) else {
        return false;
    };
    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) else {
        return false;
    };
    parsed
        .get("plugins")
        .and_then(serde_json::Value::as_object)
        .is_some_and(|plugins| plugins.keys().any(|key| key.starts_with(prefix)))
}

/// 裸名形态探测：`~/.claude/skills/` 下哨兵目录双命中。
fn bare_sentinels_present(home: &Path, sentinels: &[&str]) -> bool {
    sentinels
        .iter()
        .all(|name| home.join(".claude/skills").join(name).is_dir())
}

/// 检测已安装的技能来源（安装时 Step 0，Issue #141）。
///
/// 依次探测 plugin 形态与裸名形态，返回所有在场的来源（0/1/2 个）。
/// 两者共存时全部返回，由调用方决定提示方式。
#[must_use]
pub fn detect_skill_sources(home: &Path) -> Vec<SkillSourceKind> {
    let mut found = Vec::new();
    if plugin_source_present(home, SUPERPOWERS_PLUGIN_PREFIX)
        || bare_sentinels_present(home, SUPERPOWERS_BARE_SENTINELS)
    {
        found.push(SkillSourceKind::Superpowers);
    }
    if plugin_source_present(home, MATTPOCOCK_PLUGIN_PREFIX)
        || bare_sentinels_present(home, MATTPOCOCK_BARE_SENTINELS)
    {
        found.push(SkillSourceKind::Mattpocock);
    }
    found
}

/// 安装时技能来源前置检查（`install_skills` Step 0，Issue #141）。
///
/// 仅 Claude 平台执行；其他平台提示跳过。两来源皆无时返回错误（非 0 退出码）
/// 并打印三条安装引导——硬阻断，保证「装了 gf-workflow 就必然能跑」。
///
/// # Errors
///
/// Claude 平台且两来源皆未安装，或无法确定 HOME 目录时返回错误。
fn check_skill_source(platform: AgentPlatform) -> miette::Result<()> {
    if !matches!(platform, AgentPlatform::Claude) {
        println!("ℹ 非 Claude 平台，跳过技能来源检测");
        return Ok(());
    }
    let home = dirs::home_dir().ok_or_else(|| miette::miette!("无法确定 HOME 目录"))?;
    check_skill_source_at(&home)
}

/// 未检测到技能来源时打印的安装引导。
///
/// 硬阻断前展示三条安装路径，并附上 Node.js 版本前置条件——
/// `mattpocock-skills` / `npx skills` 均要求 Node.js ≥ 22.20.0，
/// 早期文档缺此说明导致用户装技能来源时踩坑（Issue #192）。
const SKILL_SOURCE_GUIDANCE: &str = "\
⛔ 未检测到任何技能来源，gf-workflow 无法运行。请先安装其一：
  · claude plugins install superpowers
  · claude plugins install mattpocock-skills
  · npx skills@latest add mattpocock/skills
提示：安装 mattpocock-skills / npx skills 需要 Node.js ≥ 22.20.0，先运行 `node --version` 确认。";

/// 核心检测逻辑（参数化 HOME，便于单测注入临时目录）。
fn check_skill_source_at(home: &Path) -> miette::Result<()> {
    let sources = detect_skill_sources(home);
    if sources.is_empty() {
        eprintln!("{SKILL_SOURCE_GUIDANCE}");
        return Err(miette::miette!("技能来源缺失，安装中止"));
    }
    let names: Vec<String> = sources.iter().map(ToString::to_string).collect();
    println!("✓ 检测到技能来源: {}", names.join(" + "));
    Ok(())
}

// ---------------------------------------------------------------------------
// Command handlers
// ---------------------------------------------------------------------------

/// 处理 `gf skills` 命令。
pub fn handle(command: &SkillsCommand) -> miette::Result<()> {
    match command {
        SkillsCommand::Install(args) => install_skills(args),
        SkillsCommand::List(args) => list_skills(args),
        SkillsCommand::Uninstall(args) => uninstall_skills(args),
        SkillsCommand::Update(args) => update_skills(args),
    }
}

/// 安装 skills。
fn install_skills(args: &InstallArgs) -> miette::Result<()> {
    // 一次性解析目标平台；避免 `AgentPlatform::detect()` 被重复调用。
    let platform = args.agent.unwrap_or_else(AgentPlatform::detect);

    // 处理外部技能集安装（--source superpowers|mattpocock）
    if let Some(source) = &args.source {
        return install_external_skills(source, args, platform);
    }

    // Step 0（Issue #141）：技能来源前置检查；两来源皆无硬阻断。
    check_skill_source(platform)?;

    let target = resolve_target_dir(args.global, Some(platform), args.custom_path.as_deref())?;
    let source = skills_source_dir();

    let has_source = source.exists();
    let has_bundled = !SKILLS.is_empty();

    if has_source {
        // 从文件系统目录安装（开发场景或 cargo install --path）
        let level = if args.global { "全局" } else { "项目级" };
        println!("目标: {} ({level})", target.display());

        let result = copy_skills_dir(&source, &target, args.force)?;
        println!();
        println!(
            "安装完成: 新增 {} 个，覆盖 {} 个，跳过 {} 个",
            result.installed, result.overwritten, result.skipped
        );
        if !result.failures.is_empty() {
            println!(
                "⚠ 失败 {} 个: {}",
                result.failures.len(),
                result.failures.join(", ")
            );
            return Err(miette::miette!(
                "{} 个 skill 安装失败: {}",
                result.failures.len(),
                result.failures.join(", ")
            ));
        }
    } else if has_bundled {
        install_skills_bundled(&target, args)?;
    } else {
        println!("⚠ Skills 源目录未找到，且 binary 未内嵌 skills 数据");
        println!("  请从源码目录运行，或手动指定 --source <skills 目录路径>");
    }

    Ok(())
}

/// `copy_skills_dir` 的汇总结果。
#[derive(Debug)]
struct CopySkillsResult {
    /// 新增安装的 skill 数量。
    installed: u32,
    /// 覆盖已存在的 skill 数量。
    overwritten: u32,
    /// 跳过的 skill 数量（`force=false` 且已存在）。
    skipped: u32,
    /// 失败列表 — 单项失败不中止整体。
    failures: Vec<String>,
}

/// 将源目录下的 `gf-*` skills 复制到目标目录。
///
/// `force` 为 `true` 时覆盖已存在项；否则跳过。
/// 单个 skill 复制失败时跳过并记录到 `failures`，不中止整体流程。
/// 返回汇总结果，包含计数和失败列表。
fn copy_skills_dir(source: &Path, target: &Path, force: bool) -> miette::Result<CopySkillsResult> {
    std::fs::create_dir_all(target)
        .map_err(|e| miette::miette!("无法创建目标目录 {}: {e}", target.display()))?;

    let mut result = CopySkillsResult {
        installed: 0,
        overwritten: 0,
        skipped: 0,
        failures: Vec::new(),
    };

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
                if let Err(e) = std::fs::remove_dir_all(&dest) {
                    result.failures.push(name_str.to_string());
                    eprintln!("⚠ 失败: {name_str} — 无法删除旧版本: {e}");
                    continue;
                }
                if let Err(e) = copy_dir_all(&entry.path(), &dest) {
                    result.failures.push(name_str.to_string());
                    eprintln!("⚠ 失败: {name_str} — 复制失败: {e}");
                    continue;
                }
                println!("♻ 已覆盖: {name_str}");
                result.overwritten += 1;
            } else {
                eprintln!("⚠ 跳过已存在: {name_str}");
                result.skipped += 1;
            }
            continue;
        }

        if let Err(e) = copy_dir_all(&entry.path(), &dest) {
            result.failures.push(name_str.to_string());
            eprintln!("⚠ 失败: {name_str} — 复制失败: {e}");
            continue;
        }
        println!("✅ 已安装: {name_str}");
        result.installed += 1;
    }

    Ok(result)
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
        let result = copy_skills_dir(&source, &target, true)?;
        println!();
        println!(
            "✅ Skills 已更新: 覆盖 {} 个，新增 {} 个，跳过 {} 个",
            result.overwritten, result.installed, result.skipped
        );
        if !result.failures.is_empty() {
            println!(
                "⚠ 失败 {} 个: {}",
                result.failures.len(),
                result.failures.join(", ")
            );
            return Err(miette::miette!(
                "{} 个 skill 更新失败: {}",
                result.failures.len(),
                result.failures.join(", ")
            ));
        }
    } else if has_bundled {
        let install_args = InstallArgs {
            global: args.global,
            agent: args.agent,
            custom_path: args.custom_path.clone(),
            force: true,
            source: None,
        };
        install_skills_bundled(&target, &install_args)?;
    } else {
        println!("⚠ Skills 源目录未找到，且 binary 未内嵌 skills 数据");
        println!("  请从源码目录运行，或手动指定 --path <skills 目录路径>");
    }

    Ok(())
}

/// 从编译时嵌入的 SKILLS 数据安装 skills。
fn install_skills_bundled(target: &std::path::Path, args: &InstallArgs) -> miette::Result<()> {
    std::fs::create_dir_all(target).map_err(|e| miette::miette!("无法创建目标目录: {e}"))?;

    println!(
        "目标: {} ({})",
        target.display(),
        if args.global { "全局" } else { "项目级" }
    );
    println!("使用内嵌 skills 数据（{} 个文件）", SKILLS.len());

    // 按 skill 目录分组
    let mut skill_dirs: std::collections::HashMap<&str, Vec<(&str, &[u8])>> =
        std::collections::HashMap::new();
    for (path, data) in SKILLS {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 2
            && parts.first().is_some_and(|f| f.starts_with("gf-"))
            && let Some(&first) = parts.first()
        {
            let relative = &path[first.len() + 1..];
            skill_dirs.entry(first).or_default().push((relative, *data));
        }
    }

    let mut installed = 0u32;
    let mut skipped = 0u32;
    let mut overwritten = 0u32;

    for (skill_name, files) in &skill_dirs {
        let dest = target.join(skill_name);
        install_single_skill_bundled(
            &dest,
            files,
            args,
            &mut installed,
            &mut skipped,
            &mut overwritten,
        )?;
    }

    println!();
    println!("安装完成: 新增 {installed} 个，覆盖 {overwritten} 个，跳过 {skipped} 个");
    Ok(())
}

fn install_single_skill_bundled(
    dest: &std::path::Path,
    files: &[(&str, &[u8])],
    args: &InstallArgs,
    installed: &mut u32,
    skipped: &mut u32,
    overwritten: &mut u32,
) -> miette::Result<()> {
    let is_overwrite = dest.exists();

    if is_overwrite {
        if args.force {
            std::fs::remove_dir_all(dest).map_err(|e| miette::miette!("无法删除: {e}"))?;
        } else {
            eprintln!("⚠ 跳过已存在: {}", dest.display());
            *skipped += 1;
            return Ok(());
        }
    }

    for (rel_path, data) in files {
        let file_path = dest.join(rel_path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| miette::miette!("创建目录失败: {e}"))?;
        }
        std::fs::write(&file_path, data).map_err(|e| miette::miette!("写入失败: {e}"))?;
    }

    if is_overwrite && args.force {
        println!("♻ 已覆盖: {}", dest.display());
        *overwritten += 1;
    } else {
        println!("✅ 已安装: {}", dest.display());
        *installed += 1;
    }
    Ok(())
}

/// 从外部 GitHub 仓库安装技能集（superpowers 或 mattpocock）。
///
/// # Errors
///
/// Returns an error if git clone fails, or if the skills directory cannot be found.
#[allow(
    clippy::disallowed_methods,
    reason = "Sync process invocation for git clone during skill installation"
)]
fn install_external_skills(
    source: &SkillSource,
    args: &InstallArgs,
    platform: AgentPlatform,
) -> miette::Result<()> {
    let target = resolve_target_dir(args.global, Some(platform), args.custom_path.as_deref())?;

    // Determine repository URL and skills subdirectory
    let (repo_url, skills_subdir) = match source {
        SkillSource::Superpowers => ("https://github.com/anthropics/superpowers.git", "skills"),
        SkillSource::Mattpocock => ("https://github.com/mattpocock/skills.git", "skills"),
    };

    println!("📦 从 {repo_url} 克隆技能集...");

    // Create temporary directory for cloning
    let temp_dir = tempfile::tempdir().map_err(|e| miette::miette!("创建临时目录失败: {e}"))?;
    let clone_path = temp_dir.path().join("repo");

    // Clone repository
    let output = std::process::Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "--filter=blob:none",
            repo_url,
            &clone_path.to_string_lossy(),
        ])
        .output()
        .map_err(|e| miette::miette!("git clone 失败: {e}\n请确保已安装 git"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(miette::miette!("git clone 失败:\n{}", stderr.trim()));
    }

    println!("✅ 克隆完成，正在提取 skills...");

    // Find skills directory
    let skills_source = clone_path.join(skills_subdir);
    if !skills_source.exists() {
        return Err(miette::miette!(
            "未找到 skills 目录: {}\n仓库结构可能已变更",
            skills_source.display()
        ));
    }

    // Copy skills to target
    let level = if args.global { "全局" } else { "项目级" };
    println!("目标: {} ({})", target.display(), level);

    let result = copy_skills_dir(&skills_source, &target, args.force)?;

    // Clean up temporary directory
    drop(temp_dir);

    println!();
    println!(
        "安装完成: 新增 {} 个，覆盖 {} 个，跳过 {} 个",
        result.installed, result.overwritten, result.skipped
    );
    if !result.failures.is_empty() {
        println!(
            "⚠ 失败 {} 个: {}",
            result.failures.len(),
            result.failures.join(", ")
        );
        return Err(miette::miette!(
            "{} 个 skill 安装失败: {}",
            result.failures.len(),
            result.failures.join(", ")
        ));
    }

    Ok(())
}

/// 获取当前仓库根目录（不在 git 仓库中则回退到当前目录）。
fn git_repo_root() -> miette::Result<std::path::PathBuf> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output();
    match output {
        Ok(out) if out.status.success() => Ok(std::path::PathBuf::from(
            String::from_utf8_lossy(&out.stdout).trim(),
        )),
        _ => std::env::current_dir().map_err(|e| miette::miette!("无法获取当前目录: {e}")),
    }
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
        if name_str.starts_with("gf-") {
            println!("  {name_str}");
            found += 1;
        }
    }

    if found == 0 {
        println!("(未安装任何 gf skills)");
    } else {
        println!();
        println!("共 {found} 个 skills");
    }
    Ok(())
}

/// 卸载 skills。
fn uninstall_skills(args: &UninstallArgs) -> miette::Result<()> {
    // 一次性解析目标平台；与 `install_skills` 对称，避免重复 detect()。
    let platform = args.agent.unwrap_or_else(AgentPlatform::detect);

    let target = resolve_target_dir(args.global, Some(platform), args.custom_path.as_deref())?;

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
        if name_str.starts_with("gf-") {
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
        println!("(未安装任何 gf skills)");
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
// Interactive prompts
// ---------------------------------------------------------------------------

/// Read a Y/n confirmation from stdin.
///
/// Displays `prompt` and reads one line. Accepts `y/yes` (case-insensitive) as true,
/// `n/no` as false, and empty input as `default`. On EOF or invalid input after 3
/// retries, returns `default`.
///
/// # Errors
///
/// Returns an error if reading from stdin fails.
pub(crate) fn confirm(prompt: &str, default: bool) -> miette::Result<bool> {
    let stdin = std::io::stdin();
    let mut reader = stdin.lock();
    confirm_with_reader(prompt, default, &mut reader)
}

/// Testable core of [`confirm`] — reads from any `BufRead` source.
#[allow(
    dead_code,
    reason = "called by `confirm`; direct calls from tests for isolation"
)]
fn confirm_with_reader(
    prompt: &str,
    default: bool,
    reader: &mut impl std::io::BufRead,
) -> miette::Result<bool> {
    use std::io::Write;
    let hint = if default { "[Y/n]" } else { "[y/N]" };
    print!("{prompt} {hint} ");
    let _ = std::io::stdout().flush();

    for _ in 0..3 {
        let mut line = String::new();
        let bytes_read = reader
            .read_line(&mut line)
            .map_err(|e| miette::miette!("读取输入失败: {e}"))?;

        if bytes_read == 0 {
            // EOF
            return Ok(default);
        }

        match line.trim().to_lowercase().as_str() {
            "" => return Ok(default),
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => {
                print!("请输入 y 或 n: ");
                let _ = std::io::stdout().flush();
            }
        }
    }

    Ok(default)
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
        assert_eq!(AgentPlatform::Claude.skills_dir_name(), ".claude/skills");
    }

    #[test]
    fn test_agent_platform_codex_dir() {
        assert_eq!(AgentPlatform::Codex.skills_dir_name(), ".codex/skills");
    }

    #[test]
    fn test_agent_platform_opencode_dir() {
        assert_eq!(
            AgentPlatform::OpenCode.skills_dir_name(),
            ".opencode/skills"
        );
    }

    #[test]
    fn test_agent_platform_qoder_dir() {
        assert_eq!(AgentPlatform::Qoder.skills_dir_name(), ".qoder-cn/skills");
    }

    #[test]
    fn test_agent_platform_pi_dir() {
        assert_eq!(AgentPlatform::Pi.skills_dir_name(), ".pi/agent/skills");
    }

    #[test]
    fn test_agent_platform_gemini_dir() {
        assert_eq!(AgentPlatform::Gemini.skills_dir_name(), ".gemini/skills");
    }

    #[test]
    fn test_agent_platform_copilot_dir() {
        assert_eq!(AgentPlatform::Copilot.skills_dir_name(), ".copilot/skills");
    }

    #[test]
    fn test_agent_detect_always_returns_claude() {
        // 契约：detect() 默认固定返回 Claude，不扫描 $HOME 下其他平台目录。
        // 其他平台必须通过 `--agent` 显式指定。
        assert_eq!(AgentPlatform::detect(), AgentPlatform::Claude);
    }

    #[test]
    fn test_resolve_global_target_claude() {
        let dir = resolve_target_dir(true, Some(AgentPlatform::Claude), None).expect("resolve");
        assert!(dir.ends_with(".claude/skills"));
    }

    #[test]
    fn test_resolve_global_target_codex() {
        let dir = resolve_target_dir(true, Some(AgentPlatform::Codex), None).expect("resolve");
        assert!(dir.ends_with(".codex/skills"));
    }

    #[test]
    fn test_resolve_global_target_gemini() {
        let dir = resolve_target_dir(true, Some(AgentPlatform::Gemini), None).expect("resolve");
        assert!(dir.ends_with(".gemini/skills"));
    }

    #[test]
    fn test_resolve_global_target_qoder() {
        let dir = resolve_target_dir(true, Some(AgentPlatform::Qoder), None).expect("resolve");
        assert!(dir.ends_with(".qoder-cn/skills"));
    }

    #[test]
    fn test_resolve_global_target_opencode_xdg() {
        let dir = resolve_target_dir(true, Some(AgentPlatform::OpenCode), None).expect("resolve");
        assert!(
            dir.ends_with(".config/opencode/skills"),
            "OpenCode global must use XDG path, got {}",
            dir.display()
        );
    }

    #[test]
    fn test_resolve_global_target_pi() {
        let dir = resolve_target_dir(true, Some(AgentPlatform::Pi), None).expect("resolve");
        assert!(dir.ends_with(".pi/agent/skills"));
    }

    #[test]
    fn test_resolve_global_target_copilot() {
        let dir = resolve_target_dir(true, Some(AgentPlatform::Copilot), None).expect("resolve");
        assert!(dir.ends_with(".copilot/skills"));
    }

    #[test]
    fn test_global_skills_dir_name_defaults_to_skills_dir() {
        assert_eq!(
            AgentPlatform::Claude.global_skills_dir_name(),
            AgentPlatform::Claude.skills_dir_name()
        );
        assert_eq!(
            AgentPlatform::Pi.global_skills_dir_name(),
            ".pi/agent/skills"
        );
    }

    #[test]
    fn test_global_skills_dir_name_opencode_xdg() {
        assert_eq!(
            AgentPlatform::OpenCode.global_skills_dir_name(),
            ".config/opencode/skills"
        );
        assert_ne!(
            AgentPlatform::OpenCode.global_skills_dir_name(),
            AgentPlatform::OpenCode.skills_dir_name()
        );
    }

    #[test]
    fn test_resolve_project_target_respects_agent() {
        // 项目级必须遵循 --agent；不能硬编码到 .claude/skills
        let repo = PathBuf::from("/tmp/test-repo-skills");
        let dir = resolve_project_target(&repo, AgentPlatform::Codex);
        assert!(
            dir.ends_with(".codex/skills"),
            "project-level install must respect --agent, got {}",
            dir.display()
        );

        let dir_gemini = resolve_project_target(&repo, AgentPlatform::Gemini);
        assert!(dir_gemini.ends_with(".gemini/skills"));

        let dir_qoder = resolve_project_target(&repo, AgentPlatform::Qoder);
        assert!(dir_qoder.ends_with(".qoder-cn/skills"));
    }

    #[test]
    fn test_resolve_custom_path_overrides_all() {
        let dir = resolve_target_dir(false, Some(AgentPlatform::Claude), Some("/tmp/my-skills"))
            .expect("resolve");
        assert_eq!(dir, PathBuf::from("/tmp/my-skills"));
    }

    #[test]
    fn test_resolve_custom_path_rejects_parent_dir_traversal() {
        let result = resolve_target_dir(
            false,
            Some(AgentPlatform::Claude),
            Some("/tmp/my-skills/../../etc"),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_custom_path_rejects_null_byte() {
        let result = resolve_target_dir(false, Some(AgentPlatform::Claude), Some("/tmp/x\0y"));
        assert!(result.is_err());
    }

    #[test]
    fn test_skills_source_dir_is_valid_path() {
        let dir = skills_source_dir();
        assert!(dir.ends_with("skills"));
    }

    #[test]
    fn test_should_count_overwritten_in_bundled_path() {
        // Verify that install_single_skill_bundled correctly increments `overwritten`
        // ONLY when force=true AND dest already existed before the call.
        // This catches the dead-code bug where `args.force && dest.exists()` was
        // checked AFTER files were written (so dest.exists() was always true),
        // causing fresh installs with force=true to falsely increment overwritten.
        let tmp = tempfile::tempdir().expect("create temp dir");

        // Case 1: fresh install (dest does NOT exist), force=true
        // Expected: overwritten=0, installed=1
        let dest_fresh = tmp.path().join("fresh-skill");
        let args = InstallArgs {
            agent: Some(AgentPlatform::Claude),
            global: false,
            force: true,
            custom_path: None,
            source: None,
        };
        let files: &[(&str, &[u8])] = &[("test.md", b"# Test Skill")];

        let mut installed = 0u32;
        let mut skipped = 0u32;
        let mut overwritten = 0u32;

        install_single_skill_bundled(
            &dest_fresh,
            files,
            &args,
            &mut installed,
            &mut skipped,
            &mut overwritten,
        )
        .expect("install should succeed");

        assert_eq!(
            overwritten, 0,
            "fresh install with force=true should NOT count as overwritten"
        );
        assert_eq!(installed, 1, "fresh install should count as installed");
        assert_eq!(skipped, 0, "fresh install should not count as skipped");

        // Case 2: overwrite (dest already exists), force=true
        // Expected: overwritten=1, installed stays at 1
        let mut installed = 0u32;
        let mut skipped = 0u32;
        let mut overwritten = 0u32;

        // Pre-create the destination directory so it counts as an overwrite
        let dest_existing = tmp.path().join("existing-skill");
        std::fs::create_dir_all(&dest_existing).expect("create dest dir");
        std::fs::write(dest_existing.join("old.txt"), b"old data").expect("write old file");

        install_single_skill_bundled(
            &dest_existing,
            files,
            &args,
            &mut installed,
            &mut skipped,
            &mut overwritten,
        )
        .expect("install should succeed");

        assert_eq!(
            overwritten, 1,
            "overwrite of existing dir with force=true should increment overwritten"
        );
        assert_eq!(installed, 0, "overwrite should not count as fresh install");
        assert_eq!(
            skipped, 0,
            "overwrite with force should not count as skipped"
        );
    }

    // -----------------------------------------------------------------------
    // confirm / confirm_with_reader tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_should_return_default_on_empty_input() {
        // Simulate empty input by providing a reader with just a newline
        let input = b"\n";
        let result = confirm_with_reader("Continue?", true, &mut &input[..]).expect("confirm");
        assert!(result, "empty input should return default=true");

        let result = confirm_with_reader("Continue?", false, &mut &input[..]).expect("confirm");
        assert!(!result, "empty input should return default=false");
    }

    #[test]
    fn test_should_accept_yes_variants() {
        for answer in &[b"y\n" as &[u8], b"Y\n", b"yes\n", b"YES\n"] {
            let result = confirm_with_reader("Continue?", false, &mut &**answer).expect("confirm");
            assert!(result, "input {answer:?} should be accepted as yes");
        }
    }

    #[test]
    fn test_should_accept_no_variants() {
        for answer in &[b"n\n" as &[u8], b"N\n", b"no\n", b"NO\n"] {
            let result = confirm_with_reader("Continue?", true, &mut &**answer).expect("confirm");
            assert!(!result, "input {answer:?} should be accepted as no");
        }
    }

    #[test]
    fn test_should_return_default_on_eof() {
        let input: &[u8] = b"";
        let result = confirm_with_reader("Continue?", true, &mut &input[..]).expect("confirm");
        assert!(result, "EOF should return default=true");
    }

    /// 在临时 HOME 写入 plugin 形态注册表。
    fn seed_plugin_registry(home: &std::path::Path, plugin_keys: &[&str]) {
        let dir = home.join(".claude/plugins");
        std::fs::create_dir_all(&dir).expect("create plugins dir");
        let mut plugins = serde_json::Map::new();
        for key in plugin_keys {
            plugins.insert((*key).to_string(), serde_json::json!([]));
        }
        let content = serde_json::json!({ "version": 2, "plugins": plugins });
        std::fs::write(
            dir.join("installed_plugins.json"),
            serde_json::to_string(&content).expect("serialize"),
        )
        .expect("write registry");
    }

    /// 在临时 HOME 写入裸名 skill 目录。
    fn seed_bare_skills(home: &std::path::Path, names: &[&str]) {
        for name in names {
            let dir = home.join(".claude/skills").join(name);
            std::fs::create_dir_all(&dir).expect("create skill dir");
            std::fs::write(dir.join("SKILL.md"), "---\nname: x\n---\n").expect("write SKILL.md");
        }
    }

    #[test]
    fn test_detect_plugin_superpowers() {
        let tmp = tempfile::tempdir().expect("temp dir");
        seed_plugin_registry(tmp.path(), &["superpowers@claude-plugins-official"]);
        let found = detect_skill_sources(tmp.path());
        assert_eq!(found, vec![SkillSourceKind::Superpowers]);
    }

    #[test]
    fn test_detect_plugin_mattpocock() {
        let tmp = tempfile::tempdir().expect("temp dir");
        seed_plugin_registry(tmp.path(), &["mattpocock-skills@mattpocock"]);
        let found = detect_skill_sources(tmp.path());
        assert_eq!(found, vec![SkillSourceKind::Mattpocock]);
    }

    #[test]
    fn test_detect_bare_mattpocock_requires_double_sentinel() {
        let tmp = tempfile::tempdir().expect("temp dir");
        seed_bare_skills(tmp.path(), &["to-spec", "grilling"]);
        let found = detect_skill_sources(tmp.path());
        assert_eq!(found, vec![SkillSourceKind::Mattpocock]);
    }

    #[test]
    fn test_detect_bare_partial_sentinel_is_not_detected() {
        // 只有 to-spec 缺 grilling → 部分命中视同缺失（防同名碰撞误判）
        let tmp = tempfile::tempdir().expect("temp dir");
        seed_bare_skills(tmp.path(), &["to-spec"]);
        assert!(
            detect_skill_sources(tmp.path()).is_empty(),
            "partial sentinel hit must not be detected"
        );
    }

    #[test]
    fn test_detect_bare_superpowers_requires_double_sentinel() {
        let tmp = tempfile::tempdir().expect("temp dir");
        seed_bare_skills(tmp.path(), &["brainstorming", "writing-plans"]);
        let found = detect_skill_sources(tmp.path());
        assert_eq!(found, vec![SkillSourceKind::Superpowers]);
    }

    #[test]
    fn test_detect_empty_home_finds_nothing() {
        let tmp = tempfile::tempdir().expect("temp dir");
        assert!(detect_skill_sources(tmp.path()).is_empty());
    }

    #[test]
    fn test_detect_both_sources_when_both_installed() {
        let tmp = tempfile::tempdir().expect("temp dir");
        seed_plugin_registry(
            tmp.path(),
            &[
                "superpowers@claude-plugins-official",
                "mattpocock-skills@mattpocock",
            ],
        );
        let found = detect_skill_sources(tmp.path());
        assert_eq!(found.len(), 2, "both sources must be reported: {found:?}");
    }

    #[test]
    fn test_detect_malformed_registry_falls_back_to_bare() {
        // 注册表损坏不应 panic，降级到裸名探测
        let tmp = tempfile::tempdir().expect("temp dir");
        let dir = tmp.path().join(".claude/plugins");
        std::fs::create_dir_all(&dir).expect("create dir");
        std::fs::write(dir.join("installed_plugins.json"), "not json").expect("write");
        seed_bare_skills(tmp.path(), &["to-spec", "grilling"]);
        let found = detect_skill_sources(tmp.path());
        assert_eq!(found, vec![SkillSourceKind::Mattpocock]);
    }

    #[test]
    fn test_install_check_blocks_when_no_skill_source() {
        let tmp = tempfile::tempdir().expect("temp dir");
        // 临时目录下既无 plugin 注册表也无裸名哨兵 → 应阻断
        let result = check_skill_source_at(tmp.path());
        let err = result.expect_err("must block when no source installed");
        assert!(
            err.to_string().contains("技能来源缺失"),
            "error must state missing source: {err}"
        );
    }

    #[test]
    fn test_should_include_node_version_hint_in_skill_source_guidance() {
        // Issue #192：装 mattpocock-skills / npx skills 需 Node.js ≥ 22.20.0，
        // 硬阻断引导须内联提示该前置条件，避免用户装来源时二次踩坑。
        assert!(
            SKILL_SOURCE_GUIDANCE.contains("Node.js"),
            "guidance must mention Node.js: {SKILL_SOURCE_GUIDANCE}"
        );
        assert!(
            SKILL_SOURCE_GUIDANCE.contains("22.20.0"),
            "guidance must state the Node.js minimum version 22.20.0: {SKILL_SOURCE_GUIDANCE}"
        );
    }

    #[test]
    fn test_install_check_passes_when_source_detected() {
        let tmp = tempfile::tempdir().expect("temp dir");
        seed_plugin_registry(tmp.path(), &["superpowers@claude-plugins-official"]);
        check_skill_source_at(tmp.path()).expect("must pass with source present");
    }

    #[test]
    fn test_install_check_skips_non_claude_platform() {
        // 非 Claude 平台不做来源检查（技能来源是 Claude Code 生态概念）
        check_skill_source(AgentPlatform::Codex).expect("non-claude must skip check");
    }

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

        let result = copy_skills_dir(&tmp.path().join("source"), &tmp.path().join("target"), true)
            .expect("copy");
        assert_eq!(result.installed, 0);
        assert_eq!(result.overwritten, 1);
        assert_eq!(result.skipped, 0);
        assert!(result.failures.is_empty());

        let content = std::fs::read_to_string(tmp.path().join("target/gf-alpha/SKILL.md"))
            .expect("read updated");
        assert_eq!(content, "# gf-alpha\n", "content must be replaced");
    }

    #[test]
    fn test_copy_skills_dir_installs_new() {
        let tmp = tempfile::tempdir().expect("tempdir");
        seed_source(&tmp, &["gf-alpha", "gf-beta"]);
        seed_target(&tmp, &["gf-alpha"]);

        let result = copy_skills_dir(&tmp.path().join("source"), &tmp.path().join("target"), true)
            .expect("copy");
        assert_eq!(result.installed, 1, "gf-beta must be newly installed");
        assert_eq!(result.overwritten, 1);
        assert!(result.failures.is_empty());
        assert!(tmp.path().join("target/gf-beta/SKILL.md").exists());
    }

    #[test]
    fn test_copy_skills_dir_preserves_other_dirs() {
        let tmp = tempfile::tempdir().expect("tempdir");
        seed_source(&tmp, &["gf-alpha"]);
        seed_target(&tmp, &["gf-alpha"]);
        let other = tmp.path().join("target/not-a-skill");
        std::fs::create_dir_all(&other).expect("create other dir");
        std::fs::write(other.join("README.md"), "keep me\n").expect("write other dir");

        copy_skills_dir(&tmp.path().join("source"), &tmp.path().join("target"), true)
            .expect("copy");

        assert!(
            other.join("README.md").exists(),
            "non-gf-* dirs must be left untouched"
        );
    }

    #[test]
    fn test_copy_skills_dir_collects_failures_and_continues() {
        let tmp = tempfile::tempdir().expect("tempdir");
        // 源: 3 个 skills
        seed_source(&tmp, &["gf-alpha", "gf-beta", "gf-gamma"]);
        // 目标: gf-beta 位置放一个普通文件（非目录），使 remove_dir_all 失败
        std::fs::create_dir_all(tmp.path().join("target")).expect("create target");
        std::fs::write(tmp.path().join("target/gf-beta"), "i am a file, not a dir")
            .expect("write file where skill dir should be");

        let result = copy_skills_dir(&tmp.path().join("source"), &tmp.path().join("target"), true)
            .expect("copy_skills_dir must not abort on individual skill failure");

        // gf-alpha 和 gf-gamma 必须成功安装
        assert!(
            tmp.path().join("target/gf-alpha/SKILL.md").exists(),
            "gf-alpha must be installed despite gf-beta failure"
        );
        assert!(
            tmp.path().join("target/gf-gamma/SKILL.md").exists(),
            "gf-gamma must be installed despite gf-beta failure"
        );
        // gf-beta 必须在失败列表中
        assert_eq!(result.failures.len(), 1, "exactly one failure");
        assert_eq!(result.failures[0], "gf-beta");
        // 计数: 2 新增 (alpha, gamma), 0 覆盖, 0 跳过
        assert_eq!(result.installed, 2);
        assert_eq!(result.overwritten, 0);
        assert_eq!(result.skipped, 0);
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
        let cli = TestCli::parse_from([
            "test", "update", "-g", "--agent", "codex", "--path", "/tmp/x",
        ]);
        let TestCmd::Update(args) = cli.cmd;
        assert!(args.global);
        assert_eq!(args.agent, Some(AgentPlatform::Codex));
        assert_eq!(args.custom_path.as_deref(), Some("/tmp/x"));
    }
}
