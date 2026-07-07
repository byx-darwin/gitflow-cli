# gitflow-pr-inline-review Skill 分析报告

> **分析日期：** 2026-07-07
> **分析目标：** `skills/gitflow-pr-inline-review/SKILL.md`
> **对应 Issue：** #32
> **分析维度：** 4 个维度（结构规范、职责边界、可测试性、Superpowers 最佳实践）

---

## 一、当前状态总览

| 维度 | 评分 | 说明 |
|------|------|------|
| 维度 1：Skill 结构和文档规范 | ⚠️ 需改进 | description 为功能/流程描述而非触发条件；缺少 Overview / When to Use / Core Pattern / Quick Reference 等结构化章节；操作步骤偏"操作手册"风格（step-by-step bash 块 + 表格 + checklist）；token 数 479 词（勉强通过 500 上限但无余裕）；无多语言稀释，无流程图嵌入代码 |
| 维度 2：职责边界清晰度 | ❌ 不合格 | 完全缺失职责边界声明、禁止行为清单、职责范围说明、红旗列表和"合理化借口"反制表格；且 skill 涉及通过 `gitflow-cli commit comment` 向远端 PR 发布会审评论（写操作副作用） |
| 维度 3：可测试性 | ❌ 不合格 | 完全缺失测试场景、基线测试、压力测试和成功标准 |
| 维度 4：与 Superpowers 最佳实践的差距 | ⚠️ 需改进 | 工作流步骤可执行（5 步）、4 维度审查清单实用、命令精确；但 description 不合规、无关键词覆盖、无跨引用其他 skills、未遵循 writing-skills 方法论 |

**总体评估：** gitflow-pr-inline-review 是一个"操作手册 + 命令模板库"混合体——它有清晰的 5 步工作流、结构化的 4 维度审查清单、精确的命令格式。但与 Superpowers skill 形态差距在于：**description 不合规导致触发边界模糊**、**职责边界完全缺失**（涉及发布会审评论的写操作副作用）、**无可测试性设计**。

---

## 二、维度 1 分析 — Skill 结构和文档规范

### 2.1 检查清单

| 检查项 | 状态 | 说明 |
|--------|------|------|
| YAML frontmatter 含 name 字段 | ✅ | `name: gitflow-pr-inline-review` |
| YAML frontmatter 含 description 字段 | ✅ | 存在 description |
| description 以 "Use when..." 开头 | ❌ | "PR 行内评论工作流 — 获取 PR diff 并逐文件分析..." |
| description 只描述触发条件 | ❌ | 混合功能描述（4 维度审查）、流程描述（"获取 diff → 分析 → 生成评论"）、效果承诺 |
| 含 Overview 章节 | ⚠️ | 有 H1 标题 + 引导句，但缺少结构化 Overview（核心能力 1-2 句概括） |
| 含 When to Use 章节 | ❌ | 无触发条件说明 |
| 含 Core Pattern 章节 | ❌ | 无核心模式/算法骨架（4 维度审查清单是方法论，但未被命名为 Core Pattern） |
| 含 Quick Reference | ❌ | 5 步工作流缺少"触发→执行→输出"速查对照表 |
| 含 Implementation 章节 | ✅ | 步骤 1-5 工作流明确可执行（高质量） |
| 含 Common Mistakes | ⚠️ | 有"注意事项"段落（7 条），接近 Common Mistakes |
| Token 效率 < 500 词 | ⚠️ | 479 词（`wc -w`），通过但余裕仅 21 词 |
| 无叙事性示例反模式 | ⚠️ | 使用示例给出 3 个完整 bash 块（含问题分类、路径、行号、代码建议），接近"复制黏贴命令模板" |
| 无多语言稀释 | ✅ | 全中文，单一语言专注 |
| 无流程图中嵌入代码 | ✅ | 无 Mermaid/Graphviz 流程图嵌入 |

### 2.2 具体问题

