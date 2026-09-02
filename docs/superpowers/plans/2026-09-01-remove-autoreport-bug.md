# 删除自动上报bug功能与共建计划安装提示 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 从代码库中彻底移除"共建计划"体系（自动上报 bug 能力 + 安装时的加入提示 + doctor 健康检查项），并清理现状类文档中的引用。

**Architecture:** 纯删除工作，不引入新代码路径。按"叶子先删"的顺序推进：先删除被依赖的底层模块（`error_reporter.rs`），再删除依赖它的上层调用点（`skills.rs`、`doctor.rs`、`main.rs`），最后删除脚本/skill 目录与文档引用。每个任务结束后跑 `cargo build -p gitflow-cli`（或 `cargo check -p gitflow-cli`），编译器会把任何遗漏的调用点暴露为编译错误——这就是本计划里"可验证的交付物"。

**Tech Stack:** Rust 2024 (workspace crate `apps/cli`)，`cargo`/`make` 构建。

**Spec:** `docs/superpowers/specs/2026-09-01-remove-autoreport-bug-design.md`

## Global Constraints

- 只做删除与文档清理，不新增功能、不加兼容层、不加 deprecation 警告（CLAUDE.md：移除死代码而不是抑制它）。
- 不修改 `deny.toml`、`.pre-commit-config.yaml`、`rust-toolchain.toml`。
- 每个任务完成后运行 `cargo build -p gitflow-cli`（快速反馈），全部任务完成后跑一次完整 `make build && make test && make fmt && make clippy`。
- 每个任务是一次独立可编译（或至少可判断是否需要下一任务修复的）状态；只有 Task 6（最终验证）要求整个 workspace 全绿。
- `docs/superpowers/plans`、`specs`、`research`、`reports` 等历史快照文档不动，只清理"现状类"文档（README、architecture、integration-guide、index、被点名的 4 个 SKILL.md）。
- 不 commit、不 push（按用户/CLAUDE.md 要求，commit 需在 Phase 3 worktree 流程中显式征得同意）。

---

### Task 1: 删除 `error_reporter.rs` 模块及其在 `main.rs` 中的接线

**Files:**
- Delete: `apps/cli/src/error_reporter.rs`
- Modify: `apps/cli/src/main.rs:46`（`mod error_reporter;`）、`apps/cli/src/main.rs:99-107`、`apps/cli/src/main.rs:122-131`、`apps/cli/src/main.rs:143-151`、`apps/cli/src/main.rs:159-170`（`report_error_noninteractive` 及其三处调用）

**Interfaces:**
- Produces: `main.rs` 不再声明 `mod error_reporter;`，也不再有 `report_error_noninteractive` 函数；三处错误分支从"报告 + 打印"简化为"仅打印"。
- Consumes: 无（本任务是叶子删除，后续任务依赖"`error_reporter` 已不存在"这一事实）。

- [ ] **Step 1: 删除整个模块文件**

```bash
rm apps/cli/src/error_reporter.rs
```

- [ ] **Step 2: 移除模块声明**

在 `apps/cli/src/main.rs` 删除第 46 行：

```rust
mod error_reporter;
```

（保留同区块的 `mod commands;` 和 `mod errors;`）

- [ ] **Step 3: 删除 `report_error_noninteractive` 函数定义**

删除 `apps/cli/src/main.rs` 中的：

```rust
/// Best-effort error reporting for non-interactive mode.
///
/// Delegates to [`error_reporter::maybe_report_error`], silently
/// discarding any I/O errors. The error report is a diagnostic aid;
/// a failure to write it must never block or alter the exit code.
fn report_error_noninteractive(
    command: &str,
    platform: &str,
    error_message: &str,
    error_code: &str,
) {
    let _ = error_reporter::maybe_report_error(command, platform, error_message, error_code);
}
```

- [ ] **Step 4: 删除三处调用点，改为直接打印**

第一处（运行时创建失败分支）：

