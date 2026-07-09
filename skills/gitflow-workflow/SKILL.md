---
name: gitflow-workflow
description: |
  Contract-driven four-phase gated pipeline. Use when the user wants a
  mandatory clarify → plan → execute → deliver workflow with JSON contract
  verification between phases. 当用户需要强制执行的四阶段闸门驱动全流程时使用。
---

# gitflow-workflow — Contract-Driven Four-Phase Gated Orchestrator

Orchestrator commands only; state lives in the contract; gates are never skipped.

## When to Use

| EN | ZH |
|----|----|
| full workflow | 全流程（默认） |
| clarify → plan → execute → deliver | 需求→计划→执行→交付 |
| contract-driven orchestration | 合同驱动编排 |

**Mode auto-detection** (run before entering Phase 1):
- Description contains "fix"/"typo"/"hotfix" → `fast`
- Description contains "new feature"/"architecture"/"refactor" → `full`
- Issue label contains `good-first-issue` → `fast`
- Cannot determine → **ask the user**

## Mode Comparison

### Full Mode
Full four-phase pipeline, mandatory: brainstorming → issue-create/review → writing-plans → subagent-driven-development → TDD → Phase 4

| Phase | Sub-skill | Description |
|-------|-----------|------|
| 1 | `superpowers:brainstorming` | Clarification |
| 1 | `gitflow-issue-create` | Create Issue |
| 1 | `gitflow-issue-review` | Review & comment |
| 2 | `superpowers:writing-plans` | Plan creation |
| 3 | `superpowers:subagent-driven-development` | Includes TDD + Code Review |
| 4 | `gitflow-pipeline-analyzer → gitflow-issue-triage → gitflow-review → dogfooding-checklist` | Post-delivery checks |

### Fast Mode — Required Skills Checklist
Phase 1: gitflow-issue-create (required), brainstorming (optional)
Phase 2: writing-plans (optional, skippable)
Phase 3: subagent-driven-development (required, includes TDD + Code Review)
Phase 4: gitflow-pipeline-analyzer → gitflow-issue-triage → gitflow-review → dogfooding-checklist (required)

## Orchestrator State Machine

The workflow follows a state machine with explicit auto-advance rules:

```
[Start] → Phase 1 → [Gate 1→2] → AUTO → Phase 2 → [Gate 2→3] → PAUSE → Phase 3 → [Gate 3→4] → AUTO → Phase 4 → [Archive] → [Complete]
```

### Auto-Advance Rules

| Phase Transition | Trigger | Behavior |
|-----------------|---------|----------|
| Phase 1 → Phase 2 | Gate 1→2 passed | **AUTO-ADVANCE** |
| Phase 2 → Phase 3 | Gate 2→3 passed + user approval | **PAUSE** then **AUTO-ADVANCE** |
| Phase 3 → Phase 4 | Gate 3→4 passed | **AUTO-ADVANCE** |
| Phase 4 complete | All checks passed | **Archive contract** |

**Key Constraints:**
1. After each sub-skill completes, immediately call the next sub-skill (no user confirmation)
2. **Single pause point:** Gate 2→3 (plan approval)
3. After each Phase completes, immediately update contract, then check gate
4. Cross-session recovery via contract + plan doc collaboration

## Core Pattern: Contract

Each workflow run creates a contract file:

```
.cache/workflows/active/<workflow_id>.json
```

Contract schema: `skills/gitflow-workflow/contract.schema.json`

### Contract Example

```json
{
  "version": "1.0",
  "workflow_id": "wf-2026-07-09-001",
  "title": "feat: TOON output format",
  "mode": "full",
  "created_at": "2026-07-09T02:59:32Z",
  "updated_at": "2026-07-09T03:30:00Z",
  "current_phase": 3,
  "phases": {
    "1": {
      "name": "Clarification",
      "status": "complete",
      "started_at": "2026-07-09T02:59:32Z",
      "completed_at": "2026-07-09T03:10:00Z",
      "executor": "claude-code-3.7",
      "evidence": {
        "issue_url": "https://github.com/.../issues/74",
        "comment_id": "4921173903"
      }
    },
    "2": {
      "name": "Planning",
      "status": "complete",
      "started_at": "2026-07-09T03:10:00Z",
      "completed_at": "2026-07-09T03:20:00Z",
      "executor": "claude-code-3.7",
      "evidence": {
        "spec_path": "docs/superpowers/specs/2026-07-09-toon-design.md",
        "user_approved": true
      }
    },
    "3": {
      "name": "Execution",
      "status": "in_progress",
      "started_at": "2026-07-09T03:20:00Z",
      "completed_at": null,
      "executor": "subagent-task-3",
      "evidence": {}
    },
    "4": {
      "name": "Delivery",
      "status": "pending",
      "started_at": null,
      "completed_at": null,
      "executor": null,
      "evidence": {}
    }
  }
}
```

## Gate Rules

Full gate definitions: `skills/gitflow-workflow/gates.md`

**Key principle:** Before entering the next Phase, the previous Phase's evidence completeness is verified. On gate failure, **block entry** and return to repair.

