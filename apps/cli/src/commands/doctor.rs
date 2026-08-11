//! `gf doctor` — environment diagnostic command.
//!
//! Provides a comprehensive health check covering:
//! - Platform CLI status (`gh`, `glab`, `gc`)
//! - Agent platform and skills
//! - `gf` binary self-check
//! - Agent runtime environment

use std::path::PathBuf;

use clap::Args;
use gitflow_core::doctor::{CheckItem, CheckStatus, DoctorReport, HealthCheck};

use crate::commands::skills::AgentPlatform;

/// Checks all three platform CLIs (`gh`, `glab`, `gc`) for installation, version, and auth.
///
/// Unlike `prerequisites::check()` which fast-fails on the target platform,
/// this check collects results for ALL platforms.
pub struct PlatformCliCheck;

impl HealthCheck for PlatformCliCheck {
    fn category(&self) -> &'static str {
        "platform_cli"
    }

    fn run(&self) -> Vec<CheckItem> {
        let platforms = ["github", "gitlab", "gitcode"];
        let labels = ["GitHub", "GitLab", "GitCode"];
        let mut items = Vec::new();

        for (platform, label) in platforms.iter().zip(labels.iter()) {
            let Some(req) = super::prerequisites::requirement_for(platform) else {
                items.push(CheckItem::fail(
                    self.category(),
                    format!("{label} CLI"),
                    format!("{label} 平台配置缺失"),
                    "请报告此问题",
                ));
                continue;
            };

            // Check binary existence
            let binary_path = if *platform == "gitcode" {
                find_gitcode_binary()
            } else {
                which::which(req.binary)
                    .ok()
                    .map(|p| (p, req.binary.to_string()))
            };

            let Some((_, ref binary_name)) = binary_path else {
                items.push(CheckItem::fail(
                    self.category(),
                    format!("{label} CLI"),
                    format!("{label} {} 未安装", req.binary),
                    format!("安装：{}", req.install_cmd),
                ));
                continue;
            };

            // Check version
            let version = get_cli_version(binary_name);
            let version_str = version.as_deref().unwrap_or("unknown");

            if let Some(ref v) = version
                && !super::prerequisites::version_meets_minimum(v, req.min_version)
            {
                items.push(
                    CheckItem::warn(
                        self.category(),
                        format!("{label} CLI"),
                        format!("{label} {} 版本过低", req.binary),
                        format!("升级：{}", req.install_cmd),
                    )
                    .with_detail(format!("当前 v{v}，需要 v{}", req.min_version)),
                );
                continue;
            }

            // Check auth
            let auth_checker = create_auth_checker_for(platform);
            if auth_checker.is_authenticated() {
                let check_result = auth_checker.check_status();
                let user_info = check_result.user.as_deref().unwrap_or("unknown");
                items.push(
                    CheckItem::pass(
                        self.category(),
                        format!("{label} CLI"),
                        format!("{label} {} 已认证", req.binary),
                    )
                    .with_detail(format!("v{version_str} ({user_info})")),
                );
            } else {
                let check_result = auth_checker.check_status();
                let hint = check_result.hint.unwrap_or(req.login_cmd.to_string());
                items.push(
                    CheckItem::fail(
                        self.category(),
                        format!("{label} CLI"),
                        format!("{label} 未认证"),
                        format!("运行 `{hint}` 完成登录"),
                    )
                    .with_detail(format!("v{version_str}")),
                );
            }
        }

        items
    }
}

/// Find `GitCode` CLI binary (tries `gc` then `gitcode`, including pip paths).
#[allow(
    clippy::disallowed_methods,
    reason = "Sync binary discovery before async runtime"
)]
fn find_gitcode_binary() -> Option<(PathBuf, String)> {
    for binary in &["gc", "gitcode"] {
        if let Ok(path) = which::which(binary) {
            return Some((path, (*binary).to_string()));
        }
    }
    // Check pip user install paths
    if let Ok(home) = std::env::var("HOME") {
        let lib = std::path::PathBuf::from(&home).join("Library/Python");
        if let Ok(entries) = std::fs::read_dir(&lib) {
            for entry in entries.flatten() {
                for binary in &["gc", "gitcode"] {
                    let p = entry.path().join("bin").join(binary);
                    if p.exists() {
                        return Some((p, (*binary).to_string()));
                    }
                }
            }
        }
    }
    None
}

