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
use gitflow_core::AuthChecker;
use is_terminal::IsTerminal;

use crate::error_reporter::read_co_contribution_flag;

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

    /// 返回该 Agent 的 hook 子目录名（相对于项目根或 home）。
    #[must_use]
    pub fn hooks_dir_name(self) -> &'static str {
        match self {
            AgentPlatform::Claude => ".claude/hooks",
            AgentPlatform::Codex => ".codex/hooks",
            AgentPlatform::OpenCode => ".opencode/hooks",
            AgentPlatform::Gemini => ".gemini/hooks",
            AgentPlatform::Copilot => ".copilot/hooks",
        }
    }

    /// 返回该 Agent 的 settings.json 路径（相对于项目根或 home）。
    #[must_use]
    pub fn settings_file_path(self) -> &'static str {
        match self {
            AgentPlatform::Claude => ".claude/settings.json",
            AgentPlatform::Codex => ".codex/settings.json",
            AgentPlatform::OpenCode => ".opencode/settings.json",
            AgentPlatform::Gemini => ".gemini/settings.json",
            AgentPlatform::Copilot => ".copilot/settings.json",
        }
    }

    /// 该 Agent 是否支持 Stop hook 配置（写入 `settings.json` 的 `hooks.Stop`）。
    ///
    /// 当前仅 Claude Code 与 Codex 识别此 schema；
    /// `OpenCode` / Gemini / Copilot 不支持，安装时应跳过 hook 以避免污染其配置。
    #[must_use]
    pub const fn supports_hooks(self) -> bool {
        matches!(self, AgentPlatform::Claude | AgentPlatform::Codex)
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

    /// 启用自动 bug 上报（Stop Hook），默认开启
    #[arg(long = "report-bug", default_value_t = true, action = ArgAction::Set)]
    pub report_bug: bool,
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
    // 自定义路径优先
    if let Some(p) = custom_path {
        return Ok(PathBuf::from(p));
    }

    let platform = agent.unwrap_or_else(AgentPlatform::detect);

    if global {
        let home = dirs::home_dir().ok_or_else(|| miette::miette!("无法确定 HOME 目录"))?;
        Ok(home.join(platform.skills_dir_name()))
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
    // 一次性解析目标平台；避免 `AgentPlatform::detect()` 在 resolve_target_dir
    // 与 install_hook 分支被重复调用。
    let platform = args.agent.unwrap_or_else(AgentPlatform::detect);

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

    // 安装 auto-report-bug hook（可通过 --report-bug=false 跳过）
    // 仅当目标 Agent 支持 Stop hook 时才写入；其他平台只装 skills。
    if args.report_bug {
        if platform.supports_hooks() {
            install_hook(args.global, args.force, platform)?;
        } else {
            // `AgentPlatform` 是 derived `ValueEnum`，`to_possible_value()` 对
            // 所有 variant 都返回 `Some`；保留 fallback 以满足 `-D clippy::expect_used`
            // 与 `-D clippy::unwrap_used`，避免 panic 路径。
            let name = platform.to_possible_value().map_or_else(
                || format!("{platform:?}").to_lowercase(),
                |pv| pv.get_name().to_owned(),
            );
            println!("⚠ Agent {name} 不支持 Stop hook，已跳过 hook 安装");
        }
    }

    // Co-contribution plan — interactive opt-in
    try_enable_co_contribution(platform)?;

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

/// Resolve hook directory, settings path, and command for a given install scope.
///
/// 拆成 global / project 两个私有 helper：
/// - `resolve_global_hook_paths`：基于 HOME，命令用 `~/` 简写
/// - `resolve_project_hook_paths`：基于 repo，命令用 `$(git rev-parse ...)` 解析
///
/// 两个 helper 都接受路径参数，便于单测覆盖。
fn resolve_hook_paths(
    global: bool,
    platform: AgentPlatform,
) -> miette::Result<(PathBuf, PathBuf, String)> {
    if global {
        let home = dirs::home_dir().ok_or_else(|| miette::miette!("无法确定 HOME 目录"))?;
        Ok(resolve_global_hook_paths(&home, platform))
    } else {
        let repo = git_repo_root()?;
        Ok(resolve_project_hook_paths(&repo, platform))
    }
}

/// 构建 Stop hook 命令，使用正确的平台特定 hooks 路径。
///
/// 生成的命令解析 git 仓库根目录，检查脚本是否存在且可执行，然后运行。
/// guard 保证非 git 仓库或脚本缺失时静默跳过，可安全用于全局注册。
fn build_auto_report_hook_cmd(hooks_dir: &str) -> String {
    format!(
        "bash -c 'p=$(git rev-parse --show-toplevel 2>/dev/null) && [ -x \
         \"$p/{hooks_dir}/auto-report-bug.sh\" ] && bash \"$p/{hooks_dir}/auto-report-bug.sh\"'"
    )
}

fn resolve_global_hook_paths(
    home: &std::path::Path,
    platform: AgentPlatform,
) -> (PathBuf, PathBuf, String) {
    let hooks_dir = platform.hooks_dir_name();
    let settings_file = platform.settings_file_path();
    let cmd = build_auto_report_hook_cmd(hooks_dir);
    (home.join(hooks_dir), home.join(settings_file), cmd)
}

fn resolve_project_hook_paths(
    repo: &std::path::Path,
    platform: AgentPlatform,
) -> (PathBuf, PathBuf, String) {
    let hooks_dir = platform.hooks_dir_name();
    let settings_file = platform.settings_file_path();
    let cmd = build_auto_report_hook_cmd(hooks_dir);
    (repo.join(hooks_dir), repo.join(settings_file), cmd)
}

/// 安装 Stop hook 到项目级或全局配置。
///
/// `install_hook` 向平台 hooks 目录（如 `.claude/hooks/`）复制脚本，
/// 并生成引用该路径的 hook 命令写入 settings 文件。
/// 用于 `uninstall_hook` 清理历史遗留副本，与向后兼容。
/// 配置写入平台对应的 settings 文件（Claude 下为
/// `.claude/settings.json` 或 `~/.claude/settings.json`）。
fn install_hook(global: bool, force: bool, platform: AgentPlatform) -> miette::Result<()> {
    let hook_script = include_bytes!("../../hooks/auto-report-bug.sh");

    let (hook_dir, settings_path, cmd) = resolve_hook_paths(global, platform)?;

    // 写 hook 脚本
    std::fs::create_dir_all(&hook_dir).map_err(|e| miette::miette!("无法创建 hook 目录: {e}"))?;
    let hook_path = hook_dir.join("auto-report-bug.sh");
    if !hook_path.exists() || force {
        std::fs::write(&hook_path, hook_script)
            .map_err(|e| miette::miette!("无法写入 hook 脚本: {e}"))?;
    }

    // 合并 Hook 配置到 settings.json
    let settings_json = if settings_path.exists() {
        let content = std::fs::read_to_string(&settings_path)
            .map_err(|e| miette::miette!("无法读取配置: {e}"))?;
        serde_json::from_str::<serde_json::Value>(&content)
            .map_err(|e| miette::miette!("无法解析配置: {e}"))?
    } else {
        serde_json::json!({})
    };

    let new_settings = merge_stop_hook(settings_json, &cmd);
    let formatted =
        serde_json::to_string_pretty(&new_settings).map_err(|e| miette::miette!("JSON: {e}"))?;
    std::fs::write(&settings_path, formatted).map_err(|e| miette::miette!("写入配置: {e}"))?;

    println!(
        "✅ Hook 已安装 ({})",
        if global { "全局" } else { "项目级" }
    );
    Ok(())
}

/// 合并 Stop Hook 配置到 JSON 对象中。
fn merge_stop_hook(mut json: serde_json::Value, cmd: &str) -> serde_json::Value {
    let hook = serde_json::json!({
        "matcher": "gitflow",
        "hooks": [
            {
                "type": "command",
                "command": cmd
            }
        ]
    });

    if let serde_json::Value::Object(obj) = &mut json {
        let hooks = obj
            .entry("hooks")
            .or_insert(serde_json::json!({"Stop": []}));
        if let serde_json::Value::Object(h) = hooks {
            let stops = h.entry("Stop").or_insert(serde_json::json!([]));
            if let serde_json::Value::Array(arr) = stops {
                // 替换已存在的 gitflow hook 或追加
                if let Some(existing) = arr
                    .iter_mut()
                    .find(|v| v.get("matcher").and_then(|m| m.as_str()) == Some("gitflow"))
                {
                    *existing = hook;
                } else {
                    arr.push(hook);
                }
            }
        }
    } else {
        json = serde_json::json!({
            "hooks": {
                "Stop": [hook]
            }
        });
    }

    json
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

    // 移除 Hook 配置（幂等操作，对所有平台都尝试；未安装时静默退出）
    uninstall_hook(args.global, platform)?;

    Ok(())
}

/// 从配置中移除 Stop Hook，并清理 hook 脚本文件。
fn uninstall_hook(global: bool, platform: AgentPlatform) -> miette::Result<()> {
    let (hook_dir, settings_path) = if global {
        let home = dirs::home_dir().ok_or_else(|| miette::miette!("无法确定 HOME 目录"))?;
        (
            home.join(platform.hooks_dir_name()),
            home.join(platform.settings_file_path()),
        )
    } else {
        let repo = git_repo_root()?;
        (
            repo.join(platform.hooks_dir_name()),
            repo.join(platform.settings_file_path()),
        )
    };

    // 删除 hook 脚本文件
    let hook_script = hook_dir.join("auto-report-bug.sh");
    if hook_script.exists() {
        std::fs::remove_file(&hook_script)
            .map_err(|e| miette::miette!("无法删除 hook 脚本 {}: {e}", hook_script.display()))?;
        // 如果 hook 目录为空，也删除目录
        if hook_dir.exists()
            && std::fs::read_dir(&hook_dir).map_or(true, |mut d| d.next().is_none())
        {
            std::fs::remove_dir(&hook_dir).ok();
        }
    }

    if !settings_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&settings_path)
        .map_err(|e| miette::miette!("无法读取配置: {e}"))?;
    let mut json: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| miette::miette!("无法解析: {e}"))?;

    if let Some(obj) = json.as_object_mut()
        && let Some(hooks) = obj.get_mut("hooks")
        && let Some(stop) = hooks.get_mut("Stop")
        && let Some(arr) = stop.as_array_mut()
    {
        arr.retain(|v| v.get("matcher").and_then(|m| m.as_str()) != Some("gitflow"));
    }

    let formatted =
        serde_json::to_string_pretty(&json).map_err(|e| miette::miette!("JSON: {e}"))?;
    std::fs::write(&settings_path, formatted).map_err(|e| miette::miette!("写入: {e}"))?;
    println!("✅ Hook 已卸载");

    Ok(())
}

