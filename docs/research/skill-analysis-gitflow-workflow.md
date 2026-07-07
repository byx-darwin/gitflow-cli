# gitflow-workflow Skill 分析报告

> **分析日期：** 2026-07-07
> **分析目标：** `skills/gitflow-workflow/SKILL.md`
> **对应 Issue：** #38
> **分析维度：** 4 个维度（结构规范、职责边界、可测试性、Superpowers 最佳实践）

---

## 一、当前状态总览

| 维度 | 评分 | 说明 |
|------|------|------|
| 维度 1：Skill 结构和文档规范 | ❌ 不合格 | description 为功能描述而非触发条件；缺少 Overview / When to Use / Core Pattern / Quick Reference / Common Mistakes 章节；token 1725 词远超上限（超标 1225 词 / 245%）；含大量叙事性示例；四阶段流程瀑布图嵌入代码 |
| 维度 2：职责边界清晰度 | ⚠️ 需改进 | 有禁止行为清单和流程检查点，但缺少"合理化借口"反制表格、红旗列表和结构化 ✅/❌ 职责范围 |
| 维度 3：可测试性 | ❌ 不合格 | 完全缺失测试场景、基线测试、压力测试和成功标准 |
| 维度 4：与 Superpowers 最佳实践差距 | ⚠️ 需改进 | description 违反规范（混合功能+流程）；无 TDD 流程；有部分关键词覆盖和跨引用但非结构化；flowchart 存在但嵌入代码 |

**总体评估：** gitflow-workflow 是一份完整的"四阶段全流程编排手册"，它定义了从需求澄清到交付后检查的完整工作流，闸门机制和合规检查点设计合理。但文档定位为"操作手册 + 流程规范"而非"可执行 skill 指令"——它告诉 Claude "这个流程是什么"，却没有告诉 Claude "在什么情况下加载这个 skill"以及"如何逐步遵循并验证"。1725 词远超 Superpowers 要求的 500 词上限（超标 245%），加上 3 个完整使用场景叙事，冗长的描述会干扰 Claude 执行精确度。

---

## 二、维度 1 分析 — Skill 结构和文档规范

### 2.1 检查清单

| 检查项 | 状态 | 说明 |
|--------|------|------|
| YAML frontmatter 含 name 字段 | ✅ | `name: gitflow-workflow` |
| YAML frontmatter 含 description 字段 | ✅ | 存在 description |
| description 以 "Use when..." 开头 | ❌ | 当前为 "gitflow-cli 全流程开发编排 — 从需求澄清到代码交付的四阶段闸门驱动工作流..." |
| description 只描述触发条件 | ❌ | 混合功能描述、流程承诺和效果声明 |
| 含 Overview 章节 | ❌ | 无结构化 Overview，仅有第一段说明 |
| 含 When to Use 章节 | ❌ | 无触发条件说明 |
| 含 Core Pattern 章节 | ❌ | 无核心模式/算法骨架 |
| 含 Quick Reference | ❌ | 无快速对照表（虽有模式对比表，但非"触发→执行→输出"速查） |
| 含 Implementation 章节 | ✅ | 四个阶段含步骤说明和 bash 命令 |
| 含 Common Mistakes | ❌ | 无常见错误说明（"注意事项"段落为操作提示而非 Common Mistakes） |
| Token 效率 < 500 词 | ❌ | 1725 词（`wc -w`），超标 1225 词（245%） |
| 无叙事性示例反模式 | ❌ | 含 3 个完整使用场景（从零开始新功能、修复 bug、已有关联 Issue），每个 50-80 行叙事描述 |
| 无多语言稀释 | ⚠️ | 全文中文，未提供英文 description |
| 无流程图中嵌入代码 | ❌ | 阶段总览图为 ASCII art 嵌入代码块；嵌入 Superpowers/gitflow 调用关系 |

### 2.2 具体问题

