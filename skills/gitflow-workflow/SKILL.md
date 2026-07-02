---
name: gitflow-workflow
description: gitflow-cli 全流程开发编排 — 从需求澄清到代码交付的四阶段闸门驱动工作流，指挥 gitflow CLI 和 Superpowers 协同完成完整开发周期
---

# gitflow-workflow 全流程编排

顶层指挥中心，按照四阶段闸门驱动完整的开发生命周期。编排层只做指挥，不直接执行操作 —— 所有平台交互通过 `gitflow` CLI 完成，所有本地开发循环通过 Superpowers 完成。

## 阶段总览

```
Phase 1: 需求澄清          Phase 2: 开发实现          🔒 Phase 3: 质量关卡         Phase 4: 交付
───────────────────────────────────────────────────────────────────────────────────────────────
Superpowers:               Superpowers:               gitflow-quality:           gitflow-pr-create
  brainstorming              writing-plans              build      ✅/❌           gitflow-pr-review
gitflow-issue-create         TDD (per task)             test       ✅/❌           Superpowers:
gitflow-issue-review         subagent-dev               coverage   ✅/❌             finishing-a-dev-branch
                             requesting-review          format     ✅/❌           gitflow-release-helper
                                                       static     ✅/❌
     ↓                             ↓                    ↓                               ↓
产出: Issue URL              产出: 全绿原子任务清单      产出: Quality Report       产出: Merged PR + Release
      需求分析报告
```

## 启动条件

在启动工作流之前，确认以下前置条件：

```bash
# 1. 确认在 git 仓库中
git rev-parse --show-toplevel

# 2. 确认 gitflow CLI 可用
gitflow --version

# 3. 确认认证状态
gitflow auth status
```

如果认证失败，先执行 `gitflow auth login`。

---

## Phase 1: 需求澄清

**目标**：将模糊的想法转化为结构化的 Issue，并完成需求分析。

### 步骤 1.1：触发 brainstorming

如果用户尚未进行需求澄清，触发 Superpowers brainstorming：

- 调用 `superpowers:brainstorming` 探索用户意图、需求边界和设计约束
- 产出：明确的功能描述、验收标准、边界条件

如果用户已有明确需求（如已关联 Issue），跳过此步骤。

### 步骤 1.2：创建 Issue

调用 `gitflow-issue-create` skill 引导创建 Issue：

```
使用 gitflow-issue-create 技能，根据 brainstorming 产出创建 Issue。
```

关键产出：
- Issue URL（如 `https://github.com/owner/repo/issues/123`）
- Issue 编号（如 `#123`）

### 步骤 1.3：需求分析

调用 `gitflow-issue-review`（计划中）对 Issue 进行结构化的需求分析：

```
使用 gitflow-issue-review 技能，对 Issue #N 进行需求分析。
```

分析维度：
- 需求完整性：验收标准是否明确、可测试
- 技术可行性：是否与现有架构兼容
- 范围界定：是否有隐含需求或范围蔓延风险
- 依赖关系：是否阻塞其他 Issue 或被阻塞

产出：需求分析报告（Markdown 格式）

### 步骤 1.4：发布审计日志

将 Phase 1 产出物评论到 Issue：

```bash
gitflow issue comment <number> --body "## Phase 1: 需求澄清完成

### 需求分析报告

<分析报告内容>

### 产出物
- Issue URL: <url>
- 验收标准: <N> 项
- 技术风险: <评估>

✅ 已通过阶段闸门，可进入 Phase 2: 开发实现"
```

---

### 🔒 闸门：需求 → 开发

**必须出示的证据**：
1. Issue URL（可访问的链接）
2. 需求分析报告（评论在 Issue 上）

**校验方法**：
```bash
# 确认 Issue 存在且可访问
gitflow issue view <number>
```

**违规处理**：
如果缺少任一证据，输出阻断信息并停止：

```
🔒 闸门未通过：不允许开始开发。必须先完成需求澄清阶段。

缺少以下证据：
- [ ] Issue URL：未创建或未记录
- [ ] 需求分析报告：未发布到 Issue 评论
```

---

## Phase 2: 开发实现

**目标**：将需求拆解为原子任务，通过 TDD 和子代理驱动完成全部开发。

### 步骤 2.1：制定实现计划

调用 Superpowers writing-plans 将需求拆解为原子任务：

```
使用 superpowers:writing-plans 技能，基于 Issue #N 的需求分析报告，制定实现计划。
```

计划要求：
- 每个原子任务 2-5 分钟可完成
- 任务间依赖关系明确
- 每个任务有对应的测试用例

