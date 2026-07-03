# Hook 配置格式修复与 Bug 上报可配置化

**日期**：2026-07-03
**状态**：已批准
**范围**：`apps/cli/src/commands/skills.rs`、`scripts/install.sh`

---

## 背景

`gitflow skills install` 注册的 Stop Hook 配置使用了错误的 JSON 格式。当前生成的配置：

```json
{
  "matcher": "gitflow",
  "command": "bash hooks/auto-report-bug.sh"
}
```

Claude Code 官方 schema 要求的正确格式：

```json
{
  "matcher": "gitflow",
  "hooks": [
    {
      "type": "command",
      "command": "bash hooks/auto-report-bug.sh"
    }
  ]
}
```

此外，当前 install 流程**强制**注册 bug 上报 hook，无法跳过。需要增加 install 时开关。

---

## 目标

1. 修复 hook 配置生成格式，对齐 Claude Code 官方 schema
2. 保留 `"type": "command"` 字段，为未来扩展留空间
3. 提供 `--report-bug` flag（默认 `true`），install 时控制是否注册 hook
4. 为 `merge_stop_hook` / `uninstall_hook` 补充单元测试

---

## 设计

### 1. 核心修复 — `skills.rs::merge_stop_hook()`

**改动位置**：`apps/cli/src/commands/skills.rs` 第 329-362 行

**当前代码**：

```rust
let hook = serde_json::json!({
    "matcher": "gitflow",
    "command": cmd
});
```

**改为**：

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

**同步修改 fallback 分支**（第 354-358 行）：

```rust
json = serde_json::json!({
    "hooks": {
        "Stop": [hook]
    }
});
```

此分支已使用 `hook` 变量，格式自动跟随，无需单独改动。

**`uninstall_hook()` 无需修改**：匹配逻辑基于顶层 `matcher` 字段（第 480 行），新格式仍保留顶层 `matcher`，逻辑兼容。

### 2. 脚本修复 — `install.sh`

**问题**：三处重复定义 hook JSON（第 439、456、471 行），DRY 违反。

**方案**：提取公共变量，三处引用。

**在常量区新增**（约第 35 行后）：

```bash
readonly HOOK_CONFIG='{"matcher": "gitflow", "hooks": [{"type": "command", "command": "bash hooks/auto-report-bug.sh"}]}'
```

**三处替换**：

| 位置 | 旧 | 新 |
|------|----|----|
| 第 439 行（单行提示） | `echo '  {"matcher": "gitflow", "command": "..."}'` | `echo "  ${HOOK_CONFIG}"` |
| 第 451-460 行（MERGE_EOF） | 扁平结构 | 嵌套结构引用 `${HOOK_CONFIG}` |
| 第 465-476 行（SETTINGS_EOF） | 扁平结构 | 嵌套结构引用 `${HOOK_CONFIG}` |

### 3. `--report-bug` Flag

**CLI 变更**（`InstallArgs`）：

```rust
/// 启用自动 bug 上报（Stop Hook），默认 true
#[arg(long, default_value_t = true, action = ArgAction::SetTrue)]
pub report_bug: bool,
```

**`install_skills` 调用链调整**：

```rust
// 仅当 report_bug 为 true 时注册 hook
if args.report_bug {
    install_hook(args.global, args.force)?;
}
```

**`uninstall_skills`**：不检查 `report_bug`，始终调用 `uninstall_hook()` 清理。用户可能在安装后改变主意，卸载时应彻底清理。

**行为矩阵**：

| 命令 | 行为 |
|------|------|
| `gitflow skills install` | 安装 hook（默认 true，向后兼容） |
| `gitflow skills install --report-bug` | 安装 hook |
| `gitflow skills install --report-bug=false` | 跳过 hook |

### 4. 测试策略

**`merge_stop_hook` 测试（3 个）**：

1. `test_merge_stop_hook_creates_nested_format` — 空 JSON 输入，验证输出含 `hooks` 数组和 `type: command`
2. `test_merge_stop_hook_replaces_existing_gitflow` — 已有旧格式 gitflow hook，验证被新格式替换
3. `test_merge_stop_hook_preserves_other_hooks` — 存在其他 matcher 的 hook，验证不被影响

**`uninstall_hook` 测试（2 个）**：

4. `test_uninstall_hook_removes_gitflow` — 验证能正确移除新格式的 gitflow hook
5. `test_uninstall_hook_preserves_others` — 验证不影响其他 hook

**测试方式**：用 `serde_json::json!` 构造输入，断言输出 JSON 结构和字段值。

---

## 不改动的部分

- `uninstall_hook()` 匹配逻辑 — 顶层 `matcher` 仍存在，无需改动
- 已有 `.claude/settings.json` 配置 — 不自动迁移，用户手动修复或重装
- 其他 Agent 平台（Codex/Gemini/Copilot）— hooks 是 Claude Code 专属，其他 Agent 不读此配置

---

## 验证方式

1. `cargo test -p gitflow-cli --lib` — 新测试全部通过
2. `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic` — 无新警告
3. 手动运行 `gitflow skills install -f` 后检查 `.claude/settings.json` 格式正确
4. 手动运行 `gitflow skills install --report-bug=false` 验证 hook 未注册

---

## 回滚策略

纯格式修正 + 新增 flag，无数据迁移。回滚即 revert，无副作用。
