# gitflow-pipeline-analyzer Skill 分析报告

> **分析日期：** 2026-07-07
> **分析目标：** `skills/gitflow-pipeline-analyzer/SKILL.md`
> **对应 Issue：** #28
> **分析维度：** 4 个维度（结构规范、职责边界、可测试性、Superpowers 最佳实践）

---

## 一、当前状态总览

| 维度 | 评分 | 说明 |
|------|------|------|
| 维度 1：Skill 结构和文档规范 | ⚠️ 需改进 | description 为功能描述而非触发条件；缺少 Overview / When to Use / Core Pattern / Quick Reference 章节；token 数量超标（813 词）；含叙事性示例（完整报告模板） |
| 维度 2：职责边界清晰度 | ❌ 不合格 | 完全缺失职责边界声明、禁止行为清单和红旗列表；skill 为只读分析型，边界风险较低但边界声明仍为必须 |
| 维度 3：可测试性 | ❌ 不合格 | 完全缺失测试场景、基线测试和成功标准 |
| 维度 4：与 Superpowers 最佳实践的差距 | ⚠️ 需改进 | 工作流程步骤明确可执行，分析维度（成功率/失败模式/耗时）结构化程度高；但 description 不合规、无关键词覆盖、无跨引用、未遵循 writing-skills 方法论 |

**总体评估：** gitflow-pipeline-analyzer 是一个"分析报告生成型 skill"——它定义了从数据采集到报告输出的完整分析流水线，三个分析维度（成功率趋势、失败模式、耗时分布）的结构化程度较高，输出模板清晰可操作。但与 Superpowers 要求的 skill 形态差距在于：无法自动判断何时触发、无边界约束（即使只读也应声明"不修改任何流水线配置"）、无测试验证机制、token 超标。

---

## 二、维度 1 分析 — Skill 结构和文档规范

### 2.1 检查清单

| 检查项 | 状态 | 说明 |
|--------|------|------|
| YAML frontmatter 含 name 字段 | ✅ | `name: gitflow-pipeline-analyzer` |
| YAML frontmatter 含 description 字段 | ✅ | 存在 description |
| description 以 "Use when..." 开头 | ❌ | "流水线分析工作流 — 调用 gitflow-cli pipeline report 获取流水线健康数据..." |
| description 只描述触发条件 | ❌ | 混合了功能描述、流程描述和效果承诺 |
| 含 Overview 章节 | ⚠️ | 有 H1 标题和简短介绍，但缺少结构化 Overview |
| 含 When to Use 章节 | ❌ | 无触发条件说明 |
| 含 Core Pattern 章节 | ❌ | 无核心模式/算法骨架 |
| 含 Quick Reference | ❌ | 工作流步骤缺少"触发→执行→输出"快速对照表 |
| 含 Implementation 章节 | ✅ | 步骤 1-7 的工作流明确可执行（高质量） |
| 含 Common Mistakes | ⚠️ | 有"注意事项"段落（8 条），接近 Common Mistakes |
| Token 效率 < 500 词 | ❌ | 813 词（`wc -w`），超标约 313 词（63%） |
| 无叙事性示例反模式 | ⚠️ | 含完整报告模板（约 30 行 markdown）和完整输出示例（约 25 行），属于"复制黏贴模板"而非"指导执行" |
| 无多语言稀释 | ⚠️ | 全中文，未提供英文 description |
| 无流程图中嵌入代码 | ✅ | 无 Mermaid/AST 流程图嵌入 |

### 2.2 具体问题

1. **description 违反 Superpowers 规范**：
   - 当前：`流水线分析工作流 — 调用 gitflow-cli pipeline report 获取流水线健康数据，分析成功率趋势、失败模式、最长耗时，输出分析报告和改进建议`
   - 问题：这是功能描述 + 流程描述 + 效果承诺，不是触发条件
   - 应为：`Use when the user wants to analyze CI/CD pipeline health (success rate trends, failure patterns, duration bottlenecks), diagnose flaky tests, or generate a pipeline improvement report.`
   - 后果：Claude 无法基于自然语言请求准确判断是否加载此 skill（"流水线最近老挂" vs "帮我看看 CI 为什么慢" vs "生成一份流水线报告" 都可能是触发）

