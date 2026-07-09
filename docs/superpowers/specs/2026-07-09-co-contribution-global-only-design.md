# 共建计划标识改为用户级设计文档

**日期**: 2026-07-09
**状态**: Draft
**Issue**: #82
**前置设计**: [2026-07-09-co-contribution-plan-design.md](./2026-07-09-co-contribution-plan-design.md)

## 概述

将共建计划标识（`gitflow.co_contribution`）的写入位置从"跟随 `--global` 标志"改为"强制写入用户级全局 settings"，避免每个项目重复加入共建计划。

## 动机

当前 `merge_co_contribution(args.global, platform)` 根据 `--global` 标志决定写入项目级或全局 settings。默认 `global=false`，标识被写入项目级 `.claude/settings.json`（或其他平台对应路径）。

用户每进入一个新项目运行 `skills install`，都需要重新加入共建计划。但共建计划本质上是**用户意愿的表达**，应该只加入一次、全局生效。

## 设计决策

| 决策 | 选择 | 理由 |
|------|------|------|
| 写入位置 | 始终全局 | 共建计划是用户级选择，不应绑定项目 |
| 函数签名 | 移除 `global` 参数 | 让非法状态不可表示，避免误用 |
| 项目级旧标记 | 不做迁移 | `is_co_contribution_enabled()` 已兼容双位置检查，不会破坏现有用户 |
| 提示文案 | 重写为简洁版 | 明确表达"用户级、一次生效" |

## 改动范围

### `apps/cli/src/commands/skills.rs`

#### 1. `merge_co_contribution` 签名变更

```rust
// Before
fn merge_co_contribution(global: bool, platform: AgentPlatform) -> miette::Result<()> {
    let (_hook_dir, settings_path, _cmd) = resolve_hook_paths(global, platform)?;
    // ...
}

// After
fn merge_co_contribution(platform: AgentPlatform) -> miette::Result<()> {
    let home = dirs::home_dir().ok_or_else(|| miette::miette!("无法确定 HOME 目录"))?;
    let (_hook_dir, settings_path, _cmd) = resolve_global_hook_paths(&home, platform);
    // ...
}
```

直接调用 `resolve_global_hook_paths` 而非 `resolve_hook_paths(true, ...)`，语义更清晰。

`platform` 参数保留，通过 `AgentPlatform::settings_file_path()` 获取对应平台的全局路径：

| 平台 | 全局路径 |
|------|---------|
| Claude | `~/.claude/settings.json` |
| Codex | `~/.codex/settings.json` |
| OpenCode | `~/.opencode/settings.json` |
| Gemini | `~/.gemini/settings.json` |
| Copilot | `~/.copilot/settings.json` |

#### 2. 调用点更新

`try_enable_co_contribution` 中两处调用去掉 `args.global`：

```rust
// Before
merge_co_contribution(args.global, platform)?;

// After
merge_co_contribution(platform)?;
```

#### 3. 提示文案重写

```
// Before
🤝 共建计划：加入后，CLI 错误将自动上报为 GitHub Issue，帮助改进 gitflow-cli。
   仅非交互模式（Agent/CI）下生效，普通控制台使用不受影响。

// After
🤝 共建计划：加入后，CLI 错误将自动上报为 GitHub Issue，帮助改进 gitflow-cli。
   用户级设置，加入一次即所有项目生效。
```

### 不改动

| 组件 | 原因 |
|------|------|
| `is_co_contribution_enabled()` | 已兼容双位置检查，保留项目级检查作为向后兼容 |
| `resolve_hook_paths()` | 其他功能（hook 安装等）仍需要 project/global 切换 |
| `merge_co_contribution_json()` | 纯 JSON 合并逻辑不变 |
| 已有项目级标记 | 不做迁移，`is_co_contribution_enabled()` 继续兼容 |

## 测试策略

| 测试 | 覆盖 |
|------|------|
| `merge_co_contribution` 单元测试 | 验证始终写入全局路径，与 `--global` 标志无关 |
| `merge_co_contribution_json` 现有测试 | 保持不变（JSON 合并逻辑不变） |
| `is_co_contribution_enabled` 现有测试 | 保持不变（双位置检查逻辑不变） |

## 向后兼容

- 已有项目级标记的用户：`is_co_contribution_enabled()` 仍会检测到，行为不变
- 用户下次运行 `skills install --force` 时自动写入全局，完成自然迁移
- 无需手动迁移步骤