1. **description 违反 Superpowers 规范**：
   - 当前：`PR 行内评论工作流 — 获取 PR diff 并逐文件分析，针对具体代码行生成行内评论，覆盖逻辑错误、安全隐患、命名规范、边界条件四个维度`
   - 问题：这是功能描述 + 方法论承诺，不是触发条件
   - 应为：`Use when reviewing a Pull Request with inline comments on specific lines, conducting PR feedback focused on logic errors/security/naming/boundaries, or filing line-level review feedback on GitHub/GitLab/GitCode PRs.`
   - 后果：用户说"帮我看下 PR #101 有没有安全问题"、"审查 PR 的代码质量"、"行内审查"都是潜在触发，但当前 description 无法帮助 Claude 判断

2. **Token 余裕极窄（479 vs 500）**：
   - 主因：使用示例给出 3 个完整 bash 块（含问题分类、文件路径、行号、建议修改代码），每个 bash 块约 10-15 行
   - 4 维度审查清单详细展开（约 20 行 bullet 条目）
   - 步骤 3 的格式模板和步骤 5 的汇总模板共占约 30 行
   - 问题：技能定位是"引导 Claude 执行 PR 行内审查"，不需要那么多"展示型"模板
   - 建议：审查清单缩略为 1 个维度标签表 + 1 行示例；使用示例保留 1 个最简示例 + 3 行变体速查

3. **缺少结构化快速导航**：
   - Superpowers skill 推荐 Overview + When to Use + Core Pattern + Quick Reference + Implementation + Common Mistakes
   - 当前用户期待的是"告诉我 PR 行内审查怎么发起"，但文档结构是 5 步骤线性教程
   - 建议：添加 Quick Reference 速查表（`pr diff` → 分析 → `commit comment` → 汇总）

4. **步骤 2 内容过度展开 — 4 维度清单偏操作手册**：
   - 步骤 2 将 4 维度（logic/security/naming/boundary）各展开为 4-5 个 bullet 子项
   - 这些是审查方法论参考，skill 执行时 Claude 会自行判断，无需逐条罗列
   - 建议：维度清单浓缩为 1 个 4 行标签表 + 维度名简要；详细检查点移至 `docs/references/pr-review-checklist.md`

5. **缺少 When NOT to use 指导**：
   - 何时应使用 `gitflow-pr-review`（总体审查）而非 `gitflow-pr-inline-review`（行内审查）？边界未定义
   - 建议：明确"总体架构反馈 → pr-review；具体行问题 → pr-inline-review"

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

1. **无职责边界 → 涉及写操作副作用风险**：
   - 本 skill 调用 `gitflow-cli commit comment` 向远端 PR 发布会审评论——这是不可逆的写操作
   - GitHub/GitLab/GitCode 上的评论一旦发布，编辑/删除可能受限或有可见痕迹
   - 没有边界声明时，Claude 可能在以下场景过度执行：
     - 用户仅问"这个 PR 有没有安全问题"时，Claude 自动分析并批量发布 10+ 条行内评论
     - 不确定是否是真正 bug 时，仍以 `[security]` 标签发布评论（可能误报影响团队信任）
     - 用户说"先告诉我要评论什么"，Claude 直接发布而非先展示草稿
     - 在需要 Review Approve/Request Changes 决策时，Claude 仅发表内评论但未决策

2. **缺少禁止行为清单 — 应明确**：
   - 🚫 不得在未获用户确认前发布行内评论到远端 PR
   - 🚫 不得将未经确认的疑似问题以 `[security]` 标签发布（误报损害信任）
   - 🚫 不得代替用户做出 Review 决策（Approve / Request Changes / Comment）
   - 🚫 不得读取或评论非 PR 变更范围内的文件
   - 🚫 不得将评审草稿同时发布到多个 PR（编号混淆）
   - 🚫 不得在 CI 环境（如 gitflow-workflow 执行中）未经用户审阅批量发布会审评论

3. **缺少红旗信号 — 应标识**：
   - 用户说"直接帮评论一下"但未确认发布（需先展示草稿）
   - 行号来自 diff 上下文但行号不明确（避免评论到错误的行）
   - PR 关闭状态下尝试发布会审评论
   - PR 来自不受支持的平台（非 GitHub/GitLab/GitCode）
   - 用户使用评论发布后自行决定 PR 状态

4. **缺少职责范围说明**：
   - ✅ 负责：获取 PR diff、按 4 维度分析变更、生成行内评论草稿、发布评论到指定行、输出审查汇总
   - ❌ 不负责：做出 Review 决策（Approve / Request Changes 由用户决定）、修复 PR 代码、反复审查已关闭的 PR、处理 Review 回复
   - ❌ 不负责：在 PR 上创建或关闭 PR（属于 gitflow-pr skill）