### 步骤 2.2：子代理驱动开发

调用 Superpowers subagent-driven-development 执行计划：

```
使用 superpowers:subagent-driven-development 技能，执行实现计划。
```

执行规则：
- 每个原子任务遵循 TDD 循环：RED → GREEN → REFACTOR
- 每个任务完成后更新任务状态
- 遇到阻塞时暂停，不跳过任务

### 步骤 2.3：TDD 循环（每个原子任务）

对每个原子任务执行标准 TDD 循环：

```
使用 superpowers:test-driven-development 技能，完成当前原子任务。
```

循环步骤：
1. **RED**：编写失败测试，描述期望行为
2. **GREEN**：编写最小代码使测试通过
3. **REFACTOR**：清理代码，保持测试绿色
4. 验证：`make test` 确认通过

### 步骤 2.4：代码审查（每个原子任务）

每个原子任务完成后，执行代码审查：

```
使用 superpowers:requesting-code-review 技能，审查当前任务的变更。
```

审查维度参考 `gitflow-pr-review` 的 6 维度清单。

### 步骤 2.5：发布任务清单

所有原子任务完成后，将任务清单评论到 Issue：

```bash
gitflow issue comment <number> --body "## Phase 2: 开发实现完成

### 原子任务清单

| # | 任务 | 状态 | 提交 |
|---|------|------|------|
| 1 | <任务描述> | ✅ done | <commit hash> |
| 2 | <任务描述> | ✅ done | <commit hash> |
| ... | ... | ... | ... |

### 变更统计
- 文件变更: <N> 个文件
- 新增行数: +<N>
- 删除行数: -<N>
- 测试用例: <N> 个

✅ 已通过阶段闸门，可进入 Phase 3: 质量关卡"
```

---

### 🔒 闸门：开发 → 质量

**必须出示的证据**：
1. 全部原子任务状态为 `done`
2. 每个任务有对应的提交记录

**校验方法**：
```bash
# 检查工作区状态
git status
git log --oneline -10
```

**违规处理**：
如果有未完成的任务，输出阻断信息并停止：

```
🔒 闸门未通过：不允许运行质量检查。所有原子任务必须标记为 done。

以下原子任务未完成：
- [ ] Task 3: <任务描述> — 状态: in_progress
- [ ] Task 5: <任务描述> — 状态: pending
```

---

## Phase 3: 质量关卡

**目标**：运行 5 项质量检查，确保代码达到交付标准。

### 步骤 3.1：运行质量检查

调用 `gitflow-quality`（计划中）skill 运行完整质量检查：

```
使用 gitflow-quality 技能，对当前分支运行 5 项质量检查。
```

5 项检查：

| # | 检查项 | 命令 | 通过标准 |
|---|--------|------|---------|
| 1 | build | `cargo build --workspace` | 退出码 0 |
| 2 | test | `cargo test --workspace` | 全部通过 |
| 3 | coverage | `cargo tarpaulin --workspace` | > 80% |
| 4 | format | `cargo +nightly fmt -- --check` | 退出码 0 |
| 5 | static | `cargo clippy --workspace --all-targets -- -D warnings` | 退出码 0 |

### 步骤 3.2：处理检查结果

**全部通过**：进入步骤 3.3。

**有失败项**：返回 Phase 2 修复。

```
⚠️ 质量关卡未通过，返回 Phase 2 修复

失败项：
- [❌] static: clippy 发现 3 个警告
- [❌] format: 2 个文件格式不符

修复建议：
1. 运行 `cargo +nightly fmt` 自动修复格式
2. 运行 `cargo clippy --fix` 自动修复 lint 问题
3. 手动修复无法自动修复的问题
4. 重新运行质量检查
```

修复后返回步骤 3.1 重新检查。

### 步骤 3.3：发布质量报告

将质量检查结果评论到 Issue：

```bash
gitflow issue comment <number> --body "## Phase 3: 质量关卡通过

### 质量检查报告

| # | 检查项 | 结果 | 详情 |
|---|--------|------|------|
| 1 | build | ✅ 通过 | 编译成功 |
| 2 | test | ✅ 通过 | <N> 个测试全部通过 |
| 3 | coverage | ✅ 通过 | <N>% (阈值: 80%) |
| 4 | format | ✅ 通过 | 格式规范 |
| 5 | static | ✅ 通过 | 无警告 |

✅ 已通过阶段闸门，可进入 Phase 4: 交付"
```

---

### 🔒 闸门：质量 → 交付

