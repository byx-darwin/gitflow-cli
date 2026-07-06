# gitflow-workflow skill 改造设计

**日期**：2026-07-06
**状态**：设计阶段
**作者**：gitflow-cli team

---

## 背景

当前 gitflow-workflow skill 已有完整的四阶段流程，但缺乏以下能力：

1. **从 open issues 读取需求**：用户需要手动描述需求，无法直接基于现有 issues 启动工作流
2. **计划文档不完整**：开发计划缺少质量关卡、交付、收尾等后续任务
3. **执行流程分散**：阶段 4 的执行逻辑不够清晰
4. **README.md 未对齐**：README.md 没有反映 gitflow-workflow 的最新流程和用法

目标：改造 gitflow-workflow skill，使其更加自动化和完整，并同步更新 README.md。

---

## 设计方案

### 核心思路

```
gitflow-workflow

完整模式:
├─ Phase 1: 需求澄清
│   ├─ 读取所有 open issues
│   ├─ 按类型分组（feature/enhancement/bug）
│   ├─ 用户选择要处理的 issues
│   ├─ 逐个讨论需求
│   └─ 产出：设计文档
│
├─ Phase 2: 计划制定
│   ├─ 调用 writing-plans
│   ├─ 生成完整计划文档
│   └─ 包含：开发 + 质量关卡 + 交付 + 收尾
│
└─ Phase 3: 执行
    └─ 调用 /subagent-driven-development

---

快速模式（--fast）:
├─ Phase 1: 需求确认
│   ├─ 读取 bug 类型 open issues
│   ├─ 按优先级排序
│   ├─ 用户选择要修复的 bug
│   └─ 产出：修复方案（可选设计文档）
│
├─ Phase 2: 计划制定（可选）
│   └─ 简单 bug 可跳过详细计划
│
└─ Phase 3: 执行
    ├─ 有计划：调用 /subagent-driven-development
    └─ 无计划：直接开发
```

---

## Phase 1: 需求澄清

### 步骤 1.1：读取 Open Issues

**完整模式**：读取所有 open issues

```bash
# 获取所有 open issues
gitflow-cli issue list --state open --limit 100 --output json
```

按类型分组显示：
- feature/enhancement 类 issues
- bug 类 issues
- question/discussion 类 issues

**快速模式**：只读取 bug 类型 issues

```bash
# 获取 bug 类型的 open issues
gitflow-cli issue list --state open --label bug --limit 50 --output json
```

按优先级排序显示。

用户选择要处理的 issues，进入下一步。

### 步骤 1.2：标签分析

调用 `gitflow-label-stats` 分析现有标签使用情况：

```
使用 gitflow-label-stats 技能，分析现有标签，识别未分类的 issues。
```

### 步骤 1.3：需求讨论

对选中的 issues 逐个讨论：

**完整模式**：
- 调用 `superpowers:brainstorming` 探索需求边界
- 产出：设计文档

**快速模式**：
- 直接分析 bug 原因
- 产出：修复方案

### 步骤 1.4：创建 Issue

如果用户选择的是已有 issue，则跳过创建。

如果需要创建新 issue，调用 `gitflow-issue-create` skill。

### 步骤 1.5：需求分析

调用 `gitflow-issue-review` 进行需求分析：

```
使用 gitflow-issue-review 技能，对 Issue #N 进行需求分析。
```

### 步骤 1.6：安全审计（可选）

调用 `gitflow-security-check` 进行安全审计：

```
使用 gitflow-security-check 技能，对需求进行安全审计。
```

### 步骤 1.7：发布审计日志

将 Phase 1 产出物评论到 Issue。

### Phase 1 产出

- 设计文档/修复方案
- Issue #N
- 需求讨论记录
- 安全审计报告（如执行）

---

## Phase 2: 计划制定

### 步骤 2.1：制定完整计划

调用 Superpowers writing-plans 制定计划。

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

### Phase 2 产出

