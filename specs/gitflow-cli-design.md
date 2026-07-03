# GitFlow CLI 设计规格

## 概述

gitflow-cli 是一个跨平台 Git 工程化工作流编排框架。通过 Rust CLI 统一封装 GitHub、GitLab、GitCode 三大平台的差异，结合 Superpowers 的本地开发循环能力，提供从需求澄清到代码发布的完整开发生命周期管理。

---

## 核心设计决策

| 决策项 | 选择 |
|--------|------|
| 项目名 | gitflow-cli |
| 定位 | 社区项目，全套解决方案（编排层 + 所有平台 skills） |
| 平台 | GitHub / GitLab / GitCode（放弃 Gitee） |
| 架构模式 | CLI-centric，统一抽象层 |
| CLI 实现 | Rust，多 crate workspace |
| 平台检测 | 根据 `git remote get-url origin` 自动识别 |
| 与 Superpowers 关系 | 依赖，参考 gitcode-dev-workflow 模式 |
| Skills 结构 | 对齐 gitcode-cli（核心命令层 + 工作流层 + 编排层） |

---

## 三层架构

```
编排层（Orchestration Skills）
├── gitflow-workflow           # 全流程编排
├── gitflow-quality            # 质量关卡
└── gitflow-autoreport-bug     # 自动错误反馈

工作流层（Workflow Skills）
├── gitflow-repo-onboarding        # 仓库入门
├── gitflow-security-check         # 安全审计
├── gitflow-issue-create           # 引导写 Issue
├── gitflow-issue-review           # Issue 需求分析
├── gitflow-issue-triage           # Issue 分类分流
├── gitflow-pr-create              # 引导创建 PR
├── gitflow-pr-review              # 工程审查
├── gitflow-pr-inline-review       # 行内评论
├── gitflow-pr-apply-feedback      # 应用审查反馈
├── gitflow-release-helper         # 发布助手
├── gitflow-pipeline-analyzer      # 流水线分析（通用接口）
└── gitflow-label-stats            # 标签统计

核心命令层（Core Command Skills）
├── gitflow-auth                   # 认证管理
├── gitflow-repo                   # 仓库操作
├── gitflow-issue                  # Issue 直接命令
├── gitflow-pr                     # PR 直接命令
├── gitflow-review                 # 评论机制
├── gitflow-release                # Release 直接命令
├── gitflow-commit                 # Commit 操作
├── gitflow-precommit              # Pre-commit 检查
├── gitflow-label-milestone        # 标签/里程碑管理
└── gitflow-regression             # 冒烟测试
```

**各层职责**：
- **编排层**：全流程指挥者，知道什么时候该调哪个 skill，定义阶段闸门和交接点
- **工作流层**：具体操作的引导者，确保操作规范（查重、模板填充、多维度审查等）
- **核心命令层**：平台操作的执行者，直接调用 `gitflow` CLI 操作 Git 平台

---

## Rust Workspace 扩展

基于现有 `crates/*` + `apps/*` 结构，新增平台 crate：

```
gitflow-cli/
├── apps/
│   └── cli/                    # CLI 入口（已存在，扩展命令）
│       └── src/
│           ├── main.rs         # clap 入口
│           ├── config.rs       # 配置加载（已存在）
│           └── commands/       # CLI 命令模块
│               ├── mod.rs      # （已存在）
│               ├── run.rs      # （已存在）
│               ├── completions.rs  # （已存在）
│               ├── auth.rs     # 新增：认证管理
│               ├── issue.rs    # 新增：Issue 操作
│               ├── pr.rs       # 新增：PR 操作
│               ├── release.rs  # 新增：Release 操作
│               ├── repo.rs     # 新增：仓库操作
│               ├── review.rs   # 新增：Review 操作
│               ├── commit.rs   # 新增：Commit 操作
│               ├── label.rs    # 新增：标签管理
│               └── pipeline.rs # 新增：流水线分析
│
├── crates/
│   ├── core/                   # 核心库（已存在，扩展 domain types）
│   │   └── src/
│   │       ├── lib.rs          # （已存在）
│   │       ├── platform.rs     # 新增：Platform trait + 检测逻辑
│   │       ├── auth.rs         # 新增：Auth trait
│   │       ├── issue.rs        # 新增：Issue trait
│   │       ├── pr.rs           # 新增：PR trait
│   │       ├── release.rs      # 新增：Release trait
│   │       ├── review.rs       # 新增：Review trait
│   │       ├── repo.rs         # 新增：Repo trait
│   │       ├── commit.rs       # 新增：Commit trait
│   │       ├── label.rs        # 新增：Label/Milestone trait
│   │       ├── pipeline.rs     # 新增：Pipeline trait（通用接口）
│   │       └── output.rs       # 新增：JSON 输出格式化
│   │
│   ├── github/                 # 新增：GitHub 实现
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── auth.rs         # 调用 gh auth
│   │       ├── issue.rs        # 调用 gh issue
│   │       ├── pr.rs           # 调用 gh pr
│   │       ├── release.rs      # 调用 gh release
│   │       ├── pipeline.rs     # 调用 gh run / gh workflow
│   │       └── ...
│   │
│   ├── gitlab/                 # 新增：GitLab 实现
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── auth.rs         # 调用 glab auth
│   │       ├── issue.rs        # 调用 glab issue
│   │       ├── mr.rs           # 调用 glab mr
│   │       ├── release.rs      # 调用 glab release
│   │       ├── pipeline.rs     # 调用 glab ci / glab pipeline
│   │       └── ...
│   │
│   └── gitcode/                # 新增：GitCode 实现
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── auth.rs         # 调用 gitcode auth
│           ├── issue.rs        # 调用 gitcode issue
│           ├── pr.rs           # 调用 gitcode pr
│           ├── release.rs      # 调用 gitcode release
│           ├── pipeline.rs     # 调用 gitcode pipeline
│           └── ...
│
├── skills/                     # 新增：Claude Code skills
│   ├── gitflow-workflow/
│   │   └── SKILL.md
│   ├── gitflow-quality/
│   │   └── SKILL.md
│   ├── gitflow-autoreport-bug/
│   │   └── SKILL.md
│   ├── gitflow-issue-create/
│   │   └── SKILL.md
│   ├── gitflow-issue-review/
│   │   └── SKILL.md
│   ├── gitflow-issue-triage/
│   │   └── SKILL.md
│   ├── gitflow-pr-create/
│   │   └── SKILL.md
│   ├── gitflow-pr-review/
│   │   └── SKILL.md
│   ├── gitflow-pr-inline-review/
│   │   └── SKILL.md
│   ├── gitflow-pr-apply-feedback/
│   │   └── SKILL.md
│   ├── gitflow-release-helper/
│   │   └── SKILL.md
│   ├── gitflow-security-check/
│   │   └── SKILL.md
│   ├── gitflow-pipeline-analyzer/
│   │   └── SKILL.md
│   ├── gitflow-repo-onboarding/
│   │   └── SKILL.md
│   ├── gitflow-auth/
│   │   └── SKILL.md
│   ├── gitflow-repo/
│   │   └── SKILL.md
│   ├── gitflow-issue/
│   │   └── SKILL.md
│   ├── gitflow-pr/
│   │   └── SKILL.md
│   ├── gitflow-review/
│   │   └── SKILL.md
│   ├── gitflow-release/
│   │   └── SKILL.md
│   ├── gitflow-commit/
│   │   └── SKILL.md
│   ├── gitflow-precommit/
│   │   └── SKILL.md
│   ├── gitflow-label-milestone/
│   │   └── SKILL.md
│   ├── gitflow-label-stats/
│   │   └── SKILL.md
│   └── gitflow-regression/
│       └── SKILL.md
│
├── scripts/                    # 新增：辅助脚本
│   ├── install.sh              # 安装脚本（CLI + skills）
│   ├── smoke-test.sh           # 跨平台冒烟测试
│   └── _common.sh              # 共享函数
│
├── hooks/                      # 新增：Claude Code hooks
│   ├── auto-smoke-test.sh      # CLI 变更时自动冒烟测试
│   └── sync-readme-check.sh    # 目录结构 vs README 检查
│
├── docs/                       # 项目文档（已存在，扩展）
│   └── integration-guide.md    # 新增：与 Superpowers 集成指南
│
├── specs/                      # 设计文档（已存在）
│   ├── index.md                # （已存在，更新索引）
│   └── gitflow-cli-design.md   # 本文档
│
├── Cargo.toml                  # workspace 配置（已存在，扩展 members）
├── Makefile                    # （已存在）
└── CLAUDE.md                   # （已存在）
```

