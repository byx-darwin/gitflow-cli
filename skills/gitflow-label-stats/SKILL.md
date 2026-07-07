---
name: gitflow-label-stats
description: |
  Use when the user wants to analyze label usage statistics across issues — grouping
  issues by label, analyzing priority distribution, identifying unclassified issues,
  and producing a label health report.
  当用户想要按标签分组统计 Issue 数量、分析优先级分布、识别未分类 Issue、
  或输出标签统计报告时使用。
---

# gitflow-label-stats

Analyzes label usage across repository issues. Combines `gitflow-cli label list` with
per-label `gitflow-cli issue list --label` counts to produce a three-dimensional report:
label grouping, priority distribution, and unclassified-issue identification. Read-only —
never creates, edits, or deletes labels or issues.

## When to Use

| English | 中文 | Context |
|---------|------|---------|
| label statistics | 标签统计 | count issues per label |
| label usage report | 标签使用报告 | analyze label coverage |
| priority distribution | 优先级分布 | analyze priority tag spread |
| unclassified issues | 未分类 Issue | find issues missing type/priority tags |
| label health check | 标签健康度 | overall label management health |
| triage coverage | 分类覆盖率 | measure how many issues are fully tagged |
| issue taxonomy | Issue 分类体系 | audit label taxonomy consistency |

## Core Pattern

```bash
command -v gitflow-cli && git rev-parse --show-toplevel    # preconditions
gitflow-cli label list                                       # all label definitions
gitflow-cli issue list --label "<label>" --state open --limit 1000   # per-label count
gitflow-cli issue list --label "<label>" --state closed --limit 1000 # closed count
gitflow-cli issue list --state open --limit 1000             # unclassified detection
```

## Quick Reference

| Goal | Command |
|------|---------|
| List all labels | `gitflow-cli label list` |
| Count open issues for a label | `gitflow-cli issue list --label "<label>" --state open --limit 1000` |
| Count closed issues for a label | `gitflow-cli issue list --label "<label>" --state closed --limit 1000` |
| All open issues (unclassified scan) | `gitflow-cli issue list --state open --limit 1000` |

| Category | Common Labels | Purpose |
|----------|---------------|---------|
| Type | `bug`, `enhancement`, `documentation`, `question` | Issue type |
| Priority | `priority:urgent`, `priority:high`, `priority:medium`, `priority:low` | Priority level |
| Status | `triage:done`, `in-progress`, `blocked` | Workflow state |
| Platform | `github`, `gitlab`, `gitcode` | Platform tag |
| Special | `good-first-issue`, `help-wanted`, `auto-reported` | Special markers |

## Implementation

### Preconditions

- `command -v gitflow-cli` succeeds
- `git rev-parse --show-toplevel` succeeds (run inside a git repo)
- `gitflow-cli label list` returns non-empty (repo has labels defined)

### Step 1: Fetch Label Definitions

```bash
gitflow-cli label list
```

Capture for each label: name, color, description (if any). This is the universe of labels
to iterate over in Step 2.

**Empty result →** output "No labels defined in this repository"; stop.

### Step 2: Per-Label Issue Count

For every label from Step 1, run:

```bash
gitflow-cli issue list --label "<label>" --state open --limit 1000
gitflow-cli issue list --label "<label>" --state closed --limit 1000
```

Record open count, closed count, total, and percentage of overall open issues.

**Output table:**

```markdown
### Label Grouping

| Label | Open | Closed | Total | Share |
|-------|------|--------|-------|-------|
| bug | <n> | <n> | <n> | <x%> |
| enhancement | <n> | <n> | <n> | <x%> |
| ... | | | | |
```

> Note: an issue can carry multiple labels, so per-label totals may exceed the issue count.

### Step 3: Priority Distribution

Count open issues for each priority label:

```bash
gitflow-cli issue list --label "priority:urgent" --state open
gitflow-cli issue list --label "priority:high" --state open
gitflow-cli issue list --label "priority:medium" --state open
gitflow-cli issue list --label "priority:low" --state open
```

