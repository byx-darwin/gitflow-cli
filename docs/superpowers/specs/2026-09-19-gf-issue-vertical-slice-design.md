# gf-issue-create / gf-issue-review 垂直切片与可证伪验收 — 设计

**Issue:** #335
**Date:** 2026-09-19
**Related:** #330（批量拆解 skill，共用同一套判定标准，避免结论冲突）

## Context

`skills/gf-issue-create/SKILL.md:70-78` 的 Body 模板只有 `## Context / ## Goal /
## Acceptance Criteria` 三段，验收标准是自由文本 checklist，没有任何形状约束——既不要求
端到端窄路径（vertical slice），也不要求每条标准可证伪。

`skills/gf-issue-review/SKILL.md` 当前的需求完整性分析是三维（标题清晰度 / 描述充分度 /
验收标准清晰度），缺「切片方向」这一维，也不校验验收标准在 base commit 上是否已经为真
（即该标准其实什么都没约束住）。

参考 `smallnest/goal-workflow` 的 `skills/to-issues/SKILL.md`：
- `:15-22` 垂直切片：一张票贯穿一条端到端窄路径；明确「横向切层」（如「只改数据层」）是
  模型最常犯的默认错误。
- `:81` 可证伪验收：每条标准须指出「什么观察能证明它为假」；驳回三种形状——base 已成立 /
  依赖他票 / 复述需求。
- `:24` 兜底：整个改动能装进一个上下文窗口就不拆。

## Goal

1. `gf-issue-create` 的 Body 模板与创建流程加入「垂直切片方向」约束，拒绝横向切层票；
   验收标准要求写成可证伪形式。
2. `gf-issue-review` 的三维评分扩展为四维，新增「切片方向」评分；同时对既有的
   「验收标准清晰度」维度补充 base-commit-red 校验——若某条标准在 base commit 上已经
   成立（没有改动就已经为真），标记为无效标准。
3. 两个 skill 与 #330（`gf-issue-decompose` 批量拆解）共用同一套判定标准，避免同一份需求
   在单票创建路径与批量拆解路径下产生冲突结论。

## Non-Goals

- 不引入硬性上下文窗口大小检测（`smallnest/goal-workflow` 的 `:24` 兜底本次不照抄，仅
  作为写作指引提及）。
- 不改动 `gf-issue-decompose`（#330）本身的实现，只保证判定标准一致。

## Design

### gf-issue-create 改动点

- Body 模板追加一段一句话「Vertical Slice」自检提示：本票是否贯穿一条端到端窄路径，
  而非仅改动单一层（如「只改数据层」）。
- 验收标准撰写指引改为要求每条陈述「什么可观察结果能证明其为假」，而非静态复述需求。

### gf-issue-review 改动点

- Step 2 评分表从三行扩为四行，新增 `Slice Direction`（切片方向）维度，检查项：
  是否端到端 / 是否存在纯横向分层描述。
- `Acceptance Criteria Clarity` 维度的 Checks 追加一项：对每条 `- [ ]`，判断其陈述的
  观察在 base commit（未改动前）是否已经成立；若已成立，标记该条为 🔴 并在报告中列出。
- 报告模板的评分表增加第四行；不改变现有的 Improvement Suggestions / Suggested Title
  结构。

### 与 #330 的一致性

`gf-issue-decompose` 拆票时使用的「端到端窄路径」与「可证伪验收」判断标准，与本 Issue
在 `gf-issue-create`/`gf-issue-review` 中实现的判断标准必须是同一套文字表述，避免两条
路径对同一形状的票给出矛盾结论。落地时以本 Issue 的表述为准，#330 若已存在等价表述则
对齐引用，不重复定义。

## Acceptance Criteria（同 Issue 原文）

- [ ] 创建的票覆盖一条端到端窄路径，出现「只改数据层」这类横向切层票则为假
- [ ] 每条验收标准可指出「什么观察能证明它为假」，复述需求式的标准被驳回
- [ ] `gf-issue-review` 评分维度含「切片方向」，仍为三维则为假
- [ ] `gf-issue-review` 校验验收标准在 base commit 上是否为红，base 上已成立的标准被标记
- [ ] 与 #330 的判定标准一致，两者结论冲突则为假