---

## 依赖流

```
apps/cli ──依赖──> crates/core ──定义──> Platform traits
                       ↑
       crates/github ──┤
       crates/gitlab ──┤
       crates/gitcode ─┤

skills/* ──subprocess调用──> gitflow CLI (apps/cli)
```

- `crates/core` 定义 traits，不依赖任何平台 crate
- 每个平台 crate 实现 `core` 的 traits
- `apps/cli` 依赖 `core` + 所有平台 crate，在启动时注册各平台实现
- Skills 通过 subprocess 调用 CLI，不直接依赖 Rust 代码

---

## CLI 命令接口

```bash
gitflow <resource> <action> [options]

# 资源类型
gitflow auth        login | logout | status | token
gitflow repo        clone | list | create | delete | stats | sync | view
gitflow issue       create | list | view | edit | close | reopen | comment | label
gitflow pr          create | list | view | edit | checkout | comment | merge | close | reopen | ready | wip | sync
gitflow review      comment | approve | request-changes | submit
gitflow release     create | list | view | edit | upload | download | delete
gitflow commit      view | diff | patch | comment
gitflow label       create | list | edit | delete
gitflow milestone   create | list | edit | close | reopen
gitflow pipeline    status | logs | jobs | report

# 全局 flags
gitflow --platform <github|gitlab|gitcode>  # 覆盖自动检测（可选）
gitflow --output <json|text>                # 输出格式，默认 json
gitflow --verbose                           # 详细日志

# 安装 skills
gitflow skills install  # 复制 skills 到 ~/.claude/skills/
```

---

## JSON 输出格式

所有命令统一输出 JSON，Skill 通过 `jq` 解析。

```json
// 成功
{
  "success": true,
  "data": {
    "url": "https://github.com/user/repo/issues/123",
    "number": 123,
    "title": "Bug fix",
    "state": "open",
    "created_at": "2026-01-15T10:30:00Z"
  },
  "platform": "github",
  "command": "issue create"
}

// 失败
{
  "success": false,
  "error": {
    "code": "AUTH_FAILED",
    "message": "Authentication failed. Run 'gitflow auth login' first.",
    "hint": "gh auth status"
  },
  "platform": "github"
}
```

---

## 平台检测

基于 `git remote get-url origin` 的结果：

```rust
fn detect_platform() -> Result<Platform> {
    let remote = get_git_remote_url("origin")?;

    if remote.contains("github.com") || remote.contains("github.") {
        Ok(Platform::GitHub)
    } else if remote.contains("gitlab.com") || remote.contains("gitlab.") {
        Ok(Platform::GitLab)
    } else if remote.contains("gitcode.com") || remote.contains("gitcode.") {
        Ok(Platform::GitCode)
    } else {
        // 尝试通过 API 探测
        probe_platform_via_api(&remote)
    }
}
```

用户可通过 `--platform` flag 强制覆盖自动检测。

---

## Core Trait 设计

### Platform trait

```rust
/// 平台标识
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Platform {
    /// GitHub
    GitHub,
    /// GitLab
    GitLab,
    /// GitCode
    GitCode,
}
```

### Issue trait

```rust
/// Issue 操作的平台抽象
#[async_trait]
pub trait IssueProvider: Debug + Send + Sync {
    /// 创建 Issue
    async fn create(&self, args: CreateIssueArgs) -> Result<IssueData>;
    /// 列出 Issue
    async fn list(&self, args: ListIssueArgs) -> Result<Vec<IssueData>>;
    /// 查看 Issue 详情
    async fn view(&self, number: u64) -> Result<IssueData>;
    /// 编辑 Issue
    async fn edit(&self, number: u64, args: EditIssueArgs) -> Result<IssueData>;
    /// 关闭 Issue
    async fn close(&self, number: u64) -> Result<()>;
    /// 重新打开 Issue
    async fn reopen(&self, number: u64) -> Result<()>;
    /// 添加评论
    async fn comment(&self, number: u64, body: &str) -> Result<CommentData>;
    /// 添加标签
    async fn add_labels(&self, number: u64, labels: &[String]) -> Result<()>;
    /// 移除标签
    async fn remove_label(&self, number: u64, label: &str) -> Result<()>;
}
```

其他 traits（`PrProvider`、`ReleaseProvider`、`PipelineProvider`、`AuthProvider` 等）遵循相同模式。

### 平台实现示例

```rust
// crates/github/src/issue.rs
pub struct GitHubIssueProvider;

#[async_trait]
impl IssueProvider for GitHubIssueProvider {
    async fn create(&self, args: CreateIssueArgs) -> Result<IssueData> {
        // 构建 gh issue create 命令参数
        let mut cmd = Command::new("gh");
        cmd.args(["issue", "create"])
            .arg("--title").arg(&args.title)
            .arg("--body").arg(&args.body);

        if let Some(labels) = &args.labels {
            cmd.arg("--label").arg(labels.join(","));
        }

        // 执行并解析 JSON 输出
        let output = cmd.output()?;
        if !output.status.success() {
            return Err(parse_gh_error(&output));
        }

        let issue: GitHubIssue = serde_json::from_slice(&output.stdout)?;
        Ok(issue.into_domain())
    }
}
```

---

## 工作流编排