```rust
// 删除前
Err(e) => {
    let report = miette::miette!("Failed to create async runtime: {e}");
    report_error_noninteractive(
        &command_name,
        "unknown",
        &report.to_string(),
        "RUNTIME_ERROR",
    );
    eprintln!("{report:?}");
    return std::process::ExitCode::from(1);
}
```

```rust
// 删除后
Err(e) => {
    let report = miette::miette!("Failed to create async runtime: {e}");
    eprintln!("{report:?}");
    return std::process::ExitCode::from(1);
}
```

第二处（平台解析失败分支）：

```rust
// 删除前
Err(e) => {
    report_error_noninteractive(
        &command_name,
        "unknown",
        &e.to_string(),
        "PLATFORM_ERROR",
    );
    eprintln!("{e:?}");
    return std::process::ExitCode::from(1);
}
```

```rust
// 删除后
Err(e) => {
    eprintln!("{e:?}");
    return std::process::ExitCode::from(1);
}
```

第三处（`async_main` 顶层错误分支）：

```rust
// 删除前
Err(e) => {
    if platform_needed {
        let error_code = if e.code().is_some_and(|c| c.to_string() == "gf::user_input") {
            "USER_INPUT_ERROR"
        } else {
            "CLI_ERROR"
        };
        report_error_noninteractive(&command_name, &platform, &e.to_string(), error_code);
    }
    eprintln!("{e:?}");
    std::process::ExitCode::from(1)
}
```

```rust
// 删除后
Err(e) => {
    eprintln!("{e:?}");
    std::process::ExitCode::from(1)
}
```

注意：`error_code` 的计算逻辑（区分 `USER_INPUT_ERROR`/`CLI_ERROR`）只被 `report_error_noninteractive` 消费，整段一并删除；`platform_needed` 变量本身仍被其他逻辑使用（平台解析跳过判断），不要删除它的定义。

- [ ] **Step 5: 编译验证，捕获遗漏引用**

```bash
cargo build -p gitflow-cli 2>&1 | tee /tmp/task1-build.log
```

Expected: 编译失败，报错集中在 `apps/cli/src/commands/skills.rs`（`use crate::error_reporter::...`）和 `apps/cli/src/commands/doctor.rs`（`crate::error_reporter::...`）——这些留给 Task 2、Task 3 修复，属于预期中间态。确认失败原因**只**来自这两个文件，没有其他文件报错。

---

### Task 2: 删除 `skills.rs` 中的共建计划与 auto-report-bug hook 逻辑

**Files:**
- Modify: `apps/cli/src/commands/skills.rs`

**Interfaces:**
- Consumes: Task 1 完成后 `error_reporter` 模块已不存在。
- Produces: `InstallArgs` 不再有 `report_bug` 字段；`install_skills` 不再调用 hook 安装或共建计划提示；模块不再有任何 hook-install / co-contribution 相关的私有函数。

- [ ] **Step 1: 删除顶部对 `error_reporter` 的引用与仅被共建计划逻辑使用的导入**

删除：

```rust
use crate::error_reporter::read_co_contribution_flag;
```

删除（仅 `try_enable_co_contribution` 使用）：

```rust
use is_terminal::IsTerminal;
```

- [ ] **Step 2: 删除 `InstallArgs.report_bug` 字段**

删除：

```rust
    /// 启用自动 bug 上报（Stop Hook），默认开启
    #[arg(long = "report-bug", default_value_t = true, action = ArgAction::Set)]
    pub report_bug: bool,
```

- [ ] **Step 3: 删除 `install_skills` 中的 hook 安装与共建计划调用**

删除：

```rust
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
```

保留 `install_skills` 函数其余逻辑（skills 目录复制部分）不变。

- [ ] **Step 4: 删除 hook-install 专用函数群**

在 `apps/cli/src/commands/skills.rs` 中删除以下函数整体（含文档注释），它们只服务于 auto-report-bug hook 的安装/卸载，无其他调用方：

- `resolve_hook_paths`
- `build_auto_report_hook_cmd`
- `autoreport_repo_slug`
- `autoreport_repo_slug_from_url`
- `is_safe_slug_segment`
- `resolve_global_hook_paths`
- `resolve_project_hook_paths`
- `install_hook`
- `merge_stop_hook`
- `uninstall_hook`