1. **description 严重违反 Superpowers 规范**：
   - 当前：`gitflow-cli 全流程开发编排 — 从需求澄清到代码交付的四阶段闸门驱动工作流，指挥 gitflow-cli 和 Superpowers 协同完成完整开发周期`
   - 问题：这是功能描述 + 流程描述 + 效果承诺，不是触发条件
   - 应为：`Use when the user wants to start a full development cycle for a new feature, bug fix, or enhancement — from requirement clarification through code delivery. Use when the user explicitly asks for a "workflow", "develop a feature", "fix a bug systematically", or provides a high-level requirement that needs breakdown.`
   - 后果：Claude 无法准确判断何时应加载此 skill——用户说"我想加个功能"和"帮我修复 #42"是否都应触发？

2. **Token 严重超标（1725 词 vs 500 词上限）**：
   - 主因分析：
     - 模式对比表 + 说明：约 80 词
     - 阶段总览 ASCII 图：约 20 词
     - 完整模式/快速模式 Skills 清单 + 说明：约 200 词
     - 禁止行为 + 流程检查点：约 80 词
     - Phase 1-4 各步骤详细描述 + bash 命令：约 600 词
     - 计划文档完整结构模板（含 task N+1/N+2/N+3）：约 350 词
     - 3 个完整使用场景叙事：约 300 词
     - 阶段回退说明：约 50 词
     - 注意事项：约 50 词
   - 最大可优化项：计划文档模板（移至 `docs/templates/workflow-plan.md`）、3 个使用场景叙事（由 1 行触发语句替代）、模式对比（由 Quick Reference 替代）

3. **缺少结构化快速导航**：
   - Superpowers 技能推荐 Overview + When to Use + Core Pattern + Quick Reference + Implementation + Common Mistakes 的组合
   - 当前结构类似于线性流程手册，Claude 无法快速判断当前处于哪个阶段、该做什么

4. **使用示例为叙事性反模式**：
   - 场景 1："从零开始一个新功能"——先讲故事再列步骤
   - 场景 2："修复一个 Bug"——同上
   - 场景 3："已有关联 Issue 的增量开发"——同上
   - 叙事性示例消耗 token 但不增加执行精确度——Claude 可能会"背诵"故事而非执行步骤

5. **嵌入代码块的总览图**：
   - Phase 1-4 阶段总览使用了 ASCII art 嵌入代码块
   - 这是不必要的视觉装饰——对于编排型 skill，Quick Reference 表格比 ASCII 图更高效

### 2.3 评分：❌ 不合格

---

## 三、维度 2 分析 — 职责边界清晰度

### 3.1 检查清单

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 有明确的职责边界声明 | ⚠️ | 文档开头有简短说明（"编排层只做指挥，不直接执行操作"），但无独立的职责边界声明章节 |
| 有禁止行为清单（🚫 不得...） | ✅ | "禁止行为"章节含 5 条具体禁止项 |
| 有职责范围说明（✅ 负责... / ❌ 不负责...） | ⚠️ | 有隐含说明但无结构化表格 |
| 有"合理化借口"反制表格 | ❌ | 完全缺失 |
| 有红旗列表（Red Flags） | ❌ | 完全缺失 |

### 3.2 具体问题

1. **职责边界声明非结构化**：
   - 第一段提到"编排层只做指挥，不直接执行操作"，但缺乏独立章节和结构化声明
   - 未使用 ✅/❌ 对比表格明确表达职责范围

2. **禁止行为清单质量较高**：
   - ✅ "完整模式禁止跳过任何 Superpowers skill"
   - ✅ "快速模式禁止跳过 TDD 和 Code Review"
   - ✅ "两种模式禁止跳过 Phase 4"
   - ✅ "禁止合并步骤"
   - ✅ "禁止自行实现流程"
   - 这 5 条禁止行为是文档中结构最好的部分

