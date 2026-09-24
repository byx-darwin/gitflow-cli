---
name: gf-issue-triage
description: |
  Use when the user wants to classify all open Issues by type and priority, then apply triage:done tags.
  当用户希望对所有 open Issue 按类型/优先级分类并打上 triage:done 标签时使用。
---

# gf-issue-triage

Batch classification of all open Issues — assigns one `type:*` label and one `priority:*` label per Issue, then marks `triage:done`. Outputs a priority-ranked report. Idempotent — skip already-triaged Issues.

## CLI Requirement

**MUST use `gf` CLI, NOT `gh` CLI.**

| CLI | Scope | Platform Support |
|-----|-------|------------------|
| `gf` | This project | GitHub + GitLab + GitCode |
| `gh` | GitHub only | GitHub only |

**Why**: `gf` is the unified CLI for this project. Using `gh` breaks GitLab/GitCode compatibility.

## Preconditions
- `gf` installed: `command -v gf`
- `gf` authenticated: `gf auth status`
## When to Use

| English | 中文 | Context |
|---------|------|---------|
| triage all issues | 对全部分类 | backlog grooming |
| classify issues | 分类 Issue | sprint planning |
| analyze an issue's requirement | 分析需求质量 | **NOT** → `/gf-issue-review` |
| label statistics | 标签统计 | **NOT** → `/gf-label-stats` |

## When NOT to Use

| Scenario | Why Not | Use Instead |
|----------|---------|-------------|
| Analyzing a single Issue's requirement quality | This skill batch-classifies all open Issues, not deep analysis of one | `/gf-issue-review` for three-dimension requirement review |
| Viewing label distribution statistics | This skill applies labels, not analyzes their distribution | `/gf-label-stats` for read-only label analytics |
| Creating or editing label definitions | This skill assigns labels to Issues, not manages label metadata | `/gf-label-milestone` for label CRUD |
| Creating a new Issue | This skill classifies existing Issues only | `/gf-issue-create` for Issue creation |
| Editing Issue body text | This skill only adds labels, never modifies Issue content | `/gf-issue` for edit operations |

