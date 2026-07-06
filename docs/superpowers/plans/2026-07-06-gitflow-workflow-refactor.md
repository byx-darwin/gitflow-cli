# gitflow-workflow 改造 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 改造 gitflow-workflow skill，整合从 open issues 读取需求的流程，实现完整计划闭环（开发+质量+交付+收尾），并同步更新 README.md。

**Architecture:** 按照 gitflow-workflow 的四阶段流程（需求探索 → Issue 管理 → 计划制定 → 执行）重新组织 skill，支持完整模式和快速模式。

**Tech Stack:** Markdown, bash, gitflow-cli

## Global Constraints

- 保持与现有 skills 的兼容性
- 确保计划文档包含完整闭环（开发/质量/交付/收尾）
- README.md 必须与 SKILL.md 保持一致
- 保留 gitflow-issue-create 和 gitflow-issue-review skills

---

## Phase 1: 需求探索

### 任务 1.1：读取 Open Issues

**目标**：从仓库 open issues 读取需求，按类型分组显示。

**完整模式**：读取所有 open issues
```bash
gitflow-cli issue list --state open --limit 100 --output json
```
按类型分组：feature/enhancement、bug、question/discussion

**快速模式**：只读取 bug 类型 issues
```bash
gitflow-cli issue list --state open --label bug --limit 50 --output json
```
按优先级排序显示

**产出**：issues 列表，用户选择要处理的 issues

---

### 任务 1.2：需求讨论

**完整模式**：调用 `superpowers:brainstorming` 探索需求边界
- 逐个讨论选中的 issues
- 产出：设计文档

**快速模式**：直接分析 bug 原因
- 分析根因和修复方案
- 产出：修复方案（可选设计文档）

**产出**：设计文档/修复方案

---

### 任务 1.3：创建 Issue

如果用户选择已有 issue，跳过创建。

否则调用 `gitflow-issue-create` skill 创建 Issue。

**产出**：Issue #N

---

## Phase 2: Issue 管理

### 任务 2.1：Issue 分析

调用 `gitflow-issue-review` 对 Issue 进行结构化分析：
- 需求完整性
- 技术可行性
- 范围界定
- 依赖关系

**产出**：需求分析报告

---

### 任务 2.2：同步状态

将 Issue 状态同步为 in-progress。

**产出**：更新状态的 Issue

---

### 任务 2.3：发布审计日志

将 Phase 1-2 产出物评论到 Issue。

**产出**：Issue 评论

---

## Phase 3: 计划制定

### 任务 3.1：制定完整计划

调用 `superpowers:writing-plans` 制定计划。

**计划文档必须包含完整闭环**：

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
- [ ] Build 检查：cargo build --workspace
- [ ] Test 检查：cargo test --workspace
- [ ] Coverage 检查：cargo tarpaulin --workspace
- [ ] Format 检查：cargo +nightly fmt --check
- [ ] Static 检查：cargo clippy --workspace -- -D warnings
- [ ] Pre-commit 检查：调用 gitflow-precommit skill

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

**产出**：完整计划文档

---

## Phase 4: 执行

### 任务 4.1：执行计划

调用 `superpowers:subagent-driven-development` 执行计划：

```
使用 superpowers:subagent-driven-development 技能，执行完整计划文档。
```

执行规则：
- 按计划文档逐任务执行
- 每个任务完成后标记 checkbox
- 遇到阻塞时暂停，不跳过任务
- 错误自动上报：调用 gitflow-autoreport-bug skill

**产出**：
- 代码实现
- 通过的测试
- 合并的 PR
- 关闭的 Issue

---

### 任务 4.2：交付后检查

- 流水线分析：调用 `gitflow-pipeline-analyzer`
- Issue 分类：调用 `gitflow-issue-triage`
- 代码审查：调用 `gitflow-review`

**产出**：交付后报告

---

## 实施步骤

### Step 1: 修改 SKILL.md Phase 1

修改 `skills/gitflow-workflow/SKILL.md` 的 Phase 1 部分，整合 open issues 读取流程。

文件：`skills/gitflow-workflow/SKILL.md:75-120`

---

### Step 2: 修改 SKILL.md Phase 2

修改 SKILL.md 的 Phase 2 部分，整合完整计划闭环模板。

文件：`skills/gitflow-workflow/SKILL.md:121-180`

---

### Step 3: 修改 SKILL.md Phase 3

修改 SKILL.md 的 Phase 3 部分，调用 subagent-driven-development。

文件：`skills/gitflow-workflow/SKILL.md:181-220`

---

### Step 4: 同步 README.md

更新 README.md 中的 gitflow-workflow 描述。

文件：`README.md`

---

### Step 5: 验证

验证所有修改：
```bash
# 验证 Phase 数量
grep -c "## Phase" skills/gitflow-workflow/SKILL.md
# 期望：3

# 验证 README.md 一致性
grep -A5 "gitflow-workflow" README.md
```

---

## Self-Review

**Spec coverage:**
- ✅ 从 open issues 读取需求 → Phase 1
- ✅ Issue 管理 → Phase 2
- ✅ 计划制定包含完整闭环 → Phase 3
- ✅ 执行调用 subagent → Phase 4
- ✅ README.md 同步更新 → Step 4

**Placeholder scan:** 无 TBD/TODO

**Type consistency:** 所有接口一致

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-07-06-gitflow-workflow-refactor.md`. Two execution options:

**1. Subagent-Driven (recommended)** - 每个任务派发一个 fresh subagent，任务间审查

**2. Inline Execution** - 在当前会话中执行，批量执行带检查点

**Which approach?**