/// Get CLI version string by running `<binary> --version` or `<binary> version`.
#[allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "Sync CLI version probe before async runtime"
)]
fn get_cli_version(binary: &str) -> Option<String> {
    for arg in &["--version", "version"] {
        if let Ok(output) = std::process::Command::new(binary).arg(arg).output()
            && output.status.success()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(v) = super::prerequisites::extract_semver(&stdout) {
                return Some(v);
            }
        }
    }
    None
}

/// Create an auth checker for the given platform.
fn create_auth_checker_for(platform: &str) -> Box<dyn gitflow_core::AuthChecker> {
    match platform {
        "github" => Box::new(gitflow_github::GitHubAuthProvider::new()),
        "gitlab" => Box::new(gitflow_gitlab::GitLabAuthProvider::new()),
        "gitcode" => Box::new(gitflow_gitcode::GitCodeAuthProvider::new()),
        _ => unreachable!("platform already validated"),
    }
}

/// Checks Agent platform detection and skills installation status.
pub struct AgentSkillsCheck;

impl HealthCheck for AgentSkillsCheck {
    fn category(&self) -> &'static str {
        "agent"
    }

    fn run(&self) -> Vec<CheckItem> {
        let mut items = Vec::new();

        // Agent platform detection
        let platform = AgentPlatform::detect();
        items.push(
            CheckItem::pass(
                self.category(),
                "Agent 平台",
                format!("检测到 {platform:?}"),
            )
            .with_detail(format!("{platform:?}")),
        );

        // Skills directory check
        let skills_dir = resolve_skills_dir();
        if skills_dir.is_dir() {
            let count = count_skills(&skills_dir);
            if count > 0 {
                items.push(
                    CheckItem::pass(
                        self.category(),
                        "Skills 安装",
                        format!("{count} 个 skills 已安装"),
                    )
                    .with_detail(skills_dir.display().to_string()),
                );
            } else {
                items.push(CheckItem::warn(
                    self.category(),
                    "Skills 安装",
                    "Skills 目录存在但为空".to_string(),
                    "运行 `gf skills install` 安装 skills".to_string(),
                ));
            }
        } else {
            items.push(CheckItem::warn(
                self.category(),
                "Skills 安装",
                "Skills 目录不存在".to_string(),
                "运行 `gf skills install` 安装 skills".to_string(),
            ));
        }

        items
    }
}

/// Resolve the skills directory for the detected agent platform.
#[allow(
    clippy::disallowed_methods,
    reason = "Sync home directory lookup for skills path resolution"
)]
fn resolve_skills_dir() -> PathBuf {
    if let Some(home) = dirs::home_dir() {
        home.join(".claude").join("skills")
    } else {
        PathBuf::from(".claude").join("skills")
    }
}

/// Count installed `gf-*` skills in the given directory.
#[allow(
    clippy::disallowed_methods,
    reason = "Sync directory listing for environment diagnostics"
)]
fn count_skills(dir: &std::path::Path) -> usize {
    std::fs::read_dir(dir).map_or(0, |entries| {
        entries
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().starts_with("gf-"))
            .count()
    })
}

/// Checks `gf` binary version, path, and update availability.
pub struct GfSelfCheck;

#[allow(
    clippy::disallowed_methods,
    reason = "Sync binary path lookup for self-diagnostics"
)]
impl HealthCheck for GfSelfCheck {
    fn category(&self) -> &'static str {
        "gf_self"
    }

    fn run(&self) -> Vec<CheckItem> {
        let version = env!("CARGO_PKG_VERSION");
        let binary_path = std::env::current_exe()
            .map_or_else(|_| "unknown".to_string(), |p| p.display().to_string());

        vec![
            CheckItem::pass(self.category(), "gf 版本", format!("gf v{version}"))
                .with_detail(binary_path),
        ]
    }
}

/// Checks Agent runtime environment (`.claude/`, `CLAUDE.md`, hooks).
pub struct AgentEnvCheck;

