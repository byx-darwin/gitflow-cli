---
name: gitflow-workflow
description: gitflow-cli 全流程开发编排 — 从需求澄清到代码交付的四阶段闸门驱动工作流，指挥 gitflow-cli 和 Superpowers 协同完成完整开发周期
---

# gitflow-workflow 全流程编排

顶层指挥中心，按照四阶段闸门驱动完整的开发生命周期。编排层只做指挥，不直接执行操作 —— 所有平台交互通过 `gitflow` CLI 完成，所有本地开发循环通过 Superpowers 完成。

## 两种工作模式

### 🐢 完整模式（默认）
适用于**新功能开发**、**重大重构**、**架构变更**等需要充分探索需求的场景。

```
/gitflow-workflow
我想添加自动补全功能
```

### 🐇 快速模式
适用于 **Bug 修复**、**小优化**、**文档更新**等需求明确的场景。

```
/gitflow-workflow --fast
修复 auth status 无法显示登录状态的问题
```

## 模式对比

| 阶段 | 完整模式 | 快速模式 | 说明 |
|------|---------|---------|------|
| brainstorming | ✅ 必须 | ⚠️ 可选 | bug 描述即需求时可跳过 |
| issue-create | ✅ 必须 | ✅ 必须 | 都需要 Issue 追踪 |
| issue-review | ✅ 必须 | ⚠️ 可选 | bug 原因明确时可跳过 |
| writing-plans | ✅ 必须 | ⚠️ 可选 | 修复方案简单时可跳过 |
| subagent-dev | ✅ 必须 | ✅ 必须 | 所有模式下 Phase 3 均由 subagent 执行 |
| **TDD** | ✅ 计划内嵌 | ✅ 计划内嵌 | TDD 循环已嵌入计划模板，由 subagent 执行 |
| code-review | ✅ 计划内嵌 | ✅ 计划内嵌 | 代码审查已嵌入计划模板，由 subagent 执行 |
| quality | ✅ 计划内嵌 | ✅ 计划内嵌 | 质量关卡已嵌入计划模板，由 subagent 执行 |
| PR | ✅ 必须 | ✅ 必须 | 都需要 PR 流程 |
| pipeline-analyzer | ✅ 必须 | ✅ 必须 | CI/CD 流水线健康分析 |
| issue-triage | ✅ 必须 | ✅ 必须 | Issue 分类和优先级排序 |
| review | ✅ 必须 | ✅ 必须 | 整体变更代码审查 |

## 阶段总览

```
Phase 1: 需求澄清          Phase 2: 计划制定          Phase 3: 执行                Phase 4: 交付后检查
────────────────────────────────────────────────────────────────────────────────────────────────────────
Superpowers:               Superpowers:               Superpowers:               gitflow-pipeline-analyzer
  brainstorming              writing-plans              subagent-driven-development  gitflow-issue-triage
gitflow-issue-create                                                                 gitflow-review
gitflow-issue-review
     ↓                             ↓                    ↓                             ↓
产出: Issue URL              产出: 完整计划文档          产出: 已完成的代码实现        产出: 流水线分析报告
      需求分析报告                   (含质量关卡)                                       Issue 分类报告
                                                                                         代码审查报告
```

## 🚨 强制执行规则

以下规则不可跳过，任何一步不满足就不能进入下一步。

### 完整模式 - 必须调用的 Skills 清单

**Phase 1 必须调用：**
- ✅ `superpowers:brainstorming` - 即使需求明确也要走流程
- ✅ `gitflow-issue-create` - 创建 Issue
- ✅ `gitflow-issue-review` - 需求分析

**Phase 2 必须调用：**
- ✅ `superpowers:writing-plans` - 制定实现计划

**Phase 3 必须调用：**
- ✅ `superpowers:subagent-driven-development` - 按计划执行代码实现和质量验证

**Phase 4 必须调用：**
- ✅ `gitflow-pipeline-analyzer` - CI/CD 流水线健康分析
- ✅ `gitflow-issue-triage` - Issue 分类和优先级排序
- ✅ `gitflow-review` - 整体变更代码审查

### 快速模式 - 必须调用的 Skills 清单

**Phase 1（简化）：**
- ⚠️ `superpowers:brainstorming` - **可选**（bug 描述即需求时可跳过）
- ✅ `gitflow-issue-create` - 创建 Issue（必须）
- ⚠️ `gitflow-issue-review` - **可选**（bug 原因明确时可跳过）

**Phase 2（简化）：**
- ⚠️ `superpowers:writing-plans` - **可选**（修复方案简单时可跳过）

**Phase 3（必须）：**
- ✅ `superpowers:subagent-driven-development` - 按计划执行代码实现和质量验证

