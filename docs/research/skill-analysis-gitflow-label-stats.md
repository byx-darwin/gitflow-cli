# gitflow-label-stats Skill 分析报告

> **分析日期：** 2026-07-07
> **分析目标：** `skills/gitflow-label-stats/SKILL.md`
> **分析维度：** 4 个维度（结构规范、职责边界、可测试性、Superpowers 最佳实践）

---

## 一、当前状态总览

| 维度 | 评分 | 说明 |
|------|------|------|
| 维度 1：Skill 结构和文档规范 | ⚠️ 需改进 | frontmatter 基本合规，但 description 违反触发条件规范；文档结构以「工作流 + 模板 + 示例 + 注意事项」代替 Superpowers 核心章节；token 估算约 700-900 词当量（含模板与示例）；输出示例占大量篇幅 |
| 维度 2：职责边界清晰度 | ❌ 不合格 | 完全缺失职责边界声明、禁止行为清单和红旗列表 |
| 维度 3：可测试性 | ❌ 不合格 | 完全缺失测试场景、基线测试和成功标准 |
| 维度 4：与 Superpowers 最佳实践的差距 | ⚠️ 需改进 | 触发关键词完全缺失；未遵循 TDD；有跨引用（步骤 6 提及 `gitflow-issue-triage`）但无 See Also 章节；flowchart 非必要（线性步骤） |

**总体评估：** gitflow-label-stats 当前是一份「统计模板 + 工作流说明」，它在报告模板结构（标签分组统计、优先级分布、分类覆盖率、改进建议四层）和具体健康度阈值判断上做得相当扎实，但不符合 Superpowers 对 skill 的结构性要求。它告诉 Claude "统计什么维度" 和 "输出什么样" ，但没有回答 "何时触发此 skill"、"报告质量如何验证"、"何时不应输出推断性结论"。

---

## 二、维度 1 分析 — Skill 结构和文档规范

### 2.1 检查清单

| 检查项 | 状态 | 说明 |
|--------|------|------|
| YAML frontmatter 含 name 字段 | ✅ | `name: gitflow-label-stats` |
| YAML frontmatter 含 description 字段 | ✅ | 存在 description |
| description 以 "Use when..." 开头 | ❌ | 当前为 "标签统计分析工作流 — 按标签分组统计 Issue 数量…" 功能描述 |
| description 只描述触发条件 | ❌ | 混合功能描述与关键词，未使用触发条件格式 |
| 含 Overview 章节 | ⚠️ | 有简短介绍段落，但非正式 Overview 章节 |
| 含 When to Use 章节 | ❌ | 无 |
| 含 Core Pattern 章节 | ❌ | 无 |
| 含 Quick Reference | ❌ | 无 "触发 → 执行 → 输出" 快速对照 |
| 含 Implementation 章节 | ⚠️ | 有 6 步骤工作流，但偏描述性说明而非命令式指令 |
| 含 Common Mistakes | ❌ | 无结构化常见错误说明 |
| Token 效率 < 500 词 | ❌ | 估算约 700-900 词当量（含输出模板、示例、多表格），远超建议上限 |
| 无叙事性示例反模式 | ⚠️ | 使用示例中出现具体仓库名 `org/gitflow-cli`、具体数字（47 个 Issue）、具体仓库路径 `/tmp/*.txt`，属于虚构示例 |
| 无多语言稀释 | ⚠️ | 内容纯中文，未提供英文对照；Superpowers 主流为英文 |
| 无流程图中嵌入代码 | ✅ | 无流程图 |

### 2.2 具体问题

1. **Frontmatter description 违反 Sonicpowers 规范**：
   - 当前：`标签统计分析工作流 — 按标签分组统计 Issue 数量，分析优先级分布，识别未分类 Issue，输出标签统计报告`
   - 问题：description 混合了三类内容——工作流说明、功能描述、输出结果描述
   - 应为：`Use when the user asks to analyze label statistics, summarize label distribution, identify unclassified issues, or says "标签统计", "标签分布", "issue 分布", "label analysis", "issue health check", "show label report".`
   - 触发关键词（标签统计、标签分布、issue 分布、label analysis、issue health check）应作为核心内容