#[allow(
    clippy::disallowed_methods,
    reason = "Sync directory/env inspection for diagnostics"
)]
impl HealthCheck for AgentEnvCheck {
    fn category(&self) -> &'static str {
        "agent_env"
    }

    fn run(&self) -> Vec<CheckItem> {
        let mut items = Vec::new();
        let cwd = std::env::current_dir().unwrap_or_default();

        // .claude/ directory
        let claude_dir = cwd.join(".claude");
        if claude_dir.is_dir() {
            items.push(CheckItem::pass(
                self.category(),
                ".claude/ 目录",
                ".claude/ 目录存在".to_string(),
            ));
        } else {
            items.push(CheckItem::warn(
                self.category(),
                ".claude/ 目录",
                ".claude/ 目录不存在".to_string(),
                "运行 `gf skills install` 初始化 Agent 环境".to_string(),
            ));
        }

        // CLAUDE.md
        let claude_md = cwd.join("CLAUDE.md");
        if claude_md.is_file() {
            items.push(CheckItem::pass(
                self.category(),
                "CLAUDE.md",
                "CLAUDE.md 存在".to_string(),
            ));
        } else {
            items.push(CheckItem::warn(
                self.category(),
                "CLAUDE.md",
                "CLAUDE.md 不存在".to_string(),
                "创建 CLAUDE.md 以配置 Agent 行为".to_string(),
            ));
        }

        // Hooks
        let hooks_dir = claude_dir.join("hooks");
        if hooks_dir.is_dir() {
            let hook_count =
                std::fs::read_dir(&hooks_dir).map_or(0, |entries| entries.flatten().count());
            if hook_count > 0 {
                items.push(CheckItem::pass(
                    self.category(),
                    "Hooks",
                    format!("{hook_count} 个 hooks 已配置"),
                ));
            } else {
                items.push(CheckItem::warn(
                    self.category(),
                    "Hooks",
                    "Hooks 目录存在但为空".to_string(),
                    "运行 `gf skills install` 安装 hooks".to_string(),
                ));
            }
        } else {
            items.push(CheckItem::warn(
                self.category(),
                "Hooks",
                "Hooks 未配置".to_string(),
                "运行 `gf skills install` 安装 auto-report-bug hook".to_string(),
            ));
        }

        items
    }
}

/// `gf doctor` 子命令参数。
#[derive(Debug, Args)]
pub struct DoctorArgs {
    /// 输出格式。
    #[arg(long, default_value = "text")]
    pub format: DoctorFormat,
}

/// Doctor output format.
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum DoctorFormat {
    /// Colored terminal report.
    Text,
    /// Structured JSON.
    Json,
}

/// Handle the `gf doctor` command.
///
/// # Errors
///
/// Returns an error if JSON serialization fails.
pub fn handle(args: &DoctorArgs) -> miette::Result<()> {
    let checks: Vec<Box<dyn HealthCheck>> = vec![
        Box::new(PlatformCliCheck),
        Box::new(AgentSkillsCheck),
        Box::new(GfSelfCheck),
        Box::new(AgentEnvCheck),
    ];

    let mut all_items: Vec<CheckItem> = Vec::new();
    for check in &checks {
        all_items.extend(check.run());
    }

    let report = DoctorReport::from_items(all_items);

    match args.format {
        DoctorFormat::Json => {
            let json = serde_json::to_string_pretty(&report)
                .map_err(|e| miette::miette!("JSON serialization failed: {e}"))?;
            println!("{json}");
        }
        DoctorFormat::Text => {
            print_text_report(&report);
        }
    }

    // Exit code: 0 = all pass, 1 = any fail, 2 = any warn
    if report.summary.failed > 0 {
        std::process::exit(1);
    }
    if report.summary.warned > 0 {
        std::process::exit(2);
    }
    Ok(())
}

