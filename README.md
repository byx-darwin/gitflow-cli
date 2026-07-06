# gitflow-cli

[![Release](https://img.shields.io/github/v/release/byx-darwin/gitflow-cli)](https://github.com/byx-darwin/gitflow-cli/releases/latest)
[![gitflow-cli/0.3.0](https://img.shields.io/badge/gitflow--cli-0.3.0-blue)](https://github.com/byx-darwin/gitflow-cli)

多平台 Git 锻造 CLI + Superpowers Skills 集合，覆盖从需求到交付的完整 AI 编程工程循环。

## 架构

```
Phase 1: 需求澄清         Phase 2: 计划制定          Phase 3: 执行              Phase 4: 交付后检查
(远端 + Superpowers)      (Superpowers)              (Superpowers)               (gitflow-cli)
    │                         │                        │                          │
    ├─ brainstorming          ├─ writing-plans         ├─ TDD (内嵌)              ├─ pipeline-analyzer
    ├─ issue-create           └─ 完整计划文档           ├─ subagent-dev            ├─ issue-triage
    ├─ issue-review             (含质量关卡)            └─ quality (内嵌)          └─ review
    └─ 需求分析报告                                      ↓
                                                       PR → 合并 → 发布
            ↑                         ↑                    ↑                          ↑
            └─────────────── gitflow-workflow（编排层）──────────────────────────────┘
```

- **Superpowers**：需求澄清 / 原子任务拆解 / TDD / 子代理隔离 / 任务审查 / 收尾
- **gitflow-cli skills**：Issue 管理 / PR 创建审查 / 跨平台命令 / 安全审计 / Release 发布
- **`gitflow-workflow`**：四阶段编排层 — 需求澄清 → 计划制定 → 执行 → 交付后检查

## 平台支持

### Git 平台

`gitflow-cli` CLI 统一封装了三大 Git 平台的差异，通过 `--platform` 自动检测或手动指定：

| 平台 | CLI 依赖 | 特性 |
|------|---------|------|
| **GitHub** | `gh` (v2.0.0+) | Issue / PR / Release / Review / Pipeline / Repo（含 Enterprise） |
| **GitLab** | `glab` (v1.30.0+) | Issue / PR(MR) / Release / Review / Pipeline / Repo（含自建实例） |
| **GitCode** | `gitcode` (v0.6.0+) | Issue / PR(MR) / Release / Review / Pipeline / Repo |

```bash
# 自动检测（基于 git remote，支持 gitlab.com 及自建 GitLab 实例）
gitflow-cli issue list

# 手动指定平台
gitflow-cli issue list --platform gitlab --output text
```

### Agent 平台

Skills 可安装到任意支持的 AI Agent 平台，通过 `-g --agent` 指定：

| Agent | 安装目录 | 安装命令 |
|-------|---------|---------|
| **Claude Code** | `~/.claude/skills/` | `gitflow-cli skills install -g --agent claude` |
| **Codex** (OpenAI) | `~/.codex/skills/` | `gitflow-cli skills install -g --agent codex` |
| **OpenCode** | `~/.opencode/skills/` | `gitflow-cli skills install -g --agent open-code` |
| **Gemini CLI** | `~/.gemini/skills/` | `gitflow-cli skills install -g --agent gemini` |
| **Copilot CLI** | `~/.copilot/skills/` | `gitflow-cli skills install -g --agent copilot` |

默认 `-g` 不指定 agent 时自动检测当前环境已有的平台目录。

## Skill 矩阵

### 编排

| Skill | 做什么 |
|-------|--------|
| `gitflow-workflow` | 四阶段全流程编排：需求澄清 → 计划制定 → 执行 → 交付后检查。支持完整模式（默认）和快速模式（`--fast`） |
| `gitflow-quality` | 本地质量门禁：build → test → coverage → format → static → pre-commit，支持 Rust/Node.js/Python/Go/Java 自动检测 |

### Issue 流水线

| Skill | 时机 | 做什么 |
|-------|------|--------|
| `gitflow-issue-create` | 提交前 | 引导填 issue → 模板填充 → 创建 |
| `gitflow-issue-review` | 开发前 | 需求分析 → 完整性检查 → 改进建议 → 回写评论 |
| `gitflow-issue-triage` | 提交后 | 分类 → 标签 → 优先级 → 分流 |

### PR 流水线

| Skill | 时机 | 做什么 |
|-------|------|--------|
| `gitflow-pr-create` | 提交时 | 检查变更 → PR 标题描述 → 提交 |
| `gitflow-pr-review` | 提交后 | 6 维审查 → 审查结论 → 提交 |
| `gitflow-pr-inline-review` | 审查时 | 逐文件逐行评论 → 逻辑/安全/命名/边界 |
| `gitflow-pr-apply-feedback` | 审查后 | 获取反馈 → 逐条本地应用 → 标记 resolved |

### 交付

| Skill | 时机 | 做什么 |
|-------|------|--------|
| `gitflow-release-helper` | 发布时 | 分析变更 → 生成 Release Note → 创建 Release |
| `gitflow-label-stats` | 发布前 | 标签统计 → 优先级分布 → 未分类识别 |
| `gitflow-pipeline-analyzer` | 发布前 | 流水线健康 → 成功率/失败模式 → 改进建议 |

### 辅助

| Skill | 时机 | 做什么 |
|-------|------|--------|
| `gitflow-security-check` | 审查时 | 代码安全审计：凭证/注入/认证/依赖/加密 |
| `gitflow-precommit` | 提交前 | fmt → clippy → test → 配置 pre-commit hook |
| `gitflow-regression` | 验证时 | 冒烟测试 → 解析结果 → 失败自动上报 |
| `gitflow-repo-onboarding` | 入门时 | 仓库结构 → 构建 → 测试 → 贡献流程 |
| `gitflow-autoreport-bug` | 出错时 | 错误捕获 → 去重 → 自动创建 Issue |

## 快速开始

### Step 1：安装 gitflow-cli

```bash
# Homebrew (macOS)
brew tap byx-darwin/gitflow-cli
brew install gitflow-cli

# 或 Cargo
cargo install gitflow-cli
```

### Step 2：安装 Skills

```bash
# 项目级（推荐 — 跟随仓库）
gitflow-cli skills install

# 全局（所有项目可用）
gitflow-cli skills install -g
```

### Step 3：验证

```bash
gitflow-cli skills list
# 应看到 25 个 gitflow-* skills

gitflow-cli --version
# gitflow-cli 0.1.0
```

### Step 4：开始开发

```
/开发工作流，我要做 X
```

## gitflow-workflow 工作模式

**完整模式**（默认）：适用于新功能开发、重大重构
```
/gitflow-workflow
```

**快速模式**（`--fast`）：适用于 bug 修复、小优化
```
/gitflow-workflow --fast
```

## 典型工作流

```
Phase 1 需求澄清    Phase 2 计划制定    Phase 3 执行              Phase 4 交付后检查
brainstorming  →  writing-plans  →  TDD + subagent-dev  →  pipeline-analyzer
issue-create       完整计划文档        quality (6 项检查)      issue-triage
issue-review       (含质量关卡)        PR → 合并 → 发布        review
```

## CLI 命令一览

| 命令 | 用途 |
|------|------|
| `gitflow-cli issue {create,list,view,close,reopen,comment}` | Issue 管理 |
| `gitflow-cli pr {create,list,view,close,merge,checkout}` | PR 管理 |
| `gitflow-cli release {create,list,view,edit}` | 发布管理 |
| `gitflow-cli review {comment,approve,request-changes,submit}` | 代码审查 |
| `gitflow-cli auth {login,logout,status,token}` | 认证管理 |
| `gitflow-cli pipeline {status,logs,jobs,report}` | CI/CD 流水线 |
| `gitflow-cli commit {view,diff,patch,comment}` | 提交操作 |
| `gitflow-cli label/milestone` | 标签/里程碑管理 |
| `gitflow-cli repo {clone,list,create,stats,sync,view}` | 仓库操作 |
| `gitflow-cli skills {install,list,uninstall}` | Skills 管理 |
| `gitflow-cli completions {bash,zsh,fish}` | Shell 补全 |

支持 `--platform github|gitlab|gitcode` 和 `--output json|text`。

## 设计原则

- **步骤化工作流**：每个 skill 有明确的步骤顺序，不跳步
- **先验证再行动**：PR 创建前检查分支和变更；Issue 创建前引导填写模板
- **生态互补**：本地开发循环 (Superpowers) + 远端协作 (gitflow-cli) 明确分工
- **多 Agent 兼容**：skills 可安装到 Claude Code / Codex / OpenCode / Gemini / Copilot
- **质量门闸门**：build → test → coverage → format → static → pre-commit 全部通过才能交付，支持多语言自动检测

## 贡献

详见 [CONTRIBUTING.md](CONTRIBUTING.md)。
