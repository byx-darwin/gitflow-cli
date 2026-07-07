# gitflow-issue Skill 分析报告

> **分析日期：** 2026-07-07
> **分析目标：** `skills/gitflow-issue/SKILL.md`
> **对应 Issue：** #36
> **分析维度：** 4 个维度（结构规范、职责边界、可测试性、Superpowers 最佳实践）

---

## 一、当前状态总览

| 维度 | 评分 | 说明 |
|------|------|------|
| 维度 1：Skill 结构和文档规范 | ⚠️ 需改进 | Frontmatter 存在但 description 违反规范；文档结构为"命令参考手册"风格，缺少 Superpowers 要求的结构化章节 |
| 维度 2：职责边界清晰度 | ❌ 不合格 | 完全缺失职责边界、禁止行为、红旗列表；7 个子命令各自有不同副作用，无边界声明风险极高 |
| 维度 3：可测试性 | ❌ 不合格 | 完全缺失测试场景、基线、成功标准 |
| 维度 4：与 Superpowers 最佳实践的差距 | ❌ 不合格 | description 描述能力而非触发条件；缺少 TDD 流程、关键词覆盖、跨引用 |

**总体评估：** `gitflow-issue` 是一个"命令参考手册"风格的文档，以表格形式罗列了 7 个子命令及其参数。它不具备 Superpowers skill 所需的可执行性、边界清晰性和可测试性。该 skill 与 `gitflow-pr`、`gitflow-commit` 采用相同模式——三者都是纯命令封装，缺乏 skill 方法论要求的结构化设计。

---

## 二、维度 1 分析 — Skill 结构和文档规范

### 2.1 检查清单

| 检查项 | 状态 | 说明 |
|--------|------|------|
| YAML frontmatter 含 name 字段 | ✅ | `name: gitflow-issue` |
| YAML frontmatter 含 description 字段 | ✅ | 存在 description |
| description 以 "Use when..." 开头 | ❌ | 当前为 `"gitflow-cli 的 Issue 操作命令封装，支持创建、列表、查看、关闭、重新打开、评论和标签管理"` |
| description 只描述触发条件 | ❌ | 描述了完整能力列表（7 个子命令），而非触发时机 |
| 含 Overview 章节 | ❌ | 无 Overview 章节（仅一行简介） |
| 含 When to Use 章节 | ❌ | 无触发条件说明 |
| 含 Core Pattern 章节 | ❌ | 无核心模式提炼 |
| 含 Quick Reference | ⚠️ | 有命令概览表格，但非独立的 Quick Reference 卡片 |
| 含 Implementation 章节 | ❌ | 无实现章节（仅有参数表格） |
| 含 Common Mistakes | ❌ | 无独立章节 |
| Token 效率 < 500 词 | ✅ | 约 280 词（中文），远低于上限 |
| 无叙事性示例反模式 | ✅ | 示例为单行命令，无叙事性描述 |
| 无多语言稀释 | ✅ | 全文中文 |
| 无流程图中嵌入代码 | ✅ | 无流程图 |

### 2.2 具体问题

1. **description 严重违反 Superpowers 规范**：当前 description 列举了 7 个子命令的能力（"创建、列表、查看、关闭、重新打开、评论和标签管理"），这是典型的"能力清单"反模式。Superpowers 要求 description 仅描述触发条件，因为：
   - Claude 用 description 决定**何时加载** skill，而非**了解能做什么**
   - 列举能力会导致 description 无法被 Claude 的 skill 加载决策机制精确匹配
   - 增加 token 消耗（description 在每次 skill 匹配时都会被评估）

2. **文档结构是"命令参考手册"而非"可执行指令"**：
   - 整个文档由参数表格 + 4 个一行示例组成，缺少执行流程、前置条件、错误处理
   - 未说明何时应使用哪个子命令（如"close 之前是否需要先 comment？"）
   - 未说明子命令之间的依赖关系（如"label 操作是否需要 issue 已存在？"）

3. **与 gitflow-pr / gitflow-commit 同构**：三个 skill 采用完全相同的"命令参考手册"模式，说明这是系统性的模式问题，而非单个 skill 的偶然缺陷。

4. **缺少结构化章节**：按照 writing-skills 方法论，应包含：
   - `## Overview` — 一段话说明 skill 的核心价值
   - `## When to Use` — 触发场景列表
   - `## Core Pattern` — 核心执行模式
   - `## Quick Reference` — 一页纸命令速查
   - `## Implementation` — 详细实现步骤
   - `## Common Mistakes` — 常见错误

