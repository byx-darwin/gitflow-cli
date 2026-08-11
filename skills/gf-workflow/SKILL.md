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
## Core Rule: Contract First

**Before ANY phase executes, the orchestrator MUST:**

1. **Check for active contracts** — list `.cache/workflows/active/*.json`
   - Incomplete workflow exists (`status != "complete"`) → **RESUME** it: read `current_phase`, load context, continue from next step
   - Multiple exist → ask user which to resume
   - None exist → proceed to step 2
2. Run mode auto-detection (full / standard / fast)
3. **Detect skill source** — see `## Skill Source Resolution`. Runs BEFORE the contract exists; if both sources are absent the user chooses inline-continue or abort (abort → no contract)
4. Create the contract file at `.cache/workflows/active/<workflow_id>.json` (schema: `contract.schema.json`), then record `skill_source` via jq immediately after creation
5. Announce the workflow start with: workflow_id, mode, title, `skill_source`

**If no contract exists, no sub-skill may be invoked.** The contract is the
single source of truth for the workflow's state.

## Skill Source Resolution

gf-workflow runs on top of ONE external skill source: `superpowers` or `mattpocock/skills`.
Phase steps below use **role aliases** only; actual skill names resolve via
`references.md` → **Dual-Source Mapping Table** (single point of maintenance).

### Detection (at Bootstrap, BEFORE contract creation)

1. Introspect the session's available-skills list (primary signal — "invocable as detected").
   Filesystem probing is diagnostics-only for error messages, never a decision source.
2. Sentinels (each matches namespaced or bare form; bare form requires double hits):
   - superpowers: `superpowers:brainstorming` (or bare `brainstorming` + `writing-plans`)
   - mattpocock: `to-spec` + `grilling` double hit (namespaced `mattpocock-skills:*` or bare)
3. Result matrix:

| Detection | Action | `skill_source` |
|---|---|---|
| Only superpowers | adopt | `superpowers` |
| Only mattpocock | adopt | `mattpocock` |
| Both present | **ASK user** which source this workflow uses — no default priority | user's choice |
| Neither | **ASK**: continue inline / abort (never degrade silently; abort → no contract) | `inline` if continuing |

4. Record: `jq '.skill_source = "<value>"'` on the contract right after creation.
5. Resume: reuse the contract's `skill_source`, then re-verify its sentinels are still
   present; if vanished, re-run the neither-present prompt.

### Pause Semantics (mattpocock path)

mattpocock's `to-spec` / `to-tickets` / `implement` are `disable-model-invocation: true` —
the orchestrator MUST NOT attempt to invoke them. At each such step: ✋ **PAUSE**, print the
exact slash command with its constraint instructions from `references.md` → Source Branch
Semantics, and wait for the user to run it.

### Cross-Session Resume

When resuming an existing contract, load context based on `current_phase`:

| Phase | Context to Load | Resume From |
|-------|----------------|-------------|
| 1 | `design_doc_path` (if exists) | Next uncompleted step in Phase 1 |
| 2 | `design_doc_path` + `spec_path` | Gate 2→3 pause (await user approval) |
| 3 | `spec_path` (plan doc) | Next step after last evidence |
| 4 | `pr_url` + review reports | Next check in Phase 4 |

Full recovery procedure: see `references.md` → Cross-Session Recovery.
`skill_source` is always loaded from the contract (never re-detected silently) and re-verified per `## Skill Source Resolution`.

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
| About to invoke a source sub-skill without reading `references.md` mapping table | **STOP** — read the mapping table first; resolve names from the session skills list |
| About to auto-invoke a user-invoked skill (`/to-spec`, `/to-tickets`, `/implement`) | **STOP** — these are `disable-model-invocation`; ✋ PAUSE and prompt the user |
| About to run Phase 3 same-session without an explicit user request | **STOP** — Gate 2→3 includes execution-mode choice; same-session is explicit-only |
| About to let `/to-spec` publish to the tracker | **STOP** — local-only constraint; issue creation belongs to `gf-issue-create` |

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
| "to-spec can publish the issue itself" | No — to-spec is constrained local-only; issue creation is unified under `gf-issue-create`. |
| "The background agent can run /implement" | No — /implement is user-invoked; mattpocock's mode menu is trimmed to new-window / same-session. |
| "Both sources installed — pick the better one" | No priority — ask the user which source this workflow uses. |

## When to Use

| EN | ZH |
|----|----|
| full workflow | 全流程（默认） |
| clarify → plan → execute → deliver | 需求→计划→执行→交付 |

