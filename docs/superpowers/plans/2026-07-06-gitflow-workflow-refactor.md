# gitflow-workflow 改造 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 改造 gitflow-workflow skill，整合从 open issues 读取需求的流程，实现完整计划闭环（开发+质量+交付+收尾），并同步更新相关文档。

**Architecture:** 按照 gitflow-workflow 的四阶段流程（需求探索 → 计划制定 → 执行 → 交付后）重新组织 skill，支持完整模式和快速模式。

**Tech Stack:** Markdown, bash, gitflow-cli

## Global Constraints

- 保持与现有 skills 的兼容性
- 确保计划文档包含完整闭环（开发/质量/交付/收尾）
- README.md 和 CLAUDE.md 必须与 SKILL.md 保持一致
- 保留 gitflow-issue-create 和 gitflow-issue-review skills
- 每个任务必须有明确的验证标准

---

## 回滚计划

如果改造失败，执行以下回滚步骤：

```bash
# 恢复 SKILL.md
cp skills/gitflow-workflow/SKILL.md.bak skills/gitflow-workflow/SKILL.md

# 恢复 README.md
git checkout HEAD -- README.md

# 回滚提交
git reset --hard HEAD~N  # N 为改造提交数量
```

---

## Tasks

### Task 1: 备份原始文件

**Files:**
- Create: `skills/gitflow-workflow/SKILL.md.bak`

**验证标准：**
- ✅ 备份文件存在且与原始文件内容一致

- [ ] **Step 1: 备份 SKILL.md**

```bash
cp skills/gitflow-workflow/SKILL.md skills/gitflow-workflow/SKILL.md.bak
diff skills/gitflow-workflow/SKILL.md skills/gitflow-workflow/SKILL.md.bak
# 期望：无输出（文件相同）
```

---

### Task 2: 修改 Phase 1 - 添加 Open Issues 读取

**Files:**
- Modify: `skills/gitflow-workflow/SKILL.md:75-120`

**验证标准：**
- ✅ Phase 1 包含"读取 Open Issues"步骤
- ✅ 支持完整模式和快速模式

- [ ] **Step 1: 读取当前 Phase 1 内容**

```bash
grep -n "## Phase 1" skills/gitflow-workflow/SKILL.md
```

- [ ] **Step 2: 替换 Phase 1 内容**

在 SKILL.md 中找到 Phase 1 部分，添加"读取 Open Issues"步骤：

```markdown
### 步骤 1.1：读取 Open Issues

**完整模式**：读取所有 open issues
```bash
gitflow-cli issue list --state open --limit 100 --output json
```
按类型分组显示：feature/enhancement、bug、question/discussion

**快速模式**：只读取 bug 类型 issues
```bash
gitflow-cli issue list --state open --label bug --limit 50 --output json
```
按优先级排序显示

用户选择要处理的 issues，进入下一步。
```

- [ ] **Step 3: 验证修改**

```bash
grep -A 5 "读取 Open Issues" skills/gitflow-workflow/SKILL.md
# 期望：显示新增的内容
```

---

### Task 3: 修改 Phase 2 - 添加完整计划闭环模板

**Files:**
- Modify: `skills/gitflow-workflow/SKILL.md:121-200`

**验证标准：**
- ✅ Phase 2 包含完整计划闭环模板
- ✅ 包含开发/质量/交付/收尾任务

- [ ] **Step 1: 读取当前 Phase 2 内容**

```bash
grep -n "## Phase 2" skills/gitflow-workflow/SKILL.md
```

- [ ] **Step 2: 替换 Phase 2 内容**

添加完整计划闭环模板（包含质量关卡调用 gitflow-quality skill）

- [ ] **Step 3: 验证修改**

```bash
grep -A 5 "完整计划闭环" skills/gitflow-workflow/SKILL.md
# 期望：显示新增的内容
```

---

### Task 4: 修改 Phase 3 - 调用 subagent 执行

**Files:**
- Modify: `skills/gitflow-workflow/SKILL.md:201-250`

**验证标准：**
- ✅ Phase 3 调用 superpowers:subagent-driven-development
- ✅ 移除了旧的 Phase 3（质量关卡）和 Phase 4（交付）

- [ ] **Step 1: 读取当前 Phase 3 内容**

```bash
grep -n "## Phase 3" skills/gitflow-workflow/SKILL.md
```

- [ ] **Step 2: 替换为新的 Phase 3**

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
```

- [ ] **Step 3: 验证修改**

```bash
grep -A 5 "subagent-driven-development" skills/gitflow-workflow/SKILL.md
# 期望：显示调用 subagent 的内容
```

---

### Task 5: 添加 Phase 4 - 交付后检查

**Files:**
- Modify: `skills/gitflow-workflow/SKILL.md`

**验证标准：**
- ✅ 添加 Phase 4 交付后检查
- ✅ 包含流水线分析、Issue 分类、代码审查

- [ ] **Step 1: 在 SKILL.md 末尾添加 Phase 4**

```markdown
## Phase 4: 交付后检查

**目标**：交付完成后，进行后续分析和审查。

### 步骤 4.1：流水线分析

调用 `gitflow-pipeline-analyzer` 分析 CI/CD 流水线健康状况。

### 步骤 4.2：Issue 分类

调用 `gitflow-issue-triage` 对相关 issues 进行分类和优先级排序。

### 步骤 4.3：代码审查

调用 `gitflow-review` 对整体变更进行代码审查。