### 2.3 评分：⚠️ 需改进

---

## 三、维度 2 分析 — 职责边界清晰度

### 3.1 检查清单

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 有明确的职责边界声明 | ❌ | 完全缺失 |
| 有禁止行为清单（🚫 不得...） | ❌ | 缺失 |
| 有职责范围说明（✅ 负责... / ❌ 不负责...） | ❌ | 缺失 |
| 有"合理化借口"反制表格 | ❌ | 缺失 |
| 有红旗列表（Red Flags） | ❌ | 缺失 |

### 3.2 具体问题

1. **完全无职责边界**：该 skill 封装了 7 个子命令，每个都有不同的副作用：
   - `create` — 创建新 Issue（不可逆的仓库写入）
   - `close` / `reopen` — 改变 Issue 状态（影响项目看板）
   - `comment` — 添加评论（通知订阅者）
   - `label` — 修改标签（影响分类和筛选）

   没有边界声明可能导致：
   - Claude 在用户仅要求"查看"时误执行"关闭"操作
   - Claude 批量关闭 issues（如用户说"清理旧 issues"）
   - Claude 在错误的仓库中执行操作（缺少 repo 上下文验证）

2. **缺少前置条件检查**：未声明执行前应验证：
   - gitflow-cli 是否已安装
   - 是否已认证（auth status）
   - 是否在 git 仓库中
   - 目标 issue 是否存在

3. **缺少禁止行为清单**：应明确禁止：
   - 不得在未确认 issue 编号的情况下执行 close/comment/label
   - 不得批量操作（close 多个 issues、label 多个 issues）除非用户明确要求
   - 不得修改不属于自己的 issues（除非用户是仓库维护者）
   - 不得删除 comments（本 skill 不支持，但应明确声明）

4. **缺少红旗信号**：应标识以下场景为红旗：
   - 用户要求"关闭所有 issues"
   - 用户要求"删除 issue"
   - 用户要求操作其他仓库的 issue 但未指定 --repo
   - 用户要求对 issue 执行非 CRUD 操作（如修改标题、转移仓库）

5. **与 gitflow-issue-create / gitflow-issue-review / gitflow-issue-triage 的边界模糊**：
   - `gitflow-issue create` vs `gitflow-issue-create`：前者是命令封装，后者是交互式工作流。用户何时用哪个？
   - `gitflow-issue label` vs `gitflow-label-milestone`：标签管理的边界在哪里？
   - `gitflow-issue view` vs `gitflow-issue-review`：查看详情和需求分析的区别？

### 3.3 评分：❌ 不合格

**对比参考（gitflow-autoreport-bug）：** 该 skill 是项目中唯一具备完整职责边界声明的文档。gitflow-issue 应参照此模式，尤其需要声明 7 个子命令各自的边界和前置条件。

---

## 四、维度 3 分析 — 可测试性

### 4.1 检查清单

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 定义了测试场景 | ❌ | 无测试场景定义 |
| 有基线测试（无 skill 时的行为） | ❌ | 缺失 |
| 有压力测试场景 | ❌ | 缺失 |
| 有成功标准 | ❌ | 缺失 |
| 可使用 writing-skills 方法论测试 | ❌ | 无测试钩子 |

### 4.2 具体问题

1. **零测试覆盖**：作为 skill，应定义如何验证 Claude 正确执行了 issue 操作：
   - 如何判断 "Claude 正确创建了 issue 并返回了 URL"？
   - 如何判断 "Claude 在 close 前确认了 issue 编号"？
   - 如何判断 "Claude 没有误操作其他 issue"？

2. **缺少基线对比**：应定义不使用 skill 时 Claude 的典型行为作为对照基线。例如：
   - 基线：Claude 直接运行 `gitflow-cli issue create` 但不确认参数，可能遗漏 --label 或 --assignee
   - 期望：Claude 按 skill 指引收集所有参数后执行

3. **无压力测试场景**：应覆盖：
   - issue 编号不存在（view/close/comment 不存在的 issue）
   - 网络超时或 API 限流
   - 未认证状态下的操作
   - 跨仓库操作（--repo 参数）
   - 特殊字符标题（Unicode、引号、换行）
   - 空 body 的 comment

