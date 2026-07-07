# Workflow Phase 1-4 详细描述

> 由 `gitflow-workflow` skill 引用，包含四阶段完整执行细节。

## Phase 1: 需求澄清

**目标**：从 open issues 读取需求，或手动描述需求，转化为结构化的 Issue。

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

### 步骤 1.2：需求讨论

对选中的 issues 逐个讨论：

**完整模式**：
- 调用 `superpowers:brainstorming` 探索需求边界
- 产出：设计文档

**快速模式**：
- 直接分析 bug 原因
- 产出：修复方案（可选设计文档）

### 步骤 1.3：创建 Issue

调用 `gitflow-issue-create` skill 创建 Issue：

```
使用 gitflow-issue-create 技能，根据讨论产出创建 Issue。
```

关键产出：
- Issue URL
- Issue 编号

### 步骤 1.4：生成需求文档

**完整模式**：调用 `gitflow-issue-review` 进行需求分析，生成需求文档

```
使用 gitflow-issue-review 技能，对 Issue #N 进行需求分析。
```

产出：需求分析报告（Markdown 格式）
- 需求完整性分析
- 技术可行性评估
- 验收标准确认
- 风险识别

**快速模式**：生成简要修复方案文档

```markdown
## Bug 修复方案

### 问题描述
<从 issue 读取的问题描述>

### 根因分析
<分析 bug 的根本原因>

### 修复方案
<具体的修复步骤>

### 测试计划
<回归测试方案>

### 验收标准
- [ ] 标准 1
- [ ] 标准 2
```

### 步骤 1.5：发布审计日志

将 Phase 1 产出物评论到 Issue：

```bash
# 写入临时文件
cat > /tmp/phase-report.md << 'REPORT'
## Phase 1: 需求澄清完成

### 需求文档
- 设计文档/修复方案：<路径或链接>
- 验收标准：<N> 项

### 产出物
- Issue URL: <url>
- 需求文档：已生成

✅ 已通过阶段闸门，可进入 Phase 2: 计划制定
REPORT

gitflow-cli issue comment <number> --body-file /tmp/phase-report.md
rm -f /tmp/phase-report.md
```

---

### 🔒 闸门：需求 → 计划

**必须出示的证据**：
1. Issue URL（可访问的链接）
2. 需求分析报告（评论在 Issue 上）

**校验方法**：
```bash
# 确认 Issue 存在且可访问
gitflow-cli issue view <number>
```

**违规处理**：
如果缺少任一证据，输出阻断信息并停止：

```
🔒 闸门未通过：不允许开始开发。必须先完成需求澄清阶段。

缺少以下证据：
- [ ] Issue URL：未创建或未记录
- [ ] 需求分析报告：未发布到 Issue 评论
```

### Phase 1 合规检查

```
Phase 1 合规检查:
  [ ] 步骤 1: brainstorming — 明确的功能描述、验收标准、边界条件
  [ ] 步骤 2: issue-create — Issue URL 已获取
  [ ] 步骤 3: issue-review — 需求分析报告已完成
  [ ] Issue 评论已发布: <comment_link>

  缺失项: <list or "无">
  是否允许进入下一阶段: [ ]
```

---

## Phase 2: 计划制定

**目标**：制定完整计划文档，包含所有开发任务和质量关卡。

### 步骤 2.1：制定完整计划

调用 Superpowers writing-plans 制定计划，**计划文档必须包含完整闭环**：

```
使用 superpowers:writing-plans 技能，基于 Issue #N 的需求文档，制定完整实现计划。
```

### 步骤 2.2：创建 worktree 和分支

为了隔离开发环境，需要在独立的 worktree 中执行任务：

```bash
# 确定 worktree 路径和工作分支名
WORKTREE_DIR="../gitflow-cli-<issue-number>"
BRANCH_NAME="fix/issue-<issue-number>"  # 或 feat/issue-<number>

# 创建 worktree 和分支
git worktree add "$WORKTREE_DIR" -b "$BRANCH_NAME"

# 切换到 worktree 目录
cd "$WORKTREE_DIR"

# 验证当前分支
git branch --show-current
# 输出应为: fix/issue-<issue-number>
```

**注意：**
- 如果在 worktree 中工作，后续所有操作都在该 worktree 目录中进行
- 原始仓库目录保持不变，可以继续在 main 分支工作
- 多个任务可以使用不同的 worktree 并行执行

**计划文档结构**：

```markdown
# [Feature Name] Implementation Plan

## 任务清单

### Task 1-3: Issue 管理
- [ ] Task 1: 创建 Issue
- [ ] Task 2: 同步状态为 in-progress
- [ ] Task 3: 更新 Issue 描述（如需要）

### Task 4-N: 开发任务（每个任务包含）
- [ ] TDD 循环
  - [ ] 写失败测试（RED）
  - [ ] 写最小实现（GREEN）
  - [ ] 重构优化（REFACTOR）
  - [ ] 验证：运行测试确认通过
- [ ] 代码审查
  - [ ] 调用 superpowers:requesting-code-review
  - [ ] 审查并修复问题
- [ ] 提交
  - [ ] git add -A
  - [ ] git commit -m "feat: ... (#N)"

### Task N+1: 质量关卡（完整计划闭环）
- [ ] 调用 gitflow-quality 技能，运行 6 项质量检查
  ```
  使用 gitflow-quality 技能，对当前分支运行 6 项质量检查。
  ```
  检查项（语言自动检测，非 Rust 项目自动适配）：
  - Build 检查
  - Test 检查
  - Coverage 检查（默认 > 80%）
  - Format 检查
  - Static 检查
  - Pre-commit 检查（如配置了 `.pre-commit-config.yaml`）
- [ ] 确认 Quality Report 结果为 `ALL CHECKS PASSED`
- [ ] 如有失败项，按报告修复建议修复后重新运行

### Task N+2: 交付
- [ ] 创建 PR：gitflow-cli pr create
  - PR 描述必须包含 "## Issues Fixed" 章节，列出所有要关闭的 Issue
  - 使用 `Closes #N` 或 `Fixes #N` 格式
  - 示例：
    ```markdown
    ## Issues Fixed
    - Closes #11
    - Closes #12
    ```
