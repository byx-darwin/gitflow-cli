# Issue Triage Report — Issue #345 Phase 4 (gf-workflow)

- **Date**: 2026-09-21
- **Context**: `gf-workflow` Phase 4 post-delivery check for Issue #345 (`feat(workflow): 从 workflow 合同渲染进度看板 HTML`), just merged into `dev` at `d352cc3` (adds `scripts/render-workflow-dashboard.py` — a zero-dependency, stdlib-only Python3 script — plus a `make render-workflow-dashboard` Makefile target).
- **Skill**: `gf-issue-triage` (report-only run — **no labels were added or changed**, per the task's explicit no-side-effects instruction; this deviates from the skill's normal "apply labels" behavior by request).
- **Source**: `gf issue list --state open --limit 100` (25 Issues returned, well under the requested limit — full coverage; no `pagination.truncated` field present in this CLI version's output, consistent with an untruncated result).

## Coverage

All 25 open Issues already carry `triage:done` plus one `type:*` and one `priority:*` label from the triage pass run earlier today for Issue #344's delivery (and re-verified again for Issue #344's own Phase 4 report). This run re-verifies those classifications rather than performing a first pass, and specifically checks for staleness introduced by #345 landing. Issue #345 itself is now `state: closed` and has dropped out of the open-issue set, as expected.

The two relabels from the earlier #344-triage pass are confirmed still in effect and correct — not re-flagged, per instruction:
- **#380**: `type:enhancement` (was `type:refactor`, non-standard) — confirmed.
- **#343**: `priority:medium` (was `priority:low`) — confirmed.

## Priority-ranked summary

| Priority | Count | % |
|---|---|---|
| 🟠 high | 2 | 8.0% |
| 🟡 medium | 6 | 24.0% |
| 🟢 low | 17 | 68.0% |
| 🔴 urgent | 0 | 0% |

| Type | Count | % |
|---|---|---|
| type:feature | 12 | 48.0% |
| type:enhancement | 8 | 32.0% |
| type:bug | 4 | 16.0% |
| type:docs | 1 | 4.0% |

(#345 dropping out of the open set removes one `type:enhancement`/`priority:low` row from the prior report's tally; totals above reflect the current 25 open Issues.)

## Findings

### 1. #345's landed implementation sets a concrete precedent that #346 should follow (recommendation, no relabel)

**Issue #346** ("feat(skills): diff 交互式审阅网页渲染") is a close sibling of #345: both propose deriving a self-contained, offline HTML artifact from repo/workflow state, both cite the same upstream reference (`smallnest/goal-workflow`), and #346's own Acceptance Criteria already independently demand the same constraints #345 just implemented:

- 零第三方依赖 (stdlib-only) — #345 shipped `scripts/render-workflow-dashboard.py` as pure-stdlib Python3 (`glob`, `html`, `json`, `os`, `sys` only, no pip deps).
- 产物为自包含单文件，离线可用 — #345's output (`.cache/workflows/dashboard.html`) is a single self-contained HTML file with inline CSS, no external CDN.
- 产物落在 gitignore 覆盖的路径下 — #345 writes under `.cache/workflows/`, which line 45 of `.gitignore` already covers (`.cache/`).
- 派生视图，禁止手工编辑 — #345's script embeds an explicit Chinese-language banner in the generated HTML plus a module docstring warning "DERIVED VIEW — never hand-edit."
- Automation entry point via `make <target>` — #345 added `render-workflow-dashboard` to the `Makefile` with a `##`-commented help line, following the repo's existing target-with-description convention.

None of this makes #346 redundant — #346's scope (rendering a `git diff` into an interactive three-pane review page with Prism-based syntax highlighting) is materially different from #345's scope (rendering workflow-contract JSON into a status dashboard). But #345 is now a concrete, merged, in-repo example of exactly the pattern #346's Acceptance Criteria describe in the abstract (stdlib-only script, `.cache/`-scoped output, derived-view labeling, Makefile wiring). #346's Context section currently references only the external `smallnest/goal-workflow` template; it does not yet reference #345 as an in-repo precedent to follow for implementation conventions (script location under `scripts/`, output path shape, banner wording style, Makefile target/help-comment format).

**Recommendation** (not applied — report only): when #346 is picked up for implementation, its plan/design doc should explicitly reference `scripts/render-workflow-dashboard.py` and the `render-workflow-dashboard` Makefile target as the established in-repo convention for "derive an offline HTML artifact from repo state," rather than only Ruby-quoting the external upstream template. No label change is warranted — #346 remains correctly `type:enhancement` / `priority:low`; this is an implementation-guidance note, not a classification defect.

### No obsoleted, newly-actionable, or re-prioritized issues found

No open Issue is superseded, unblocked, or requires a priority change purely because #345 landed:

- No open Issue references "看板"/"dashboard"/"进度" other than #346 (addressed in Finding 1) and #345 itself (now closed).
- No open Issue references `gf-workflow-batch` in a way that #345's dashboard renderer would resolve or invalidate — the workflow-batch topology work (Issue #337, already delivered) and the dashboard renderer (#345) are independent concerns; no remaining open Issue proposes dashboard-adjacent work that #345 makes redundant.
- The Jev decision-layer epic (#382 and children #383–#393) and the milestone/close-reopen bug cluster (#394–#396) are unrelated to #345's scope; none reference workflow-contract visualization.

## Full classification (unchanged from prior triage pass; reviewed, not reapplied)

| # | Title | Type | Priority | Note |
|---|---|---|---|---|
| 396 | fix(gitlab): milestone 命令自建实例 404 | bug | high | ok |
| 395 | fix(github,gitcode): issue close/reopen 丢失 milestone | bug | medium | ok |
| 394 | fix(gitcode): pr close 反序列化报错 | bug | high | ok |
| 393 | feat(decision): Jev 离线评测与阈值校准 | feature | medium | ok |
| 392 | feat(workflow): Agent trace 无进展/循环检测 | feature | low | ok |
| 391 | feat(context): 选择性上下文压缩 | feature | low | ok |
| 390 | feat(query): 混合式自然语言过滤 | feature | low | ok |
| 389 | test(regression): 语义回归判定与对抗测试 | feature | low | ok |
| 388 | feat(workflow): Jev 语义规则检查 | feature | low | ok |
| 387 | feat(issue): 需求质量语义预检 | feature | low | ok |
| 386 | feat(review): PR 多维语义预筛 | feature | low | ok |
| 385 | feat(pipeline): CI 失败语义分类 | feature | low | ok |
| 384 | feat(skills): 任务到 Skill 置信度路由 | feature | low | ok |
| 383 | feat(workflow): 工作流模式推荐 | feature | low | ok |
| 382 | feat(decision): 引入 Jev 决策层（Issue triage 切片） | feature | medium | ok — epic parent |
| 380 | refactor(core): created_at 回落 Utc::now() | enhancement | medium | ok — prior relabel confirmed |
| 371 | docs(architecture-diagram): 语义架构信息丢失 | docs | low | ok |
| 370 | fix(architecture-diagram): Stage 5 检查恒真 | bug | medium | ok |
| 349 | refactor(skills): 语言画像单源化 | enhancement | low | ok |
| 347 | feat(dist): .claude-plugin 清单第二安装入口 | enhancement | low | ok |
| 346 | feat(skills): diff 交互式审阅网页渲染 | enhancement | low | see Finding 1 |
| 343 | feat(skills): 只读 skill allowed-tools 收敛 | enhancement | medium | ok — prior relabel confirmed |
| 240 | upstream: glab 1.115.0 | enhancement | low | ok — bot-filed |
| 227 | upstream: gh 2.98.0 | enhancement | low | ok — bot-filed |
| 188 | upstream: gitcode 0.11.1 | enhancement | low | ok — bot-filed |

## Archiving

Adding this report brought `docs/issue-triage-report-*.md` under `docs/` to 6 files, exceeding the skill's 5-file retention threshold. Per `docs/index.md`'s Reports Archive policy, files are ordered oldest-first by the issue/PR number embedded in the filename (a filename with no embedded number, like a bare date, sorts as oldest). The oldest — `issue-triage-report-2026-09-17.md` (no issue number embedded) — was moved to `docs/reports-archive/2026-Q3/` (its own date, 2026-09-17, falls in Q3), leaving the 5 most recent under `docs/`: `issue-triage-report-2026-09-19-issue333-phase4.md`, `issue-triage-report-2026-09-19-issue334-phase4.md`, `issue-triage-report-2026-09-19-issue337-phase4.md`, `issue-triage-report-344-2026-09-21.md`, and this report.