4. **无成功标准**：应定义：
   - create 成功 → 返回有效 Issue URL
   - list 成功 → 返回 JSON 数组或表格
   - view 成功 → 返回 issue 详情
   - close/reopen 成功 → 状态变更确认
   - comment 成功 → 评论 URL 或 ID
   - label 成功 → 标签列表更新

### 4.3 评分：❌ 不合格

---

## 五、维度 4 分析 — 与 Superpowers 最佳实践的差距

### 5.1 检查清单

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 遵循 TDD for skills（RED-GREEN-REFACTOR） | ❌ | 无 TDD 流程 |
| description 只描述触发条件，不描述流程 | ❌ | description 列举 7 种能力 |
| 关键词覆盖（错误信息、症状、同义词、工具） | ❌ | 无关键词覆盖 |
| 跨引用其他 skills | ❌ | 无 See Also / 相关 Skills |
| 必要时使用 flowchart | ⚠️ | 子命令选择逻辑适合用 flowchart 但未提供 |

### 5.2 具体问题

1. **description 包含能力清单（严重违规）**：
   - ❌ 当前：`"gitflow-cli 的 Issue 操作命令封装，支持创建、列表、查看、关闭、重新打开、评论和标签管理"`
   - ✅ 应为：`"Use when the user needs to manage GitHub/GitLab/GitCode issues through gitflow-cli — including creating, listing, viewing, closing, reopening, commenting on issues, or managing issue labels."`

2. **未遵循 writing-skills 方法论**：
   - 未进行 "baseline test"（先不用 skill 让 Claude 执行 issue 操作，记录差距）
   - 未基于差距优化 skill 内容
   - 未迭代验证

3. **缺少关键词覆盖**：应覆盖用户可能的表达方式：
   - "创建 issue" / "新建 issue" / "open an issue" / "file a bug"
   - "查看 issue" / "issue 详情" / "show issue" / "issue view"
   - "关闭 issue" / "close issue" / "resolve issue"
   - "评论 issue" / "add comment" / "issue comment"
   - "标签" / "label" / "加标签" / "移除标签"
   - "issue 列表" / "list issues" / "查看待办"
   - `gitflow-cli issue` / `gitflow issue`

4. **缺少跨引用**：应引用相关 skills：
   - `gitflow-issue-create`（交互式创建 Issue 工作流）
   - `gitflow-issue-review`（Issue 需求分析）
   - `gitflow-issue-triage`（Issue 分类分流）
   - `gitflow-label-milestone`（标签和里程碑管理）
   - `gitflow-autoreport-bug`（自动错误报告）
   - `gitflow-workflow`（完整开发工作流）

5. **缺少子命令选择 flowchart**：7 个子命令的选择逻辑适合用 flowchart 表达。当前文档假设用户已知要使用哪个子命令，但实际场景中用户可能只说"帮我处理一下 issue 42"，Claude 需要根据上下文判断应使用 view/close/comment 中的哪一个。

6. **与 gitflow-pr / gitflow-commit 同构问题**：三个 skill 共享相同的"命令参考手册"模式，说明这是系统性的设计问题。理想情况下，这类"命令封装型 skill"应有统一的模板，包含：
   - 触发条件（而非能力清单）
   - 子命令选择决策逻辑
   - 每个子命令的前置条件和边界
   - 跨引用到更高级的工作流 skill

### 5.3 评分：❌ 不合格

---

## 六、改进建议

### P0（必须修复 — 阻断性问题）

| # | 改进项 | 维度 | 说明 |
|---|--------|------|------|
| P0-1 | 重写 description 为触发条件 | D1, D4 | description 决定 Claude 何时加载 skill，必须改为 "Use when..." 格式，移除能力清单 |
| P0-2 | 添加职责边界声明章节 | D2 | 7 个子命令各有副作用，必须声明 🚫 禁止行为（不得批量操作、不得未确认执行写入）和 ✅ 职责范围 |
| P0-3 | 添加前置条件检查 | D1, D2 | 执行前验证 gitflow-cli 是否可用、是否已认证、是否在 git 仓库中 |
| P0-4 | 添加关键词覆盖 | D4 | 覆盖常见表达（"创建 issue"、"关闭 issue"、"issue 列表"、"加标签"）和工具名（`gitflow-cli issue`） |
| P0-5 | 添加跨引用 | D4 | 引用 gitflow-issue-create、gitflow-issue-review、gitflow-issue-triage、gitflow-label-milestone |

### P1（建议修复 — 提升质量）

