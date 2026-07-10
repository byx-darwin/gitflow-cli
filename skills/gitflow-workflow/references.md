# gitflow-workflow — Reference

Operations reference for the `gitflow-workflow` orchestrator. Main execution flow: see `SKILL.md`.

## Contract Operations API

### Create Contract

```bash
DATE=$(date -u +%Y-%m-%d)
COUNT=$(ls .cache/workflows/active/ 2>/dev/null | grep "wf-${DATE}" | wc -l)
WORKFLOW_ID="wf-${DATE}-$(printf '%03d' $((COUNT + 1)))"

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
jq --arg next "$((PHASE_NUM + 1))" \
   --arg started_at "$COMPLETED_AT" \
  '.current_phase = ($next | tonumber) | .phases[$next].status = "in_progress" | .phases[$next].started_at = $started_at | .updated_at = $started_at' \
  ".cache/workflows/active/${WORKFLOW_ID}.json" > "${WORKFLOW_ID}.tmp" \
  && mv "${WORKFLOW_ID}.tmp" ".cache/workflows/active/${WORKFLOW_ID}.json"
```

### Read Contract

```bash
jq '.current_phase, .phases | to_entries[] | select(.value.status == "in_progress") | .key' \
  ".cache/workflows/active/${WORKFLOW_ID}.json"
```

## Cross-Session Recovery

Workflows may be interrupted at any Phase. New sessions recover via contract + plan doc.

```
New Session Starts
    ↓
1. List .cache/workflows/active/*.json → find workflow with status != "complete"
2. Read current_phase and evidence
3. Load context:
   • Phase 1: No doc needed (start fresh)
   • Phase 2: Read design_doc_path
   • Phase 3: Read spec_path (plan document)
   • Phase 4: Read pr_url + review reports
4. Resume from current_phase, follow auto-trigger rules
```

**Key principles:** Contract = state machine. Plan doc = execution manual. Design doc = requirement source. All state in contract and documents, no external dependencies.

## Multi-Workflow Concurrency

```
.cache/workflows/
├── active/
│   ├── wf-2026-07-09-001.json  ← feat: TOON
│   └── wf-2026-07-09-002.json  ← fix: pr merge
└── archive/
    └── 2026-07/
        └── wf-2026-07-08-001.json
```

Each workflow uses its own worktree, branch, and contract file — no interference.

## Lifecycle Management

| Status | Location | Retention | Cleanup |
|--------|----------|-----------|---------|
| active | `.cache/workflows/active/` | In progress | Move to archive on completion |
| archive | `.cache/workflows/archive/YYYY-MM/` | 90 days | `gitflow workflow cleanup --older-than 90` |

## CLI Integration

```bash
gitflow workflow list                      # List active workflows
gitflow workflow status <workflow_id>      # View contract details
gitflow workflow archive <workflow_id>     # Archive completed workflow
gitflow workflow cleanup --older-than 90   # Clean up expired archives
```
