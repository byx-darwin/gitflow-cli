---
name: gf-workflow
description: |
  Use when the user wants a mandatory four-phase gated workflow with
  contract verification between phases, or invokes `/gf-workflow`.
  Enforces: clarify → plan → execute → deliver with JSON state tracking.
  当用户需要强制执行的四阶段闸门驱动全流程时使用。
---

# gf-workflow — Contract-Driven Four-Phase Gated Orchestrator

Orchestrator commands only; state lives in the contract; gates are never skipped.

> **⚠️ ORCHESTRATOR MANDATE**
>
> This skill is an **ORCHESTRATOR**, not a sub-skill. When invoked, it drives a
> four-phase pipeline end-to-end. The orchestrator **retains control** at all times.
> Sub-skills (`brainstorming`, `writing-plans`, `subagent-driven-development`, etc.)
> are **called and return** — they do NOT take over the conversation.
>
> **Violating the letter of these rules is violating the spirit of these rules.**
> No "I'm following the spirit" rationalizations. The rules are explicit for a reason.

## Core Rule: Contract First

**Before ANY phase executes, the orchestrator MUST:**

1. **Check for active contracts** — list `.cache/workflows/active/*.json`
   - Incomplete workflow exists (`status != "complete"`) → **RESUME** it: read `current_phase`, load context, continue from next step
   - Multiple exist → ask user which to resume
   - None exist → proceed to step 2
2. Run mode auto-detection (full / standard / fast)
3. Create the contract file at `.cache/workflows/active/<workflow_id>.json` (schema: `contract.schema.json`)
4. Announce the workflow start with: workflow_id, mode, title

**If no contract exists, no sub-skill may be invoked.** The contract is the
single source of truth for the workflow's state.

### Cross-Session Resume

When resuming an existing contract, load context based on `current_phase`:

| Phase | Context to Load | Resume From |
|-------|----------------|-------------|
| 1 | `design_doc_path` (if exists) | Next uncompleted step in Phase 1 |
| 2 | `design_doc_path` + `spec_path` | Gate 2→3 pause (await user approval) |
| 3 | `spec_path` (plan doc) | Next step after last evidence |
| 4 | `pr_url` + review reports | Next check in Phase 4 |

Full recovery procedure: see `references.md` → Cross-Session Recovery.

## Sub-Skill Invocation Rules

| Rule | Description |
|------|-------------|
| **Call and Return** | After invoking a sub-skill, the orchestrator MUST resume at the next step. Sub-skills do NOT chain to other skills. |
| **Brainstorming Override** | When `brainstorming` is called as a Phase 1 sub-skill, its terminal state is **RETURN TO ORCHESTRATOR** (not `writing-plans`). The orchestrator handles the transition to `gf-issue-create`. |
| **Single Active Orchestrator** | Only this workflow's state machine drives the conversation. No other skill may claim orchestration while a contract is active. |
| **Evidence Before Gate** | A gate check MAY NOT pass until all required evidence fields are populated. |
| **No Implicit Completion** | A Phase is complete ONLY when the orchestrator sets `status = "complete"` in the contract. Sub-skill completion ≠ Phase completion. |

## Smart Subagent Batching

### Complexity Scoring

```python
def classify_task_complexity(task):
    score = 0
    score += len(task.files_changed) * 1
    score += 3 if task.crosses_module_boundary else 0
    score += 2 if task.changes_public_api else 0
    score += 1 if task.requires_migration else 0

    if score <= 2:
        return "simple"    # batch
    elif score <= 6:
        return "medium"    # independent subagent
    else:
        return "complex"   # independent subagent + extra review
```

### Execution by Complexity

| Complexity | Method | Description |
|-----------|--------|-------------|
| Simple (score ≤ 2) | Batch in main agent | Implement all tasks in main agent, single review pass |
| Medium (score 3-6) | Independent subagent | One subagent per task + TDD + review |
| Complex (score > 6) | Independent subagent + extra review | One subagent per task + TDD + review + extra scrutiny |

### Batch Execution Flow (Simple Tasks)

```
Phase A: Batch Implement (main agent)
  ├── task_1: RED → GREEN
  ├── task_2: RED → GREEN
  └── task_3: RED → GREEN

Phase B: Batch Review (single subagent reviews all changes)

Phase C: Fix (if needed, main agent addresses findings)
```

### Mode-Batch Defaults

| Mode | Default Behavior |
|------|------------------|
| fast | Lean toward batch (changes usually simple) |
| standard | Score-based decision |
| full | Lean toward independent subagent (changes usually complex) |

