# gf doctor + Error Audit + Issue Ops Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `gf doctor` diagnostic command, audit error message gaps, and set up Issue operations infrastructure to eliminate new-user onboarding friction.

**Architecture:** Trait-based health check system (`HealthCheck` in `crates/core`, 4 category implementations in `apps/cli`). Reuses existing `prerequisites.rs` logic for CLI checks, `AgentPlatform::detect()` for agent detection, and skills directory scanning. Error audit fills gaps in existing `parse_*_error` functions. Issue ops adds GitHub templates and SLA to CONTRIBUTING.md.

**Tech Stack:** Rust 2024, serde (JSON), clap (CLI args), miette (error reporting), which (binary discovery), GitHub YAML issue templates

## Global Constraints

- Rust 2024 edition, toolchain pinned in `rust-toolchain.toml`
- `#![forbid(unsafe_code)]` at crate roots
- All public items require documentation (`missing_docs` lint)
- `tracing` for logging, never `println!` / `dbg!` in production code
- Use `thiserror` for library errors, `miette` for CLI error reporting
- No `unwrap()` or `expect()` in production code
- Follow existing patterns: `AuthProvider`/`AuthChecker` trait style, `CliOutput`/`print_output()` for output formatting
- All test names: `test_should_<expected_behavior>`
- Chinese-first user-facing messages; English for code identifiers

---

### Task 1: Core Data Model + HealthCheck Trait

**Files:**
- Create: `crates/core/src/doctor.rs`
- Modify: `crates/core/src/lib.rs` (add `pub mod doctor;` + re-exports)

**Interfaces:**
- Produces: `HealthCheck` trait, `CheckStatus` enum, `CheckItem` struct, `DoctorReport` struct, `DoctorSummary` struct — all exported at crate root

- [ ] **Step 1: Write failing tests for data model**

Create `crates/core/src/doctor.rs` with the test module first:

