---
name: gf-workflow-batch
description: |
  Use when the user wants to batch-process multiple open Issues through the gf-workflow four-phase gate, or invokes `/gf-workflow-batch`. Serial-only, stateless: progress is derived from `.cache/workflows/` on disk every round, never kept in conversation memory.
  当用户希望串行批量对多个 open Issue 依次执行 gf-workflow 四阶段流程，或调用 `/gf-workflow-batch` 时使用。
---

# gf-workflow-batch

Serial outer driver for `gf-workflow`. Dispatches one fresh subagent per open
Issue via the `Agent` tool (never `fork`), blocks until it finishes
(including its Gate 2→3 approval pause), records a one-line summary, then
re-derives the next Issue from disk. Never modifies `gf-workflow` itself.

## CLI Requirement

**MUST use `gf` CLI, NOT `gh` CLI.** `gf` is the unified multi-platform CLI
for this project (GitHub + GitLab + GitCode); `gh` is GitHub-only.

## Preconditions

- `gf` installed: `command -v gf`
- `gf` authenticated: `gf auth status`
- `superpowers:brainstorming` and `gf-issue-create` available (needed only
  when `pending` is empty — see Discussion Mode below)

## When to Use

| English | 中文 | Trigger Context |
|---------|------|-----------------|
| batch process issues | 批量处理 issue | multiple open Issues need full gf-workflow runs |
| run gf-workflow on all open issues | 对所有 open issue 跑 gf-workflow | user wants serial end-to-end automation |
| process the issue backlog | 处理 issue 积压 | backlog clearing, one Issue at a time |
| decompose this into issues | 把这个拆成多个 issue | no pending Issue, user has a large ask |

## Trigger Keywords

| English | 中文 |
|---------|------|
| batch workflow | 批量工作流 |
| process all issues | 处理所有 issue |
| serial driver | 串行驱动器 |

## Core Pattern

```bash
gf issue list --state open --output json
# pending = open issues NOT covered by any active/*.json (status != complete)
#           NOR by any archive/**/*.json contract
# if pending empty → Discussion Mode (see references.md)
# else → dispatch Agent(prompt: "/gf-workflow #<n>") for pending[0], serially
```

## Implementation

Each round: compute `pending`; empty triggers Discussion Mode, then
recompute. Otherwise dispatch `pending[0]` via `Agent` (never `fork`),
block until it returns (including Gate 2→3), append a summary line, and
loop. Stop and print the summary table once `pending` is empty and
Discussion Mode already ran with nothing left to create. Full algorithm:
see `references.md`.

### Parameters

- `--limit N` — stop dispatching after N Issues have been processed this run.
- `--label <label>` — restrict candidate Issues to those carrying `<label>`.

## Responsibility

### ✅ In Scope

- Compute `pending` Issues from disk each round
- Dispatch one `/gf-workflow` subagent at a time, block on completion
- Trigger Discussion Mode + `gf-issue-create` when `pending` is empty
- Print run summary

### ❌ Out of Scope

- Any change to `gf-workflow`'s own phases, gates, or contract schema
- Parallel dispatch (explicitly out of scope — spec §Non-Goals)
- Requirement quality analysis → `/gf-issue-review`
- Bulk labeling/classification → `/gf-issue-triage`

### 🚫 Do Not

- ❌ Dispatch the next Issue before the current one's subagent returns
- ❌ Auto-approve a Gate 2→3 pause on the user's behalf
- ❌ Keep batch progress in conversation memory instead of re-deriving it
- ❌ Dispatch via `fork` (forks inherit this conversation's history)

## Rationalization Excuses

| Excuse | Reality |
|--------|---------|
| "Dispatch two at once, it's faster" | Out of scope — base branch drift, interleaved approvals. |
| "I already have the pending list from last round" | Recompute from disk every round — stale lists cause duplicate dispatch. |
| "This one failed, abort the whole batch" | Failures are isolated — record and continue. |

## Red Flags

- 🚩 About to dispatch a second `Agent` call before the first returned — STOP, this driver is serial-only.
- 🚩 About to reuse a `pending` list computed in an earlier round — STOP, recompute from disk.
- 🚩 About to use `subagent_type: "fork"` — STOP, use a fresh (non-fork) dispatch.

## Test Scenarios

### 1: Happy Path
- **Given** 3 open Issues, none covered by any contract — **When** `/gf-workflow-batch` runs — **Then** 3 sequential `Agent` dispatches of `/gf-workflow #<n>`, one at a time, summary table with 3 rows.

### 2: Negative
- **Given** "review issue #42's requirement quality" — **Then** NOT loaded → `/gf-issue-review`.

### 3: Boundary
- **Given** all pending Issues dispatched, one subagent fails at Gate 2→3 (user rejects) — **Then** driver records it as `rejected` and continues to the next pending Issue, does not abort the batch.

### 4: Error
- **Given** `gf auth status` fails — **Then** stop before computing `pending`, prompt `gf auth login`.

### 5: Boundary
- **Given** zero open Issues (or zero uncovered ones) — **When** `/gf-workflow-batch` runs — **Then** enters Discussion Mode: `superpowers:brainstorming` then `gf-issue-create` per sub-task, then recomputes `pending` and continues the dispatch loop.

## See Also

- `/gf-workflow` — the four-phase engine this driver dispatches, once per Issue
- `/gf-issue-create` — creates Issues in Discussion Mode
- `/gf-issue-review` — Issue requirement analysis (not this skill's job)
- `docs/superpowers/templates/skill-conventions.md` — skill conventions
