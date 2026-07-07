# gitflow-commit Skill 分析报告

> **分析日期：** 2026-07-07
> **分析目标：** `skills/gitflow-commit/SKILL.md`
> **对应 Issue：** #16
> **分析维度：** 4 个维度（结构规范、职责边界、可测试性、Superpowers 最佳实践）

---

## 一、当前状态总览

| 维度 | 评分 | 说明 |
|------|------|------|
| 维度 1：Skill 结构和文档规范 | ⚠️ 需改进 | Frontmatter 基本合规，但 description 和内容结构不符合 Superpowers 规范 |
| 维度 2：职责边界清晰度 | ❌ 不合格 | 完全缺失职责边界声明 |
| 维度 3：可测试性 | ❌ 不合格 | 完全缺失测试场景和成功标准 |
| 维度 4：与 Superpowers 最佳实践的差距 | ❌ 合格 | 未遵循 writing-skills 方法论 |

**总体评估：** gitflow-commit 当前是一个纯命令参考手册，而非符合 Superpowers 规范的 skill。它描述了"命令能做什么"，但没有描述"何时触发""如何执行""边界在哪""如何验证"。与 gitflow-auth 存在完全相同的结构性问题。

---

## 二、维度 1 分析 — Skill 结构和文档规范

### 2.1 检查清单

| 检查项 | 状态 | 说明 |
|--------|------|------|
| YAML frontmatter 含 name 字段 | ✅ | `name: gitflow-commit` |
| YAML frontmatter 含 description 字段 | ✅ | 存在 description |
| description 以 "Use when..." 开头 | ❌ | 当前为 "gitflow-cli 的 Commit 操作命令封装，支持查看、差异比较、补丁导出和行内评论"——这是功能描述而非触发条件 |
| description 只描述触发条件 | ❌ | 描述了功能而非触发时机 |
| 含 Overview 章节 | ❌ | 无 Overview 章节（仅有"命令概览"表格） |
| 含 When to Use 章节 | ❌ | 无触发条件说明 |
| 含 Core Pattern 章节 | ❌ | 无核心模式 |
| 含 Quick Reference | ⚠️ | 有命令概览表格，但缺少参数与返回值的快速对照 |
| 含 Implementation 章节 | ❌ | 无实现步骤 |
| 含 Common Mistakes | ❌ | 无常见错误说明 |
| Token 效率 < 500 词 | ✅ | 约 260 词，简洁 |
| 无叙事性示例反模式 | ⚠️ | 示例仅展示 happy path，无失败场景 |
| 无多语言稀释 | ⚠️ | 使用中文注释但未提供英文对照 |
| 无流程图中嵌入代码 | ✅ | 无流程图 |

### 2.2 具体问题

1. **Frontmatter description 违反 Superpowers 规范**：Superpowers 要求 description 仅描述触发条件（"When to load this skill"），以便 Claude 决定是否加载。当前 description 是功能概述，会导致：
   - Claude 无法准确判断何时应加载此 skill
   - 增加误触发或漏触发的概率
   - 在 code review 场景中可能不会自动加载（因 description 未提及 review/commit 关键词）

2. **缺乏结构化章节**：当前文档是"命令参考手册"风格，不是"可执行指令"风格。缺少：
   - 前置条件（prerequisites）
   - 输入验证
   - 输出格式
   - 错误处理
   - 执行步骤

3. **使用示例仅覆盖 happy path**：示例只展示成功场景，缺少：
   - 无效 SHA 场景
   - 不存在的 Commit 场景
   - 评论时文件路径不匹配场景
   - 跨平台差异场景

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

1. **完全无职责边界**：commit skill 虽然不涉及敏感的凭据操作，但职责边界仍然重要：
   - Claude 可能在 review 时直接修改代码（超出 commit 评论的范围）
   - 可能在不应创建评论时频繁创建评论
   - 可能在 commit comment 后尝试自动修复问题代码

2. **缺少禁止行为清单**：commit 相关的禁止行为包括：
   - 不得使用 commit 评论功能直接修改代码
   - 不得在 CI 中频繁触发评论导致噪音
   - 不得将 patch 内容用于非预期用途

3. **缺少红旗信号**：应有的红旗包括：
   - 用户要求"修改此 commit 的代码"（commit 不可修改）
   - 尝试对 merge commit 添加行内评论
   - 尝试泄露 patch 内容到外部

### 3.3 评分：❌ 不合格

**对比参考（gitflow-autoreport-bug）：** 该 skill 是项目中唯一具备完整职责边界声明的文档。gitflow-commit 应参照此模式，明确定义其仅负责查看、获取 diff/patch 和添加评论，不负责代码修改或自动修复。

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

1. **零测试覆盖**：作为 skill，应定义如何验证 Claude 正确执行了命令：
   - 如何判断 "Claude 正确识别了触发条件"？
   - 如何判断 "Claude 正确处理了无效 SHA 错误"？
   - 如何判断 "Claude 正确选择了 view/diff/patch/comment 子命令"？

2. **缺少基线对比**：应定义不使用 skill 时 Claude 的典型行为作为对照基线。例如：
   - 不使用 skill 时，Claude 可能使用 `git show` 而非 `gitflow-cli commit view`
   - 不使用 skill 时，Claude 可能不知道可以添加行内评论

3. **无压力测试场景**：应覆盖：
   - 超大 Commit 的 diff 处理
   - 跨多个文件的 patch 应用
   - 并发评论时的冲突处理

### 4.3 评分：❌ 不合格

---

## 五、维度 4 分析 — 与 Superpowers 最佳实践的差距

