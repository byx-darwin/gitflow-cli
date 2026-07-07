# gitflow-auth Skill 分析报告

> **分析日期：** 2026-07-07
> **分析目标：** `skills/gitflow-auth/SKILL.md`
> **对应 Issue：** #15
> **分析维度：** 4 个维度（结构规范、职责边界、可测试性、Superpowers 最佳实践）

---

## 一、当前状态总览

| 维度 | 评分 | 说明 |
|------|------|------|
| 维度 1：Skill 结构和文档规范 | ⚠️ 需改进 | Frontmatter 基本合规，但 description 和内容结构不符合 Superpowers 规范 |
| 维度 2：职责边界清晰度 | ❌ 不合格 | 完全缺失职责边界声明 |
| 维度 3：可测试性 | ❌ 不合格 | 完全缺失测试场景和成功标准 |
| 维度 4：与 Superpowers 最佳实践的差距 | ❌ 不合格 | 未遵循 writing-skills 方法论 |

**总体评估：** gitflow-auth 当前是一个纯命令参考文档，而非符合 Superpowers 规范的 skill。它描述了"命令能做什么"，但没有描述"何时触发""如何执行""边界在哪""如何验证"。

---

## 二、维度 1 分析 — Skill 结构和文档规范

### 2.1 检查清单

| 检查项 | 状态 | 说明 |
|--------|------|------|
| YAML frontmatter 含 name 字段 | ✅ | `name: gitflow-auth` |
| YAML frontmatter 含 description 字段 | ✅ | 存在 description |
| description 以 "Use when..." 开头 | ❌ | 当前为 "gitflow-cli 的认证操作命令封装，支持登录、登出、状态查询和 Token 获取"——这是功能描述而非触发条件 |
| description 只描述触发条件 | ❌ | 描述了功能而非触发时机 |
| 含 Overview 章节 | ❌ | 无 Overview 章节（仅有"命令概览"表格） |
| 含 When to Use 章节 | ❌ | 无触发条件说明 |
| 含 Core Pattern 章节 | ❌ | 无核心模式 |
| 含 Quick Reference | ⚠️ | 有命令概览表格，但缺少参数与返回值的快速对照 |
| 含 Implementation 章节 | ❌ | 无实现步骤 |
| 含 Common Mistakes | ❌ | 无常见错误说明 |
| Token 效率 < 500 词 | ✅ | 约 280 词，简洁 |
| 无叙事性示例反模式 | ⚠️ | 示例仅展示 happy path，无失败场景 |
| 无多语言稀释 | ⚠️ | 使用中文注释但未提供英文对照 |
| 无流程图中嵌入代码 | ✅ | 无流程图 |

### 2.2 具体问题

1. **Frontmatter description 违反 Superpowers 规范**：Superpowers 要求 description 仅描述触发条件（"When to load this skill"），以便 Claude 决定是否加载。当前 description 是功能概述，会导致：
   - Claude 无法准确判断何时应加载此 skill
   - 增加误触发或漏触发的概率

2. **缺乏结构化章节**：当前文档是"命令参考手册"风格，不是"可执行指令"风格。缺少：
   - 前置条件（prerequisites）
   - 输入验证
   - 输出格式
   - 错误处理

3. **使用示例仅覆盖 happy path**：示例只展示成功场景，缺少：
   - Token 过期场景
   - 多平台切换场景
   - 无网络环境场景

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

1. **完全无职责边界**：对于 auth skill，职责边界尤其重要，因为它涉及敏感的凭据操作。没有边界声明可能导致：
   - Claude 在 token 过期时尝试自动重新认证（而非提示用户）
   - 在脚本中过度使用 `gitflow-cli auth token` 导致凭据泄露
   - 尝试读取或修改本地凭据存储文件

2. **缺少禁止行为清单**：auth 相关的禁止行为包括：
   - 不得将 token 写入日志或输出
   - 不得在不安全的环境中传递 token
   - 不得修改凭据存储文件格式

3. **缺少红旗信号**：应有的红旗包括：
   - 用户要求"获取 token 并发送到外部"
   - 尝试在 CI/CD 中存储明文 token
   - 试图绕过认证直接访问 API

### 3.3 评分：❌ 不合格

**对比参考（gitflow-autoreport-bug）：** 该 skill 是项目中唯一具备完整职责边界声明的文档（⚠️ 职责边界声明 章节含 🚫 禁止行为、✅ 职责范围、🔧 修复流程）。gitflow-auth 应参照此模式。

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
   - 如何判断 "Claude 正确处理了 token 过期错误"？
   - 如何判断 "Claude 没有在不应调用时调用 auth"？

2. **缺少基线对比**：应定义不使用 skill 时 Claude 的典型行为作为对照基线

