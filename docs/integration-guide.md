# Superpowers 集成指南

本指南说明如何将 gitflow-cli 的 Skills 与 [Superpowers](https://github.com/anthropics/superpowers) 的 SDD（Specification-Driven Development）工作流深度集成，实现从需求到交付的全流程自动化。

## 概览

gitflow-cli 与 Superpowers 形成**互补分层**的协作关系：

```
┌─────────────────────────────────────────────────────────────────┐
│                      Superpowers 层                              │
│  (本地开发能力: 创意、计划、TDD、Code Review、分支管理)             │
│                                                                   │
│  brainstorming ──► writing-plans ──► TDD / subagent-dev          │
│       │                  │                   │                    │
│       │                  │                   │                    │
│  requesting-code-review  │    finishing-a-development-branch      │
└──────────┬───────────────┬───────────────────┬────────────────────┘
           │               │                   │
           ▼               ▼                   ▼
┌─────────────────────────────────────────────────────────────────┐
│                    gitflow-cli Skills 层                         │
│  (平台交互能力: Issue / PR / Review / Release / 错误报告)         │
│                                                                   │
│  gitflow-issue-create ──► gitflow-pr-create ──► gitflow-release  │
│       │                        │                     │            │
│  gitflow-issue            gitflow-pr-review     gitflow-pr       │
│       │                                                  │        │
│  gitflow-autoreport-bug                                  │        │
│  (Stop Hook 自动触发)                                      │        │
└─────────────────────────────────────────────────────────────────┘
           │               │                   │
           ▼               ▼                   ▼
┌─────────────────────────────────────────────────────────────────┐
│                     gitflow CLI (Rust 二进制)                    │
│  统一命令行接口: GitHub / GitLab / Gitee / GitCode / Bitbucket   │
└─────────────────────────────────────────────────────────────────┘
```

**核心原则:**

- **Superpowers 负责「怎么做」** — 本地开发流程、代码生成、质量保证。
- **gitflow-cli 负责「在哪做」** — 平台交互、Issue/PR 管理、跨平台适配。
- **Skills 是桥梁** — 将两层能力编排为端到端的自动化工作流。

## 开发流程集成

以下是一个完整的 feature 开发流程，展示各 Skill 如何与 Superpowers 协作。

### Phase 1: 需求探索 → Issue 创建

```
用户: "实现多平台 Pipeline 支持"
         │
         ▼
┌─ Superpowers: brainstorming ─────────────────────────────────┐
│  • 探索用户意图、边界条件、验收标准                             │
│  • 输出: 需求规格、技术方案                                    │
└──────────────────────────────┬────────────────────────────────┘
                               │
                               ▼
┌─ gitflow-issue-create ──────────────────────────────────────┐
│  • 引导 Issue 标题、正文、标签、里程碑                         │
│  • 调用 gitflow issue create --platform github               │
│  • 输出: Issue URL (如 #42)                                  │
└──────────────────────────────┬────────────────────────────────┘
                               │
                               ▼
                    Issue #42 已创建
```

**触发方式:** 用户说「创建一个 Issue」或 `gitflow-workflow` Phase 1 自动触发。

### Phase 2: 计划制定 → 原子任务

```
┌─ Superpowers: writing-plans ─────────────────────────────────┐
│  • 基于 Issue 正文生成实现计划                                 │
│  • 拆分为可独立执行的原子任务                                  │
│  • 每个任务标注 TDD 步骤 (RED → GREEN → REFACTOR)            │
│  • 输出: docs/plans/{date}-{feature}.md                      │
└──────────────────────────────┬────────────────────────────────┘
                               │
                               ▼
┌─ Superpowers: subagent-driven-development ──────────────────┐
│  • 并行执行独立任务 (使用 git worktree 隔离)                  │
│  • 每个任务遵循 TDD: 写测试 → 写实现 → 重构                   │
│  • 输出: 通过测试的代码变更                                   │
└──────────────────────────────┬────────────────────────────────┘
                               │
                               ▼
                    所有任务完成，代码就绪
```

### Phase 3: 质量闸门

```
┌─ gitflow-quality ────────────────────────────────────────────┐
│  5 项检查，快速失败:                                          │
│                                                               │
│  1. build     cargo build --workspace              ✅/❌      │
│  2. test      cargo test --workspace               ✅/❌      │
│  3. coverage  cargo tarpaulin (>80%)               ✅/❌      │
│  4. format    cargo +nightly fmt -- --check        ✅/❌      │
│  5. static    cargo clippy -D warnings             ✅/❌      │
│                                                               │
│  • 自动检测项目语言 (Rust/Node/Python/Go)                      │
│  • 生成 Markdown 质量报告                                      │
│  • 如有 Issue 链接，自动发布为 Issue 评论                       │
└──────────────────────────────┬────────────────────────────────┘
                               │
                    ┌──────────┴──────────┐
                    │                     │
                 ALL PASS            GATE FAILED
                    │                     │
                    ▼                     ▼
              Phase 4            返回 Phase 2 修复
```

**触发方式:** `gitflow-workflow` Phase 3 自动触发，或用户手动调用「运行质量检查」。

### Phase 4: 交付

```
┌─ gitflow-pr-create ──────────────────────────────────────────┐
│  • 自动生成 PR 标题、正文 (引用 Issue #42)                     │
│  • 关联 Issue: closes #42                                    │
│  • 调用 gitflow pr create --platform github                  │
│  • 输出: PR URL (如 #43)                                     │
└──────────────────────────────┬────────────────────────────────┘
                               │
                               ▼
┌─ gitflow-pr-review ─────────────────────────────────────────┐
│  • 6 维度代码审查:                                            │
│    正确性 / 安全性 / 性能 / 可维护性 / 测试覆盖 / 文档         │
│  • 输出: 审查评论或 approve                                   │
└──────────────────────────────┬────────────────────────────────┘
                               │
                               ▼
┌─ Superpowers: finishing-a-development-branch ───────────────┐
│  • 决定集成方式: merge / squash / rebase                     │
│  • 清理 worktree 和临时分支                                   │
│  • 输出: 已合并的 PR + 关联的 Issue                           │
└─────────────────────────────────────────────────────────────┘
```

### 完整流程速查表

| 阶段 | Superpowers 技能 | gitflow 技能 | 输出物 |
|------|------------------|-------------|--------|
| 需求 | `brainstorming` | `gitflow-issue-create` | Issue URL |
| 计划 | `writing-plans` | — | 实现计划文档 |
| 实现 | `TDD` + `subagent-dev` | — | 通过测试的代码 |
| 质量 | — | `gitflow-quality` | 质量报告 |
| 交付 | `finishing-a-branch` | `gitflow-pr-create` + `gitflow-pr-review` | 已合并 PR |

## 错误反馈集成

gitflow-cli 内置了自动错误报告机制，当 CLI 命令失败时，会自动将错误信息反馈为 GitHub Issue。

### 数据流

```
gitflow CLI 命令失败
       │
       ▼
.error_reporter 写入 .cache/bug-reports/pending.json
       │
       ▼
Claude Code Stop Hook 触发 .claude/hooks/auto-report-bug.sh
       │
       ▼
脚本检测到 pending.json → 打印错误 banner
       │
       ▼
Claude 加载 gitflow-autoreport-bug Skill
       │
       ▼
┌─ 自动 Bug 报告流程 ─────────────────────────────────────────┐
│  1. 读取 pending.json (error_id, command, error_code 等)     │
│  2. Claude 分析根因 + 生成 Issue 标题/正文                    │
│  3. 去重检查: gitflow issue list --search                    │
│  4. 创建 Issue: gitflow issue create --label bug,auto-report │
│  5. 清理 pending.json                                        │
└─────────────────────────────────────────────────────────────┘
```

### pending.json 格式

```json
{
  "error_id": "550e8400-e29b-41d4-a716-446655440000",
  "command": "issue create",
  "platform": "github",
  "error_code": "AUTH_TOKEN_EXPIRED",
  "error_message": "GitHub API returned 401: Bad credentials",
  "timestamp": "2026-07-02T10:30:00Z",
  "stack_trace": "..."
}
```

### 触发条件

Stop Hook 仅在以下条件**全部满足**时触发:

1. 当前目录是 git 仓库。
2. `.cache/bug-reports/pending.json` 文件存在。
3. 文件内容包含有效的 `error_code` 字段。
4. **非交互模式** (stdout/stdin 不是 TTY)。

### 错误去重

创建 Issue 前，Skill 会通过 `gitflow issue list --search` 检查是否已有相同 `error_code` 的 Issue。如已存在，跳过创建并删除 `pending.json`，避免重复报告。

## 配置示例

### `.claude/settings.json` Hook 配置

以下是完整的 Hook 配置示例:

```json
{
  "hooks": {
    "Stop": [
      {
        "matcher": "gitflow",
        "hooks": [
          {
            "type": "command",
            "command": "bash \"$(git rev-parse --show-toplevel 2>/dev/null || pwd)/.claude/hooks/auto-report-bug.sh\""
          }
        ]
      }
    ]
  }
}
```

**配置说明:**

| 字段 | 说明 |
|------|------|
| `hooks.Stop` | Claude Code 停止时触发的 Hook 数组 |
| `matcher` | 匹配器，`"gitflow"` 表示与 gitflow 相关的会话触发 |
| `command` | 要执行的 shell 命令，此处调用自动错误报告脚本 |

### 个人化配置建议

#### 1. 平台选择

gitflow-cli 支持多平台，根据你的代码托管平台配置:

```bash
# GitHub (默认)
gitflow auth login --platform github

# GitLab
gitflow auth login --platform gitlab

# Gitee
gitflow auth login --platform gitee

# GitCode
gitflow auth login --platform gitcode
```

#### 2. 质量闸门阈值

通过环境变量自定义质量检查阈值:

```bash
# 覆盖率阈值 (默认 80%)
export COVERAGE_THRESHOLD=85

# 日志级别 (默认 info)
export APP_LOG_LEVEL=debug
```

#### 3. Skill 路径配置

确保 Claude Code 能找到 Skills 目录。在项目的 `.claude/settings.json` 中添加:

```json
{
  "skills": {
    "paths": ["skills"]
  }
}
```

#### 4. 多仓库工作流

如果同时维护多个仓库，建议:

- 每个仓库独立配置 `.claude/settings.json`。
- 共享 `_common.sh` 通过 symlink 或 git submodule 引入。
- 使用 `gitflow auth status` 确认各仓库的认证状态。

## 常见问题

### Q: Hook 没有触发怎么办?

检查以下几点:

1. `.claude/settings.json` 中 `hooks.Stop` 配置是否正确。
2. `.claude/hooks/auto-report-bug.sh` 是否有执行权限: `chmod +x .claude/hooks/auto-report-bug.sh`。
3. `.cache/bug-reports/pending.json` 是否存在。
4. 确认是非交互模式 (Hook 在 TTY 环境下会跳过)。

### Q: 如何禁用自动错误报告?

从 `.claude/settings.json` 中移除 `hooks.Stop` 配置即可:

```json
{
  "hooks": {}
}
```

### Q: Skill 之间的调用顺序是固定的吗?

`gitflow-workflow` 定义了推荐的 4 阶段流程，但每个 Skill 也可以独立使用。例如:

- 单独运行 `gitflow-quality` 做质量检查，不一定要在 workflow 中。
- 单独运行 `gitflow-pr-create` 创建 PR，不需要从 Issue 开始。
- 单独运行 `gitflow-autoreport-bug` 手动触发错误报告。
