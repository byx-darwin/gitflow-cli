<!-- Issue: #N -->

# Phase 1: Core + CLI 基础（MVP）实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为单一平台（GitHub）打通 Issue 和 PR 的核心读写链路，验证 Rust trait 抽象 + subprocess 调用原生 CLI 的架构可行性。

**Architecture:** 扩展 `crates/core`（domain types + traits）→ 新建 `crates/github`（GitHubProvider 实现，调用 `gh` CLI）→ 扩展 `apps/cli`（clap 子命令 + 前置检查 + JSON 输出）。平台检测基于 `git remote get-url origin`。

**Tech Stack:** Rust 2024, clap, tokio, serde_json, async-trait, chrono, which, regex

## Global Constraints

- Rust 2024 edition，工具链 1.96.0
- `#![forbid(unsafe_code)]` 所有 crate
- 禁止 `unwrap()` / `expect()`，返回 `Result<T>`
- 公共项必须文档化（`#![warn(missing_docs)]`）
- 所有领域类型派生/实现 `Debug`
- TDD：先写测试，确认失败，再写实现
- 遵循 workspace lint policy（`[workspace.lints.rust]` / `[workspace.lints.clippy]`）

## Issue 规划

**Issue 标题:** feat: Phase 1 — Core + CLI 基础（MVP），打通 GitHub Issue/PR 读写链路

**Issue 标签:** enhancement,core,cli,github,phase-1

**Issue 描述:**
实现 gitflow-cli 的 Phase 1 MVP：扩展 crates/core 定义平台抽象 trait 和 domain types，新建 crates/github 通过 subprocess 调用 `gh` CLI 实现 Issue/PR 操作，扩展 apps/cli 新增 `gitflow issue` 和 `gitflow pr` 子命令。同时实现平台自动检测和原生 CLI 前置检查（PATH + 版本最低要求）。此阶段仅支持 GitHub，为后续 GitLab/GitCode 扩展建立架构基础。

**验收标准:**
- [ ] 所有任务完成
- [ ] 测试通过（单元测试 + 集成测试）
- [ ] 代码审查通过
- [ ] `cargo build` 全 workspace 通过
- [ ] `cargo test` 全 workspace 通过
- [ ] `cargo clippy -- -D warnings` 通过
- [ ] `cargo +nightly fmt -- --check` 通过
- [ ] `gitflow issue create/list/view` 三个子命令可执行
- [ ] `gitflow pr create/list/view` 三个子命令可执行
- [ ] 平台检测（git remote）正常工作
- [ ] 原生 CLI 缺失时打印明确安装指引
- [ ] 覆盖率 > 80%

**关联:**
- 计划文件: `docs/superpowers/plans/2026-07-01-phase1-core-cli-foundation.md`
- 设计文档: `specs/gitflow-cli-design.md`（Phase 1 详细设计）
- 里程碑: MVP

## File Structure

```
gitflow-cli/
├── Cargo.toml                  # 新增 workspace dependencies（async-trait, chrono, which, regex）
├── crates/
│   ├── core/
│   │   ├── Cargo.toml          # 新增依赖：async-trait, chrono, serde, serde_json
│   │   └── src/
│   │       ├── lib.rs          # 修改：pub mod 声明 + CoreError 新增 Platform variant
│   │       ├── platform.rs     # 新增：Platform 枚举 + detect_from_remote_url()
│   │       ├── types.rs        # 新增：UserSummary, State, Label
│   │       ├── issue.rs        # 新增：IssueProvider trait + IssueData + args
│   │       ├── pr.rs           # 新增：PrProvider trait + PrData + args
│   │       └── output.rs       # 新增：CliOutput<T>, CliError
│   └── github/
│       ├── Cargo.toml          # 新增
│       └── src/
│           ├── lib.rs          # 新增：pub mod 声明
│           ├── error.rs        # 新增：GhError + parse_gh_error()
│           ├── issue.rs        # 新增：GitHubIssueProvider
│           └── pr.rs           # 新增：GitHubPrProvider
├── apps/
│   └── cli/
│       ├── Cargo.toml          # 新增依赖：gitflow-cli-github, miette, which
│       └── src/
│           ├── main.rs         # 修改：Cli 结构扩展 + resolve_platform() + maybe_report_error()
│           ├── config.rs       # 现有，保持不变
│           ├── error_reporter.rs  # 新增：非交互模式下错误写入 pending.json
│           └── commands/
│               ├── mod.rs      # 修改：新增 issue, pr 模块声明
│               ├── run.rs      # 现有，保持不变
│               ├── completions.rs  # 现有，保持不变
│               ├── issue.rs    # 新增：IssueCommand + handle()
│               ├── pr.rs       # 新增：PrCommand + handle()
│               └── prerequisites.rs  # 新增：原生 CLI 前置检查
├── scripts/
│   └── smoke-test.sh           # 新增：Phase 1 冒烟测试
├── hooks/
│   └── auto-report-bug.sh     # 新增：Stop Hook 检测 pending.json
├── specs/
│   └── gitflow-cli-design.md   # 已更新：Phase 1 详细设计
└── .claude/
    └── gh-issue/
        └── current-issue.txt   # 运行时生成（gitignore）
    └── settings.json           # 修改：注册 Stop Hook
```