3. **缺少"合理化借口"反制**：
   - 编排型 skill 特别容易受到合理化借口攻击：
     - "这个需求很简单，不需要 brainstorming" → 违反"完整模式禁止跳过"
     - "我已经有计划了，直接开始执行吧" → 违反"闸门不可跳过"
     - "TDD 太慢了，这次先跳过" → 违反"快速模式禁止跳过 TDD"
     - "Issue 已经存在了，不需要再创建" → 违反 Phase 1 必须步骤
   - 应在文档中明确列出这些借口并给出反制回应

4. **缺少红旗信号**：
   - 用户要求"跳过某个阶段"
   - 用户要求"这个太简单了，不用走完整流程"
   - 用户要求"先开始编码，后面再补文档"
   - 用户要求"合并多个步骤一起执行"
   - 用户要求"这次不需要代码审查"
   - Tech Lead 要求绕过流程

### 3.3 评分：⚠️ 需改进

**对比参考（gitflow-autoreport-bug）：** 该 skill 通过完整的"职责边界声明 + 🚫 禁止行为 + ✅/❌ 职责范围 + 🔧 修复流程"章节树立了边界声明标杆。gitflow-workflow 有禁止行为清单（质量较高），但缺少反制借口表格和红旗列表，且有结构化职责范围说明的空间。

---

## 四、维度 3 分析 — 可测试性

### 4.1 检查清单

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 定义了测试场景 | ❌ | 无测试场景定义 |
| 有基线测试（无 skill 时的行为） | ❌ | 缺失 |
| 有压力测试场景 | ❌ | 缺失 |
| 有成功标准 | ❌ | 有合规检查清单但非可验证的成功标准 |
| 可使用 writing-skills 方法论测试 | ❌ | 无测试钩子 |

### 4.2 具体问题

1. **零测试覆盖**：
   - 对于编排型 skill，测试场景应覆盖：
     - 触发条件测试：用户说"我想加个功能"时 Claude 是否加载了此 skill？
     - 边界测试：用户只说"帮我分析一下 issue"时 Claude 是否不应加载此 skill？
     - 流程遵循测试：Claude 是否按 4 阶段顺序执行？
     - 闸门遵守测试：跳过某个阶段时 Claude 是否阻止？
     - 边界遵守测试：用户要求跳过某个阶段时 Claude 是否拒绝？

2. **合规检查清单 ≠ 成功标准**：
   - 当前每个 Phase 结尾有"合规检查清单"，但这些是给 Claude 执行的内部检查，而非给人类或测试框架验证的"成功标准"
   - 成功标准应可被独立验证：运行完成后，是否有客观证据证明 skill 被正确执行？

3. **缺少基线对比**：
   - 基线行为：用户说"我想添加自动补全功能" → Claude 可能直接开始编写代码（不创建 Issue、不做需求分析、不制定计划、不执行代码审查）
   - 基线行为：用户说"修复 #42" → Claude 可能直接读取 Issue 然后修改代码（跳过 phase 1/2/4）
   - 基线差距：编排型 skill 的核心价值是强制 Claude 遵循完整流程，而非"能简则简"

4. **无压力测试场景**：
   - 时间压力：用户要求"30 分钟内完成"
   - 简化诱惑：用户说"这个太简单了，不需要走完整流程"
   - 疲劳压力：用户要求"赶紧完成，已经工作 12 小时了"
   - 权威压力：Tech Lead 要求跳过某些步骤
   - 沉没成本：已经花了 2 小时分析，不想走流程
   - 信息过载：多个 issue 同时处理时

5. **未使用 writing-skills 方法论验证**：
   - skill 未经历 baseline → gap analysis → write skill → verify 的 TDD 循环
   - 未记录"不用 skill 时的 Claude 差距是什么"

### 4.3 评分：❌ 不合格

---

## 五、维度 4 分析 — 与 Superpowers 最佳实践的差距