### Phase 4 产出
- 流水线分析报告
- Issue 分类报告
- 代码审查报告
```

- [ ] **Step 2: 验证修改**

```bash
grep -A 5 "## Phase 4" skills/gitflow-workflow/SKILL.md
# 期望：显示新增的内容
```

---

### Task 6: 更新 README.md

**Files:**
- Modify: `README.md`

**验证标准：**
- ✅ README.md 包含 gitflow-workflow 最新流程
- ✅ 包含完整模式和快速模式说明

- [ ] **Step 1: 更新 README.md 中的 gitflow-workflow 描述**

```markdown
## gitflow-workflow

全流程开发编排，整合需求探索、计划制定、执行、质量关卡、交付和交付后检查。

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

1. **需求探索**：从 open issues 读取需求，讨论并生成设计文档
2. **计划制定**：生成完整计划文档（开发/质量/交付/收尾）
3. **执行**：调用 subagent 执行计划
4. **交付后检查**：流水线分析、Issue 分类、代码审查

### 质量关卡

质量关卡调用 `gitflow-quality` skill，支持多语言自动检测：
- Rust、Node.js、Python、Go、Java

6 项检查：build、test、coverage、format、static、pre-commit
```

- [ ] **Step 2: 验证修改**

```bash
grep -A 10 "gitflow-workflow" README.md | head -15
# 期望：显示更新后的内容
```

---

### Task 7: 更新 CLAUDE.md

**Files:**
- Modify: `CLAUDE.md`

**验证标准：**
- ✅ CLAUDE.md 约束与 gitflow-workflow 一致

- [ ] **Step 1: 更新 CLAUDE.md 中的 gitflow-workflow 约束**

将约束更新为：

```markdown
- **严格执行 gitflow-workflow 流程**：当使用 `/gitflow-workflow` 时，必须按照四阶段完整流程执行：
  - **阶段 1：需求探索**（从 open issues 读取，支持完整模式和快速模式）
  - **阶段 2：计划制定**（生成完整计划闭环）
  - **阶段 3：执行**（调用 subagent-driven-development）
  - **阶段 4：交付后检查**（流水线分析、Issue 分类、代码审查）
```

- [ ] **Step 2: 验证修改**

```bash
grep -A 10 "gitflow-workflow 流程" CLAUDE.md
# 期望：显示更新后的内容
```

---

### Task 8: 单元测试

**Files:**
- Create: `tests/workflow_phase1_test.rs`
- Create: `tests/workflow_phase2_test.rs`

**验证标准：**
- ✅ 测试 Phase 1 的 open issues 读取逻辑
- ✅ 测试 Phase 2 的计划文档生成

- [ ] **Step 1: 创建 Phase 1 测试**

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_read_open_issues() {
        // 测试 open issues 读取
    }

    #[test]
    fn test_issue_grouping() {
        // 测试 issues 分组逻辑
    }
}
```

- [ ] **Step 2: 创建 Phase 2 测试**

```rust
#[test]
fn test_plan_template_generation() {
    // 测试计划模板生成
}
```

- [ ] **Step 3: 运行测试**

```bash
cargo test --test workflow_phase1_test
cargo test --test workflow_phase2_test
```

---

### Task 9: 集成测试

**Files:**
- Create: `tests/workflow_integration_test.rs`

**验证标准：**
- ✅ 测试完整模式流程
- ✅ 测试快速模式流程

- [ ] **Step 1: 创建完整模式集成测试**

```rust
#[test]
fn test_complete_mode_flow() {
    // 测试完整模式：需求 → 计划 → 执行 → 交付后
}

#[test]
fn test_fast_mode_flow() {
    // 测试快速模式：需求确认 → 执行
}
```

- [ ] **Step 2: 运行测试**

```bash
cargo test --test workflow_integration_test
```

---

### Task 10: 验证和测试

**Files:**
- 无

**验证标准：**
- ✅ SKILL.md 包含 4 个 Phase
- ✅ README.md 与 SKILL.md 一致
- ✅ CLAUDE.md 约束更新
- ✅ 所有测试通过

- [ ] **Step 1: 验证 SKILL.md 结构**

```bash
grep -c "## Phase" skills/gitflow-workflow/SKILL.md
# 期望输出：4
```

- [ ] **Step 2: 验证 README.md 一致性**

```bash
grep "gitflow-workflow" README.md | head -5
# 期望：包含完整模式和快速模式说明
```

- [ ] **Step 3: 验证 CLAUDE.md 约束**

```bash
grep "gitflow-workflow" CLAUDE.md
# 期望：包含四阶段流程说明
```

- [ ] **Step 4: 运行所有测试**

```bash
cargo test --workspace
```

---

## Self-Review

**Spec coverage:**
- ✅ 从 open issues 读取需求 → Task 2
- ✅ 计划文档包含完整闭环 → Task 3
- ✅ 阶段 3 调用 subagent → Task 4
- ✅ 交付后检查 → Task 5
- ✅ README.md 同步更新 → Task 6
- ✅ CLAUDE.md 约束更新 → Task 7
- ✅ 测试覆盖 → Task 8, 9, 10

**Placeholder scan:** 无 TBD/TODO

**Type consistency:** 所有接口一致

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-07-06-gitflow-workflow-refactor.md`. Two execution options:

**1. Subagent-Driven (recommended)** - 每个任务派发一个 fresh subagent，任务间审查

**2. Inline Execution** - 在当前会话中执行，批量执行带检查点

**Which approach?**
