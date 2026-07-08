//! `gitflow skills` 子命令实现。
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

use std::path::PathBuf;

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

    /// 目标 Agent 平台（默认自动检测）
    #[arg(long, value_enum)]
    pub agent: Option<AgentPlatform>,

    /// 自定义安装路径（最高优先级）
    #[arg(long = "path")]
    pub custom_path: Option<String>,

    /// 强制覆盖已存在的 skills
    #[arg(short = 'f', long, action = ArgAction::SetTrue)]
    pub force: bool,

    /// 启用自动 bug 上报（Stop Hook），默认开启
    #[arg(long, default_value_t = true, action = ArgAction::SetTrue)]
    pub report_bug: bool,
}

/// `skills list` 参数。
#[derive(Debug, Args)]
pub struct ListArgs {
    /// 列出全局用户目录下的 skills
    #[arg(short = 'g', long, action = ArgAction::SetTrue)]
    pub global: bool,

    /// 目标 Agent 平台（默认自动检测）
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

    /// 目标 Agent 平台（默认自动检测）
    #[arg(long, value_enum)]
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
/// 优先级：`custom_path` > `-g` 全局（按 agent）> 项目级（按 agent，默认自动检测）
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
    // 一次性解析目标平台；避免 `AgentPlatform::detect()` 在 resolve_target_dir
    // 与 install_hook 分支被重复调用（每次 detect 会扫 5 个平台目录）。
    let platform = args.agent.unwrap_or_else(AgentPlatform::detect);

    let target = resolve_target_dir(args.global, Some(platform), args.custom_path.as_deref())?;
    let source = skills_source_dir();

    let has_source = source.exists();
    let has_bundled = !SKILLS.is_empty();

    if has_source {
        // 从文件系统目录安装（开发场景或 cargo install --path）
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
                    std::fs::remove_dir_all(&dest)
                        .map_err(|e| miette::miette!("无法删除旧版本 {}: {e}", dest.display()))?;
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
        println!("安装完成: 新增 {installed} 个，覆盖 {overwritten} 个，跳过 {skipped} 个");
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
            && parts.first().is_some_and(|f| f.starts_with("gitflow-"))
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
    if dest.exists() {
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

    if args.force && dest.exists() {
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

fn resolve_global_hook_paths(
    home: &std::path::Path,
    platform: AgentPlatform,
) -> (PathBuf, PathBuf, String) {
    let hooks_dir = platform.hooks_dir_name();
    let settings_file = platform.settings_file_path();
    let cmd = format!("bash ~/{hooks_dir}/auto-report-bug.sh");
    (home.join(hooks_dir), home.join(settings_file), cmd)
}

fn resolve_project_hook_paths(
    repo: &std::path::Path,
    platform: AgentPlatform,
) -> (PathBuf, PathBuf, String) {
    let hooks_dir = platform.hooks_dir_name();
    let settings_file = platform.settings_file_path();
    let cmd = format!(
        "bash \"$(git rev-parse --show-toplevel 2>/dev/null || \
         pwd)/{hooks_dir}/auto-report-bug.sh\""
    );
    (repo.join(hooks_dir), repo.join(settings_file), cmd)
}

/// 从文件系统目录安装 skills（开发场景）。
///
/// hook 脚本安装到平台对应的 hooks 目录（Claude 下为 `.claude/hooks/`），
/// 配置写入平台对应的 settings 文件（Claude 下为 `.claude/settings.json`）。
fn install_hook(global: bool, force: bool, platform: AgentPlatform) -> miette::Result<()> {
    let hook_script = include_bytes!("../../../../hooks/auto-report-bug.sh");

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
            "command should reference .claude/hooks/ path"
        );
    }

    #[test]
    fn test_resolve_global_hook_paths_uses_claude_hooks_dir() {
        let home = PathBuf::from("/home/user");
        let (hook_dir, settings_path, cmd) =
            resolve_global_hook_paths(&home, AgentPlatform::Claude);
        assert_eq!(hook_dir, home.join(".claude/hooks"));
        assert_eq!(settings_path, home.join(".claude/settings.json"));
        assert!(cmd.contains("~/.claude/hooks/auto-report-bug.sh"));
    }
}