2. **文档结构偏向「模板 + 示例」而非「可执行指令」**：
   - 当前结构：工作流 6 步骤 → 使用示例 3 段 → 输出示例 → 注意事项 8 条
   - 输出模板 + 完整示例占据文档近 50% 篇幅（完整报告模板 + 具体数据填充示例）
   - 缺少 Superpowers 要求的核心章节：
     - 前置条件（是否需要 `gitflow-cli` 认证）
     - 输入验证（仓库不存在时的处理）
     - 条件分支（仓库无标签时如何报告）
     - 输出汇总（"用户最终收到什么"一句话摘要）

3. **Token 效率问题**：
   - 当前约 700-900 词当量，远超 500 词上限
   - 主要贡献者：
     - 步骤 1-6 的多个 Markdown 表格（标签分类参考、统计维度、汇总统计表、优先级分布、健康度判断）
     - 完整输出模板 24 行（含虚构数据）
     - 使用示例 3 段（含 bash 脚本）
   - 优化方向：将完整输出模板移至独立文件（如 `templates/label-stats-report.tmpl`），skill 中仅保留模板引用和关键约束

4. **虚构示例违反零虚构原则**：
   - 输出示例中出现 `org/gitflow-cli` 仓库名、`2026-07-02` 具体日期、具体数字（47、12、28、40 等）
   - 虽然是"示例"，但可能被 GitHub Copilot 等工具直接引用为真实数据
   - 应使用明确占位符（`<repo-owner>/<repo-name>`、`<n>`、`<timestamp>`）

5. **Implementation 步骤偏描述性，缺少命令式骨架**：
   - 当前步骤 1-6 以"做什么"为主，缺少"如何做"的精确指令
   - 例如步骤 4 说"找出没有标签或分类不完整的 Issue"，但没有给出确定性筛选逻辑
   - 步骤 5 给出了报告模板但没有列出必填章节清单
   - 步骤 6 的改进建议只给建议类型，缺少"何时应该给出某条建议"的判断条件

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

1. **完全无职责边界 — 统计类 skill 尤其需要边界声明**：
   - 虽然本 skill 只读（不修改 issue），但无边界声明可能导致：
     - Claude 在分析标签分布时推断团队工作效率（"bug 偏多" → "代码质量差"）
     - Claude 在执行统计时读取非 issue 相关的仓库文件（如 PR discussion）
     - Claude 为"增加报告深度"而推断未分类 issue 的可能标签（分类推断超出统计角色）
   - 职责边界风险等级：🟡 中（只读但可能产生评价性文本）

2. **"改进建议"章节的行为边界模糊**：
   - 步骤 6 的改进建议表："运行 `gitflow-issue-triage` 对所有 Open Issue 进行分类"
   - "运行"是建议还是执行？Claude 是否应该在生成报告后自动执行 `gitflow-issue-triage`？
   - 需要明确：本 skill 只生成报告和建议，是否自动触发行动取决于用户确认

3. **缺少禁止行为清单** — 应明确：
   - 🚫 不得修改任何 Issue 或 Label（本 skill 只读统计）
   - 🚫 不得为未分类 Issue 自动添加标签（应由专门分类 skill 决定）
   - 🚫 不得基于标签分布推断团队或个人绩效
   - 🚫 不得虚构或估算未直接获取的统计数字
   - 🚫 不得访问非 Issue/Label 数据的文件

4. **缺少红旗信号** — 应标识：
   - 用户要求"看看代码质量如何"（误用统计 skill 评估）
   - 用户要求"给所有未分类的 issue 打标"（自动分类超出范围）
   - 用户要求"对比团队成员提交情况"（越界为绩效评估）
   - 用户要求"生成周报"（应改用 `gitflow-weekly-report`）

5. **缺少职责范围说明**：
   - ✅ 负责：调用 issue list、label list 获取数据；计算分组统计；输出格式化报告；提供改进建议文本
   - ❌ 不负责：修改 Issue 或 Label 状态；为 Issue 自动分类；评估团队绩效；替代 `gitflow-issue-triage`

