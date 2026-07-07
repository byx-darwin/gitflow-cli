---
name: gitflow-issue-triage
description: |
  Use when the user asks to triage, categorize, or prioritize open issues, or requests a backlog health report.
  当用户对 open issues 进行分类、分流、优先级评估，或要求待办全景报告时使用。
---

# gitflow-issue-triage

Classifies open issues by type/priority, applies triage labels, emits a report. Does not close, merge, or edit issue content.

## Overview

Triage loop: fetch open issues → classify by type → evaluate priority → apply labels → emit report. Idempotent — re-runs skip issues already bearing `triage:done`.

## When to Use

| English | 中文 | Fire? |
|---------|------|-------|
| triage / categorize backlog | 分类 / 整理待办 | ✅ |
| prioritize / backlog health | 优先级排序 / 待办全景 | ✅ |
| label issues | 打标签 / issue 报告 | ✅ |
| close all low-priority | 关闭低优先级 | ❌ → `gitflow-issue` |

## Core Pattern

```bash
gitflow-cli auth status                # 1. preconditions
gitflow-cli issue list --state open    # 2. fetch
gitflow-cli issue label <n> --add "type:<t>" --add "priority:<p>" --add "triage:done"  # 3. per-issue
gitflow-cli issue list --label "triage:done" --state open  # 4. verify
```

## Quick Reference

| Goal | Command |
|------|---------|
| List open | `gitflow-cli issue list --state open` |
| Add type / priority | `gitflow-cli issue label <n> --add "type:<t>" --add "priority:<p>"` |
| Mark triaged | `gitflow-cli issue label <n> --add "triage:done"` |

## Implementation

### Preconditions

`command -v gitflow-cli` / `gitflow-cli auth status` / `git rev-parse --is-inside-work-tree`.

### Step 1: Fetch

`gitflow-cli issue list --state open` — empty → "No open issues", stop.

### Step 2: Classify (per issue missing `triage:done`)

Assign one type + one priority:

| Type | Signal | Priority | Signal |
|------|--------|----------|--------|
| bug | crash / error | urgent | outage / security |
| feature | new capability | high | core bug / milestone |
| enhancement | UX/perf | medium | standard |
| docs / question / unknown | unclear | low | nice-to-have |

Existing type label → keep. Ambiguous → unknown + medium.

### Step 3: Label (Idempotent)

`--add` only; never `--remove`. triage:done issues skipped; re-runs process only new issues.

### Step 4: Report

Totals, type %, priority %, detail table, action items. Use triage:done URLs as evidence.

### Error Handling

| Error | Recovery |
|-------|----------|
| Auth non-zero | Stop. Direct user to auth login. |
| Rate-limit | Wait 60s, retry once; else report partial. |
| Label fails for one issue | Log, continue. |
| Empty list | "No open issues to triage", stop. |

## Responsibility

### ✅ In Scope

- Fetch, classify, label, skip triaged, report

### ❌ Out of Scope

- Close/edit → `gitflow-issue`; assign → `gitflow-issue-create`; bulk ops → manual

### 🚫 Do Not

- ❌ Overwrite existing labels
- ❌ Triage closed issues
- ❌ Close, merge, or modify issues
- ❌ Assign without instruction
- ❌ Invent priority; uncertain → medium + unknown

## Rationalization Excuses

| Excuse | Reality |
|--------|---------|
| "I'll close low-priority issues to clean up" | Out of scope. Triage only labels. |
| "The existing type label looks wrong, I'll fix it" | Do not overwrite; flag in report. |
| "Skip auth — we just did it" | Invocations independent. Preconditions always run. |

## Red Flags

- 🚩 "Skip the auth check" — Refuse.
- 🚩 "Close the low-priority ones" — Refuse; redirect to `gitflow-issue`.
- 🚩 "Remove type:unknown labels" — Refuse.
- 🚩 Tool fails, Claude improvises — Follow Error Handling.

## Test Scenarios

### Scenario 1: Happy Path

- **Given** 5 open issues, none triaged, auth valid
- **When** user says "triage open issues"
- **Then** each issue gets type, priority, triage:done; report lists all 5.

### Scenario 2: Negative — "close all low-priority issues"

- **Then** Not loaded. Redirect to `gitflow-issue`.

### Scenario 3: Boundary — Issue has existing `type:enhancement`, user says "overwrite it"

- **Then** Refuses. Keeps label, --add only, flags in report.

### Scenario 4: Error — `issue label` returns 500 for #2 of 4

- **Then** Logs failure, continues, notes in report.

## Success Criteria

- [ ] Each untriaged issue gets one type + one priority
- [ ] No labels removed or overwritten
- [ ] Report matches issue list counts

## Common Mistakes

- ❌ **Overwriting existing type labels** — Keep; never `--remove`.
- ❌ **Triaging closed issues** — `--state open` mandatory; wider scope misleads.

## Trigger Keywords

| English | 中文 |
|---------|------|
| triage / issue triage | 分类 / 分流 issues |
| prioritize backlog | 优先级排序 / 待办排序 |
| backlog health report | 待办全景 / issue 报告 |
| label issues / categorize | 标记分类 / 整理 issues |

## See Also

- `gitflow-issue` — Issue CRUD (close, reopen, edit, comment)
- `gitflow-issue-review` — Structured issue review
- `gitflow-label-milestone` — Label and milestone management
