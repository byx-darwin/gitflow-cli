---
name: gitflow-workflow
description: |
  Use when the user wants a mandatory four-phase gated workflow with
  contract verification between phases, or invokes `/gitflow-workflow`.
  Enforces: clarify → plan → execute → deliver with JSON state tracking.
  当用户需要强制执行的四阶段闸门驱动全流程时使用。
---

# gitflow-workflow — Contract-Driven Four-Phase Gated Orchestrator

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
2. Run mode auto-detection (full / fast)
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
| **Brainstorming Override** | When `brainstorming` is called as a Phase 1 sub-skill, its terminal state is **RETURN TO ORCHESTRATOR** (not `writing-plans`). The orchestrator handles the transition to `gitflow-issue-create`. |
| **Single Active Orchestrator** | Only this workflow's state machine drives the conversation. No other skill may claim orchestration while a contract is active. |
| **Evidence Before Gate** | A gate check MAY NOT pass until all required evidence fields are populated. |
| **No Implicit Completion** | A Phase is complete ONLY when the orchestrator sets `status = "complete"` in the contract. Sub-skill completion ≠ Phase completion. |

## Red Flags — STOP and Reassert Control

| Red Flag | Action |
|----------|--------|
| About to invoke `brainstorming` without a contract | **STOP** — create contract first |
| About to create a new contract when an active one exists | **STOP** — resume the existing contract instead |
| `brainstorming` starts invoking `writing-plans` | **STOP** — interrupt, return to orchestrator, execute `gitflow-issue-create` |
| About to skip `gitflow-issue-create` or `gitflow-issue-review` | **STOP** — MANDATORY in Phase 1 |
| About to advance without updating contract evidence | **STOP** — update contract first |
| User says "just write the code" | **CHECK** — Scenario C? If no contract, refuse and start Phase 1 |
| About to let a sub-skill chain to another | **STOP** — sub-skills return to orchestrator |

## Rationalization Table

| Excuse | Reality |
|--------|---------|
| "brainstorming will handle Issue creation" | No — brainstorming chains to `writing-plans`, not Issue creation. Orchestrator must do it. |
| "Contract can be created later" | No — contract MUST exist before any sub-skill. It is the single source of truth. |
| "User just wants to discuss" | If they invoked `/gitflow-workflow`, run the workflow. |
| "Issue review is optional" | No — `gitflow-issue-review` is MANDATORY in both full and fast modes. |
| "Brainstorming asked questions, Phase 1 is done" | No — brainstorming is ONE step. Issue list/create/review are separate mandatory steps. |
| "Requirement is clear, skip to Phase 3" | Scenario C. If `phases.2.evidence.spec_path` is empty, refuse and go to Phase 2. |
| "New session, start fresh" | No — check `.cache/workflows/active/` first. If incomplete contract exists, resume it. |
| "Different agent should start over" | No — contract is agent-agnostic. Any agent can resume from `current_phase` + evidence. |

## When to Use

| EN | ZH |
|----|----|
| full workflow | 全流程（默认） |
| clarify → plan → execute → deliver | 需求→计划→执行→交付 |

**When NOT to Use:** quick fix → `gitflow-commit` · PR review → `gitflow-pr-review` · architecture discussion → `superpowers:brainstorming` directly · user says "don't create an Issue" → do NOT invoke.

**Mode auto-detection:** "fix"/"typo"/"hotfix" → `fast` · "new feature"/"architecture"/"refactor" → `full` · `good-first-issue` label → `fast` · unclear → **ask user**.

## Mode Comparison

| Phase | Full Mode | Fast Mode |
|-------|-----------|-----------|
| 1 | brainstorming + issue-create + issue-review | issue-create (required), brainstorming (optional) |
| 2 | writing-plans + quality gate | **skippable** |
| 3 | subagent-driven-development (TDD + Code Review) | **required** |
| 4 | pipeline + triage + review + dogfooding | **required** |

## Fast Mode — Required Skills Checklist

In fast mode, the following skills are invoked per phase:

**Phase 1:** `gitflow-issue-create` (required), `superpowers:brainstorming` (optional)

**Phase 2:** `superpowers:writing-plans` (optional, skippable)

**Phase 3:** `superpowers:subagent-driven-development` with TDD + Code Review (required)

**Phase 4:** `gitflow-pipeline-analyzer` → `gitflow-issue-triage` → `gitflow-review` → dogfooding checklist (all required)

## State Machine

```
[Start] → Bootstrap → Phase 1 → [Gate 1→2] → AUTO → Phase 2 → [Gate 2→3] → PAUSE → Phase 3 → [Gate 3→4] → AUTO → Phase 4 → [Archive] → [Complete]
```