> **Historical note:** before the pagination fix (Issue #360), `gf issue list --state open`
> silently returned only the first 30 Issues. Triage reports generated before that fix may
> have classified an incomplete set. Re-run triage rather than trusting an older report.

## Core Pattern

```bash
gf issue list --state open
gf issue add-label <n> --label "type:<t>" --label "priority:<p>" --label "triage:done"
```

## Quick Reference

| Goal | Command |
|------|---------|
| List open | `gf issue list --state open` |
| Add label | `gf issue add-label <n> --label "<l>"` |
| Filter by label | `gf issue list --label "<l>" --state open` |

**Type labels:** `type:bug` · `type:feature` · `type:enhancement` · `type:docs` · `type:question`
**Priority labels:** `priority:urgent` · `priority:high` · `priority:medium` · `priority:low`

## Implementation

### Preconditions

- `gf` authenticated
- Sufficient scope to label Issues
- Single type label per Issue; single priority label per Issue

### Step 1: Fetch all open Issues — `issue list --state open`. Skip those already with `triage:done` (idempotent).

### Step 1b: Verify coverage — read `pagination.truncated` from the response.

If `truncated` is `true`, the fetch hit the limit and more Issues exist. Re-run with a
higher `--limit`, or state the partial coverage explicitly in the report. Never present
a truncated fetch as a complete classification.

### Step 2: Classify each Issue by title + description body

Optional Jev guidance: when `gf` was built with `gitflow-jev`, `GF_DECISION_PROVIDER=jev`
is set, and a TypeSafe key is available from `TYPESAFE_API_KEY` or the macOS
`ai.typesafe.api-key` Keychain item, fetch each Issue
through `gf issue view <n> --output json` and prepare one bounded JSON request for
`gf decide batch --input <file>`. Include only the Issue title and a short, reviewed
excerpt of its body. Remove credentials, tokens, personal data, private URLs, and
unrelated fields before transmission. Never send the raw Issue JSON. The request
must contain four independent questions over the same state:

Use the concrete request shape in [Optional Jev decisions](../../docs/jev-decision.md).

| ID | Type | Meaning |
|----|------|---------|
| `type` | Choice | `bug`, `feature`, `enhancement`, `docs`, `question`, `other` |
| `priority` | Score | ordered descriptions of low, medium, high, urgent impact |
| `security_related` | Noul | probability of a security issue |
| `blocked` | Noul | probability of work being blocked |

Treat the returned answers as suggestions for the existing classification steps.
The `priority` Score is a graded signal, not a direct `priority:*` label. Do not
apply any threshold until it is calibrated on labeled Issues in this repository.
Likewise, calibrate the `type`, `security_related`, and `blocked` thresholds
separately. Review uncertain cases manually. Never assign labels solely from Jev.
The [pilot evaluation](../../docs/jev-triage-evaluation-2026-09-22.md) has not
established a safe automatic acceptance threshold. Until separate thresholds
are validated, review every Jev suggestion before proposing a label.
If the feature is absent, the key is missing, input is too large, the request
times out, the response is malformed, or confidence is insufficient, continue
with the heuristics below. Do not paste the API key into chat or a file.

| Type | Heuristic |
|------|-----------|
| `type:bug` | reports crash / error / regression |
| `type:feature` | new capability / module |
| `type:enhancement` | UX / perf improvement |
| `type:docs` | missing / stale doc |
| `type:question` | question / discussion |

Keep existing type label if already correct. Mark `type:unknown` only when ambiguous.

### Step 3: Apply priority by impact

| Priority | Heuristic |
|----------|-----------|
| `priority:urgent` | production outage / security / blocked |
| `priority:high` | core feature defect / milestone-bound |
| `priority:medium` | general feature / UX |
| `priority:low` | nice-to-have / doc tweak |

Reference: core user path · affected user count · workaround · milestone proximity · security relevance.

### Step 4: Apply labels

```bash
gf issue add-label <n> --label "type:<t>" --label "priority:<p>" --label "triage:done"
```

### Step 5: Output report — priority-ranked (🔴 urgent → 🟢 low) with count + percentage tables.

### Report Output & Archiving

When persisted for audit trail (e.g. by `gf-workflow` Phase 4), save the report
to `docs/issue-triage-report-<YYYY-MM-DD>[-<context>].md`. Once
`issue-triage-report-*.md` files under `docs/` exceed 5, move all but the 5
most recent (ordered by the Issue/PR number embedded in the filename) into
`docs/reports-archive/<YYYY>-Q<N>/`, bucketed by each report's own date. See
`docs/index.md` → Reports Archive for the full policy.

### Error Handling

| Error | Recovery |
|-------|----------|
| Auth | Stop. `auth login`. |
| Label API failure | Skip Issue, continue. |
| Ambiguous type | Mark `type:unknown`; note in report. |
| Duplicate skip | Idempotent; safe. |

## Responsibility

### ✅ In Scope

- Fetch open Issues
- Assign one type + one priority
- Mark `triage:done`
- Output ranked report

### ❌ Out of Scope

- Requirement analysis → `/gf-issue-review`
- Label statistics → `/gf-label-stats`
- Editing Issue body → `/gf-issue`

### 🚫 Do Not

- ❌ Assign multiple type labels
- ❌ Speculate beyond available info
- ❌ Mark duplicate Issues as triaged — mark `duplicate` instead
- ❌ Take >2 min per Issue; triage fast

## Rationalization Excuses

| Excuse | Reality |
|--------|---------|
| "{rd", "just guess one" | Use `type:unknown`; don't fabricate. |
| "Skip labels — just report" | Label application is the deliverable. |
| "All issues are urgent" | Apply threshold; ≤10% should be urgent. |

## Red Flags

- 🚩 "Skip triage:done" — Always mark on completion.
- 🚩 "Just label everything urgent" — Apply priority thresholds.
- 🚩 "Infer details not in description" — Don't speculate. Use `type:unknown` or `priority:medium`.

## Test Scenarios

### 1: Happy Path
- **Given** 8 open Issues — **When** "triage all" — **Then** each Issue gets type+priority+`triage:done`; report with % tables returned.

### 2: Negative
- **Given** "analyze issue #42 depth" — **Then** NOT loaded. → `/gf-issue-review`.

### 3: Boundary
- **Given** "triage and also close duplicates" — **Then** triage only; label `duplicates` but do not close.

### 4: Error
- **Given** Issue not found — **Then** skip.

### 5: Idempotency
- **Given** second run — **Then** `triage:done` Issues skipped.

### 6: Optional Jev uncertainty
- **Given** a missing key, provider failure, malformed answer, or low-confidence
  Choice/Score result — **When** triage runs — **Then** continue with the manual
  classification steps, present uncertainty, and do not assign a label solely
  from the Jev response.

## Success Criteria

- [ ] Every open Issue has type + priority
- [ ] All tagged `triage:done`
- [ ] Report with counts + percentages
- [ ] Duplicates marked, not triaged

## Common Mistakes

- ❌ **Multiple type labels** — one per Issue.
- ❌ **All marked urgent** — apply thresholds strictly.

## See Also

- `gf-issue-review` — analyze requirement depth
- `gf-issue-decompose` — batch decomposition into dependency-ordered Issues
- `gf-label-stats` — label distribution statistics
- `gf-issue` — Issue CRUD reference
- `gf-label-milestone` — label CRUD