2. **Token 超标（813 词 vs 500 词上限）**：
   - 主因：完整报告模板（约 30 行 markdown 表格）+ 完整输出示例（约 25 行填充数据）+ 使用示例（约 20 行 bash）
   - 技能设计考虑：skill 中嵌入完整模板以便 Claude 直接生成报告，但与"触发→运行→输出"的 skill 职责不符——报告模板应是 skill 执行时动态生成的数据源，而非硬编码在 skill 中
   - 建议：skill 保留报告结构概要（3 维度 + 改进建议优先级），完整模板移至 `docs/templates/pipeline-report.md`

3. **缺少结构化快速导航**：
   - Superpowers 技能推荐 When to Use + Core Pattern + Quick Reference + Common Mistakes 的组合
   - 当前用户期待的是"告诉我流水线为什么老失败"的速查，但文档结构是 7 步骤线性教程
   - 建议：添加 Quick Reference 速查表（核心命令 + 分析维度 + 质量等级阈值）

4. **使用示例偏 happy-path**：
   - 步骤 2 中未说明 `pipeline report` 返回空数据时的降级策略
   - 步骤 5 中未说明 `pipeline jobs` 调用失败时的处理
   - 缺少明确分支：数据不足时（如新分支无运行记录）应提示用户而非生成空报告

5. **"使用示例"和"分析报告输出示例"的定位模糊**：
   - "使用示例"展示的是 bash 命令调用，属于 Implementation 的一部分
   - "分析报告输出示例"展示的是完整填充后的报告，属于输出模板
   - 两者应分别归入 Implementation 和 Output Template，而非独立的"使用示例"章节

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

1. **无职责边界 → 只读 skill 仍需要声明**：
   - 虽然 skill 仅调用 `pipeline report`、`pipeline status`、`pipeline jobs`、`pipeline logs` 等只读命令，但缺少边界声明时，Claude 可能在以下场景过度执行：
     - 用户仅要求"看看流水线状态"时，Claude 自动触发重试失败 Pipeline 或取消正在运行的 Pipeline
     - 在分析报告中自动创建 Issue 跟踪问题（未授权）
     - 自动修改流水线配置以"修复"发现的问题
     - 将分析结果自动推送到团队频道（未授权）

2. **缺少禁止行为清单 — 应明确**：
   - 🚫 不得触发、重试、取消或修改任何 Pipeline 运行
   - 🚫 不得自动创建 Issue 或 PR 来跟踪发现的问题
   - 🚫 不得修改流水线配置文件（如 `.gitlab-ci.yml`、`.github/workflows/*.yml`）
   - 🚫 不得将分析结果自动推送到 Slack/邮件等外部渠道
   - 🚫 不得在用户未确认时执行写操作

3. **缺少红旗信号 — 应标识**：
   - 用户要求"自动修复流水线问题"
   - 用户要求"重试所有失败的 Pipeline"
   - 用户要求"把报告发到群里"
   - 在分析过程中用户要求修改 CI 配置

4. **缺少职责范围说明**：
   - ✅ 负责：获取流水线数据、分析成功率/失败模式/耗时、生成报告、提供改进建议
   - ❌ 不负责：修改流水线配置、重试/取消 Pipeline、创建 Issue/PR、推送通知、修复代码问题

5. **"合理化借口"反制（针对分析场景）**：
   - "just fix it" → "分析报告说 build 失败，我直接帮你改代码修一下" → 超出职责范围
   - "auto-retry" → "失败的 Pipeline 我帮你重试一下" → 需要用户确认
   - "share results" → "报告生成好了，我帮你发到群里" → 需要用户确认

### 3.3 评分：❌ 不合格

**对比参考（gitflow-autoreport-bug）：** 该 skill 通过"职责边界声明"章节明确了"修复建议 ≠ 自动修复"的边界。gitflow-pipeline-analyzer 虽然为只读分析型 skill（边界风险低于涉及写操作的 skill），但"分析后自动采取行动"是 Claude 常见的过度执行模式，仍需明确声明边界。

---

## 四、维度 3 分析 — 可测试性

### 4.1 检查清单

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 定义了测试场景 | ❌ | 无测试场景定义 |
| 有基线测试（无 skill 时的行为） | ❌ | 缺失 |
| 有压力测试场景 | ❌ | 缺失 |
| 有成功标准 | ❌ | 仅有格式化的报告模板，未定义为可验证标准 |
| 可使用 writing-skills 方法论测试 | ❌ | 无测试钩子 |

