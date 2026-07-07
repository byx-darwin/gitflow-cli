---
name: gitflow-label-stats
description: |
  Use when the user wants Issue label statistics — group counts by label, priority health, and unclassified Issue identification.
  当用户希望分析 Issue 标签分布（按标签分组计数、优先级分布、未分类 Issue 识别）时使用。
---

# gitflow-label-stats

Read-only label analytics. Queries `gitflow-cli label list`, then `gitflow-cli issue list --label` per label and per priority. Produces a unified report: label group counts, priority distribution with health indicators, and unclassified Issue identification. Propose fixes — never mutate.

See [full label taxonomy](../references/gitflow-label-stats-taxonomy.md) for canonical label category reference.

## When to Use

| English | 中文 | Context |
|---------|------|---------|
| show label statistics | 显示标签统计 | health check |
| label distribution | 标签分布 | backlog grooming |
| unclassified issues | 未分类 Issue | triage backlog |
| triage all unclassified | 对所有未分类 triage | **NOT** → `gitflow-issue-triage` |
| delete unused label | 删除无用标签 | **NOT** — read-only skill |
| modify issue labels | 修改标签 | **NOT** → `gitflow-issue-triage` |

## Core Pattern

```bash
gitflow-cli label list
gitflow-cli issue list --label "<l>" --state open --limit 1000
gitflow-cli issue list --state open --limit 1000
```

## Quick Reference

| Goal | Command |
|------|------|
| All labels | `gitflow-cli label list` |
| Filter by label | `gitflow-cli issue list --label "<l>" --state open [--limit 1000]` |
| All open Issues | `gitflow-cli issue list --state open --limit 1000` |

## Implementation

### Preconditions

- `gitflow-cli` authenticated
- Read-only; no Issue/label mutation

### Step 1–6 Summary

1. Load labels via `label list`; capture name/color/description.
2. Per label: `issue list --label "<l>" --state open --limit 1000`; record open + closed; share = total/Σ.
3. Priority health: `issue list --label "priority:<p>" --state open` — thresholds: <10% urgent 🟢 · 10–20% 🟡 · >20% 🔴.
4. Unclassified: load all open; compare labels vs taxonomy.

| Category | Action |
|----------|--------|
| Fully unlabeled | → `gitflow-issue-triage` |
| Missing type | add type |
| Missing priority | add priority |

5. Report: priority-ranked, with suggested actions.
6. Propose (never mutate):

| Finding | Recommendation |
|---------|----------------|
| unlabeled > 30% | run `gitflow-issue-triage` |
| urgent share high | recalibrate |
| pile-up | clear backlog |

### Error Handling

| Error | Recovery |
|-------|----------|
| Auth failure | Stop. `auth login`. |
| Label API failure | Skip label; continue. |
| >1000 Issues | Paginate with `--limit` + `--page`. |

## Responsibility

### ✅ In Scope

- Label list + counts
- Priority health scoring
- Unclassified Issue identification
- Improvement recommendations (text only)

### ❌ Out of Scope

- Mutating labels / Issues → `gitflow-issue-triage`
- Classifying Issues → `gitflow-issue-triage`
- Cleaning up labels → suggest + `gitflow-issue-triage`

## Rationalization Excuses

| Excuse | Reality |
|--------|---------|
| "Just fix the labels while we're here" | Read-only skill — output recommendations only. |
| "Probable count is close enough" | Always use exact `issue list` output; no estimation. |

## Red Flags

- 🚩 "Just update the labels" — Stop; out of scope.
- 🚩 "Estimate the count" — Always use exact CLI output.

## Test Scenarios

- **Happy**: Given "label stats" — When pipelines execute — Then grouped table + health + unclassified + recommendations.
- **Negative**: Given "label issue #42 as bug" — Then NOT loaded. → `gitflow-issue-triage`.
- **Boundary**: Given "stats then fix unlabeled" — Then output report only; redirect → `gitflow-issue-triage`.
- **Idempotency**: Run twice — identical outputs.

## Success Criteria

- [ ] All labels counted
- [ ] Priority health scored with thresholds applied
- [ ] Unclassified Issues identified with categories
- [ ] No mutation performed

## Common Mistakes

- ❌ **Double-counting** — each label standalone; note overlaps.
- ❌ **Inconsistent normalization** — always map synonyms.

## See Also

- `gitflow-issue-triage` — apply labels based on stats findings
- `gitflow-label-milestone` — label CRUD operations