## Tasks

### Task 1: 创建 Issue

**Description:** 从 "Issue 规划" 部分提取信息，创建 GitHub Issue 并保存编号。

- [ ] **Step 1: 运行 scripts/create-issue.sh**

```bash
bash /Users/byx/.claude/skills/writing-plans-with-issue/scripts/create-issue.sh docs/superpowers/plans/2026-07-01-phase1-core-cli-foundation.md
```

- [ ] **Step 2: 验证 Issue 已创建**

```bash
cat .claude/gh-issue/current-issue.txt
gh issue view "$(cat .claude/gh-issue/current-issue.txt)"
```

### Task 2: 同步 Issue 状态为 in-progress

**Description:** 将 Issue 状态更新为 `status: in-progress`。

- [ ] **Step 1: 运行 scripts/sync-status.sh**

```bash
bash /Users/byx/.claude/skills/writing-plans-with-issue/scripts/sync-status.sh in-progress
```

- [ ] **Step 2: 确认**

```bash
echo "✅ Issue #$(cat .claude/gh-issue/current-issue.txt) 已标记为 in-progress"
```

### Task 3: crates/core — 错误类型扩展 + domain types

**Description:** 在 `CoreError` 新增 `Platform` variant；新建 `types.rs` 定义 `UserSummary`、`State`、`Label` 共享类型。均为纯数据结构，无外部依赖。

**Dependencies:** 无（Task 1/2 之后第一个开发任务）

- [ ] **Step 1: 扩展 CoreError（lib.rs）**

在现有 `CoreError` 枚举中新增 `Platform(String)` variant，派生 `#[error("platform error: {0}")]`。不修改现有 variants。

- [ ] **Step 2: 新建 types.rs**

```rust
// UserSummary { login: String, id: u64 } — Serialize + Deserialize
// State { Open, Closed } — Serialize, rename_all = "snake_case"
// Label { name, color: Option<String>, description: Option<String> }
```

- [ ] **Step 3: 写单元测试**

```rust
// test_should_serialize_state_to_snake_case
// test_should_deserialize_user_summary_from_json
// test_should_deserialize_label_with_optional_fields
```

- [ ] **Step 4: 在 lib.rs 中声明 `pub mod types;`**

> **Commit:** `feat(core): add domain types and Platform error variant (#N)`

### Task 4: crates/core — Platform 枚举 + 检测逻辑

**Description:** 新建 `platform.rs`，定义 `Platform` 枚举（GitHub/GitLab/GitCode），实现 `detect_from_remote_url()` 方法。

**Dependencies:** Task 3

- [ ] **Step 1: 新建 platform.rs**

```rust
// Platform enum: GitHub, GitLab, GitCode — Debug, Clone, Copy, PartialEq, Eq, Hash
// detect_from_remote_url(url: &str) -> Option<Self>
// 匹配规则：github.com/github. → GitHub; gitlab.com/gitlab. → GitLab; gitcode.com/gitcode. → GitCode
// 不匹配 → None
```

- [ ] **Step 2: 扩展 CoreError 新增检测相关 variant**

```rust
// 或直接复用 Platform(String) variant
```

- [ ] **Step 3: 写单元测试**

```rust
// test_should_detect_github_from_https_url
// test_should_detect_github_from_ssh_url
// test_should_detect_gitlab_from_https_url
// test_should_detect_gitlab_from_self_hosted_url (e.g. gitlab.mycorp.com)
// test_should_detect_gitcode
// test_should_return_none_for_unrecognized_url
// test_should_be_case_insensitive
```

