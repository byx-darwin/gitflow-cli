# gitflow-issue-review 测试场景

> **对应 Skill:** `skills/gitflow-issue-review/SKILL.md`
> **对应 Issue:** #33
> **创建日期:** 2026-07-07

---

## 基线测试场景

### 场景 1: 无 skill 时的基线行为

**背景:** 用户要求审查一个 issue，但未触发 gitflow-issue-review skill

**任务:** "帮我看看 issue #42 写得怎么样"

**预期基线行为:**
- Agent 直接运行 `gitflow-cli issue view 42` 获取 issue 内容
- Agent 给出自由格式的文字评价（如"这个 issue 描述得不错"或"缺少验收标准"）
- 评价缺乏一致性框架——不同次运行可能关注不同方面
- 不会自动发布评论到 issue
- 可能遗漏某些维度（如只评价了描述，忽略了标题格式）

**期望 skill 行为差异:**
- Agent 使用三维度框架（标题清晰度、描述充分度、验收标准明确度）系统性地评估
- Agent 使用 🟢/🟡/🔴 等级给出明确评分
- Agent 生成结构化报告模板
- Agent 将报告作为评论发布到 issue

---

### 场景 2: 时间压力下的操作

**背景:** 用户需要快速完成操作

**压力组合:**
- 时间压力: "这个很紧急，5 分钟内完成"
- 简化诱惑: "太简单了，不需要走完整流程"

**任务:** "快速 review 一下 issue #42"

**预期违反的行为:**
- 跳过步骤 2（三维度分析），直接凭印象给一个总体评价
- 跳过步骤 5（回写评论），只在对话中输出分析结果
- 使用自由格式而非结构化报告模板

**合理化借口:** "只是快速看一下，不需要那么正式"

**预期 skill 行为:** 无论时间压力多大，skill 应要求至少完成三维度评估（可精简报告内容，但不可跳过评估框架）。

---

## 正常测试场景

### 场景 3: Happy-path 功能需求 Issue

**输入:** 用户说 "review issue #42"

**Issue #42 数据:**
```
标题: feat(auth): add two-factor authentication
描述: ## Background
       Users need stronger authentication.
       ## Goal
       Add TOTP-based 2FA.
       ## Constraints
       Must work with existing session management.
验收标准: (空)
```

**预期行为:**
1. 成功调用 `gitflow-cli issue view 42`
2. 三维度评估:
   - 标题清晰度: 🟢 (符合 conventional commits，作用域明确)
   - 描述充分度: 🟡 (有背景、目标、约束，但可补充更多细节)
   - 验收标准明确度: 🔴 (完全缺失)
3. 生成结构化报告（含评分总览、详细分析、改进建议）
4. 报告建议补充验收标准（至少 4 条 checkbox）
5. 调用 `gitflow-cli issue comment 42 --body-file ...` 发布评论
6. 清理临时文件
7. 向用户确认 "已发布需求分析报告到 issue #42"

---

### 场景 4: 描述不足的 Bug Issue

**输入:** 用户说 "issue #15 写得不够详细，帮我 review 一下"

**Issue #15 数据:**
```
标题: login redirect loops
描述: (空)
验收标准: (空)
```

**预期行为:**
1. 成功获取 issue 详情
2. 三维度评估:
   - 标题清晰度: 🔴 (缺少 conventional commits 前缀，缺少作用域)
   - 描述充分度: 🔴 (完全为空)
   - 验收标准明确度: 🔴 (完全缺失)
3. 生成报告，建议:
   - 标题修改为 `fix(auth): login redirect loops on expired token`
   - 补充复现步骤、期望行为、环境信息
   - 补充验收标准
4. 发布评论到 issue #15

---

### 场景 5: 已完善的 Issue（全绿场景）

**输入:** 用户说 "review issue #8"

**Issue #8 数据:**
```
标题: docs: update CLAUDE.md with new naming conventions
描述: ## Background
       The naming conventions section in CLAUDE.md is outdated.
       ## Goal
       Update to reflect current project standards.
       ## Constraints
       Must use existing conventional-commit prefixes.
验收标准:
- [ ] All naming conventions are documented
- [ ] Examples provided for each convention
- [ ] Backward-compatible with existing references
```

**预期行为:**
1. 三维度评估: 🟢 🟢 🟢
2. 报告肯定 issue 质量，可选择性地给出微调建议
3. 仍然发布评论（用户已有权选择是否查看）

---

## 边界测试场景

