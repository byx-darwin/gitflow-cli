---
name: gitflow-label-stats
description: |
  Use when the user wants to analyze label usage statistics across issues — grouping
  issues by label, analyzing priority distribution, identifying unclassified issues,
  and producing a label health report.
---

# gitflow-label-stats

Read-only analysis of label usage. Produces a three-dimensional report: label grouping, priority distribution, and unclassified-issue identification. Never creates, edits, or deletes labels or issues.

## Trigger Keywords

| English | 中文 |
|---------|------|
| label statistics / 标签统计 | label usage / 标签使用报告 |
| priority distribution / 优先级分布 | unclassified issues / 未分类 Issue |
| label health / 标签健康度 | triage coverage / 分类覆盖率 |

## Command Overview

| Goal | Command |
|------|---------|
| List all labels | `gitflow-cli label list` |
| Open issues for a label | `gitflow-cli issue list --label "<label>" --state open --limit 1000` |
| Closed issues for a label | `gitflow-cli issue list --label "<label>" --state closed --limit 1000` |
| All open issues (unclassified) | `gitflow-cli issue list --state open --limit 1000` |

## Workflow

1. **Preconditions**: `gitflow-cli` available, inside git repo, `label list` non-empty.
2. **Fetch labels** → name, color, description.
3. **Per-label count** → open/closed for each label; record share.
4. **Priority distribution** → count open for `priority:urgent|high|medium|low`.
5. **Unclassified scan** → classify each open issue: untagged, missing-type, missing-priority, missing-triage, or fully-classified.
6. **Render report** → markdown with all three tables + evidence-based suggestions.

### Priority Health

| Condition | Health |
|-----------|--------|
| urgent < 10% | 🟢 Normal |
| urgent 10–20% | 🟡 Watch |
| urgent > 20% or urgent+high > 50% | 🔴 Alert |

### Suggestions

| Finding | Action |
|---------|--------|
| untagged+missing > 30% | Run `gitflow-issue-triage` |
| urgent share high | Re-evaluate priority criteria |
| large single-label backlog | Focus resources |
| inconsistent naming | Propose unified naming |

## Examples

- "帮我统计标签使用情况" → full 3-dimension report
- "帮我修标签命名" → refuse; read-only; redirect to `gitflow-label-milestone`

## Common Mistakes

- ❌ Double-counting across labels — issues can have multiple labels
- ❌ Treating `bug` and `type:bug` as same — distinct unless aliased
- ❌ Auto-labeling — read-only; suggest `gitflow-issue-triage`
- ❌ Skipping unclassified scan — all three dimensions required
- ❌ Fabricating counts on empty data — report honestly and stop

## Error Handling

| Error | Recovery |
|-------|----------|
| `label list` empty | "No labels defined"; stop |
| `label list` fails | Check auth; stop |
| per-label `issue list` empty | Count=0; continue |
| per-label `issue list` fails | Skip; note in report |

## See Also

`gitflow-label-milestone` · `gitflow-issue-triage` · `gitflow-issue` · `gitflow-weekly-report`