- [ ] **Step 5: 删除共建计划函数群**

删除以下函数整体（含文档注释）：

- `try_enable_co_contribution`
- `merge_co_contribution_json`
- `merge_co_contribution`
- `iso8601_utc_now_co_contribution`
- `confirm`
- `confirm_with_reader`

- [ ] **Step 6: 检查 `uninstall_skills` 中对 `uninstall_hook` 的调用**

`uninstall_skills`（约第 1054 行起）此前会调用 `uninstall_hook` 卸载 Stop Hook 注册；随 Step 4 一并删除该调用及相关的 hook 卸载分支，`uninstall_skills` 只保留 skills 目录卸载逻辑。

- [ ] **Step 7: 删除对应的 `#[cfg(test)] mod tests` 用例**

删除测试模块中所有引用被删函数的测试（`merge_co_contribution_json` 系列、`merge_co_contribution` 系列、`try_enable_co_contribution` 系列、`merge_stop_hook` 系列、`build_auto_report_hook_cmd`/`autoreport_repo_slug*`/`is_safe_slug_segment` 系列、`install_hook`/`uninstall_hook` 系列测试，以及 `InstallArgs { report_bug: false, .. }` 这类测试夹具中的 `report_bug` 字段初始化）。

- [ ] **Step 8: 编译验证**

```bash
cargo build -p gitflow-cli 2>&1 | tee /tmp/task2-build.log
```

Expected: 剩余编译错误应只来自 `apps/cli/src/commands/doctor.rs`（Task 3 处理）。若 `skills.rs` 自身仍报错（如未清理的调用点、未使用的 import），在本任务内修完，不要带着 `skills.rs` 的编译错误进入下一任务。

- [ ] **Step 9: 运行 skills.rs 单元测试**

```bash
cargo test -p gitflow-cli --lib commands::skills:: 2>&1 | tail -40
```

Expected: 剩余测试全部通过（保留下来的 skills 安装/列出/卸载逻辑不受影响）。

---

### Task 3: 删除 `doctor.rs` 中的 `CoContributionCheck`

**Files:**
- Modify: `apps/cli/src/commands/doctor.rs`

**Interfaces:**
- Consumes: Task 1、Task 2 完成，`error_reporter` 与其调用方均已清理。
- Produces: `HealthCheck` 实现列表（`handle` 函数中的 `checks` vec）不再包含 `CoContributionCheck`；`co_contribution` 分类从 doctor 报告中消失。

- [ ] **Step 1: 删除 `CoContributionCheck` 结构体与其 `HealthCheck` 实现**

删除（约第 278-303 行）：

```rust
/// Checks the co-contribution flag (bug auto-report opt-in).
///
/// Reports whether the user has joined the co-contribution plan and how to
/// opt out, making the auto-report feature discoverable and reversible.
pub struct CoContributionCheck;

impl HealthCheck for CoContributionCheck {
    fn category(&self) -> &'static str {
        "co_contribution"
    }

    fn run(&self) -> Vec<CheckItem> {
        let global_path = dirs::home_dir().map(|h| h.join(".claude/settings.json"));
        let project_path = crate::error_reporter::project_settings_path();

        match (global_path, project_path) {
            (Some(global), Some(project)) => co_contribution_check_items_with(&global, &project),
            _ => vec![CheckItem::pass(
                self.category(),
                "共建计划",
                "未加入共建计划，bug 自动上报未开启",
            )],
        }
    }
}
```

- [ ] **Step 2: 删除 `co_contribution_check_items_with` 辅助函数**

删除该函数整体（紧跟在上面结构体之后，直到函数结束的 `}`，覆盖 pending-ack 分支与 enabled/未 enabled 分支）。

- [ ] **Step 3: 从 `handle` 中移除注册**

在 `pub fn handle(args: &DoctorArgs)` 的 `checks` vec 中删除：

```rust
        Box::new(CoContributionCheck),
```