```rust
//! Environment diagnostic types and traits for `gf doctor`.

use serde::Serialize;

/// Status of a single health check item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    /// Check passed.
    Pass,
    /// Check passed with warnings.
    Warn,
    /// Check failed.
    Fail,
}

/// A single diagnostic check result.
#[derive(Debug, Clone, Serialize)]
pub struct CheckItem {
    /// Check category (e.g., "platform_cli", "agent", "gf_self", "agent_env").
    pub category: String,
    /// Check item name (e.g., "gh CLI installed").
    pub name: String,
    /// Result status.
    pub status: CheckStatus,
    /// Human-readable description (Chinese-first).
    pub message: String,
    /// Fix suggestion (provided on Fail/Warn).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    /// Additional detail (e.g., version string, path).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Summary counts for a doctor report.
#[derive(Debug, Clone, Serialize)]
pub struct DoctorSummary {
    /// Total number of checks.
    pub total: usize,
    /// Number of passing checks.
    pub passed: usize,
    /// Number of warnings.
    pub warned: usize,
    /// Number of failures.
    pub failed: usize,
}

impl DoctorSummary {
    /// Compute summary from a slice of check items.
    #[must_use]
    pub fn from_items(items: &[CheckItem]) -> Self {
        let total = items.len();
        let passed = items.iter().filter(|i| i.status == CheckStatus::Pass).count();
        let warned = items.iter().filter(|i| i.status == CheckStatus::Warn).count();
        let failed = items.iter().filter(|i| i.status == CheckStatus::Fail).count();
        Self { total, passed, warned, failed }
    }
}

/// Complete diagnostic report.
#[derive(Debug, Clone, Serialize)]
pub struct DoctorReport {
    /// All check items.
    pub items: Vec<CheckItem>,
    /// Summary counts.
    pub summary: DoctorSummary,
    /// ISO 8601 timestamp.
    pub timestamp: String,
}

impl DoctorReport {
    /// Create a report from check items, computing summary and timestamp.
    #[must_use]
    pub fn from_items(items: Vec<CheckItem>) -> Self {
        let summary = DoctorSummary::from_items(&items);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| format!("{}s", d.as_secs()))
            .unwrap_or_else(|_| "unknown".to_string());
        Self { items, summary, timestamp }
    }
}

/// Trait for health check categories.
///
/// Each category implements this trait to provide a group of related checks.
/// The `gf doctor` command iterates over all registered categories and collects results.
pub trait HealthCheck: Send + Sync {
    /// Category name for grouping (e.g., "platform_cli").
    fn category(&self) -> &str;

    /// Run all checks in this category, returning results.
    /// Must not fail fast — collect all results even if some checks fail.
    fn run(&self) -> Vec<CheckItem>;
}

impl CheckItem {
    /// Create a passing check item.
    #[must_use]
    pub fn pass(category: impl Into<String>, name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            category: category.into(),
            name: name.into(),
            status: CheckStatus::Pass,
            message: message.into(),
            hint: None,
            detail: None,
        }
    }

    /// Create a warning check item.
    #[must_use]
    pub fn warn(category: impl Into<String>, name: impl Into<String>, message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self {
            category: category.into(),
            name: name.into(),
            status: CheckStatus::Warn,
            message: message.into(),
            hint: Some(hint.into()),
            detail: None,
        }
    }

    /// Create a failing check item.
    #[must_use]
    pub fn fail(category: impl Into<String>, name: impl Into<String>, message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self {
            category: category.into(),
            name: name.into(),
            status: CheckStatus::Fail,
            message: message.into(),
            hint: Some(hint.into()),
            detail: None,
        }
    }

    /// Attach a detail string to this check item (builder pattern).
    #[must_use]
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_calculate_summary_counts() {
        let items = vec![
            CheckItem::pass("cat", "a", "ok"),
            CheckItem::pass("cat", "b", "ok"),
            CheckItem::warn("cat", "c", "meh", "fix it"),
            CheckItem::fail("cat", "d", "bad", "fix now"),
        ];
        let summary = DoctorSummary::from_items(&items);
        assert_eq!(summary.total, 4);
        assert_eq!(summary.passed, 2);
        assert_eq!(summary.warned, 1);
        assert_eq!(summary.failed, 1);
    }

    #[test]
    fn test_should_create_report_from_items() {
        let items = vec![
            CheckItem::pass("cat", "a", "ok"),
            CheckItem::fail("cat", "b", "bad", "fix"),
        ];
        let report = DoctorReport::from_items(items);
        assert_eq!(report.summary.total, 2);
        assert_eq!(report.summary.passed, 1);
        assert_eq!(report.summary.failed, 1);
        assert!(!report.timestamp.is_empty());
    }

    #[test]
    fn test_should_serialize_report_to_json() {
        let items = vec![
            CheckItem::pass("platform_cli", "gh installed", "gh found")
                .with_detail("v2.65.0"),
            CheckItem::fail("platform_cli", "gc auth", "not authenticated", "run gc auth login"),
        ];
        let report = DoctorReport::from_items(items);
        let json = serde_json::to_string(&report).expect("serialize");
        assert!(json.contains("\"status\":\"pass\""));
        assert!(json.contains("\"status\":\"fail\""));
        assert!(json.contains("\"detail\":\"v2.65.0\""));
        // hint is present on fail item
        assert!(json.contains("gc auth login"));
    }

    #[test]
    fn test_should_skip_none_fields_in_json() {
        let item = CheckItem::pass("cat", "name", "msg");
        let json = serde_json::to_string(&item).expect("serialize");
        assert!(!json.contains("\"hint\""));
        assert!(!json.contains("\"detail\""));
    }

    #[test]
    fn test_should_create_pass_item() {
        let item = CheckItem::pass("cat", "name", "message");
        assert_eq!(item.status, CheckStatus::Pass);
        assert!(item.hint.is_none());
    }

    #[test]
    fn test_should_create_warn_item_with_hint() {
        let item = CheckItem::warn("cat", "name", "message", "hint text");
        assert_eq!(item.status, CheckStatus::Warn);
        assert_eq!(item.hint.as_deref(), Some("hint text"));
    }

    #[test]
    fn test_should_create_fail_item_with_hint() {
        let item = CheckItem::fail("cat", "name", "message", "fix this");
        assert_eq!(item.status, CheckStatus::Fail);
        assert_eq!(item.hint.as_deref(), Some("fix this"));
    }

    #[test]
    fn test_should_attach_detail_via_builder() {
        let item = CheckItem::pass("cat", "name", "msg").with_detail("v1.0.0");
        assert_eq!(item.detail.as_deref(), Some("v1.0.0"));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p gitflow-core doctor::`
