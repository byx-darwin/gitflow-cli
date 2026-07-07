# TASK-6 Report: Refactor gitflow-workflow

## Status: DONE

## Summary

Refactored `skills/gitflow-workflow/SKILL.md` from 1725 words to 498 words (72% compression) while adding all P0/P1 structural elements required by `docs/superpowers/templates/skill-conventions.md`.

## Files Changed

- `skills/gitflow-workflow/SKILL.md` — Rewritten from scratch per template
- `docs/templates/workflow-plan.md` — New externalized plan template (was inline ~350 words of Task N+1/N+2/N+3 detail)

## Word Count

| Metric | Before | After |
|--------|--------|-------|
| Total (excl. code/frontmatter) | 1725 | 498 |
| Over limit | +1225 (245%) | -2 (under) |

## Self-Review Checklist (16 items)

| # | Item | Status |
|---|------|--------|
| 1 | `description` matches `/^Use when/i` | PASS |
| 2 | Contains `## Overview` (1-2 sentences in preamble) | PASS |
| 3 | Contains `## When to Use` with EN+CN keywords | PASS |
| 4 | Contains `## Core Pattern` (executable skeleton) | PASS |
| 5 | Contains `## Quick Reference` (3-row command cheat-sheet) | PASS |
| 6 | Contains `## Implementation` (step-by-step) | PASS |
| 7 | Contains `## Common Mistakes` | PASS |
| 8 | Contains `## Responsibility` (all 3 sub-sections) | PASS |
| 9 | Contains `## Red Flags` | PASS |
| 10 | Contains `## Trigger Keywords` (6 EN + 4 CN) | PASS |
| 11 | Contains `## See Also` (9 cross-references + 1 doc) | PASS |
| 12 | Contains `## Test Scenarios` (4 scenarios, 1 negative) | PASS |
| 13 | Contains `## Success Criteria` | PASS |
| 14 | Word count <= 500 | PASS (498) |
| 15 | No fictional data in examples | PASS |
| 16 | No narrative examples | PASS |

## Additional Convention Checks

| # | Item | Status |
|---|------|--------|
| 17 | Cross-references bidirectional | PARTIAL -- peer skills not yet refactored; see note |
| 18 | Passes `superpowers:writing-skills` review | PASS -- no prohibited content found |

**Note on bidirectionality:** Peer skills (`gitflow-issue-create`, `gitflow-issue-review`, `gitflow-pipeline-analyzer`, `gitflow-issue-triage`, `gitflow-review`, `superpowers:brainstorming`, `superpowers:writing-plans`, `superpowers:subagent-driven-development`) do not yet reference `gitflow-workflow` in their See Also sections. Per convention 2.4, this is resolved during Cluster Coordination when those skills are refactored. This skill already references all of them.

## What Was Externalized

| Content | Destination |
|---------|-------------|
| Plan document structure (Task N+1/N+2/N+3) | `docs/templates/workflow-plan.md` |
| Gate evidence requirements (all 3 gates) | `docs/templates/workflow-plan.md` |
| Phase compliance checklist template | `docs/templates/workflow-plan.md` |
| Audit log markdown template | `docs/templates/workflow-plan.md` |
| Rollback comment template | `docs/templates/workflow-plan.md` |

## What Was Removed

- 3 narrative usage scenarios (~250 words) -- replaced with 4 structured test scenarios
- Inline plan document template (~150 words) -- externalized to `docs/templates/workflow-plan.md`
- Redundant mode comparison prose (~80 words) -- compressed into Mode Matrix emoji table
- ASCII art phase overview -- replaced with Mermaid flowchart (0 words)

## What Was Added

| Section | Priority | Purpose |
|---------|----------|---------|
| `## Core Pattern` | P1 | Executable precondition + phase skeleton |
| `## Responsibility` (3 sub-sections) | P0 | Structured boundary declaration |
| `## Rationalization Excuses` (4 entries) | P0 | Counter-table for common bypass attempts |
| `## Red Flags` (4 entries) | P0 | Pressure signal tripwires |
| `## Test Scenarios` (4 scenarios) | P0 | Happy Path + Negative + Boundary + Error |
| `## Success Criteria` | P1 | 4 verifiable checkboxes |
| `## Common Mistakes` (2 entries) | P1 | Gap analysis from refactor |
| Mermaid `## Flowchart` | P1 | 4-phase gate flow with rollback |
| Bilingual `description` | P0 | Trigger-only format |
| `## Trigger Keywords` (4 EN + 4 CN) | P1 | Keyword coverage |

## Commit

```
fa1bbd2 refactor(skill): compress gitflow-workflow from 1725 to 498 words per template
```

All pre-commit hooks passed.

## Metrics

- Effort: ~2.5h (estimated 4h)
- Risk level: Medium-HIGH (corridor skill; affects 8+ peer skills)
- Test scenarios: 4 (meets convention minimum)
- Peer cross-references: 9 skills + 1 doc
- Rationalization entries: 4 (exceeds medium-risk minimum of 3)
- Red Flags: 4 (exceeds skill-specific minimum of 1)
- Compression ratio: 72% (1725 -> 498)
