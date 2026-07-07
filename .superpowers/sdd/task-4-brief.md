# TASK-4: Refactor gitflow-review

**ID:** TASK-4
**Issue:** #39
**Effort:** 4h
**Depends on:** TASK-0
**Risk:** 🔴 High (merge gate — approve affects PR mergeability)

## P0 Items
- description: rewrite to "Use when..." trigger format
- boundaries: approve requires prior pr-review analysis
- prohibition list
- red flags
- trigger keywords
- cross-references (gitflow-pr, gitflow-pr-review, gitflow-pr-inline-review, gitflow-pr-apply-feedback)
- testability hooks (RED phase: 4 scenarios)

## P1 Items
- structured template compliance
- error handling
- preconditions
- rationalization excuse counter-table
- flowchart (approve vs submit decision)
- Quick Reference

## Context

Approve directly affects PR mergeability. Must require prior pr-review analysis. Also has approve-vs-submit ambiguity that needs a flowchart.

Reference: `docs/superpowers/templates/skill-template.md` (from TASK-0)

Work from: `/Users/byx/Documents/workspace/github.com/byx-darwin/gitflow-cli`