参考 gitcode-dev-workflow 的模式：

### 阶段总览

```
Phase 1: 需求               Phase 2: 开发               🔒 Quality Gate            Phase 3: 交付
──────────────────────────────────────────────────────────────────────────────────────────
Superpowers:               Superpowers:               gitflow-quality:           gitflow-pr-create
  brainstorming              writing-plans              build      ✅/❌           gitflow-pr-review
gitflow-issue-create         TDD (per task)             test       ✅/❌           Superpowers:
gitflow-issue-review         subagent-dev               coverage   ✅/❌             finishing-a-dev-branch
                             requesting-review          format     ✅/❌           gitflow-release-helper
                             (per task)                 static     ✅/❌
     ↓                             ↓                    ↓                               ↓
产出: Issue URL              产出: 全绿原子任务清单      产出: Quality Report       产出: Merged PR + Release
```

### 阶段闸门

| 阶段转换 | 必须出示的证据 | 违规后果 |
|---------|--------------|---------|
| 需求 → 开发 | Issue URL + 需求分析报告 | 🔒 不允许写一行代码 |
| 开发 → 质量关卡 | 全部原子任务清单（全部 done） | 🔒 不允许跑质量检查 |
| 质量关卡 → 交付 | 5 项检查全部绿色 | 🔒 不允许创建 PR |

### Issue 审计日志

每个阶段产出物以评论形式发布到关联 Issue：

```bash
Phase 1 完成 → gitflow issue comment <number> --body-file /tmp/phase1-report.md
Phase 2 完成 → gitflow issue comment <number> --body-file /tmp/phase2-tasks.md
Quality Gate → gitflow issue comment <number> --body-file /tmp/quality-report.md
Phase 3 完成 → gitflow issue comment <number> --body "PR #N: <url> — 审查通过"
```

长内容用 `--body-file`，短链接用 `--body`。

### 关键交接点

1. **brainstorming → gitflow-issue-create**：将 Superpowers 需求澄清输出整理为 `gitflow issue create` 输入
2. **gitflow-issue-review → writing-plans**：将需求分解传递给 writing-plans 拆解为 2-5min 原子任务
3. **TDD + subagent 完成 → gitflow-quality**：运行质量检查（build → test → coverage → format → static）
4. **质量通过 → gitflow-pr-create**：检查变更、conventional commit 标题、创建 PR
5. **gitflow-pr-create → gitflow-pr-review**：6 维度审查 + 行内评论
6. **merge → gitflow-release-helper**：生成 release note、创建 release

---

## 自动错误反馈（auto-report-bug）

### 概述

当 `gitflow` CLI 或 Skills 脚本执行出错时，自动捕获错误上下文、去重检查、生成 Issue 提交到 `byx-darwin/gitflow-cli` 仓库，实现错误的可追踪、可系统性修复。

采用五层架构：

```
┌─────────────────┐
│  捕获层          │  CLI 错误 handler + Skills 脚本 ERR trap
└────────┬────────┘
         ▼
┌─────────────────┐
│  缓冲层          │  写入 .cache/bug-reports/pending.json
└────────┬────────┘
         ▼
┌─────────────────┐
│  检测层          │  Claude Code Stop Hook 检查 pending.json
└────────┬────────┘
         ▼
┌─────────────────┐
│  处理层          │  gitflow-autoreport-bug 技能
│                  │  读取 → Claude 分析 → 去重 → 创建 Issue
└────────┬────────┘
         ▼
┌─────────────────┐
│  通知层          │  输出 Issue 链接
└─────────────────┘
```

### 错误来源

gitflow-cli 是 Rust CLI + Skills 混合项目，错误来源有两类：

| 错误来源 | 捕获方式 |
|---------|---------|
| Rust CLI 执行失败（gh 错误、平台检测失败、前置检查失败）| `miette` 错误 handler 写入 `pending.json` |
| Skills 脚本执行失败 | `_common.sh` 的 `report_error()` + ERR trap |

### 捕获层设计

#### A. Rust CLI 侧（`apps/cli` 新增 `error_reporter` 模块）

```rust
// apps/cli/src/error_reporter.rs

use std::path::PathBuf;
use serde::Serialize;

/// 错误报告结构，写入 pending.json
#[derive(Debug, Serialize)]
struct ErrorReport {
    id: String,              // CLI 进程 PID + 时间戳 hash
    command: String,         // 用户执行的 gitflow 命令
    platform: String,        // 目标平台
    exit_code: i32,          // 退出码
    error_code: String,      // 错误码（如 "AUTH_FAILED", "PLATFORM_NOT_FOUND"）
    error_message: String,   // 错误消息
    hint: Option<String>,    // 修复提示
    timestamp: String,       // ISO 8601 UTC
}

impl ErrorReport {
    /// 从 miette 错误 + 命令上下文构建。
    fn from_error(command: &str, platform: &str, error: &str, error_code: &str) -> Self { ... }

    /// 写入 `.cache/bug-reports/pending.json`。
    /// 仅在非交互模式（CI/piped）下写入，交互模式下由用户自行判断。
    fn write_to_disk(&self, repo_root: &Path) -> std::result::Result<(), std::io::Error> { ... }
}
```

在 `async_main` 中，所有 `miette::Result` 的 Err 分支调用 `maybe_report_error()`：

```rust
// async_main 的 Err 处理
Err(e) => {
    // 交互式终端 → 用户能看到错误，不自动报告
    // 非交互式（CI / subprocess）→ 写入 pending.json
    if !std::io::stderr().is_terminal() {
        let _ = error_reporter::report(
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

#### B. Skills 脚本侧（Phase 2+ 启用）

Skills 脚本侧（Phase 2+ 启用）：`_common.sh` 提供 `report_error()` 函数，各脚本在 `trap ... ERR` 中调用。

### 缓冲层

**`pending.json` 格式：**

```json
{
  "id": "a1b2c3d4",
  "source": "cli",
  "command": "gitflow issue create --title 'bug'",
  "platform": "github",
  "exit_code": 1,
  "error_code": "AUTH_FAILED",
  "error_message": "Authentication failed. Run 'gh auth login' first.",
  "hint": "gh auth status",
  "timestamp": "2026-07-01T10:30:00Z"
}
```

路径：`$REPO_ROOT/.cache/bug-reports/pending.json`

### 检测层

**`hooks/auto-report-bug.sh`：**

```bash
#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || exit 0)
PENDING_FILE="$REPO_ROOT/.cache/bug-reports/pending.json"

if [ ! -f "$PENDING_FILE" ]; then
  exit 0
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  🐛 检测到 gitflow CLI 错误，正在生成 Bug 报告..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