5. **"合理化借口"反制**：
   - "just leave a quick comment" → "留个评论很快" → 发布会审评论前需用户确认
   - "the author needs to see this" → "作者应该知道" → 用户决定是否发布
   - "I'm helping the review process" → "我在协助审查" → Review 决策权在用户
   - "it's constructive feedback" → "这是建设性反馈" → 确认后再发布
   - "the line numbers are probably right" → "行号应该没错" → 行号必须基于 diff 精确验证

### 3.3 评分：❌ 不合格

**对比参考（gitflow-autoreport-bug）：** autoreport-bug 本身有完整的"职责边界声明"章节，明确"只报告不修复"。gitflow-pr-inline-review 涉及发布会审评论（写操作），但边界风险更高——误发的评论可能被 PR 作者和团队看到，一旦发布难以无痕撤回。这种"写入公开可见内容"的操作比普通 Issue 创建更需要明确边界。

---

## 四、维度 3 分析 — 可测试性

### 4.1 检查清单

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 定义了测试场景 | ❌ | 无测试场景定义 |
| 有基线测试（无 skill 时的行为） | ❌ | 缺失 |
| 有压力测试场景 | ❌ | 缺失 |
| 有成功标准 | ❌ | 仅有格式化的"审查汇总"模板，未定义为可验证标准 |
| 可使用 writing-skills 方法论测试 | ❌ | 无测试钩子 |

### 4.2 具体问题

1. **零测试覆盖 — 无法验证 Claude 执行质量**：
   - 如何判断 "Claude 在用户未确认前未发布会审评论"？
   - 如何判断 "Claude 的行号计算基于 diff 而非猜测"？
   - 如何判断 "Claude 在 PR 来自不支持的平台时正确拒绝"？
   - 如何判断 "Claude 正确区分了 [security] 级别问题与建议性问题"？
   - 如何判断 "Claude 在 Review 决策前咨询用户而非擅自 Approve"？

2. **缺少基线对比**：
   - 基线行为：用户说"review PR #101 with inline comments" → Claude 可能仅给出文本式反馈而不调用 `commit comment` 命令发布
   - 基线行为：用户说"这个 PR 有哪些安全问题" → Claude 可能仅文本输出分析，不生成结构化评论格式
   - 基线行为：用户说"leave comments on the PR" → Claude 可能直接发布而不先展示草稿确认
   - 基线行为：用户说"行号 42 有问题" → Claude 可能基于文件内容猜测行号，而非基于 diff 的 `+` 行号

3. **无压力测试场景**：
   - PR 有 50+ 变更文件，总计 200 处 `+` 行 → Claude 是否逐一分析？是否需要批量化处理？
   - diff 显示行号 100 但实际评论目标行为在 115（跨 hunk 引用）→ 行号验证策略
   - PR 描述中写"不要评论 lint 问题"但 diff 中包含大量 lint 违规 → Claude 是否遵循用户指令？
   - 用户在多个会话中重复请求审查同一 PR → Claude 是否去重？是否报告"已评论过"？
   - PR 处于 Draft 状态 → Claude 是否仍发布会审评论？

4. **无成功标准**：
   - 应定义：行内评论发布的必要前置条件（用户确认、行号验证、PR 状态检查）
   - 应定义：成功生成评论的最低内容结构（维度标签 + 问题简述 + 建议修改 + 行号）
   - 应定义：评论发布失败的回退策略（网络错误重试上限、认证失败处理、平台 API 限流）
   - 应定义：审查汇总报告的最低字段（PR 编号、审查文件数、问题总数、按维度统计、问题清单表）

5. **未使用 writing-skills 方法论验证**：
   - skill 未经历 baseline → gap analysis → write skill → verify 的 TDD 循环
   - skill 创建时未记录"不用 skill 时的 Claude 差距是什么"

### 4.3 评分：❌ 不合格

---

## 五、维度 4 分析 — 与 Superpowers 最佳实践的差距