### 4.2 具体问题

1. **零测试覆盖 — 无法验证 Claude 执行质量**：
   - 如何判断 "Claude 正确识别了应触发流水线分析的场景"？（"流水线老挂" vs "帮我跑一下测试"的区别）
   - 如何判断 "Claude 在数据不足时正确降级（提示用户）而非生成空报告"？
   - 如何判断 "Claude 正确计算了成功率趋势（前后半段对比）"？
   - 如何判断 "Claude 在分析过程中未执行任何写操作"？

2. **缺少基线对比**：
   - 基线行为：用户说"流水线最近老挂" → Claude 可能仅运行 `gitflow-cli pipeline status` 获取当前状态，不进行趋势分析和失败模式归类
   - 基线行为：用户说"CI 太慢了" → Claude 可能仅查看最近一次 Pipeline 的耗时，不做 P90/P95 分布分析
   - 基线行为：用户说"帮我看看流水线" → Claude 可能仅列出 Pipeline 列表，不做三维度深度分析

3. **无压力测试场景**：
   - 新分支无任何 Pipeline 运行记录（`pipeline report` 返回空数据）
   - 大量 Pipeline 运行记录（>1000 次）导致分析超时
   - `pipeline report` API 返回部分字段缺失（如无 duration 数据）
   - 多分支并行分析请求
   - 网络隔离环境无法访问 Pipeline API

4. **无成功标准**：
   - 应定义：完整分析应覆盖哪些维度？（成功率趋势 + 失败模式 + 耗时分布 三维度）
   - 应定义：报告的最低字段（概览表 + 三维度详细分析 + 改进建议）
   - 应定义：质量等级阈值（≥95% 健康、80%-94% 关注、<80% 告警）
   - 应定义：数据不足时的降级输出（提示用户无足够数据 + 建议缩小时间范围）

5. **未使用 writing-skills 方法论验证**：
   - skill 未经历 baseline → gap analysis → write skill → verify 的 TDD 循环
   - skill 创建时未记录"不用 skill 时的 Claude 差距是什么"，无法衡量 skill 解决了什么问题

### 4.3 评分：❌ 不合格

---

## 五、维度 4 分析 — 与 Superpowers 最佳实践的差距

### 5.1 检查清单

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 遵循 TDD for skills（RED-GREEN-REFACTOR） | ❌ | 无 TDD 流程记录 |
| description 只描述触发条件，不描述流程 | ❌ | description 混合功能描述与流程描述 |
| 关键词覆盖（错误信息、症状、同义词、工具） | ⚠️ | 提到了 `pipeline report`、`pipeline status`、`pipeline jobs`、`pipeline logs`、`flaky test`、`build 失败`、`test 失败`、`lint 失败`、`deploy 失败`、`timeout` 等术语 |
| 跨引用其他 skills | ❌ | 无 See Also / 相关 Skills |
| 必要时使用 flowchart | ⚠️ | 当前无流程图，但 7 步骤工作流+多分支判断（数据充足 vs 数据不足）需要流程图指导 |

### 5.2 具体问题

1. **description 应为触发条件，当前是功能描述+流程描述**：
   - ❌ 当前：`流水线分析工作流 — 调用 gitflow-cli pipeline report 获取流水线健康数据，分析成功率趋势、失败模式、最长耗时，输出分析报告和改进建议`
   - ✅ 应为：`Use when the user wants to analyze CI/CD pipeline health (success rate trends, failure patterns, duration bottlenecks), diagnose flaky tests, or generate a pipeline improvement report.`
   - 建议关键词覆盖：用户可能表达为"流水线老挂"、"CI 太慢"、"流水线健康检查"、"分析 CI 失败"、"flaky test"、"pipeline report"、"build 不稳定"、"测试超时"、"流水线优化"、"CI analysis"

2. **缺少跨引用**：应明确引用：
   - `gitflow-precommit`（pre-commit 检查与流水线失败的关联）
   - `gitflow-quality`（代码质量检查与 CI 质量的关联）
   - `gitflow-regression`（回归测试与流水线测试失败的关联）
   - `gitflow-weekly-report`（周报中可引用流水线分析结果）
   - `superpowers:systematic-debugging`（流水线失败的系统化调试方法）

