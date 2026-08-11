# gf doctor + 错误信息改造 + Issue 运营 — Design Spec

**Issue**: #100 · **Date**: 2026-08-11 · **Status**: Draft

## 1. Background & Goals

Issue #100 aims to eliminate friction for new users (identified as the core weakness in the end-user perspective evaluation). The original Issue listed three task lines:

1. `gitflow doctor` — one-stop environment self-check
2. Error message overhaul — unified hints + doc links
3. Issue ops infrastructure — templates + SLA + good first issue labels

### Scope Refinement (from brainstorming)

Project exploration revealed that **much of the error handling infrastructure already exists**:

| Assumed missing | Actually exists |
|-----------------|-----------------|
| CLI existence + version + auth check | `prerequisites.rs` — runs before every command |
| Unified error type with hint + doc_link | `PlatformCliError` + `parse_gh_error` / `parse_glab_error` / `parse_gitcode_error` |
| No raw stderr leaks | `Display` impl hides `raw_stderr`, shows `hint` + `doc_link` |

**What's still missing**:
- No standalone `gf doctor` command (prerequisites check runs silently as fast-fail)
- No all-platforms-at-once diagnostic view
- No skills / agent / gf-self / agent-env checks
- Some error paths still bypass `parse_*_error`
- No GitHub Issue templates
- No CONTRIBUTING.md with SLA

### Refined Scope

