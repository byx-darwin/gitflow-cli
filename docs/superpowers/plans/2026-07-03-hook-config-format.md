# Hook 配置格式修复与 Bug 上报可配置化 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复 `gitflow skills install` 生成的 Stop Hook 配置格式，并对 bug 上报提供 install 时开关。

**Architecture:** 直接修改 `merge_stop_hook()` 的 JSON 字面量为嵌套结构；在 `install.sh` 提取公共变量消除三处重复；新增 `--report-bug` flag（默认 true）控制是否注册 hook。`uninstall_hook` 匹配逻辑不变（顶层 matcher 保留）。

**Tech Stack:** Rust 2024、serde_json、clap (derive)、Bash

## Global Constraints

- Rust 2024 + pinned toolchain in `rust-toolchain.toml`
- `#![forbid(unsafe_code)]` at crate root
- 所有公开的 Rust 项必须有文档注释
- 测试命名：`test_should_<expected_behavior>` 或描述性名称
- 不允许 `unwrap()` / `expect()` 出现在生产代码
- `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic` 必须无警告
- 不运行 `cargo clean`
- 不 commit/push/merge/发布/部署/修改 ticket 状态，除非用户明确许可

---

## 文件结构

| 文件 | 责任 |
|------|------|
| `apps/cli/src/commands/skills.rs` | 核心逻辑：`merge_stop_hook` 格式修正、`--report-bug` flag、install 时条件调用、单元测试 |
| `scripts/install.sh` | 提取 `HOOK_CONFIG` 常量，三处引用替换为嵌套格式 |

无新文件创建。

---

## Task 1: 为 `merge_stop_hook` 添加失败测试（TDD — RED）

**Files:**
- Modify: `apps/cli/src/commands/skills.rs::tests` 模块末尾

**Interfaces:**
- Consumes: `merge_stop_hook` (private fn, 同模块测试可见)
- Produces: 3 个失败测试，描述新格式预期

- [ ] **Step 1: 添加 `test_merge_stop_hook_creates_nested_format` 测试**

在 `mod tests` 末尾追加：

```rust
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
        hooks[0].get("type").and_then(serde_json::Value::as_str()),
        Some("command")
    );
    assert_eq!(
        hooks[0].get("command").and_then(serde_json::Value::as_str()),
        Some("bash hooks/auto-report-bug.sh")
    );
    assert_eq!(
        result
            .pointer("/hooks/Stop/0/matcher")
            .and_then(serde_json::Value::as_str()),
        Some("gitflow")
    );
}
```

- [ ] **Step 2: 添加 `test_merge_stop_hook_replaces_existing_gitflow` 测试**

```rust
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
```

- [ ] **Step 3: 添加 `test_merge_stop_hook_preserves_other_hooks` 测试**

```rust
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
```

- [ ] **Step 4: 运行测试确认失败**

```bash
cargo test -p gitflow-cli --lib -- commands::skills::tests::test_merge_stop_hook
```

Expected: 3 个测试 FAIL — `merge_stop_hook` 当前仍生成扁平格式，`hooks` 数组不存在。

- [ ] **Step 5: 不修改代码，仅提交测试（保持 RED 状态）**

测试已写好，进入 Task 2 实现后才会 GREEN。为避免工作丢失，此处先提交：

```bash
git add apps/cli/src/commands/skills.rs
git commit -m "test: add failing tests for nested hook format"
```

---

## Task 2: 修复 `merge_stop_hook` 格式（TDD — GREEN）

**Files:**
- Modify: `apps/cli/src/commands/skills.rs:329-362` (`merge_stop_hook` 函数)

**Interfaces:**
- Consumes: `cmd: &str`
- Produces: JSON 值包含嵌套 `hooks` 数组

- [ ] **Step 1: 修改 hook 字面量**

将 `merge_stop_hook` 函数中的这段：

```rust
let hook = serde_json::json!({
    "matcher": "gitflow",
    "command": cmd
});
```

改为：

```rust
let hook = serde_json::json!({
    "matcher": "gitflow",
    "hooks": [
        {
            "type": "command",
            "command": cmd
        }
    ]
});
```

- [ ] **Step 2: 验证 fallback 分支无需改动**

确认第 354-358 行：

```rust
json = serde_json::json!({
    "hooks": {
        "Stop": [hook]
    }
});
```

此分支使用 `hook` 变量，格式自动跟随新结构，无需额外修改。

- [ ] **Step 3: 运行 Task 1 的测试确认通过**

```bash
cargo test -p gitflow-cli --lib -- commands::skills::tests::test_merge_stop_hook
```

Expected: 3 个测试 PASS。

- [ ] **Step 4: 运行 clippy 确认无新警告**

```bash
cargo clippy -p gitflow-cli --all-targets --all-features -- -D warnings -W clippy::pedantic
```

Expected: 无警告，无错别字，无格式问题。如 clippy 报 `ignored_unit_patterns` 或其他 lint，按提示加 `#[allow(...)]` 并注明 reason。

- [ ] **Step 5: 提交实现**

