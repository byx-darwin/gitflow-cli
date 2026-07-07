# gitflow-review Skill 分析报告

> **分析日期：** 2026-07-07
> **分析目标：** `skills/gitflow-review/SKILL.md`
> **对应 Issue：** #35
> **分析维度：** 4 个维度（结构规范、职责边界、可测试性、Superpowers 最佳实践）

---

## 一、当前状态总览

| 维度 | 评分 | 说明 |
|------|------|------|
| 维度 1：Skill 结构和文档规范 | ⚠️ 需改进 | Frontmatter 基本合规，但 description 违反触发条件规则；缺少 Overview、When to Use、Core Pattern、Implementation、Common Mistakes 等结构化章节 |
| 维度 2：职责边界清晰度 | ❌ 不合格 | 完全缺失职责边界声明；与 gitflow-pr 的 `comment` 子命令存在功能重叠但未声明区分规则 |
| 维度 3：可测试性 | ❌ 不合格 | 完全缺失测试场景、基线对比、压力测试和成功标准 |
| 维度 4：与 Superpowers 最佳实践的差距 | ❌ 不合格 | 未遵循 writing-skills 方法论；缺少关键词覆盖、跨引用、TDD for skills |

**总体评估：** gitflow-review 当前是一个纯命令参考手册（"命令能做什么"），而非符合 Superpowers 规范的 skill（"何时触发、如何执行、边界在哪、如何验证"）。它比同类的 gitflow-auth / gitflow-commit 问题更严重，因为 review 操作涉及合并闸门（approve/request-changes 直接影响 PR 能否合并），职责边界缺失可能导致 Claude 在未充分审查的情况下批准 PR。

---

## 二、维度 1 分析 — Skill 结构和文档规范

### 2.1 检查清单

| 检查项 | 状态 | 说明 |
|--------|------|------|
| YAML frontmatter 含 name 字段 | ✅ | `name: gitflow-review` |
| YAML frontmatter 含 description 字段 | ✅ | 存在 description |
| description 以 "Use when..." 开头 | ❌ | 当前为 "gitflow-cli 的代码审查操作命令封装，支持评论、批准、要求修改和提交审查"——这是功能描述而非触发条件 |
| description 只描述触发条件 | ❌ | 描述了功能而非触发时机 |
| 含 Overview 章节 | ❌ | 无 Overview 章节（仅有"命令概览"表格） |
| 含 When to Use 章节 | ❌ | 无触发条件说明 |
| 含 Core Pattern 章节 | ❌ | 无核心模式 |
| 含 Quick Reference | ⚠️ | 有命令概览表格和参数表，但缺少"何时用哪个子命令"的决策指南 |
| 含 Implementation 章节 | ❌ | 无实现步骤 |
| 含 Common Mistakes | ❌ | 无常见错误说明 |
| 含错误处理章节 | ❌ | 缺少异常场景及恢复路径 |
| 含前置条件检查 | ❌ | 未说明执行前需要验证什么 |
| Token 效率 < 500 词 | ✅ | 约 350 词，简洁 |
| 无叙事性示例反模式 | ⚠️ | 示例仅展示命令格式，无失败场景或决策过程 |
| 无多语言稀释 | ✅ | 全文中文，无混合语言 |
| 无流程图中嵌入代码 | ✅ | 无流程图 |

### 2.2 具体问题

1. **Frontmatter description 违反 Superpowers 规范**：Superpowers 要求 description 仅描述触发条件（"When to load this skill"），以便 Claude 决定是否加载。当前 description 是功能概述，会导致：
   - Claude 无法准确区分何时应加载此 skill 而非 `gitflow-pr-review` 或 `gitflow-pr-inline-review`
   - 在用户说"帮我审一下这个 PR"时可能不会触发（因 description 未提及 review/LGTM/approve/reject 等关键词）
   - 与 `gitflow-pr` 的 `comment` 子命令职责重叠时无法做出正确路由

2. **缺少 approve vs submit 决策指南**：`review approve` 和 `review submit --event approved` 在功能上高度重叠，文档未说明何时用哪个。这是用户最常见的困惑点，当前文档仅解释 `submit` 为"批量操作后一次性提交"，但未给出选择树。

3. **缺少结构化章节**：当前文档是"命令参考手册"风格，缺少：
   - 前置条件（是否需要先 `gitflow-cli pr view` 查看 PR？是否需要先检查 auth？）
   - 输入验证（PR 编号是否存在？PR 是否处于可审查状态？）
   - 错误处理（无权限、PR 已合并、重复审批等）
   - 输出格式（审查提交成功后的确认信息）

