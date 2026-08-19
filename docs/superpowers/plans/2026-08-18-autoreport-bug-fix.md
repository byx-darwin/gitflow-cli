# 修复主动上报 bug 功能（h2 升级 + P0 修复）实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复「主动上报 bug」功能 4 项 P0 阻断问题，使功能端到端可交付（用户输入错误不再上报；真实缺陷可被确定性触发并创建 Issue）。

**Architecture:** 4 个独立任务按「依赖/配置 → hook → Rust 核心」顺序推进。T1（h2 升级）与 T2（label 创建）为环境级修复；T3（触发确定性）改 hook + settings + 文档 + bats；T4（错误分类）用 miette code 标记用户输入错误、main.rs 分类、error_reporter 过滤，走完整 TDD。

**Tech Stack:** Rust 2024 + miette 7 + thiserror · bash（Stop Hook）· `gf` CLI · GitHub label API · bats 测试

**Spec:** `docs/superpowers/specs/2026-08-18-autoreport-bug-fix-design.md`（随计划传递，执行者需同时阅读）

## Global Constraints

- **不修改 `deny.toml`**（h2 用升级解决，禁止改动 deny 策略；涉及 `allow`/`deny`/`wildcards` 变更需用户授权）。
- 不实施 P1/P2 建议（去重粒度、co_contribution 可发现性、pending 多报告队列、品牌统一等）——本计划仅 P0。
- `error_reporter.rs` 是同步模块，`maybe_report_error` 签名 `(command: &str, platform: &str, error_message: &str, error_code: &str) -> std::io::Result<()>` 保持兼容，不改调用方签名。
- 新 `UserInputError` 只用于 CLI 内部，不进公共 API。
- 所有 Rust 变更须过 `make test` + `cargo clippy --all-targets --all-features -- -D warnings`。
- Skill 源在 `skills/`，本次不涉及 skill 文件。

---

### Task 1: h2 漏洞升级

**Files:**
- Modify: `Cargo.lock`（由 `cargo update` 生成）

**Interfaces:**
- Consumes: 无（独立）
- Produces: `h2` 锁至 0.4.16+，消除 RUSTSEC-2026-0258

- [ ] **Step 1: 升级 h2**

```bash
cargo update -p h2
```

- [ ] **Step 2: 验证 h2 版本已 ≥ 0.4.16**

```bash
grep -A2 'name = "h2"' Cargo.lock
# 期望 version = "0.4.16" 或更高
```

- [ ] **Step 3: 验证 advisory 不再报警**

```bash
cargo deny check advisories 2>&1 | grep -i "h2" && echo "❌ 仍有 h2 报警" || echo "✅ h2 无报警"
```

- [ ] **Step 4: 验证构建**

```bash
make build
```

- [ ] **Step 5: 提交**

```bash
git add Cargo.lock
git commit -m "fix(deps): upgrade h2 to 0.4.16+ (RUSTSEC-2026-0258)"
```

---

### Task 2: 创建 auto-report label

**Files:**
- Modify: GitHub 仓库配置（`byx-darwin/gitflow-cli`，非本地文件）

**Interfaces:**
- Consumes: 无（独立）
- Produces: 仓库存在 `auto-report` label，消除 `gf issue create --label auto-report` 的 422

- [ ] **Step 1: 创建 label**

```bash
gf label create --color d73a4a auto-report --output json
```

- [ ] **Step 2: 验证 label 存在**

```bash
gf label list 2>&1 | grep "auto-report"
# 期望命中，输出存在证明
```

- [ ] **Step 3: 提交说明**

本任务无本地文件变更（远程仓库配置）。在 commit message 中说明：
```bash
git commit --allow-empty -m "chore(labels): create auto-report label on byx-darwin/gitflow-cli"
```

---

### Task 3: 触发确定性（hook + settings + 文档 + bats）

**Files:**
- Modify: `hooks/auto-report-bug.sh:11,133`（技能名 `gitflow-autoreport-bug` → `gf-autoreport-bug` + banner 指令强化）
- Modify: `.claude/settings.json`（Stop matcher `"gitflow"` → `"gf"`）
- Modify: `docs/integration-guide.md:254,272`（matcher 说明同步）
- Test: `hooks/tests/auto-report-bug.bats`（banner 技能名断言）

**Interfaces:**
- Consumes: Task 1（无依赖，可并行）
- Produces: hook banner 引用正确技能名 + 强制指令 + matcher 兼容 gf

- [ ] **Step 1: 写失败测试（bats 断言 banner 含 gf-autoreport-bug）**

在 `hooks/tests/auto-report-bug.bats` 末尾追加：

```bash
@test "auth success -> banner references gf-autoreport-bug (not gitflow-)" {
  write_pending
  GH_AUTH_STATUS="ok"
  run_hook

  [ "$status" -eq 0 ]
  echo "$output" | grep -q "gf-autoreport-bug"
  # 确保过时技能名不再出现
  if echo "$output" | grep -q "gitflow-autoreport-bug"; then
    echo "❌ banner still references stale gitflow-autoreport-bug" >&2
    return 1
  fi
}
```