3. **无压力测试场景**：应覆盖：
   - Token 过期 + 多平台切换
   - 凭据文件损坏
   - 网络不可达
   - 并发访问凭据

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
   - ❌ 当前：`gitflow-cli 的认证操作命令封装，支持登录、登出、状态查询和 Token 获取`
   - ✅ 应为：`Use when ... scenarios related to authentication state, login/logout flows, or token retrieval for API calls`

3. **缺少关键词覆盖**：应覆盖用户可能的表达方式：
   - "登录" / "login" / "sign in" / "认证"
   - "token 过期" / "unauthorized" / "401"
   - "当前用户" / "whoami" / "认证状态"
   - "登出" / "logout" / "sign out"

4. **缺少跨引用**：应引用相关 skills：
   - `gitflow-autoreport-bug`（auth cache 机制）
   - `gitflow-repo-onboarding`（首次设置认证）
   - `gitflow-security-check`（凭据安全检查）

### 5.3 评分：❌ 不合格

---

## 六、改进建议

### P0（必须修复 — 阻断性问题）

| # | 改进项 | 维度 | 说明 |
|---|--------|------|------|
| P0-1 | 重写 description 为触发条件 | D1, D4 | description 决定 Claude 何时加载 skill，必须改为 "Use when..." 格式 |
| P0-2 | 添加职责边界声明章节 | D2 | auth 涉及敏感凭据，必须有明确的 🚫 禁止行为和 ✅ 职责范围 |
| P0-3 | 添加关键词覆盖 | D4 | 覆盖常见错误信息（"未授权"、"401"、"token expired"）和同义词 |
| P0-4 | 添加跨引用 | D4 | 引用 autoreport-bug（auth cache）、security-check（凭据安全）等相关 skills |

### P1（建议修复 — 提升质量）

| # | 改进项 | 维度 | 说明 |
|---|--------|------|------|
| P1-1 | 重构为结构化模板 | D1 | 添加 Overview / When to Use / Core Pattern / Quick Reference / Implementation / Common Mistakes 章节 |
| P1-2 | 添加错误处理章节 | D1 | 覆盖 token 过期、网络不可达、凭据损坏等异常场景 |
| P1-3 | 添加前置条件检查 | D1 | 执行前验证 `gitflow-cli` 是否可用、是否在 git 仓库中 |
| P1-4 | 添加红旗列表 | D2 | 标识敏感场景（token 外传请求、CI 中存储凭据等） |

### P2（可选改进 — 锦上添花）

| # | 改进项 | 维度 | 说明 |
|---|--------|------|------|
| P2-1 | 添加基线测试场景 | D3 | 定义不用 skill 时 Claude 的基线行为 |
| P2-2 | 定义成功标准 | D3 | 每个子命令的预期输出和退出码 |
| P2-3 | 添加压力测试场景 | D3 | 多平台切换、并发访问、凭据损坏恢复等 |
| P2-4 | 提供英文版 description | D1 | 当前仅中文，可考虑 bilingual |

---

## 七、验收标准

- [ ] description 以 "Use when..." 开头，仅描述触发条件
- [ ] 含职责边界声明章节（含 🚫 禁止行为）
- [ ] 含关键词覆盖（错误信息、同义词、工具名）
- [ ] 含跨引用（至少引用 1 个相关 skill）
- [ ] 文档结构包含 Overview / When to Use / Quick Reference
- [ ] 含错误处理章节
- [ ] 新增/修改内容通过 `cargo` 之外的一致性检查（skill 不适用 Rust gate）

---

## 八、与同类 Skill 对比

| 对比项 | gitflow-auth | gitflow-autoreport-bug | 差距 |
|--------|-------------|----------------------|------|
| 职责边界 | ❌ 缺失 | ✅ 完整 | 差距大 |
| 测试场景 | ❌ 缺失 | ❌ 缺失 | — |
| 触发条件 | ❌ 功能描述 | ⚠️ 描述流程 | autoreport-bug 也需改进 |
| 跨引用 | ❌ 缺失 | ❌ 缺失 | — |
| 错误处理 | ❌ 缺失 | ✅ 有异常处理章节 | 差距大 |

---

## 九、总结

gitflow-auth 当前的定位是"命令参考手册"，它描述了命令的输入输出，但不具备 Superpowers skill 所需的**可执行性**、**边界清晰性**和**可测试性**。

核心差距：
1. **缺乏触发条件** → Claude 何时应加载此 skill？
2. **缺乏执行步骤** → Claude 应按什么顺序执行？
3. **缺乏错误处理** → 出错时 Claude 应如何反应？
4. **缺乏边界** → Claude 不应做什么？

重构方向：将其从"参考手册"转型为"可执行指令 + 边界声明 + 验证标准"的完整 skill。