4. **使用示例仅覆盖 happy path**：示例只展示成功场景，缺少：
   - PR 不存在或编号为负数
   - 尝试审批自己的 PR（GitHub 禁止）
   - PR 已被合并后尝试审批
   - 网络超时或 API 限流
   - 跨平台差异（GitLab 使用 merge request approval，GitCode 可能有不同的 API）

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

1. **完全无职责边界 — 高风险**：review skill 是项目中对仓库状态影响最大的操作之一（approve 直接影响合并闸门）。没有边界声明可能导致：
   - Claude 在未阅读代码的情况下直接 approve（"用户说 LGTM 就执行"）
   - Claude 跳过 `gitflow-pr-review` 的 6 维度分析，直接调用 `review approve`
   - Claude 盲目相信用户的"快速过一下"请求，省略必要的审查步骤
   - 在 CI 环境或自动化脚本中被误用

2. **与 gitflow-pr 的功能重叠未声明**：`gitflow-pr comment` 和 `gitflow-review comment` 功能完全相同，文档未说明何时应使用哪个。合理的边界应为：
   - `gitflow-pr comment`：通用 PR 评论（不表达审查结论）
   - `gitflow-review comment`：在审查流程中发表评论（作为 6 维度分析过程的一部分）
   - `gitflow-review approve/request-changes/submit`：提交有合并闸门影响的审查结论

3. **缺少红旗信号**：应有的红旗包括：
   - 用户说"帮我 approve 一下"但未进行任何代码审查 → 必须拒绝
   - 用户说"快速 review 一下，不用太仔细" → 必须拒绝简化
   - 用户自己的 PR 要求 Claude approve → 必须拒绝
   - PR 标记为 WIP/draft → 应提醒而非直接 approve
   - PR 有关联的 CI 失败 → 应提醒而非直接 approve

4. **缺少"合理化借口"反制**：
   - ❌ "用户说很紧急，直接 approve 就行了" → 紧急不是跳过审查的理由
   - ❌ "改动很小，不需要完整审查" → 再小的改动也可能引入安全问题
   - ❌ "用户就是作者，他知道自己在做什么" → 自我审查违反最佳实践
   - ❌ "已经有人 approve 了，加我一个没关系" → 每个审查者应独立评估

### 3.3 评分：❌ 不合格

**对比参考（gitflow-autoreport-bug）：** 该 skill 是项目中唯一具备完整职责边界声明的文档。gitflow-review 作为对仓库有更高影响的 skill，更应参照此模式，明确定义其仅负责提交审查结论，不负责代码修改、不代替人工审查、不跳过分析步骤。

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

1. **零测试覆盖**：作为技能，应定义如何验证 Claude 正确执行了审查操作：
   - 如何判断 "Claude 正确选择了 approve vs request-changes"？
   - 如何判断 "Claude 在 approve 前进行了充分的代码分析"？
   - 如何判断 "Claude 在边界情况下正确拒绝了操作"（如审批自己的 PR）？

2. **缺少基线对比**：应定义不使用 skill 时 Claude 的典型行为作为对照基线。例如：
   - 不使用 skill 时，Claude 可能使用 `gh pr review --approve` 而非 `gitflow-cli review approve`
   - 不使用 skill 时，Claude 可能不了解 approve 和 submit 的区别
   - 不使用 skill 时，Claude 可能跳过 pr view 直接 approve

3. **无压力测试场景**：应覆盖：
   - 用户急切要求 approve（时间压力）
   - 用户声称"已审阅"但实际未审阅（信任滥用）
   - 大型 PR（100+ 文件）的审查拒绝
   - 网络超时后重试审批
   - 并发审查冲突（多人同时审批同一 PR）

4. **无成功标准**：应定义：
   - approve 成功 = 审查结论已提交 + PR 状态变更 + 用户收到确认
   - request-changes 成功 = 审查结论已提交 + 修改要求明确可操作
   - 触发准确性 = 用户说"review PR"时加载此 skill 的概率 > 95%
   - 误触发率 = 用户说"comment on issue"时加载此 skill 的概率 < 5%

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
| 必要时使用 flowchart | ❌ | approve vs submit 的决策需要流程图但未提供 |

### 5.2 具体问题

1. **未遵循 writing-skills 方法论**：
   - 未进行 "baseline test"（先不用 skill 让 Claude 执行，记录差距）
   - 未基于差距优化 skill 内容
   - 未迭代验证