cat "$PENDING_FILE"
```

**Hook 配置（`.claude/settings.json`）：**

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

### 处理层

**Skills 三层结构新增：**

```
skills/
├── gitflow-autoreport-bug/    # 新增：编排层辅助
│   └── SKILL.md
```

**`gitflow-autoreport-bug/SKILL.md` 核心流程：**

1. 读取 `.cache/bug-reports/pending.json`
2. 生成搜索关键词：`[auto-report] {command} {error_code}`
3. 去重检查：
   ```bash
   gh issue list \
     --repo byx-darwin/gitflow-cli \
     --search "$SEARCH_QUERY" \
     --state all \
     --json number,title,state \
     --limit 5
   ```
4. 找到匹配 → 跳过，删除 `pending.json`；未找到 → Claude 分析 → 创建 Issue
5. 创建成功 → 删除 `pending.json`，输出 Issue 链接

**Issue 格式：**

```markdown
标题: [auto-report] gitflow {command} — {error_code}

正文:
## 错误信息
- **命令**: `{command}`
- **平台**: {platform}
- **错误码**: {error_code}
- **退出码**: {exit_code}
- **时间**: {timestamp}

## 错误详情
```
{error_message}
```

## LLM 分析
{Claude 基于错误上下文生成的分析：可能原因、建议修复方向}

---
*此 Issue 由 gitflow-autoreport-bug 技能自动创建*
```

**标签：** `bug,auto-reported,{platform},{error_code}`

### 异常处理

| 异常 | 处理 |
|------|------|
| `gh auth status` 失败 | 记录到 `.cache/bug-reports/failed.log`，保留 `pending.json` 下次重试 |
| `gh issue create` 失败 | 同上 |
| `pending.json` 格式异常 | 重命名加 `.invalid` 后缀，不阻塞流程 |
| `pending.json` 不存在 | Hook 静默退出 |
| 重复 Issue 命中 | 删除 `pending.json`，输出通知 |
| 非 git 仓库 | Hook 静默退出 |
| 交互式终端（Rust 侧） | 不写入 `pending.json` |

### 涉及设计变更

| 影响范围 | 变更 |
|---------|------|
| `apps/cli/src/` | 新增 `error_reporter.rs` 模块 |
| `apps/cli/src/main.rs` | `async_main` Err 分支调用 `maybe_report_error()` |
| `hooks/auto-report-bug.sh` | 新建 |
| `.claude/settings.json` | 注册 Stop Hook |
| `skills/gitflow-autoreport-bug/SKILL.md` | 新建（Phase 2+） |
| `skills/_common.sh` | 新建（Phase 2+），提供 `report_error()` |

### 实现时机

- **Phase 1**：Rust 侧 error_reporter + pending.json 写入 + Hook 检测脚本
- **Phase 2+**：Skills 侧 `_common.sh` 错误捕获 + 完整 `gitflow-autoreport-bug` skill

---

## 测试策略

基于项目已有的测试基础设施（rstest、proptest、assert_cmd、trycmd）：

| 层级 | 内容 | 触发 |
|------|------|------|
| Level 1: 编译检查 | `cargo check` + `cargo clippy` | 每次 push |
| Level 2: 单元测试 | trait 实现逻辑、平台检测 | `make test` |
| Level 3: 集成测试 | Mock CLI 输出 | CI 自动 |
| Level 4: E2E 测试 | 真实平台 API（冒烟测试） | CI 手动触发 / hooks 自动 |

### 冒烟测试

```bash
# 11 个核心操作，覆盖三平台
prerequisites → auth_status → issue_create → issue_get → issue_list \
→ issue_add_labels → issue_remove_label → pr_create → pr_get \
→ issue_close → verify_closed

bash scripts/smoke-test.sh                    # 自动检测平台
bash scripts/smoke-test.sh --platform gitlab  # 强制 GitLab
bash scripts/smoke-test.sh --write            # 包含写操作
bash scripts/smoke-test.sh --keep             # 保留测试 issue
```

---

## 安装与发布

### 安装

```bash
# CLI（从源码编译）
cargo install gitflow-cli

# 或从 GitHub Releases 下载预编译二进制
curl -sSL https://github.com/byx-darwin/gitflow-cli/releases/latest/download/install.sh | bash

# Skills 安装
gitflow skills install  # 复制到 ~/.claude/skills/

# 验证
gitflow --version
gitflow auth status
ls ~/.claude/skills/ | grep gitflow
```

### 使用

```bash
# 在 Claude Code 中启动完整工作流
"启动开发工作流，我要做 X"

# 或直接调用单个操作
gitflow issue create --title "修复登录 bug" --body "用户无法使用手机号登录"
```

---

## Phase 1 详细设计：Core + CLI 基础（MVP）

### 目标

为单一平台（GitHub）打通 Issue 和 PR 的核心读写链路，验证 Rust trait 抽象 + subprocess 调用原生 CLI 的架构可行性。

**不做**：close/reopen/comment/label 等扩展操作、GitLab/GitCode 平台、Skills 文件、编排层。

---

### 1. Workspace 变更

#### 1.1 Cargo.toml 新增 members

```toml
[workspace]
members = ["crates/*", "apps/*"]
# Phase 1 不变，crates/github 会被 glob 自动匹配
```

#### 1.2 新增 workspace dependencies

```toml
[workspace.dependencies]
# 现有依赖保持不变，新增：
async-trait = "0.1"
chrono = { version = "0.4", features = ["serde"] }
```

#### 1.3 crates/github/Cargo.toml

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

---

### 2. crates/core 扩展

#### 2.1 文件结构

```
crates/core/src/
├── lib.rs              # 现有：SafePath、Config、CoreError（保持不变）
├── platform.rs         # 新增：Platform 枚举 + 检测逻辑
├── types.rs            # 新增：共享 domain types
├── issue.rs            # 新增：IssueProvider trait + IssueData + args
├── pr.rs               # 新增：PrProvider trait + PrData + args
└── output.rs           # 新增：JSON 输出类型
```

#### 2.2 platform.rs — 平台枚举与检测

```rust
/// 平台标识
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Platform {
    /// GitHub
    GitHub,
    /// GitLab
    GitLab,
    /// GitCode
    GitCode,
}

impl Platform {
    /// 从 git remote URL 检测平台。
    ///
    /// 匹配规则按优先级：
    /// 1. 显式的 `--platform` flag → 函数外处理，不在此方法
    /// 2. URL 包含已知域名关键字
    /// 3. 无法识别 → 返回 `None`
    ///
    /// URL 格式示例：
    /// - `https://github.com/user/repo.git`
    /// - `git@github.com:user/repo.git`
    /// - `https://gitlab.example.com/user/repo.git`
    pub fn detect_from_remote_url(url: &str) -> Option<Self> {
        let url_lower = url.to_lowercase();

        if url_lower.contains("github.com") || url_lower.contains("github.") {
            Some(Self::GitHub)
        } else if url_lower.contains("gitlab.com") || url_lower.contains("gitlab.") {
            Some(Self::GitLab)
        } else if url_lower.contains("gitcode.com") || url_lower.contains("gitcode.") {
            Some(Self::GitCode)
        } else {
            None
        }
    }
}
```

#### 2.3 types.rs — 共享 domain types

```rust
/// 用户摘要
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserSummary {
    pub login: String,
    pub id: u64,
}

