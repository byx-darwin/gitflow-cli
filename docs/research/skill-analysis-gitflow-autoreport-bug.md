# gitflow-autoreport-bug Skill 分析报告

> **分析日期：** 2026-07-07
> **分析目标：** `skills/gitflow-autoreport-bug/SKILL.md`
> **对应 Issue：** #40
> **分析维度：** 4 个维度（结构规范、职责边界、可测试性、Superpowers 最佳实践）

---

## 一、当前状态总览

| 维度 | 评分 | 说明 |
|------|------|------|
| 维度 1：Skill 结构和文档规范 | ⚠️ 需改进 | Frontmatter 基本合规，但 description 描述的是流程而非触发条件；文档涵盖了执行步骤和异常处理，但缺少 Overview / When to Use / Core Pattern / Common Mistakes 等 Superpowers 标准章节 |
| 维度 2：职责边界清晰度 | ⚠️ 需改进 | 已具备职责边界声明、禁止行为清单和职责范围（项目中唯一），但仍缺少"合理化借口"反制表格和红旗列表（Red Flags） |
| 维度 3：可测试性 | ❌ 不合格 | 无测试场景、无基线测试、无压力测试、无成功标准、无 writing-skills 钩子 |
| 维度 4：与 Superpowers 最佳实践的差距 | ⚠️ 需改进 | description 违反"只描述触发条件"规则；缺少关键词覆盖和跨引用；触发方式依赖 Stop Hook（非典型 skill 加载路径） |

**总体评估：** gitflow-autoreport-bug 是项目中**职责边界最完善**的 skill，但其 description 是**功能+流程描述**而非触发条件，导致 Claude 无法在正常对话中自主判断是否加载。它实际上是一个"自动化 pipeline 脚本的 Markdown 包装"，而非标准的"可触发 skill"。

---

## 二、维度 1 分析 — Skill 结构和文档规范

### 2.1 检查清单

| 检查项 | 状态 | 说明 |
|--------|------|------|
| YAML frontmatter 含 name 字段 | ✅ | `name: gitflow-autoreport-bug` |
| YAML frontmatter 含 description 字段 | ✅ | 存在 description |
| description 以 "Use when..." 开头 | ❌ | 当前为"自动分析 CLI 错误报告，auth cache 加速认证检查..." — 完整描述了功能流程 |
| description 只描述触发条件 | ❌ | 描述了完整 pipeline（检测 → 验证 → 去重 → 分析 → 创建 → 清理） |
| 含 Overview 章节 | ⚠️ | 有一段简短概述（一行），但不符合 Superpowers 的 Overview 格式 |
| 含 When to Use 章节 | ❌ | 无触发条件说明（仅在末尾"触发机制"章节提到 Stop Hook） |
| 含 Core Pattern 章节 | ❌ | 无核心模式 |
| 含 Quick Reference | ❌ | 无快速参考卡片（token/参数/返回值对照表） |
| 含 Implementation 章节 | ✅ | Step 1–6 实质上是实现步骤 |
| 含 Common Mistakes | ❌ | 无常见错误说明 |
| Token 效率 < 500 词 | ❌ | 约 680 词（含代码块），超出 Superpowers 推荐的 500 词上限 |
| 无叙事性示例反模式 | ⚠️ | 代码片段以 bash 内联 shell 逻辑呈现，不算叙事性示例，但有"过程式脚本"倾向 |
| 无多语言稀释 | ✅ | 全文中文，注释与描述语言一致 |
| 无流程图中嵌入代码 | ✅ | 无流程图 |

### 2.2 具体问题

1. **description 严重违规**：Superpowers 要求 description 只描述触发条件（"use when X happens"），当前 description 是一个完整流程摘要：
   - ❌ 当前：`自动分析 CLI 错误报告，auth cache 加速认证检查，去重检查后创建 GitHub/GitLab/GitCode Issue，失败记录保留到 failed.log 待重试。由 Stop Hook (hooks/auto-report-bug.sh) 自动触发。`
   - 问题：如果 Claude 扫描 description 来判断是否加载，这段文字会强烈暗示"当存在 pending.json 时触发"，但掩盖了真正的问题——这个 skill 实际上**不需要 Claude 通过 description 判断是否加载**，因为它由 Stop Hook 驱动。这引出了架构层面的问题：这个 skill 是否应该是 skill？还是应该只是一个 bash 脚本？

