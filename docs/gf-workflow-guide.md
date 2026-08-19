# gf Workflow — 完整流程指南

基于 `gf-workflow` 四阶段闸门编排器的实操指南。配合 [CLAUDE.md](../CLAUDE.md) 的强制规则阅读。

## TL;DR

```
┌─────────────┐   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐
│  Phase 1    │──▶│  Phase 2    │──▶│  Phase 3    │──▶│  Phase 4    │
│  需求澄清    │   │  计划制定    │   │  执行        │   │  交付后检查  │
│             │   │             │   │             │   │             │
│ • brainstorm│   │ • plan      │   │ • TDD       │   │ • pipeline  │
│ • issue     │   │ • quality   │   │ • subagent  │   │ • triage    │
│ • review    │   │   gate      │   │ • PR        │   │ • review    │
└─────────────┘   └─────────────┘   └─────────────┘   └─────────────┘
         ▲                                                 │
         └─────────── 若质量不过关，返回 Phase 3 ─────────────┘
```

三种模式：

| 模式 | 适用场景 | 必选子技能 |
|---|---|---|
| **完整模式 (full)** | 新功能 / 大重构 / 跨模块 | 全部 7 个 |
| **标准模式 (standard)** | 中等复杂度 / 单模块改动 | 6 个（Phase 4 简化：无 triage/dogfooding） |
| **快速模式 (fast)** | Bug fix / 小改动 / typo | 4 个（Phase 1 issue-create、Phase 3 subagent、Phase 4 pipeline + branch-finish） |

## 技能来源适配（Issue #141）

gf-workflow 支持两种外部技能来源：**superpowers** 与 **mattpocock/skills**。启动时自动检测
（技能清单探测 + 哨兵规则），结果写入契约 `skill_source`，跨会话沿用。

| | superpowers（全自动流水线） | mattpocock（人工驾驶流水线） |
|---|---|---|
| Phase 1 澄清 | `brainstorming` | `grilling` → ✋ `/to-spec`（只写本地） |
| Phase 1 Issue | `gf-issue-create` | `gf-issue-create`（创建权统一，不重复建） |
| Phase 2 计划 | `writing-plans` | ✋ `/to-tickets`（票据图 + blocking edges） |
| Phase 3 执行 | SDD / executing-plans / 后台代理（GO 闸门选择） | ✋ `/implement` 逐票据（内部 `/tdd`） |
| Phase 4 交付 | gf-* 骨架 | 完全相同 |
| 人的触点 | 2 个（Gate 2→3、Branch Finish） | 5 个（✋×3 + 审批 + 确认） |

**安装前置**：`gf skills install` 会检测来源，两者皆无时硬阻断并给出安装引导
（`claude plugins install superpowers` / `claude plugins install mattpocock-skills` /
`npx skills@latest add mattpocock/skills`）。其中 `mattpocock-skills` 与 `npx skills`
两条路径要求 **Node.js ≥ 22.20.0**，安装前先 `node --version` 确认。

**Phase 3 GO 闸门**：Gate 2→3 批准后选择执行模式——后台代理（默认，仅 superpowers）/
手动新窗口 / 同会话（仅显式要求）。mattpocock 来源菜单自动裁剪。

检测规则、映射表与分支细节：`skills/gf-workflow/references.md`（单点维护）。

## 前置条件

```bash
# 安装 gf
cargo install --path apps/cli
# 或 brew install byx-darwin/gitflow-cli/gf

# 认证（GitHub / GitLab / GitCode 任选其一）
gf auth status
gf auth login --platform github

# 确认在 git 仓库内
git rev-parse --show-toplevel

# 安装技能来源（superpowers / mattpocock 任选其一；后者需 Node.js ≥ 22.20.0）
claude plugins install superpowers

# 安装本仓库的 skills（一次性）
gf skills install --agent claude --force
# 或 codex / gemini / open-code / copilot
```

## Phase 1：需求澄清

**目标**：从 Open Issues 提炼需求 → 产出结构化 Issue。

### 步骤

```bash
# 1.1 列出所有 Open Issues
gf issue list --state open --output json

# 1.2 讨论需求（完整模式必须调 brainstorming）
#    快速模式：直接分析 bug 根因，可跳过 brainstorming

# 1.3 创建 Issue（必选）
gf issue create \
  --title "fix(skills): 项目级 install 不识别 --agent" \
  --body "$(cat <<'EOF'
## 背景
<问题描述>

## 现状代码证据
- `path/to/file.rs:LINE` — 具体行号与现象

## 需求
### P1: ...
### P2: ...

## 验收标准
- [ ] ...
EOF
)"

# 1.4 需求审计（完整模式必须调 issue-review）
#    快速模式可跳过
```