- [ ] **Step 4: 在 lib.rs 中声明 `pub mod platform;`**

> **Commit:** `feat(core): add Platform enum with remote URL detection (#N)`

### Task 5: crates/core — IssueProvider + PrProvider trait 定义

**Description:** 新建 `issue.rs` 和 `pr.rs`，定义 Issue/PR 的 trait、data types、args types。这是整个架构的核心抽象。

**Dependencies:** Task 3, Task 4

- [ ] **Step 1: 新建 issue.rs**

```rust
// IssueData { number, title, body, state, labels, author, assignees, created_at, updated_at, url }
// CreateIssueArgs { title, body, labels, assignees }
// ListIssueArgs { state, labels, assignee, search, limit } — Default
// #[async_trait] IssueProvider { create, list, view }
```

- [ ] **Step 2: 新建 pr.rs**

```rust
// PrData { number, title, body, state, draft, author, base_branch, head_branch, created_at, updated_at, url }
// CreatePrArgs { title, body, head, base, draft, repo }
// ListPrArgs { state, limit } — Default
// #[async_trait] PrProvider { create, list, view }
```

- [ ] **Step 3: 写单元测试（doc tests）**

每个 data type 的 JSON 序列化/反序列化测试。

- [ ] **Step 4: 在 lib.rs 中声明 `pub mod issue;` `pub mod pr;`**

- [ ] **Step 5: 在 Cargo.toml 中添加依赖**

```toml
# crates/core/Cargo.toml
[dependencies]
async-trait.workspace = true
chrono.workspace = true
serde.workspace = true
serde_json.workspace = true
```

> **Commit:** `feat(core): define IssueProvider and PrProvider traits (#N)`

### Task 6: crates/core — JSON 输出类型 + lib.rs 重新导出

**Description:** 新建 `output.rs`，定义 `CliOutput<T>` 和 `CliError` 类型。更新 `lib.rs` 模块文档和重新导出。

**Dependencies:** Task 5

- [ ] **Step 1: 新建 output.rs**

```rust
// CliOutput<T> { success, data: Option<T>, error: Option<CliError>, platform, command }
// CliError { code, message, hint: Option<String> }
// CliOutput::success(data, platform, command) -> Self
// CliOutput::failure(error, platform) -> Self
// CliError::new(code, message) -> Self + with_hint(hint) -> Self
```

- [ ] **Step 2: 写单元测试**

```rust
// test_should_serialize_success_output
// test_should_serialize_failure_output
// test_should_omit_data_field_on_failure
// test_should_omit_error_field_on_success
// test_should_add_hint_to_error
```

- [ ] **Step 3: 更新 lib.rs 模块文档和重新导出**

```rust
//! 重新导出主要类型
pub use issue::{CreateIssueArgs, IssueData, IssueProvider, ListIssueArgs};
pub use output::{CliError, CliOutput};
pub use platform::Platform;
pub use pr::{CreatePrArgs, ListPrArgs, PrData, PrProvider};
pub use types::{Label, State, UserSummary};
```

> **Commit:** `feat(core): add JSON output types and public re-exports (#N)`

### Task 7: crates/github — crate 搭建 + GitHubProvider 实现

**Description:** 新建 `crates/github` crate。实现 `GhError` 错误解析、`GitHubIssueProvider`、`GitHubPrProvider`。通过 `tokio::process::Command` 调用 `gh` CLI，解析 JSON 输出。

**Dependencies:** Task 6（需要 core 的 traits 和 types）

- [ ] **Step 1: 新建 crates/github/Cargo.toml**

```toml
[package]
name = "gitflow-cli-github"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[dependencies]
gitflow-cli-core.workspace = true
async-trait.workspace = true
serde.workspace = true
serde_json.workspace = true
tokio.workspace = true
tracing.workspace = true
thiserror.workspace = true
```

- [ ] **Step 2: 新建 src/error.rs**

实现 `GhError` 结构体和 `parse_gh_error(stderr: &[u8]) -> GhError`。优先尝试 JSON 解析 gh 的错误输出，回退到文本前三行。

- [ ] **Step 3: 新建 src/issue.rs**

实现 `GitHubIssueProvider { repo: String }`，impl `IssueProvider` trait。三个方法：`create` / `list` / `view`。

