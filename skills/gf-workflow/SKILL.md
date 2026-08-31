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

> **Token-cost tuning:** file count alone is capped at 5 — it's a weak risk signal and
> was dominating the score under the old formula, pushing token-cost-heavy independent
> subagents at borderline task sizes. `crosses_module_boundary` / `changes_public_api` /
> `requires_migration` stay full-weight — those are real architectural-risk signals and
> must still force isolation regardless of file count.

```python
def classify_task_complexity(task):
    score = 0
    score += min(len(task.files_changed), 5) * 1
    score += 3 if task.crosses_module_boundary else 0
    score += 2 if task.changes_public_api else 0
    score += 1 if task.requires_migration else 0

    if score <= 4:
        return "simple"    # batch
    elif score <= 9:
        return "medium"    # independent subagent
    else:
        return "complex"   # independent subagent + extra review
```

### Execution by Complexity

| Complexity | Method | Description |
|-----------|--------|-------------|
| Simple (score ≤ 4) | Batch in main agent | Implement all tasks in main agent, single review pass |
| Medium (score 5-9) | Independent subagent | One subagent per task + TDD + review |
| Complex (score > 9) | Independent subagent + extra review | One subagent per task + TDD + review + extra scrutiny |

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
| About to `git worktree add` without classifying the main tree | **STOP** — run Worktree Preflight first; the worktree forks a committed state and would leave `spec_path` behind |
| About to delete untracked files to make the tree look clean | **STOP** — untracked is not disposable. Bucket B is classified and asked about, never removed |

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
| "Phase 1/2 docs can just stay in the main tree" | No — `git worktree add` forks a committed state. The contract points at `spec_path`, and the executor cannot read a file that never entered its worktree. |
| "Gate 2→3 already checked the tree" | No — that gate validates contract evidence. And in modes ① and ② the *executor* creates the worktree, so the preflight must travel with the handoff. |

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
| 1 | **[AUTO]** Record `base_branch` via `git rev-parse --abbrev-ref HEAD`. **Preflight is a hard precondition to `git worktree add`**: that command forks a *committed* state, so an unclassified tree leaves this workflow's own `design_doc_path` / `spec_path` behind while the contract still points at them. Classify `git status --porcelain` → bucket A (workflow artifacts: carry into the worktree, commit on the **feature branch**, never `base_branch`) / bucket B (unrelated: ✋ PAUSE, four options, never auto-commit, never delete) / bucket C (gitignored: skip). Then create the worktree: path FIXED at `.worktree/<branch-name>` (covered by `.worktree/` in `.gitignore`), branch `feat/<issue-number>-<short-description>`. Created here for same-session mode; created by the executor (background agent / new window) otherwise — the handoff MUST carry the preflight steps verbatim; see `references.md` → Phase 3 Execution Modes. **After worktree creation**: symlink shared directories so workflow contracts and Claude config are accessible from the worktree: `mkdir -p <worktree-path>/.cache && ln -s ../../.cache/workflows <worktree-path>/.cache/workflows && ln -s ../../.claude <worktree-path>/.claude`. **Then assert** each contract-referenced document exists under `<worktree-path>/` — abort if missing. Remove bucket A's main-tree copies only after their commit is verified, otherwise the later merge aborts on untracked-overwrite. Full procedure: `references.md` → Worktree Preflight. | `branch`, `base_branch`, `worktree_path`, `worktree_preflight` |
| 2 | **[AUTO] Execution engine** (per `skill_source` + chosen execution mode): superpowers → `superpowers:subagent-driven-development` (same-session) or `superpowers:executing-plans` (new window / background agent); mattpocock → ✋ PAUSE per ticket → user runs `/implement` in dependency order (internal `/tdd` mandatory). All paths: TDD RED → GREEN → REFACTOR | implementation |
| 3 | **[AUTO]** `gf-pr-create` — PR body MUST include `Closes #<issue-number>` | `pr_url` |
| 4 | **[AUTO]** `make test` or `cargo test` — **本地前置自检**，不等于 CI 把关 | `tests_passed` |
| 5 | **[AUTO]** 排队合并：`gf pr merge <n> --auto` —— 约 2.5 秒返回，平台在必需检查/pipeline 通过后自动完成。**不得在排队后再往该分支推 commit**：排队绑定的是已通过检查的那个 SHA，后推的 commit 不会被带上（实测踩过）。返回 `merged: false` 属正常（已排期），须原样转述 `message`，不得报"已合并"。GitCode 无此能力 → ✋ 告知用户需手动合并 | `merge_queued` |
| 6 | **[AUTO]** Update contract: `evidence = { branch, base_branch, worktree_path, worktree_preflight, unresolved_dirty_paths, pr_url, tests_passed, merge_queued }` | — |
| 7 | **[AUTO]** Gate 3→4 — `pr_url` + `tests_passed = true` → **AUTO-ADVANCE to Phase 4**。（真正的合并闸门是平台必需检查 + 排队合并，不由本 workflow 判定） | — |

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

