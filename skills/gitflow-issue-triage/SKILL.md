---
name: gitflow-issue-triage
description: |
  Use when the user wants to classify all open Issues by type and priority, then apply triage:done tags.
  当用户希望对所有 open Issue 按类型/优先级分类并打上 triage:done 标签时使用。
---

# gitflow-issue-triage

Batch classification of all open Issues — assigns one `type:*` label and one `priority:*` label per Issue, then marks `triage:done`. Outputs a priority-ranked report. Idempotent — skip already-triaged Issues.

## When to Use

| English | 中文 | Context |
|---------|------|---------|
| triage all issues | 对全部分类 | backlog grooming |
| classify issues | 分类 Issue | sprint planning |
| new issues since ... | 对近期新增分类 | `--since` flag |
| analyze an issue's requirement | 分析需求质量 | **NOT** → `/gitflow-issue-review` |
| label statistics | 标签统计 | **NOT** → `/gitflow-label-stats` |

## Core Pattern

```bash
gitflow-cli issue list --state open [--since <date>]
gitflow-cli issue label <n> --label "type:<t>" --label "priority:<p>" --label "triage:done"
```

## Quick Reference

| Goal | Command |
|------|---------|
| List open | `gitflow-cli issue list --state open [--since <date>]` |
| Add label | `gitflow-cli issue label <n> --label "<l>"` |
| Filter by label | `gitflow-cli issue list --label "<l>" --state open` |

**Type labels:** `type:bug` · `type:feature` · `type:enhancement` · `type:docs` · `type:question`
**Priority labels:** `priority:urgent` · `priority:high` · `priority:medium` · `priority:low`

## Implementation

### Preconditions

- `gitflow-cli` authenticated
- Sufficient scope to label Issues
- Single type label per Issue; single priority label per Issue

### Step 1: Fetch all open Issues — `issue list --state open [--since <date>]`. Skip those already with `triage:done` (idempotent).

### Step 2: Classify each Issue by title + description body

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
gitflow-cli issue label <n> --label "type:<t>" --label "priority:<p>" --label "triage:done"
```

### Step 5: Output report — priority-ranked (🔴 urgent → 🟢 low) with count + percentage tables.

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

- Requirement analysis → `/gitflow-issue-review`
- Label statistics → `/gitflow-label-stats`
- Editing Issue body → `/gitflow-issue`

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
- **Given** "analyze issue #42 depth" — **Then** NOT loaded. → `/gitflow-issue-review`.

### 3: Boundary
- **Given** "triage and also close duplicates" — **Then** triage only; label `duplicates` but do not close.

### 4: Error
- **Given** Issue not found — **Then** skip.

### 5: Idempotency
- **Given** second run — **Then** `triage:done` Issues skipped.

## Success Criteria

- [ ] Every open Issue has type + priority
- [ ] All tagged `triage:done`
- [ ] Report with counts + percentages
- [ ] Duplicates marked, not triaged

## Common Mistakes

- ❌ **Multiple type labels** — one per Issue.
- ❌ **All marked urgent** — apply thresholds strictly.

## See Also

- `gitflow-issue-review` — analyze requirement depth
- `gitflow-label-stats` — label distribution statistics
- `gitflow-issue` — Issue CRUD reference
- `gitflow-label-milestone` — label CRUD