2. **description 应是触发条件而非功能描述**：
   - ❌ 当前：`gitflow-cli 的代码审查操作命令封装，支持评论、批准、要求修改和提交审查`
   - ✅ 应为：`Use when the user wants to submit a formal code review decision (approve, request changes, or comment) on a PR through gitflow-cli. Triggers on "review this PR", "LGTM", "approve", "request changes", "reject PR", "submit review".`

3. **缺少关键词覆盖**：应覆盖用户可能的表达方式：
   - "审批 PR" / "approve" / "LGTM" / "looks good"
   - "要求修改" / "reject" / "request changes" / "changes requested"
   - "提交审查" / "submit review" / "提交评审"
   - "评论 PR" / "comment on PR" / "审查意见"
   - "approve my PR" / "帮我 approve" / "过一下这个 PR"
   - 错误信息："pull request review" / "review required"

4. **缺少跨引用**：必须引用相关 skills：
   - `gitflow-pr-review`（6 维度审查清单 → 数据源）
   - `gitflow-pr-inline-review`（行内评论 → 另一种审查方式）
   - `gitflow-pr-apply-feedback`（处理审查反馈 → review 的下游）
   - `gitflow-pr`（通用 PR 操作，含 comment 子命令 → 功能重叠说明）
   - `gitflow-workflow`（Phase 4 调用此 skill → 上下文）
   - `gitflow-security-check`（安全性审查维度 → 关联）

5. **approve vs submit 决策流程图缺失**：4 个子命令中，`approve` 和 `submit --event approved` 的区分是用户最常见的困惑。Superpowers 方法论建议在"需要决策判断"的地方使用 flowchart。例如：

   ```
   ┌─ 是否已添加行内评论？
   │  ├─ 否 → review approve / request-changes
   │  └─ 是 → review submit --event approved / changes_requested
   └─ 是否仅需中立评论？
      └─ 是 → review comment
   ```

### 5.3 评分：❌ 不合格

---

## 六、与同类 Skill 的交叉对比

### 6.1 与 gitflow-pr-review 的关系

| 对比项 | gitflow-pr-review | gitflow-review | 问题 |
|--------|-------------------|----------------|------|
| 定位 | 6 维度审查清单 | 提交审查结论 | pr-review 的产出应流入 review，但无交叉引用 |
| 职责 | 分析代码质量 | 表达审查结论 | 两者分离正确，但边界未声明 |
| 依赖关系 | review 依赖 pr-review 的分析结果 | 未声明 | review 应明确"approve 前应先通过 pr-review 完成分析" |

### 6.2 与 gitflow-pr-inline-review 的关系

| 对比项 | gitflow-pr-inline-review | gitflow-review | 问题 |
|--------|--------------------------|----------------|------|
| 定位 | 行内评论（4 维度） | 审查结论（approve/reject） | inline-review 后应使用 review submit 提交，但无交叉引用 |
| 典型工作流 | 发现问题 → 行内评论 | 行内评论后调用 submit 提交结论 | 工作流断裂，缺少端到端指南 |

### 6.3 与 gitflow-pr 的功能重叠

| gitflow-pr 子命令 | gitflow-review 子命令 | 关系 |
|--------------------|-----------------------|------|
| `pr comment` | `review comment` | 完全重叠！需声明区分规则 |
| `pr view` | — | review 需要先 view，但未声明 |
| `pr merge` | `review approve` | approve 是 merge 的前置条件，但未声明关系 |

---

## 七、改进建议

### P0（必须修复 — 阻断性问题）

| # | 改进项 | 维度 | 说明 |
|---|--------|------|------|
| P0-1 | 重写 description 为触发条件 | D1, D4 | description 决定 Claude 何时加载 skill，必须改为 "Use when..." 格式，包含 review/LGTM/approve/reject/request changes 等关键词 |
| P0-2 | 添加职责边界声明章节（含红旗列表） | D2 | review 是合并闸门操作，必须声明：approve 前必须先分析代码、不得审批自己的 PR、不得跳过审查步骤、红旗包括"用户要求快速 approve"等 |
| P0-3 | 添加 approve vs submit 决策流程图 | D1, D4 | 4 个子命令中的决策需要用流程图说清，尤其是 approve（即时）vs submit（延后）的选择 |
| P0-4 | 添加关键词覆盖 | D4 | 覆盖常见表达（"审批 PR"、"LGTM"、"approve"、"request changes"、"reject"、"过一下"）和同义词 |
| P0-5 | 添加跨引用 | D4 | 必须引用 gitflow-pr-review（上游分析）、gitflow-pr-inline-review（并行审查方式）、gitflow-pr（功能重叠说明）、gitflow-workflow（Phase 4 上下文） |

