# Task 11 Report: Refactor `gitflow-repo` skill

**Status:** ✅ Complete
**Date:** 2026-07-07
**Issue:** #19
**Commit:** `b167855` — `refactor(skill): gitflow-repo — conform to Superpowers template`

## Summary

Refactored `skills/gitflow-repo/SKILL.md` from a 900+ word command-reference manual into a 497-word Superpowers-compliant skill. Added all required sections (Overview, When to Use, Core Pattern, Preconditions, Error Handling, Responsibility, Red Flags, Trigger Keywords, Test Scenarios, Success Criteria, See Also). Removed narrative examples, fictional data, and output-format templates.

## Before / After

| Metric | Before | After |
|--------|--------|-------|
| Word count | ~900+ | 497 |
| Sections | 2 (命令表格 + 注意事项) | 13 (full template) |
| Responsibility boundaries | ❌ None | ✅ ✅ In Scope / ❌ Out of Scope / 🚫 Do Not |
| Red Flags | ❌ None | ✅ 3 |
| Test Scenarios | ❌ None | ✅ 4 (Happy / Negative / Boundary / Error) |
| Cross-references | ⚠️ 1 (buried in 注意事项) | ✅ 4 (See Also) |
| Description | ❌ Functional description | ✅ Trigger-only (Use when...) |

## 16-Item Checklist

| # | Item | Status |
|---|------|--------|
| 1 | `description` matches `/^Use when/i` | ✅ |
| 2 | Contains `## Overview` | ✅ |
| 3 | Contains `## When to Use` (EN + 中文) | ✅ |
| 4 | Contains `## Core Pattern` | ✅ |
| 5 | Contains `## Quick Reference` | ✅ |
| 6 | Contains `## Implementation` (Preconditions + Error Handling) | ✅ |
| 7 | Contains `## Common Mistakes` | ✅ |
| 8 | Contains `## Responsibility` (3 subsections) | ✅ |
| 9 | Contains `## Red Flags` | ✅ |
| 10 | Contains `## Trigger Keywords` | ✅ |
| 11 | Contains `## See Also` (≥ 2 cross-refs) | ✅ (4) |
| 12 | Contains `## Test Scenarios` (≥ 4, incl. negative) | ✅ |
| 13 | Contains `## Success Criteria` | ✅ |
| 14 | Word count ≤ 500 | ✅ (497) |
| 15 | No fictional data | ✅ |
| 16 | No narrative examples | ✅ |

## P0 Items Addressed

- ✅ Description rewritten as trigger-only (Use when...)
- ✅ Responsibility boundaries added (In Scope / Out of Scope / Do Not)
- ✅ Red Flags added (3 skill-specific)
- ✅ Trigger Keywords added (EN + 中文)
- ✅ Cross-references added (See Also: 4 skills)
- ✅ Token compression: 900+ → 497 words

## P1 Items Addressed

- ✅ Structured template applied
- ✅ Error Handling table added (5 error cases)
- ✅ Preconditions added (3 checks)
- ✅ Quick Reference table added (6 commands)

## Test Scenarios

| # | Type | Description |
|---|------|-------------|
| 1 | Happy Path | Clone repo and view — exits 0, outputs stats |
| 2 | Negative | "open an issue on repo X" — should NOT load gitflow-repo, redirect to gitflow-issue |
| 3 | Boundary | User bundles create + onboarding — confirm visibility, redirect to gitflow-repo-onboarding |
| 4 | Error | Merge conflict during sync — pause, list files, ask user, do NOT push |

## Files Changed

- `skills/gitflow-repo/SKILL.md` — 94 insertions, 262 deletions (net -168 lines)

## Constraints Honored

- ✅ No unrelated files modified
- ✅ No dependencies changed
- ✅ No new dependencies added
- ✅ Commit message follows conventional format
- ✅ Pre-commit hooks passed
