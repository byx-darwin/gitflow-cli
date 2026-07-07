# TASK-5: Refactor gitflow-pr-apply-feedback

**ID:** TASK-5
**Issue:** #33
**Effort:** 4h
**Depends on:** TASK-0
**Risk:** 🔴 High (code modify + commit + push)

## P0 Items
- description: rewrite to "Use when..." trigger format
- boundaries: each modification requires confirmation, push requires explicit confirmation
- prohibition list
- red flags
- trigger keywords
- cross-references (gitflow-pr, gitflow-pr-review, gitflow-pr-inline-review)
- token compression
- testability hooks (RED phase: 4 scenarios)

## P1 Items
- structured template compliance
- error handling
- preconditions
- rationalization excuse counter-table
- flowchart
- Quick Reference

## Context

Highest combination of destructive operations: code modification + git commit + git push. Each step must have explicit user confirmation.

Reference: `docs/superpowers/templates/skill-template.md` (from TASK-0)

Work from: `/Users/byx/Documents/workspace/github.com/byx-darwin/gitflow-cli`