### P1（建议修复 — 提升质量）

| # | 改进项 | 维度 | 说明 |
|---|--------|------|------|
| P1-1 | 重构为结构化模板 | D1 | 添加 Overview / When to Use / Core Pattern / Quick Reference / Implementation / Common Mistakes 章节 |
| P1-2 | 添加错误处理章节 | D1 | 覆盖 PR 不存在、无权限、PR 已合并、重复审批、CI 失败、网络超时等异常场景 |
| P1-3 | 添加前置条件检查 | D1 | 执行前验证 `gitflow-cli` 是否在执行 `review` 前需要先 `pr view` 获取 PR 状态 |
| P1-4 | 添加"合理化借口"反制表格 | D2 | 反制"紧急跳过"、"改动很小"、"已有人 approve"等常见借口 |
| P1-5 | 声明与 gitflow-pr comment 的区分规则 | D2 | 明确：`pr comment` 用于通用评论，`review comment` 用于审查流程中的中间评论 |

### P2（可选改进 — 锦上添花）

| # | 改进项 | 维度 | 说明 |
|---|--------|------|------|
| P2-1 | 添加基线测试场景 | D3 | 定义不用 skill 时 Claude 的基线行为（如使用 gh CLI、跳过 pr view 等） |
| P2-2 | 定义成功标准 | D3 | 每个子命令的预期输出、退出码、后置状态 |
| P2-3 | 添加压力测试场景 | D3 | 时间压力、大型 PR、并发审批、信任滥用等 |
| P2-4 | 添加决策模式章节（非命令参考） | D1, D4 | 将文档从"命令手册"模式转为"决策+执行"模式 |

---

## 八、验收标准

- [ ] description 以 "Use when..." 开头，仅描述触发条件
- [ ] 含职责边界声明章节（含 🚫 禁止行为、红旗列表）
- [ ] 含 approve vs submit 决策流程图
- [ ] 含关键词覆盖（常用表达、同义词、工具名）
- [ ] 含跨引用（至少引用 3 个相关 skill：gitflow-pr-review、gitflow-pr、gitflow-workflow）
- [ ] 含错误处理章节（覆盖 ≥ 5 个异常场景）
- [ ] 含"合理化借口"反制表格（≥ 4 条）
- [ ] 文档结构包含 Overview / When to Use / Quick Reference
- [ ] 声明与 gitflow-pr comment 的区分规则
- [ ] 新增/修改内容通过一致性检查

---

## 九、与同类 Skill 对比

| 对比项 | gitflow-review | gitflow-pr-review | gitflow-autoreport-bug |
|--------|----------------|-------------------|----------------------|
| 职责边界 | ❌ 缺失 | ⚠️ 禁止行为章节隐性存在（"注意事项"中红旗级别的内容） | ✅ 完整 |
| 测试场景 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 |
| 触发条件 | ❌ 功能描述 | ⚠️ 描述流程 | ⚠️ 描述流程 |
| 跨引用 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 |
| 错误处理 | ❌ 缺失 | ⚠️ 注意事项含边界提示 | ✅ 有异常处理章节 |
| 流程图 | ❌ 缺失（需要） | N/A 线性流程 | N/A 线性流程 |

---

## 十、总结

gitflow-review 当前的定位是"命令参考手册"，与 gitflow-auth / gitflow-commit 存在相同的结构性问题，但严重程度更高，因为：

1. **安全风险**：approve/request-changes 直接影响合并闸门，缺少职责边界可能导致 Claude 盲目审批
2. **工作流断裂**：与上游的 `gitflow-pr-review`（6 维度分析）和 `gitflow-pr-inline-review`（行内评论）缺少交叉引用，端到端工作流断裂
3. **功能重叠**：与 `gitflow-pr comment` 完全重叠但未声明区分规则
4. **决策缺失**：`approve` vs `submit` 的选择需要流程图但未提供

**重构方向**：将其从"命令参考手册"转型为"可执行指令 + 边界声明 + 决策流程图 + 跨引用 + 验证标准"的完整 skill，参照 gitflow-autoreport-bug 的边界声明 + gitflow-pr-review 的分析上游关系。