### 3.3 评分：❌ 不合格

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

1. **零测试覆盖 — 无法验证 Claude 执行质量**：
   - 如何判断"Claude 正确统计了每个标签的 open/closed 数量"？
   - 如何判断"Claude 正确识别了未分类 Issue"？
   - 如何判断"Claude 的健康度阈值判断实现了步骤 3 的规范"？
   - 如何判断"Claude 在标签体系不一致时（如 `bug` vs `type:bug`）正确合并了同类统计"？

2. **缺少基线对比**：
   - 不使用 skill 时，Claude 的典型行为是：直接运行 `gitflow-cli issue list`，统计标签出现频次，输出无序列表
   - 差距正是 skill 价值：按状态分组、健康度阈值、覆盖率分析、改进建议
   - 基线行为未明确记录，无法定义 skill 的增量价值

3. **无压力测试场景**：
   - 仓库有 100+ 标签（报告格式的可读性）
   - 仓库 Issue > 1000（注意事项提到分批统计但无具体策略）
   - 标签体系混乱（如同一概念有 5 种命名）
   - 仓库完全没有标签
   - 所有 Issue 均未分类

4. **无成功标准**：
   - 应定义：每个标签的 Issue 数量必须与 `gitflow-cli issue list --label <label> --state open` 实际返回一致
   - 应定义：优先级分布百分比总和必须等于 100%
   - 应定义：分类覆盖率各分项加总必须等于总 Issue 数
   - 应定义：健康度判断必须严格遵循阈值表（urgent < 10% → 正常）
   - 应定义：改进建议必须基于实际数据，不得包含无数据支撑的建议

### 4.3 评分：❌ 不合格

---

## 五、维度 4 分析 — 与 Superpowers 最佳实践的差距

### 5.1 检查清单

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 遵循 TDD for skills（RED-GREEN-REFACTOR） | ❌ | 无 TDD 流程记录 |
| description 只描述触发条件，不描述流程 | ❌ | description 混合功能描述与工作流说明 |
| 关键词覆盖（错误信息、症状、同义词、工具） | ❌ | 无显式关键词覆盖；description 未列出触发关键词 |
| 跨引用其他 skills | ⚠️ | 步骤 6 提及 `gitflow-issue-triage`，但无独立 See Also 章节 |
| 必要时使用 flowchart | N/A | 工作流是线性步骤 + 判断表，无需流程图 |

### 5.2 具体问题

1. **description 格式不符合 Superpowers 规范**：
   - ❌ 当前：中文功能描述，列出所有统计维度
   - ✅ 应为：`Use when the user asks to analyze label statistics, summarize label distribution, identify unlabeled or unclassified issues, check issue management health, or says "标签统计", "标签分布", "issue 分布情况", "label analysis", "issue health check", "show label report", "label usage summary".`

2. **关键词覆盖基本缺失**：
   - 未覆盖中文：`标签统计`、`标签分析`、`issue 分布`、`分类情况`、`标签覆盖`、`issue 现状`
   - 未覆盖英文：`label analysis`、`label distribution`、`issue health`、`label coverage`、`tag overview`
   - 未覆盖同义词：`标签健康度`、`标签管理`、`分类覆盖`、`标签使用统计`

3. **跨引用不足**：
   - 步骤 6 建议提及 `gitflow-issue-triage`，但未作为结构化引用
   - 应引用 3 个以上相关 skills：
     - `gitflow-label-milestone`（标签 CRUD）
     - `gitflow-issue-triage`（Issue 自动分类）
     - `gitflow-weekly-report`（周报中可能引用统计结果）
     - `gitflow-repo-onboarding`（首次设置仓库标签）

4. **内容组织偏向「模板 + 规则说明」而非「执行指令」**：
   - Superpowers skill 应告诉 Claude "做什么 + 怎么做 + 不做什么"
   - 当前文档告诉读者"统计什么"和"输出格式"
   - 缺少"条件→动作"决策逻辑（如果所有 Issue 已完整分类 → 报告"覆盖率 100%"）

