# Superpowers 集成指南

本指南说明如何将 gf 的 Skills 与 [Superpowers](https://github.com/anthropics/superpowers) 的 SDD（Specification-Driven Development）工作流深度集成，实现从需求到交付的全流程自动化。

## 概览

gf 与 Superpowers 形成**互补分层**的协作关系：

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
│                    gf Skills 层                         │
│  (平台交互能力: Issue / PR / Review / Release / 错误报告)         │
│                                                                   │
│  gf-issue-create ──► gf-pr-create ──► gf-release  │
│       │                        │                     │            │
│  gf-issue            gf-pr-review     gf-pr       │
│       │                                                  │        │
│  gf-autoreport-bug                                  │        │
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
- **gf 负责「在哪做」** — 平台交互、Issue/PR 管理、跨平台适配。
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
┌─ gf-issue-create ──────────────────────────────────────┐
│  • 引导 Issue 标题、正文、标签、里程碑                         │
│  • 调用 gf issue create --platform github               │
│  • 输出: Issue URL (如 #42)                                  │
└──────────────────────────────┬────────────────────────────────┘
                               │
                               ▼
                    Issue #42 已创建
```

**触发方式:** 用户说「创建一个 Issue」或 `gf-workflow` Phase 1 自动触发。

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
┌─ gf-quality ────────────────────────────────────────────┐
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

**触发方式:** `gf-workflow` Phase 3 自动触发，或用户手动调用「运行质量检查」。

### Phase 4: 交付

```
┌─ gf-pr-create ──────────────────────────────────────────┐
│  • 自动生成 PR 标题、正文 (引用 Issue #42)                     │
│  • 关联 Issue: closes #42                                    │
│  • 调用 gf pr create --platform github                  │
│  • 输出: PR URL (如 #43)                                     │
└──────────────────────────────┬────────────────────────────────┘
                               │
                               ▼
┌─ gf-pr-review ─────────────────────────────────────────┐
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
| 需求 | `brainstorming` | `gf-issue-create` | Issue URL |
| 计划 | `writing-plans` | — | 实现计划文档 |
| 实现 | `TDD` + `subagent-dev` | — | 通过测试的代码 |
| 质量 | — | `gf-quality` | 质量报告 |
| 交付 | `finishing-a-branch` | `gf-pr-create` + `gf-pr-review` | 已合并 PR |

## mattpocock/skills 集成（Issue #141）

gf-workflow 同样支持 [mattpocock/skills](https://github.com/mattpocock/skills)
（plugin 名 `mattpocock-skills`）作为技能来源，与 Superpowers 互斥检测、按来源分支。

### 与 Superpowers 的关键差异

| | Superpowers | mattpocock/skills |
|---|---|---|
| 触发模型 | 模型全自动触发 | user-invoked 硬约束（`disable-model-invocation`）→ 暂停语义 ✋ |
| 计划产物 | 单一 plan 文档 | 票据图 + blocking edges（`ticket_refs`） |
| 主线 token | ≈14k + subagent 扇出 | ≈4.8k，不触发零消耗 |

### 前置条件

运行 `setup-mattpocock-skills` 生成 `docs/agents/issue-tracker.md`（tracker 与 triage
标签词表配置）。缺失时 gf-workflow 会询问：先配置或中止。

### 集成要点

- `to-spec` 受约束**只写本地** spec 文件、不发布 tracker；Issue 创建权统一归 `gf-issue-create`
- Phase 3 逐票据 ✋ `/implement`（内部强制 `/tdd` + `/code-review`）；Gate 2→3 执行模式
  菜单裁剪为「手动新窗口 / 同会话」（后台代理无法调用 user-invoked 技能）
- Phase 4 骨架不变：`gf-pipeline-analyzer` / `gf-issue-triage` / `gf-review` 照常
- 检测哨兵：`to-spec` + `grilling` 双命中（plugin 形 `mattpocock-skills:*` 或裸名）

完整映射表见 `skills/gf-workflow/references.md` → Dual-Source Skill Resolution。

## 错误反馈集成

gf 内置了自动错误报告机制，当 CLI 命令失败时，会自动将错误信息反馈为 GitHub Issue。

### 数据流

```
gitflow CLI 命令失败
       │
       ▼
.error_reporter 写入 .cache/bug-reports/pending.json
       │
       ▼
Claude Code Stop Hook（全局注册）触发 git 跟踪的 hooks/auto-report-bug.sh
       │
       ▼
脚本检测到 pending.json → 打印错误 banner
       │
       ▼
Claude 加载 gf-autoreport-bug Skill
       │
       ▼
┌─ 自动 Bug 报告流程 ─────────────────────────────────────────┐
│  1. 读取 pending.json (error_id, command, error_code 等)     │
│  2. Claude 分析根因 + 生成 Issue 标题/正文                    │
│  3. 去重检查: gf issue list --search                    │
│  4. 创建 Issue: gf issue create --label bug,auto-report │
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

创建 Issue 前，Skill 会通过 `gf issue list --search` 检查是否已有相同 `error_code` 的 Issue。如已存在，跳过创建并删除 `pending.json`，避免重复报告。

## 配置示例

### Hook 配置

Stop Hook 注册在**全局** `~/.claude/settings.json`，命令指向 **git 跟踪**的 `hooks/auto-report-bug.sh`（而非 gitignored 的 `.claude/hooks/`）。这样 `git worktree add` 创建的新 worktree 会自动带上脚本，hook 立即可用。

```json
{
  "hooks": {
    "Stop": [
      {
        "matcher": "gf|gitflow",
        "hooks": [
          {
            "type": "command",
            "command": "bash -c 'p=$(git rev-parse --show-toplevel 2>/dev/null) && [ -x \"$p/hooks/auto-report-bug.sh\" ] && bash \"$p/hooks/auto-report-bug.sh\"'"
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
| `matcher` | 匹配器，`"gf\|gitflow"` 表示 gf 或 gitflow 相关会话触发（兼容新旧 CLI 名） |
| `command` | 解析 repo 根目录并执行 `hooks/auto-report-bug.sh`；非 git 仓库或脚本缺失时静默跳过 |

> **为什么用全局注册 + git 跟踪脚本?** `.claude/` 目录被 `.gitignore` 忽略，
> 因此 `git worktree add` 不会物化 `.claude/hooks/` 下的脚本与注册。
> 将脚本放入 git 跟踪的 `hooks/`，并在全局 `~/.claude/settings.json` 注册，
> 所有项目与 worktree 都会自动生效。

### 个人化配置建议

#### 1. 平台选择

gf 支持多平台，根据你的代码托管平台配置:

```bash
# GitHub (默认)
gf auth login --platform github

# GitLab
gf auth login --platform gitlab

# Gitee
gf auth login --platform gitee

# GitCode
gf auth login --platform gitcode
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
- 使用 `gf auth status` 确认各仓库的认证状态。

## 常见问题

### Q: Hook 没有触发怎么办?

检查以下几点:

1. 全局 `~/.claude/settings.json` 中 `hooks.Stop` 配置是否正确。
2. git 跟踪的 `hooks/auto-report-bug.sh` 是否有执行权限: `chmod +x hooks/auto-report-bug.sh`。
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

`gf-workflow` 定义了推荐的 4 阶段流程，但每个 Skill 也可以独立使用。例如:

- 单独运行 `gf-quality` 做质量检查，不一定要在 workflow 中。
- 单独运行 `gf-pr-create` 创建 PR，不需要从 Issue 开始。
- 单独运行 `gf-autoreport-bug` 手动触发错误报告。