- [ ] **Step 2: 运行测试验证失败**

```bash
bats hooks/tests/auto-report-bug.bats
# 期望新增用例 FAIL（当前 banner 含 gitflow-autoreport-bug）
```

- [ ] **Step 3: 修改 hook 技能名 + 强化指令**

编辑 `hooks/auto-report-bug.sh`：
- 行 11 注释：`gitflow-autoreport-bug` → `gf-autoreport-bug`
- 行 133 banner 文本：`请加载 gitflow-autoreport-bug Skill 执行自动 Bug 报告流程。` →
  `MUST load the gf-autoreport-bug skill now to process this error report.`
- 行 134 路径保持 `$CLAUDE_DIR/skills/gf-autoreport-bug/SKILL.md`（已正确）

- [ ] **Step 4: 更新 Stop matcher**

编辑 `.claude/settings.json`：`"matcher": "gitflow"` → `"matcher": "gf"`

- [ ] **Step 5: 同步文档**

编辑 `docs/integration-guide.md:254,272`：matcher 值 `"gitflow"` → `"gf"`，说明改为「gf 相关会话触发」。

- [ ] **Step 6: 运行测试验证通过**

```bash
bats hooks/tests/auto-report-bug.bats
# 期望全部通过（含新增用例）
```

- [ ] **Step 7: 提交**

```bash
git add hooks/auto-report-bug.sh .claude/settings.json docs/integration-guide.md hooks/tests/auto-report-bug.bats
git commit -m "fix(hook): reference gf-autoreport-bug skill, strengthen trigger directive, matcher gf"
```

---

### Task 4: 写入端错误分类（Rust + TDD）

**Files:**
- Create: `apps/cli/src/errors.rs`（`UserInputError`，miette `code = "gf::user_input"`）
- Modify: `apps/cli/src/error_reporter.rs`（`maybe_report_error` 过滤 `USER_INPUT_ERROR`）
- Modify: `apps/cli/src/main.rs:144`（按 miette code 分类 error_code）
- Modify: `apps/cli/src/commands/issue.rs:210`（state 校验抛 `UserInputError`）
- Modify: `apps/cli/src/commands/pr.rs:259`（state 校验抛 `UserInputError`）
- Modify: `apps/cli/src/main.rs`（`mod errors;` 声明）
- Test: `apps/cli/src/error_reporter.rs` 测试模块（过滤测试）

**Interfaces:**
- Consumes: 无（独立于 T1-T3）
- Produces:
  - `UserInputError::new(message: impl Into<String>) -> Self`（`pub(crate)`，在 `crate::errors`）
  - `maybe_report_error` 在 `error_code == "USER_INPUT_ERROR"` 时返回 `Ok(())` 不落盘
  - `main.rs` 顶层错误处理：miette code 为 `gf::user_input` → `error_code = "USER_INPUT_ERROR"`，否则 `"CLI_ERROR"`
  - issue.rs/pr.rs 的 state 解析错误类型从 `miette::miette!` 改为 `UserInputError`

- [ ] **Step 1: 写失败测试（error_reporter 过滤）**

在 `apps/cli/src/error_reporter.rs` 测试模块末尾追加：

```rust
#[test]
fn test_should_skip_user_input_error() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let result = maybe_report_error("issue list", "github", "Invalid state 'x'", "USER_INPUT_ERROR");
    assert!(result.is_ok(), "user input errors must be silently skipped");
    let pending = tmp.path().join(".cache/bug-reports/pending.json");
    assert!(!pending.exists(), "user input errors must not write pending.json");
}
```

> 注：`maybe_report_error` 使用 `find_repo_root()`（git 定位 repo root），测试需在 git 仓库内运行；该测试置于现有测试模块，验证 `USER_INPUT_ERROR` 分支短路逻辑。如 `find_repo_root` 在测试环境可用，保留；否则用 tempdir 模拟 repo_root 路径并调整断言方式（见 Step 3 实现说明）。

- [ ] **Step 2: 运行测试验证失败**

```bash
cargo test -p gitflow-cli error_reporter 2>&1 | tail -20
# 期望：当前实现会写入 pending.json（或 find_repo_root 失败），新增用例 FAIL
```

- [ ] **Step 3: 创建 UserInputError 类型**

新建 `apps/cli/src/errors.rs`：

```rust
//! CLI 内部错误分类。
//!
//! 区分「用户输入/参数错误」与「真实运行缺陷」，供主动上报 bug 功能过滤误报。

use miette::Diagnostic;
use thiserror::Error;

/// 用户输入/参数校验错误。
///
/// 携带 miette code `gf::user_input`，供 `main.rs` 顶层分类识别。
/// 此类错误不会被主动上报（避免把用户传参错误当 bug 上报）。
#[derive(Debug, Error, Diagnostic)]
#[error("{message}")]
#[diagnostic(code = "gf::user_input")]
pub(crate) struct UserInputError {
    message: String,
}

impl UserInputError {
    /// 构造用户输入错误。
    #[must_use]
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}
```

