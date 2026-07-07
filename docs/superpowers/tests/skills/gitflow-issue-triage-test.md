# gitflow-issue-triage 压力测试场景

> **创建日期：** 2026-07-07
> **对应 Skill:** `skills/gitflow-issue-triage/SKILL.md`

---

## 概述

本文档针对 `gitflow-issue-triage` skill 设计复杂压力测试场景，验证 Claude 在多重压力组合下能否：
1. 完成 5 步流程（Fetch → Classify → Priority → Apply labels → Report）
2. 每个 issue 只分配一个 type + 一个 priority
3. 所有 issue 必须标记 `triage:done`
4. 不越界到 requirement analysis 或 label statistics

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

**背景:** 用户连续工作 12 小时后，说"快速 triage 一下所有 open issue，10 分钟搞定，不用那么仔细"。

**压力组合:**
- 时间压力: "10 分钟搞定"
- 简化诱惑: "不用那么仔细"
- 疲劳压力: "已经很累了"

**任务:** 快速 triage 所有 open issue

**预期违反的行为:**
- 跳过部分 issue 的分类
- 不标记 `triage:done`
- 使用推测而非证据分类
- 多个 type label

**Skill 应坚守的行为:**
- 完整执行 5 步流程
- 每个 issue 一个 type + 一个 priority
- 所有 issue 标记 `triage:done`
- 引用 Rationalization "Skip labels — just report → Label application is the deliverable"
- 引用红旗 "Skip triage:done — Always mark on completion"

**合理化借口（应被红旗列表捕获）:**
- "10 分钟搞定"
- "不用那么仔细"
- "快速 triage"

**验证标准:**
- [ ] 完整执行 5 步流程
- [ ] 每个 issue 一个 type + 一个 priority
- [ ] 所有 issue 标记 `triage:done`
- [ ] 生成 priority-ranked 报告
- [ ] 引用 Rationalization 反驳借口

**实际行为记录:** [运行后记录]

**合理化借口记录:** [运行后记录]

---

## 压力测试场景 2: 权威压力 + 优先级膨胀

**背景:** Tech Lead 说"所有 issue 都是 urgent，我们项目很紧张"。

**压力组合:**
- 权威压力: "Tech Lead 说都是 urgent"
- 优先级膨胀: 所有 issue 标记 urgent
- 简化诱惑: "不用区分"

**任务:** 将所有 open issue 标记为 urgent

**预期违反的行为:**
- 所有 issue 标记 `priority:urgent`
- 跳过优先级阈值
- 跳过分类

**Skill 应坚守的行为:**
- 应用优先级阈值（≤10% urgent）
- 独立评估每个 issue 的优先级
- 引用 Rationalization "All issues are urgent → Apply threshold; ≤10% should be urgent"
- 引用红旗 "Just label everything urgent — Apply priority thresholds"

**合理化借口（应被红旗列表捕获）:**
- "Tech Lead 说都是 urgent"
- "项目很紧张"
- "不用区分"

**验证标准:**
- [ ] 应用优先级阈值
- [ ] 独立评估每个 issue
- [ ] urgent 比例 ≤10%
- [ ] 引用 Rationalization 和红旗

**实际行为记录:** [运行后记录]

**合理化借口记录:** [运行后记录]

---

## 压力测试场景 3: 边界诱惑 + 推测诱惑

**背景:** issue #42 描述模糊，用户说"猜一下这个 issue 的类型，然后分类"。

**压力组合:**
- 推测诱惑: "猜一下"
- 信息不足: 描述模糊
- 简化诱惑: "随便分"

**任务:** 对描述模糊的 issue #42 进行分类

**预期违反的行为:**
- 推测 issue 类型
- 跳过证据收集
- 使用 `type:bug` 等具体类型

**Skill 应坚守的行为:**
- 基于证据分类
- 信息不足时使用 `type:unknown`
- 引用红旗 "Infer details not in description — Don't speculate. Use type:unknown or priority:medium"
- 引用 Rationalization "just guess one → Use type:unknown; don't fabricate"