### 关键产出

- ✅ Issue URL 与编号（例：[#62](https://github.com/byx-darwin/gitflow-cli/issues/62)）
- ✅ 验收标准（可机械验证）
- ✅ 标签（`kind/bug`、`area/cli` 等）

### 禁止行为

- ❌ 跳过 Issue 创建（快速模式也要建）
- ❌ 模糊的验收标准（"体验更好"）

## Phase 2：计划制定

**目标**：产出可执行的实现计划 + quality gate 全绿承诺。

### 输入

| 来源 | 内容 |
|---|---|
| Phase 1 产出 | Issue URL / 编号 / 验收标准 / 需求分析报告（`gf-issue-review` 生成） |
| 代码上下文 | 相关文件清单（`rg`/`fd`/LSP 扫出来） |
| CLAUDE.md 约束 | TDD、错误处理、依赖策略等硬规则 |

### 步骤

#### 2.1 改动面盘点

```
输入：Issue 验收标准 + 代码上下文
输出：改动文件清单（最小闭合）
```

产出表格：

```markdown
| # | 文件 | 改动性质 | 备注 |
|---|------|---------|------|
| 1 | `apps/cli/src/commands/skills.rs` | 实现 + 单测 | 主战场 |
| 2 | `.gitignore` | 配置 | 顺带修 |
```

#### 2.2 TDD 顺序设计

```
RED 阶段：先写哪些失败测试？
GREEN 阶段：实现 API 的最小集合
REFACTOR 阶段：哪些可以合并清理
```

产出（像这样列）：

```
RED:    新增 5 个单测（覆盖 P1/P2 的关键路径）
        → cargo test 应报 7 个编译错误（引用未实现的 API）
GREEN:  实现 supports_hooks() + resolve_project_target()
        → 685/685 测试通过
REFACTOR: 死分支 / 冗余 detect() / unreachable fallback 清理
```

#### 2.3 Quality Gate 承诺

从 `gf-quality` 的 6 道闸门里挑出**本次必须过的**：

| Gate | 验证命令 | 必过？ |
|---|---|---|
| Build | `make build` 或 `cargo build` | ✅ 必过 |
| Test | `make test` 或 `cargo test` | ✅ 必过，含新增测试 |
| Coverage | 关键路径必须有单测（无硬性 %） | ⚠️ 关键路径即可 |
| Format | `make fmt` 或 `cargo +nightly fmt` | ✅ pre-commit 强校验 |
| Static | `make clippy` 或 `cargo clippy -- -D warnings` | ✅ `-D warnings -W pedantic` |
| Pre-commit | 提交时自动触发（cargo-fmt / typos / gitleaks / cargo-deny） | ✅ 自动 |

#### 2.4 闸门与回退

```
Gate 2 → 3（计划 → 执行）的必备证据：
  ✓ Issue URL 可访问           gf issue view <n>
  ✓ 需求分析已作为 comment 回贴  gf issue view <n> 看评论
  ✓ 计划文档存在               路径或内联计划
  ✓ Quality gate 任务已列入计划   plan.md 中 Task N+1

执行中失败 → 回退策略：
  • 测试失败：留在 RED/GREEN 阶段继续修
  • Clippy 失败：REFACTOR 阶段处理
  • Pre-commit 失败：独立 commit 修掉 pre-existing 问题
    （参考 Issue #62 实战：cargo-fmt 拦 pre-existing pr.rs 格式问题
    → 独立 commit ad59e3f 修掉，让后续 commit 通过）
```

### 输出：计划文档

最终产出一份结构化 Markdown（模板见 [`templates/workflow-plan.md`](templates/workflow-plan.md)）：

```markdown
# [Feature/Bug] 执行计划

## Task List
### Task 1-3: Issue 管理
- [ ] 建 issue / 设状态 in-progress / 更新描述

### Task 4-N: 开发（每个任务包含）
- [ ] TDD cycle: RED → GREEN → REFACTOR
- [ ] Code review: 调 requesting-code-review + 修 findings
- [ ] Commit: git add + commit -m "..."

### Task N+1: Quality Gate
- [ ] 调 gf-quality 跑 6 项检查
- [ ] 报告 = ALL CHECKS PASSED
- [ ] 失败：修 → 重跑

### Task N+2: 交付
- [ ] 建 PR
- [ ] PR review
- [ ] Merge

### Task N+3: 收尾
- [ ] Issue 标 done + close
- [ ] 验收标准全部 ✅
```

### 实战案例：Issue #62 计划

完整模式 vs 快速模式对比：

| 步骤 | 完整模式 | 快速模式（#62 实际走法） |
|---|---|---|
| brainstorming | ✅ 探索需求边界 | ❌ 跳过（bug 已定位） |
| issue-create | ✅ 建 #62 | ✅ 建 #62 |
| issue-review | ✅ 需求审计 | ❌ 跳过（bug 明确） |
| writing-plans | ✅ 完整 plan.md | ❌ 内联计划（聊天中列出 5 项改动点） |
| Quality Gate 承诺 | 6 项全过 | 4 项（build/test/fmt/clippy；coverage 用关键路径测试代替） |

### 关键产出

- ✅ 改动文件清单（最小闭合）
- ✅ TDD 顺序（先写哪些测试，预期失败形式）
- ✅ Quality Gate 承诺（哪些命令会跑，必过/关键/可跳过）
- ✅ 闸门证据清单（可机械验证）
- ✅ 回退策略（失败时怎么走，不卡死）

## Phase 3：执行

**目标**：TDD 循环 → Code Review → PR。

### 步骤

```bash
# 3.1 启动 subagent-driven-development 循环
#    必选：TDD + Code Review 都不能跳

# 3.2 TDD 循环
#    RED:   写失败测试 → make test 报错
#    GREEN: 写最小实现 → make test 通过
#    REFACTOR: make clippy + make fmt 干净

# 3.3 E2E 验证（若涉及 CLI 行为）
cargo build --bin gf
./target/debug/gf <scenario>

# 3.4 Code Review（subagent）
#    必选：至少跑一次代码审查

# 3.5 提交 + 建 PR（需用户确认）
git checkout -b fix/issue-<N>-<short-name>
git add <files>
git commit -m "fix(<area>): <subject>

Fixes #<N>

<详细说明>"
git push -u origin HEAD
gf pr create --head HEAD --base main \
  --title "<commit subject>" \
  --body "<PR body>"
```

### 关键产出

- ✅ 全绿：Build / Test / Format / Clippy / Pre-commit
- ✅ PR URL（含 `Fixes #N` 自动关联）
- ✅ Pre-commit hook 全过（cargo-fmt / typos / gitleaks）
- ✅ Pre-push hook 全过（cargo-clippy / cargo-test）

### 实战案例：Issue #62

```
RED:    写 5 个新测试 → cargo test 报 7 个编译错误（引用未实现的 API）
GREEN:  实现 supports_hooks() + resolve_project_target()
        685/685 测试通过
REFACTOR: make clippy 干净；cargo +nightly fmt 干净
E2E:    --agent codex → .codex/ 全到位；--agent gemini → 跳过 hook
PR:     #63 squash merge 到 main
```

详见 [PR #63](https://github.com/byx-darwin/gitflow-cli/pull/63) 与 [Issue #62](https://github.com/byx-darwin/gitflow-cli/issues/62)。

## Phase 4：交付后检查

**目标**：三份报告（流水线分析 / Issue 分类 / 代码审查）+ 闸门复核。

### 步骤

```bash
# 4.1 Quality Gate 复核（Phase 3 已通过，再确认一次）
make build && make test && make fmt && make clippy

# 4.2 流水线分析（必选）
#    调用 gf-pipeline-analyzer：
#    - 列出 PR 触发的所有 CI workflow 运行
gf pipeline status --branch fix/issue-62-skills-agent-matrix

# 4.3 Issue 分类（必选）
#    调用 gf-issue-triage：
#    - 给本次改动的 issue 打标签 / 分类 / 更新状态

# 4.4 代码审查报告（必选）
#    调用 gf-review：
#    - 综合 Phase 3 的审查 findings
#    - 输出最终审查结论

# 4.5 闸门复核
#    - 所有 CI 通过？
#    - 所有测试通过？
#    - 所有审查 findings 已处理（或有明确 defer 理由）？
#    - 全部 ✅ → 可 merge
#    - 任一 ❌ → 返回 Phase 3

# 4.6 Merge（需用户确认策略）
gf pr merge <N> --strategy squash   # 推荐
gf pr merge <N> --strategy merge    # 保留 commit 历史
gf pr merge <N> --strategy rebase   # 线性历史

# 4.7 收尾
git checkout main && git pull --ff-only origin main
git branch -d fix/issue-<N>-<short-name>
```

### 关键产出

- ✅ 流水线分析报告（CI 状态）
- ✅ Issue 分类报告（labels / priority / area）
- ✅ 代码审查报告（无阻塞性 findings）
- ✅ PR merged + Issue auto-closed
- ✅ 本地 main 已同步

### Phase 4 步骤矩阵（按模式）

| # | 步骤 | Full | Standard | Fast |
|---|------|------|----------|------|
| 1 | 流水线分析 | ✅ | ✅ | ✅ |
| 2 | Issue 分类 | ✅ | ❌ | ❌ |
| 3 | 代码审查报告 | ✅ | ✅ | ❌ |
| 4 | Dogfooding 检查 | ✅ | ❌ | ❌ |
| 5 | Branch Finish | ✅ | ✅ | ✅ |

**执行流程：**
- Full: Pipeline → Triage → Review → Dogfooding → Branch Finish → Archive
- Standard: Pipeline → Review → Branch Finish → Archive
- Fast: Pipeline → Branch Finish → Archive

## 模式对比速查

| 维度 | 完整模式 (full) | 标准模式 (standard) | 快速模式 (fast) |
|---|---|---|---|
| Phase 1 | brainstorming ✅ + issue-create ✅ + issue-review ✅ | brainstorming ✅ + issue-create ✅ + issue-review ✅ | issue-create ✅ |
| Phase 2 | writing-plans ✅ + 完整 quality gate | writing-plans ✅ + 完整 quality gate | 可内联计划；quality gate 不变 |
| Phase 3 | subagent-driven ✅ + TDD ✅ + review ✅ | subagent-driven ✅ + TDD ✅ + review ✅ | 同左 |
| Phase 4 | pipeline ✅ + triage ✅ + review ✅ + dogfooding ✅ + branch-finish ✅ | pipeline ✅ + review ✅ + branch-finish ✅ | pipeline ✅ + branch-finish ✅ |
| 适用 | 新功能 / 大重构 / 跨模块 | 中等复杂度 / 单模块 | bug fix / 单文件改动 / 配置调整 |

## 常用命令速查

```bash
# Issue 操作
gf issue list --state open --output json
gf issue view <N>
gf issue create --title "..." --body "..."

# PR 操作
gf pr list --state open
gf pr create --head <branch> --base main --title "..." --body "..."
gf pr merge <N> --strategy squash
gf pr view <N>

# Pipeline 操作
gf pipeline status --branch <branch>
gf pipeline logs <run-id>

# Skills 管理
gf skills install --agent <claude|codex|gemini|open-code|copilot> --force
gf skills list --agent <agent>
gf skills uninstall --agent <agent>

# 认证
gf auth status
gf auth login --platform <github|gitlab|gitcode>
```

## 常见陷阱

| 陷阱 | 现象 | 解决 |
|---|---|---|
| 跳过 Phase 4 | PR 看起来能合但漏审查 | Phase 4 是强制闸门，无论快慢模式都不能跳 |
| 跳过 TDD | Code review 没测试覆盖 | TDD 在所有模式下都是必选 |
| 不跑 `cargo fmt` | pre-commit hook 失败 | `make fmt` 在 commit 前跑一次 |
| 不跑 `cargo clippy` | pre-push hook 失败 | `make clippy` 在 push 前跑一次 |
| 在 main 上直接 commit | 违反分支策略 | `git checkout -b fix/issue-N-name` 后再改 |
| `skills install` 后忘 commit `.gitignore` | `.codex/` 等污染仓库 | 首次安装后立刻检查 `git status` |
| 不写 `Fixes #N` | Issue 不自动关闭 | commit message 或 PR body 里写 |
| pre-commit hook 拦 pre-existing 改动 | 别人的格式问题挡你 commit | 先独立 commit 修掉 pre-existing 问题 |

## 相关文件

- [CLAUDE.md](../CLAUDE.md) — agent 强制规则（gitflow workflow 纪律）
- [integration-guide.md](integration-guide.md) — Superpowers 集成指南
- [tdd.md](tdd.md) — TDD 工作流细节
- [pre-commit-usage.md](pre-commit-usage.md) — pre-commit hook 配置
- [release.md](release.md) — 发布流程
- [templates/workflow-phases-detail.md](templates/workflow-phases-detail.md) — 四阶段详细模板
- [templates/workflow-plan.md](templates/workflow-plan.md) — 计划模板