User can override batching strategy during plan phase.

## Red Flags — STOP and Reassert Control

| Red Flag | Action |
|----------|--------|
| About to invoke `brainstorming` without a contract | **STOP** — create contract first |
| About to create a new contract when an active one exists | **STOP** — resume the existing contract instead |
| `brainstorming` starts invoking `writing-plans` | **STOP** — interrupt, return to orchestrator, execute `gf-issue-create` |
| About to skip `gf-issue-create` or `gf-issue-review` | **STOP** — MANDATORY in Phase 1 |
| About to advance without updating contract evidence | **STOP** — update contract first |
| User says "just write the code" | **CHECK** — Scenario C? If no contract, refuse and start Phase 1 |
| About to let a sub-skill chain to another | **STOP** — sub-skills return to orchestrator |

## Rationalization Table

| Excuse | Reality |
|--------|---------|
| "brainstorming will handle Issue creation" | No — brainstorming chains to `writing-plans`, not Issue creation. Orchestrator must do it. |
| "Contract can be created later" | No — contract MUST exist before any sub-skill. It is the single source of truth. |
| "User just wants to discuss" | If they invoked `/gf-workflow`, run the workflow. |
| "Issue review is optional" | No — `gf-issue-review` is MANDATORY in both full and fast modes. |
| "Brainstorming asked questions, Phase 1 is done" | No — brainstorming is ONE step. Issue list/create/review are separate mandatory steps. |
| "Requirement is clear, skip to Phase 3" | Scenario C. If `phases.2.evidence.spec_path` is empty, refuse and go to Phase 2. |
| "New session, start fresh" | No — check `.cache/workflows/active/` first. If incomplete contract exists, resume it. |
| "Different agent should start over" | No — contract is agent-agnostic. Any agent can resume from `current_phase` + evidence. |

## When to Use

| EN | ZH |
|----|----|
| full workflow | 全流程（默认） |
| clarify → plan → execute → deliver | 需求→计划→执行→交付 |

**When NOT to Use:** quick fix → `gf-commit` · PR review → `gf-pr-review` · architecture discussion → `superpowers:brainstorming` directly · user says "don't create an Issue" → do NOT invoke.

**Mode auto-detection:** "fix"/"typo"/"hotfix"/"docs"/"chore" → `fast` · "refactor: small"/"fix: bug" → `standard` · "feat"/"refactor: large"/breaking → `full` · `good-first-issue` label → `fast` · unclear → `standard` (default). User can override with `--mode <mode>`.

## Mode Comparison

| Phase | Full Mode | Standard Mode | Fast Mode |
|-------|-----------|---------------|-----------|
| 1 | brainstorming + issue-create + issue-review | brainstorming + issue-create + issue-review | issue-create (required), brainstorming (optional) |
| 2 | writing-plans + quality gate | writing-plans + quality gate | **skippable** |
| 3 | subagent-driven-development (TDD + Code Review) | subagent-driven-development (TDD + Code Review) | **required** |
| 4 | pipeline + triage + review + dogfooding | pipeline + review | pipeline + branch-finish |

## Mode Auto-Detection

Detection priority (highest to lowest):

1. **User explicit override** → `--mode fast` / `--mode standard` / `--mode full`
2. **Issue labels** → `good-first-issue` / `kind/typo` → fast; `kind/feature` → full
3. **Issue title prefix** → conventional commit format:
   - `fix: typo`, `docs:`, `chore:`, `hotfix` → **fast**
   - `fix:`, `refactor:`, `perf:` (single file/module) → **standard**
   - `feat:`, `refactor:` (cross-module), `!` (breaking change) → **full**
4. **Default** → **standard** (balanced safety vs efficiency)

### Confirmation Flow

```
检测到 `refactor(skills)` 前缀 → 建议 standard 模式
自动检测结果：standard
是否确认？[Y/n/override]
```

User input:
- `Y` / Enter → accept suggested mode
- `n` → enter mode selection menu (fast/standard/full)
- `fast` / `standard` / `full` → direct override

## Fast Mode — Required Skills Checklist

In fast mode, the following skills are invoked per phase:

**Phase 1:** `gf-issue-create` (required), `superpowers:brainstorming` (optional)

**Phase 2:** `superpowers:writing-plans` (optional, skippable)

**Phase 3:** `superpowers:subagent-driven-development` with TDD + Code Review (required)

**Phase 4:** `gf-pipeline-analyzer` → `gf-issue-triage` → `gf-review` → dogfooding checklist (all required)

## Standard Mode — Required Skills Checklist