**Single pause point:** Gate 2→3 (plan approval). All other transitions auto-advance.

## Gate Rules

Full definitions: `skills/gitflow-workflow/gates.md`

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
   - Otherwise → `gitflow-cli issue list --state open`

3. **[CALL] `superpowers:brainstorming`**
   - Pass: Issue description or user requirements
   - **⚠️ RETURN RULE:** Terminal state = **RETURN TO ORCHESTRATOR** (not `writing-plans`)
   - Brainstorming will: explore context → ask questions → propose approaches → present design → write spec → **return control**
   - Output: `design_doc_path`

4. **[AUTO] `gitflow-issue-create`** — **MANDATORY**
   - Create Issue (or use existing), reference design doc in body
   - Output: `issue_url`

5. **[AUTO] `gitflow-issue-review`** — **MANDATORY**
   - Review Issue quality, add review comment
   - Output: `comment_id`

6. **[AUTO] Update contract** — `phases.1.evidence = { issue_url, comment_id, design_doc_path }`, `status = "complete"`

7. **[AUTO] Gate 1→2** — All evidence non-empty → **AUTO-ADVANCE to Phase 2**

## Phase 2: Planning

**Entry:** Gate 1→2 passed · **Exit:** `phases.2.status = complete` · **Pause:** yes (Gate 2→3)

| Step | Action | Output |
|------|--------|--------|
| 1 | **[CALL]** `superpowers:writing-plans` (input: `design_doc_path`) — **⚠️ RETURN to orchestrator**. Create a full plan covering architecture, data flow, API design, component tree, and route design. The plan must create a full plan document with all design decisions. | `spec_path` |
| 2 | **[AUTO]** `gitflow-quality` gate — runs all quality checks: Build check, Test check, Coverage check, Format check, Static check, and Pre-commit check. Report shows status per check. | all checks passed |
| 3 | **[AUTO]** Update contract: `evidence = { spec_path, user_approved: false }` | — |
| 4 | **[PAUSE]** Gate 2→3 + user approval: "approved" → Phase 3 · "changes" → revise · "rejected" → terminate | `user_approved` |

If any quality check fails, the gate blocks advancement. Only when ALL CHECKS PASSED does the workflow continue.

## Phase 3: Execution

**Entry:** Gate 2→3 passed (`user_approved = true`) · **Exit:** `phases.3.status = complete` · **Auto-advance:** yes

| Step | Action | Output |
|------|--------|--------|
| 1 | **[AUTO]** Create worktree: `feat/<issue-number>-<short-description>` | `branch` |
| 2 | **[AUTO]** `superpowers:subagent-driven-development` (TDD: RED → GREEN → REFACTOR) | implementation |
| 3 | **[AUTO]** `gitflow-pr-create` — PR body MUST include `Closes #<issue-number>` | `pr_url` |
| 4 | **[AUTO]** `make test` or `cargo test` | `tests_passed` |
| 5 | **[AUTO]** Update contract: `evidence = { branch, pr_url, tests_passed }` | — |
| 6 | **[AUTO]** Gate 3→4 — `pr_url` + `tests_passed = true` → **AUTO-ADVANCE to Phase 4** | — |

## Phase 4: Post-Delivery Checks

**Entry:** Gate 3→4 passed · **Exit:** `phases.4.status = complete` · **Auto-advance:** archive on complete

| Step | Action | Output |
|------|--------|--------|
| 1 | **[AUTO]** `gitflow-pipeline-analyzer` — generates pipeline analysis report | `pipeline_ok` |
| 2 | **[AUTO]** `gitflow-issue-triage` — produces Issue triage report | — |
| 3 | **[AUTO]** `gitflow-review` — creates code review report | `review_report_path` |
| 4 | **[AUTO]** Dogfooding checklist (`docs/specs/phase4-dogfooding-checklist.md`) | `dogfooding_passed` |
| 5 | **[AUTO]** Update contract: `evidence = { pipeline_ok, review_report_path, dogfooding_passed }` | — |
| 6 | **[AUTO]** Archive contract → `.cache/workflows/archive/YYYY-MM/` | — |

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
| Brainstorming chained to `writing-plans` | Interrupt: return to orchestrator, execute `gitflow-issue-create` |
| Gate check failed | Return to current Phase to complete evidence |
| Skip gate / inline sub-skill / advance before contract update / worktree leak | Fix and re-run |
| **Invoke sub-skill without contract** / **let sub-skill chain** / **skip Issue create/review** | **STOP** — see Red Flags |

## Reference

Contract operations, cross-session recovery, CLI commands, lifecycle management: see `references.md`.