- [ ] PR 审查：gitflow-cli pr-review
- [ ] 合并 PR：gitflow-cli pr merge

### Task N+3: 收尾
- [ ] 同步 Issue 状态为 done
- [ ] 关闭 Issue
- [ ] **验证所有关联 Issue 已关闭**（重要！）
  ```bash
  # 检查 PR 描述中 "Closes #N" 对应的所有 Issue 是否已关闭
  # 如果 GitHub auto-close 未生效，手动关闭
  gitflow-cli issue view <number>
  # 如果仍然 OPEN
  gitflow-cli issue close <number> --yes
  ```
- [ ] 验证：所有验收标准已满足
- [ ] **清理 worktree 和分支**
  ```bash
  # 切回主仓库目录
  cd <original-repo-path>

  # 拉取最新 main
  git checkout main
  git pull origin main

  # 删除 worktree
  git worktree remove ../gitflow-cli-<issue-number>

  # 删除本地分支
  git branch -d fix/issue-<number>
  ```
```

---

### 🔒 闸门：计划 → 执行

**必须出示的证据**：
1. 完整计划文档已生成
2. 计划包含质量关卡任务（Task N+1）

**校验方法**：
```bash
# 确认计划文档存在
ls -la plan.md
```

**违规处理**：
如果计划文档不完整，输出阻断信息并停止：

```
🔒 闸门未通过：不允许进入执行阶段。必须先完成完整计划文档。

缺少以下证据：
- [ ] 计划文档：未生成
- [ ] 质量关卡任务：未包含在计划中
```

### Phase 2 合规检查

```
Phase 2 合规检查:
  [ ] 步骤 1: writing-plans — 完整计划文档已生成
  [ ] 计划包含质量关卡任务（Task N+1）
  [ ] 计划文档结构完整（任务清单、TDD、审查、提交）
  [ ] 步骤 2: worktree 和分支已创建
  [ ] Task N+2 包含 PR 描述中 "Closes #N" 格式
  [ ] Task N+3 包含 Issue 关闭验证步骤
  [ ] Task N+3 包含 worktree 和分支清理步骤

  缺失项: <list or "无">
  是否允许进入下一阶段: [ ]
```

---

## Phase 3: 执行

**目标**：按计划文档逐任务执行，完成代码实现和质量验证。

**重要：** 此阶段在 worktree 目录中进行（参见步骤 2.2）

### 步骤 3.1：执行计划

调用 Superpowers subagent-driven-development 执行计划：

```
使用 superpowers:subagent-driven-development 技能，执行完整计划文档。
```

执行规则：
- 按计划文档逐任务执行（单个 issue 内串行）
- 每个任务完成后标记 checkbox
- 遇到阻塞时暂停，不跳过任务
- 所有操作在 worktree 目录中进行
- **多个 issue 可并行**：每个 issue 使用独立 worktree，互不干扰

---

## Phase 4: 交付后检查

**目标**：交付完成后，进行后续分析和审查。

### 步骤 4.1：流水线分析

调用 `gitflow-pipeline-analyzer` 分析 CI/CD 流水线健康状况。

```
使用 gitflow-pipeline-analyzer 技能，分析当前仓库的 CI/CD 流水线健康状况。
```

### 步骤 4.2：Issue 分类

调用 `gitflow-issue-triage` 对相关 issues 进行分类和优先级排序。

```
使用 gitflow-issue-triage 技能，对当前仓库的 open issues 进行分类和优先级排序。
```

### 步骤 4.3：代码审查

调用 `gitflow-review` 对整体变更进行代码审查。

```
使用 gitflow-review 技能，对本次交付的整体变更进行代码审查。
```

### Phase 4 产出
- 流水线分析报告
- Issue 分类报告
- 代码审查报告

### Phase 4 合规检查

```
Phase 4 合规检查:
  [ ] 步骤 1: pipeline-analyzer — 流水线分析已运行
  [ ] 步骤 2: issue-triage — Issue 分类已完成
  [ ] 步骤 3: review — 代码审查已完成
  [ ] 所有产出物已记录

  缺失项: <list or "无">
  是否允许进入下一阶段: [N/A - 流程结束]
```

---

## 阶段回退

任何阶段都可以回退到前一个阶段：

- **Phase 4 → Phase 3**：代码审查发现关键问题需返回执行阶段修复
- **Phase 3 → Phase 2**：执行过程中发现需求不明确或实现方案需要调整
- **Phase 2 → Phase 1**：实现过程中发现需求不明确

回退时，在 Issue 上记录回退原因：

```bash
gitflow-cli issue comment <number> --body "## 阶段回退

**从 Phase <X> 回退到 Phase <Y>**

### 回退原因
<原因描述>

### 需要处理的事项
- [ ] <待办事项 1>
- [ ] <待办事项 2>"
```