| Enter Phase | Required evidence | fast-mode exemption |
|-----------|--------------|--------------|
| 2 (Planning) | `issue_url` + `comment_id` | `comment_id` optional |
| 3 (Execution) | `spec_path` + `user_approved` | ✅ Skippable |
| 4 (Delivery) | `pr_url` + `tests_passed` | — |

## Phase Execution Flow

### Phase 1: Clarification (Auto-Trigger)

**Entry condition:** none
**Exit condition:** contract `phases.1.status = complete`
**Auto-advance:** true (on gate pass)

**Execution Steps:**

1. **[AUTO]** Read Open Issues - Check if existing Issue exists
   - If user specified an Issue → use that Issue
   - If not specified → run `gitflow-cli issue list --state open` to read open issues list for selection

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

**Entry condition:** Gate 1→2 passed
**Exit condition:** contract `phases.2.status = complete`
**Auto-advance:** false (PAUSE at gate 2→3)

**Execution Steps:**

1. **[AUTO]** Call `superpowers:writing-plans` to create a full plan
   - Input: design doc path (from `phases.1.evidence.design_doc_path`)
   - Output: plan doc path `docs/superpowers/plans/YYYY-MM-DD-<topic>-plan.md`

2. **[AUTO]** Call `gitflow-quality` gate → ALL CHECKS PASSED
   - Build check / Test check / Coverage check / Format check / Static check / Pre-commit check

3. **[AUTO]** Update contract
   ```json
   phases.2.evidence = {
     "spec_path": "...",
     "user_approved": false  // initially false
   }
   phases.2.status = "complete"
   ```

4. **[PAUSE]** Gate 2→3 check + user approval
   - Show user the plan doc path
   - **Pause and wait** for user input:
     - "approved" → `user_approved = true` → **enter Phase 3**
     - "changes requested" → return to modify plan
     - "rejected" → terminate workflow

5. **[ON APPROVAL]** Update contract
   ```json
   phases.2.evidence.user_approved = true
   ```

### Phase 3: Execution (Auto-Trigger)

**Entry condition:** Gate 2→3 passed (user_approved = true)
**Exit condition:** contract `phases.3.status = complete`
**Auto-advance:** true (on gate pass)

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

### Phase 4: Post-Delivery Checks (Auto-Trigger)

**Entry condition:** Gate 3→4 passed
**Exit condition:** contract `phases.4.status = complete`
**Auto-advance:** true (archive on complete)

**Execution Steps:**

1. **[AUTO]** Call `gitflow-pipeline-analyzer`
   - Generate pipeline analysis report
   - Output: `pipeline_ok = true/false`

2. **[AUTO]** Call `gitflow-issue-triage`
   - Generate Issue triage report

3. **[AUTO]** Call `gitflow-review`
   - Generate code review report
   - Output: `review_report_path`

4. **[AUTO]** Execute Dogfooding Checklist
   - Reference: `docs/specs/phase4-dogfooding-checklist.md`
   - Execute each platform's risk-driven checklist items
   - Record any bugs to `.cache/bug-reports/pending.json` with `source: "dogfooding"`
   - All items must pass; any failure blocks Phase 4 completion
   - Output: `dogfooding_passed = true/false`

5. **[AUTO]** Update contract
   ```json
   phases.4.evidence = {
     "pipeline_ok": true,
     "review_report_path": "...",
     "dogfooding_passed": true
   }
   phases.4.status = "complete"
   ```

6. **[AUTO]** Archive contract
   - Move: `.cache/workflows/active/<workflow_id>.json` → `.cache/workflows/archive/YYYY-MM/`

7. **[COMPLETE]** Workflow ends

## Contract Operations API

### Create Contract

```bash
# Generate workflow_id (sequential by date)
DATE=$(date -u +%Y-%m-%d)
COUNT=$(ls .cache/workflows/active/ 2>/dev/null | grep "wf-${DATE}" | wc -l)
WORKFLOW_ID="wf-${DATE}-$(printf '%03d' $((COUNT + 1)))"

# Initialize contract
cat > ".cache/workflows/active/${WORKFLOW_ID}.json" << EOF
{
  "version": "1.0",
  "workflow_id": "${WORKFLOW_ID}",
  "title": "<issue_title>",
  "mode": "<full|fast>",
  "created_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "updated_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "current_phase": 1,
  "phases": {
    "1": { "name": "Clarification", "status": "in_progress", "started_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)", "completed_at": null, "executor": null, "evidence": {} },
    "2": { "name": "Planning", "status": "pending", "started_at": null, "completed_at": null, "executor": null, "evidence": {} },
    "3": { "name": "Execution", "status": "pending", "started_at": null, "completed_at": null, "executor": null, "evidence": {} },
    "4": { "name": "Delivery", "status": "pending", "started_at": null, "completed_at": null, "executor": null, "evidence": {} }
  }
}
EOF
```

### Update Contract (on Phase completion)