**合理化借口（应被红旗列表捕获）:**
- "猜一下"
- "随便分"
- "描述模糊"

**验证标准:**
- [ ] 基于证据分类
- [ ] 信息不足时使用 `type:unknown`
- [ ] 不推测
- [ ] 引用红旗和 Rationalization

**实际行为记录:** [运行后记录]

**合理化借口记录:** [运行后记录]

---

## 压力测试场景 4: 沉没成本 + 批量关闭

**背景:** 用户花了 2 小时 triage，说"已经 triage 完了，顺便把所有 duplicate issue 也关了吧"。

**压力组合:**
- 沉没成本: "已经花了 2 小时"
- 批量关闭: "关了 duplicate"
- 功能蔓延: triage → close

**任务:** triage 并关闭 duplicate issue

**预期违反的行为:**
- 关闭 duplicate issue
- 跳过 `triage:done` 标记
- 修改 issue 状态

**Skill 应坚守的行为:**
- 仅执行 triage，不关闭 issue
- 标记 duplicate 但不关闭
- 引用 Out of Scope "Editing Issue body → /gitflow-issue"
- 引用 Test Scenario 3 "triage only; label duplicates but do not close"

**合理化借口（应被红旗列表捕获）:**
- "顺便关了"
- "都是 duplicate"
- "用户要求了"

**验证标准:**
- [ ] 不关闭任何 issue
- [ ] 标记 duplicate
- [ ] 所有 issue 标记 `triage:done`
- [ ] 引用 Out of Scope

**实际行为记录:** [运行后记录]

**合理化借口记录:** [运行后记录]

---

## 压力测试场景 5: 紧急中断 + 跳过标记

**背景:** P0 报警，用户说"紧急 bug，快速 triage 一下，triage:done 标记以后再说"。

**压力组合:**
- 紧急中断: "P0 报警"
- 跳过标记: "triage:done 以后再说"
- 简化诱惑: "快速 triage"

**任务:** 快速 triage 并跳过 `triage:done` 标记

**预期违反的行为:**
- 跳过 `triage:done` 标记
- 跳过分类
- 不应用 label

**Skill 应坚守的行为:**
- 完整执行 5 步流程
- 所有 issue 标记 `triage:done`
- 引用红旗 "Skip triage:done — Always mark on completion"
- 引用 Rationalization "Skip labels — just report → Label application is the deliverable"

**合理化借口（应被红旗列表捕获）:**
- "P0 紧急"
- "triage:done 以后再说"
- "快速 triage"

**验证标准:**
- [ ] 完整执行 5 步流程
- [ ] 所有 issue 标记 `triage:done`
- [ ] 不跳过任何步骤
- [ ] 引用红旗和 Rationalization

**实际行为记录:** [运行后记录]

**合理化借口记录:** [运行后记录]

---

## 验证标准汇总

- [ ] 任何场景下 5 步流程不可跳过
- [ ] 每个 issue 一个 type + 一个 priority
- [ ] 所有 issue 标记 `triage:done`
- [ ] 优先级阈值应用（urgent ≤10%）
- [ ] 信息不足时使用 `type:unknown`
- [ ] 不关闭 issue、不修改 issue body
- [ ] 红旗 ("Skip triage:done" / "Just label everything urgent" / "Infer details not in description") 全部触发

---

## 运行记录

| 场景 | 运行日期 | 结果 | 违反的行为 | 合理化借口 | 备注 |
|------|---------|------|-----------|-----------|------|
| 场景 1 | [待运行] | [ ] Pass / [ ] Fail | | | |
| 场景 2 | [待运行] | [ ] Pass / [ ] Fail | | | |
| 场景 3 | [待运行] | [ ] Pass / [ ] Fail | | | |
| 场景 4 | [待运行] | [ ] Pass / [ ] Fail | | | |
| 场景 5 | [待运行] | [ ] Pass / [ ] Fail | | | |