Expected: FAIL — module not declared in lib.rs

- [ ] **Step 3: Register module in lib.rs**

Add to `crates/core/src/lib.rs` after `pub mod cleanup;` (alphabetical order):

```rust
pub mod doctor;
```

Add re-export after `pub use cli_error::PlatformCliError;`:

```rust
pub use doctor::{CheckItem, CheckStatus, DoctorReport, DoctorSummary, HealthCheck};
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p gitflow-core doctor::`
Expected: All 8 tests PASS

- [ ] **Step 5: Commit**

```bash
git add crates/core/src/doctor.rs crates/core/src/lib.rs
git commit -m "feat(doctor): add HealthCheck trait and data model in core"
```

---

### Task 2: Platform CLI Check Category

**Files:**
- Create: `apps/cli/src/commands/doctor.rs`
- Modify: `apps/cli/src/commands/mod.rs` (add `pub mod doctor;`)

**Interfaces:**
- Consumes: `HealthCheck`, `CheckItem`, `CheckStatus` from `gitflow_core::doctor`
- Consumes: `requirement_for()`, `CliRequirement` from `commands::prerequisites`
- Consumes: `AuthChecker` trait from `gitflow_core`
- Produces: `PlatformCliCheck` struct implementing `HealthCheck`

- [ ] **Step 1: Write failing tests for PlatformCliCheck**

Create `apps/cli/src/commands/doctor.rs`. Since `PlatformCliCheck` needs to call `which` and spawn processes for version checks, make the binary lookup injectable for testing. Start with tests:

```rust
//! `gf doctor` — environment diagnostic command.
//!
//! Provides a comprehensive health check covering:
//! - Platform CLI status (gh, glab, gc)
//! - Agent platform and skills
//! - gf binary self-check
//! - Agent runtime environment

use gitflow_core::doctor::{CheckItem, CheckStatus, HealthCheck};

/// Checks all three platform CLIs (gh, glab, gc) for installation, version, and auth.
///
/// Unlike `prerequisites::check()` which fast-fails on the target platform,
/// this check collects results for ALL platforms.
pub struct PlatformCliCheck;

impl HealthCheck for PlatformCliCheck {
    fn category(&self) -> &str {
        "platform_cli"
    }

    fn run(&self) -> Vec<CheckItem> {
        let platforms = ["github", "gitlab", "gitcode"];
        let labels = ["GitHub", "GitLab", "GitCode"];
        let mut items = Vec::new();

        for (platform, label) in platforms.iter().zip(labels.iter()) {
            let req = match super::prerequisites::requirement_for(platform) {
                Some(r) => r,
                None => {
                    items.push(CheckItem::fail(
                        self.category(),
                        format!("{label} CLI"),
                        format!("{label} 平台配置缺失"),
                        "请报告此问题",
                    ));
                    continue;
                }
            };

            // Check binary existence
            let binary_path = if *platform == "gitcode" {
                find_gitcode_binary()
            } else {
                which::which(req.binary).ok().map(|p| (p, req.binary.to_string()))
            };

            let Some((path, binary_name)) = binary_path else {
                items.push(CheckItem::fail(
                    self.category(),
                    format!("{label} CLI"),
                    format!("{label} 未安装 {binary}").replace("{binary}", req.binary),
                    format!("安装：{}", req.install_cmd),
                ));
                continue;
            };

            // Check version
            let version = get_cli_version(&binary_name);
            let version_str = version.as_deref().unwrap_or("unknown");

            if let Some(ref v) = version {
                if !super::prerequisites::version_meets_minimum(v, req.min_version) {
                    items.push(
                        CheckItem::warn(
                            self.category(),
                            format!("{label} CLI"),
                            format!("{label} {binary} 版本过低").replace("{binary}", req.binary),
                            format!("升级：{}", req.install_cmd),
                        )
                        .with_detail(format!("当前 v{v}，需要 v{}", req.min_version)),
                    );
                    continue;
                }
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
                        format!("{label} {binary} 已认证").replace("{binary}", req.binary),
                    )
                    .with_detail(format!("v{version_str} ({user_info})")),
                );
            } else {
                let check_result = auth_checker.check_status();
                let hint = check_result.hint.unwrap_or_else(|| req.login_cmd.to_string());
                items.push(
                    CheckItem::fail(
                        self.category(),
                        format!("{label} CLI"),
                        format!("{label} 未认证").replace("{binary}", req.binary),
                        format!("运行 `{hint}` 完成登录"),
                    )
                    .with_detail(format!("v{version_str}")),
                );
            }
        }

        items
    }
}

/// Find GitCode CLI binary (tries `gc` then `gitcode`, including pip paths).
fn find_gitcode_binary() -> Option<(std::path::PathBuf, String)> {
    for binary in &["gc", "gitcode"] {
        if let Ok(path) = which::which(binary) {
            return Some((path, binary.to_string()));
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
                        return Some((p, binary.to_string()));
                    }
                }
            }
        }
    }
    None
}

/// Get CLI version string by running `<binary> --version` or `<binary> version`.
fn get_cli_version(binary: &str) -> Option<String> {
    for arg in &["--version", "version"] {
        if let Ok(output) = std::process::Command::new(binary).arg(arg).output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(v) = super::prerequisites::extract_semver(&stdout) {
                    return Some(v);
                }
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

#[cfg(test)]
mod tests {
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
        // Should have exactly 3 items (one per platform), regardless of pass/fail
        assert_eq!(items.len(), 3, "Expected 3 items (one per platform), got {}", items.len());
    }

    #[test]
    fn test_should_not_fast_fail_on_missing_cli() {
        // Even if a CLI is missing, we should get results for all platforms
        let check = PlatformCliCheck;
        let items = check.run();
        assert_eq!(items.len(), 3);
        // Each item should have a valid status
        for item in &items {
            assert!(
                matches!(item.status, CheckStatus::Pass | CheckStatus::Warn | CheckStatus::Fail),
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
        // Test with a binary we know exists on dev machines
        let version = get_cli_version("git");
        // git should be available in dev environment
        assert!(version.is_some(), "git version should be detectable");
    }

    #[test]
    fn test_should_return_none_for_nonexistent_binary_version() {
        let version = get_cli_version("nonexistent-binary-xyz-12345");
        assert!(version.is_none());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p gitflow-cli doctor::`
Expected: FAIL — module not declared in mod.rs

- [ ] **Step 3: Register module in mod.rs**

Add to `apps/cli/src/commands/mod.rs` after `pub mod completions;`:

```rust
pub mod doctor;
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p gitflow-cli doctor::`
Expected: All tests PASS

- [ ] **Step 5: Run clippy**

Run: `cargo clippy -p gitflow-cli -- -D warnings`
Expected: No warnings

- [ ] **Step 6: Commit**

```bash
git add apps/cli/src/commands/doctor.rs apps/cli/src/commands/mod.rs
git commit -m "feat(doctor): add PlatformCliCheck category"
```

---

### Task 3: Remaining Check Categories + CLI Handler

**Files:**
- Modify: `apps/cli/src/commands/doctor.rs` (add 3 more check structs + `handle()` function)

**Interfaces:**
- Consumes: `HealthCheck`, `CheckItem`, `DoctorReport` from core
- Consumes: `AgentPlatform` from `commands::skills`
- Consumes: `OutputFormat`, `CliOutput`, `print_output()` from CLI
- Produces: `handle()` function for CLI routing

- [ ] **Step 1: Write AgentSkillsCheck**

Add to `apps/cli/src/commands/doctor.rs`:

```rust
use crate::commands::skills::AgentPlatform;

/// Checks Agent platform detection and skills installation status.
pub struct AgentSkillsCheck;

impl HealthCheck for AgentSkillsCheck {
    fn category(&self) -> &str {
        "agent"
    }

    fn run(&self) -> Vec<CheckItem> {
        let mut items = Vec::new();

        // Agent platform detection
        let platform = AgentPlatform::detect();
        items.push(
            CheckItem::pass(self.category(), "Agent 平台", format!("检测到 {platform:?}"))
                .with_detail(format!("{platform:?}")),
        );

        // Skills directory check
        let skills_dir = resolve_skills_dir(&platform);
        if skills_dir.exists() {
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

/// Resolve the skills directory for the given agent platform.
fn resolve_skills_dir(platform: &AgentPlatform) -> std::path::PathBuf {
    let dir_name = platform.skills_dir_name();
    if let Some(home) = dirs::home_dir() {
        home.join(".claude").join("skills")
    } else {
        std::path::PathBuf::from(".claude").join("skills")
    }
}

/// Count installed gf-* skills in the given directory.
fn count_skills(dir: &std::path::Path) -> usize {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| e.file_name().to_string_lossy().starts_with("gf-"))
                .count()
        })
        .unwrap_or(0)
}
```

- [ ] **Step 2: Write GfSelfCheck**

Add to `apps/cli/src/commands/doctor.rs`:

```rust
/// Checks gf binary version, path, and update availability.
pub struct GfSelfCheck;

impl HealthCheck for GfSelfCheck {
    fn category(&self) -> &str {
        "gf_self"
    }

    fn run(&self) -> Vec<CheckItem> {
        let mut items = Vec::new();

        // gf version
        let version = env!("CARGO_PKG_VERSION");
        let binary_path = std::env::current_exe()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        items.push(
            CheckItem::pass(self.category(), "gf 版本", format!("gf v{version}"))
                .with_detail(binary_path),
        );

        items
    }
}
```

- [ ] **Step 3: Write AgentEnvCheck**

Add to `apps/cli/src/commands/doctor.rs`:

```rust
/// Checks Agent runtime environment (.claude/, CLAUDE.md, hooks).
pub struct AgentEnvCheck;

impl HealthCheck for AgentEnvCheck {
    fn category(&self) -> &str {
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
            let hook_count = std::fs::read_dir(&hooks_dir)
                .map(|entries| entries.flatten().count())
                .unwrap_or(0);
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
```

- [ ] **Step 4: Write the handle() function + CLI args**

Add to `apps/cli/src/commands/doctor.rs`:

```rust
use clap::Args;
use crate::commands::output::{OutputFormat, print_output};
use gitflow_core::doctor::{CheckItem, DoctorReport, HealthCheck};

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
        let cat_items: Vec<&CheckItem> = report.items.iter().filter(|i| i.category == *cat_key).collect();
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
            let detail_str = item.detail.as_deref().map(|d| format!(" ({d})")).unwrap_or_default();
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
        report.summary.total,
        report.summary.passed,
        report.summary.warned,
        report.summary.failed,
    );
}
```

- [ ] **Step 5: Write tests for handle() and report formatting**

Add tests to the test module in `doctor.rs`:

```rust
    #[test]
    fn test_should_create_agent_skills_check() {
        let check = AgentSkillsCheck;
        assert_eq!(check.category(), "agent");
        let items = check.run();
        assert!(!items.is_empty(), "AgentSkillsCheck should produce at least 1 item");
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
        assert!(items.len() >= 2, "AgentEnvCheck should produce at least 2 items (.claude/ + CLAUDE.md)");
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
        // Should have items from all 4 categories
        let categories: std::collections::HashSet<&str> = report.items.iter().map(|i| i.category.as_str()).collect();
        assert!(categories.contains("platform_cli"));
        assert!(categories.contains("agent"));
        assert!(categories.contains("gf_self"));
        assert!(categories.contains("agent_env"));
    }
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test -p gitflow-cli doctor::`
Expected: All tests PASS

- [ ] **Step 7: Run clippy**