```bash
git add apps/cli/src/commands/skills.rs
git commit -m "fix: use nested hooks format for Stop Hook config

Aligns with Claude Code official schema:
{\"matcher\": ..., \"hooks\": [{\"type\": \"command\", \"command\": ...}]}"
```

---

## Task 3: 为 `uninstall_hook` 添加失败测试（TDD — RED）

**Files:**
- Modify: `apps/cli/src/commands/skills.rs::tests` 模块末尾

**Interfaces:**
- Consumes: `uninstall_hook` (private fn) — 注意此函数会读写文件系统，需用临时目录隔离
- Produces: 2 个失败测试，验证对新格式的兼容性

- [ ] **Step 1: 确认 `uninstall_hook` 函数签名和文件路径依赖**

当前 `uninstall_hook(global: bool)` 内部自己解析 settings 路径（`~/.claude/settings.json` 或项目级）。测试时会写真实文件到临时 home。

- [ ] **Step 2: 添加 `test_uninstall_hook_removes_gitflow` 测试**

```rust
#[test]
fn test_uninstall_hook_removes_gitflow() {
    // 用临时目录隔离，避免污染真实 HOME
    let tmp = tempfile::tempdir().expect("create temp dir");
    let prev_home = std::env::var("HOME").ok();
    std::env::set_var("HOME", tmp.path());

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

    // 调用 uninstall_hook（全局模式）
    super::uninstall_hook(true).expect("uninstall should succeed");

    // 验证 gitflow hook 已被删除
    let after = std::fs::read_to_string(&settings_path).expect("read after");
    let parsed: serde_json::Value =
        serde_json::from_str(&after).expect("parse after");
    let stop = parsed
        .pointer("/hooks/Stop")
        .and_then(serde_json::Value::as_array)
        .expect("Stop should exist");
    assert!(
        stop.iter()
            .all(|v| v.get("matcher").and_then(serde_json::Value::as_str) != Some("gitflow")),
        "gitflow hook should be removed"
    );

    // 恢复 HOME
    if let Some(prev) = prev_home {
        std::env::set_var("HOME", prev);
    }
}
```

- [ ] **Step 3: 添加 `test_uninstall_hook_preserves_others` 测试**

```rust
#[test]
fn test_uninstall_hook_preserves_others() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let prev_home = std::env::var("HOME").ok();
    std::env::set_var("HOME", tmp.path());

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

    super::uninstall_hook(true).expect("uninstall should succeed");

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

    if let Some(prev) = prev_home {
        std::env::set_var("HOME", prev);
    }
}
```

- [ ] **Step 4: 运行测试确认通过**

因为 `uninstall_hook` 原本就读 `v.get("matcher")`，新格式仍保留顶层 matcher，这两个测试应该**直接通过**（无需额外实现改动）。

```bash
cargo test -p gitflow-cli --lib -- commands::skills::tests::test_uninstall_hook
```

Expected: 2 个测试 PASS — 证明新格式向后兼容。

- [ ] **Step 5: 提交**

```bash
git add apps/cli/src/commands/skills.rs
git commit -m "test: verify uninstall_hook works with nested hook format"
```

---

## Task 4: 添加 `--report-bug` Flag 与 install 条件调用

**Files:**
- Modify: `apps/cli/src/commands/skills.rs:106-122` (InstallArgs)
- Modify: `apps/cli/src/commands/skills.rs:212-270` (install_skills)

**Interfaces:**
- Consumes: CLI args
- Produces: `report_bug: bool` 字段（默认 true），控制 `install_hook` 是否被调用

- [ ] **Step 1: 添加 flag 到 `InstallArgs`**

在 `pub force: bool,` 之后加：

```rust
    /// 启用自动 bug 上报（Stop Hook），默认开启
    #[arg(long, default_value_t = true, action = ArgAction::SetTrue)]
    pub report_bug: bool,
```

- [ ] **Step 2: 在 `install_skills` 中添加条件调用**

将这段（约第 267-268 行）：

```rust
    // 安装 auto-report-bug hook
    install_hook(args.global, args.force)?;
```

改为：

```rust
    // 安装 auto-report-bug hook（可通过 --report-bug=false 跳过）
    if args.report_bug {
        install_hook(args.global, args.force)?;
    }
```

- [ ] **Step 3: 运行测试确认现有功能不回归**

```bash
cargo test -p gitflow-cli --lib -- commands::skills::tests
```

Expected: 全部通过。`report_bug` 默认 true，不影响现有调用。

- [ ] **Step 4: 运行 clippy**

```bash
cargo clippy -p gitflow-cli --all-targets --all-features -- -D warnings -W clippy::pedantic
```

Expected: 无警告。

- [ ] **Step 5: 提交**

```bash
git add apps/cli/src/commands/skills.rs
git commit -m "feat: add --report-bug flag to toggle Stop Hook installation

Default true for backward compatibility; --report-bug=false skips
hook registration."
```

---

## Task 5: 重构 `scripts/install.sh` — 提取 HOOK_CONFIG 公共变量