**Phase 4（必须）：**
- ✅ `gitflow-pipeline-analyzer` - CI/CD 流水线健康分析
- ✅ `gitflow-issue-triage` - Issue 分类和优先级排序
- ✅ `gitflow-review` - 整体变更代码审查

### 禁止行为

- ❌ **完整模式禁止跳过任何 Superpowers skill** - 即使任务"简单"也必须走完所有步骤
- ❌ **快速模式禁止跳过 TDD 和 Code Review** - 这是质量保证的底线
- ❌ **两种模式禁止跳过 Phase 4** - 交付后检查是完整流程的必要环节
- ❌ **禁止合并步骤** - 每个 skill 必须独立调用
- ❌ **禁止自行实现流程** - 必须使用指定的 skills

### 流程检查点

每个阶段结束时，必须输出检查清单：

```
Phase X 检查点:
- [ ] skill-1 已调用
- [ ] skill-2 已调用
- [ ] skill-3 已调用
- [ ] 产出物已记录

是否允许进入下一阶段: [ ]
```

## 启动条件

在启动工作流之前，确认以下前置条件：

```bash
# 1. 确认在 git 仓库中
git rev-parse --show-toplevel

# 2. 确认 gitflow-cli 可用
gitflow-cli --version

# 3. 确认认证状态
gitflow-cli auth status
```

如果认证失败，先执行 `gitflow-cli auth login`。

---

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

## 使用示例

### 场景 1：从零开始一个新功能

用户说："我想给 gitflow-cli 添加命令行自动补全功能"

**执行流程**：

1. **Phase 1**：
   - brainstorming：探索自动补全的需求边界（哪些 shell？哪些命令？）
   - 创建 Issue：`feat(cli): add shell completion for bash/zsh/fish`
   - 需求分析：确认技术方案（clap 的 completions 功能）

2. **Phase 2**：
   - 拆解任务：实现 completions 命令 → 添加 shell 检测 → 生成补全脚本 → 测试
   - TDD 循环：每个任务先写测试再实现

3. **Phase 3**：
   - 按计划文档逐任务执行
   - 完成代码实现和质量验证

4. **Phase 4**：
   - 流水线分析：检查 CI/CD 配置和健康状况
   - Issue 分类：对相关 issues 进行优先级排序
   - 代码审查：对整体变更进行最终审查

### 场景 2：修复一个 Bug

用户说："gitflow-cli issue create 在 GitLab 上报错 401"

**执行流程**：

1. **Phase 1**：
   - brainstorming：复现问题，确认是认证问题还是 API 问题
   - 创建 Issue：`fix(gitlab): issue create returns 401 on valid token`
   - 需求分析：确认是 GitLab API v4 的认证头格式问题

2. **Phase 2**：
   - 拆解任务：添加回归测试 → 修复认证头格式 → 验证修复
   - TDD 循环：先写失败测试，再修复

3. **Phase 3**：
   - 按计划文档逐任务执行
   - 完成代码实现和质量验证

4. **Phase 4**：
   - 流水线分析：检查 CI/CD 配置和健康状况
   - Issue 分类：对相关 issues 进行优先级排序
   - 代码审查：对整体变更进行最终审查

### 场景 3：已有关联 Issue 的增量开发

用户说："Issue #42 已经分析好了，开始实现"

**执行流程**：

1. **Phase 1**：
   - 跳过 brainstorming（已有明确需求）
   - 跳过创建 Issue（已存在 #42）
   - 需求分析：读取 Issue #42 的现有描述和评论
   - 如果分析不完整，补充需求分析报告

2. **Phase 2-3**：同场景 1

3. **Phase 4**：同场景 1

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

---

## 注意事项

- **编排层不执行操作**：所有平台交互通过 `gitflow` CLI，所有开发循环通过 Superpowers
- **闸门不可跳过**：必须出示完整证据才能进入下一阶段
- **审计日志必须发布**：每个阶段的产出物必须评论到 Issue
- **回退是正常流程**：发现问题时回退是质量保证的一部分
- **可选步骤需确认**：如 Release 步骤，需与用户确认是否执行
- **--body-file 强制**：长内容（>100 字节）必须通过 `--body-file` 传入，不使用 `--body` 直接传
- **合规检查不可跳过**：每个阶段结束必须输出合规清单，逐项打勾后才能进入下一阶段
- **worktree 隔离开发**：Phase 3 执行必须在 worktree 中进行，避免污染主仓库
- **Issue 关闭验证**：PR 合并后必须验证所有 "Closes #N" 对应的 Issue 已关闭，如 GitHub auto-close 未生效则手动关闭
- **worktree 清理**：任务完成后必须清理 worktree 和本地分支，保持仓库整洁