在 `apps/cli/src/main.rs` 顶部模块声明区添加：

```rust
mod errors;
```

- [ ] **Step 4: 修改 error_reporter 过滤逻辑**

编辑 `apps/cli/src/error_reporter.rs::maybe_report_error`（行 175-193），在 `is_co_contribution_enabled()` 检查之后、构造 report 之前插入：

```rust
    // User input/argument errors are not bugs — do not report them.
    if error_code == "USER_INPUT_ERROR" {
        return Ok(());
    }
```

- [ ] **Step 5: 修改 main.rs 顶层错误分类**

编辑 `apps/cli/src/main.rs:144`，将：

```rust
            if platform_needed {
                report_error_noninteractive(&command_name, &platform, &e.to_string(), "CLI_ERROR");
            }
```

改为：

```rust
            if platform_needed {
                let error_code = if e
                    .code()
                    .is_some_and(|c| c.to_string() == "gf::user_input")
                {
                    "USER_INPUT_ERROR"
                } else {
                    "CLI_ERROR"
                };
                report_error_noninteractive(&command_name, &platform, &e.to_string(), error_code);
            }
```

> `miette::Report::code()` 返回 `Option<&dyn DiagnosticCode>`，`.to_string()` 得到 code 字符串。

- [ ] **Step 6: 修改 issue.rs state 校验**

编辑 `apps/cli/src/commands/issue.rs:210`，将：

```rust
                    other => Err(miette::miette!(
                        "Invalid state '{other}'. Expected 'open', 'closed', or 'all'."
                    )),
```

改为：

```rust
                    other => Err(crate::errors::UserInputError::new(format!(
                        "Invalid state '{other}'. Expected 'open', 'closed', or 'all'."
                    ))),
```

> `async_main` 返回 `miette::Result<()>`，`UserInputError` 实现 `Diagnostic`，`?` 可自动转换（miette 7 支持自定义 `Diagnostic` 类型直接作为错误返回）。

- [ ] **Step 7: 修改 pr.rs state 校验**

编辑 `apps/cli/src/commands/pr.rs:259`，将同样的 `miette::miette!("Invalid state...")` 改为 `crate::errors::UserInputError::new(format!(...))`（文本与 issue.rs 完全一致）。

- [ ] **Step 8: 运行测试验证通过**

```bash
make test
cargo clippy --all-targets --all-features -- -D warnings
```

期望：全部通过，clippy 干净。

- [ ] **Step 9: 端到端验证错误分类**

```bash
# 模拟用户输入错误：传非法 state，应不产生 pending.json
gf issue list --state invalid >/dev/null 2>&1; echo "exit=$?"
ls .cache/bug-reports/pending.json 2>/dev/null && echo "❌ 误写 pending.json" || echo "✅ 用户输入错误未写 pending.json"

# 确认现有 pending.json（上轮积压的误报）可清理
rm -f .cache/bug-reports/pending.json
```

- [ ] **Step 10: 提交**

```bash
git add apps/cli/src/errors.rs apps/cli/src/error_reporter.rs apps/cli/src/main.rs apps/cli/src/commands/issue.rs apps/cli/src/commands/pr.rs
git commit -m "fix(cli): classify user input errors as USER_INPUT_ERROR, skip auto-report"
```

---

## Self-Review（写后自检）

**Spec 覆盖：**
- T1 → §2 变更面 `Cargo.lock` · §3.1 ✅
- T2 → §2 变更面 GitHub label · §3.2 ✅
- T3 → §2 变更面 hook/settings/文档/bats · §3.3 ✅
- T4 → §2 变更面 errors.rs/error_reporter.rs/main.rs/issue.rs/pr.rs · §3.4 ✅
- 验收标准：AC1→T1S3 · AC2→T2S2 · AC3→T3S2/S4/S6 · AC4→T4S8/S9 · AC5→各任务 Step 8/6 ✅
- 范围外：无 P1/P2、不改 deny.toml（Global Constraints）✅

**占位符扫描：** 无 TBD/TODO；T4 的 `UserInputError::new` 签名、`maybe_report_error` 过滤条件、miette code 字符串在 T4 Step 3-6 内统一定义，无跨任务悬空引用。

**类型一致性：** `UserInputError`（`crate::errors`）在 Step 3 定义、Step 6/7 消费；`USER_INPUT_ERROR` 常量在 Step 4 过滤、Step 5 生成；`gf::user_input` code 在 Step 3 标注、Step 5 匹配——三处一致。

**测试注意：** T4 Step 1 的测试依赖 `find_repo_root` 在测试环境可用（测试在 git 仓库内运行，`tempfile` 不会改变 cwd）。若 CI 沙箱无 git 上下文，Step 3 实现时调整测试为直接构造 repo_root 传入（与现有 `test_should_write_pending_json_to_disk` 一致的方式：直接调 `write_to_disk(tmp)` 验证过滤逻辑抽出的函数）。本计划 Step 8 的 `make test` 覆盖。