保留 `PlatformCliCheck`、`AgentSkillsCheck`、`GfSelfCheck`、`AgentEnvCheck` 四项。

- [ ] **Step 4: 清理测试模块**

- 在 `test_should_collect_all_categories_in_report` 测试中：从 `checks` vec 删除 `Box::new(CoContributionCheck),`，并删除断言 `assert!(categories.contains("co_contribution"));`。
- 整体删除以下测试函数：`test_co_contribution_check_reports_opt_out_guide`、`test_co_contribution_items_warn_when_global_pending_ack`、`test_co_contribution_items_not_enabled_when_project_explicit_false`、`test_co_contribution_items_enabled_when_project_explicit_true`、`test_co_contribution_items_not_enabled_when_global_true_but_project_explicit_false`。

- [ ] **Step 5: 全量编译验证**

```bash
cargo build -p gitflow-cli 2>&1 | tee /tmp/task3-build.log
```

Expected: 编译成功，0 错误。这是本计划中第一次要求整个 `apps/cli` crate 干净编译的检查点——如果还有报错，说明 Task 1/2/3 中某处遗漏，需回头修复，不得带着编译错误进入 Task 4。

- [ ] **Step 6: 运行 doctor.rs 单元测试**

```bash
cargo test -p gitflow-cli --lib commands::doctor:: 2>&1 | tail -40
```

Expected: 全部通过。

---

### Task 4: 删除 hook 脚本、`gf-autoreport-bug` skill 与参数文档

**Files:**
- Delete: `hooks/auto-report-bug.sh`
- Delete: `skills/gf-autoreport-bug/` (整个目录)
- Delete: `docs/references/gf-autoreport-bug-params.md`

**Interfaces:**
- Consumes: Task 2 已删除 `install_hook`/`uninstall_hook`（唯一通过 `include_bytes!("../../hooks/auto-report-bug.sh")` 引用该脚本的代码路径），本任务删除脚本文件不会产生新的编译错误。
- Produces: 无遗留的 hook 脚本、无遗留的 `gf-autoreport-bug` skill 目录、无遗留的该 skill 专属参数文档。

- [ ] **Step 1: 确认脚本已无代码引用**

```bash
grep -rn "auto-report-bug.sh" apps/cli/src/ 2>/dev/null
```

Expected: 无输出（Task 2 已删除唯一的 `include_bytes!` 引用点）。若有残留，先回到 Task 2 补删。

- [ ] **Step 2: 删除脚本与 skill 目录**

```bash
rm -f hooks/auto-report-bug.sh
rm -rf skills/gf-autoreport-bug/
rm -f docs/references/gf-autoreport-bug-params.md
```

- [ ] **Step 3: 检查 `hooks/tests/` 中是否有针对该脚本的测试**

```bash
grep -rln "auto-report-bug" hooks/tests/ 2>/dev/null
```

若有匹配文件，删除文件中专门测试该脚本的用例；若整份文件只测这一个脚本，整份删除。若无匹配，跳过。

- [ ] **Step 4: 编译 + 全仓库搜索验证**

```bash
cargo build -p gitflow-cli
grep -rn "auto-report-bug\|gf-autoreport-bug" apps/ hooks/ skills/ docs/references/ 2>/dev/null
```

Expected: `cargo build` 成功；`grep` 除本计划 Task 5 要清理的现状类文档（README/architecture/integration-guide/index/4 个 SKILL.md）与 `docs/superpowers/{plans,specs,research}` 历史快照外，无其他匹配。

---

### Task 5: 清理现状类文档中的引用

**Files:**
- Modify: `README.md`
- Modify: `docs/architecture.md`
- Modify: `docs/integration-guide.md`
- Modify: `docs/index.md`
- Modify: `skills/gf-issue-create/SKILL.md`
- Modify: `skills/gf-issue/SKILL.md`
- Modify: `skills/gf-regression/SKILL.md`
- Modify: `skills/gf-security-check/SKILL.md`

