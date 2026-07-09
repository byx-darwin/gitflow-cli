# gitflow-workflow Auto-Trigger Orchestration Design

**Issue:** #79 - gitflow-workflow orchestration should auto-trigger sub-skills without manual confirmation
**Date:** 2026-07-09
**Status:** Draft

## Background

The `gitflow-workflow` skill is designed as a contract-driven four-phase gated orchestrator that should automatically drive sub-skills through Phase 1-4. However, in practice, the workflow requires manual confirmation at each step, causing:

1. The orchestrator doesn't truly "orchestrate" - becomes manual stepping
2. Brainstorming's interactive design disrupts automation flow
3. Fragmented user experience requiring repeated guidance

## Goal

Improve the `gitflow-workflow` orchestration logic to automatically trigger sub-skills in full mode, reducing intermediate confirmations while maintaining quality gates at critical decision points.

## Design Decisions

### Decision 1: Automation Level

**Chosen:** Fully automatic driving (Option A)

The orchestrator calls sub-skills, they execute and return control automatically, and the orchestrator immediately proceeds to the next step. Pausing occurs **only at critical gates**.

**Rationale:** Issue #79 explicitly states "reduce intermediate confirmations" and "no need for user step-by-step confirmation". The workflow should be autonomous except at critical decision points.

### Decision 2: Brainstorming for Existing Issues

**Chosen:** Run brainstorming even for existing issues (Option B)

Even when an Issue already exists, still run brainstorming to refine requirements and generate/update design documents. However, brainstorming should proceed quickly based on existing Issue information rather than starting from scratch.

**Rationale:** Even if an Issue exists, brainstorming adds value by refining the design. But it should leverage existing Issue context to minimize unnecessary interaction.

### Decision 3: Implementation Approach

**Chosen:** Documentation-driven (Option A)

Improve SKILL.md to make auto-triggering explicit through documentation, rather than implementing a code-based orchestrator in gitflow-cli.

**Rationale:**
1. Current architecture is already a documentation-driven skill system
2. gitflow-cli is a Rust CLI tool; adding orchestration logic increases complexity
3. The core issue is "Agent doesn't auto-drive"; solution is making document instructions more explicit
4. Future code-based orchestrator can be added as a separate enhancement (new Issue)

### Decision 4: Pause Points

**Chosen:** Pause only at Gate 2→3 (Option A)

Only one pause point in the entire workflow:
- **Phase 1 (brainstorming internal):** Design document review (handled by brainstorming skill itself)
- **Phase 2 end (Gate 2→3):** Plan document approval (`user_approved` field)

**Rationale:**
- Gate 2→3's `user_approved` is already defined as required in the schema
- Phase 1's brainstorming and issue-review already include quality assurance
- Reducing pause points improves automation

### Decision 5: Cross-Session Recovery

**Chosen:** Contract (state) + Plan document (guidance) combination (Option C)

- Contract records state (current Phase, evidence) - tells Agent "where am I"
- Plan document provides execution guidance (specific steps) - tells Agent "what to do"
- Agent reads contract first to locate position, then reads plan document for context

**Rationale:**
- Separation of concerns: Contract is state machine, plan document is execution manual
- No external dependencies; all state in contract and documents
- Easy to understand and implement

## State Machine Design

### Orchestrator State Machine