关键实现细节：
- `create`：`gh issue create --repo <repo> --title <title> --json <fields>`
- `list`：`gh issue list --repo <repo> --json <fields> [--state --search --limit]`
- `view`：`gh issue view <number> --repo <repo> --json <fields>`
- stdout JSON 反序列化为 `IssueData`
- stderr 非空 → `parse_gh_error()` → `CoreError::Platform`
- spawn 失败 → `CoreError::Platform`

- [ ] **Step 4: 新建 src/pr.rs**

实现 `GitHubPrProvider`，impl `PrProvider` trait。与 issue.rs 模式一致。

- [ ] **Step 5: 新建 src/lib.rs**

```rust
#![forbid(unsafe_code)]
pub mod error;
pub mod issue;
pub mod pr;
pub use issue::GitHubIssueProvider;
pub use pr::GitHubPrProvider;
```

- [ ] **Step 6: 写单元测试（mock gh 输出）**

```rust
// test_should_parse_gh_error_from_json_stderr
// test_should_parse_gh_error_from_plain_text_stderr
// test_should_serialize_issue_data_from_gh_output — 用示例 gh JSON 输出验证反序列化
// test_should_serialize_pr_data_from_gh_output
```

> **Commit:** `feat(github): add GitHubIssueProvider and GitHubPrProvider (#N)`

### Task 8: apps/cli — 原生 CLI 前置检查模块

**Description:** 新建 `apps/cli/src/commands/prerequisites.rs`。在确定目标平台后，检查对应的原生 CLI 是否在 PATH 上且版本满足最低要求。失败则阻断执行并打印安装指引。

**Dependencies:** Task 7（需要知道 crate 结构，但逻辑独立）

- [ ] **Step 1: 新建 prerequisites.rs**

```rust
// CliRequirement { binary, min_version, install_url, install_hint }
// requirement_for(platform: &str) -> CliRequirement
//   "github" → gh v2.0.0+
//   "gitlab" → glab v1.30.0+
//   "gitcode" → gc v0.6.0+
// check(platform: &str) -> Result<(), PrerequisiteError>  // 主入口
//   → which::which(binary) 检查 PATH
//   → Command::new(binary).arg("--version") 获取版本
//   → extract_semver() + version_meets_minimum() 比较版本
// PrerequisiteError { NotFound, VersionTooLow, VersionParseFailed } — thiserror + 友好错误信息
```

- [ ] **Step 2: 写单元测试**

```rust
// test_should_return_requirement_for_github
// test_should_extract_semver_from_gh_version_output ("gh version 2.50.0 (2024-01-01)")
// test_should_extract_semver_from_glab_version_output
// test_should_version_meets_minimum_pass
// test_should_version_meets_minimum_fail
// test_should_version_meets_minimum_equal
```

- [ ] **Step 3: 在 apps/cli/Cargo.toml 中新增依赖**

```toml
which.workspace = true
regex.workspace = true
```

> **Commit:** `feat(cli): add native CLI prerequisite checker (#N)`

### Task 9: apps/cli — main.rs 扩展 + 平台检测 + 命令框架

**Description:** 扩展 `main.rs` 的 `Cli` 结构，新增 `--platform`、`--output` 全局 flag 和 `Issue`/`Pr`/`SkillsInstall` 子命令。实现 `resolve_platform()` 和 `extract_repo_from_url()` 函数。更新 `async_main` 流程：解析平台 → 前置检查 → 分发命令。

**Dependencies:** Task 8（需要 prerequisites::check）

- [ ] **Step 1: 扩展 Cli 结构**

```rust
// 新增: PlatformArg enum (GitHub, Gitlab, Gitcode) — clap::ValueEnum
// 新增: OutputFormat enum (Json, Text, Default=Json) — clap::ValueEnum
// Cli 结构: #[arg(long, global = true)] platform, output, verbose
// Commands: Issue(IssueCommand), Pr(PrCommand), SkillsInstall, Completions(Shell)
```

- [ ] **Step 2: 实现 resolve_platform()**

```rust
// 1. --platform flag → 直接使用
// 2. git remote get-url origin → Platform::detect_from_remote_url()
// 3. 失败 → miette error + 提示使用 --platform
```

- [ ] **Step 3: 实现 extract_repo_from_url()**

```rust
// 支持 HTTPS: https://github.com/owner/repo.git → owner/repo
// 支持 SSH: git@github.com:owner/repo.git → owner/repo
// trim "https://" / "http://" / "git@" / ".git", split by ":" or "/"
```

- [ ] **Step 4: 更新 async_main 流程**