**Interfaces:**
- Consumes: Task 4 完成，`gf-autoreport-bug` 已不存在于代码库。
- Produces: 现状类文档不再提及已删除的功能；`docs/superpowers/{plans,specs,research}` 下的历史文档保持不动。

- [ ] **Step 1: `README.md`**

第 97 行的辅助 skill 表格行：

```
| 辅助 | `gf-security-check` / `gf-precommit` / `gf-regression` / `gf-repo-onboarding` / `gf-autoreport-bug` | 安全审计 / 预提交 / 回归 / 入门 / 自动报障 |
```

改为（去掉 `gf-autoreport-bug` 及对应的"自动报障"）：

```
| 辅助 | `gf-security-check` / `gf-precommit` / `gf-regression` / `gf-repo-onboarding` | 安全审计 / 预提交 / 回归 / 入门 |
```

- [ ] **Step 2: `docs/architecture.md`**

第 130 行：

```
26+ AI agent skills (`gf-*` commands) extend the CLI by invoking `gf` subcommands via shell. Git hooks (`auto-report-bug`, `pre-commit`) integrate with the command pipeline.
```

改为（去掉已删除的 hook）：

```
26+ AI agent skills (`gf-*` commands) extend the CLI by invoking `gf` subcommands via shell. Git hooks (`pre-commit`) integrate with the command pipeline.
```

- [ ] **Step 3: `docs/integration-guide.md`**

该文档整节围绕 auto-report-bug 的 Stop Hook 流程展开（第 29、198、204、247、258、273、339、359 行及其上下文）。由于该功能已整体删除，删除整个描述 auto-report-bug Stop Hook 流程的章节（从该章节标题开始，到下一个同级标题之前结束），而不是逐行摘除——保留文档中与该功能无关的其余集成说明段落。执行前先读取该文件全文，确认章节边界后再删除，避免误删相邻的无关章节。

- [ ] **Step 4: `docs/index.md`**

第 42 行的参数文档清单，删除 `` `gf-autoreport-bug-params.md` `` 这一项（该文件已在 Task 4 删除）：

```
- [Skill Parameter References](./references/) — CLI parameter docs and reusable checklists consumed by skills: `gf-pr-params.md`, `gf-label-milestone-params.md`, `gf-label-stats-taxonomy.md`, `gf-pipeline-analyzer-params.md`, `gf-precommit-params.md`, `gf-precommit-hook-template.md`, `gf-quality-params.md`, `gf-release-helper-params.md`, `pr-review-checklist.md`.
```

第 31 行、第 68 行指向 `docs/superpowers/specs/2026-08-30-autoreport-bug-hardening-design.md`、`docs/superpowers/plans/2026-08-30-autoreport-bug-hardening.md`、`2026-08-18-autoreport-bug-multi-role-eval-report.md` 的条目——这些链接指向的是历史快照文档本身（未删除），**保留不动**，`docs/index.md` 作为索引如实反映历史记录的存在。

- [ ] **Step 5: `skills/gf-issue-create/SKILL.md`**

第 42 行的表格行：

```
| Automated bug reporting from CLI errors | This skill requires manual input, not automated detection | `/gf-autoreport-bug` for automated `pending.json` processing |
```

整行删除（该场景已不存在对应能力，不需要保留"何时不用"的引导）。

- [ ] **Step 6: `skills/gf-issue/SKILL.md`**

第 51 行的表格行，同上整行删除：

```
| Automated bug reporting | This skill requires manual command invocation | `/gf-autoreport-bug` for automated `pending.json` processing |
```

第 184 行的列表项整行删除：

```
- `gf-autoreport-bug` — auto-create from CLI error
```

- [ ] **Step 7: `skills/gf-security-check/SKILL.md`**

第 107 行列表项删除：

```
- Reporting vulns to Issue — `/gf-autoreport-bug`
```

第 176 行 See Also 条目删除：

```
- `/gf-autoreport-bug` — file vuln as Issue
```

- [ ] **Step 8: `skills/gf-regression/SKILL.md`** — 行为级修改，不只是删引用