**Files:**
- Modify: `scripts/install.sh` 常量区（约第 35 行后）
- Modify: `scripts/install.sh` 第 439 行提示输出
- Modify: `scripts/install.sh` 第 451-460 行 MERGE_EOF
- Modify: `scripts/install.sh` 第 465-476 行 SETTINGS_EOF

**Interfaces:**
- Produces: `HOOK_CONFIG` 常量，三处引用

- [ ] **Step 1: 在常量区末尾追加 HOOK_CONFIG**

在 `readonly SETTINGS_FILE=.claude/settings.json` 后加空行，再加：

```bash
# 嵌套 Stop Hook 配置（对齐 Claude Code 官方 schema）
# matcher 在顶层，hooks 数组包含 type+command 对象
readonly HOOK_CONFIG='{"matcher": "gitflow", "hooks": [{"type": "command", "command": "bash hooks/auto-report-bug.sh"}]}'
```

- [ ] **Step 2: 替换第 439 行单行提示**

将：

```bash
echo '  {"matcher": "gitflow", "command": "bash hooks/auto-report-bug.sh"}'
```

改为：

```bash
echo "  ${HOOK_CONFIG}"
```

- [ ] **Step 3: 替换第 451-460 行 MERGE_EOF heredoc**

将：

```bash
cat <<'MERGE_EOF'
    "hooks": {
      "Stop": [
        {
          "matcher": "gitflow",
          "command": "bash hooks/auto-report-bug.sh"
        }
      ]
    }
MERGE_EOF
```

改为：

```bash
cat <<MERGE_EOF
    "hooks": {
      "Stop": [
        ${HOOK_CONFIG}
      ]
    }
MERGE_EOF
```

注意：去掉了单引号（`'MERGE_EOF'` → `MERGE_EOF`），这样才能展开 `${HOOK_CONFIG}` 变量。

- [ ] **Step 4: 替换第 465-476 行 SETTINGS_EOF heredoc**

将：

```bash
cat > "$settings_target" <<'SETTINGS_EOF'
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
SETTINGS_EOF
```

改为：

```bash
cat > "$settings_target" <<SETTINGS_EOF
{
  "hooks": {
    "Stop": [
      ${HOOK_CONFIG}
    ]
  }
}
SETTINGS_EOF
```

同样去掉单引号以支持变量展开。

- [ ] **Step 5: 检查语法**

```bash
bash -n scripts/install.sh
```

Expected: 无语法错误。

- [ ] **Step 6: 提交**

```bash
git add scripts/install.sh
git commit -m "refactor: extract HOOK_CONFIG variable and use nested format

Removes 3x duplication of hook JSON; aligns script output with the
nested Claude Code hook schema."
```

---

## Task 6: 最终验证与文档同步

**Files:**
- Verify: `apps/cli/src/commands/skills.rs`
- Verify: `scripts/install.sh`

**Interfaces:**
- Consumes: 所有前述改动
- Produces: 全部测试通过、clippy 干净、bash 语法 OK

- [ ] **Step 1: 运行完整 skills 测试套件**

```bash
cargo test -p gitflow-cli --lib -- commands::skills
```

Expected: 全部通过（含 Task 1-3 新增的 5 个测试 + 原有测试）。

- [ ] **Step 2: 运行 clippy**

```bash
cargo clippy -p gitflow-cli --all-targets --all-features -- -D warnings -W clippy::pedantic
```

Expected: 无警告。

- [ ] **Step 3: 运行 cargo fmt**

```bash
cargo +nightly fmt -p gitflow-cli -- --check
```

Expected: 无格式问题。

- [ ] **Step 4: bash 语法检查**

```bash
bash -n scripts/install.sh
```

Expected: OK。

- [ ] **Step 5: 验证 spec 覆盖**

对照 `docs/superpowers/specs/2026-07-03-hook-config-format-design.md` 逐项：

- [x] `merge_stop_hook` 改为嵌套格式 — Task 2
- [x] fallback 分支同步 — Task 2（自动跟随）
- [x] `install.sh` 三处修复 — Task 5
- [x] `--report-bug` flag 默认 true — Task 4
- [x] install 时条件调用 — Task 4
- [x] uninstall 始终清理 — 默认行为，无需改动
- [x] 5 个新单元测试 — Tasks 1 + 3

---

## 不在范围内

- 不迁移已有 `.claude/settings.json` 中的旧格式配置
- 不改动 `uninstall_hook` 匹配逻辑
- 不为其他 Agent 平台（Codex/Gemini/Copilot）添加 hooks 配置
- 不修改 `apps/cli/src/error_reporter.rs` 或 hook 脚本本身
- 不提交/push/合并，除非用户明确许可

---

## 验证命令速查

```bash
# 运行全部 skills 单元测试
cargo test -p gitflow-cli --lib -- commands::skills

# Clippy 严格检查
cargo clippy -p gitflow-cli --all-targets --all-features -- -D warnings -W clippy::pedantic

# 格式检查
cargo +nightly fmt -p gitflow-cli -- --check

# Bash 语法检查
bash -n scripts/install.sh
```
