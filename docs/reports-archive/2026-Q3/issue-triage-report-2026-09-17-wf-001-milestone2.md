# Issue Triage Report — 2026-09-17

Skill: `gf-issue-triage` · Scope: all open Issues (repo-wide) · CLI: `gf`
Context: `gf-workflow` contract `wf-2026-09-17-001`, milestone #2 "覆盖率度量口径统一" delivery. `fix/340-coverage-metric-unification` was local-merged into `dev` as `eb14dc0` (preceded by `9df5d32`, `9250b58`, `a9b0497`, `ce243e8` on the same branch), substantively resolving **#340**, **#348**, **#354**.

> **Note — this report supersedes the prior same-day file at this path.** A file already existed at `docs/issue-triage-report-2026-09-17.md` before this run, dated 2026-09-17 but reporting on a *different* context (contract `wf-2026-09-16-002`, Issue #329, merge `3e9bcc5`). That content has been overwritten per this run's explicit output-path instruction. See "Anomalies" below.

## Summary

- Open Issues scanned: **32** (`gf issue list --state open --limit 100`)
- Already `triage:done` (skipped, idempotent): **32 — all of them**
- Newly triaged this run: **0**
- `triage:done` labels applied this run: **0** (nothing needed applying; all 32 already carry `type:*` + `priority:*` + `triage:done`)

No open Issue required classification or labeling this run. Every open Issue already carries a complete `type:*` / `priority:*` / `triage:done` label set from prior triage runs (2026-08-31 through 2026-09-16). This is expected idempotent behavior per the skill's Test Scenario 5.

## Delivered-but-open (done-pending-closure) — #340, #348, #354

These three Issues are **substantively resolved** by today's milestone #2 delivery (local merge `eb14dc0` into `dev`), but remain **open** in the tracker because the merge was local (no PR), so there is no `Closes #N` linkage to trigger an auto-close:

| # | Title | Current labels | Status |
|---|-------|-----------------|--------|
| 340 | fix(quality): 覆盖率工具分裂 —— gf-quality 用 tarpaulin，Makefile 用 llvm-cov | `type:bug`, `priority:medium`, `triage:done` | ✅ done, pending closure |
| 348 | fix(quality): gf-quality references 三处功能性缺陷 —— 断链、与 SKILL.md 冲突、覆盖率口径名实不符 | `type:bug`, `priority:medium`, `triage:done` | ✅ done, pending closure |
| 354 | fix(gf-quality): Gate 3 判据称增量覆盖率，命令实测全量，口径不一致 | `type:bug`, `priority:medium`, `triage:done` | ✅ done, pending closure |

**These are classified as done-pending-closure, not as outstanding work.** All three already carried correct `type:bug` / `priority:medium` / `triage:done` labels from the 2026-09-16 triage run, so no re-labeling was needed or performed. Verified via `gf issue view <n> --output json`: all three show `"state": "open"`. **Do not treat these as unstarted work in future triage or planning passes** — the fix is merged into `dev`; only the tracker-side close action (owner permission required) remains. This run does not close them (closing a ticket requires explicit user permission, not given for this run).

Cross-reference: milestone #2 itself (`gf milestone list`) lists `openIssues: 0`, `closedIssues: 0` for all three — not because the work is unfinished, but because `gf` currently has no way to attach Issues to a milestone (`gf issue create`/`edit` lack `--milestone`; tracked as new Issue **#357**, filed as part of this delivery). The milestone's own description text cross-references #340/#348/#354 by number as a manual grouping workaround.

## Full Priority-Ranked View (all 32 open Issues, existing labels — unchanged this run)

### 🔴 Urgent (0 — 0%)

None.

### 🟠 High (1 — 3%)

| # | Title | Type |
|---|-------|------|
| 93 | 多角色项目评估与 2026 下半年产品路线图（稳定化 → 增长 → 扩张） | feature |

### 🟡 Medium (20 — 63%)

| # | Title | Type | Note |
|---|-------|------|------|
| 357 | 无法将 Issue/PR 挂到 milestone，进度计数器结构性不可用 | feature | filed this delivery (blocks milestone tracking, see above) |
| 354 | Gate 3 判据称增量覆盖率，命令实测全量，口径不一致 | bug | ✅ done, pending closure |
| 353 | worktree 软链深度公式对 .claude 多退一级，自检未覆盖该软链 | bug | |
| 352 | 外置到 docs/ 的 skill 依赖在 install-skills 后不可达（影响 9+ skills） | bug | |
| 350 | skill-conventions 词数统计命令恒返回 1，500 词硬限从未执行 | bug | |
| 348 | gf-quality references 三处功能性缺陷 | bug | ✅ done, pending closure |
| 344 | gf-security-check 与 gf-regression 接入 gf-workflow | enhancement | |
| 340 | 覆盖率工具分裂 —— gf-quality 用 tarpaulin，Makefile 用 llvm-cov | bug | ✅ done, pending closure |
| 338 | gf-pr frontmatter 非法键与 gf-pr-review 引用断链 | bug | |
| 337 | gf-workflow-batch 按依赖边拓扑排序取票 | enhancement | |
| 336 | gf-workflow 补错误分类恢复表与重试上限 | enhancement | |
| 335 | gf-issue-create / gf-issue-review 引入垂直切片与可证伪验收 | enhancement | |
| 334 | gf-pr-apply-feedback 补审查闭环契约与轮次上限 | enhancement | |
| 333 | gf-quality / gf-review 引入三级证据标注与失败测试 commit 溯源 | enhancement | |
| 332 | 新增 gf-refactor | feature | |
| 331 | 新增 gf-architecture-diagram | feature | |
| 330 | 新增 gf-issue-decompose | feature | |
| 103 | 效率分析报表 + v1.1.0 + 2.0 预告 | feature | |
| 102 | MCP 服务器（Agent 原生接口） | feature | |
| 101 | 贡献者路径 + 月度发布节奏 | feature | |

### 🟢 Low (11 — 34%)

| # | Title | Type |
|---|-------|------|
| 356 | e2e-github/gitlab 的 noauth 测试自称「任何环境均可运行」，实则依赖 gh/glab | bug |
| 355 | check-walkthrough-skill 校验项 #3 的 awk 用 getline 跳过相邻条目 | bug |
| 349 | 语言画像单源化 | enhancement |
| 347 | 新增 .claude-plugin 清单 | enhancement |
| 346 | diff 交互式审阅网页渲染 | enhancement |
| 345 | 渲染进度看板 HTML | enhancement |
| 343 | 只读 skill 加 allowed-tools | enhancement |
| 339 | _common.sh 零调用方 | enhancement |
| 240 | upstream CLI 新版本: glab 1.115.0 | enhancement |
| 227 | upstream CLI 新版本: gh 2.98.0 | enhancement |
| 188 | upstream CLI 新版本: gitcode 0.11.1 | enhancement |

## Change Since 2026-09-16 Report

- **Closed since 2026-09-16**: #324, #327, #329, #341, #351 (the last was filed and closed within this window; not present in the 2026-09-16 report).
- **New since 2026-09-16**: #350, #352, #353, #354, #355, #356, #357 — all arrived already `triage:done` (pre-labeled at creation time; confirmed no re-triage was needed).
- **#93 and #101 reappear** in this run's full scan (32 issues) but were **absent from the 2026-09-16 report's 30-issue scan**. Root cause: the 2026-09-16 (and earlier) runs used `gf issue list --state open` without `--limit`, which defaults to 30 and silently truncated the open set below the true total. This run used `--limit 100` and confirmed 32 open Issues exist. No content impact — both #93 and #101 already carried correct `type:feature`/`priority:high` and `type:feature`/`priority:medium` labels respectively — but the default-limit truncation risk is worth flagging (see Anomalies).

## Anomalies

1. **Default page size silently truncates `gf issue list --state open`.** Without `--limit`, the command returns only 30 of the 32 open Issues (confirmed: #93, #101 dropped). At least the 2026-09-16 report's "30 open Issues" figure was affected by this truncation, though its label conclusions were unaffected since the two missing Issues were already correctly triaged. Future triage runs should always pass `--limit` well above the expected count (or the CLI should default to fetching all pages) to avoid silently missing genuinely new, untriaged Issues.
2. **Report-path collision.** This run wrote to `docs/issue-triage-report-2026-09-17.md`, a path that already held a committed report from an earlier same-day run (different `gf-workflow` contract, `wf-2026-09-16-002`, for Issue #329). Per this task's explicit instruction the new content overwrites the old at the same path; the prior 2026-09-17 report content (for #329) is not preserved elsewhere. If both runs' audit trails need to survive, future same-day multi-context reports should use the skill's `[-<context>]` filename suffix (e.g. `issue-triage-report-2026-09-17-milestone2.md`) instead of the bare date.
3. **#340/#348/#354 are done but open** — flagged above, not actioned (closing requires explicit user permission not given for this run).

## Labels Applied

None. All 32 open Issues already carried complete `type:*` / `priority:*` / `triage:done` label sets prior to this run; no `gf issue add-label` calls were made.