### 5.1 检查清单

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 遵循 TDD for skills（RED-GREEN-REFACTOR） | ❌ | 无 TDD 流程记录 |
| description 只描述触发条件，不描述流程 | ❌ | description 混合功能描述、流程、方法论承诺 |
| 关键词覆盖（错误信息、症状、同义词、工具） | ⚠️ | 提到 `gitflow-cli pr diff`、`gitflow-cli commit comment`、`PR diff`、`行内评论`、`HEAD commit SHA`、hunk 等术语 |
| 跨引用其他 skills | ❌ | 仅文字提及 `gitflow-cli review comment`（注意事项段落），无结构化 See Also / 相关 Skills |
| 必要时使用 flowchart | ⚠️ | 当前无流程图，但 5 步骤工作流 + 3 个决策点（评论前确认、行号验证、发布前 PR 状态检查）其实需要流程图指导 |

### 5.2 具体问题

1. **description 应为触发条件，当前是功能+流程+方法论描述**：
   - ❌ 当前：`PR 行内评论工作流 — 获取 PR diff 并逐文件分析，针对具体代码行生成行内评论，覆盖逻辑错误、安全隐患、命名规范、边界条件四个维度`
   - ✅ 应为：`Use when reviewing a Pull Request with line-level inline comments, filing PR feedback on specific changed lines, or conducting PR code review focused on logic/security/naming/boundaries.`
   - 关键词覆盖建议（中文）："行内审查"、"行内评论"、"PR 逐行审查"、"代码审查 PR"、"line review"、"inline review on PR"、"review PR line by line"
   - 关键词覆盖建议（英文）："inline review"、"line-level comments"、"PR line review"、"review PR with inline feedback"、"comment on specific lines"、"code review per-file"

2. **缺少结构化跨引用**：应明确引用：
   - `gitflow-pr-review`（总审审查技能，与本 skill 互补——何时用 pr-review 何时用 pr-inline-review）
   - `gitflow-pr`（PR 操作命令封装，包含 `pr diff`、`pr view`）
   - `gitflow-pr-create`（创建 PR 的技能）
   - `gitflow-pr-apply-feedback`（审查后应用反馈）
   - `gitflow-review`（Review 总体决策：approve/request-changes/comment）
   - `superpowers:requesting-code-review`（代码审查请求方法论）
   - `superpowers:receiving-code-review`（接收审查反馈方法论）

3. **内容组织偏向"操作手册 + 命令模板库"而非"执行指令"**：
   - 步骤 1 "获取 PR diff"给出一个 bash 块 + 解析指引——这在人类手册中有用，但 Claude 不需要"解析 diff"的旁白式指导，而是需要 diff 输出结构化提取规则
   - 步骤 3 "生成行内评论"给出完整的 markdown 模板 + 维度标签表——模板本身有价值，但文档格式更像是让用户复制粘贴
   - 建议：skill 用"命令速查 + 4 维度标签速查 + 决策分支 + 发布确认"的决策式结构

4. **缺少"决策点"分支提示**：
   - 决策 1：用户是否要求"直接发布"评论？（no → 展示草稿，等用户确认）
   - 决策 2：行号是否基于 diff 的 `+` 行？（no → 拒绝发布，提示行号不精确）
   - 决策 3：PR 状态是否 open？（no → 拒绝发布，提醒 PR 已关闭）
   - 决策 4：是否需要 Review 决策（Approve/Request Changes）？（是 → 咨询用户而非擅自决策）
   - 四个决策点应使用 if/else 分支结构

5. **技术内容质量优势**：
   - 4 维度审查清单（logic/security/naming/boundary）结构清晰且互斥，实用价值高
   - 行内评论格式模板统一（**[维度标签]** + 问题简述 + 修改建议），便于作者理解和处理
   - SHA、path、line 三要素精确（`commit comment <sha> --path <file> --line <n>`）
   - 注意事项中"如果问题涉及整体架构，使用 review comment"明确区分了行内 vs 总体审查
   - "大量问题（>15）先沟通"体现了对 PR 作者体验的关注
   - 这些优质内容是重构时应保留的核心价值

### 5.3 评分：⚠️ 需改进

---

## 六、改进建议

### P0（必须修复 — 阻断性问题）