/// Issue/PR 状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Open,
    Closed,
}

/// 标签
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Label {
    pub name: String,
    pub color: Option<String>,
    pub description: Option<String>,
}
```

#### 2.4 issue.rs — IssueProvider trait

```rust
use async_trait::async_trait;
use crate::types::{Label, State, UserSummary};
use crate::Result;

/// Issue 数据
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IssueData {
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: State,
    pub labels: Vec<Label>,
    pub author: UserSummary,
    pub assignees: Vec<UserSummary>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub url: String,
}

/// 创建 Issue 参数
#[derive(Debug, Clone)]
pub struct CreateIssueArgs {
    pub title: String,
    pub body: Option<String>,
    pub labels: Vec<String>,
    pub assignees: Vec<String>,
}

/// 列出 Issue 参数
#[derive(Debug, Clone, Default)]
pub struct ListIssueArgs {
    pub state: Option<State>,
    pub labels: Vec<String>,
    pub assignee: Option<String>,
    pub search: Option<String>,
    pub limit: Option<u32>,
}

/// Issue 操作的平台抽象
#[async_trait]
pub trait IssueProvider: std::fmt::Debug + Send + Sync {
    /// 创建 Issue
    async fn create(&self, args: CreateIssueArgs) -> Result<IssueData>;

    /// 列出 Issue
    async fn list(&self, args: ListIssueArgs) -> Result<Vec<IssueData>>;

    /// 查看 Issue 详情
    async fn view(&self, number: u64) -> Result<IssueData>;
}
```

#### 2.5 pr.rs — PrProvider trait

```rust
use async_trait::async_trait;
use crate::types::{State, UserSummary};
use crate::Result;

/// PR 数据
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PrData {
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: State,
    pub draft: bool,
    pub author: UserSummary,
    pub base_branch: String,
    pub head_branch: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub url: String,
}

/// 创建 PR 参数
#[derive(Debug, Clone)]
pub struct CreatePrArgs {
    pub title: String,
    pub body: Option<String>,
    pub head: String,
    pub base: String,
    pub draft: bool,
    pub repo: Option<String>,
}

/// 列出 PR 参数
#[derive(Debug, Clone, Default)]
pub struct ListPrArgs {
    pub state: Option<State>,
    pub limit: Option<u32>,
}

/// PR 操作的平台抽象
#[async_trait]
pub trait PrProvider: std::fmt::Debug + Send + Sync {
    /// 创建 PR
    async fn create(&self, args: CreatePrArgs) -> Result<PrData>;

    /// 列出 PR
    async fn list(&self, args: ListPrArgs) -> Result<Vec<PrData>>;

    /// 查看 PR 详情
    async fn view(&self, number: u64) -> Result<PrData>;
}
```

#### 2.6 output.rs — JSON 输出类型

```rust
/// CLI 统一 JSON 输出格式
#[derive(Debug, Clone, serde::Serialize)]
pub struct CliOutput<T: serde::Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<CliError>,
    pub platform: String,
    pub command: String,
}

/// CLI 错误信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct CliError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

impl CliError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            hint: None,
        }
    }

    #[must_use]
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

impl<T: serde::Serialize> CliOutput<T> {
    pub fn success(data: T, platform: &str, command: &str) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            platform: platform.into(),
            command: command.into(),
        }
    }

    pub fn failure(error: CliError, platform: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            platform: platform.into(),
            command: String::new(),
        }
    }
}
```

---

### 3. crates/github 实现

#### 3.1 文件结构

```
crates/github/
├── Cargo.toml
└── src/
    ├── lib.rs         # pub mod 声明
    ├── issue.rs       # GitHubIssueProvider
    ├── pr.rs          # GitHubPrProvider
    └── error.rs       # gh CLI 错误解析
```

#### 3.2 核心模式：subprocess 调用 `gh`

每个 trait 方法通过 `tokio::process::Command` 调用 `gh` CLI，捕获 stdout 并解析 JSON：

```rust
// crates/github/src/issue.rs

use async_trait::async_trait;
use gitflow_cli_core::issue::{CreateIssueArgs, IssueData, IssueProvider, ListIssueArgs};
use gitflow_cli_core::Result;

use crate::error::{parse_gh_error, GhError};

/// GitHub Issue 提供者，通过 `gh` CLI 操作。
#[derive(Debug, Clone)]
pub struct GitHubIssueProvider {
    /// GitHub owner/repo，如 "byx-darwin/gitflow-cli"
    repo: String,
}

impl GitHubIssueProvider {
    /// 创建新的 GitHub Issue 提供者。
    ///
    /// `repo` 格式为 `owner/repo`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> Self {
        Self { repo: repo.into() }
    }
}

#[async_trait]
impl IssueProvider for GitHubIssueProvider {
    async fn create(&self, args: CreateIssueArgs) -> Result<IssueData> {
        let mut cmd = tokio::process::Command::new("gh");
        cmd.args(["issue", "create"])
            .arg("--repo").arg(&self.repo)
            .arg("--title").arg(&args.title)
            .arg("--json")
            .arg("number,title,body,state,labels,author,assignees,createdAt,updatedAt,url");

        if let Some(body) = &args.body {
            cmd.arg("--body").arg(body);
        }

        if !args.labels.is_empty() {
            cmd.arg("--label").arg(args.labels.join(","));
        }

        if !args.assignees.is_empty() {
            cmd.arg("--assignee").arg(args.assignees.join(","));
        }

        let output = cmd.output().await.map_err(|e| {
            gitflow_cli_core::CoreError::Platform(format!("Failed to spawn gh: {e}"))
        })?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(gitflow_cli_core::CoreError::Platform(format!("{gh_err}")));
        }

        let issue: IssueData = serde_json::from_slice(&output.stdout)
            .map_err(|e| gitflow_cli_core::CoreError::Serialization(e))?;