```rust
// let (platform, repo) = resolve_platform(cli.platform)?;
// prerequisites::check(&platform)?;
// match cli.command { Commands::Issue(cmd) => ... Commands::Pr(cmd) => ... }
```

- [ ] **Step 5: 写单元测试**

```rust
// test_should_extract_repo_from_https_url
// test_should_extract_repo_from_ssh_url
// test_should_extract_repo_without_dot_git_suffix
```

- [ ] **Step 6: 更新 commands/mod.rs**

```rust
pub mod completions;
pub mod issue;
pub mod pr;
pub mod prerequisites;
pub mod run;
```

> **Commit:** `feat(cli): extend CLI structure with platform detection and Issue/Pr subcommands (#N)`

### Task 10: apps/cli — commands/issue.rs 实现

**Description:** 实现 `gitflow issue {create,list,view}` 三个子命令。解析 clap 参数 → 构造 provider → 调用 trait 方法 → JSON 输出。

**Dependencies:** Task 9（需要 main.rs 中的 IssueCommand 类型引用）

- [ ] **Step 1: 新建 commands/issue.rs**

```rust
// IssueCommand enum (clap::Subcommand): Create, List, View
// handle(command, platform, repo, output_format) -> miette::Result<()>
//   → match platform: "github" → GitHubIssueProvider::new(repo)
//   → 其他 → miette error "not yet supported"
//   → match command: Create/List/View → 调用 provider 方法 → print_output
// resolve_body(body, body_file) -> miette::Result<Option<String>>
//   → --body_file 暂未实现，返回 miette error "body-file not yet supported in Phase 1"
// print_output(output, format) → match format { Json => serde_json::to_string_pretty, Text => TODO }
```

- [ ] **Step 2: 写集成测试（assert_cmd）**

在 `apps/cli/tests/` 下新建 issue 测试。由于实际执行需要 `gh` CLI 和 GitHub 仓库，集成测试仅验证 help 输出和参数解析，不实际调用 gh。

```rust
// test_should_show_issue_create_help
// test_should_show_issue_list_help
// test_should_reject_missing_required_args
```

> **Commit:** `feat(cli): implement gitflow issue create/list/view commands (#N)`

### Task 11: apps/cli — commands/pr.rs 实现

**Description:** 实现 `gitflow pr {create,list,view}` 三个子命令。与 issue.rs 一致的模式。`create` 额外支持 `--head`、`--base`、`--draft`、`--repo`。

**Dependencies:** Task 10（可参考 issue.rs 模式，无代码依赖）

- [ ] **Step 1: 新建 commands/pr.rs**

```rust
// PrCommand enum: Create, List, View
// Create args: --title, --head, --base, --body, --body-file, --draft, --repo
// List args: --state, --limit
// View args: number
// handle() 逻辑与 issue.rs 对称
```

- [ ] **Step 2: 集成测试**

```rust
// test_should_show_pr_help
// test_should_reject_missing_create_args
```

> **Commit:** `feat(cli): implement gitflow pr create/list/view commands (#N)`

### Task 12: apps/cli — 错误自动报告模块 + Hook

**Description:** 新建 `apps/cli/src/error_reporter.rs`，实现非交互模式下将 CLI 错误写入 `.cache/bug-reports/pending.json`。新建 `hooks/auto-report-bug.sh`，由 Claude Code Stop Hook 触发检测。更新 `main.rs` 的 `async_main` Err 分支。

**Dependencies:** Task 11（在所有 CLI 命令实现完成后，统一的错误报告入口）

- [ ] **Step 1: 新建 error_reporter.rs**

```rust
// ErrorReport { id, source, command, platform, exit_code, error_code, error_message, hint, timestamp }
//   id = 时间戳 + PID 的 hash（用于去重）
//   source = "cli"
// ErrorReport::from_error(command, platform, error, error_code) -> Self
// ErrorReport::write_to_disk(&self, repo_root: &Path) -> Result<(), io::Error>
//   写入路径: repo_root/.cache/bug-reports/pending.json
//   自动创建 .cache/bug-reports/ 目录
// maybe_report_error(command, platform, error_str, error_code) → 
//   仅在 !stderr.is_terminal() 时写入（非交互模式：CI / subprocess）
```

- [ ] **Step 2: 新建 hooks/auto-report-bug.sh**