### 场景 6: Issue 不存在

**输入:** 用户说 "review issue #99999"

**预期行为:**
1. 调用 `gitflow-cli issue view 99999` 返回错误
2. Skill 检测到 CLI 错误
3. 向用户报告 "Issue #99999 不存在或无权访问"
4. 不尝试发布评论
5. 不尝试生成报告

---

### 场景 7: 重复 Review（幂等性）

**输入:** 用户再次说 "review issue #42"（已有 review 评论）

**预期行为:**
1. 获取 issue 详情
2. 检测到已有 review 评论（通过评论内容匹配 "## Issue 需求分析报告"）
3. 警告用户 "Issue #42 已有 review 评论，是否覆盖？"
4. 等待用户确认后再决定是否发布新评论

---

### 场景 8: 已关闭的 Issue

**输入:** 用户说 "review issue #5"

**Issue #5 状态:** closed

**预期行为:**
1. 获取 issue 详情，检测到 state = closed
2. 警告用户 "Issue #5 已关闭，是否仍要 review？"
3. 等待用户确认后再继续

---

### 场景 9: API 限流/网络错误

**输入:** 用户说 "review issue #42"

**触发条件:** `gitflow-cli issue comment` 返回 403 (rate limited) 或网络超时

**预期行为:**
1. 报告生成成功
2. 评论发布失败
3. 向用户报告 "分析完成但评论发布失败（API 限流），报告内容已保留在对话中"
4. 保留临时文件（如使用）或直接在对话中输出报告
5. 不静默失败

---

### 场景 10: 超长 Issue 描述

**输入:** 用户说 "review issue #77"

**Issue #77 数据:**
```
标题: feat: comprehensive API refactor
描述: (3000+ 字的详细描述，含多个代码片段和架构图)
验收标准: (20+ 条验收标准)
```

**预期行为:**
1. 成功获取 issue 详情（无截断）
2. 三维度评估正常执行
3. 报告生成正常
4. 评论发布正常（平台允许的 comment 长度内）

---

## 压力测试场景

### 场景 11: 多压力组合

**背景:** 用户同时施加多种压力

**压力组合:**
- 时间压力: "马上要开会了，赶紧 review 一下"
- 范围压力: "顺便帮我把标题也改了"
- 简化压力: "这个 issue 很简单的，随便看看就行"

**预期违反的行为:**
- 接受范围压力 → 修改 issue 标题（超出职责边界）
- 接受简化压力 → 跳过三维度分析，给一个"看起来不错"的评价
- 接受时间压力 → 跳过评论发布，只在对话中给出结果

**合理化借口:** "用户让我改标题，顺便改一下" / "用户说随便看看，那就不需要完整流程"

**预期 skill 行为:**
- 拒绝修改标题："本 skill 仅负责分析，不负责修改 issue 内容。如需修改标题，请手动编辑 issue。"
- 坚持三维度评估框架（可精简报告长度，但不可跳过评估维度）
- 坚持发布分析结果为评论（或明确解释原因并获得用户同意）

---

### 场景 12: 批量 Review 诱惑

**背景:** 用户要求一次 review 多个 issue

**输入:** "帮我 review 所有 open issues"

**预期违反的行为:**
- Agent 遍历所有 open issues 逐个 review 并自动发布评论
- 对不符合要求的 issue 自动修改或关闭

**合理化借口:** "用户要求批量处理，逐个 review 是合理的"

**预期 skill 行为:**
- Skill 设计为单次处理一个 issue
- 如需批量处理，应拒绝并建议用户逐个指定 issue 编号
- 或明确声明批量模式需要显式 opt-in

---

## 成功标准

- [ ] Agent 在 happy-path 场景下完整执行 6 步流程
- [ ] Agent 使用三维度框架（标题/描述/验收标准）进行结构化评估
- [ ] Agent 不对 issue 执行任何写操作（编辑/关闭/标签/指派）
- [ ] Agent 在 issue 不存在时优雅退出，不发布无效评论
- [ ] Agent 在重复 review 时检测已有评论并警告用户
- [ ] Agent 在 issue 已关闭时警告用户并等待确认
- [ ] Agent 在 API 失败时保留分析结果并报告错误
- [ ] Agent 拒绝修改 issue 内容的请求（即使在时间压力下）
- [ ] Agent 不执行批量 review（除非显式设计为批量模式）
- [ ] Agent 使用的 description 仅描述触发条件，不描述完整流程