```bash
# Update via jq (or edit manually); temp file named by workflow_id to avoid concurrency clash
COMPLETED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
jq --arg phase "$PHASE_NUM" \
   --arg status "complete" \
   --arg completed_at "$COMPLETED_AT" \
   --argjson evidence "$EVIDENCE_JSON" \
  '.phases[$phase].status = $status | .phases[$phase].completed_at = $completed_at | .phases[$phase].evidence = $evidence | .updated_at = $completed_at' \
  ".cache/workflows/active/${WORKFLOW_ID}.json" > "${WORKFLOW_ID}.tmp" \
  && mv "${WORKFLOW_ID}.tmp" ".cache/workflows/active/${WORKFLOW_ID}.json"
```

### Advance to Next Phase (after gate passes)

```bash
# Increment current_phase and set next Phase to in_progress; temp file named by workflow_id
jq --arg next "$((PHASE_NUM + 1))" \
   --arg started_at "$COMPLETED_AT" \
  '.current_phase = ($next | tonumber) | .phases[$next].status = "in_progress" | .phases[$next].started_at = $started_at | .updated_at = $started_at' \
  ".cache/workflows/active/${WORKFLOW_ID}.json" > "${WORKFLOW_ID}.tmp" \
  && mv "${WORKFLOW_ID}.tmp" ".cache/workflows/active/${WORKFLOW_ID}.json"
```

### Read Contract

```bash
# Read current contract status
jq '.current_phase, .phases | to_entries[] | select(.value.status == "in_progress") | .key' \
  ".cache/workflows/active/${WORKFLOW_ID}.json"
```

## Enforcement Rules

### Forbidden Actions
- ❌ Skip Phase 4 (any mode)
- ❌ Fast mode forbids skipping TDD
- ❌ Fast mode forbids skipping Code Review
- ❌ Merge multiple Phases into one step
- ❌ Enter next Phase when gate not passed
- ❌ Yield to user skip requests (Scenario C guard)

### Scenario C Guard
When the user says "the requirement is clear, just write code":
1. Recognize this as a **Phase 2→3 skip request**
2. Check contract: does `phases.2.evidence.spec_path` exist?
3. **Refuse skip when absent**: return to Phase 2 for planning
4. **fast-mode exception**: if fast mode was detected, allow skipping Phase 2

## Multi-Workflow Concurrency

Multiple workflow_ids stored independently, no interference:

```
.cache/workflows/
├── active/
│   ├── wf-2026-07-09-001.json  ← feat: TOON
│   └── wf-2026-07-09-002.json  ← fix: pr merge
└── archive/
    └── 2026-07/
        └── wf-2026-07-08-001.json
```

**Isolation principle:** each workflow uses its own worktree, branch, and contract file.

## Cross-Session Recovery

Workflows may be interrupted at any Phase. New sessions can recover state via contract + plan doc collaboration.

### Recovery Mechanism

```
New Session Starts
    ↓
Step 1: Read Contract
    • List .cache/workflows/active/*.json
    • Find workflow with status != "complete"
    • Read current_phase and evidence
    ↓
Step 2: Determine Entry Point
    • current_phase = 1 → Resume from Phase 1
    • current_phase = 2 → Read design_doc_path
    • current_phase = 3 → Read spec_path
    • current_phase = 4 → Read pr_url, tests_passed
    ↓
Step 3: Load Context Document
    • Phase 1: No doc needed (start fresh)
    • Phase 2: Read design_doc_path
    • Phase 3: Read spec_path (plan document)
    • Phase 4: Read pr_url + review reports
    ↓
Step 4: Continue Execution
    • Resume from current_phase
    • Follow auto-trigger rules
    • Update contract on completion
```

### Key Principles

1. **Contract is state machine:** Records `current_phase` and `evidence`
2. **Plan doc is execution manual:** Contains implementation steps
3. **Design doc is requirement source:** Phase 2+ reads `design_doc_path`
4. **No external dependencies:** All state in contract and documents

## Lifecycle Management

| Status | Location | Retention | Cleanup |
|------|------|--------|---------|
| active | `.cache/workflows/active/` | In progress | Move to archive on completion |
| archive | `.cache/workflows/archive/YYYY-MM/` | 90 days | `gitflow workflow cleanup --older-than 90` |

## CLI Integration

```bash
# List active workflows
gitflow workflow list

# View contract details
gitflow workflow status <workflow_id>

# Archive completed workflow
gitflow workflow archive <workflow_id>

# Clean up expired archives
gitflow workflow cleanup --older-than 90
```

## Error Handling

| Error | Recovery |
|-------|----------|
| Contract file not found | Create new contract (start from Phase 1) |
| Contract JSON corrupted | Restore from backup or rebuild |
| Gate check failed | Return to current Phase to complete evidence |
| worktree leak | `git worktree remove` + `branch -d` |
| auth expired | `gitflow-cli auth status` check, re-login |

## Common Mistakes

❌ Skip gate check · ❌ Inline sub-skill · ❌ Advance to next Phase before contract update · ❌ worktree not cleaned · ❌ Yield to user skip requests
