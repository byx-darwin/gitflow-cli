# TASK-10 Report: Refactor gitflow-release

**Status:** ✅ Complete

## Summary

Rewrote `skills/gitflow-release/SKILL.md` from command-reference style to executable Superpowers skill, conforming to `skill-template.md` and `skill-conventions.md`. Delete operations now require explicit confirmation (irreversible op, medium risk).

## What Changed

| Area | Before | After |
|------|--------|-------|
| Description | Functional description ("Release 操作命令封装...") | Trigger-only ("Use when the user needs to manage releases: create, list, view, edit, upload/download assets, or delete a release on GitHub/GitLab/GitCode.") |
| Structure | Command overview + params + 4 happy-path examples | Overview / When to Use / Core Pattern / Quick Reference / Implementation / Responsibility / Rationalization / Red Flags / Test Scenarios / Success Criteria / Common Mistakes / Trigger Keywords / See Also |
| Boundaries | None declared | Full Responsibility section with ✅ In Scope, ❌ Out of Scope, 🚫 Do Not (5 prohibitions including "Delete without explicit confirmation") |
| Error handling | None | 5-row table covering 401/409/timeout/file-missing |
| Testability | None | 4 scenarios (happy / negative / boundary / error) + 5 Success Criteria |
| Confirmation gating | None | Red Flags require explicit user confirmation for delete, overwrite, publish |
| Rationalization | None | 5-entry counter-table pre-busting common rationalizations |
| Word count | ~440 words (reference-style) | 500 words (exactly at budget limit) |

## Validation

- Word count: **500/500** (within budget)
- Pre-commit hooks: all passed (typos, gitleaks, trailing whitespace, etc.)
- 16-item self-review checklist: all items verified pass
- Bidirectional cross-references:
  - `gitflow-release` → `gitflow-release-helper` ✅ (release-helper already calls `gitflow-cli release create`)
  - `gitflow-release` → `gitflow-auth` ✅
  - `gitflow-release` → `skill-conventions.md` ✅

## Commit

```
bc39f98 refactor(skill): gitflow-release — conform to Superpowers template (#18)
```

Branch: `fix/hook-path-and-auth-parsing` (shared with parallel tasks)

## 16-Item Self-Review Checklist

- [x] `description` matches `/^Use when/i`
- [x] Contains `## Overview` (1 sentence + negative scope)
- [x] Contains `## When to Use` (bilingual keywords + context)
- [x] Contains `## Core Pattern` (executable 4-step skeleton)
- [x] Contains `## Quick Reference` (7-row command cheat-sheet)
- [x] Contains `## Implementation` (steps + preconditions + error handling)
- [x] Contains `## Common Mistakes` (3 entries)
- [x] Contains `## Responsibility` (all 3 sub-sections)
- [x] Contains `## Red Flags` (5 entries)
- [x] Contains `## Trigger Keywords` (6 EN + 6 ZH)
- [x] Contains `## See Also` (3 cross-refs)
- [x] Contains `## Test Scenarios` (4 scenarios, includes negative)
- [x] Contains `## Success Criteria` (5 checkboxes)
- [x] Word count ≤ 500
- [x] No fictional data (all placeholders)
- [x] No narrative examples (pattern language throughout)
- [x] Cross-references verified bidirectional against release-helper

## Key Design Decisions

1. **Delete confirmation is P0**: The `When to Use` table, Red Flags, and three `Do Not` entries all explicitly reinforce that deletion is irreversible and requires explicit user confirmation.

2. **Version/changelog scope is carved out**: The When To Use table explicitly routes "changelog / version decision" to `gitflow-release-helper`. Rationalization table has an entry pre-empting confusion ("That skill drives workflow; this executes CRUD").

3. **Compress strategy**: Combined Steps 1-4 into a single numbered list; collapsed prose in Rationalization/Red Flags; kept all mandatory tables but trimmed cell text with pattern-language density.