**When NOT to Use:** quick fix → `gf-commit` · PR review → `gf-pr-review` · architecture discussion → the installed source's clarification skill directly (per `references.md` mapping) · user says "don't create an Issue" → do NOT invoke.

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

**Phase 1:** `gf-issue-create` (required), Clarification skill per `skill_source` (optional)

**Phase 2:** Planning skill per `skill_source` (optional, skippable)

**Phase 3:** Execution engine per `skill_source` with TDD + Code Review (required)

**Phase 4:** `gf-pipeline-analyzer` → `gf-issue-triage` → `gf-review` → dogfooding checklist (all required)

## Standard Mode — Required Skills Checklist

In standard mode, the following skills are invoked per phase:

**Phase 1:** Clarification skill per `skill_source` (required), `gf-issue-create` (required), `gf-issue-review` (required)

**Phase 2:** Planning skill per `skill_source` (required) + `gf-quality` gate (required)

**Phase 3:** Execution engine per `skill_source` with TDD + Code Review (required)

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
   - **Also read issue comments** → `gf issue comments <number>` to capture additional context from discussions

3. **[CALL] Clarification skill** (per `skill_source`; names in `references.md` → Dual-Source Mapping Table)
   - superpowers: `superpowers:brainstorming` (model-invoked)
   - mattpocock: `grilling` (model-invoked), then ✋ PAUSE → user runs `/to-spec` (local-only constraint — see references.md; issue creation stays with `gf-issue-create`)
   - inline: orchestrator self-interviews, then writes the design doc itself
   - Pass: Issue description or user requirements
   - **⚠️ RETURN RULE:** Terminal state = **RETURN TO ORCHESTRATOR** (not `writing-plans`)
   - Output: `design_doc_path`

4. **[AUTO] `gf-issue-create`** — **MANDATORY**
   - Create Issue (or use existing), reference design doc in body
   - mattpocock path: issue creation authority is UNIFIED here — `/to-spec` never publishes to the tracker
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
| 1 | **[CALL] Planning skill** (per `skill_source`) — **⚠️ RETURN to orchestrator**. superpowers: `superpowers:writing-plans` (input: `design_doc_path`) — create a full plan document (architecture, data flow, API design, component tree, route design). mattpocock: ✋ PAUSE → user runs `/to-tickets` on the Phase 1 spec; orchestrator records `ticket_refs` and sets `spec_path` = the spec file `to-tickets` consumed; the gate presents the ticket list + blocking edges. | `spec_path` (+ `ticket_refs` on mattpocock) |
| 2 | **[AUTO]** `gf-quality` gate — runs all quality checks: Build check, Test check, Coverage check, Format check, Static check, and Pre-commit check. Report shows status per check. | all checks passed |
| 3 | **[AUTO]** Update contract: `evidence = { spec_path, user_approved: false }` | — |
| 4 | **[PAUSE]** Gate 2→3 + user approval: "approved" → **execution-mode choice (GO gate)**: ① background agent (default, superpowers only) ② manual new window ③ same-session (explicit request only); mattpocock menu is trimmed — see `references.md` → Phase 3 Execution Modes · "changes" → revise · "rejected" → terminate | `user_approved` |

If any quality check fails, the gate blocks advancement. Only when ALL CHECKS PASSED does the workflow continue.

## Phase 3: Execution

**Entry:** Gate 2→3 passed (`user_approved = true`) · **Exit:** `phases.3.status = complete` · **Auto-advance:** yes

| Step | Action | Output |
|------|--------|--------|
| 1 | **[AUTO]** Record `base_branch` via `git rev-parse --abbrev-ref HEAD`. Worktree path is FIXED at `.worktree/<branch-name>` (covered by `.worktree/` in `.gitignore`). Branch name: `feat/<issue-number>-<short-description>`. Created here for same-session mode; created by the executor (background agent / new window) otherwise — see `references.md` → Phase 3 Execution Modes. **After worktree creation**: symlink shared directories so workflow contracts and Claude config are accessible from the worktree: `mkdir -p <worktree-path>/.cache && ln -s ../../.cache/workflows <worktree-path>/.cache/workflows && ln -s ../../.claude <worktree-path>/.claude` | `branch`, `base_branch`, `worktree_path` |
| 2 | **[AUTO] Execution engine** (per `skill_source` + chosen execution mode): superpowers → `superpowers:subagent-driven-development` (same-session) or `superpowers:executing-plans` (new window / background agent); mattpocock → ✋ PAUSE per ticket → user runs `/implement` in dependency order (internal `/tdd` mandatory). All paths: TDD RED → GREEN → REFACTOR | implementation |
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
   - Note: `worktree_path` follows the convention `.worktree/<branch-name>`
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