| # | 改进项 | 维度 | 说明 |
|---|--------|------|------|
| P0-1 | 重写 description 为触发条件 | D1, D4 | description 必须改为 "Use when..." 格式，仅包含触发关键词 |
| P0-2 | 添加职责边界声明章节 | D2 | 明确评论发布前需用户确认；行号必须基于 diff 精确计算；PR 状态非 open 时禁止发布 |
| P0-3 | 添加红旗列表 | D2 | 标识敏感场景：用户要求直接发布、行号不确定、PR 已关闭、平台不受支持、大量问题（>15）未先沟通 |
| P0-4 | 添加禁止行为清单 | D2 | 🚫 未经确认发布行内评论；🚫 凭猜测定位行号；🚫 代替用户做 Review 决策；🚫 评论非变更行 |
| P0-5 | 添加评论发布确认机制 | D2, D4 | 步骤 4 改为"展示评论草稿 → 等用户确认 → 发布"的两步机制 |

### P1（建议修复 — 提升质量）

| # | 改进项 | 维度 | 说明 |
|---|--------|------|------|
| P1-1 | 重构为结构化模板 | D1 | 添加 Overview / When to Use / Core Pattern / Quick Reference / Implementation / Common Mistakes 章节 |
| P1-2 | 添加 Quick Reference 速查 | D1 | 3 行核心命令（`pr diff` + 分析 + `commit comment`）+ 维度标签速查表 + 发布前确认清单 |
| P1-3 | 添加决策点分支结构 | D1, D4 | 用户确认 → 行号验证 → PR 状态检查 → 发布 / 抑制 使用明确 if/else 分支 |
| P1-4 | 添加关键词覆盖 | D4 | 覆盖中英触发词："行内审查"、"inline review"、"PR 逐行审查"、"line-level review"、"代码审查 PR"、"review PR line by line" |
| P1-5 | 添加结构化跨引用 | D4 | 引用 gitflow-pr-review（总体审查互补）、gitflow-pr、gitflow-pr-create、gitflow-pr-apply-feedback、gitflow-review、superpowers:requesting-code-review |
| P1-6 | 添加基线测试场景 | D3 | 定义不用 skill 时 Claude 的基线行为（可能仅文本输出分析、不发布会审评论、不区分 Review 决策权属） |
| P1-7 | 补充成功标准 | D3 | 发布前验证清单（用户确认 + 行号精确验证 + PR 状态）；成功评论的最低内容结构；失败回退策略 |

### P2（可选改进 — 锦上添花）

| # | 改进项 | 维度 | 说明 |
|---|--------|------|------|
| P2-1 | 添加压力测试场景 | D3 | 50+ 文件 PR 处理策略、跨 hunk 行号验证、Draft PR 处理、重复审查去重、平台 API 限流 |
| P2-2 | 提供英文版 description | D1 | Superpowers 主流语言为英文，description 改用英文可提高国际兼容性 |
| P2-3 | 添加 TDD for skills 验证记录 | D3, D4 | 记录 baseline → 编写 → 验证迭代的过程 |
| P2-4 | 添加 workflow 流程图 | D4 | 5 步骤 + 4 个决策点用 Mermaid flowchart 简化阅读 |
| P2-5 | 区分 pr-review 与 pr-inline-review 互补场景 | D2, D4 | 明确何时用 pr-review（总体审查决策）vs pr-inline-review（行内细节反馈） |

---

## 七、验收标准

- [ ] description 以 "Use when..." 开头，仅描述触发条件（推荐英文）
- [ ] 含职责边界声明章节（含 🚫 禁止行为、✅/❌ 职责范围）
- [ ] 含红旗列表（确认缺失、行号错误、PR 关闭、平台不支持、大量问题未沟通等）
- [ ] 含关键词覆盖（中英触发词、工具名、错误信息）
- [ ] 含跨引用（至少引用 gitflow-pr-review、gitflow-pr、gitflow-review）
- [ ] 文档结构包含 Overview / When to Use / Core Pattern / Quick Reference / Common Mistakes
- [ ] 快速加载时 token 数 < 400 词（留有余裕，详细审查清单移至 `docs/references/`）
- [ ] 含决策点分支结构（用户确认、行号验证、PR 状态检查、Review 决策权属）
- [ ] 含成功标准定义（发布前验证清单、评论内容结构、失败回退）
- [ ] 含前置条件检查（PR 存在、平台支持、diff 可获取）
- [ ] 评论发布前必须先展示草稿并获用户确认
- [ ] 不代替用户做 Review 决策、不评论非变更行、不在 PR 关闭后发布