        Ok(issue)
    }

    async fn list(&self, args: ListIssueArgs) -> Result<Vec<IssueData>> {
        let mut cmd = tokio::process::Command::new("gh");
        cmd.args(["issue", "list"])
            .arg("--repo").arg(&self.repo)
            .arg("--json")
            .arg("number,title,body,state,labels,author,assignees,createdAt,updatedAt,url");

        if let Some(state) = &args.state {
            cmd.arg("--state").arg(match state {
                gitflow_cli_core::types::State::Open => "open",
                gitflow_cli_core::types::State::Closed => "closed",
            });
        }

        if let Some(ref search) = args.search {
            cmd.arg("--search").arg(search);
        }

        if let Some(limit) = args.limit {
            cmd.arg("--limit").arg(limit.to_string());
        }

        let output = cmd.output().await.map_err(|e| {
            gitflow_cli_core::CoreError::Platform(format!("Failed to spawn gh: {e}"))
        })?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(gitflow_cli_core::CoreError::Platform(format!("{gh_err}")));
        }

        let issues: Vec<IssueData> = serde_json::from_slice(&output.stdout)
            .map_err(|e| gitflow_cli_core::CoreError::Serialization(e))?;

        Ok(issues)
    }

    async fn view(&self, number: u64) -> Result<IssueData> {
        let output = tokio::process::Command::new("gh")
            .args(["issue", "view"])
            .arg(number.to_string())
            .arg("--repo").arg(&self.repo)
            .arg("--json")
            .arg("number,title,body,state,labels,author,assignees,createdAt,updatedAt,url")
            .output()
            .await
            .map_err(|e| {
                gitflow_cli_core::CoreError::Platform(format!("Failed to spawn gh: {e}"))
            })?;

        if !output.status.success() {
            let gh_err = parse_gh_error(&output.stderr);
            return Err(gitflow_cli_core::CoreError::Platform(format!("{gh_err}")));
        }

        let issue: IssueData = serde_json::from_slice(&output.stdout)
            .map_err(|e| gitflow_cli_core::CoreError::Serialization(e))?;

        Ok(issue)
    }
}
```

#### 3.3 gh 错误解析

```rust
// crates/github/src/error.rs

use std::fmt;

/// 解析 `gh` CLI 的 stderr 输出为结构化错误。
pub fn parse_gh_error(stderr: &[u8]) -> GhError {
    let text = String::from_utf8_lossy(stderr);

    // 尝试解析 gh 的 JSON 错误格式
    if let Ok(json) = serde_json::from_slice::<serde_json::Value>(stderr) {
        if let Some(msg) = json.get("message").and_then(|v| v.as_str()) {
            return GhError {
                message: msg.into(),
                code: json.get("code").and_then(|v| v.as_str()).map(String::from),
                hint: None,
            };
        }
    }

    // 回退：取 stderr 文本的前三行
    let message = text.lines().take(3).collect::<Vec<_>>().join("\n");
    GhError {
        message,
        code: None,
        hint: Some("Run 'gh auth status' to verify authentication.".into()),
    }
}

#[derive(Debug, Clone)]
pub struct GhError {
    pub message: String,
    pub code: Option<String>,
    pub hint: Option<String>,
}

impl fmt::Display for GhError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "gh: {}", self.message)?;
        if let Some(ref code) = self.code {
            write!(f, " [{code}]")?;
        }
        if let Some(ref hint) = self.hint {
            write!(f, "\nHint: {hint}")?;
        }
        Ok(())
    }
}
```

#### 3.4 lib.rs — 模块出口

```rust
// crates/github/src/lib.rs

#![forbid(unsafe_code)]

pub mod error;
pub mod issue;
pub mod pr;

pub use issue::GitHubIssueProvider;
pub use pr::GitHubPrProvider;
```

---

### 4. apps/cli 扩展

#### 4.1 文件结构

```
apps/cli/src/
├── main.rs              # 扩展 Cli 结构，新增 Issue/Pr 子命令
├── config.rs            # 现有，保持不变
├── commands/
│   ├── mod.rs           # 新增 issue、pr 模块
│   ├── run.rs           # 现有
│   ├── completions.rs   # 现有
│   ├── issue.rs         # 新增：gitflow issue <action>
│   └── pr.rs            # 新增：gitflow pr <action>
```

#### 4.2 main.rs — 扩展子命令

```rust
// 在 Cli 结构上新增全局 flags 和子命令

/// gitflow CLI 入口
#[derive(Debug, Parser)]
#[command(name = "gitflow", about = "跨平台 Git 工程化工作流编排工具")]
struct Cli {
    /// 覆盖平台自动检测
    #[arg(long, global = true)]
    platform: Option<PlatformArg>,

    /// 输出格式
    #[arg(long, global = true, default_value = "json")]
    output: OutputFormat,

    /// 详细输出
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum PlatformArg {
    GitHub,
    Gitlab,
    Gitcode,
}

#[derive(Debug, Clone, clap::ValueEnum, Default)]
enum OutputFormat {
    #[default]
    Json,
    Text,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Issue 操作
    #[command(subcommand)]
    Issue(IssueCommand),

    /// PR 操作
    #[command(subcommand)]
    Pr(PrCommand),

    /// 安装 skills
    #[command(name = "skills")]
    SkillsInstall,