2. **文档结构是"流水线指令"而非"Skill 模板"**：
   - Step 1–6 按照时间顺序展开，符合脚本执行逻辑，但不符合 Superpowers 的"Context → Trigger → Action → Verify"心智模型
   - 缺少 When to Use（什么场景下触发）
   - 缺少 Core Pattern（可复用的核心命令模式）
   - 缺少 Common Mistakes（已知陷阱）

3. **Token 数超标**（约 680 词）：对于 Stop Hook 触发的 skill（非高频对话加载），这个体量可以接受，但如果要适配"Claude 自主加载"的路径，需要压缩到 < 500 词。

4. **代码块中的 bash 逻辑过于具体**：Step 2 的 auth cache 检查是一个完整的 shell if-then 块——如果 skill 的目标是让 Claude 执行，这段 shell 逻辑是合适的；如果目标是让 Claude 调用 `gitflow-cli` 命令完成，这段就是越界（把实现细节写在 skill 里）。

### 2.3 评分：⚠️ 需改进

---

## 三、维度 2 分析 — 职责边界清晰度

### 3.1 检查清单

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 有明确的职责边界声明 | ✅ | "本 skill 仅负责检测和报告 bug，绝对不会修复 bug" |
| 有禁止行为清单（🚫 不得...） | ✅ | 5 条禁止行为，具体且可执行 |
| 有职责范围说明（✅ 负责... / ❌ 不负责...） | ✅ | 列出 6 项负责 + 1 项不负责 |
| 有"合理化借口"反制表格 | ❌ | 缺失，这是 Superpowers 推荐的关键防御机制 |
| 有红旗列表（Red Flags） | ❌ | 缺失 |

### 3.2 具体问题

1. **禁止行为清单是亮点，但缺少"反制合理化借口"层**：
   - 当前 5 条 🚫 禁令是直接命令式（"不得修改代码"），这在 Claude 愿意遵守时有效
   - 但 Claude 在"想帮忙"时会产生合理化借口：
     - "只是看一眼源码不修改" → 仍属于越界
     - "用户肯定想让我顺便修复" → 未被禁令覆盖
     - "这是同一个 bug，顺手一起改了" → 利用模糊性
   - 应添加一张表格，列出常见借口及其反驳

2. **缺少红旗列表（Red Flags）**：
   - 应标识 Claude 正在越界的早期信号：
     - 🔴 Claude 开始读取 `src/` 目录下的文件
     - 🔴 Claude 提到"这个 bug 很容易修复"
     - 🔴 Claude 询问"要不要我一起修复？"
     - 🔴 Claude 将 issue 创建标记为"已完成"后继续操作
     - 🔴 Claude 在创建 issue 前运行 `git diff` 或 `git status`

3. **职责范围的"修复流程"部分过于温和**：
   - "如果需要修复 bug，必须由用户手动触发" — 这是正确的
   - 但后续 3 个条件中有模糊空间："用户明确指示'立即修复这个 bug'" — 这实际上已经不在本 skill 范围内，应说明此时应转交给 `gitflow-workflow` skill 而非继续本 skill

### 3.3 评分：⚠️ 需改进

**对比参考：** gitflow-autoreport-bug 是整个项目中职责边界最完善的 skill（比其他所有 skill 都好），但距离 Superpowers 最佳实践仍差"合理化借口反制"和"红旗列表"两层防御。

---

## 四、维度 3 分析 — 可测试性

### 4.1 检查清单

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 定义了测试场景 | ❌ | 无测试场景定义 |
| 有基线测试（无 skill 时的行为） | ❌ | 缺失 |
| 有压力测试场景 | ❌ | 缺失（将由 task-26 brief 的 Step 8 补充） |
| 有成功标准 | ❌ | 缺失 |
| 可使用 writing-skills 方法论进行测试 | ❌ | 无测试钩子 |

### 4.2 具体问题

1. **零测试覆盖**：整个 skill 没有任何"如何验证 Claude 正确执行了本 skill"的钩子。应定义：
   - **触发验证**：给定一个包含 `pending.json` 的环境，Claude 是否识别到应执行本 skill？
   - **执行验证**：Claude 是否按正确顺序执行了 Step 1–6？
   - **越界验证**：Claude 是否在 Step 5 之后停止（而非继续修改代码）？
   - **错误处理验证**：当 auth 失败时，Claude 是否保留了 pending.json 并写入了 failed.log？

2. **缺少基线对比**：应先不用 skill 测试 Claude 对 "发现 pending.json 文件" 的典型反应，记录差距。可能基线行为包括：
   - Claude 询问"这个文件是什么？"（无 skill 时最常见的反应）
   - Claude 尝试读取并解释内容（但不会创建 issue）
   - Claude 完全忽略（最坏情况）