// ---------------------------------------------------------------------------
// Co-contribution opt-in
// ---------------------------------------------------------------------------

/// Prompt the user to join the co-contribution plan and verify GitHub auth.
///
/// In non-interactive mode, silently skips. In interactive mode, asks the user
/// whether to join, checks `gh auth status`, and writes the settings.json marker
/// on success.
fn try_enable_co_contribution(platform: AgentPlatform) -> miette::Result<()> {
    if !std::io::stderr().is_terminal() {
        println!("ℹ️ 非交互模式，已跳过共建计划");
        return Ok(());
    }

    // 已加入则不再重复询问（全局 settings 一次性标记）
    if let Some(home) = dirs::home_dir() {
        let (_hook_dir, settings_path, _cmd) = resolve_global_hook_paths(&home, platform);
        if read_co_contribution_flag(&settings_path) {
            return Ok(());
        }
    }

    println!();
    println!("🤝 共建计划：加入后，CLI 错误将自动上报为 GitHub Issue，帮助改进 gf。");
    println!("   用户级设置，加入一次即所有项目生效。");
    println!();

    if !confirm("是否加入共建计划？", true)? {
        println!("已跳过共建计划。你可以稍后运行 `skills install --force` 重新加入。");
        return Ok(());
    }

    // Check GitHub auth
    let auth_provider = gitflow_github::GitHubAuthProvider::new();
    if auth_provider.is_authenticated() {
        merge_co_contribution(platform)?;
        println!("✅ 共建计划已激活");
    } else {
        println!("⚠️ 未检测到 GitHub 登录。");
        if confirm("是否现在执行 `gh auth login`？", true)? {
            let status = std::process::Command::new("gh")
                .args(["auth", "login"])
                .stdin(std::process::Stdio::inherit())
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .status();
            match status {
                Ok(s) if s.success() => {
                    merge_co_contribution(platform)?;
                    println!("✅ 共建计划已激活");
                }
                _ => {
                    println!(
                        "登录失败。请手动运行 `gh auth login`，然后重新 `skills install --force`。"
                    );
                }
            }
        } else {
            println!(
                "请手动运行 `gh auth login`，然后重新 `skills install --force` 激活共建计划。"
            );
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Co-contribution marker
// ---------------------------------------------------------------------------

/// Merge the co-contribution marker into a settings JSON object.
///
/// Sets `gitflow.co_contribution = true` and `gitflow.joined_at` to the given
/// ISO 8601 timestamp. Preserves all existing fields.
fn merge_co_contribution_json(mut json: serde_json::Value, joined_at: &str) -> serde_json::Value {
    if let serde_json::Value::Object(ref mut obj) = json {
        let gitflow = obj.entry("gitflow").or_insert(serde_json::json!({}));
        if let serde_json::Value::Object(gf) = gitflow {
            gf.insert("co_contribution".into(), serde_json::json!(true));
            gf.insert("joined_at".into(), serde_json::json!(joined_at));
        }
    } else {
        json = serde_json::json!({
            "gitflow": {
                "co_contribution": true,
                "joined_at": joined_at
            }
        });
    }
    json
}

/// Write the co-contribution marker to the platform's settings.json.
///
/// Reads the existing settings file (or creates an empty JSON object),
/// merges the `gitflow.co_contribution` field, and writes back.
fn merge_co_contribution(platform: AgentPlatform) -> miette::Result<()> {
    let home = dirs::home_dir().ok_or_else(|| miette::miette!("无法确定 HOME 目录"))?;
    let (_hook_dir, settings_path, _cmd) = resolve_global_hook_paths(&home, platform);

    let existing = if settings_path.exists() {
        let content = std::fs::read_to_string(&settings_path)
            .map_err(|e| miette::miette!("无法读取配置 {}: {e}", settings_path.display()))?;
        serde_json::from_str::<serde_json::Value>(&content)
            .map_err(|e| miette::miette!("无法解析配置 {}: {e}", settings_path.display()))?
    } else {
        serde_json::json!({})
    };

    let joined_at = iso8601_utc_now_co_contribution();
    let new_settings = merge_co_contribution_json(existing, &joined_at);
    let formatted = serde_json::to_string_pretty(&new_settings)
        .map_err(|e| miette::miette!("JSON 序列化失败: {e}"))?;

    // 确保目录存在
    if let Some(parent) = settings_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| miette::miette!("无法创建目录 {}: {e}", parent.display()))?;
    }

    std::fs::write(&settings_path, formatted)
        .map_err(|e| miette::miette!("写入配置失败 {}: {e}", settings_path.display()))?;

    Ok(())
}

/// Format the current UTC time as ISO 8601 for the co-contribution marker.
#[allow(
    dead_code,
    reason = "called by `merge_co_contribution`; direct calls from tests for isolation"
)]
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_lossless,
    clippy::cast_sign_loss,
    reason = "Howard Hinnant's algorithm operates on mixed-sign integer ranges within known bounds"
)]
fn iso8601_utc_now_co_contribution() -> String {
    // Reuse the same algorithm as error_reporter::iso8601_utc_now
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    // Inline Howard Hinnant's algorithm (same as error_reporter)
    let day_secs = secs % 86_400;
    let hours = day_secs / 3_600;
    let minutes = (day_secs % 3_600) / 60;
    let seconds = day_secs % 60;
    let days = (secs / 86_400) as i64;
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe as i64 + era * 400;
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}T{hours:02}:{minutes:02}:{seconds:02}Z")
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
    fn test_agent_detect_always_returns_claude() {
        // 契约：detect() 默认固定返回 Claude，不扫描 $HOME 下其他平台目录。
        // 其他平台必须通过 `--agent` 显式指定。
        assert_eq!(AgentPlatform::detect(), AgentPlatform::Claude);
    }

    #[test]
    fn test_agent_platform_claude_hooks_dir() {
        assert_eq!(AgentPlatform::Claude.hooks_dir_name(), ".claude/hooks");
    }

    #[test]
    fn test_agent_platform_codex_hooks_dir() {
        assert_eq!(AgentPlatform::Codex.hooks_dir_name(), ".codex/hooks");
    }

    #[test]
    fn test_agent_platform_opencode_hooks_dir() {
        assert_eq!(AgentPlatform::OpenCode.hooks_dir_name(), ".opencode/hooks");
    }

    #[test]
    fn test_agent_platform_gemini_hooks_dir() {
        assert_eq!(AgentPlatform::Gemini.hooks_dir_name(), ".gemini/hooks");
    }

    #[test]
    fn test_agent_platform_copilot_hooks_dir() {
        assert_eq!(AgentPlatform::Copilot.hooks_dir_name(), ".copilot/hooks");
    }

    #[test]
    fn test_agent_platform_claude_settings_path() {
        assert_eq!(
            AgentPlatform::Claude.settings_file_path(),
            ".claude/settings.json"
        );
    }

    #[test]
    fn test_agent_platform_codex_settings_path() {
        assert_eq!(
            AgentPlatform::Codex.settings_file_path(),
            ".codex/settings.json"
        );
    }

    #[test]
    fn test_agent_platform_opencode_settings_path() {
        assert_eq!(
            AgentPlatform::OpenCode.settings_file_path(),
            ".opencode/settings.json"
        );
    }

    #[test]
    fn test_agent_platform_gemini_settings_path() {
        assert_eq!(
            AgentPlatform::Gemini.settings_file_path(),
            ".gemini/settings.json"
        );
    }

    #[test]
    fn test_agent_platform_copilot_settings_path() {
        assert_eq!(
            AgentPlatform::Copilot.settings_file_path(),
            ".copilot/settings.json"
        );
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
    }

    #[test]
    fn test_agent_supports_hooks_matrix() {
        assert!(AgentPlatform::Claude.supports_hooks());
        assert!(AgentPlatform::Codex.supports_hooks());
        assert!(!AgentPlatform::OpenCode.supports_hooks());
        assert!(!AgentPlatform::Gemini.supports_hooks());
        assert!(!AgentPlatform::Copilot.supports_hooks());
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

    #[test]
    fn test_merge_stop_hook_creates_nested_format() {
        let input = serde_json::json!({});
        let result = merge_stop_hook(input, "bash hooks/auto-report-bug.sh");

        let hooks = result
            .pointer("/hooks/Stop/0/hooks")
            .and_then(serde_json::Value::as_array)
            .expect("should create nested hooks array");
        assert_eq!(hooks.len(), 1);
        assert_eq!(
            hooks[0].get("type").and_then(serde_json::Value::as_str),
            Some("command")
        );
        assert_eq!(
            hooks[0].get("command").and_then(serde_json::Value::as_str),
            Some("bash hooks/auto-report-bug.sh")
        );
        assert_eq!(
            result
                .pointer("/hooks/Stop/0/matcher")
                .and_then(serde_json::Value::as_str),
            Some("gitflow")
        );
    }

    #[test]
    fn test_merge_stop_hook_replaces_existing_gitflow() {
        let input = serde_json::json!({
            "hooks": {
                "Stop": [
                    {
                        "matcher": "gitflow",
                        "command": "old-command.sh"
                    }
                ]
            }
        });
        let result = merge_stop_hook(input, "bash hooks/auto-report-bug.sh");

        let stop = result
            .pointer("/hooks/Stop")
            .and_then(serde_json::Value::as_array)
            .expect("Stop array should exist");
        assert_eq!(stop.len(), 1, "should replace, not duplicate");
        assert!(
            stop[0].get("hooks").is_some(),
            "should use nested hooks format"
        );
        assert!(
            stop[0].get("command").is_none(),
            "flat command field should be gone"
        );
    }

    #[test]
    fn test_merge_stop_hook_preserves_other_hooks() {
        let input = serde_json::json!({
            "hooks": {
                "Stop": [
                    {
                        "matcher": "other-agent",
                        "hooks": [
                            {
                                "type": "command",
                                "command": "other-command.sh"
                            }
                        ]
                    }
                ]
            }
        });
        let result = merge_stop_hook(input, "bash hooks/auto-report-bug.sh");

        let stop = result
            .pointer("/hooks/Stop")
            .and_then(serde_json::Value::as_array)
            .expect("Stop array should exist");
        assert_eq!(stop.len(), 2, "should keep other matcher and add gitflow");
    }

    /// Test is Unix-only: uses `dirs::home_dir()` which on Windows ignores HOME env var.
    #[cfg(unix)]
    #[test]
    fn test_uninstall_hook_removes_gitflow() {
        // 用临时目录隔离，避免污染真实 HOME
        let tmp = tempfile::tempdir().expect("create temp dir");

        // 准备一个含 gitflow hook 的 settings.json（新嵌套格式）
        let settings_path = tmp.path().join(".claude/settings.json");
        std::fs::create_dir_all(tmp.path().join(".claude")).expect("create .claude dir");
        let content = serde_json::json!({
            "hooks": {
                "Stop": [
                    {
                        "matcher": "gitflow",
                        "hooks": [
                            {
                                "type": "command",
                                "command": "bash hooks/auto-report-bug.sh"
                            }
                        ]
                    }
                ]
            }
        });
        std::fs::write(
            &settings_path,
            serde_json::to_string_pretty(&content).expect("serialize"),
        )
        .expect("write settings");

        // 调用 uninstall_hook（全局模式），用 temp_env 隔离 HOME
        temp_env::with_var("HOME", Some(tmp.path()), || {
            super::uninstall_hook(true, AgentPlatform::Claude).expect("uninstall should succeed");
        });

        // 验证 gitflow hook 已被删除
        let after = std::fs::read_to_string(&settings_path).expect("read after");
        let parsed: serde_json::Value = serde_json::from_str(&after).expect("parse after");
        let stop = parsed
            .pointer("/hooks/Stop")
            .and_then(serde_json::Value::as_array)
            .expect("Stop should exist");
        assert!(
            stop.iter()
                .all(|v| v.get("matcher").and_then(serde_json::Value::as_str) != Some("gitflow")),
            "gitflow hook should be removed"
        );
    }

    /// Test is Unix-only: uses `dirs::home_dir()` which on Windows ignores HOME env var.
    #[cfg(unix)]
    #[test]
    fn test_uninstall_hook_preserves_others() {
        let tmp = tempfile::tempdir().expect("create temp dir");

        let settings_path = tmp.path().join(".claude/settings.json");
        std::fs::create_dir_all(tmp.path().join(".claude")).expect("create .claude dir");
        let content = serde_json::json!({
            "hooks": {
                "Stop": [
                    {
                        "matcher": "gitflow",
                        "hooks": [
                            {
                                "type": "command",
                                "command": "bash hooks/auto-report-bug.sh"
                            }
                        ]
                    },
                    {
                        "matcher": "other-agent",
                        "hooks": [
                            {
                                "type": "command",
                                "command": "other.sh"
                            }
                        ]
                    }
                ]
            }
        });
        std::fs::write(
            &settings_path,
            serde_json::to_string_pretty(&content).expect("serialize"),
        )
        .expect("write settings");

        temp_env::with_var("HOME", Some(tmp.path()), || {
            super::uninstall_hook(true, AgentPlatform::Claude).expect("uninstall should succeed");
        });

        let after = std::fs::read_to_string(&settings_path).expect("read after");
        let parsed: serde_json::Value = serde_json::from_str(&after).expect("parse after");
        let stop = parsed
            .pointer("/hooks/Stop")
            .and_then(serde_json::Value::as_array)
            .expect("Stop should exist");
        assert_eq!(stop.len(), 1, "other-agent hook should remain");
        assert_eq!(
            stop[0].get("matcher").and_then(serde_json::Value::as_str),
            Some("other-agent")
        );
    }

    /// Test is Unix-only: uses `dirs::home_dir()` which on Windows ignores HOME env var.
    #[cfg(unix)]
    #[test]
    fn test_uninstall_hook_deletes_script_file_and_empty_dir() {
        let tmp = tempfile::tempdir().expect("create temp dir");

        // Create .claude/hooks/ directory with a fake hook script
        let hooks_dir = tmp.path().join(".claude/hooks");
        std::fs::create_dir_all(&hooks_dir).expect("create hooks dir");
        let hook_script = hooks_dir.join("auto-report-bug.sh");
        std::fs::write(&hook_script, b"#!/bin/bash\necho test\n").expect("write hook script");

        // Create settings.json with gitflow hook
        let settings_path = tmp.path().join(".claude/settings.json");
        let content = serde_json::json!({
            "hooks": {
                "Stop": [
                    {
                        "matcher": "gitflow",
                        "hooks": [
                            {
                                "type": "command",
                                "command": "bash .claude/hooks/auto-report-bug.sh"
                            }
                        ]
                    }
                ]
            }
        });
        std::fs::write(
            &settings_path,
            serde_json::to_string_pretty(&content).expect("serialize"),
        )
        .expect("write settings");

        temp_env::with_var("HOME", Some(tmp.path()), || {
            super::uninstall_hook(true, AgentPlatform::Claude).expect("uninstall should succeed");
        });

        // Verify script file was deleted
        assert!(
            !hook_script.exists(),
            "hook script should be deleted by uninstall"
        );
        // Verify empty hooks dir was removed
        assert!(
            !hooks_dir.exists(),
            "empty hooks directory should be removed"
        );
    }

    #[test]
    fn test_build_auto_report_hook_cmd_uses_provided_hooks_dir() {
        let cmd = build_auto_report_hook_cmd(".claude/hooks");
        assert!(
            cmd.contains(".claude/hooks/auto-report-bug.sh"),
            "command should reference .claude/hooks/auto-report-bug.sh, got: {cmd}"
        );
        assert!(
            cmd.contains("git rev-parse --show-toplevel"),
            "command should resolve git repo root"
        );
        assert!(
            cmd.contains("[ -x"),
            "command should check script is executable"
        );
    }

    #[test]
    fn test_build_auto_report_hook_cmd_works_for_other_platforms() {
        let cmd = build_auto_report_hook_cmd(".codex/hooks");
        assert!(cmd.contains(".codex/hooks/auto-report-bug.sh"));
    }

    #[test]
    fn test_resolve_project_hook_paths_uses_hooks_dir() {
        let repo = PathBuf::from("/tmp/test-repo");
        let (hook_dir, settings_path, cmd) =
            resolve_project_hook_paths(&repo, AgentPlatform::Claude);
        assert_eq!(
            hook_dir,
            repo.join(".claude/hooks"),
            "hook should be in .claude/hooks/"
        );
        assert_eq!(settings_path, repo.join(".claude/settings.json"));
        assert!(
            cmd.contains(".claude/hooks/auto-report-bug.sh"),
            "command should reference .claude/hooks/auto-report-bug.sh, got: {cmd}"
        );
    }

    #[test]
    fn test_resolve_global_hook_paths_uses_claude_hooks_dir() {
        let home = PathBuf::from("/home/user");
        let (hook_dir, settings_path, cmd) =
            resolve_global_hook_paths(&home, AgentPlatform::Claude);
        assert_eq!(hook_dir, home.join(".claude/hooks"));
        assert_eq!(settings_path, home.join(".claude/settings.json"));
        assert!(
            cmd.contains(".claude/hooks/auto-report-bug.sh"),
            "command should reference .claude/hooks/auto-report-bug.sh, got: {cmd}"
        );
    }

    #[test]
    fn test_should_parse_report_bug_false() {
        use clap::Parser;

        #[derive(Debug, Parser)]
        struct TestCli {
            #[command(subcommand)]
            cmd: TestCmd,
        }

        #[derive(Debug, Subcommand)]
        enum TestCmd {
            Install(InstallArgs),
        }

        let cli = TestCli::parse_from(["test", "install", "--report-bug=false"]);
        let TestCmd::Install(args) = cli.cmd;
        assert!(
            !args.report_bug,
            "--report-bug=false must set report_bug to false"
        );
    }

    #[test]
    fn test_should_default_report_bug_to_true() {
        use clap::Parser;

        #[derive(Debug, Parser)]
        struct TestCli {
            #[command(subcommand)]
            cmd: TestCmd,
        }

        #[derive(Debug, Subcommand)]
        enum TestCmd {
            Install(InstallArgs),
        }

        let cli = TestCli::parse_from(["test", "install"]);
        let TestCmd::Install(args) = cli.cmd;
        assert!(args.report_bug, "report_bug must default to true");
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
            report_bug: false,
            custom_path: None,
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

    // -----------------------------------------------------------------------
    // merge_co_contribution_json tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_should_merge_co_contribution_into_empty_settings() {
        let input = serde_json::json!({});
        let result = merge_co_contribution_json(input, "2026-07-09T08:30:00Z");

        assert_eq!(
            result
                .pointer("/gitflow/co_contribution")
                .and_then(serde_json::Value::as_bool),
            Some(true)
        );
        assert_eq!(
            result
                .pointer("/gitflow/joined_at")
                .and_then(serde_json::Value::as_str),
            Some("2026-07-09T08:30:00Z")
        );
    }

    #[test]
    fn test_should_preserve_existing_hooks_when_merging_co_contribution() {
        let input = serde_json::json!({
            "hooks": {
                "Stop": [
                    {
                        "matcher": "gitflow",
                        "hooks": [{"type": "command", "command": "bash hook.sh"}]
                    }
                ]
            }
        });
        let result = merge_co_contribution_json(input, "2026-07-09T08:30:00Z");

        // hooks must be preserved
        let stop = result
            .pointer("/hooks/Stop")
            .and_then(serde_json::Value::as_array);
        assert!(stop.is_some(), "existing hooks must be preserved");
        assert_eq!(stop.expect("stop array").len(), 1);

        // co_contribution must be added
        assert_eq!(
            result
                .pointer("/gitflow/co_contribution")
                .and_then(serde_json::Value::as_bool),
            Some(true)
        );
    }

    #[test]
    fn test_should_update_existing_gitflow_section() {
        let input = serde_json::json!({
            "gitflow": {
                "co_contribution": false,
                "joined_at": "2020-01-01T00:00:00Z"
            }
        });
        let result = merge_co_contribution_json(input, "2026-07-09T08:30:00Z");

        assert_eq!(
            result
                .pointer("/gitflow/co_contribution")
                .and_then(serde_json::Value::as_bool),
            Some(true),
            "co_contribution must be updated to true"
        );
        assert_eq!(
            result
                .pointer("/gitflow/joined_at")
                .and_then(serde_json::Value::as_str),
            Some("2026-07-09T08:30:00Z"),
            "joined_at must be updated"
        );
    }

    /// Test is Unix-only: uses `dirs::home_dir()` which on Windows ignores HOME env var.
    #[cfg(unix)]
    #[test]
    fn test_should_write_co_contribution_to_global_path() {
        let temp = tempfile::tempdir().expect("tempdir");
        let home = temp.path();

        // 使用 temp_env 安全地设置 HOME 环境变量
        temp_env::with_var("HOME", Some(home), || {
            // 验证始终写入全局路径（不依赖 --global 标志）
            let result = merge_co_contribution(AgentPlatform::Claude);

            assert!(
                result.is_ok(),
                "merge_co_contribution should succeed: {:?}",
                result.err()
            );

            // 验证写入全局路径
            let global_settings = home.join(".claude/settings.json");
            assert!(
                global_settings.exists(),
                "global settings.json must be created"
            );

            let content = std::fs::read_to_string(&global_settings).expect("read");
            let json: serde_json::Value = serde_json::from_str(&content).expect("parse");

            assert_eq!(
                json.pointer("/gitflow/co_contribution")
                    .and_then(serde_json::Value::as_bool),
                Some(true),
                "co_contribution must be true in global settings"
            );
            assert!(
                json.pointer("/gitflow/joined_at")
                    .and_then(serde_json::Value::as_str)
                    .is_some(),
                "joined_at must be set in global settings"
            );
        });
    }

    /// 验证：全局 settings 已存在 `co_contribution=true` 时，`try_enable_co_contribution`
    /// 应直接返回 Ok 且不触发任何交互/写入。
    #[cfg(unix)]
    #[test]
    fn test_should_skip_prompt_when_co_contribution_already_enabled() {
        let temp = tempfile::tempdir().expect("tempdir");
        let home = temp.path();
        let settings_path = home.join(".claude/settings.json");
        std::fs::create_dir_all(home.join(".claude")).expect("create .claude dir");
        std::fs::write(
            &settings_path,
            r#"{"gitflow": {"co_contribution": true, "joined_at": "2026-07-09T08:30:00Z"}}"#,
        )
        .expect("write settings");

        temp_env::with_var("HOME", Some(home), || {
            // 非交互模式 + 已加入 → 应直接返回 Ok，不报错
            let result = try_enable_co_contribution(AgentPlatform::Claude);
            assert!(
                result.is_ok(),
                "try_enable_co_contribution should succeed when already joined: {:?}",
                result.err()
            );

            // 文件内容应保持不变（未被覆盖）
            let content = std::fs::read_to_string(&settings_path).expect("read");
            let json: serde_json::Value = serde_json::from_str(&content).expect("parse");
            assert_eq!(
                json.pointer("/gitflow/joined_at")
                    .and_then(serde_json::Value::as_str),
                Some("2026-07-09T08:30:00Z"),
                "joined_at must not be modified when already joined"
            );
        });
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