```
┌─────────────────────────────────────────────────────────────────┐
│                    Gitflow Workflow State Machine               │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  [Start]                                                        │
│     │                                                            │
│     ▼                                                            │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ Phase 1: Clarification                                    │   │
│  │ ──────────────────────────────────────────────────────── │   │
│  │ 1. brainstorming (auto, internal review)                  │   │
│  │ 2. issue-create (auto)                                    │   │
│  │ 3. issue-review (auto)                                    │   │
│  │                                                            │   │
│  │ Gate 1→2: issue_url + comment_id + design_doc_path ✅     │   │
│  │ → AUTO-ADVANCE to Phase 2                                  │   │
│  └──────────────────────────────────────────────────────────┘   │
│     │                                                            │
│     ▼                                                            │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ Phase 2: Planning                                         │   │
│  │ ──────────────────────────────────────────────────────── │   │
│  │ 1. writing-plans (auto)                                   │   │
│  │                                                            │   │
│  │ Gate 2→3: spec_path + user_approved ✅                     │   │
│  │ → PAUSE for user approval                                  │   │
│  │ → WAIT for "approved" or "changes requested"              │   │
│  │ → On approval: AUTO-ADVANCE to Phase 3                     │   │
│  └──────────────────────────────────────────────────────────┘   │
│     │                                                            │
│     ▼                                                            │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ Phase 3: Execution                                        │   │
│  │ ──────────────────────────────────────────────────────── │   │
│  │ 1. subagent-driven-development (auto)                     │   │
│  │ 2. pr-create (auto)                                       │   │
│  │                                                            │   │
│  │ Gate 3→4: pr_url + tests_passed ✅                         │   │
│  │ → AUTO-ADVANCE to Phase 4                                  │   │
│  └──────────────────────────────────────────────────────────┘   │
│     │                                                            │
│     ▼                                                            │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ Phase 4: Delivery                                         │   │
│  │ ──────────────────────────────────────────────────────── │   │
│  │ 1. pipeline-analyzer (auto)                               │   │
│  │ 2. issue-triage (auto)                                    │   │
│  │ 3. review (auto)                                          │   │
│  │                                                            │   │
│  │ → Archive contract                                         │   │
│  │ → COMPLETE                                                 │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Auto-Advance Rules

| Phase Transition | Trigger Condition | Behavior |
|-----------------|-------------------|----------|
| Phase 1 → Phase 2 | Gate 1→2 passed | **AUTO-ADVANCE** to Phase 2 |
| Phase 2 → Phase 3 | Gate 2→3 passed + `user_approved = true` | **PAUSE** for user approval, then **AUTO-ADVANCE** |
| Phase 3 → Phase 4 | Gate 3→4 passed | **AUTO-ADVANCE** to Phase 4 |
| Phase 4 complete | All checks passed | **Archive contract**, workflow ends |

### Key Constraints

1. **Sub-skill auto-invocation:** After each sub-skill completes, the orchestrator immediately calls the next sub-skill without waiting for user input
2. **Single pause point:** Only pause at Gate 2→3 (plan approval)
3. **Contract-driven:** After each Phase completes, immediately update contract, then check gate conditions
4. **Cross-session recovery:** If session interrupts, new session recovers state by reading contract

## Phase Execution Flow

### Phase 1: Clarification (Auto-Trigger)

```yaml
entry_condition: none
exit_condition: phases.1.status = complete
auto_advance: true (on gate pass)
```

**Execution Steps:**

1. **[AUTO]** Check if existing Issue exists
   - If user specified an Issue → use that Issue
   - If not specified → read open issues list for selection

2. **[CALL]** Call `superpowers:brainstorming`
   - Pass context: Issue description (if exists) or user requirements
   - **brainstorming internally will:**
     - Explore project context
     - Ask clarifying questions (one at a time)
     - Propose 2-3 approaches
     - Present design and get user approval
     - Write design doc to `docs/superpowers/specs/YYYY-MM-DD-<topic>-design.md`
     - User reviews written spec
     - **Only completes after user approval**
   - Output: `design_doc_path`

3. **[AUTO]** Call `gitflow-issue-create`
   - Create GitHub Issue (or use existing Issue)
   - Reference design doc path in Issue body
   - Output: `issue_url`

4. **[AUTO]** Call `gitflow-issue-review`
   - Review Issue quality
   - Add review comment
   - Output: `comment_id`

5. **[AUTO]** Update contract
   ```json
   phases.1.evidence = {
     "issue_url": "...",
     "comment_id": "...",
     "design_doc_path": "..."
   }
   phases.1.status = "complete"
   ```

6. **[AUTO]** Gate 1→2 check
   - Conditions: `issue_url` + `comment_id` + `design_doc_path` non-empty
   - Pass → **AUTO-ADVANCE to Phase 2**
   - Fail → return to repair

### Phase 2: Planning (Auto-Trigger + Pause at Gate)

```yaml
entry_condition: Gate 1→2 passed
exit_condition: phases.2.status = complete
auto_advance: false (PAUSE at gate 2→3)
```

**Execution Steps:**

1. **[AUTO]** Call `superpowers:writing-plans`
   - Input: design doc path (from `phases.1.evidence.design_doc_path`)
   - Output: plan doc path `docs/superpowers/plans/YYYY-MM-DD-<topic>-plan.md`

2. **[AUTO]** Update contract
   ```json
   phases.2.evidence = {
     "spec_path": "...",
     "user_approved": false  // initially false
   }
   phases.2.status = "complete"
   ```

3. **[PAUSE]** Gate 2→3 check + user approval
   - Show user the plan doc path
   - **Pause and wait** for user input:
     - "approved" → `user_approved = true` → **enter Phase 3**
     - "changes requested" → return to modify plan
     - "rejected" → terminate workflow

4. **[ON APPROVAL]** Update contract
   ```json
   phases.2.evidence.user_approved = true
   ```

### Phase 3: Execution (Auto-Trigger)

```yaml
entry_condition: Gate 2→3 passed (user_approved = true)
exit_condition: phases.3.status = complete
auto_advance: true (on gate pass)
```

**Execution Steps:**

1. **[AUTO]** Create worktree
   - Branch name: `feat/<issue-number>-<short-description>`

2. **[AUTO]** Call `superpowers:subagent-driven-development`
   - Input: plan doc path (from `phases.2.evidence.spec_path`)
   - Includes TDD cycle: RED → GREEN → REFACTOR
   - Output: implementation complete

3. **[AUTO]** Call `gitflow-pr-create`
   - PR body MUST include `Closes #<issue-number>`
   - Output: `pr_url`