| # | 改进项 | 维度 | 说明 |
|---|--------|------|------|
| P1-1 | 重构为结构化模板 | D1 | 添加 Overview / When to Use / Core Pattern / Quick Reference / Implementation / Common Mistakes 章节 |
| P1-2 | 添加错误处理章节 | D1 | 覆盖 issue 不存在、API 限流、未认证、网络超时等异常场景 |
| P1-3 | 添加红旗列表 | D2 | 标识敏感场景（批量关闭 issues、操作他人 issue、跨仓库操作未指定 --repo） |
| P1-4 | 添加子命令选择 flowchart | D4 | 用 flowchart 表达"用户说处理 issue N → 应使用哪个子命令"的决策逻辑 |
| P1-5 | 添加"合理化借口"反制表格 | D2 | 预防 Claude 越界操作（如"顺便帮用户关闭其他 issues"、"自动添加标签"） |
| P1-6 | 添加 Quick Reference 卡片 | D1 | 一页纸命令速查，包含最常用的 3-5 个命令模式 |

### P2（可选改进 — 锦上添花）

| # | 改进项 | 维度 | 说明 |
|---|--------|------|------|
| P2-1 | 添加基线测试场景 | D3 | 定义不用 skill 时 Claude 的基线行为（直接执行命令但不确认参数） |
| P2-2 | 定义成功标准 | D3 | 每个子命令的成功输出格式和验证方式 |
| P2-3 | 添加压力测试场景 | D3 | issue 不存在、特殊字符、API 限流、未认证 |
| P2-4 | 统一命令封装型 skill 模板 | D1, D4 | 与 gitflow-pr、gitflow-commit 共享统一结构模板 |
| P2-5 | 添加输出格式说明 | D1 | 说明 --output json 和 --output text 的差异和使用场景 |

---

## 七、验收标准

- [ ] description 以 "Use when..." 开头，仅描述触发条件，不包含能力清单
- [ ] 含职责边界声明章节（含 🚫 禁止行为和 ✅ 职责范围）
- [ ] 含前置条件检查（gitflow-cli 可用、已认证、在 git 仓库中）
- [ ] 含关键词覆盖（"创建 issue"、"关闭 issue"、"issue 列表"、"加标签"等）
- [ ] 含子命令选择 flowchart（view/close/comment/label/reopen 决策逻辑）
- [ ] 文档结构包含 Overview / When to Use / Quick Reference / Implementation
- [ ] 含错误处理章节（issue 不存在、API 限流、未认证）
- [ ] 含跨引用（至少引用 3 个相关 skill）
- [ ] 含红旗列表（批量操作、操作他人 issue、跨仓库未指定 --repo）
- [ ] 含"合理化借口"反制表格

---

## 八、与同类 Skill 对比

| 对比项 | gitflow-issue | gitflow-autoreport-bug | gitflow-issue-create | 差距 |
|--------|--------------|----------------------|---------------------|------|
| 职责边界 | ❌ 缺失 | ✅ 完整 | ❌ 缺失 | 差距大 |
| 测试场景 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 | — |
| 触发条件 | ❌ 能力清单 | ⚠️ 描述流程 | ⚠️ 描述流程 | 均需改进 |
| 跨引用 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 | — |
| 错误处理 | ❌ 缺失 | ✅ 有异常处理章节 | ❌ 缺失 | 差距大 |
| 结构化章节 | ❌ 缺失 | ⚠️ 部分 | ⚠️ 部分 | — |

---

## 九、总结

`gitflow-issue` 当前的定位是"命令参考手册"，它以表格形式罗列了 7 个子命令及其参数，但不具备 Superpowers skill 所需的**可执行性**、**边界清晰性**和**可测试性**。

核心差距：
1. **description 包含能力清单** → Claude 无法用此判断何时加载 skill
2. **缺乏职责边界** → Claude 可能误操作 issues（批量关闭、未确认写入）
3. **缺乏错误处理** → issue 不存在或 API 失败时 Claude 无指导
4. **缺乏可测试性** → 无法验证 Claude 是否正确执行了 issue 操作
5. **与 gitflow-pr / gitflow-commit 同构** → 这是系统性的模式问题，需要统一模板

重构方向：将其从"命令参考手册"转型为"可执行指令 + 边界声明 + 子命令决策流程图 + 验证标准"的完整 skill。建议与 `gitflow-pr`、`gitflow-commit` 统一重构，采用相同的"命令封装型 skill"模板。
