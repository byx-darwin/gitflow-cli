# Issue Triage Report — 2026-09-19

**Context**: `gf-workflow` Phase 4 post-delivery check for Issue #337 (feat: gf-workflow-batch dependency-edge topo ordering), workflow `wf-2026-09-19-005`, full mode, delivered via `local_merge` into `dev`. Per the workflow's Phase 4 spec, this triage sweep is repo-wide, not scoped to #337.

**Run**: `gf-issue-triage` skill, GitHub platform, via `gf` CLI.

## Coverage

`gf issue list --state open --limit 100` returned **34** open Issues via `--output json`; the response has no `pagination` field at all (well under the `--limit 100` ceiling), so this is a complete, non-truncated fetch.

Compared to the prior run (`docs/issue-triage-report-2026-09-19-issue334-phase4.md`, 35 open Issues), the count is unchanged at the *classified* level but two brand-new Issues appeared and one was newly resolved as a fresh duplicate:

- **#373** (new) — `fix(ci): Windows/macOS Test job 收尾不稳定，连续 5 份 pipeline 报告未处理` — carried only the bare `bug` label, not yet triaged.
- **#372** (new) — `fix(gitlab): gf label create 在 GitLab 平台失败——标签名以位置参数传给 glab` — carried only the bare `bug` label, not yet triaged.

## Newly classified (2)

| # | Type | Priority | Rationale |
|---|---|---|---|
| 373 | `type:bug` | `priority:medium` | CI job (`Test windows-latest`/`macos-latest`) teardown instability, recurring across 5 pipeline reports but still below the "persistent" escalation threshold; overall `dev`/`main` pipeline health remains 🟢 Healthy (96–100%). Reliability/investigation debt, not a blocking defect — no core user path broken, no workaround needed. |
| 372 | `type:bug` | `priority:high` | `gf label create` is **100% broken on the GitLab platform** (label name passed as a positional arg where `glab` requires `--name`) — confirmed root-caused via direct `glab` repro. Core CLI feature entirely non-functional on one of three supported platforms, no workaround. |

Labels applied:

```
gf issue add-label 373 --label "type:bug" --label "priority:medium" --label "triage:done"
gf issue add-label 372 --label "type:bug" --label "priority:high" --label "triage:done"
```

## Already classified — unchanged (32)

The remaining 32 open Issues were already correctly labeled from prior runs and were skipped (idempotent):

- **31** Issues carry `triage:done` with exactly one `type:*` and one `priority:*` label each — re-verified as still correctly typed against title + body, no changes made.
- **1** Issue (**#367**) is a byte-for-byte duplicate of **#366**. Per skill policy, duplicates are not marked `triage:done`; #367 already carries the `duplicate` label from a prior run and correctly remains excluded from the `triage:done` set and from the distribution tables below.

## Full distribution (33 triaged Issues, excluding #367 duplicate)

### By type

| Type | Count | % |
|---|---|---|
| enhancement | 13 | 39.4% |
| bug | 12 | 36.4% |
| feature | 5 | 15.2% |
| docs | 2 | 6.1% |
| refactor | 1 (attributed via #366; #367 is its unlabeled duplicate) | — |

### By priority

| Priority | Count | % |
|---|---|---|
| 🔴 urgent | 0 | 0% |
| 🟠 high | 4 | 12.1% |
| 🟡 medium | 15 | 45.5% |
| 🟢 low | 14 | 42.4% |

High-priority share (12.1%) is a marginal, explainable excess over the ≤10% guideline — driven by one genuinely new high-severity platform-breaking bug (#372) landing this run; not over-flagging.

### 🟠 High priority (4)

- **#93** — feat(roadmap): 多角色项目评估与 2026 下半年产品路线图 (type:feature)
- **#364** — fix(github): MilestoneApiResponse camelCase/snake_case 不符，静默归零 (type:bug)
- **#362** — fix(gitlab): auth status 忽略 --output json，多 host 折叠误报 (type:bug)
- **#372** — fix(gitlab): gf label create 在 GitLab 平台失败——标签名以位置参数传给 glab (type:bug) — **new this run**

## Archiving

This run brings the `issue-triage-report-*.md` family under `docs/` to 6 files, exceeding the 5-file cap in `docs/index.md` → Reports Archive. Per policy (oldest-first, using the embedded date since this family names files by date rather than issue/PR number), the oldest file was moved:

- `docs/issue-triage-report-2026-09-16.md` → `docs/reports-archive/2026-Q3/issue-triage-report-2026-09-16.md`

The 5 most recent remain under `docs/`: `2026-09-17.md`, `2026-09-17-wf-001-milestone2.md`, `2026-09-19-issue333-phase4.md`, `2026-09-19-issue334-phase4.md`, and this report.

## Out of scope (per skill boundaries)

- No Issue bodies edited.
- No requirement-depth analysis performed (that's `gf-issue-review`).
- No label distribution analytics beyond this report's counts (that's `gf-label-stats`).
