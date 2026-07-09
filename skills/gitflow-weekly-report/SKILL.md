---
name: gitflow-weekly-report
description: |
  Use when the user wants a weekly/biweekly dev-report summarizing commits across one or more repos.
  当用户需要按类型汇总一个或多个仓库的提交、生成研发周报时使用。
---

# gitflow-weekly-report — Read-Only Aggregator

Read-only Git log scan → group by project + type → plain-text weekly report. Template: [`docs/templates/weekly-report-template.md`](docs/templates/weekly-report-template.md) · Quality thresholds: [`docs/references/gitflow-quality-params.md`](docs/references/gitflow-quality-params.md)

## When to Use

| EN | ZH |
|----|----|
| weekly report 周报 | 研发总结 |
| multi-repo 多项目 | 跨仓汇总 |
| rate my output | 拒绝 |

## Core Pattern

```bash
git log --format="%h %ai %s" --since="<s>" --until="<u>"
grep -E '^(feat|fix|refactor|docs|chore|ci|test):'
```

## Quick Reference

| Goal | Tool |
|------|------|
| Fetch log | `git log --format="%h %ai %s" --since <s> --until <u>` |
| Count commits | `git log --format="%h" --since <s> --until <u> \| wc -l` |
| Diff stat | `git diff --stat --since <s> --until <u> \| tail -1` |
| Classify | grep conventional prefix |

## Implementation

### Preconditions

Valid paths (skip invalid) · window derived from the cutoff date · year matches the commit year.

### Steps

1. **Window** — derive `--since`/`--until` from the cutoff date
2. **Scan** — `git log` hash + date + subject
3. **Classify & merge** — group `feat`/`fix`/`refactor`/`docs`/`chore|ci|test` into one sentence
4. **Render** — weekly-report-template.md; backtick hashes; Chinese output

### Error Handling

| Error | Recovery |
|-------|----------|
| Invalid path | Skip |
| No commits | Write `"暂无提交"` |
| Cross-year | Full ISO |
| Single commit | Still full template |
| Rating requested | Refuse |

## Responsibility

### ✅ In Scope

Read-only across N repos · prefix-based classification · template + real counts.

### ❌ Out of Scope

Modifying any repo · scoring · tables.

### 🚫 Do Not

❌ Fabricate commits · ❌ Judge performance · ❌ Omit sections.

## Rationalization

| Excuse | Reality |
|--------|---------|
| "Estimates are good enough" | Must be exact via `wc -l` |
| "Add a performance score" | Out of scope |
| "Little content, so omit sections" | Full template required |

## Red Flags

🚩 "Give me a score" — Refuse · 🚩 "Rate productivity" — Out of scope · 🚩 "Round the numbers" — Be exact

## Common Mistakes

❌ Fabricating commits when there are none · ❌ Reusing prior-year dates · ❌ Omitting fragments

## Trigger Keywords

| EN | ZH |
|----|----|
| weekly report recap | 周报 本周总结 |
| multi-repo summary | 多项目汇总 |

## Test Scenarios

### 1: Happy
Three repos with commits → classify & merge; full plain text.

### 2: Negative
"rate my output" → refuse to score.

### 3: Boundary
No commits → write `"暂无提交"`; count 0.

### 4: Error
`/nope` + one valid path → skip; do not abort entirely.

## Success Criteria

- [ ] All counts come from `git log`
- [ ] Plain text is complete
- [ ] No performance judgment
- [ ] Invalid paths skipped

## See Also

`/gitflow-workflow` — Phase 4 trigger
`/gitflow-pipeline-analyzer` — CI health
`/gitflow-commit` — single commit
`/gitflow-label-milestone` — milestones