### 5.1 检查清单

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 遵循 TDD for skills（RED-GREEN-REFACTOR） | ❌ | 无 TDD 流程记录 |
| description 只描述触发条件，不描述流程 | ❌ | description 混合功能描述、流程承诺和角色声明 |
| 关键词覆盖（错误信息、症状、同义词、工具） | ⚠️ | 文档涉及 workflow、brainstorming、issue、PR、pipeline 等关键词，但无显式关键词章节 |
| 跨引用其他 skills | ⚠️ | 文档中多次提到其他 skills 的名称（如 `superpowers:brainstorming`），但无结构化 See Also 章节 |
| 必要时使用 flowchart | ⚠️ | 有 ASCII art 图展示四阶段，但嵌入代码块；四阶段闸门流转确实需要流程图指导 |

### 5.2 具体问题

1. **description 违反规范 — 是功能描述而非触发条件**：
   - ❌ 当前：`gitflow-cli 全流程开发编排 — 从需求澄清到代码交付的四阶段闸门驱动工作流，指挥 gitflow-cli 和 Superpowers 协同完成完整开发周期`
   - ✅ 应为：`Use when the user wants to start a full development cycle for a new feature, bug fix, or enhancement — from requirement clarification through code delivery. Use when the user explicitly asks for a "workflow", "develop a feature", "fix a bug systematically", or provides a high-level requirement that needs breakdown.`
   - 建议关键词覆盖（中英双语）：
     - "开发功能" / "develop a feature" / "add a feature" / "新功能"
     - "修复 bug" / "fix a bug" / "bugfix" / "修复问题"
     - "完整流程" / "workflow" / "开发流程" / "gitflow"
     - "需求澄清" / "requirements" / "需求分析"
     - "计划制定" / "plan" / "实现计划"
     - "代码审查" / "code review" / "PR"

2. **缺少跨引用结构化章节**：
   - 文档中涉及的 skills 包括：
     - `superpowers:brainstorming`
     - `gitflow-issue-create`
     - `gitflow-issue-review`
     - `superpowers:writing-plans`
     - `superpowers:subagent-driven-development`
     - `gitflow-pipeline-analyzer`
     - `gitflow-issue-triage`
     - `gitflow-review`
     - `gitflow-quality`
     - `gitflow-pr-create`
     - `gitflow-pr-review`
   - 应在文档末尾添加结构化的 "See Also" 章节

3. **内容组织偏向"流程规范文档"而非"执行指令"**：
   - 大量使用"目标"、"说明"、"注意"等旁叙式表述
   - Superpowers skill 应使用命令式和条件判断：`当 X 时，执行 Y`
   - 重构方向：将 4 个 Phase 从"描述+示例"模式改为"条件→动作→产出→验证"模式

4. **flowchart 的具体建议**：
   - 当前 ASCII art 总览图可改为 Mermaid flowchart（更简洁）
   - 确实需要流程图：四阶段闸门流转 + 回退路径是复杂流程，flowchart 能显著提升可读性
   - 推荐在 Overview 中嵌入 Mermaid flowchart

5. **唯一的优势 — 闸门机制设计合理**：
   - 4 个 Phase 之间的闸门校验方法明确（如 `gitflow-cli issue view <number>`）
   - 违规处理路径清晰（输出阻断信息并停止）
   - 合规检查清单逐项打勾的设计实用
   - 这些优质内容是重构时应保留的核心

### 5.3 评分：⚠️ 需改进

---

## 六、改进建议

### P0（必须修复 — 阻断性问题）

| # | 改进项 | 维度 | 说明 |
|---|--------|------|------|
| P0-1 | 重写 description 为纯触发条件 | D1, D4 | description 决定 Claude 何时加载 skill，必须改为 "Use when..." 格式，移除功能描述和流程承诺 |
| P0-2 | 降低 token 至 < 500 词 | D1 | 从 1725 词降至 < 500 词：移除 3 个叙事性场景（300→1 行概要），将计划文档模板移至独立文件（350→50 行引用），模式对比表压缩至 Quick Reference（80→30 词） |
| P0-3 | 添加职责边界声明章节（含结构化 ✅/❌ 职责范围） | D2 | 即使是编排型 skill 也需明确：✅ 负责（流程编排、闸门校验、审计日志） vs ❌ 不负责（代码实现、需求分析、计划文档内容填充） |
| P0-4 | 添加"合理化借口"反制表格 | D2 | 反制"太简单了"、"已经知道需求了"、"先编码后补流程"等常见借口 |
| P0-5 | 添加红旗列表 | D2 | 标识压力场景：用户要求跳过阶段、权威人物要求简化、时间紧迫等 |