3. **压力测试场景缺失**：应覆盖以下组合：
   - pending.json 字段缺失 + auth 缓存过期
   - auth 失败 + 网络不可达
   - 去重搜索返回大量候选 + 需要人工判断
   - 并发的多个 pending.json 文件
   - Stop Hook 触发但 skill 文件不存在

4. **"自动化"skill 的特殊可测试性问题**：本 skill 由 Stop Hook 触发，这意味着标准"Claude 自主加载 skill"的测试方法不适用。需要设计专门的 Hook-on/Hook-off 切换测试。

### 4.3 评分：❌ 不合格

---

## 五、维度 4 分析 — 与 Superpowers 最佳实践的差距

### 5.1 检查清单

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 遵循 TDD for skills（RED-GREEN-REFACTOR） | ❌ | 无 TDD 流程证据 |
| description 只描述触发条件，不描述流程 | ❌ | description 描述了完整 pipeline |
| 关键词覆盖（错误信息、症状、同义词、工具） | ❌ | 无关键词/触发词章节 |
| 跨引用其他 skills | ⚠️ | 提到了 `gitflow-workflow` 和 Stop Hook，但未使用 Superpowers 的 See Also 标准格式 |
| 必要时使用 flowchart | N/A | 6 步线性流程 + 多个分支（失败路径），flowchart 会有帮助但非必需 |

### 5.2 具体问题

1. **description 违反 Superpowers 核心约定**：
   - Superpowers 的 description 字段被 Claude 用来决定"何时加载此 skill"
   - 当前 description 是**功能摘要**而非**触发词列表**
   - ✅ 应改写为：`Use when a CLI command fails with an error, a pending bug report needs to be filed as a GitHub/GitLab/GitCode issue, error deduplication is needed, or after a Stop Hook detects a pending.json file.`
   - 注意：该 skill 的实际触发方式是 Stop Hook，这意味着 description 的"触发词"角色次要于 Hook 机制——但 Superpowers 推荐 description 仅描述触发条件

2. **缺少关键词覆盖章节**：
   - 本 skill 实际要覆盖的关键词：
     - "报错了" / "error" / "命令失败" / "non-zero exit"
     - "自动报告" / "auto-report" / "自动创建 issue"
     - "pending.json" / "bug report" / "错误日志"
     - "去重" / "deduplication" / "重复报告"
     - "issue 创建失败" / "failed to create"
   - 当前文档未显式列出这些触发词，Claude 需要从内容中推断关联

3. **跨引用形式化不足**：
   - 提到了 `gitflow-workflow --fast` 和 `hooks/auto-report-bug.sh`
   - 但没有标准化的 See Also 章节：
     ```
     ## See Also
     - gitflow-workflow — 用于修复 bug 的完整工作流（Phase 1–4）
     - gitflow-issue — issue 命令的参考文档
     - hooks/auto-report-bug.sh — Stop Hook 脚本
     ```

4. **缺少 TDD 证据**：
   - 没有表明 skill 经过"baseline → write skill → verify → refine"循环
   - 没有 baseline 数据（不用 skill 时 Claude 的行为记录）

5. **触发方式的特殊性**：
   - 由 Stop Hook（`hooks/auto-report-bug.sh`）驱动，这意味着：
     - skill 通常是 Claude 停止后由 Hook 触发再启动 Claude
     - skill description 对"是否加载"的影响较小（Hook 加载了它）
     - 但当用户**显式**提到"报告这个 bug"时，Claude 仍需通过 description 判断是否加载
   - 建议：无论触发路径如何，description 都应符合 Superpowers 规范

### 5.3 评分：⚠️ 需改进

---

## 六、改进建议

### P0（必须修复 — 阻断性问题）

| # | 改进项 | 维度 | 说明 |
|---|--------|------|------|
| P0-1 | 重写 description 为触发条件 | D1, D4 | description 必须以 "Use when..." 开头，只描述触发场景，不包含流程步骤或实现细节 |
| P0-2 | 添加红旗列表（Red Flags） | D2 | 标识 Claude 越界的早期信号（如开始读取 src/文件、提议"顺手修复"等） |
| P0-3 | 添加"合理化借口"反制表格 | D2 | 覆盖 Claude 常见的越界借口（"只是看看""用户肯定想让我修""顺手的事"） |
| P0-4 | 添加可测试性钩子（成功标准 + 基线对比） | D3 | 定义至少 1 个基线测试和 3 个执行验证场景 |

### P1（建议修复 — 提升质量）