| Task | Scope | Type |
|------|-------|------|
| `gf doctor` | New standalone subcommand, 4 check categories, Terminal + JSON output | New feature |
| Error message audit | Audit + fill gaps (GitCode PR #90 hints, more error codes, fix bare `map_err`) | Incremental |
| Issue ops | 2 templates (bug + feature, Chinese-first) + SLA announcement + 5 good first issue labels | Config / docs |

## 2. Architecture: `gf doctor`

### Approach: Trait-based check system

```rust
// crates/core/src/doctor.rs

pub trait HealthCheck: Send + Sync {
    fn category(&self) -> &str;
    fn run(&self) -> Vec<CheckItem>;
}
```

Each category implements the trait independently. The doctor command iterates and collects results. This matches existing patterns (`AuthProvider`, `AuthChecker`) and allows future extensions (e.g., MCP server checks in Phase 3).

### Data Model

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckItem {
    pub category: String,
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DoctorReport {
    pub items: Vec<CheckItem>,
    pub summary: DoctorSummary,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DoctorSummary {
    pub total: usize,
    pub passed: usize,
    pub warned: usize,
    pub failed: usize,
}
```

### Check Categories

| Category | Struct | Location | Checks |
|----------|--------|----------|--------|
| 平台 CLI | `PlatformCliCheck` | `apps/cli/src/commands/doctor/` | 3 platforms × (installed?, version OK?, authenticated?) = 9 items |
| Agent + Skills | `AgentSkillsCheck` | `apps/cli/src/commands/doctor/` | Agent detected?, skills dir exists?, skill count |
| gf 自身 | `GfSelfCheck` | `apps/cli/src/commands/doctor/` | gf version, binary path, latest release comparison |
| Agent 环境 | `AgentEnvCheck` | `apps/cli/src/commands/doctor/` | .claude/ dir?, CLAUDE.md?, hooks configured? |

### Reuse Strategy

| Existing code | Reuse in doctor |
|---------------|-----------------|
| `prerequisites::requirement_for()` + `which` + `get_version` + `create_auth_checker` | `PlatformCliCheck` reuses all logic but collects all results instead of fast-failing |
| `skills::list_skills()` directory scan | `AgentSkillsCheck` reuses `resolve_target_dir` |
| `AgentPlatform::detect()` | `AgentSkillsCheck` calls directly |
| `CliOutput` / `print_output()` | Terminal + JSON output reuses existing pattern |

### CLI Interface

```
gf doctor [--output json|text|auto]
```

- Default: text (colored terminal report)
- `--output json`: structured `DoctorReport` JSON (for MCP/Agent consumption)
- Exit codes: 0 = all pass, 1 = any fail, 2 = any warn (CI-friendly)

### Terminal Output Format

```
$ gf doctor

🩺 gitflow-cli 环境诊断
━━━━━━━━━━━━━━━━━━━━━━━━━━━

📦 平台 CLI
  ✅ [GitHub]  gh v2.65.0 — 已认证 (byx-darwin)
  ✅ [GitLab]  glab v1.46.0 — 已认证
  ⚠️ [GitCode] gc v0.6.1 — 未认证
     🔧 修复：运行 `gc auth login` 完成登录

🤖 Agent + Skills
  ✅ Agent 平台: Claude Code
  ✅ Skills: 26 个已安装 (/Users/.../ .claude/skills)

🔧 gf 自身
  ✅ gf v0.9.0 (/Users/.../bin/gf)
  ⚠️ 最新版本: v0.9.1（建议更新）
     🔧 修复：运行 `gf update`

🏠 Agent 运行环境
  ✅ .claude/ 目录存在
  ✅ CLAUDE.md 存在
  ⚠️ Hooks 未配置（建议安装 auto-report-bug）

━━━━━━━━━━━━━━━━━━━━━━━━━━━
总计: 12 项 · ✅ 9 通过 · ⚠️ 3 警告 · ❌ 0 失败
```

### Routing

`gf doctor` must bypass `prerequisites::check()` in `async_main()` (same as `Skills`/`Completions`/`Workflow`/`Update`), since the doctor itself IS the diagnostic — it shouldn't fail on its own prerequisite checks.

## 3. Error Message Audit

### Current State

`PlatformCliError` + `parse_gh_error` / `parse_glab_error` / `parse_gitcode_error` already cover:
- JSON error parsing + error code mapping (NOT_FOUND, FORBIDDEN, UNAUTHORIZED)
- `hint` + `doc_link` auto-attachment
- `Display` impl does not leak `raw_stderr`
- Chinese user-facing messages

### Gap Analysis

| Gap | Current | Fix |
|-----|---------|-----|
| **GitCode PR errors** (#90) | `parse_gitcode_error` only has generic auth/NOT_FOUND mapping | Add PR-specific error codes: `PR_DISABLED`, `BRANCH_PROTECTED`, with GitCode-specific fix hints |
| **Non-parse error paths** | Some `map_err` directly `miette::miette!("Failed: {e}")` | Audit all `map_err` calls, ensure they go through `parse_*_error` |
| **Incomplete error code coverage** | Only 3 generic codes mapped | Add: `VALIDATION_FAILED`, `CONFLICT`, `RATE_LIMITED`, `GONE` |
| **Generic hints** | All hints are "run `X auth login`" | Refine per error type: permission denied → "check repo permissions"; PR closed → "PR is closed, cannot operate" |

### Audit Plan

1. Search all `map_err` calls in `crates/{github,gitlab,gitcode}/src/`
2. Classify: which use `parse_*_error`, which pass through raw
3. Fix raw pass-through paths: replace with `parse_*_error` calls
4. Extend error code mappings: add 3-5 common codes per platform
5. Refine hints: provide specific fix suggestions per error type

### Scope Guard

- ✅ Audit + fill gaps (new error code mappings + fix bare `map_err` + refine hints)
- ❌ Do NOT rewrite existing `parse_*_error` function structure
- ❌ Do NOT change `PlatformCliError` type definition

## 4. Issue Templates + SLA

### Issue Templates

**Location**: `.github/ISSUE_TEMPLATE/`

**Template 1: Bug Report** (`bug_report.yml`)

Fields:
- `title` prefix: `[Bug]: `
- `labels`: `["bug", "triage:needed"]`
- Markdown intro: "感谢报告 Bug！请填写以下信息帮助我们复现问题。"
- `description` (textarea, required): 问题描述
- `reproduction` (textarea, required): 复现步骤
- `environment` (textarea, required): 环境信息 — prompts user to run `gf doctor` and paste output
- `logs` (textarea, code block): 错误日志
- `checklist` (checkboxes): "已运行 gf doctor" + "已搜索现有 Issues"

**Template 2: Feature Request** (`feature_request.yml`)

Fields:
- `title` prefix: `[Feature]: `
- `labels`: `["enhancement", "triage:needed"]`
- `problem` (textarea, required): 问题背景
- `solution` (textarea, required): 期望方案
- `alternatives` (textarea): 替代方案
- `platform` (dropdown, multiple): GitHub / GitLab / GitCode / 全平台 / 平台无关
- `willing` (checkboxes): "愿意实现" + "可以提供测试反馈"

### SLA Announcement

**Location**: Append to existing `CONTRIBUTING.md`

Content:
- **48 小时内**完成新 Issue 初审（分类、标签、优先级评估）
- 紧急 Bug：**24 小时内**响应
- 标签体系说明（`triage:needed` → `triage:done`, `priority:*`, `good first issue`）
- 贡献指南入口

### Good First Issue Labels

从现有 open issues 中挑选 5 个标记 `good first issue`。候选标准：
- 范围小（单文件或单模块改动）
- 不涉及复杂架构决策
- 有明确的验收标准

## 5. Testing Strategy

### `gf doctor` Unit Tests

| Test | What it verifies |
|------|-----------------|
| `test_should_report_pass_for_installed_cli` | Installed CLI → `CheckStatus::Pass` + version detail |
| `test_should_report_fail_for_missing_cli` | Missing binary → `CheckStatus::Fail` + install hint |
| `test_should_report_warn_for_outdated_version` | Version below min → `CheckStatus::Warn` + upgrade hint |
| `test_should_report_fail_for_unauthenticated` | Not authenticated → `CheckStatus::Fail` + login hint |
| `test_should_report_warn_for_no_skills_installed` | Empty skills dir → `CheckStatus::Warn` |
| `test_should_collect_all_results_not_fast_fail` | One check failure doesn't block subsequent checks |
| `test_should_serialize_report_to_json` | `DoctorReport` → JSON structure correct |
| `test_should_calculate_summary_counts` | passed/warned/failed counts correct |

### Testability Design

- `PlatformCliCheck` accepts `CliRequirement` and `AuthChecker` as parameters for mock injection
- `which::which` calls abstracted via closure, replaced with temp directory lookup in tests

### Error Audit Tests

| Test | What it verifies |
|------|-----------------|
| `test_should_parse_gitcode_pr_disabled_error` | GitCode PR disabled → specific hint |
| `test_should_map_rate_limited_code` | `RATE_LIMITED` → "retry later" hint |
| `test_should_not_leak_stderr_in_new_error_paths` | New paths also don't leak raw stderr |

### Issue Templates

No code tests. Acceptance: templates render correctly on GitHub Issue creation page after push.

## 6. File Inventory

| File | Action | Description |
|------|--------|-------------|
| `crates/core/src/doctor.rs` | **New** | `HealthCheck` trait + data model (`CheckItem`, `DoctorReport`, `DoctorSummary`) |
| `apps/cli/src/commands/doctor.rs` | **New** | CLI command handler + 4 check category struct implementations (single file; categories are small structs implementing `HealthCheck`) |
| `apps/cli/src/main.rs` | **Edit** | Add `Commands::Doctor` variant + route to handler, skip prerequisites check |
| `crates/core/src/lib.rs` | **Edit** | Export `doctor` module |
| `crates/gitcode/src/error.rs` | **Edit** | Add PR-specific error codes + refined hints |
| `crates/github/src/error.rs` | **Edit** | Add more error code mappings + refined hints |
| `crates/gitlab/src/error.rs` | **Edit** | Add more error code mappings + refined hints |
| Various `crates/*/src/*.rs` | **Edit** | Fix bare `map_err` paths → use `parse_*_error` |
| `.github/ISSUE_TEMPLATE/bug_report.yml` | **New** | Bug report template |
| `.github/ISSUE_TEMPLATE/feature_request.yml` | **New** | Feature request template |
| `CONTRIBUTING.md` | **Edit** | Append SLA announcement section (file already exists) |

## 7. Non-Goals

- ❌ Auto-fix mode (`gf doctor --fix`) — future enhancement
- ❌ Rewrite existing `parse_*_error` function signatures
- ❌ Change `PlatformCliError` type definition
- ❌ Network-dependent checks (e.g., DNS resolution, API reachability beyond auth status)
- ❌ Plugin / extension system checks (2.0 scope)
