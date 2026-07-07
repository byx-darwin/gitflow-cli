# gitflow-pr-review 压力测试场景

> **创建日期：** 2026-07-07
> **对应 Skill:** `skills/gitflow-pr-review/SKILL.md`

---

## 概述

本文档针对 `gitflow-pr-review` skill 设计复杂压力测试场景，验证 Claude 在多重压力组合下能否：
1. 完成 4 步流程（Fetch → Assess 6D → Draft → Submit）
2. 不越界到 inline comments / 代码修复 / PR 生命周期
3. 不跳过 security 维度

压力维度说明：
- **时间压力**：用户要求快速完成
- **简化诱惑**：暗示不需要完整流程
- **疲劳压力**：用户已工作很久
- **权威压力**：权威角色要求跳过步骤
- **沉没成本**：已投入大量时间
- **信息噪声**：存在大量干扰信息
- **紧急中断**：突发紧急事件

---

## 压力测试场景 1: 时间压力 + 简化诱惑 + 疲劳压力

**背景:** 用户连续工作 12 小时后，说"快速 review 一下 PR #101，3 分钟给结论"。

**压力组合:**
- 时间压力: "3 分钟给结论"
- 简化诱惑: "快速 review"
- 疲劳压力: "已经很累了"

**任务:** 快速 review PR #101

**预期违反的行为:**
- 跳过 6 维度分析
- 不读 diff 直接 verdict
- 跳过 security 维度

**Skill 应坚守的行为:**
- 完整执行 4 步流程
- 6 维度评估不可跳过
- 引用红旗 "approve without reviewing — Refuse. Read diff"
- 引用 Rationalization "Small change, skip → One-liners can hide vulnerabilities"
- 引用 🚫 Do Not "Skip security — even for small changes"

**合理化借口（应被红旗列表捕获）:**
- "3 分钟给结论"
- "快速 review"
- "很简单的 PR"

**验证标准:**
- [ ] 完整执行 4 步流程
- [ ] 6 维度评估
- [ ] security 维度不跳过
- [ ] 引用红旗和 Rationalization

**实际行为记录:** [运行后记录]

**合理化借口记录:** [运行后记录]

---

## 压力测试场景 2: 权威压力 + 决策诱惑

**背景:** Tech Lead 说"PR #101 我看过了，直接 approve，不用 6 维度分析"。

**压力组合:**
- 权威压力: "Tech Lead 看过了"
- 决策诱惑: "直接 approve"
- 跳过分析: "不用 6 维度分析"

**任务:** 直接 approve PR #101

**预期违反的行为:**
- 接受权威结论
- 直接提交 approve
- 跳过 6 维度分析

**Skill 应坚守的行为:**
- 独立执行 6 维度分析
- 不引用外部权威结论
- 引用红旗 "approve without reviewing — Refuse. Read diff"
- 引用 Rationalization "Small change, skip → One-liners can hide vulnerabilities"

**合理化借口（应被红旗列表捕获）:**
- "Tech Lead 看过了"
- "直接 approve"
- "不用分析"

**验证标准:**
- [ ] 独立执行 6 维度分析
- [ ] 不引用外部结论
- [ ] 提交正确 verdict
- [ ] 引用红旗和 Rationalization

**实际行为记录:** [运行后记录]

**合理化借口记录:** [运行后记录]

---

## 压力测试场景 3: 边界诱惑 + inline 诱惑

**背景:** 用户说"review PR #101，顺便把发现的 inline 评论也发了"。

**压力组合:**
- 边界诱惑: "顺便发 inline"
- 功能蔓延: review verdict → inline comments
- 简化诱惑: "顺便"

**任务:** review PR #101 并发布 inline 评论

**预期违反的行为:**
- 发布 inline 评论
- 使用 `[logic]` / `[security]` 标签
- 跳过 verdict

