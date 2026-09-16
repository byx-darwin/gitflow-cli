# Issue Triage Report — 2026-09-16

Skill: `gf-issue-triage` · Scope: all open Issues (repo-wide) · CLI: `gf`
Context: Phase 4 "Issue triage" step of `/gf-workflow` full-mode run (contract `wf-2026-09-16-001`) for Issue #327 (`feat(skills): 新增 gf-smell`), delivered via local merge into `dev`.

## Summary

- Open Issues scanned: 30
- Already `triage:done` (skipped, idempotent): 7 — #240, #227, #188, #114, #103, #102, #101
- Newly triaged this run: 23 — #349, #348, #347, #346, #345, #344, #343, #342, #341, #340, #339, #338, #337, #336, #335, #334, #333, #332, #331, #330, #329, #327, #324

Issue #327 itself is **still open** — `gf issue list --state open` confirms it has no close/merge status yet, even though its delivery (local merge into `dev`, PR merge commit `530b43f`) is complete per the workflow contract. Per task constraints this run does **not** close it; it was triaged normally (`type:feature`, `priority:medium`) and flagged below as ready to close.

## Newly Triaged (this run)

| # | Title | Type | Priority | Rationale |
|---|-------|------|----------|-----------|
| 324 | fix(pipeline): running/queued 状态误计入失败桶 | bug | high | Confirmed recurring defect (9 prior PR reproductions, #311–#323), corrupts `gf-pipeline-analyzer`'s success-rate output — a core, frequently-used reporting path. |
| 341 | fix(build): local-rebuild 含 cargo clean，违反 CLAUDE.md 硬禁令 | bug | high | Makefile target unconditionally runs a destructive operation the project's own CLAUDE.md explicitly forbids without confirmation; direct policy conflict with real cost risk. |
| 327 | feat(skills): 新增 gf-smell | feature | medium | New skill capability; delivery already merged into `dev` per this workflow run — see closing note above. |
| 329 | feat(skills): 新增 gf-walkthrough | feature | medium | New skill (delivery walkthrough packages + evidence tiers); planned as part of the same audit batch as #327/#330–332. |
| 330 | feat(skills): 新增 gf-issue-decompose | feature | medium | New skill (vertical-slice batch decomposition); feeds `Blocked by` edges consumed by #337. |
| 331 | feat(skills): 新增 gf-architecture-diagram | feature | medium | New skill (dependency-manifest-driven diagram generation). |
| 332 | feat(skills): 新增 gf-refactor | feature | medium | New skill; fills the unsupported REFACTOR stage of the project's own TDD cycle. |
| 333 | feat(skills): gf-quality / gf-review 引入三级证据标注 | enhancement | medium | Backfills evidence-tier vocabulary (shared with #329) into two existing, actively used skills. |
| 334 | feat(skills): gf-pr-apply-feedback 补审查闭环契约 | enhancement | medium | Closes an open-loop review-fix cycle in an actively used skill; bounded scope (reuses existing 3-retry pattern). |
| 335 | feat(skills): gf-issue-create / gf-issue-review 垂直切片 | enhancement | medium | Adds falsifiable-acceptance-criteria and slice-direction rigor to the Issue-authoring pipeline itself. |
| 336 | feat(workflow): gf-workflow 补错误分类恢复表 | enhancement | medium | Adds a bounded-retry error taxonomy to the orchestration skill; currently has no retry ceiling. |
| 337 | feat(workflow): gf-workflow-batch 按依赖边拓扑排序 | enhancement | medium | Prevents wasted full 4-phase runs on blocked Issues; depends on `Blocked by` data that #330 will populate. |
| 338 | fix(skills): gf-pr frontmatter 非法键与 gf-pr-review 引用断链 | bug | medium | Verified broken links (3x) plus an illegal frontmatter key with undefined parser behavior; localized fix. |
| 339 | chore(skills): _common.sh 零调用方 | enhancement | low | Dead/unused shared library with minor `set -e`/stderr side effects on any future caller; no current functional impact. |
| 340 | fix(quality): 覆盖率工具分裂（tarpaulin vs llvm-cov） | bug | medium | Two disagreeing coverage tools with a workaround (pick either consistently); affects trust in `COV_THRESHOLD` but not a hard blocker. |
| 342 | chore(ci): check-agent-sync 形同虚设 | enhancement | medium | CI gate silently checks only file existence while two real validators sit unused; a quality-gate integrity gap. |
| 343 | feat(skills): 只读 skill 加 allowed-tools | enhancement | low | Hardening of an already-documented (prompt-level) constraint; explicitly requires case-by-case verification before rollout. |
| 344 | feat(workflow): gf-security-check / gf-regression 接入 gf-workflow | enhancement | medium | Two pre-delivery checks currently never auto-trigger, leaving Phase 4 incomplete by the workflow's own definition. |
| 345 | feat(workflow): 渲染进度看板 HTML | enhancement | low | Pure observability/UX addition on top of already-existing contract state; no correctness risk. |
| 346 | feat(skills): diff 交互式审阅网页渲染 | enhancement | low | UX improvement over existing Markdown review output; net-new rendering feature, not corrective. |
| 347 | feat(dist): 新增 .claude-plugin 清单 | enhancement | low | Adds a secondary, non-replacing install/discovery path; explicitly does not touch the primary mechanism. |
| 348 | fix(quality): gf-quality references 三处功能性缺陷 | bug | medium | Verified: dangling `ruby.md` reference, a reference doc that authorizes behavior SKILL.md explicitly forbids, and a coverage-tool naming mismatch (overlaps #340). |
| 349 | refactor(skills): 语言画像单源化 | enhancement | low | Verified cross-skill and intra-file duplication/drift in `references/<lang>.md`; grows with each new per-language skill but has no user-facing breakage today. |

## Full Priority-Ranked View (all 30 open Issues)

### 🔴 Urgent (0 — 0%)

None.

### 🟠 High (2 — 7%)

| # | Title | Type |
|---|-------|------|
| 324 | gf pipeline report 对 running/queued 状态的 run 计入失败桶 | bug |
| 341 | Makefile local-rebuild 含 cargo clean，违反 CLAUDE.md 硬禁令 | bug |

### 🟡 Medium (18 — 60%)

| # | Title | Type |
|---|-------|------|
| 327 | 新增 gf-smell | feature |
| 329 | 新增 gf-walkthrough | feature |
| 330 | 新增 gf-issue-decompose | feature |
| 331 | 新增 gf-architecture-diagram | feature |
| 332 | 新增 gf-refactor | feature |
| 333 | gf-quality / gf-review 引入三级证据标注 | enhancement |
| 334 | gf-pr-apply-feedback 补审查闭环契约 | enhancement |
| 335 | gf-issue-create / gf-issue-review 垂直切片 | enhancement |
| 336 | gf-workflow 补错误分类恢复表 | enhancement |
| 337 | gf-workflow-batch 按依赖边拓扑排序 | enhancement |
| 338 | gf-pr frontmatter 非法键与 gf-pr-review 引用断链 | bug |
| 340 | 覆盖率工具分裂 | bug |
| 342 | check-agent-sync 形同虚设 | enhancement |
| 344 | gf-security-check / gf-regression 接入 gf-workflow | enhancement |
| 348 | gf-quality references 三处功能性缺陷 | bug |
| 103 | 效率分析报表 + v1.1.0 + 2.0 预告 | feature |
| 102 | MCP 服务器（Agent 原生接口） | feature |
| 101 | 贡献者路径 + 月度发布节奏 | feature |

### 🟢 Low (10 — 33%)

| # | Title | Type |
|---|-------|------|
| 339 | _common.sh 零调用方 | enhancement |
| 343 | 只读 skill 加 allowed-tools | enhancement |
| 345 | 渲染进度看板 HTML | enhancement |
| 346 | diff 交互式审阅网页渲染 | enhancement |
| 347 | 新增 .claude-plugin 清单 | enhancement |
| 349 | 语言画像单源化 | enhancement |
| 240 | upstream CLI 新版本: glab 1.115.0 | enhancement |
| 227 | upstream CLI 新版本: gh 2.98.0 | enhancement |
| 188 | upstream CLI 新版本: gitcode 0.11.1 | enhancement |
| 114 | 1.0 发布宣发文章 | docs |

## Findings / Attention Items

1. **#327 is ready to close.** It was delivered by this workflow run's local merge into `dev` (merge commit `530b43f`), but the Issue itself remains open with no `triage:done` prior state. Per task instructions this run does not close it — flagging for the workflow owner or a subsequent Phase-4 step to close explicitly.
2. **#340 and #348 overlap** on the same underlying defect (gf-quality's `rust.md` pointing at `cargo-tarpaulin` while `Makefile` uses `cargo llvm-cov`). Both were triaged independently since neither Issue body references the other; worth linking or merging when actioned.
3. **#333/#329 and #330/#335/#337 form two dependency clusters** (evidence-tier vocabulary; vertical-slice + dependency-edge tooling) called out explicitly in their own Issue bodies — useful input for `gf-workflow-batch`'s topological ordering once #337 lands.
4. No duplicates found; no `type:unknown` (ambiguous) classifications needed this run.
5. Urgent-priority threshold respected: 0/30 marked urgent (≤10% guideline); high is 2/30 (7%).
6. Idempotency confirmed: the 7 previously-triaged Issues (#240, #227, #188, #114, #103, #102, #101) were correctly skipped — no redundant `gf issue add-label` calls made against them.

## Labels Applied

| # | Labels added |
|---|---------------|
| 324 | `type:bug`, `priority:high`, `triage:done` |
| 327 | `type:feature`, `priority:medium`, `triage:done` |
| 329 | `type:feature`, `priority:medium`, `triage:done` |
| 330 | `type:feature`, `priority:medium`, `triage:done` |
| 331 | `type:feature`, `priority:medium`, `triage:done` |
| 332 | `type:feature`, `priority:medium`, `triage:done` |
| 333 | `type:enhancement`, `priority:medium`, `triage:done` |
| 334 | `type:enhancement`, `priority:medium`, `triage:done` |
| 335 | `type:enhancement`, `priority:medium`, `triage:done` |
| 336 | `type:enhancement`, `priority:medium`, `triage:done` |
| 337 | `type:enhancement`, `priority:medium`, `triage:done` |
| 338 | `type:bug`, `priority:medium`, `triage:done` |
| 339 | `type:enhancement`, `priority:low`, `triage:done` |
| 340 | `type:bug`, `priority:medium`, `triage:done` |
| 341 | `type:bug`, `priority:high`, `triage:done` |
| 342 | `type:enhancement`, `priority:medium`, `triage:done` |
| 343 | `type:enhancement`, `priority:low`, `triage:done` |
| 344 | `type:enhancement`, `priority:medium`, `triage:done` |
| 345 | `type:enhancement`, `priority:low`, `triage:done` |
| 346 | `type:enhancement`, `priority:low`, `triage:done` |
| 347 | `type:enhancement`, `priority:low`, `triage:done` |
| 348 | `type:bug`, `priority:medium`, `triage:done` |
| 349 | `type:enhancement`, `priority:low`, `triage:done` |