- 完整计划文档（包含开发/质量/交付/收尾任务）

---

## Phase 3: 执行

### 步骤 3.1：执行计划

调用 Superpowers subagent-driven-development 执行计划：

```
使用 superpowers:subagent-driven-development 技能，执行完整计划文档。
```

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

---

## Phase 4: 交付后

### 步骤 4.1：流水线分析

调用 `gitflow-pipeline-analyzer` 分析 CI/CD 流水线：

```
使用 gitflow-pipeline-analyzer 技能，分析流水线健康状况。
```

### 步骤 4.2：仓库分析

调用 `gitflow-repo` 分析仓库状态：

```
使用 gitflow-repo 技能，分析仓库整体状态。
```

### 步骤 4.3：Issue 分类

调用 `gitflow-issue-triage` 对相关 issues 进行分类：

```
使用 gitflow-issue-triage 技能，对 issues 进行分类和优先级排序。
```

### 步骤 4.4：代码审查

调用 `gitflow-review` 进行整体代码审查：

```
使用 gitflow-review 技能，对整体变更进行代码审查。
```

### 步骤 4.5：周报生成（可选）

调用 `gitflow-weekly-report` 生成周报：

```
使用 gitflow-weekly-report 技能，生成开发周报。
```

### 步骤 4.6：仓库入门（可选）

调用 `gitflow-repo-onboarding` 更新仓库入门文档：

```
使用 gitflow-repo-onboarding 技能，更新仓库入门文档。
```

### Phase 4 产出

- 流水线分析报告
- 仓库状态报告
- Issue 分类报告
- 代码审查报告
- 周报（如执行）
- 更新的入门文档（如执行）

---

## 关键设计决策

### 为什么从 open issues 读取需求？

1. **减少重复工作**：issues 已经存在，不需要重新描述
2. **批量处理**：可以同时处理多个相关 issues
3. **可追溯性**：需求直接关联到 issue

### 为什么计划文档要包含完整闭环？

1. **避免遗漏**：确保质量关卡和交付不被跳过
2. **自动化执行**：subagent 可以按计划执行所有任务
3. **可追溯性**：每个任务都有明确的状态

### 为什么阶段 3 调用 subagent-driven-development？

1. **隔离开发环境**：subagent 在独立环境中执行
2. **并行执行**：独立任务可以并行处理
3. **错误隔离**：subagent 失败不影响主流程

---

## 实施计划

### Phase 1: 修改 Phase 1（需求澄清）

1. 添加"读取 Open Issues"步骤
2. 添加"需求讨论"步骤
3. 添加"生成需求文档"步骤
4. 创建/引用 Issue

### Phase 2: 修改 Phase 2（计划制定）

1. 定义完整计划文档模板
2. 包含开发/质量/交付/收尾任务
3. 调用 writing-plans 生成计划

### Phase 3: 修改 Phase 3（执行）

1. 调用 subagent-driven-development
2. 按任务清单逐项执行
3. 检查点验证

---

## 测试策略

### 单元测试
- 测试 open issues 读取功能
- 测试需求文档生成功能
- 测试计划文档模板

### 集成测试
- 测试完整模式流程
- 测试快速模式流程
- 测试 subagent 执行流程

---

## 验收标准

- [ ] 完整模式支持从所有 open issues 读取需求
- [ ] 快速模式支持从 bug 类型 issues 读取需求
- [ ] 需求文档生成正确（完整模式：详细设计；快速模式：修复方案）
- [ ] 计划文档包含完整闭环（开发/质量/交付/收尾）
- [ ] 阶段 3 正确调用 subagent-driven-development
- [ ] **README.md 同步更新，对齐 gitflow-workflow 最新流程**
- [ ] 完整的测试覆盖

---

## 参考资料

- 当前 gitflow-workflow SKILL.md
- ncgo-code-skills 项目中的 brainstorm-from-issue
- ncgo-code-skills 项目中的 writing-plans-with-issue