4. **[AUTO]** Run tests
   - `make test` or `cargo test`
   - Output: `tests_passed = true/false`

5. **[AUTO]** Update contract
   ```json
   phases.3.evidence = {
     "branch": "...",
     "pr_url": "...",
     "tests_passed": true
   }
   phases.3.status = "complete"
   ```

6. **[AUTO]** Gate 3→4 check
   - Conditions: `pr_url` + `tests_passed = true`
   - Pass → **AUTO-ADVANCE to Phase 4**
   - Fail → return to TDD cycle to fix

### Phase 4: Delivery (Auto-Trigger)

```yaml
entry_condition: Gate 3→4 passed
exit_condition: phases.4.status = complete
auto_advance: true (archive on complete)
```

**Execution Steps:**

1. **[AUTO]** Call `gitflow-pipeline-analyzer`
   - Generate pipeline analysis report
   - Output: `pipeline_ok = true/false`

2. **[AUTO]** Call `gitflow-issue-triage`
   - Generate Issue triage report

3. **[AUTO]** Call `gitflow-review`
   - Generate code review report
   - Output: `review_report_path`

4. **[AUTO]** Update contract
   ```json
   phases.4.evidence = {
     "pipeline_ok": true,
     "review_report_path": "..."
   }
   phases.4.status = "complete"
   ```

5. **[AUTO]** Archive contract
   - Move: `.cache/workflows/active/<workflow_id>.json` → `.cache/workflows/archive/YYYY-MM/`

6. **[COMPLETE]** Workflow ends

## Cross-Session Recovery

### Problem Scenario

Workflow may be interrupted at any Phase:
- Session closed or timed out
- User switches to other tasks
- Network issues cause Agent disconnection

**Recovery Requirements:** New session (possibly in different window/terminal) can:
1. Identify the currently executing workflow
2. Understand which Phase it's in
3. Get sufficient context to continue execution

### Recovery Mechanism: Contract + Plan Doc Collaboration