---

## 八、与同类 Skill 对比

| 对比项 | gitflow-pr-inline-review | gitflow-pr-review | gitflow-autoreport-bug |
|--------|-------------------------|-------------------|----------------------|
| 边界风险等级 | 🔴 高（发布会审评论到公开可见的 PR） | 🟡 中（发总体审查决策） | 🟡 中（涉及 Issue 创建） |
| 职责边界 | ❌ 缺失 | ❌ 缺失 | ✅ 完整 |
| 测试场景 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 |
| 触发条件 | ❌ 功能+流程+方法论描述 | ❌ 功能描述 | ⚠️ 描述流程 |
| 跨引用 | ❌ 仅文字提及 review comment | ❌ 缺失 | ❌ 缺失 |
| 命令质量 | ✅ 高（SHA + path + line 精确） | ✅ 高（review 决策命令） | ✅ 高 |
| 方法论质量 | ✅ 高（4 维度清单结构清晰互斥） | ✅ 高（6 维度清单全面） | ⚠️ 中（分析+去重流程） |
| Token 数 | ⚠️ 紧（479 vs 500 上限） | ⚠️ 紧 | ⚠️ 中 |
| 结构化程度 | ⚠️ 线性步骤完整 | ⚠️ checklist + 步骤 | ✅ 结构化最佳 |

**关键发现：** gitflow-pr-inline-review 是所有 skill 中边界风险最高的——它向公开可见的 PR 发布会审评论，误发后难以撤回。但其职责边界完全缺失，形成"高风险操作 + 无边界声明"的最危险组合。同时，本 skill 与 `gitflow-pr-review` 存在天然的互补关系（行内 vs 总体），但两者之间无结构化跨引用，无法帮助 Claude 判断在具体场景应加载哪个 skill。

---

## 九、总结

gitflow-pr-inline-review 当前的定位是"操作手册 + 审查清单 + 命令模板库"混合体——它有结构化的 4 维度审查清单、精确的行内评论命令格式、实用的注意事项。但与 Superpowers skill 形态差距在于：

核心差距：
1. **缺乏触发条件** → Claude 何时应加载此 skill？（用户说"行内审查 PR"？"逐行 feedback"？"review with inline comments"？）
2. **缺乏职责边界** → 涉及发布会审评论（公开可见、不可逆），边界缺失是所有 skill 中风险最高的
3. **缺乏可测试性** → 如何验证 Claude 在发布确认决策、行号验证、Review 决策权属的边界内执行？
4. **token 余裕极窄** → 当前 479 词，加入职责边界后将超出 500 上限，必须同步压缩

重构方向：保留高质量 4 维度清单和命令精确性（重构为 Quick Reference + 决策分支），将详细审查清单和示例命令移至 `docs/references/pr-review-checklist.md`，添加职责边界声明和红旗列表（含发布前确认机制），重写 description 为触发条件，添加跨引用和 Success Criteria。重构后预期 token 从 479 词降至 ~280 词，同时将边界风险从 🔴 高降至 🟡 中（通过明确的发布确认机制）。

**特殊关注点（发布确认机制）：** gitflow-pr-inline-review 的核心边界要求是"发布前必须获得用户确认"。这与 TDD for skills 中"防止合理化借口"的关系尤其密切——Claude 可能以"这是作者需要的反馈"或"高质量审查应该直接发布"为理由跳过一个明确的用户确认步骤。重构需要：展示评论草稿、等待用户确认、然后发布——这是防止越界发布的最关键设计约束。

**关联关注点（与 pr-review 的互补）：** gitflow-pr-inline-review 与 gitflow-pr-review 是天然的互补技能：
- `gitflow-pr-review`：总体审查 → 6 维度 checklist → 决策 Approve/Request Changes/Comment
- `gitflow-pr-inline-review`：行内细节审查 → 4 维度 checklist → 生成行内评论 → 辅助 Review 决策

重构后应建立双向跨引用，帮助 Claude 在用户发起"审查 PR"请求时选择正确的 skill。