In standard mode, the following skills are invoked per phase:

**Phase 1:** `superpowers:brainstorming` (required), `gf-issue-create` (required), `gf-issue-review` (required)

**Phase 2:** `superpowers:writing-plans` (required) + `gf-quality` gate (required)

**Phase 3:** `superpowers:subagent-driven-development` with TDD + Code Review (required)

**Phase 4:** `gf-pipeline-analyzer` → `gf-review` → Branch Finish (all required)

## State Machine

```
[Start] → Bootstrap → Phase 1 → [Gate 1→2] → AUTO → Phase 2 → [Gate 2→3] → PAUSE → Phase 3 → [Gate 3→4] → AUTO → Phase 4 → [Archive] → [Complete]
```

**Single pause point:** Gate 2→3 (plan approval). All other transitions auto-advance.

## Gate Rules

Full definitions: `skills/gf-workflow/gates.md`

| Enter Phase | Required evidence | fast-mode exemption |
|-------------|-------------------|---------------------|
| 2 (Planning) | `issue_url` + `comment_id` + `design_doc_path` | `comment_id` optional |
| 3 (Execution) | `spec_path` + `user_approved` | ✅ Skippable |
| 4 (Delivery) | `pr_url` + `tests_passed` | — |

## Phase 1: Clarification (Critical — Issue Interaction)

**Entry:** contract MUST exist · **Exit:** `phases.1.status = complete` · **Auto-advance:** yes

1. **[AUTO] Bootstrap** — Create contract at `.cache/workflows/active/<workflow_id>.json`
   - Set `mode`, `title`, `current_phase = 1`, `phases.1.status = "in_progress"`

2. **[AUTO] Read Open Issues**
   - User specified an Issue → use it
   - Otherwise → `gf issue list --state open`

3. **[CALL] `superpowers:brainstorming`**
   - Pass: Issue description or user requirements
   - **⚠️ RETURN RULE:** Terminal state = **RETURN TO ORCHESTRATOR** (not `writing-plans`)
   - Brainstorming will: explore context → ask questions → propose approaches → present design → write spec → **return control**
   - Output: `design_doc_path`

4. **[AUTO] `gf-issue-create`** — **MANDATORY**
   - Create Issue (or use existing), reference design doc in body
   - Output: `issue_url`

5. **[AUTO] `gf-issue-review`** — **MANDATORY**
   - Review Issue quality, add review comment
   - Output: `comment_id`

6. **[AUTO] Update contract** — `phases.1.evidence = { issue_url, comment_id, design_doc_path }`, `status = "complete"`

7. **[AUTO] Gate 1→2** — All evidence non-empty → **AUTO-ADVANCE to Phase 2**

## Phase 2: Planning

**Entry:** Gate 1→2 passed · **Exit:** `phases.2.status = complete` · **Pause:** yes (Gate 2→3)

| Step | Action | Output |
|------|--------|--------|
| 1 | **[CALL]** `superpowers:writing-plans` (input: `design_doc_path`) — **⚠️ RETURN to orchestrator**. Create a full plan covering architecture, data flow, API design, component tree, and route design. The plan must create a full plan document with all design decisions. | `spec_path` |
| 2 | **[AUTO]** `gf-quality` gate — runs all quality checks: Build check, Test check, Coverage check, Format check, Static check, and Pre-commit check. Report shows status per check. | all checks passed |
| 3 | **[AUTO]** Update contract: `evidence = { spec_path, user_approved: false }` | — |
| 4 | **[PAUSE]** Gate 2→3 + user approval: "approved" → Phase 3 · "changes" → revise · "rejected" → terminate | `user_approved` |

If any quality check fails, the gate blocks advancement. Only when ALL CHECKS PASSED does the workflow continue.

## Phase 3: Execution

**Entry:** Gate 2→3 passed (`user_approved = true`) · **Exit:** `phases.3.status = complete` · **Auto-advance:** yes

| Step | Action | Output |
|------|--------|--------|
| 1 | **[AUTO]** Record `base_branch` via `git rev-parse --abbrev-ref HEAD`, then create worktree: `feat/<issue-number>-<short-description>` | `branch`, `base_branch`, `worktree_path` |
| 2 | **[AUTO]** `superpowers:subagent-driven-development` (TDD: RED → GREEN → REFACTOR) | implementation |
| 3 | **[AUTO]** `gf-pr-create` — PR body MUST include `Closes #<issue-number>` | `pr_url` |
| 4 | **[AUTO]** `make test` or `cargo test` | `tests_passed` |
| 5 | **[AUTO]** Update contract: `evidence = { branch, base_branch, worktree_path, pr_url, tests_passed }` | — |
| 6 | **[AUTO]** Gate 3→4 — `pr_url` + `tests_passed = true` → **AUTO-ADVANCE to Phase 4** | — |