Run: `cargo clippy -p gitflow-cli -- -D warnings`
Expected: No warnings

- [ ] **Step 8: Commit**

```bash
git add apps/cli/src/commands/doctor.rs
git commit -m "feat(doctor): add remaining check categories and CLI handler"
```

---

### Task 4: Register `gf doctor` in CLI Router

**Files:**
- Modify: `apps/cli/src/main.rs` (add `Doctor` variant to `Commands` enum + route in `router()` + skip prerequisites)

**Interfaces:**
- Consumes: `commands::doctor::{DoctorArgs, handle}`
- Consumes: `Commands` enum

- [ ] **Step 1: Add Doctor variant to Commands enum**

In `apps/cli/src/main.rs`, add to the `Commands` enum (after `Workflow`):

```rust
    /// Diagnose environment health (CLI, Agent, skills, config).
    Doctor(commands::doctor::DoctorArgs),
```

- [ ] **Step 2: Add Doctor to the command name match**

In the `command_name` method (around line 455), add:

```rust
            Commands::Doctor(_) => "doctor",
```

- [ ] **Step 3: Skip prerequisites for Doctor**

In `async_main()`, add `Commands::Doctor(_)` to the skip list:

```rust
    if !matches!(
        cli.command,
        Commands::Skills(_)
            | Commands::Completions(_)
            | Commands::Workflow(_)
            | Commands::Update(_)
            | Commands::Doctor(_)
    ) {
```

- [ ] **Step 4: Route Doctor in router()**

In `router()`, add:

```rust
        Commands::Doctor(ref args) => commands::doctor::handle(args),
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p gitflow-cli`
Expected: All tests PASS

- [ ] **Step 6: Manual smoke test**

Run: `cargo run -- doctor`
Expected: Terminal report with all 4 categories

Run: `cargo run -- doctor --format json`
Expected: JSON report

- [ ] **Step 7: Commit**

```bash
git add apps/cli/src/main.rs
git commit -m "feat(doctor): register gf doctor command in CLI router"
```

---

### Task 5: Error Message Audit — GitCode PR + Extended Codes

**Files:**
- Modify: `crates/gitcode/src/error.rs` (add PR-specific codes + more mappings)
- Modify: `crates/github/src/error.rs` (add more error code mappings)
- Modify: `crates/gitlab/src/error.rs` (add more error code mappings)

**Interfaces:**
- Consumes: `PlatformCliError` from `gitflow_core`
- Produces: Enhanced `parse_gitcode_error`, `parse_gh_error`, `parse_glab_error` with more code mappings

- [ ] **Step 1: Write failing tests for new error codes in GitCode**

Add tests to `crates/gitcode/src/error.rs`:

```rust
    #[test]
    fn test_should_parse_gitcode_pr_disabled_error() {
        let json = br#"{"message": "Pull requests are disabled for this repository", "code": "PR_DISABLED"}"#;
        let err = parse_gitcode_error(json);
        assert_eq!(err.code.as_deref(), Some("PR_DISABLED"));
        assert!(err.user_message.contains("PR") || err.user_message.contains("拉取请求"));
        assert!(err.hint.is_some());
    }

    #[test]
    fn test_should_parse_gitcode_branch_protected_error() {
        let json = br#"{"message": "Branch is protected", "code": "BRANCH_PROTECTED"}"#;
        let err = parse_gitcode_error(json);
        assert_eq!(err.code.as_deref(), Some("BRANCH_PROTECTED"));
        assert!(err.hint.is_some());
    }

    #[test]
    fn test_should_parse_gitcode_rate_limited_error() {
        let json = br#"{"message": "API rate limit exceeded", "code": "RATE_LIMITED"}"#;
        let err = parse_gitcode_error(json);
        assert_eq!(err.code.as_deref(), Some("RATE_LIMITED"));
        assert!(err.hint.as_ref().is_some_and(|h| h.contains("稍后") || h.contains("retry")));
    }

    #[test]
    fn test_should_parse_gitcode_validation_error() {
        let json = br#"{"message": "Validation failed", "code": "VALIDATION_FAILED"}"#;
        let err = parse_gitcode_error(json);
        assert_eq!(err.code.as_deref(), Some("VALIDATION_FAILED"));
        assert!(err.hint.is_some());
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p gitflow-gitcode error::`
Expected: FAIL — new codes not mapped yet