```
┌─────────────────────────────────────────────────────────────┐
│                  Cross-Session Recovery                      │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  New Session Starts                                          │
│         │                                                    │
│         ▼                                                    │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Step 1: Read Contract                                 │   │
│  │ ──────────────────────────────────────────────────── │   │
│  │ • List .cache/workflows/active/*.json                │   │
│  │ • Find workflow with status != "complete"            │   │
│  │ • Read current_phase and evidence                    │   │
│  │                                                       │   │
│  │ Output: "Workflow X is in Phase Y"                   │   │
│  └──────────────────────────────────────────────────────┘   │
│         │                                                    │
│         ▼                                                    │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Step 2: Determine Entry Point                         │   │
│  │ ──────────────────────────────────────────────────── │   │
│  │ • current_phase = 1 → Resume from Phase 1            │   │
│  │ • current_phase = 2 → Read design_doc_path           │   │
│  │ • current_phase = 3 → Read spec_path                 │   │
│  │ • current_phase = 4 → Read pr_url, tests_passed      │   │
│  │                                                       │   │
│  │ Output: "Resume from Phase Y, context at <path>"     │   │
│  └──────────────────────────────────────────────────────┘   │
│         │                                                    │
│         ▼                                                    │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Step 3: Load Context Document                         │   │
│  │ ──────────────────────────────────────────────────── │   │
│  │ • Phase 1: No doc needed (start fresh)               │   │
│  │ • Phase 2: Read design_doc_path                      │   │
│  │ • Phase 3: Read spec_path (plan document)            │   │
│  │ • Phase 4: Read pr_url + review reports              │   │
│  │                                                       │   │
│  │ Output: "Context loaded, ready to execute Phase Y"   │   │
│  └──────────────────────────────────────────────────────┘   │
│         │                                                    │
│         ▼                                                    │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Step 4: Continue Execution                            │   │
│  │ ──────────────────────────────────────────────────── │   │
│  │ • Resume from current_phase                          │   │
│  │ • Follow auto-trigger rules                          │   │
│  │ • Update contract on completion                      │   │
│  │                                                       │   │
│  │ Output: "Workflow X completed" or "Paused at Gate Z" │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Key Design Principles

1. **Contract is state machine:** Records `current_phase` and `evidence`, tells Agent "where am I"
2. **Plan doc is execution manual:** Contains specific implementation steps, tells Agent "what to do"
3. **Design doc is requirement source:** Phase 2+ needs to read `design_doc_path` to understand requirements
4. **No external dependencies:** All state in contract and documents, not dependent on memory or session state

### Recovery Example

**Scenario:** User closes session during Phase 3

**Recovery Flow:**
```bash
# Step 1: Find active workflow
$ ls .cache/workflows/active/
wf-2026-07-09-001.json

# Step 2: Read contract
$ cat .cache/workflows/active/wf-2026-07-09-001.json | jq '.current_phase, .phases["3"]'
3
{
  "name": "Execution",
  "status": "in_progress",
  "started_at": "2026-07-09T10:30:00Z",
  "evidence": {
    "spec_path": "docs/superpowers/plans/2026-07-09-feature-x-plan.md"
  }
}

# Step 3: New session reads plan document
$ cat docs/superpowers/plans/2026-07-09-feature-x-plan.md
# Implementation Plan: Feature X
## Step 1: ...
## Step 2: ...

# Step 4: Continue executing Phase 3
→ Call subagent-driven-development
→ Create PR
→ Update contract
```

## Contract Schema Updates

### Fields to Add

Based on the new design, add to contract schema:

1. **`design_doc_path`**: Phase 1 design document path (from brainstorming output)
2. Version upgrade: `1.0` → `1.1`

### Updated Schema (Key Changes)

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://gitflow-cli.ai/schemas/workflow-contract-v1.1.json",
  "title": "Workflow Contract",
  "version": "1.1",
  ...
  "$defs": {
    "phase": {
      ...
      "evidence": {
        "type": "object",
        "properties": {
          "issue_url": { "type": "string" },
          "comment_id": { "type": "string" },
          "design_doc_path": { "type": "string" },
          "spec_path": { "type": "string" },
          "user_approved": { "type": "boolean" },
          "branch": { "type": "string" },
          "pr_url": { "type": "string" },
          "tests_passed": { "type": "boolean" },
          "pipeline_ok": { "type": "boolean" },
          "review_report_path": { "type": "string" }
        },
        "additionalProperties": false
      }
      ...
    }
  }
}
```

### Key Changes

1. **Version upgrade:** `1.0` → `1.1`
2. **New field:** `phases.1.evidence.design_doc_path` (design document path)
3. **Backward compatible:** `design_doc_path` is optional, doesn't affect existing contracts

## Gate Rules Updates

### Updated Gate Definitions