5. **未遵循 writing-skills 方法论**：
   - 当前 skill 似乎是"基于模板一次性编写"，未记录 baseline → 编写 → 验证 → 迭代过程
   - 输出模板虽完整但缺乏"迭代验证，记录哪些格式偏差常见"的反思

### 5.3 评分：⚠️ 需改进

---

## 六、改进建议

### P0（必须修复 — 阻断性问题）

| # | 改进项 | 维度 | 说明 |
|---|--------|------|------|
| P0-1 | 重写 description 为纯触发条件 | D1, D4 | description 只含触发条件，使用 "Use when..." 格式，移除功能描述与工作流说明 |
| P0-2 | 添加职责边界声明章节 | D2 | 明确本 skill 只读统计、不修改 issue/label、不为未分类 issue 自动打标 |
| P0-3 | 添加禁止行为清单 | D2 | 🚫 不得修改 Issue 或 Label；🚫 不得自动为 Issue 添加标签；🚫 不得基于标签分布推断团队/个人绩效；🚫 不得虚构或估算未获取的统计数字 |
| P0-4 | 添加红旗列表 | D2 | 标识：用户要求评估代码质量、要求自动打标、要求对比团队成员产出 |
| P0-5 | 添加"改进建议"与"自动执行"的边界 | D2 | 明确本 skill 只输出建议文本，不自动执行 `gitflow-issue-triage`，除非用户显式确认 |
| P0-6 | 替换虚构示例为明确占位符 | D1 | 输出模板移除具体仓库名、日期、数字，统一使用 `<repo-owner>/<repo-name>`、`<n>`、`<timestamp>` |

### P1（建议修复 — 提升质量）

| # | 改进项 | 维度 | 说明 |
|---|--------|------|------|
| P1-1 | 重构为结构化模板 | D1 | 添加 Overview / When to Use / Core Pattern / Quick Reference / Implementation / Common Mistakes 章节 |
| P1-2 | 提取输出模板至独立文件 | D1 | 将完整报告模板移至 `templates/label-stats-report.tmpl`，skill 中引用路径，降低 token 占用至 < 500 词 |
| P1-3 | 添加前置条件检查 | D1 | 执行前验证：是否在 git 仓库中、`gitflow-cli` 是否可用、是否有仓库读取权限 |
| P1-4 | 添加决策逻辑（条件分支） | D1, D4 | 仓库无标签 → 报告"暂无标签，建议使用 gitflow-label-milestone 创建"；无 Issue → 报告"暂无 Issue" |
| P1-5 | 添加跨引用章节 | D4 | 引用 gitflow-label-milestone、gitflow-issue-triage、gitflow-weekly-report、gitflow-repo-onboarding |
| P1-6 | 补充关键词覆盖 | D4 | 添加中文触发词（标签分析、issue 分布、分类情况）和英文触发词（label analysis、issue health、label coverage） |
| P1-7 | 定义成功标准 | D3 | 各数字必须与 CLI 返回一致、百分比总和 100%、分类覆盖率加总等于总数 |
| P1-8 | 添加结构化 See Also | D4 | 显式列出 3-5 个相关 skill，避免仅隐式提及 |

### P2（可选改进 — 锦上添花）

| # | 改进项 | 维度 | 说明 |
|---|--------|------|------|
| P2-1 | 定义基线测试场景 | D3 | 定义不用 skill 时 Claude 的基线行为（无序列表标签频次） |
| P2-2 | 定义测试场景和预期输出 | D3 | 3-5 个标准输入（标准仓库、无标签、无 Issue、标签混乱、大量 Issue），附预期报告片段 |
| P2-3 | 添加压力测试场景 | D3 | 超大型仓库（>1000 Issue）、100+ 标签、标签体系全混乱 |
| P2-4 | 提供英文版 description | D1 | description 使用英文 + 中文触发词混合，更符合 Superpowers 主流 |
| P2-5 | 添加 TDD for skills 验证记录 | D3, D4 | 记录 baseline → 编写 → 测试 → 迭代的完整过程 |
| P2-6 | 将 bash 脚本示例移至独立文件 | D4 | "完整的统计分析流程"中的 bash 脚本不是描述 Claude 如何执行，而是 shell 操作手册，可移至 `scripts/stats-label.sh` 或删除 |