    /// 生成 shell 补全
    #[command(hide = true)]
    Completions(Shell),
}
```

#### 4.3 commands/issue.rs

```rust
use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum IssueCommand {
    /// 创建 Issue
    Create {
        /// Issue 标题
        #[arg(long)]
        title: String,
        /// Issue 正文（短文本）
        #[arg(long)]
        body: Option<String>,
        /// 从文件读取正文
        #[arg(long)]
        body_file: Option<String>,
        /// 标签（逗号分隔）
        #[arg(long)]
        label: Vec<String>,
        /// 负责人
        #[arg(long)]
        assignee: Vec<String>,
    },
    /// 列出 Issue
    List {
        /// 状态过滤
        #[arg(long)]
        state: Option<String>,
        /// 搜索关键词
        #[arg(long)]
        search: Option<String>,
        /// 标签过滤
        #[arg(long)]
        label: Vec<String>,
        /// 最大返回数
        #[arg(long)]
        limit: Option<u32>,
    },
    /// 查看 Issue
    View {
        /// Issue 编号
        number: u64,
    },
}

/// 处理 Issue 子命令。
///
/// # Errors
///
/// 返回 `miette::Result`，在 CLI 层展示格式化的错误信息。
pub async fn handle(
    command: IssueCommand,
    platform: &str,
    repo: &str,
    output_format: OutputFormat,
) -> miette::Result<()> {
    // 根据 platform 选择对应的 provider
    let provider = match platform {
        "github" => GitHubIssueProvider::new(repo),
        // Phase 2-3 扩展其他平台
        other => return Err(miette::miette!("Platform '{other}' not yet supported")),
    };

    match command {
        IssueCommand::Create { title, body, body_file, label, assignee } => {
            let body = resolve_body(body, body_file)?;
            let args = CreateIssueArgs { title, body, labels: label, assignees: assignee };
            let issue = provider.create(args).await.map_err(|e| miette::miette!("{e}"))?;
            print_output(CliOutput::success(issue, platform, "issue create"), output_format);
        }
        IssueCommand::List { state, search, label, limit } => {
            let state = state.as_deref().map(|s| match s {
                "open" => State::Open,
                "closed" => State::Closed,
                _ => State::Open,
            });
            let args = ListIssueArgs { state, labels: label, assignee: None, search, limit };
            let issues = provider.list(args).await.map_err(|e| miette::miette!("{e}"))?;
            print_output(CliOutput::success(issues, platform, "issue list"), output_format);
        }
        IssueCommand::View { number } => {
            let issue = provider.view(number).await.map_err(|e| miette::miette!("{e}"))?;
            print_output(CliOutput::success(issue, platform, "issue view"), output_format);
        }
    }

    Ok(())
}
```

#### 4.4 commands/pr.rs

```rust
// 与 issue.rs 结构一致，实现 PrCommand 的 create/list/view 三个子命令
// create 额外参数：--head, --base, --draft
```

---

### 5. 平台检测入口

在 `main.rs` 的 `async_main` 中，解析 `platform` 和 `repo` 的流程：

```rust
/// 解析目标平台和仓库。
///
/// 优先级：
/// 1. `--platform` flag（显式覆盖）
/// 2. `git remote get-url origin` 自动检测
/// 3. 两者都失败 → 返回错误
fn resolve_platform(cli_platform: Option<PlatformArg>) -> miette::Result<(String, String)> {
    // 获取 git remote URL
    let remote_url = std::process::Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .map_err(|e| miette::miette!("Failed to get git remote URL: {e}\nAre you in a git repository?"))?;

    let remote_url = String::from_utf8_lossy(&remote_url.stdout).trim().to_string();

    // 确定平台
    let platform = match cli_platform {
        Some(p) => match p {
            PlatformArg::GitHub => "github",
            PlatformArg::Gitlab => "gitlab",
            PlatformArg::Gitcode => "gitcode",
        }.to_string(),
        None => {
            let detected = gitflow_cli_core::platform::Platform::detect_from_remote_url(&remote_url)
                .ok_or_else(|| miette::miette!(
                    "Unable to detect platform from remote URL: {}\nUse --platform to specify explicitly.",
                    remote_url
                ))?;
            format!("{:?}", detected).to_lowercase()
        }
    };

    // 提取 owner/repo
    let repo = extract_repo_from_url(&remote_url)
        .ok_or_else(|| miette::miette!("Unable to parse owner/repo from URL: {}", remote_url))?;

    Ok((platform, repo))
}

/// 从 git remote URL 提取 owner/repo。
fn extract_repo_from_url(url: &str) -> Option<String> {
    // HTTPS: https://github.com/owner/repo.git → owner/repo
    // SSH:   git@github.com:owner/repo.git      → owner/repo
    let without_prefix = url.trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("git@");
    let path = without_prefix
        .splitn(2, ':')
        .nth(1)
        .unwrap_or(without_prefix);
    let no_suffix = path.trim_end_matches(".git");
    let segments: Vec<&str> = no_suffix.split('/').collect();
    if segments.len() >= 2 {
        let owner = segments[segments.len() - 2];
        let repo = segments[segments.len() - 1];
        Some(format!("{owner}/{repo}"))
    } else {
        None
    }
}
```

---

### 6. 原生 CLI 前置检查

在确定目标平台后、执行任何命令前，检查对应的原生 CLI 是否可用且版本满足最低要求。不满足则阻断执行并给出安装指引。

#### 6.1 设计原则

| 原则 | 说明 |
|------|------|
| 只检查不安装 | `gitflow` 不负责下载或管理原生 CLI |
| 阻断式 | CLI 缺失或版本过低 → 拒绝执行，不降级、不回退 |
| 明确提示 | 给出对应平台的官方安装命令 |
| 平台对应 | 只检查当前目标平台需要的 CLI（GitHub → `gh`，GitLab → `glab`，GitCode → `gitcode`） |

#### 6.2 检查流程

```
resolve_platform()
       │
       ▼
check_prerequisites(platform)
       │
       ├─ which <cli> 是否在 PATH 上？
       │   └─ 否 → 错误: "gh not found. Install: brew install gh / apt install gh / ..."
       │
       ├─ <cli> --version 版本号是否 ≥ 最低要求？
       │   └─ 否 → 错误: "gh v2.0.0+ required, found v1.9.0. Upgrade: brew upgrade gh"
       │
       └─ 通过 → 继续执行命令
```

#### 6.3 实现

```rust
// apps/cli/src/prerequisites.rs

use std::process::Command;

/// 原生 CLI 版本要求
struct CliRequirement {
    /// CLI 可执行文件名
    binary: &'static str,
    /// 最低版本号（semver）
    min_version: &'static str,
    /// 安装指引链接
    install_url: &'static str,
    /// 常见安装命令（brew / apt / choco）
    install_hint: &'static str,
}

/// 平台 → CLI 要求映射
fn requirement_for(platform: &str) -> CliRequirement {
    match platform {
        "github" => CliRequirement {
            binary: "gh",
            min_version: "2.0.0",
            install_url: "https://github.com/cli/cli#installation",
            install_hint: "brew install gh   # macOS\napt install gh    # Ubuntu\nchoco install gh  # Windows",
        },
        "gitlab" => CliRequirement {
            binary: "glab",
            min_version: "1.30.0",
            install_url: "https://gitlab.com/gitlab-org/cli#installation",
            install_hint: "brew install glab   # macOS\napt install glab    # Ubuntu",
        },
        "gitcode" => CliRequirement {
            binary: "gitcode",
            min_version: "0.6.0",
            install_url: "https://gitcode.com/gitcode-cli/gitcode-cli",
            install_hint: "# 参考官方安装指引\ncurl -sSL https://gitcode.com/.../install.sh | bash",
        },
        _ => unreachable!(),
    }
}

/// 检查原生 CLI 是否可用且满足版本要求。
///
/// # Errors
///
/// CLI 不在 PATH 上 → `PREREQUISITE_NOT_FOUND`
/// CLI 版本过低 → `PREREQUISITE_VERSION_TOO_LOW`
pub fn check(platform: &str) -> Result<(), PrerequisiteError> {
    let req = requirement_for(platform);

    // 1. 检查 CLI 是否在 PATH 上
    let path = which::which(req.binary).map_err(|_| {
        PrerequisiteError::NotFound {
            binary: req.binary.into(),
            install_hint: req.install_hint.into(),
            install_url: req.install_url.into(),
        }
    })?;

    tracing::debug!("Found {} at {}", req.binary, path.display());

    // 2. 检查版本
    let version = get_version(req.binary)?;
    if !version_meets_minimum(&version, req.min_version) {
        return Err(PrerequisiteError::VersionTooLow {
            binary: req.binary.into(),
            found: version,
            required: req.min_version.into(),
            install_hint: req.install_hint.into(),
        });
    }

    tracing::debug!("{} version {} meets minimum {}", req.binary, version, req.min_version);
    Ok(())
}

/// 解析 `<cli> --version` 输出中的 semver。
fn get_version(binary: &str) -> Result<String, PrerequisiteError> {
    let output = Command::new(binary)
        .arg("--version")
        .output()
        .map_err(|_| PrerequisiteError::VersionParseFailed {
            binary: binary.into(),
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    // 提取 semver，适配各种 CLI 的输出格式：
    // gh version 2.50.0 (2024-01-01) → 2.50.0
    // glab version 1.35.0 (2024-01-01) → 1.35.0
    // gitcode version v0.6.0 → 0.6.0
    extract_semver(&stdout).ok_or_else(|| PrerequisiteError::VersionParseFailed {
        binary: binary.into(),
    })
}

/// 粗糙的 semver 提取：找到第一个 `X.Y.Z` 模式。
fn extract_semver(s: &str) -> Option<String> {
    let re = regex::Regex::new(r"\d+\.\d+\.\d+").ok()?;
    re.find(s).map(|m| m.as_str().to_string())
}

/// 简单 semver 比较：每个 segment 逐一对比。
fn version_meets_minimum(found: &str, minimum: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> {
        v.split('.')
            .filter_map(|s| s.parse().ok())
            .collect()
    };
    let f = parse(found);
    let m = parse(minimum);
    f.len() >= m.len() && f.iter().zip(&m).all(|(a, b)| a >= b)
}

#[derive(Debug, thiserror::Error)]
pub enum PrerequisiteError {
    #[error(
        "{binary} not found.\n\nInstall from {install_url}\n\nQuick install:\n{install_hint}"
    )]
    NotFound {
        binary: String,
        install_hint: String,
        install_url: String,
    },

    #[error(
        "{binary} v{required}+ required, found v{found}.\n\nUpgrade:\n{install_hint}"
    )]
    VersionTooLow {
        binary: String,
        found: String,
        required: String,
        install_hint: String,
    },

    #[error("Failed to parse {binary} --version output")]
    VersionParseFailed {
        binary: String,
    },
}
```

#### 6.4 async_main 中的调用位置

```rust
// 修改 async_main，在 resolve_platform 之后加一步：

