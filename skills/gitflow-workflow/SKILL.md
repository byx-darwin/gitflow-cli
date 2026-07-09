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
| 4 | `gitflow-pipeline-analyzer → gitflow-issue-triage → gitflow-review` | Post-delivery checks |

### Fast Mode — Required Skills Checklist
Phase 1: gitflow-issue-create (required), brainstorming (optional)
Phase 2: writing-plans (optional, skippable)
Phase 3: subagent-driven-development (required, includes TDD + Code Review)
Phase 4: gitflow-pipeline-analyzer → gitflow-issue-triage → gitflow-review (required)

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

### Phase 1: Clarification

**Entry condition:** none
**Exit condition:** contract `phases.1.status = complete`

1. Read Open Issues → `gitflow-cli issue list --state open`
2. **full mode:** call `superpowers:brainstorming` to clarify requirements
3. Call `gitflow-issue-create` to create Issue
4. **full mode:** call `gitflow-issue-review` to audit and comment
5. Write to contract `phases.1.evidence = { issue_url, comment_id }`
6. Update `phases.1.status = complete`

**Gate 1→2 check:**
- `issue_url` non-empty ✅
- `comment_id` non-empty (fast-mode exemptible)

### Phase 2: Planning

**Entry condition:** Gate 1→2 passed
**Exit condition:** contract `phases.2.status = complete`

1. **full mode:** call `superpowers:writing-plans` to create a full plan
2. User approval → `evidence.user_approved = true`
3. Write to contract `phases.2.evidence = { spec_path, user_approved }`
4. Call `gitflow-quality` gate → ALL CHECKS PASSED
   - Build check / Test check / Coverage check / Format check / Static check / Pre-commit check
5. Update `phases.2.status = complete`

**Gate 2→3 check:**
- `spec_path` non-empty
- `user_approved = true`
- fast mode: skip this Phase, proceed directly to Phase 3

### Phase 3: Execution

**Entry condition:** Gate 2→3 passed (or fast mode skips Phase 2)
**Exit condition:** contract `phases.3.status = complete`

1. Create worktree
2. Call `superpowers:subagent-driven-development`
3. Includes TDD cycle: RED → GREEN → REFACTOR
4. Call `gitflow-pr-create` to create PR
   - **When an Issue is linked, PR body MUST include `Closes #N`** (otherwise the Issue will not auto-close)
5. Write to contract `phases.3.evidence = { branch, pr_url, tests_passed }`
6. Update `phases.3.status = complete`

**Gate 3→4 check:**
- `pr_url` non-empty
- `tests_passed = true`

### Phase 4: Post-Delivery Checks

**Entry condition:** Gate 3→4 passed
**Exit condition:** contract `phases.4.status = complete`

1. Call `gitflow-pipeline-analyzer` → generate pipeline analysis report
2. Call `gitflow-issue-triage` → generate Issue triage report
3. Call `gitflow-review` → generate code review report
4. Write to contract `phases.4.evidence = { pipeline_ok, review_report_path }`
5. Update `phases.4.status = complete`
6. Archive contract → `.cache/workflows/archive/YYYY-MM/`

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