---

## 七、验收标准

- [ ] description 以 "Use when..." 开头，仅描述触发条件（英文为主 + 中文关键词）
- [ ] description 不含功能描述、工作流说明或维度枚举
- [ ] 含职责边界声明章节（含 🚫 禁止行为、✅ 职责范围）
- [ ] 含红旗列表（代码质量评估请求、自动打标请求、团队成员对比）
- [ ] "改进建议"章节明确"只建议不执行"的边界
- [ ] 输出模板使用明确占位符（无虚构数据）
- [ ] 含关键词覆盖（中英文触发词、同义词）
- [ ] 含跨引用章节（至少引用 3 个相关 skill）
- [ ] 文档结构包含 Overview / When to Use / Core Pattern / Quick Reference / Implementation
- [ ] Token 数 < 500 词（输出模板移至独立文件）
- [ ] 定义至少 3 个基线测试场景和成功标准
- [ ] 修改内容通过一致性检查

---

## 八、与同类 Skill 对比

| 对比项 | gitflow-label-stats | gitflow-autoreport-bug | gitflow-weekly-report | gitflow-label-milestone |
|--------|--------------------|----------------------|---------------------|----------------------|
| 职责边界 | ❌ 缺失 | ✅ 完整 | ❌ 缺失 | ❌ 缺失 |
| 测试场景 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 |
| 触发条件 | ❌ 功能描述 | ⚠️ 描述流程 | ⚠️ 混合格式 | ❌ 功能描述 |
| 跨引用 | ⚠️ 隐式提及 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 |
| 虚构数据 | ⚠️ 含虚构示例 | ⚠️ 也有虚构示例 | ⚠️ 含虚构示例 | ⚠️ 有虚构示例 |
| 报告模板质量 | ✅ 高（4 层结构） | N/A | ✅ 高（4 层结构） | N/A |
| 阈值判断质量 | ✅ 高（明确阈值表） | N/A | ✅ 高（多场景） | N/A |
| 职责边界风险等级 | 🟡 中（可能推断团队绩效） | 🟡 中（Issue 创建） | 🟡 中-高（跨项目读取） | 🟡 中（CRUD 操作） |
| Token 估算 | ❌ 700-900 词（超标） | ⚠️ 中等 | ❌ 700-900 词（超标） | ✅ ~300 词 |

**关键发现：** gitflow-label-stats 与 gitflow-weekly-report 结构高度相似（模板 + 工作流 + 示例 + 注意事项），均以「报告生成」为定位。但 gitflow-label-stats 在职责边界风险上低于 weekly-report（前置读取 vs 跨项目读取），且健康度阈值判断有明确规则支撑（优于 weekly-report 的模糊推断空间）。两者共同的改进方向是：将报告模板提至独立文件以压缩 token。

---

## 九、总结

gitflow-label-stats 当前的定位是"标签统计报告模板 + 工作流说明"，它在**报告模板结构的完整性**（标签分组统计、优先级分布、分类覆盖率、改进建议四层）和**健康度阈值判断的规则化**（urgent < 10% 正常、10%-20% 关注、>20% 告警）上表现优秀，在**标签分类参考**（类型、优先级、状态、平台、其他五类）上也较周全。

核心差距：
1. **触发条件完全缺失** → Claude 何时应加载此 skill？当前 description 是"工作流 + 功能"枚举
2. **缺乏只读数据 skill 的边界声明** → 推断团队绩效、自动打标越界、虚构数据均无防护
3. **Token 严重超标** → 完整输出模板 + 虚构示例占近 50% 篇幅
4. **缺乏可测试性设计** → 无法验证 Claude 报告的数字准确性

重构方向：保持健康度阈值和分类参考的质量优势（可移至 `docs/label-stats-guide.md`），将 skill 重构为"精简触发条件 + 执行骨架 + 职责边界 + 条件分支 + 模板引用 + 成功标准"的可执行指令。将输出模板和完整示例移至独立文件，使 skill token 降至 < 500 词。