### 5.1 检查清单

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 遵循 TDD for skills（RED-GREEN-REFACTOR） | ❌ | 无 TDD 流程 |
| description 只描述触发条件，不描述流程 | ❌ | description 描述了功能而非触发条件 |
| 关键词覆盖（错误信息、症状、同义词、工具） | ❌ | 无关键词覆盖 |
| 跨引用其他 skills | ❌ | 无 See Also / 相关 Skills |
| 必要时使用 flowchart | N/A | 4 个子命令的线性关系，无需流程图 |

### 5.2 具体问题

1. **未遵循 writing-skills 方法论**：
   - 未进行 "baseline test"（先不用 skill 让 Claude 执行，记录差距）
   - 未基于差距优化 skill 内容
   - 未迭代验证

2. **description 应是触发条件而非功能描述**：
   - ❌ 当前：`gitflow-cli 的 Commit 操作命令封装，支持查看、差异比较、补丁导出和行内评论`
   - ✅ 应为：`Use when the user needs to view commit details, inspect diffs/patches, or add inline comments to a specific commit line`

3. **缺少关键词覆盖**：应覆盖用户可能的表达方式：
   - "查看 commit" / "show commit" / "commit 详情" / "what changed in commit"
   - "获取 diff" / "commit diff" / "what's the diff"
   - "添加评论" / "commit comment" / "line comment" / "inline review"
   - "导出 patch" / "commit patch" / "apply patch"

4. **缺少跨引用**：应引用相关 skills：
   - `gitflow-pr`（PR 中包含的 commit 检查）
   - `gitflow-pr-review`（在 PR review 中查看 commit）
   - `gitflow-review`（代码评审流程）
   - `gitflow-precommit`（commit 前的质量检查）

### 5.3 评分：❌ 不合格

---

## 六、改进建议

### P0（必须修复 — 阻断性问题）

| # | 改进项 | 维度 | 说明 |
|---|--------|------|------|
| P0-1 | 重写 description 为触发条件 | D1, D4 | description 决定 Claude 何时加载 skill，必须改为 "Use when..." 格式 |
| P0-2 | 添加职责边界声明章节 | D2 | 明确 commit skill 仅负责查看、diff/patch 导出和评论，不负责代码修改 |
| P0-3 | 添加关键词覆盖 | D4 | 覆盖常见表达（"查看 commit"、"diff"、"行内评论"、"patch"）和工具名 |
| P0-4 | 添加跨引用 | D4 | 引用 gitflow-pr、gitflow-pr-review、gitflow-review 等相关 skills |

### P1（建议修复 — 提升质量）

| # | 改进项 | 维度 | 说明 |
|---|--------|------|------|
| P1-1 | 重构为结构化模板 | D1 | 添加 Overview / When to Use / Core Pattern / Quick Reference / Implementation / Common Mistakes 章节 |
| P1-2 | 添加错误处理章节 | D1 | 覆盖无效 SHA、Commit 不存在、文件路径不匹配等异常场景 |
| P1-3 | 添加前置条件检查 | D1 | 执行前验证 `gitflow-cli` 是否可用、是否在 git 仓库中 |
| P1-4 | 添加红旗列表 | D2 | 标识需要用户确认的场景（对 merge commit 评论、大文件 diff 等） |

### P2（可选改进 — 锦上添花）

| # | 改进项 | 维度 | 说明 |
|---|--------|------|------|
| P2-1 | 添加基线测试场景 | D3 | 定义不用 skill 时 Claude 的基线行为 |
| P2-2 | 定义成功标准 | D3 | 每个子命令的预期输出和退出码 |
| P2-3 | 添加压力测试场景 | D3 | 超大 Commit、多文件 patch、并发评论等 |
| P2-4 | 提供英文版 description | D1 | 当前仅中文，可考虑 bilingual |

---

## 七、验收标准

- [ ] description 以 "Use when..." 开头，仅描述触发条件
- [ ] 含职责边界声明章节（含 🚫 禁止行为）
- [ ] 含关键词覆盖（常用表达、同义词、工具名）
- [ ] 含跨引用（至少引用 1 个相关 skill）
- [ ] 文档结构包含 Overview / When to Use / Quick Reference
- [ ] 含错误处理章节
- [ ] 新增/修改内容通过一致性检查（skill 不适用 Rust gate）

---

## 八、与同类 Skill 对比

| 对比项 | gitflow-commit | gitflow-autoreport-bug | 差距 |
|--------|---------------|----------------------|------|
| 职责边界 | ❌ 缺失 | ✅ 完整 | 差距大 |
| 测试场景 | ❌ 缺失 | ❌ 缺失 | — |
| 触发条件 | ❌ 功能描述 | ⚠️ 描述流程 | autoreport-bug 也需改进 |
| 跨引用 | ❌ 缺失 | ❌ 缺失 | — |
| 错误处理 | ❌ 缺失 | ✅ 有异常处理章节 | 差距大 |

---

## 九、总结

gitflow-commit 当前的定位是"命令参考手册"，它描述了命令的输入输出，但不具备 Superpowers skill 所需的**可执行性**、**边界清晰性**和**可测试性**。

核心差距：
1. **缺乏触发条件** → Claude 何时应加载此 skill？
2. **缺乏执行步骤** → Claude 应按什么顺序处理 commit？
3. **缺乏错误处理** → 遇到无效 SHA 时 Claude 应如何反应？
4. **缺乏边界** → Claude 不应修改代码（仅评论）？

重构方向：将其从"参考手册"转型为"可执行指令 + 边界声明 + 验证标准"的完整 skill，参考 gitflow-autoreport-bug 的结构化模板。