## Phase 4: Post-Delivery Checks

**Entry:** Gate 3→4 passed · **Exit:** `phases.4.status = complete` · **Auto-advance:** archive on complete

### Phase 4 Step Matrix by Mode

| # | Step | Full | Standard | Fast |
|---|------|------|----------|------|
| 1 | Pipeline analysis | ✅ | ✅ | ✅ |
| 2 | Issue triage | ✅ | ❌ | ❌ |
| 3 | Code review report | ✅ | ✅ | ❌ |
| 4 | Dogfooding checklist | ✅ | ❌ | ❌ |
| 5 | Branch Finish | ✅ | ✅ | ✅ |

### Execution Flow by Mode

- **Full:** Pipeline → Triage → Review → Dogfooding → Branch Finish → Archive
- **Standard:** Pipeline → Review → Branch Finish → Archive
- **Fast:** Pipeline → Branch Finish → Archive

| Step | Action | Output |
|------|--------|--------|
| 1 | **[AUTO]** `gf-pipeline-analyzer` — generates pipeline analysis report (all modes) | `pipeline_ok` |
| 2 | **[AUTO]** `gf-issue-triage` — produces Issue triage report (full mode only) | — |
| 3 | **[AUTO]** `gf-review` — creates code review report (full + standard modes) | `review_report_path` |
| 4 | **[AUTO]** Dogfooding checklist (`docs/specs/phase4-dogfooding-checklist.md`) (full mode only) | `dogfooding_passed` |
| 5 | **[CONFIRM]** Branch Finish — detect PR merge status, user-confirmed cleanup (all modes) | `branch_cleaned` |
| 6 | **[AUTO]** Update contract: `evidence = { pipeline_ok, review_report_path, dogfooding_passed, branch_cleaned, phase4_steps_executed }` | — |
| 7 | **[AUTO]** Archive contract → `.cache/workflows/archive/YYYY-MM/` | — |

### Phase 4 Step 5: Branch Finish

**Trigger:** After dogfooding passes. **Requires user confirmation.**

1. Read from contract: `base_branch`, `branch`, `worktree_path` (Phase 3 evidence)
2. Detect PR merge status: `gf pr view` (parse merged state)
3. **PR merged** → present confirmation prompt:
   - `cd` to main working tree (`git rev-parse --git-common-dir` parent)
   - `git checkout $base_branch && git pull origin $base_branch`
   - `git branch -d $branch`
   - `git worktree remove $worktree_path && git worktree prune`
   - `git fetch --prune origin`
   - Set `branch_cleaned = true`
4. **PR not merged** → output "PR 待合并，分支和 worktree 保留", set `branch_cleaned = false`
5. **Error tolerance:** if `git branch -d` fails (unmerged local commits), warn and preserve; do not block archive
6. **Missing fields:** if `base_branch` or `worktree_path` empty (old contract / fast mode), skip cleanup silently

## Enforcement Rules

### Forbidden Actions

- ❌ **Skip Phase 4** — Phase 4 is mandatory in all modes
- ❌ **Fast mode: skip TDD or Code Review** — Fast mode forbids skipping TDD and Code Review
- ❌ **Merge phases** — Each phase must complete before the next begins
- ❌ **Enter next Phase when gate not passed** — Gates are non-negotiable
- ❌ **Yield to user skip requests (Scenario C)** — Do not bypass workflow requirements

**Scenario C Guard:** User says "just write code" → check `phases.2.evidence.spec_path`. Absent → refuse, go to Phase 2. Fast mode exception: allow skip Phase 2.

## Error Handling & Common Mistakes

| Error / Mistake | Recovery |
|-----------------|----------|
| Contract not found | Create new contract (start from Bootstrap) |
| Sub-skill did not return | Reassert: read contract, resume at next step |
| Brainstorming chained to `writing-plans` | Interrupt: return to orchestrator, execute `gf-issue-create` |
| Gate check failed | Return to current Phase to complete evidence |
| Skip gate / inline sub-skill / advance before contract update / worktree leak | Fix and re-run |
| **Invoke sub-skill without contract** / **let sub-skill chain** / **skip Issue create/review** | **STOP** — see Red Flags |

## Reference

Contract operations, cross-session recovery, CLI commands, lifecycle management: see `references.md`.