```bash
#!/usr/bin/env bash
set -euo pipefail
REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || exit 0)
PENDING_FILE="$REPO_ROOT/.cache/bug-reports/pending.json"
# 检查 pending.json 是否存在 → 不存在则静默退出
# 存在则输出提示 + cat pending.json 内容供 Claude 读取
```

- [ ] **Step 3: 修改 main.rs async_main Err 分支**

```rust
Err(e) => {
    if !std::io::stderr().is_terminal() {
        let _ = error_reporter::maybe_report_error(
            &cli.command_name(),
            &platform,
            &e.to_string(),
            "CLI_ERROR",
        );
    }
    eprintln!("{e:?}");
    std::process::ExitCode::from(1)
}
```

- [ ] **Step 4: 新增 .claude/settings.json Stop Hook 配置**

```json
{
  "hooks": {
    "Stop": [
      {
        "matcher": "gitflow",
        "command": "bash hooks/auto-report-bug.sh"
      }
    ]
  }
}
```

- [ ] **Step 5: 写单元测试**

```rust
// test_should_create_error_report_from_error
// test_should_write_pending_json_to_disk
// test_should_generate_unique_id
// test_should_skip_when_interactive_terminal
```

> **Commit:** `feat(cli): add error auto-report module and Stop Hook (#N)`

### Task 13: 冒烟测试脚本 + 集成测试

**Description:** 编写 `scripts/smoke-test.sh`（Phase 1 最小版本）和 Rust 端到端集成测试。冒烟测试仅执行只读操作（view/list），避免在 CI 中创建垃圾数据。

**Dependencies:** Task 10, Task 11

- [ ] **Step 1: 新建 scripts/smoke-test.sh**

```bash
#!/usr/bin/env bash
set -euo pipefail
echo "=== gitflow-cli Smoke Test (Phase 1 - GitHub) ==="
echo "[1/3] issue view"
gitflow issue view 1 --platform github 2>&1 | head -5
echo "[2/3] issue list"
gitflow issue list --state open --limit 3 --platform github 2>&1 | head -5
echo "[3/3] pr list"
gitflow pr list --state open --limit 3 --platform github 2>&1 | head -5
echo "=== Smoke test passed ==="
```

- [ ] **Step 2: 扩展 existing 集成测试**

在 `apps/cli/tests/` 中新增一个 JSON 输出格式验证测试（`assert_cmd` + JSON 格式断言）。

> **Commit:** `test: add Phase 1 smoke test script and integration tests (#N)`

### Task 14: 终检 — clippy + fmt + build + test

**Description:** 运行全 workspace 的构建、测试、lint、格式化检查。修复所有 warning 和 error。确保 CI 就绪。

**Dependencies:** Task 13（全部开发任务完成后）

- [ ] **Step 1: cargo build —workspace**

```bash
cargo build --workspace 2>&1
```

- [ ] **Step 2: 修复编译错误和 warning**

```bash
cargo build --workspace 2>&1 | grep -E "error|warning"
# 逐个修复
```

- [ ] **Step 3: cargo test —workspace**

```bash
cargo test --workspace 2>&1
```

- [ ] **Step 4: 修复测试失败**

- [ ] **Step 5: cargo clippy --workspace -- -D warnings**

```bash
cargo clippy --workspace --all-targets -- -D warnings 2>&1
```

- [ ] **Step 6: 修复 clippy 告警**

- [ ] **Step 7: cargo +nightly fmt -- --check**

```bash
cargo +nightly fmt -- --check 2>&1
```

- [ ] **Step 8: 格式化所有代码**

```bash
cargo +nightly fmt
```

- [ ] **Step 9: 最终确认全部通过**

```bash
cargo build --workspace && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && cargo +nightly fmt -- --check && echo "✅ All checks passed"
```

> **Commit:** `chore: final lint and formatting pass for Phase 1 (#N)`

### Task 15: 收尾 — 本地合并后关闭 Issue

**Description:** 开发完成并本地合并到 base 分支后，push 并关闭 Issue。

- [ ] **Step 1: 确保已合并到 base 分支**

```bash
git branch --show-current
```

- [ ] **Step 2: 运行 scripts/finish-issue.sh**

```bash
bash /Users/byx/.claude/skills/writing-plans-with-issue/scripts/finish-issue.sh
```

- [ ] **Step 3: 确认 Issue 已关闭**

```bash
gh issue view "$(cat .claude/gh-issue/current-issue.txt 2>/dev/null || echo 'already cleaned')"
```