| # | 改进项 | 维度 | 说明 |
|---|--------|------|------|
| P1-1 | 添加标准化文档结构 | D1 | 补充 Overview / When to Use / Core Pattern / Quick Reference / Common Mistakes 章节 |
| P1-2 | 压缩 Token 到 < 500 词 | D1 | Step 2 的 bash 内联 shell 可用单行命令 + 注释替代，节省约 150 词 |
| P1-3 | 添加 See Also 跨引用 | D4 | 引用 gitflow-workflow、gitflow-issue、gitflow-auth 等相关 skills |
| P1-4 | 添加关键词覆盖章节 | D4 | 显式列出触发同义词（"报错了""自动报告""pending.json"等） |
| P1-5 | 修复"修复流程"章节的模糊性 | D2 | 明确"立即修复这个 bug"是转交给 gitflow-workflow skill 的信号，不是本 skill 的延续 |

### P2（可选改进 — 锦上添花）

| # | 改进项 | 维度 | 说明 |
|---|--------|------|------|
| P2-1 | 添加压力测试场景（3+ 压力组合） | D3 | 时间压力 + 简化诱惑 + 疲劳压力；权威压力 + 沉没成本；信息噪声 + 紧急中断 |
| P2-2 | 添加成功标准清单 | D3 | 每个 Step 的退出验证（Step 1 输出 JSON 字段列表、Step 5 输出 Issue URL 等） |
| P2-3 | 修复机制说明 | D4 | 如果 Claude 越界（如开始修改代码），如何检测并回退（可以红旗列表 + 用户提示组合） |
| P2-4 | 考虑分离"Hook 文档"与"Skill 文档" | D1 | 将 Stop Hook 触发机制（`hooks/auto-report-bug.sh`）的说明移到独立文档，skill 只聚焦"做什么+怎么做" |

---

## 七、验收标准

- [ ] description 以 "Use when..." 开头，仅描述触发条件（不含流程描述）
- [ ] 含红旗列表（至少 5 条 🔴 red flags）
- [ ] 含"合理化借口"反制表格（至少 5 条常见借口 + 反驳）
- [ ] 含基线测试场景（无 skill 时 Claude 的预期行为）
- [ ] 含成功标准清单（每个 Step 的完成判定）
- [ ] 含 See Also 跨引用（至少引用 2 个相关 skill）
- [ ] 含关键词覆盖章节（中英文触发词）
- [ ] Token 数 ≤ 500 词（压缩 bash 内联 shell 后）
- [ ] 文档结构含 Overview / When to Use / Core Pattern / Quick Reference / Common Mistakes

---

## 八、与同类 Skill 对比

| 对比项 | gitflow-autoreport-bug | gitflow-auth | gitflow-issue-create | 差距说明 |
|--------|----------------------|-------------|---------------------|---------|
| 职责边界 | ⚠️ 最完善（有声明+禁止+范围） | ❌ 缺失 | ❌ 缺失 | autoreport-bug 是项目标杆 |
| description 规范 | ❌ 流程描述 | ❌ 功能描述 | ❌ 功能描述 | 全部不合格 |
| 可测试性 | ❌ 无 | ❌ 无 | ❌ 无 | 项目通病 |
| 跨引用 | ⚠️ 隐式提及 | ❌ 缺失 | ❌ 缺失 | 极弱 |
| 触发机制 | Stop Hook（特殊） | 用户命令 | 用户命令 | autoreport-bug 独特 |

---

## 九、总结

gitflow-autoreport-bug 在**职责边界**上是项目中的最佳实践标杆——它有明确的禁止行为和职责范围，这在整个项目中独一无二。但它仍然不符合 Superpowers 的完整规范：

1. **description 是流程描述而非触发条件** — 这是最紧迫的问题，因为它直接决定了 Claude 能否正确识别何时应使用此 skill
2. **缺少合理化借口反制** — 对边界声明是必要的补充，Claude 在"想帮忙"时会产生合规但危险的借口
3. **完全缺失可测试性层** — 没有任何验证钩子，无法证明 skill 有效或无效
4. **特殊触发路径（Stop Hook）** 带来额外的可测试性挑战

重构的优先级建议：**先修复 description → 添加红旗清单和借口反制 → 添加测试场景 → 优化文档结构**。这样可以用最小改动覆盖最多风险。

---

## 十、备注

本分析报告遵循 task-26 brief 的 4 维度框架，产出独立 Markdown 文件，
对应 GitHub Issue #40（标题：`refactor(skill): gitflow-autoreport-bug — 符合 Superpowers 最佳实践`）。
