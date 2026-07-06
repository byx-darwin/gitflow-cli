# gitflow-workflow 改造 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 改造 gitflow-workflow skill，整合从 open issues 读取需求的流程，实现完整计划闭环（开发+质量+交付+收尾），并同步更新 README.md。

**Architecture:** 修改 gitflow-workflow SKILL.md 文件，添加 open issues 读取、完整计划闭环、subagent 执行调用；同步更新 README.md 文档；删除被整合的冗余 skills。

**Tech Stack:** Markdown, bash, gitflow-cli

## Global Constraints

- 保持与现有 skills 的兼容性
- 确保计划文档包含完整闭环（开发/质量/交付/收尾）
- README.md 必须与 SKILL.md 保持一致
- 删除冗余 skills 前需确认无其他依赖

---

## Tasks

### Task 1: 修改 gitflow-workflow Phase 1 - 需求澄清

**Files:**
- Modify: `skills/gitflow-workflow/SKILL.md:1-100`
- Modify: `apps/cli/src/commands/skills.rs:160-175`

**Interfaces:**
- Consumes: 无
- Produces: 更新后的 SKILL.md Phase 1 部分

- [ ] **Step 1: 备份原始 SKILL.md**

```bash
cp skills/gitflow-workflow/SKILL.md skills/gitflow-workflow/SKILL.md.bak
```

- [ ] **Step 2: 替换 Phase 1 内容**

在 skills/gitflow-workflow/SKILL.md 中找到 `## Phase 1: 需求澄清` 部分，替换为：

```markdown
## Phase 1: 需求澄清

**目标**：从 open issues 读取需求，或手动描述需求，转化为结构化的 Issue。

### 步骤 1.1：读取 Open Issues

**完整模式**：读取所有 open issues（限制 100 个）
```bash
gitflow-cli issue list --state open --limit 100 --output json
```
按类型分组显示：feature/enhancement、bug、question/discussion。

**快速模式**：只读取 bug 类型 issues
```bash
gitflow-cli issue list --state open --label bug --limit 50 --output json
```
按优先级排序显示。

用户选择要处理的 issues，进入下一步。

### 步骤 1.2：需求讨论

**完整模式**：调用 `superpowers:brainstorming` 探索需求边界，产出：设计文档。

**快速模式**：直接分析 bug 原因，产出：修复方案（可选设计文档）。

### 步骤 1.3：创建 Issue

如果用户选择已有 issue，跳过创建。否则调用 `gitflow-issue-create` skill。

### 步骤 1.4：生成需求文档

**完整模式**：生成详细设计文档
- 背景、目标、技术方案、验收标准、风险

**快速模式**：生成简要修复方案
- 根因分析、修复步骤、测试计划

### Phase 1 产出
- 设计文档/修复方案
- Issue #N
- 需求讨论记录
```

- [ ] **Step 3: 提交修改**

```bash
git add skills/gitflow-workflow/SKILL.md
git commit -m "feat: 改造 Phase 1 整合 open issues 读取"
```

---

### Task 2: 修改 gitflow-workflow Phase 2 - 计划制定

**Files:**
- Modify: `skills/gitflow-workflow/SKILL.md:101-150`

**Interfaces:**
- Consumes: Phase 1 产出的设计文档
- Produces: 更新后的 SKILL.md Phase 2 部分

- [ ] **Step 1: 替换 Phase 2 内容**

在 SKILL.md 中找到 `## Phase 2: 开发实现` 部分，替换为：

```markdown
## Phase 2: 计划制定

**目标**：制定完整计划文档，包含开发、质量关卡、交付、收尾任务。

### 步骤 2.1：制定完整计划

调用 Superpowers writing-plans 制定计划。**计划文档必须包含完整闭环**：

```markdown
# 实现计划

## 任务清单

### Task 1: 创建 Issue（如需要）
- [ ] 创建 Issue 并保存编号

### Task 2: 开发任务
- [ ] TDD 循环
  - [ ] 写失败测试（RED）
  - [ ] 写最小实现（GREEN）
  - [ ] 重构优化（REFACTOR）
  - [ ] 验证：cargo test
- [ ] 代码审查
  - [ ] 调用 superpowers:requesting-code-review
  - [ ] 审查并修复问题
- [ ] 提交
  - [ ] 调用 gitflow-commit skill
  - [ ] git commit -m "feat: ... (#N)"

### Task N: 质量关卡

调用 `gitflow-quality` skill 运行完整质量检查：

```
使用 gitflow-quality 技能，对当前分支运行 6 项质量检查。
```

**gitflow-quality 自动处理**：
- 多语言检测（Rust/Node.js/Python/Go）
- 6 项检查（build/test/coverage/format/static/pre-commit）
- Fast-fail 策略（任何一步失败就停止）
- Quality Report 生成
- 自动发布到关联 Issue