3. **内容组织偏向"教程文档"而非"执行指令"**：
   - 步骤 1 是"获取当前分支名称和确认分析时间范围"——这不是 Claude 需要的指令，而是在描述"Claude 不需要做什么"
   - Superpowers skill 应该用命令式和条件判断，而非旁白式文档
   - 当前结构读起来像给用户阅读的操作手册，而非给 Claude 遵循的执行指令

4. **缺少"数据充足性"分支提示**：
   - 场景 A：数据充足（有 Pipeline 运行记录）→ 执行完整三维度分析
   - 场景 B：数据不足（新分支或无运行记录）→ 提示用户并建议扩大时间范围
   - 两个场景应使用 if/else 分支进行区分，避免每次加载时生成空报告

5. **分析维度的结构化优势**：
   - 三维度分析框架（成功率趋势 + 失败模式 + 耗时分布）设计合理，覆盖了流水线健康的核心指标
   - 质量等级阈值（≥95%/80%-94%/<80%）明确可量化
   - 失败模式分类（build/test/lint/deploy/timeout）实用且可扩展
   - 改进建议优先级（🔴紧急/🟠高/🟡中/🟢低）与成功率/失败类型挂钩，逻辑清晰
   - 这些优质内容是重构时应保留的核心价值

### 5.3 评分：⚠️ 需改进

---

## 六、改进建议

### P0（必须修复 — 阻断性问题）

| # | 改进项 | 维度 | 说明 |
|---|--------|------|------|
| P0-1 | 重写 description 为触发条件 | D1, D4 | description 必须改为 "Use when..." 格式，仅包含触发关键词 |
| P0-2 | 添加职责边界声明章节 | D2 | 明确只读分析边界，禁止自动修改 Pipeline 配置/重试/创建 Issue/推送通知 |
| P0-3 | 添加红旗列表 | D2 | 标识敏感场景：用户要求自动修复流水线、重试失败 Pipeline、推送报告到外部渠道 |
| P0-4 | 添加禁止行为清单 | D2 | 🚫 不得触发/重试/取消 Pipeline；🚫 不得修改 CI 配置文件；🚫 不得自动创建 Issue/PR；🚫 不得推送通知 |
| P0-5 | 降低 token 数至 < 500 | D1 | 将完整报告模板和输出示例移至 `docs/templates/pipeline-report.md`，skill 仅保留结构概要 + 分支逻辑 |

### P1（建议修复 — 提升质量）

| # | 改进项 | 维度 | 说明 |
|---|--------|------|------|
| P1-1 | 重构为结构化模板 | D1 | 添加 Overview / When to Use / Core Pattern / Quick Reference / Implementation / Common Mistakes 章节 |
| P1-1b | 添加 Quick Reference 速查 | D1 | 核心命令（report/status/jobs/logs）+ 三维度分析框架 + 质量等级阈值速查表 |
| P1-2 | 添加数据充足性分支结构 | D1, D4 | 场景 A（数据充足）/ 场景 B（数据不足）使用明确的 if/else 逻辑 |
| P1-3 | 添加关键词覆盖 | D4 | 覆盖中文触发词："流水线老挂"、"CI 太慢"、"流水线健康检查"、"分析 CI 失败"、"flaky test"、"build 不稳定"、"测试超时"；覆盖英文触发词："pipeline health"、"CI analysis"、"flaky test"、"pipeline report"、"build failure" |
| P1-4 | 添加跨引用 | D4 | 引用 gitflow-precommit、gitflow-quality、gitflow-regression、gitflow-weekly-report、superpowers:systematic-debugging |
| P1-5 | 添加基线测试场景 | D3 | 定义不用 skill 时 Claude 的基线行为（可能仅查看当前状态，不做趋势分析和失败模式归类） |
| P1-6 | 补充成功标准 | D3 | 三维度分析清单、报告最低字段、质量等级阈值、数据不足时的降级输出 |

### P2（可选改进 — 锦上添花）