### P1（建议修复 — 提升质量）

| # | 改进项 | 维度 | 说明 |
|---|--------|------|------|
| P1-1 | 重构为结构化模板 | D1 | 添加 Overview / When to Use / Core Pattern / Quick Reference / Implementation / Common Mistakes 章节 |
| P1-2 | 添加 Quick Reference 速查表 | D1 | "触发条件 → 选择模式 → 获取 Issue → 进入 Phase 1" 一页纸速查 |
| P1-3 | 添加 Mermaid flowchart 替换 ASCII art | D1, D4 | 四阶段闸门流转 + 回退路径用 Mermaid 表示 |
| P1-4 | 添加关键词覆盖章节（中英双语） | D4 | "开发功能"、"修复 bug"、"workflow"、"feature request"、"bug fix" 等 |
| P1-5 | 添加结构化 See Also / 跨引用章节 | D4 | 列出所有编排涉及的 skills（brainstorming、issue-create、writing-plans 等） |
| P1-6 | 将计划文档模板移至 `docs/templates/workflow-plan.md` | D1 | 降低 SKILL.md 长度，通过引用方式加载详细模板 |
| P1-7 | 添加前置条件检查 | D1 | 验证 `gitflow-cli` 可用、在 git 仓库中、认证状态 |
| P1-8 | 添加错误处理章节 | D1 | 各阶段子 skill 失败时的降级策略 |
| P1-9 | 添加基线测试场景 | D3 | 定义不用 skill 时 Claude 的基线行为（直接编码、跳过流程等） |
| P1-10 | 定义成功标准 | D3 | 4 阶段流程完成的客观证据清单 |
| P1-11 | 添加工具/命令章节 | D1 | 列出所有涉及的 gitflow-cli 子命令速查 |

### P2（可选改进 — 锦上添花）

| # | 改进项 | 维度 | 说明 |
|---|--------|------|------|
| P2-1 | 添加压力测试场景 | D3 | 时间压力、简化诱惑、权威压力、疲劳压力组合 |
| P2-2 | 提供英文版 description | D1 | Superpowers 主流为英文 |
| P2-3 | 添加 TDD for skills 验证记录 | D3, D4 | 记录 baseline → 编写 → 验证迭代的过程 |
| P2-4 | 精简使用示例为 1 行概要 | D1 | 3 个叙事性场景改为 1 行模式化示例 |
| P2-5 | 添加工作流失败恢复指南 | D1 | 部分 Phase 失败后如何恢复（非回退） |
| P2-6 | 添加 Issue 模板关联 | D4 | 关联到 `docs/templates/` 中的 Issue 模板 |

---

## 七、验收标准

- [ ] description 以 "Use when..." 开头，仅描述触发条件（推荐英文）
- [ ] 不含功能描述、流程说明或效果承诺
- [ ] 含职责边界声明章节（含 🚫 禁止行为、✅/❌ 职责范围）
- [ ] 含红旗列表（要求跳过阶段、要求简化流程、先编码后补等）
- [ ] 含"合理化借口"反制表格（至少 5 个常见借口）
- [ ] 含关键词覆盖（中英触发词、同义词、工具名）
- [ ] 含结构化 See Also 章节（至少引用 5 个相关 skill）
- [ ] 文档结构包含 Overview / When to Use / Core Pattern / Quick Reference / Implementation / Common Mistakes
- [ ] Token 数 < 500 词（计划文档模板和完整场景移至独立文件）
- [ ] 含前置条件检查
- [ ] 含错误处理章节（各阶段子 skill 失败时的降级策略）
- [ ] 含 Mermaid flowchart 展示四阶段闸门流转 + 回退路径
- [ ] 含基线测试场景（至少 2 个）
- [ ] 含成功标准定义（4 阶段完成的客观证据清单）
- [ ] 3 个叙事性示例已精简为 1 行模式化示例
- [ ] 不跳阶段、不合并步骤、不自行实现流程

