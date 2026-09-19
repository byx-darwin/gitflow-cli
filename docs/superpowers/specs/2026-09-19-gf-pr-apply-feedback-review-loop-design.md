# 设计文档：gf-pr-apply-feedback 审查闭环契约与轮次上限

**Workflow:** `wf-2026-09-19-002`（full 模式）
**Issue:** #334

## 背景

`skills/gf-pr-apply-feedback/SKILL.md:110-134` 当前是开环：遍历 PR 评论列表，逐条修复，列表耗尽即结束。没有「findings 归零」的判据，修完不回跳复审，也没有轮次概念。

对照 `smallnest/goal-workflow` 的 `skills/review-it/SKILL.md`，它是语义收敛循环：直到复审不再返回可执行 finding 才停止，并配三条软护栏——finding 可被拒绝、禁止为纯消耗性变化重跑、空 diff 短路。它没有硬轮次上限；本仓库 `skills/gf-pipeline-analyzer/SKILL.md:101-109` 已有成熟的「连续同水位无补救则升级交回用户」机制，直接复用同一模式。

对方 helper 有个实质 bug 值得引以为戒：`wait ... || true; TEST_EXIT=$?` 使 `TEST_EXIT` 恒为 0，测试失败分支成了死代码。实现时不得重蹈。

## 范围

只改 `skills/gf-pr-apply-feedback/SKILL.md` 一个文件。不改 `gf-pr-review`、`gf` CLI 本身——复审复用 `gf-pr-review` 现有的 6 维度评估与 `gf review approve/request-changes` 输出，不新增子命令。

## 循环机制

### 复审来源

push 后自动调用 `gf-pr-review`（6 维度评估 + `gf review` 结论），而非重新拉取 PR 评论列表——评论列表耗尽不等于代码已无问题，AC 明确排除这个判据。

- 结论 `approve` → findings 归零 → 循环结束（DONE）
- 结论 `request-changes` → 其 ⚠️ 项即本轮 actionable findings，进入下一步

### 消耗性重跑短路

push 后先跑 `git diff --stat <上一轮 SHA>..HEAD`；若改动集合只命中注释/文档/纯字符串文案（无逻辑代码行变化），直接短路为 DONE，不触发复审调用。判定依据是**本轮 diff 内容**，不是评论文字。

### 拒绝记录（会话内，不持久化）

用户对某条 finding 选择拒绝时，记录一份会话内内存清单（dimension + `path:line` 指纹 + 拒绝理由）。下一轮复审返回的 finding 先与该清单比对，命中则直接剔除、不再向用户展示。清单不写文件、不发 PR 评论、不跨会话保留——与 Issue 描述的既有行为（"User rejects a comment → Skip; record as rejected"）保持同一持久化级别，只是把 record 的范围从单轮扩展到本次会话全程。

### 轮次上限

上限为 3，从**首轮修复**（即第一次 push）开始计数，不是从首次回跳复审开始。计数规则：每完成一次「修复 → push」算一轮。

- 第 1、2 轮：若复审仍有未拒绝的 actionable finding → 回到确认-修复环节，进入下一轮
- 第 3 轮 push 后复审仍有未拒绝的 actionable finding → **升级交回用户**：停止自动循环，列出剩余 finding 清单 + 已用轮次，等待用户手动决定（继续修 / 接受现状 / 手动拒绝剩余项），不自动重试、不报错终止整个技能

这一升级语义与 `gf-pipeline-analyzer` 的 Escalation Rule 一致（`SKILL.md:101-109`）：更强的呈现，而不是新增自动化能力。

## SKILL.md 改动点

1. **Flowchart**：`M[git push + pr comment summary] → DONE` 替换为 `M → {消耗性重跑短路?} → DONE | {回跳 gf-pr-review} → {approve?} → DONE | {request-changes} → {轮次<3?} → 剔除已拒绝项 → 回到 E(用户确认) | {轮次=3} → 升级交回用户`
2. **Core Pattern**：push 之后追加复审回跳、diff 短路判定、轮次计数的伪代码块
3. **Error Handling**：新增一行——命令退出码必须紧跟命令捕获（`cmd; EXIT=$?`），禁止 `... || true` 之后再取 `$?`，否则失败分支恒不可达
4. **Responsibility → In**：补充"回跳复审 · 轮次上限升级交回用户"
5. **Test Scenarios**：新增 4 个场景
   - 复审仍有 finding → 循环回到确认环节
   - finding 被拒绝后，下一轮复审即使再次命中同一 `path:line` 也不重复展示
   - 本轮 diff 仅改注释/文档 → 短路，不调用复审
   - 第 3 轮仍未清零 → 升级交回用户，不再自动重试
6. **Success Criteria**：补"循环终止判据是 finding 归零，不是评论列表耗尽"、"轮次上限 3，超限升级交回用户"、"测试退出码捕获不得被 `|| true` 吞掉"

## 不做的事（YAGNI）

- 不引入跨会话/跨进程持久化的拒绝记录存储（用户已确认会话内内存清单即可）
- 不新增独立 CLI 子命令或脚本，全部复用 `gf pr view` / `gf review` / `git push` / `git diff --stat`
- 不改动 `gf-pr-review` 本身的评估逻辑或输出格式