| # | 改进项 | 维度 | 说明 |
|---|--------|------|------|
| P2-1 | 添加压力测试场景 | D3 | 新分支无运行记录、大量 Pipeline 记录超时、API 返回字段缺失、多分支并行分析 |
| P2-2 | 提供英文版 description | D1 | Superpowers 主流语言为英文，description 改用英文可提高国际兼容性 |
| P2-3 | 添加 TDD for skills 验证记录 | D3, D4 | 记录 baseline → 编写 → 验证迭代的过程 |
| P2-4 | 添加 workflow 流程图 | D4 | 7 步骤 + 数据充足性分支用 Mermaid flowchart 简化阅读 |
| P2-5 | 添加多分支对比分析 | D1 | 支持同时分析多个分支（如 main vs release/*）的对比报告模式 |

---

## 七、验收标准

- [ ] description 以 "Use when..." 开头，仅描述触发条件（推荐英文）
- [ ] 含职责边界声明章节（含 🚫 禁止行为、✅/❌ 职责范围）
- [ ] 含红旗列表（要求自动修复流水线、重试失败 Pipeline、推送报告等）
- [ ] 含关键词覆盖（中英触发词、工具名、错误信息）
- [ ] 含跨引用（至少引用 3 个相关 skill）
- [ ] 文档结构包含 Overview / When to Use / Core Pattern / Quick Reference / Common Mistakes
- [ ] 快速加载时 token 数 < 500 词（详细报告模板移至 docs/templates/）
- [ ] 含数据充足性分支结构（数据充足 vs 数据不足）
- [ ] 含成功标准定义（三维度分析清单、报告最低字段、质量等级阈值）
- [ ] 含前置条件检查（Pipeline CLI 可用、有运行记录）
- [ ] 不修改任何 Pipeline 配置、不重试/取消 Pipeline、不自动创建 Issue/PR

---

## 八、与同类 Skill 对比

| 对比项 | gitflow-pipeline-analyzer | gitflow-precommit | gitflow-weekly-report |
|--------|---------------------------|-------------------|----------------------|
| 边界风险等级 | 🟢 低（只读分析） | 🟡 中（涉及文件写入） | 🟢 低（只读汇总） |
| 职责边界 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 |
| 测试场景 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 |
| 触发条件 | ❌ 功能描述 | ❌ 功能描述 | ❌ 功能描述 |
| 跨引用 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 |
| 分析框架 | ✅ 高（三维度结构化） | ⚠️ 中（线性步骤） | ⚠️ 中（汇总为主） |
| description 位置 | ❌ 混合功能+流程 | ❌ 混合功能+流程 | ❌ 混合功能+流程 |
| Token 数 | ⚠️ 高（813 词） | ⚠️ 高（885 词） | ⚠️ 中（~600） |
| 结构化程度 | ✅ 高（三维度+阈值+优先级） | ⚠️ 线性步骤完整 | ⚠️ 汇总为主 |

**关键发现：** gitflow-pipeline-analyzer 在分析框架的结构化程度上是所有 skill 中最高的——三维度分析（成功率趋势 + 失败模式 + 耗时分布）+ 质量等级阈值 + 改进建议优先级，形成了一个完整的分析闭环。但其结构形态也是"教程文档"与"执行指令"差距最大的——优秀的分析框架需要通过 Superpowers 标准结构才能被 Claude 有效消费。

---

## 九、总结

gitflow-pipeline-analyzer 当前的定位是"分析报告生成器 + 模板库"混合体——它有清晰的分析维度和高质量的输出模板，但缺乏 Superpowers skill 所需的结构性要素。

核心差距：
1. **缺乏触发条件** → Claude 何时应加载此 skill？（用户说"流水线老挂"？"CI 太慢"？"帮我看看流水线"？）
2. **缺乏职责边界** → 虽然为只读分析型，但"分析后自动采取行动"是 Claude 常见的过度执行模式
3. **缺乏可测试性** → 如何验证 Claude 在正确场景触发、在边界内执行、正确计算趋势？
4. **token 超标** → 813 词中包含大量报告模板和输出示例，应分离到项目模板库

重构方向：保留三维度分析框架和质量等级阈值（重构为 Quick Reference + 分支结构），将报告模板移至 `docs/templates/`，添加职责边界声明和红旗列表，重写 description 为触发条件，添加跨引用和 Success Criteria。重构后预期 token 从 813 词降至 ~300 词（不含模板文件），大幅提升加载效率。