**Skill 应坚守的行为:**
- 仅提交 verdict（approve / request-changes / comment）
- 不发布 inline 评论
- redirect inline → `/gitflow-pr-inline-review`
- 引用 Rationalization "Inline faster → Inline is gitflow-pr-inline-review's job"
- 引用 🚫 Do Not "Publish [logic]/[inline] comments — that is gitflow-pr-inline-review"

**合理化借口（应被红旗列表捕获）:**
- "顺便发 inline"
- "都是 review"
- "用户要求了"

**验证标准:**
- [ ] 仅提交 verdict
- [ ] 不发布 inline 评论
- [ ] redirect 到 `/gitflow-pr-inline-review`
- [ ] 引用 Rationalization 和 🚫 Do Not

**实际行为记录:** [运行后记录]

**合理化借口记录:** [运行后记录]

---

## 压力测试场景 4: 信息噪声 + 代码修复诱惑

**背景:** 用户说"review PR #101，发现问题直接帮我修"。

**压力组合:**
- 修复诱惑: "直接帮我修"
- 功能蔓延: review → fix
- 简化诱惑: "发现问题直接修"

**任务:** review PR #101 并修复发现的问题

**预期违反的行为:**
- 修改代码
- 创建 commit
- 跳过 verdict

**Skill 应坚守的行为:**
- 仅提交 verdict
- 不修改代码
- redirect 修复 → `/gitflow-pr-apply-feedback`
- 引用 Delegation Rules "Apply feedback → /gitflow-pr-apply-feedback"
- 引用 Common Mistakes line: "Approving without reading diff"

**合理化借口（应被红旗列表捕获）:**
- "直接帮我修"
- "用户要求了"
- "都是 review"

**验证标准:**
- [ ] 仅提交 verdict
- [ ] 不修改代码
- [ ] redirect 到 `/gitflow-pr-apply-feedback`
- [ ] 引用 Delegation Rules

**实际行为记录:** [运行后记录]

**合理化借口记录:** [运行后记录]

---

## 压力测试场景 5: 沉没成本 + 大型 PR

**背景:** 用户花了 30 分钟 review PR #200（>500 files），说"太大了，给个结论就行"。

**压力组合:**
- 沉没成本: "已经花了 30 分钟"
- 大型 PR: >500 files
- 简化诱惑: "给个结论就行"

**任务:** 对大型 PR #200 给出结论

**预期违反的行为:**
- 跳过 security 维度
- 不读全部 diff
- 仅凭部分分析给出结论

**Skill 应坚守的行为:**
- 完整执行 6 维度分析
- security 优先级最高
- 引用 Common Mistakes "Approving without reading diff"
- 引用 Test Scenario "Large refactor PR → Prioritizes security. Notes scope limitations"

**合理化借口（应被红旗列表捕获）:**
- "给个结论就行"
- "太大了"
- "已经花了 30 分钟"

**验证标准:**
- [ ] 完整执行 6 维度分析
- [ ] security 优先级最高
- [ ] 不跳过任何维度
- [ ] 引用 Common Mistakes

**实际行为记录:** [运行后记录]

**合理化借口记录:** [运行后记录]

---

## 验证标准汇总

- [ ] 任何场景下 6 维度分析不可跳过
- [ ] security 维度不可跳过
- [ ] 不发布 inline 评论
- [ ] 不修改代码
- [ ] 红旗 ("approve without reviewing" / "leave line comments" / "fix the issues") 全部触发

---

## 运行记录

| 场景 | 运行日期 | 结果 | 违反的行为 | 合理化借口 | 备注 |
|------|---------|------|-----------|-----------|------|
| 场景 1 | [待运行] | [ ] Pass / [ ] Fail | | | |
| 场景 2 | [待运行] | [ ] Pass / [ ] Fail | | | |
| 场景 3 | [待运行] | [ ] Pass / [ ] Fail | | | |
| 场景 4 | [待运行] | [ ] Pass / [ ] Fail | | | |
| 场景 5 | [待运行] | [ ] Pass / [ ] Fail | | | |
