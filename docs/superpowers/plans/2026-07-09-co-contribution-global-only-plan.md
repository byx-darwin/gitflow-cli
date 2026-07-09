# 共建计划标识强制全局写入实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 `merge_co_contribution()` 强制改为用户级全局写入，移除 `global` 参数。

**Architecture:** 修改 `merge_co_contribution` 函数签名，移除 `global: bool` 参数，内部直接调用 `resolve_global_hook_paths` 获取全局路径。更新两处调用点和提示文案。

**Tech Stack:** Rust, serde_json, tempdir (测试)

## Global Constraints

- 保留 `platform: AgentPlatform` 参数，支持多平台全局路径
- 不修改 `is_co_contribution_enabled()` 的双位置检查逻辑
- 不迁移已有项目级标记
- 所有新代码必须有单元测试覆盖

---

### Task 1: 编写失败测试 — `merge_co_contribution` 强制全局写入

**Files:**
- Modify: `apps/cli/src/commands/skills.rs:1585` (在 `mod tests` 末尾添加)

**Interfaces:**
- Consumes: `merge_co_contribution(platform: AgentPlatform)` (Task 2 实现)
- Produces: 测试用例验证全局路径写入

- [ ] **Step 1: 编写失败测试**

在 `apps/cli/src/commands/skills.rs` 的 `mod tests` 末尾添加：

```rust
#[test]
fn test_should_write_co_contribution_to_global_path() {
    let temp = tempfile::tempdir().expect("tempdir");
    let home = temp.path();

    // 临时覆盖 HOME 环境变量
    let original_home = std::env::var("HOME").ok();
    std::env::set_var("HOME", home);

    // 调用函数（Task 2 将移除此处的 `false` 参数）
    let result = merge_co_contribution(AgentPlatform::Claude);

    // 恢复 HOME
    match original_home {
        Some(h) => std::env::set_var("HOME", h),
        None => std::env::remove_var("HOME"),
    }

    assert!(result.is_ok(), "merge_co_contribution should succeed");

    // 验证写入全局路径
    let global_settings = home.join(".claude/settings.json");
    assert!(global_settings.exists(), "global settings.json must be created");

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
}
```

- [ ] **Step 2: 运行测试验证失败**

Run: `cargo test -p gitflow-cli test_should_write_co_contribution_to_global_path`
Expected: FAIL — 编译错误，`merge_co_contribution` 需要两个参数但只传了一个

- [ ] **Step 3: 提交失败测试**

```bash
git add apps/cli/src/commands/skills.rs
git commit -m "test(skills): add failing test for global-only co_contribution write"
```

---

### Task 2: 重构 `merge_co_contribution` 签名和实现

**Files:**
- Modify: `apps/cli/src/commands/skills.rs:799-819`

**Interfaces:**
- Consumes: `resolve_global_hook_paths(&Path, AgentPlatform)` (已存在)
- Produces: `merge_co_contribution(platform: AgentPlatform) -> miette::Result<()>`

- [ ] **Step 1: 修改函数签名**

将 `apps/cli/src/commands/skills.rs:799` 的函数签名从：

```rust
fn merge_co_contribution(global: bool, platform: AgentPlatform) -> miette::Result<()> {
    let (_hook_dir, settings_path, _cmd) = resolve_hook_paths(global, platform)?;
```

改为：

```rust
fn merge_co_contribution(platform: AgentPlatform) -> miette::Result<()> {
    let home = dirs::home_dir().ok_or_else(|| miette::miette!("无法确定 HOME 目录"))?;
    let (_hook_dir, settings_path, _cmd) = resolve_global_hook_paths(&home, platform);
```

- [ ] **Step 2: 运行测试验证通过**

Run: `cargo test -p gitflow-cli test_should_write_co_contribution_to_global_path`
Expected: PASS

- [ ] **Step 3: 提交实现**

```bash
git add apps/cli/src/commands/skills.rs
git commit -m "refactor(skills): force global write for co_contribution marker"
```

---

### Task 3: 更新调用点

**Files:**
- Modify: `apps/cli/src/commands/skills.rs:737` 和 `750`

**Interfaces:**
- Consumes: `merge_co_contribution(platform: AgentPlatform)` (Task 2 已实现)
- Produces: 两处调用点更新

- [ ] **Step 1: 更新第一处调用（第 737 行）**

将 `apps/cli/src/commands/skills.rs:737` 从：

```rust
merge_co_contribution(args.global, platform)?;
```

改为：

```rust
merge_co_contribution(platform)?;
```

- [ ] **Step 2: 更新第二处调用（第 750 行）**

将 `apps/cli/src/commands/skills.rs:750` 从：

```rust
merge_co_contribution(args.global, platform)?;
```

改为：

```rust
merge_co_contribution(platform)?;
```

- [ ] **Step 3: 运行编译检查**

Run: `cargo check -p gitflow-cli`
Expected: 编译成功，无错误

- [ ] **Step 4: 提交调用点更新**

```bash
git add apps/cli/src/commands/skills.rs
git commit -m "refactor(skills): update merge_co_contribution call sites"
```

---

### Task 4: 更新提示文案

**Files:**
- Modify: `apps/cli/src/commands/skills.rs:725-726`

**Interfaces:**
- Consumes: 无
- Produces: 用户可见的提示文案变更

- [ ] **Step 1: 重写提示文案**

将 `apps/cli/src/commands/skills.rs:725-726` 从：

```rust
println!("🤝 共建计划：加入后，CLI 错误将自动上报为 GitHub Issue，帮助改进 gitflow-cli。");
println!("   仅非交互模式（Agent/CI）下生效，普通控制台使用不受影响。");
```

改为：

```rust
println!("🤝 共建计划：加入后，CLI 错误将自动上报为 GitHub Issue，帮助改进 gitflow-cli。");
println!("   用户级设置，加入一次即所有项目生效。");
```

- [ ] **Step 2: 运行全量测试**

Run: `cargo test -p gitflow-cli`
Expected: 所有测试通过

- [ ] **Step 3: 提交文案更新**

```bash
git add apps/cli/src/commands/skills.rs
git commit -m "docs(skills): clarify co_contribution is user-level setting"
```

---

### Task 5: 运行质量门禁并提交

**Files:**
- 无新文件

**Interfaces:**
- Consumes: 所有前序任务
- Produces: 质量验证通过

- [ ] **Step 1: 运行 clippy**

Run: `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: 无警告

- [ ] **Step 2: 运行格式化检查**

Run: `cargo +nightly fmt --check`
Expected: 无格式问题

- [ ] **Step 3: 运行完整测试套件**

Run: `cargo test --workspace`
Expected: 所有测试通过

- [ ] **Step 4: 确认改动范围**

Run: `git diff main --stat`
Expected: 仅 `apps/cli/src/commands/skills.rs` 有改动

- [ ] **Step 5: 提交完成标记（如需要）**

如果前序任务已全部提交，此步骤可跳过。

---

## 完成标准

- [ ] `merge_co_contribution` 签名移除 `global` 参数
- [ ] 始终写入全局路径（`~/.claude/settings.json` 或平台对应路径）
- [ ] 提示文案更新为"用户级设置，加入一次即所有项目生效"
- [ ] 新增测试验证全局写入行为
- [ ] 所有测试通过
- [ ] clippy 无警告
- [ ] 格式化通过