/// Print a colored terminal report.
fn print_text_report(report: &DoctorReport) {
    println!();
    println!("🩺 gitflow-cli 环境诊断");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let category_labels = [
        ("platform_cli", "📦 平台 CLI"),
        ("agent", "🤖 Agent + Skills"),
        ("gf_self", "🔧 gf 自身"),
        ("agent_env", "🏠 Agent 运行环境"),
    ];

    for (cat_key, cat_label) in &category_labels {
        let cat_items: Vec<&CheckItem> = report
            .items
            .iter()
            .filter(|i| i.category == *cat_key)
            .collect();
        if cat_items.is_empty() {
            continue;
        }
        println!();
        println!("{cat_label}");
        for item in &cat_items {
            let icon = match item.status {
                CheckStatus::Pass => "✅",
                CheckStatus::Warn => "⚠️",
                CheckStatus::Fail => "❌",
            };
            let detail_str = item
                .detail
                .as_deref()
                .map(|d| format!(" ({d})"))
                .unwrap_or_default();
            println!("  {icon} {} — {}{detail_str}", item.name, item.message);
            if let Some(ref hint) = item.hint {
                println!("     🔧 修复：{hint}");
            }
        }
    }

    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!(
        "总计: {} 项 · ✅ {} 通过 · ⚠️ {} 警告 · ❌ {} 失败",
        report.summary.total, report.summary.passed, report.summary.warned, report.summary.failed,
    );
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "Tests legitimately need to unwrap fixture data"
)]
mod tests {
    use gitflow_core::doctor::CheckStatus;

    use super::*;

    #[test]
    fn test_should_return_platform_cli_category() {
        let check = PlatformCliCheck;
        assert_eq!(check.category(), "platform_cli");
    }

    #[test]
    fn test_should_collect_results_for_all_three_platforms() {
        let check = PlatformCliCheck;
        let items = check.run();
        assert_eq!(
            items.len(),
            3,
            "Expected 3 items (one per platform), got {}",
            items.len()
        );
    }

    #[test]
    fn test_should_not_fast_fail_on_missing_cli() {
        let check = PlatformCliCheck;
        let items = check.run();
        assert_eq!(items.len(), 3);
        for item in &items {
            assert!(
                matches!(
                    item.status,
                    CheckStatus::Pass | CheckStatus::Warn | CheckStatus::Fail
                ),
                "Invalid status for {}: {:?}",
                item.name,
                item.status
            );
        }
    }

    #[test]
    fn test_should_include_install_hint_on_failure() {
        let check = PlatformCliCheck;
        let items = check.run();
        for item in &items {
            if item.status == CheckStatus::Fail && item.message.contains("未安装") {
                assert!(
                    item.hint.is_some(),
                    "Missing install hint for failed check: {}",
                    item.name
                );
            }
        }
    }

    #[test]
    fn test_should_get_cli_version_for_installed_binary() {
        let version = get_cli_version("git");
        assert!(version.is_some(), "git version should be detectable");
    }

    #[test]
    fn test_should_return_none_for_nonexistent_binary_version() {
        let version = get_cli_version("nonexistent-binary-xyz-12345");
        assert!(version.is_none());
    }

    #[test]
    fn test_should_create_agent_skills_check() {
        let check = AgentSkillsCheck;
        assert_eq!(check.category(), "agent");
        let items = check.run();
        assert!(
            !items.is_empty(),
            "AgentSkillsCheck should produce at least 1 item"
        );
    }

    #[test]
    fn test_should_create_gf_self_check() {
        let check = GfSelfCheck;
        assert_eq!(check.category(), "gf_self");
        let items = check.run();
        assert_eq!(items.len(), 1, "GfSelfCheck should produce exactly 1 item");
        assert_eq!(items[0].status, CheckStatus::Pass);
    }

    #[test]
    fn test_should_create_agent_env_check() {
        let check = AgentEnvCheck;
        assert_eq!(check.category(), "agent_env");
        let items = check.run();
        assert!(
            items.len() >= 2,
            "AgentEnvCheck should produce at least 2 items (.claude/ + CLAUDE.md)"
        );
    }

    #[test]
    fn test_should_collect_all_categories_in_report() {
        let checks: Vec<Box<dyn HealthCheck>> = vec![
            Box::new(PlatformCliCheck),
            Box::new(AgentSkillsCheck),
            Box::new(GfSelfCheck),
            Box::new(AgentEnvCheck),
        ];
        let mut all_items = Vec::new();
        for check in &checks {
            all_items.extend(check.run());
        }
        let report = DoctorReport::from_items(all_items);
        let categories: std::collections::HashSet<&str> =
            report.items.iter().map(|i| i.category.as_str()).collect();
        assert!(categories.contains("platform_cli"));
        assert!(categories.contains("agent"));
        assert!(categories.contains("gf_self"));
        assert!(categories.contains("agent_env"));
    }
}