```yaml
Gate 1→2: Clarification → Planning
─────────────────────────────────────
Conditions:
  - phases.1.status == "complete"
  - phases.1.evidence.issue_url non-empty
  - phases.1.evidence.comment_id non-empty
  - phases.1.evidence.design_doc_path non-empty  # NEW

fast mode exemption:
  - comment_id can be omitted (issue-review optional)
  - design_doc_path can be omitted (brainstorming optional)

Failure handling:
  - Block entry to Phase 2
  - Return to Phase 1 to execute missing steps


Gate 2→3: Planning → Execution
─────────────────────────────────────
Conditions:
  - phases.2.status == "complete"
  - phases.2.evidence.spec_path non-empty
  - phases.2.evidence.user_approved == true

fast mode exemption:
  - spec_path and user_approved can be omitted (writing-plans optional)

Failure handling:
  - Block entry to Phase 3
  - Return to Phase 2 to modify plan

Pause behavior:
  - Gate 2→3 is the only gate requiring user confirmation
  - Orchestrator pauses and shows plan document path
  - Waits for user input: "approved" / "changes requested" / "rejected"


Gate 3→4: Execution → Delivery
─────────────────────────────────────
Conditions:
  - phases.3.status == "complete"
  - phases.3.evidence.pr_url non-empty
  - phases.3.evidence.tests_passed == true

No exemptions (must pass in any mode)

Failure handling:
  - Block entry to Phase 4
  - Return to Phase 3 TDD cycle to fix


Auto-Advance Rules:
─────────────────────────────────────
Phase 1 → Phase 2:  Gate 1→2 passed → AUTO-ADVANCE
Phase 2 → Phase 3:  Gate 2→3 passed + PAUSE for approval → AUTO-ADVANCE (on approval)
Phase 3 → Phase 4:  Gate 3→4 passed → AUTO-ADVANCE
Phase 4 complete:   Archive contract → COMPLETE
```

### Gate Check Algorithm (Updated)

```python
def check_gate(contract, target_phase):
    if target_phase == 2:
        evidence = contract["phases"]["1"]["evidence"]
        if contract["mode"] == "fast":
            # fast mode exempts comment_id and design_doc_path
            return contract["phases"]["1"]["status"] == "complete" \
                   and evidence.get("issue_url")
        return contract["phases"]["1"]["status"] == "complete" \
               and evidence.get("issue_url") \
               and evidence.get("comment_id") \
               and evidence.get("design_doc_path")

    elif target_phase == 3:
        if contract["mode"] == "fast":
            return True  # fast mode skips planning
        evidence = contract["phases"]["2"]["evidence"]
        return contract["phases"]["2"]["status"] == "complete" \
               and evidence.get("spec_path") \
               and evidence.get("user_approved")

    elif target_phase == 4:
        evidence = contract["phases"]["3"]["evidence"]
        return contract["phases"]["3"]["status"] == "complete" \
               and evidence.get("pr_url") \
               and evidence.get("tests_passed")

    return False
```

## Implementation Plan

### Changes Required

1. **Update SKILL.md:**
   - Add "Orchestrator State Machine" section
   - Rewrite Phase 1-4 execution flows to emphasize auto-triggering
   - Add "Cross-Session Recovery" section
   - Update gate rules with new conditions

2. **Update contract.schema.json:**
   - Add `design_doc_path` field to Phase 1 evidence
   - Bump version to 1.1

3. **Create new Issue:**
   - Future enhancement: Code-based orchestrator in gitflow-cli

### Files to Modify

- `.claude/skills/gitflow-workflow/SKILL.md`
- `.claude/skills/gitflow-workflow/contract.schema.json`

## Future Enhancements

### Code-Based Orchestrator (Separate Issue)

Implement `gitflow workflow run` command in Rust that:
- Programmatically drives the four-phase pipeline
- Enforces gate rules with code, not just documentation
- Provides CLI for workflow management (list, status, archive, cleanup)
- Better error handling and recovery

This is a separate enhancement beyond the scope of Issue #79.

## Success Criteria

- [ ] SKILL.md clearly defines auto-trigger rules for each Phase
- [ ] Contract schema includes `design_doc_path` field
- [ ] Gate rules updated with new conditions
- [ ] Cross-session recovery mechanism documented
- [ ] Agent can successfully execute full workflow with minimal user intervention
- [ ] Only pauses at Gate 2→3 for plan approval
- [ ] New session can recover and continue workflow from contract + plan doc

## Related Issues

- Issue #80: Phase 1→2 transition should enforce design document output (related to this design)
- Future Issue: Code-based orchestrator in gitflow-cli