async fn async_main(cli: Cli) -> miette::Result<()> {
    let (platform, repo) = resolve_platform(cli.platform)?;

    // 前置检查：原生 CLI 是否可用
    prerequisites::check(&platform).map_err(|e| miette::miette!("{e}"))?;

    match cli.command {
        Commands::Issue(cmd) => commands::issue::handle(cmd, &platform, &repo, cli.output).await,
        Commands::Pr(cmd) => commands::pr::handle(cmd, &platform, &repo, cli.output).await,
        // ...
    }
}
```

#### 6.5 用户视角的错误信息

```bash
$ gitflow issue list

错误: gh not found.

Install from https://github.com/cli/cli#installation

Quick install:
brew install gh   # macOS
apt install gh    # Ubuntu
choco install gh  # Windows
```

```bash
$ gitflow issue create --title "bug"

错误: gh v2.0.0+ required, found v1.9.0.

Upgrade:
brew upgrade gh   # macOS
```

#### 6.6 新增依赖

```toml
# workspace Cargo.toml
[workspace.dependencies]
which = "7"      # 查找可执行文件路径
regex = "1"      # 解析版本号（已在 workspace 依赖中）
```

---

### 8. crates/core 错误类型扩展

在现有 `CoreError` 上新增 variant：

```rust
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CoreError {
    // 现有 variants 保持不变...
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("application error: {0}")]
    App(String),
    #[error("path error: {0}")]
    Path(#[from] PathError),

    // Phase 1 新增
    /// 平台操作错误（CLI 执行失败、解析失败、认证失败等）。
    #[error("platform error: {0}")]
    Platform(String),
}
```

---

### 9. 冒烟测试

#### 7.1 scripts/smoke-test.sh（Phase 1 最小版本）

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "=== gitflow-cli Smoke Test (Phase 1 - GitHub) ==="

# 读取模式测试
echo "[1/3] issue view"
gitflow issue view 1 --platform github 2>&1 | head -5

echo "[2/3] issue list"
gitflow issue list --state open --limit 3 --platform github 2>&1 | head -5

echo "[3/3] pr list"
gitflow pr list --state open --limit 3 --platform github 2>&1 | head -5

echo "=== Smoke test passed ==="
```

#### 7.2 集成测试（Rust）

使用 `assert_cmd` 测试 CLI 输出：

```rust
// apps/cli/tests/issue_integration_test.rs
use assert_cmd::Command;

#[test]
fn test_should_list_issues_with_json_output() {
    let mut cmd = Command::cargo_bin("gitflow-cli").unwrap();
    cmd.args(["issue", "list", "--state", "open", "--limit", "1", "--platform", "github"]);
    let output = cmd.assert().success();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("\"success\":true"));
    assert!(stdout.contains("\"platform\":\"github\""));
    assert!(stdout.contains("\"command\":\"issue list\""));
}
```

---

### 10. Phase 1 不做

| 不做 | 理由 |
|------|------|
| close/reopen/comment/label 操作 | 留到 Phase 2 |
| `--body-file` 实现 | `CreateIssueArgs.body` 先只支持 `--body` |
| GitLab/GitCode 平台 | Phase 3 |
| `crates/core` 中的 Release/Auth/Pipeline traits | Phase 2-3 |
| Skills 文件 | Phase 4 |
| `gitflow skills install` | Phase 4 |
| `--platform` 自动检测失败时的 API 探测回退 | 当前域名字典覆盖主场景 |
| `--output text` 格式化输出 | Phase 1 只输出 JSON |

---

## 实现路线图

### Phase 1: Core + CLI 基础（MVP）

- [ ] 扩展 `crates/core`：Platform 枚举、Issue/PR trait、domain types、检测逻辑
- [ ] 新建 `crates/github`：Issue、PR 核心操作
- [ ] 扩展 `apps/cli`：`issue create/list/view`、`pr create/list/view` 命令
- [ ] 平台检测（git remote）
- [ ] 原生 CLI 前置检查（PATH + 版本最低要求）
- [ ] JSON 输出格式
- [ ] 冒烟测试脚本
- [ ] 错误自动报告：`error_reporter` 模块 + `hooks/auto-report-bug.sh` + pending.json 写入

### Phase 2: GitHub 完整支持

- [ ] 其余 Issue/PR 操作（close、reopen、comment、label）
- [ ] Release、Review、Auth 操作
- [ ] 核心 Skills (`gitflow-issue`、`gitflow-pr`、`gitflow-release`)
- [ ] 工作流 Skills (`gitflow-issue-create`、`gitflow-pr-create`、`gitflow-pr-review`)

### Phase 3: GitLab + GitCode

- [ ] 新建 `crates/gitlab`：完整操作实现
- [ ] 新建 `crates/gitcode`：完整操作实现
- [ ] pipeline 分析（通用接口，各平台实现）
- [ ] 冒烟测试覆盖三平台

### Phase 4: 编排层

- [ ] `gitflow-workflow` skill（全流程编排）
- [ ] `gitflow-quality` skill（质量关卡）
- [ ] `gitflow-autoreport-bug` skill（自动错误反馈）
- [ ] 与 Superpowers 集成指南
- [ ] install.sh 一键安装脚本

### Phase 5: 完成度提升

- [ ] 其余 skills（security-check、label-stats、precommit、regression 等）
- [ ] Shell 自动补全
- [ ] Homebrew formula
- [ ] 社区文档（CONTRIBUTING.md 等）
