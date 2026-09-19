# Issue Triage Report — 2026-09-19

**Context**: `gf-workflow` Phase 4 post-delivery check for Issue #333 (feat: gf-quality/gf-pr-review evidence grading), delivered via `local_merge` into `dev` (merge commit `ecd533a`).

**Run**: `gf-issue-triage` skill, GitHub platform, via `gf` CLI.

## Coverage

`gf issue list --state open --limit 100` returned **36** open Issues, no `pagination.truncated` flag present (confirmed via `--output json`) — full coverage, not a partial fetch.

Of the 36, 26 already carried `triage:done` from prior runs (idempotent skip candidates, re-verified as correctly typed). **10** Issues were newly classified this run: #371, #370, #369, #368, #367, #366, #364, #363, #362, #361.

## Duplicate found

**#367** is a byte-for-byte duplicate of **#366** (identical title/body, created 17 seconds apart: 366 at 08:22:50Z, 367 at 08:23:07Z). Per skill policy, duplicates are not marked `triage:done`; #367 was labeled `duplicate` and commented pointing to #366, which carries the actual `type:refactor` / `priority:medium` classification.

## Newly classified (this run)

| # | Title (abridged) | Type | Priority | Note |
|---|---|---|---|---|
| 371 | 架构图重生成后丢失了原有的语义架构信息 | docs | low | doc gap, no functional break |
| 370 | Stage 5 确定性检查逻辑恒真 | bug | medium | verification gate is a no-op |
| 369 | cargo-fmt pre-commit hook 只检查不应用 | enhancement | low | kept existing labels, marked done |
| 368 | gitcode release list 响应形状未经真实服务端验证 | enhancement | medium | kept existing labels, marked done |
| 367 | ReleaseData.created_at 回落 Utc::now() (dup of #366) | — | — | `duplicate`, not triaged |
| 366 | ReleaseData.created_at 回落 Utc::now()，静默伪造创建时间 | refactor | medium | kept existing labels, marked done |
| 364 | MilestoneApiResponse camelCase/snake_case 不符，静默归零 | bug | **high** | silent data corruption in production path |
| 363 | check-skills-drift 只比对目录名却报「内容一致」 | bug | medium | kept existing labels, marked done |
| 362 | gitlab auth status 忽略 --output json，多 host 折叠误报 | bug | **high** | causes false "unauthenticated" conclusion |
| 361 | dogfooding checklist 命令参数与 CLI 不符 | docs | low | stale doc, process-internal impact |

## Full distribution (35 triaged Issues, excluding #367 duplicate)

### By type

| Type | Count | % |
|---|---|---|
| enhancement | 17 | 48.6% |
| bug | 10 | 28.6% |
| feature | 5 | 14.3% |
| docs | 2 | 5.7% |
| refactor | 1 | 2.9% |

### By priority

| Priority | Count | % |
|---|---|---|
| 🔴 urgent | 0 | 0% |
| 🟠 high | 3 | 8.6% |
| 🟡 medium | 18 | 51.4% |
| 🟢 low | 14 | 40.0% |

Urgent share (0%) is within the ≤10% threshold; no over-flagging.

### 🟠 High priority (3)

- **#93** — feat(roadmap): 多角色项目评估与 2026 下半年产品路线图 (type:feature)
- **#364** — fix(github): MilestoneApiResponse camelCase/snake_case 静默归零 (type:bug)
- **#362** — fix(gitlab): auth status 忽略 --output json，多 host 误报未认证 (type:bug)

## Labels applied

```
gf issue add-label 371 --label "type:docs" --label "priority:low" --label "triage:done"
gf issue add-label 370 --label "type:bug" --label "priority:medium" --label "triage:done"
gf issue add-label 369 --label "triage:done"
gf issue add-label 368 --label "triage:done"
gf issue add-label 366 --label "triage:done"
gf issue add-label 367 --label "duplicate"
gf issue add-label 364 --label "type:bug" --label "priority:high" --label "triage:done"
gf issue add-label 363 --label "triage:done"
gf issue add-label 362 --label "type:bug" --label "priority:high" --label "triage:done"
gf issue add-label 361 --label "type:docs" --label "priority:low" --label "triage:done"
```

## Out of scope (per skill boundaries)

- No Issue bodies edited.
- No requirement-depth analysis performed (that's `gf-issue-review`).
- No label distribution analytics beyond this report's counts (that's `gf-label-stats`).