**必须出示的证据**：
1. 5 项检查全部绿色（评论在 Issue 上）

**校验方法**：
检查 Issue 评论中是否包含质量报告。

**违规处理**：
如果质量报告中有任何失败项，输出阻断信息并停止：

```
🔒 闸门未通过：不允许创建 PR。5 项质量检查必须全部绿色。

质量检查未全部通过：
- [✅] build
- [❌] test — 2 个测试失败
- [✅] coverage
- [✅] format
- [✅] static
```

---

## Phase 4: 交付

**目标**：创建 PR、完成代码审查、合并并可选发布。

### 步骤 4.1：创建 PR

调用 `gitflow-pr-create` skill 创建 Pull Request：

```
使用 gitflow-pr-create 技能，基于当前分支创建 PR。
```

PR 要求：
- 标题遵循 conventional commits 格式
- 描述包含 `Closes #N` 自动关联 Issue
- 包含变更说明和验证步骤

### 步骤 4.2：代码审查

调用 `gitflow-pr-review` skill 进行 6 维度审查：

```
使用 gitflow-pr-review 技能，对 PR 进行 6 维度代码审查。
```

如果审查发现问题：
1. 记录问题到 PR 评论
2. 修复问题
3. 推送修复
4. 重新审查

### 步骤 4.3：完成开发分支

调用 Superpowers finishing-a-development-branch 决定分支处理方式：

```
使用 superpowers:finishing-a-development-branch 技能，完成当前开发分支。
```

选项：
- 合并到主分支（merge commit）
- 压缩合并（squash merge）
- 变基合并（rebase merge）

### 步骤 4.4：发布（可选）

如果需要发布新版本，调用 `gitflow-release-helper`（计划中）：

```
使用 gitflow-release-helper 技能，基于合并的变更创建 Release。
```

发布步骤：
1. 生成 Release Notes（基于 conventional commits）
2. 确定版本号（semver）
3. 创建 Git tag
4. 创建 GitHub Release

### 步骤 4.5：关闭 Issue

PR 合并后，确认 Issue 自动关闭（通过 `Closes #N`）：

```bash
# 确认 Issue 状态
gitflow issue view <number>
```

如果未自动关闭，手动关闭：

```bash
gitflow issue close <number>
```

### 步骤 4.6：发布最终审计日志

```bash
gitflow issue comment <number> --body "## Phase 4: 交付完成

### PR 信息
- PR URL: <url>
- PR 编号: #<number>
- 合并策略: <squash|merge|rebase>

### Release（如适用）
- 版本号: v<X.Y.Z>
- Release URL: <url>

### 完成时间
<timestamp>

🎉 开发工作流全部完成！"
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
   - 运行质量检查：build + test + coverage + format + static
   - 全部通过

4. **Phase 4**：
   - 创建 PR：`feat(cli): add shell completion for bash/zsh/fish`
   - 审查通过
   - 合并到 main
   - 创建 Release v1.1.0

### 场景 2：修复一个 Bug

用户说："gitflow issue create 在 GitLab 上报错 401"

**执行流程**：

1. **Phase 1**：
   - brainstorming：复现问题，确认是认证问题还是 API 问题
   - 创建 Issue：`fix(gitlab): issue create returns 401 on valid token`
   - 需求分析：确认是 GitLab API v4 的认证头格式问题

2. **Phase 2**：
   - 拆解任务：添加回归测试 → 修复认证头格式 → 验证修复
   - TDD 循环：先写失败测试，再修复

3. **Phase 3**：
   - 运行质量检查
   - 全部通过

4. **Phase 4**：
   - 创建 PR：`fix(gitlab): use PRIVATE-TOKEN header for auth`
   - 审查通过
   - 合并到 main

### 场景 3：已有关联 Issue 的增量开发

用户说："Issue #42 已经分析好了，开始实现"

**执行流程**：

1. **Phase 1**：
   - 跳过 brainstorming（已有明确需求）
   - 跳过创建 Issue（已存在 #42）
   - 需求分析：读取 Issue #42 的现有描述和评论
   - 如果分析不完整，补充需求分析报告

2. **Phase 2-4**：同场景 1

---

## 阶段回退

任何阶段都可以回退到前一个阶段：

- **Phase 4 → Phase 3**：PR 审查发现质量问题
- **Phase 3 → Phase 2**：质量检查失败
- **Phase 2 → Phase 1**：实现过程中发现需求不明确

回退时，在 Issue 上记录回退原因：

```bash
gitflow issue comment <number> --body "## 阶段回退

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