该 skill 的核心职责描述是"解析冒烟测试结果并委托给 `/gf-autoreport-bug` 自动建 Issue"。`/gf-autoreport-bug` 已删除，这条委托路径不再存在，必须把"自动委托建 Issue"改为"分类展示失败、提示用户可手动 `gf issue create`"，而不是简单删字符串。逐处修改：

- 顶部描述（第 10 行）：`Runs \`scripts/smoke-test.sh\`, parses PASS/FAIL/SKIP, delegates real failures to \`/gf-autoreport-bug\`.` → `Runs \`scripts/smoke-test.sh\`, parses PASS/FAIL/SKIP, classifies real failures and surfaces them for the user to file manually via \`gf issue create\`.`
- 第 61、68、78、103、121、137、141、149、157、173、227、339、350 行：所有 `autoreport`/`autoreport-bug`/`/gf-autoreport-bug` 提法统一改为"分类展示 + 提示手动 `gf issue create`"的表述；凡是原文描述"调用 `/gf-autoreport-bug`"的具体步骤，改为"将分类结果（错误类型、命令、日志片段）整理成 Markdown 摘要，提示用户是否要据此手动创建 Issue"。
- 第 149 行 `Fixing bugs — autoreport-bug reports only` 改为 `Fixing bugs — this skill only reports, never fixes`。
- 第 173 行 `🚩 CI + autoreport — Refuse; CI uses exit code only` 改为 `🚩 CI + auto-filing — Refuse; CI uses exit code only`（CI 场景下依旧不产生任何自动上报行为，这条红线本身不变，只是不再提旧命令名）。

修改后通读全文，确认不再有任何 `/gf-autoreport-bug` 或 `pending.json` 委托流程的残留提法，且该 skill 描述的其余职责（`--read-only` 默认值、不修复 bug、不改脚本、不碰远端）保持不变。

- [ ] **Step 9: 全仓库残留引用扫描**

```bash
grep -rln "auto-report-bug\|gf-autoreport-bug\|共建计划\|co_contribution" \
  --include="*.md" --include="*.rs" --include="*.sh" . 2>/dev/null \
  | grep -v "docs/superpowers/plans\|docs/superpowers/specs\|docs/superpowers/research\|docs/superpowers/tests\|docs/research\|docs/code-review-report\|docs/pipeline-analysis-report\|docs/reviews\|docs/index.md\|target/"
```

Expected: 无输出。`docs/index.md` 单独排除是因为 Step 4 中保留了指向历史文档的链接（这些链接文本本身包含关键字，属于预期保留）。

---

### Task 6: 最终全量验证

**Files:** 无新增/修改，仅验证。

**Interfaces:**
- Consumes: Task 1-5 全部完成。
- Produces: 确认整个改动集在完整质量门禁下通过。

- [ ] **Step 1: 完整构建**

```bash
make build
```

Expected: 成功，0 错误 0 警告。

- [ ] **Step 2: 完整测试**

```bash
make test
```

Expected: 全部测试通过，无因删除产生的悬空测试引用。

- [ ] **Step 3: 格式化检查**

```bash
make fmt
```

Expected: 无格式差异（若有差异，是本计划编辑引入的空行/缩进问题，直接应用格式化结果）。

- [ ] **Step 4: Clippy**

```bash
make clippy
```

Expected: 通过 `-D warnings`。特别关注：Task 2 删除多个函数后，`skills.rs` 顶部是否有变为未使用的 `use` 语句（如 `ArgAction` 是否仍被其他字段使用、`dirs` crate 是否仍被使用）——若 clippy/编译报 unused import，直接删除对应 `use` 行。

- [ ] **Step 5: 人工浏览确认 Issue 验收标准逐条满足**

对照 https://github.com/byx-darwin/gitflow-cli/issues/278 的 6 条 Acceptance Criteria 逐条核对，全部满足后本计划视为完成（不在此处打勾 Issue 本身，交付阶段由 gf-workflow Phase 3/4 处理）。

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-09-01-remove-autoreport-bug.md`. Two execution options:

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

**Which approach?**