**输出报告**：
```markdown
## Quality Report — YYYY-MM-DD

| Check    | Status | Details |
|----------|--------|---------|
| build    | ✅     | 0 errors, 0 warnings |
| test     | ✅     | 47 passed, 0 failed |
| coverage | ✅     | 85.3% (threshold: 80%) |
| format   | ✅     | No diff |
| static   | ✅     | No warnings |
| pre-commit | ✅     | All hooks passed |

**Result: ✅ ALL CHECKS PASSED — Ready for delivery**
```

### Task N+1: 交付
- [ ] 创建 PR：调用 gitflow-pr-create skill
- [ ] PR 审查：调用 gitflow-pr-review skill
- [ ] 审查反馈：调用 gitflow-pr-apply-feedback skill（如需要）
- [ ] 合并 PR：gitflow-cli pr merge

### Task N+2: 收尾
- [ ] 同步 Issue 状态为 done
- [ ] 关闭 Issue
- [ ] 更新验收标准
- [ ] 回归测试：调用 gitflow-regression skill
- [ ] 发布（可选）：调用 gitflow-release-helper skill
```

### Phase 2 产出
- 完整计划文档（包含开发/质量/交付/收尾任务）
```

- [ ] **Step 2: 提交修改**

```bash
git add skills/gitflow-workflow/SKILL.md
git commit -m "feat: 改造 Phase 2 整合完整计划闭环"
```

---

### Task 3: 修改 gitflow-workflow Phase 3 - 执行

**Files:**
- Modify: `skills/gitflow-workflow/SKILL.md:151-200`

**Interfaces:**
- Consumes: Phase 2 产出的计划文档
- Produces: 更新后的 SKILL.md Phase 3 部分

- [ ] **Step 1: 替换 Phase 3 内容**

在 SKILL.md 中找到 `## Phase 3: 质量关卡` 部分（旧），替换为新的 `## Phase 3: 执行`：

```markdown
## Phase 3: 执行

**目标**：按计划文档逐任务执行，完成代码实现和质量验证。

### 步骤 3.1：执行计划

调用 Superpowers subagent-driven-development 执行计划：
"使用 superpowers:subagent-driven-development 技能，执行完整计划文档。"

执行规则：
- 按计划文档逐任务执行
- 每个任务完成后标记 checkbox
- 遇到阻塞时暂停，不跳过任务
- 错误自动上报：调用 gitflow-autoreport-bug skill

### Phase 3 产出
- 代码实现
- 通过的测试
- 合并的 PR
- 关闭的 Issue
```

- [ ] **Step 2: 移除旧的 Phase 3 和 Phase 4**

删除旧的 `## Phase 3: 质量关卡` 和 `## Phase 4: 交付` 部分。

- [ ] **Step 3: 提交修改**

```bash
git add skills/gitflow-workflow/SKILL.md
git commit -m "feat: 改造 Phase 3 调用 subagent 执行"
```

---

### Task 4: 同步更新 README.md

**Files:**
- Modify: `README.md`

**Interfaces:**
- Consumes: SKILL.md
- Produces: 更新后的 README.md

- [ ] **Step 1: 更新 README.md 中的 gitflow-workflow 描述**

在 README.md 中找到 gitflow-workflow 相关部分，更新为：

```markdown
## gitflow-workflow

全流程开发编排，整合需求探索、计划制定、执行、质量关卡、交付和收尾。

### 工作模式

**完整模式**（默认）：适用于新功能开发
```
/gitflow-workflow
```

**快速模式**（--fast）：适用于 bug 修复
```
/gitflow-workflow --fast
```

### 流程

1. 需求探索：从 open issues 读取需求
2. 计划制定：生成完整计划文档（开发/质量/交付/收尾）
3. 执行：调用 subagent 执行计划
4. 交付后：流水线分析、代码审查等
```

- [ ] **Step 2: 提交修改**

```bash
git add README.md
git commit -m "docs: 同步更新 README.md 对齐 gitflow-workflow"
```

---

### Task 5: 验证和测试

**Files:**
- Test: 无
- Modify: 无

**Interfaces:**
- Consumes: 所有更新后的文件
- Produces: 验证报告

- [ ] **Step 1: 验证 SKILL.md 结构完整性**

```bash
grep -c "## Phase" skills/gitflow-workflow/SKILL.md
# 期望输出：3（Phase 1, Phase 2, Phase 3）
```

- [ ] **Step 2: 验证 README.md 一致性**

```bash
grep -A5 "gitflow-workflow" README.md | head -10
# 期望：包含完整模式和快速模式说明
```

- [ ] **Step 3: 验证冗余 skills 已删除**

```bash
ls skills/ | grep -E "issue-create|issue-review"
# 期望：无输出
```

- [ ] **Step 4: 提交验证结果**

```bash
git status
```

---

## Self-Review

**Spec coverage:**
- ✅ 从 open issues 读取需求 → Task 1
- ✅ 计划文档包含完整闭环 → Task 2
- ✅ 阶段 3 调用 subagent → Task 3
- ✅ README.md 同步更新 → Task 4
- ✅ 删除冗余 skills → Task 5

**Placeholder scan:** 无 TBD/TODO

**Type consistency:** 所有接口一致

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-07-06-gitflow-workflow-refactor.md`. Two execution options:

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

**Which approach?**