**Output table:**

```markdown
### Priority Distribution

| Priority | Open | Share | Health |
|----------|------|-------|--------|
| 🔴 urgent | <n> | <x%> | 🟢/🟡/🔴 |
| 🟠 high | <n> | <x%> | 🟢/🟡/🔴 |
| 🟡 medium | <n> | <x%> | 🟢 |
| 🟢 low | <n> | <x%> | 🟢 |
```

**Health thresholds:**

| Condition | Health | Meaning |
|-----------|--------|---------|
| urgent share < 10% | 🟢 Normal | Proportion is healthy |
| urgent share 10%–20% | 🟡 Watch | Too many urgent items |
| urgent share > 20% | 🔴 Alert | Priority inflation likely |
| urgent + high share > 50% | 🔴 Alert | Priority calibration needed |

### Step 4: Unclassified Issue Identification

```bash
gitflow-cli issue list --state open --limit 1000
```

For each issue, inspect its label set and classify:

| Category | Criteria | Suggested Action |
|----------|----------|------------------|
| Fully untagged | No labels at all | Run `gitflow-issue-triage` |
| Missing type | Has priority but no type label | Add type label |
| Missing priority | Has type but no priority label | Assess and add priority |
| Missing triage marker | Not tagged `triage:done` | May need triage |
| Fully classified | Has type + priority + triage:done | No action needed |

**Output table:**

```markdown
### Classification Coverage

| Metric | Count | Share |
|--------|-------|-------|
| Fully classified | <n> | <x%> |
| Missing type | <n> | <x%> |
| Missing priority | <n> | <x%> |
| Fully untagged | <n> | <x%> |
```

### Step 5: Render Report

Produce a single markdown report with the following structure:

```markdown
## Label Statistics Report

**Repository:** <owner>/<repo>
**Timestamp:** <timestamp>
**Total Open Issues:** <total>

### Label Grouping

| Label | Open | Closed | Total | Share |
|-------|------|--------|-------|-------|

### Priority Distribution

| Priority | Open | Share | Health |
|----------|------|-------|--------|

### Classification Coverage

| Metric | Count | Share |
|--------|-------|-------|

### Improvement Suggestions

1. <!-- specific, evidence-based suggestion -->
2. <!-- ... -->
```

### Step 6: Improvement Suggestions

Derive suggestions strictly from observed data. Map findings to actions:

| Finding | Suggestion |
|---------|-----------|
| Untagged + missing-type + missing-priority > 30% | Run `gitflow-issue-triage` on all open issues |
| Urgent share too high | Re-evaluate priority criteria; avoid defaulting to urgent |
| Single label backlog is large | Focus resources on that label |
| Inconsistent label naming | Propose unified naming; merge semantic duplicates |
| No `good-first-issue` | Tag some easy tasks for newcomers |
| Closed share very low | Accelerate issue resolution; clean stale issues |

## Error Handling

| Error | Recovery |
|-------|----------|
| `label list` empty | Output "No labels defined"; stop |
| `label list` non-zero exit | Output error; suggest checking auth/platform; stop |
| `issue list --label` empty for a label | Record count = 0; continue |
| `issue list --label` non-zero exit | Skip that label; note in report; continue |
| `issue list --state open` empty | Report has zero issues; skip unclassified step |
| `--limit 1000` truncated | Note in report that counts may be incomplete |

## Responsibility

### ✅ In Scope

- Fetch label definitions via `gitflow-cli label list`
- Count issues per label via `gitflow-cli issue list --label`
- Analyze priority distribution
- Identify unclassified / under-classified issues
- Render a markdown report with tables and suggestions

### ❌ Out of Scope

- Create, edit, or delete labels
- Add or remove labels on any issue
- Change issue state (close / reopen)
- Auto-create issues or PRs
- Push reports to Slack / email / external channels
- Modify label taxonomy without explicit user request