Steps in the same bracket are dispatched **in parallel** (one message, multiple
`Agent` tool calls) since they only consume Phase 3's `pr_url`/`branch` and
don't read each other's output — the orchestrator waits for all of them before
continuing. `Dogfooding` stays sequential: it's a cheap local checklist read,
not worth a subagent round-trip.

- **Full:** [Pipeline ∥ Triage ∥ Review] → Dogfooding → Branch Finish → Archive
- **Standard:** [Pipeline ∥ Review] → Branch Finish → Archive
- **Fast:** Pipeline → Branch Finish → Archive (single step, nothing to parallelize)

### Reporting Granularity (Token-Cost Tuning)

Each Phase 4 report step (`gf-pipeline-analyzer`, `gf-review`, `gf-issue-triage`,
dogfooding checklist) defaults to a **one-line status summary** in chat
(`✅ <step>: no findings` / `⚠️ <step>: N findings, see <path>`). The full report
document is still **written to disk** every time (for audit trail), but only
**echoed in full to the conversation** when the step finds an anomaly (a failed
check, a regression, a threshold breach, a flaky/failing pipeline job, an
unresolved review finding). A clean run must never dump the whole report into
context — link the path instead.

**This convention MUST be carried into the parallel-dispatch prompt** for each
subagent — since they run outside the orchestrator's own turn, they don't
inherit it automatically. Each dispatch prompt must explicitly say: run the
skill, write the full report to disk, return only the one-line status summary.
Parallel dispatch otherwise risks re-inflating the token cost this convention
was written to cut (each subagent independently gathers its own repo/PR
context).

| Step | Action | Output |
|------|--------|--------|
| 1 | **[AUTO] Parallel dispatch** — in one message, launch one `Agent` call per `gates.md → get_phase4_steps(mode).parallel` entry: `gf-pipeline-analyzer` generates a pipeline analysis report (all modes), `gf-issue-triage` produces an Issue triage report (full only), `gf-review` creates a code review report (full + standard). Each subagent prompt MUST include the Reporting Granularity instruction above. Wait for all to return before continuing. | `pipeline_ok`, `review_report_path` (+ triage findings echoed inline if any) |
| 2 | **[AUTO]** Dogfooding checklist (`docs/specs/phase4-dogfooding-checklist.md`) — sequential, local, full mode only; echo in full only if any item fails | `dogfooding_passed` |
| 3 | **[AUTO]** Update contract: `evidence = { pipeline_ok, review_report_path, dogfooding_passed, branch_cleaned, phase4_steps_executed }` — join point; only the orchestrator writes the contract, never a dispatched subagent | — |
| 4 | **[CONFIRM]** Branch Finish — detect PR merge status, user-confirmed cleanup (all modes) | `branch_cleaned` |
| 5 | **[AUTO]** Archive contract → `.cache/workflows/archive/YYYY-MM/` | — |

### Phase 4 Step 4: Branch Finish

**Trigger:** After the parallel dispatch group (Step 1) and, in full mode, Dogfooding (Step 2) complete. **Requires user confirmation.**

1. Read from contract: `base_branch`, `branch`, `worktree_path` (Phase 3 evidence)
   - Note: `worktree_path` follows the convention `.worktree/<branch-name>`
2. Detect PR merge status: `gf pr view <n>` → 读 **`mergedAt`**
   - `mergedAt` 非空 → 判定**已合并**
   - `mergedAt` 为空且 `state == Closed` → **无法判定**：`State` 把 `MERGED` alias 进
     `Closed`，"关了没合"与"已合并"在 `state` 上完全同形；而 GitLab/GitCode 可能不返回
     `mergedAt`，此时 `None` 只代表"平台未上报"。→ ✋ **必须问用户**，给出 PR URL 与 state，
     由人确认后才允许删分支。**绝不靠推断删除**（`git branch -d` 的"未合并则失败"只是最后兜底，
     不是判定手段）
   - `state == Open` → 未合并，走下面第 4 步
3. **PR merged** → present confirmation prompt:
   - `cd` to main working tree (`git rev-parse --git-common-dir` parent)
   - **Re-run the Worktree Preflight classification** before touching branch state: `git checkout`/`git pull` are blocked by the same dirty tree that blocks `git worktree add`. Bucket A is empty by now (its commit is merged); anything left is bucket B → ✋ PAUSE, never auto-commit, never delete.
   - If `unresolved_dirty_paths` is non-empty, list it here — those are files Phase 3 deliberately left in the main tree.
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