---

## 八、与同类 Skill 对比

| 对比项 | gitflow-workflow | gitflow-autoreport-bug | gitflow-pipeline-analyzer |
|--------|------------------|----------------------|--------------------------|
| 定位 | 编排型（调用其他 skills） | 执行型（创建 Issue） | 分析型（生成报告） |
| 职责边界 | ⚠️ 有禁止行为但无章节 | ✅ 完整声明（含修复流程） | ❌ 缺失 |
| 测试场景 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 |
| 触发条件 | ❌ 功能描述 | ⚠️ 描述流程 | ❌ 功能描述 |
| 跨引用 | ⚠️ 隐式提及 | ❌ 缺失 | ❌ 缺失 |
| 流程复杂度 | ⭐⭐⭐⭐⭐（4 阶段 + 闸门） | ⭐⭐（6 步骤线性） | ⭐⭐⭐（7 步骤 + 分支） |
| Token 数 | ❌ 1725 词（超标 245%） | ✅ ~380 词 | ❌ 813 词（超标 63%） |
| 禁止行为清单质量 | ✅ 高（5 条具体禁止项） | ✅ 高（5 条具体禁止项） | N/A |
| 叙事性示例 | ❌ 3 个完整场景 | ✅ 无 | ❌ 完整报告模板 |
| 闸门/检查点设计 | ✅ 高质量 | ⚠️ 基础 | ⚠️ 基础 |

**关键发现：** gitflow-workflow 在流程复杂度上远超其他 skills——四阶段 + 闸门 + 合规检查 + 回退机制的完整设计在项目中独一无二。其禁止行为清单质量与 gitflow-autoreport-bug 并列最高。但 token 超标 245% 也是所有 skill 中最严重的——它承载了太多"计划文档模板"和"叙事性示例"这类应移至独立文件的内容。

---

## 九、总结

gitflow-workflow 当前的定位是"全流程编排手册 + 计划文档模板库 + 使用教程"混合体——它尝试在一个 SKILL.md 中同时扮演流程规范、执行指令、教学文档和模板仓库四个角色。

核心差距：
1. **缺乏触发条件** → Claude 何时应加载此 skill？（用户说"我想加个功能"？"修复 #42"？"开始一个 workflow"？）
2. **token 超标 245%** → 1725 词中约 650 词（38%）是计划文档模板和叙事性示例，应移至 `docs/templates/`
3. **职责边界非结构化** → 虽有高质量禁止行为清单，但缺反制借口和红旗——编排型 skill 是"合理化借口"重灾区
4. **零可测试性** → 如何验证 Claude 在正确场景触发、在边界内执行、遵循完整流程？

重构方向：
- 保留核心价值：闸门机制、合规检查、流程步骤、禁止行为清单
- 分离至模板文件：计划文档模板（→ `docs/templates/workflow-plan.md`）、使用示例（→ `docs/templates/workflow-examples.md`）
- 添加缺失结构：Overview / When to Use / Quick Reference / See Also / Red Flags / Rationalization Rebuttal / Common Mistakes
- 添加 Mermaid flowchart 展示四阶段闸门流转
- 添加测试场景和成功标准（→ `docs/superpowers/tests/skills/gitflow-workflow-test.md`）
- 重写 description 为触发条件格式

重构后预期 token 从 1725 词降至 ~450 词（不含模板文件），同时添加职责边界、红旗、反制借口、测试场景等必备章节，使其从"编排手册"转型为符合 Superpowers 规范的完整 skill。
