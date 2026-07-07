# TASK-14 Report: Refactor gitflow-issue-triage

**Status:** Complete
**Commit:** `ffd7d74` — `refactor(skill): gitflow-issue-triage — conform to Superpowers template`
**Branch:** `main`

## What Was Done

Rewrote `skills/gitflow-issue-triage/SKILL.md` from a narrative workflow manual into an executable skill conforming to `docs/superpowers/templates/skill-template.md` and `docs/superpowers/templates/skill-conventions.md`.

| Before | After |
|--------|-------|
| description = full 5-step workflow (violation) | description = "Use when..." trigger-only, bilingual |
| 5-step narrative walkthrough | Core Pattern + Quick Reference + 4-step Implementation |
| No boundaries | Responsibility (In Scope / Out of Scope / Do Not) |
| No idempotency | Explicit idempotency declaration in Step 3 |
| No error handling | 4-row Error Handling table |
| No testability | 4 test scenarios (Happy / Negative / Boundary / Error) |
| No red flags | 4 red flags + 3 rationalization excuses |
| No cross-reference | See Also → 3 peer issue/label skills |
| ~440 words (Chinese narrative) | 492 words (bilingual, under 500 limit) |

## P0 Items Addressed

- ✅ Description rewritten to trigger-only format (`Use when...`)
- ✅ Boundaries declared: In Scope (5 bullets), Out of Scope (with redirects), Do Not (5 concrete prohibitions)
- ✅ Idempotency declaration: `triage:done` issues are skipped on re-runs; `--add` only, never `--remove`
- ✅ Red flags (4 entries) including skill-specific "Remove type:unknown labels" and "Close the low-priority ones"
- ✅ Keywords covered (4 English + 4 Chinese pairs in Trigger Keywords table; 4 rows in When to Use)
- ✅ Cross-refs to `gitflow-issue`, `gitflow-issue-review`, `gitflow-label-milestone`
- ✅ Testability hooks: 4 test scenarios + 3 Success Criteria with observable artifacts

## P1 Items Addressed

- ✅ Structured template: Overview, When to Use, Core Pattern, Quick Reference, Implementation, Common Mistakes
- ✅ Error handling table (auth / rate-limit / label failure / empty list)
- ✅ Preconditions (CLI installed, auth valid, inside git repo)
- ✅ Rationalization table (3 entries — meets minimum for medium-risk skill)
- ✅ Quick Reference cheat-sheet (3 commands)
- ✅ Decision table for type/priority classification (rows = types, columns = signals)

## Self-Review Checklist (18 items)

- [x] `description` matches `/^Use when/i` (English), bilingual, trigger-only
- [x] `## Overview` present (2 sentences: what + idempotency redirect)
- [x] `## When to Use` with English + Chinese keywords (4 rows including negative)
- [x] `## Core Pattern` executable skeleton (4-line bash)
- [x] `## Quick Reference` (3 rows)
- [x] `## Implementation` (preconditions, 4 steps, error handling)
- [x] `## Common Mistakes` (2 entries — meets minimum)
- [x] `## Responsibility` with all 3 sub-sections
- [x] `## Red Flags` (4 entries — meets skill-specific requirement)
- [x] `## Trigger Keywords` (4 pairs)
- [x] `## See Also` (3 cross-refs — meets ≥2 minimum)
- [x] `## Test Scenarios` (4 scenarios including 1 negative)
- [x] `## Success Criteria` (3 checkboxes, all observable)
- [x] Word count = 492 (≤500 excluding code blocks/frontmatter/HTML comments)
- [x] No fictional data (uses placeholders: `<n>`, `<t>`, `<p>`)
- [x] No narrative examples — pattern language throughout
- [x] Cross-references: `gitflow-label-milestone` already bidirectionally links back; peer issue skills being refactored in parallel batches
- [x] Passes `superpowers:writing-skills` review (RED-GREEN-REFACTOR cycle observed)

## Word Count

492 words by the canonical counter (excluding fenced code blocks, frontmatter, HTML comments; inline code tokens count as 1 each per convention §1.2).

## Files Modified

- `skills/gitflow-issue-triage/SKILL.md` — complete rewrite (104 insertions, 151 deletions)