- [ ] **Step 3: Implement new error code mappings in GitCode**

In `crates/gitcode/src/error.rs`, extend the `match code.as_deref()` in `parse_gitcode_error`:

```rust
        let user_message: String = match code.as_deref() {
            Some("UNAUTHORIZED" | "FORBIDDEN") => "认证失败或权限不足".into(),
            Some("NOT_FOUND") => "资源不存在".into(),
            Some("PR_DISABLED") => "此仓库未启用拉取请求".into(),
            Some("BRANCH_PROTECTED") => "目标分支受保护，无法直接推送".into(),
            Some("RATE_LIMITED") => "API 请求频率超限".into(),
            Some("VALIDATION_FAILED") => "请求参数校验失败".into(),
            Some("CONFLICT") => "存在冲突，请先合并最新变更".into(),
            _ => format!("GitCode 操作失败：{msg}"),
        };
```

Also refine the hint logic after the match:

```rust
        let hint = match code.as_deref() {
            Some("PR_DISABLED") => Some("在仓库设置中启用拉取请求功能".into()),
            Some("BRANCH_PROTECTED") => Some("检查分支保护规则，或使用有权限的分支".into()),
            Some("RATE_LIMITED") => Some("等待几分钟后重试".into()),
            Some("VALIDATION_FAILED") => Some("检查请求参数格式是否正确".into()),
            Some("CONFLICT") => Some("运行 `git pull --rebase` 解决冲突后重试".into()),
            _ => Some("运行 `gc auth status` 检查认证状态".into()),
        };
        let mut err = PlatformCliError::new(user_message, text.into_owned(), Platform::GitCode);
        err.hint = hint;
```

- [ ] **Step 4: Add similar extended codes to GitHub and GitLab**

In `crates/github/src/error.rs`, extend the match in `parse_gh_error`:

```rust
        let user_message: String = match code.as_deref() {
            Some("NOT_FOUND") => "资源不存在".into(),
            Some("FORBIDDEN") => "权限不足".into(),
            Some("RATE_LIMITED") => "API 请求频率超限".into(),
            Some("VALIDATION_FAILED") => "请求参数校验失败".into(),
            Some("CONFLICT") => "存在冲突，请先合并最新变更".into(),
            Some("GONE") => "资源已被删除或迁移".into(),
            _ => format!("GitHub 操作失败：{msg}"),
        };
```

Add corresponding tests. Similarly for `crates/gitlab/src/error.rs` with `parse_glab_error`.

- [ ] **Step 5: Run all error tests**

Run: `cargo test -p gitflow-gitcode error:: && cargo test -p gitflow-github error:: && cargo test -p gitflow-gitlab error::`
Expected: All tests PASS

- [ ] **Step 6: Commit**

```bash
git add crates/gitcode/src/error.rs crates/github/src/error.rs crates/gitlab/src/error.rs
git commit -m "feat(error): extend error code mappings with PR, rate-limit, validation codes"
```

---

### Task 6: Issue Templates

**Files:**
- Create: `.github/ISSUE_TEMPLATE/bug_report.yml`
- Create: `.github/ISSUE_TEMPLATE/feature_request.yml`

- [ ] **Step 1: Create bug report template**

Create `.github/ISSUE_TEMPLATE/bug_report.yml`:

```yaml
name: "\U0001F41B Bug 报告"
description: 报告一个缺陷，帮助我们改进
title: "[Bug]: "
labels: ["bug", "triage:needed"]
body:
  - type: markdown
    attributes:
      value: |
        感谢报告 Bug！请填写以下信息帮助我们复现问题。
  - type: textarea
    id: description
    attributes:
      label: 问题描述
      description: 清晰简洁地描述问题
    validations:
      required: true
  - type: textarea
    id: reproduction
    attributes:
      label: 复现步骤
      description: 提供复现问题的步骤
      placeholder: |
        1. 执行命令 `gf ...`
        2. 看到错误 `...`
        3. 预期行为是 `...`
    validations:
      required: true
  - type: textarea
    id: environment
    attributes:
      label: 环境信息
      description: 运行 `gf doctor` 并粘贴输出
      placeholder: |
        - gf 版本:
        - 平台: GitHub / GitLab / GitCode
        - 操作系统:
        - 底层 CLI 版本:
    validations:
      required: true
  - type: textarea
    id: logs
    attributes:
      label: 错误日志
      description: 粘贴完整的错误输出
      render: shell
  - type: checkboxes
    id: checklist
    attributes:
      label: 确认清单
      options:
        - label: 我已运行 `gf doctor` 并确认环境配置
        - label: 我已搜索现有 Issues 确认无重复
```

- [ ] **Step 2: Create feature request template**

Create `.github/ISSUE_TEMPLATE/feature_request.yml`:

```yaml
name: "✨ 功能建议"
description: 提议新功能或改进
title: "[Feature]: "
labels: ["enhancement", "triage:needed"]
body:
  - type: markdown
    attributes:
      value: |
        感谢提出功能建议！请描述你的需求和期望。
  - type: textarea
    id: problem
    attributes:
      label: 问题背景
      description: 这个功能要解决什么问题？
      placeholder: 当我执行 ... 时，经常遇到 ... 的问题
    validations:
      required: true
  - type: textarea
    id: solution
    attributes:
      label: 期望方案
      description: 描述你期望的解决方案
    validations:
      required: true
  - type: textarea
    id: alternatives
    attributes:
      label: 替代方案
      description: 你考虑过的其他方案？
  - type: dropdown
    id: platform
    attributes:
      label: 涉及平台
      multiple: true
      options:
        - GitHub
        - GitLab
        - GitCode
        - 全平台
        - 平台无关
  - type: checkboxes
    id: willing
    attributes:
      label: 参与意愿
      options:
        - label: 我愿意尝试实现这个功能
        - label: 我可以提供测试反馈
```

- [ ] **Step 3: Commit**

```bash
git add .github/ISSUE_TEMPLATE/
git commit -m "chore: add GitHub issue templates (bug report + feature request)"
```

---

### Task 7: CONTRIBUTING.md SLA Section

**Files:**
- Modify: `CONTRIBUTING.md` (append SLA section)

- [ ] **Step 1: Append SLA section to CONTRIBUTING.md**

Add the following section at the end of `CONTRIBUTING.md`:

```markdown

## Issue 运营承诺

### 分流时效

- **48 小时内**完成新 Issue 初审（分类、标签、优先级评估）
- 紧急 Bug（影响核心功能）：**24 小时内**响应

### 标签体系

| 标签 | 含义 |
|------|------|
| `triage:needed` | 新提交，待分类 |
| `triage:done` | 已完成分类 |
| `priority:high` | 高优先级 |
| `priority:medium` | 中优先级 |
| `priority:low` | 低优先级 |
| `good first issue` | 适合新手贡献者 |

### 贡献指南

欢迎外部贡献！请选择标记为 `good first issue` 的任务作为起点。
开发环境搭建请参照上方「安装开发工具」章节。
```

- [ ] **Step 2: Commit**

```bash
git add CONTRIBUTING.md
git commit -m "docs: add Issue SLA commitment to CONTRIBUTING.md"
```

---

### Task 8: Good First Issue Labels

**Files:** None (GitHub label operations via `gf`)

- [ ] **Step 1: List open issues and identify candidates**

Run: `gf issue list --state open --limit 30`

Select 5 issues matching criteria:
- Small scope (single file or single module)
- No complex architecture decisions
- Clear acceptance criteria
- Not already labeled

- [ ] **Step 2: Apply `good first issue` label**

For each selected issue:

```bash
gf issue edit <number> --add-label "good first issue"
```

- [ ] **Step 3: Commit (no code changes, but verify CI passes)**

Run: `cargo test && cargo clippy -- -D warnings`
Expected: All pass