### 🚫 Do Not

- ❌ Create, edit, or delete any label
- ❌ Add or remove labels on any issue
- ❌ Close, reopen, or modify any issue
- ❌ Auto-create issues / PRs based on findings
- ❌ Send reports to external channels without explicit request
- ❌ Fabricate counts when `issue list` returns empty
- ❌ Infer label semantics beyond what `label list` returns

## Rationalization

| Excuse | Reality |
|--------|---------|
| "I'll auto-label the untagged issues while I'm here" | Out of scope; read-only analysis |
| "Let me create a missing label" | Out of scope; user decides |
| "I'll triage them now since I already have the data" | Redirect to `gitflow-issue-triage` |
| "Send the report to the team chat" | Out of scope; user decides |
| "No data — I'll estimate the counts" | Refuse; report "no data" and stop |
| "I'll fix the inconsistent labels" | Out of scope; user decides |

## Red Flags

- 🚩 "Auto-label the untagged issues" — refuse; read-only
- 🚩 "Create the missing priority labels" — refuse; out of scope
- 🚩 "Just fix the label naming" — refuse; user decides
- 🚩 "Skip the priority analysis" — non-skippable dimension
- 🚩 "No data, just estimate" — refuse; report honestly
- 🚩 "Send this to Slack" — refuse without explicit request

## Test Scenarios

### Scenario 1: Happy Path

- **Given** Repository with labels and ≥ 1 open issue
- **When** "帮我统计一下标签使用情况"
- **Then** Report with label grouping, priority distribution, classification coverage, and suggestions; no label or issue modified

### Scenario 2: Negative — Wants to Fix Labels

- **Given** "帮我修一下标签命名不一致的问题"
- **When** Targets label edits
- **Then** Do NOT load; redirect — skill is read-only

### Scenario 3: Boundary — Tempted to Auto-Label

- **Given** Report shows 40% untagged issues
- **When** Claude considers calling `gitflow issue label` to fix them
- **Then** Refuse; cite 🚫 Do Not; suggest running `gitflow-issue-triage` instead

### Scenario 4: Error — No Labels Defined

- **Given** Fresh repository with no labels
- **When** `gitflow-cli label list` returns empty
- **Then** Output "No labels defined in this repository"; stop; do not fabricate a report

## Success Criteria

- [ ] Label grouping table covers all labels from `label list`
- [ ] Priority distribution follows health thresholds
- [ ] Classification coverage identifies untagged / missing-type / missing-priority
- [ ] Improvement suggestions are evidence-based, not generic
- [ ] No label created, edited, or deleted
- [ ] No issue labeled, unlabeled, closed, or modified
- [ ] No report pushed to external channels
- [ ] Empty data handled with honest message, not fabrication

## Common Mistakes

- ❌ **Double-counting issues across labels** — note in report that totals may exceed issue count because issues can carry multiple labels
- ❌ **Treating `bug` and `type:bug` as the same label** — they are distinct unless `label list` shows an alias
- ❌ **Auto-suggesting label edits** — suggestions must be human-actionable, not self-executed
- ❌ **Skipping the unclassified scan** — all three dimensions are required
- ❌ **Using mean for priority health** — use share-of-total, not absolute counts

## Trigger Keywords

| English | 中文 |
|---------|------|
| label statistics | 标签统计 |
| label usage | 标签使用情况 |
| priority distribution | 优先级分布 |
| unclassified issues | 未分类 Issue |
| label health | 标签健康度 |
| triage coverage | 分类覆盖率 |
| issue taxonomy | Issue 分类体系 |
| label report | 标签报告 |

## See Also

- `gitflow-label-milestone` — create / edit / delete labels and milestones
- `gitflow-issue-triage` — classify and label untagged issues
- `gitflow-issue` — issue CRUD operations
- `gitflow-weekly-report` — cites label statistics in weekly report
