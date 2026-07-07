# TASK-20 Report: Refactor gitflow-pr (Delegation Model)

**ID:** TASK-20
**Issue:** #32
**Date:** 2026-07-07
**Status:** Complete

---

## Summary

Refactored `gitflow-pr` parent skill to conform to the Superpowers template and added a delegation model connecting it to 4 child skills. All 5 skills (parent + 4 children) now share a unified routing contract via `## 🔁 Delegation Rules` tables.

## What Changed

### Parent Skill: `gitflow-pr`

- Rewrote YAML `description` frontmatter to `Use when...` trigger-only format.
- Replaced flat CLI reference dump with structured template: `When to Use`, `Core Pattern`, `Quick Reference`, `Implementation`, `Responsibility`.
- Added `## 🔁 Delegation Rules` table that routes user intent to the correct child skill:
  - Create → `/gitflow-pr-create`
  - Review → `/gitflow-pr-review`
  - Inline → `/gitflow-pr-inline-review`
  - Apply feedback → `/gitflow-pr-apply-feedback`
  - All other 7 subcommands → direct execution in parent
- Externalized the 11-subcommand parameter tables to `docs/references/gitflow-pr-params.md` (which was already present from previous work).
- Added 4 test scenarios, 4 success criteria, 4 rationalization excuses, 4 red flags, 12 trigger keywords.
- Final word count: **494** (under 500-word budget).

### Child Skills (4 files)

| Skill | Change | Final WC |
|-------|--------|----------|
| `gitflow-pr-create` | Added `## 🔁 Delegation Rules` table | 471 |
| `gitflow-pr-review` | Added `## 🔁 Delegation Rules` table; added `gitflow-review` cross-ref | 497 |
| `gitflow-pr-inline-review` | Added `## 🔁 Delegation Rules` table | 489 |
| `gitflow-pr-apply-feedback` | Added `## 🔁 Delegation Rules` table | 454 |

Each delegation table declares:
- **This skill** rows (what it owns)
- **Sibling** rows (what it delegates to `/gitflow-pr-*`)
- **Parent** rows (lifecycle ops → `/gitflow-pr`)
- **External** rows (e.g., CI → `/gitflow-pipeline-analyzer`)

### New Files

- `docs/superpowers/tests/skills/gitflow-pr-test.md` — 5 stress scenarios for the parent skill (delegation routing, negative trigger, boundary, 404 error, authority+urgency pressure).

### Updated Files

- `docs/references/gitflow-pr-params.md` — already existed; confirmed current.

---

## Delegation Model Contract

The delegation model follows bidirectional consistency:

```
                           ┌──────────────────┐
                           │   gitflow-pr     │
                           │  (parent/router) │
                           └────────┬─────────┘
                                    │
              ┌─────────────────────┼─────────────────────┐
              │                     │                     │
              ▼                     ▼                     ▼
   ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐
   │ gitflow-pr-create│  │ gitflow-pr-review│  │gitflow-pr-inline │
   │                  │◄─┤                  │◄─┤     -review      │
   │ Validate branch  │  │ 6-dim assess     │  │ Per-line diff    │
   │ Collect title    │  │ Submit verdict   │  │ Publish comments │
   │ Invoke pr create │  │                  │  │                  │
   └──────────────────┘  └──────────────────┘  └──────────────────┘
              │                     │                     │
              │                     │                     │
              └─────────────────────┼─────────────────────┘
                                    │
                                    ▼
                         ┌──────────────────────┐
                         │gitflow-pr-apply      │
                         │     -feedback        │
                         │                      │
                         │ Fetch comments       │
                         │ Fix + test + commit  │
                         │ Resolve + push       │
                         └──────────────────────┘
```

Routing example: User says "create a PR for the auth feature"

1. Claude loads `gitflow-pr` (top-level match)
2. Delegation table routes to `/gitflow-pr-create`
3. `gitflow-pr-create` validates branch, collects metadata, invokes CLI
4. On success, delegation table routes back to `/gitflow-pr-review` (next logical step)

---

## Self-Review Checklist (per skill)

| Criterion | pr | create | review | inline | apply |
|-----------|----|--------|--------|--------|-------|
| `description` matches `/^Use when/i` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `## When to Use` with trigger keywords | ✓ | ✓ | ✓ | ✓ | ✓ |
| `## Core Pattern` skeleton | ✓ | ✓ | ✓ | ✓ | ✓ |
| `## Quick Reference` cheat-sheet | ✓ | ✓ | ✓ | ✓ | ✓ |
| `## Implementation` steps | ✓ | ✓ | ✓ | ✓ | ✓ |
| `## Responsibility` (3 subsections) | ✓ | ✓ | ✓ | ✓ | ✓ |
| `## 🚫 Do Not` prohibition list (≥3) | ✓ | ✓ | ✓ | ✓ | ✓ |
| `## Red Flags` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `## Rationalization Excuses` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `## Trigger Keywords` (≥5) | ✓ | ✓ | ✓ | ✓ | ✓ |
| `## See Also` (≥2 cross-refs) | ✓ | ✓ | ✓ | ✓ | ✓ |
| `## Test Scenarios` (≥4, 1 negative) | ✓ | ✓ | ✓ | ✓ | ✓ |
| `## Success Criteria` (checkboxes) | ✓ | ✓ | ✓ | ✓ | ✓ |
| `## 🔁 Delegation Rules` | ✓ | ✓ | ✓ | ✓ | ✓ |
| Word count ≤ 500 | ✓(494) | ✓(471) | ✓(497) | ✓(489) | ✓(454) |
| No fictional data in examples | ✓ | ✓ | ✓ | ✓ | ✓ |
| Cross-references bidirectional | ✓ | ✓ | ✓ | ✓ | ✓ |

---

## RED → GREEN → REFACTOR Trace

| Phase | Evidence |
|-------|----------|
| RED | Skill analysis docs at `docs/research/skill-analysis-gitflow-pr*.md` established 4-dimension gap (structure, boundary, testability, Superpowers alignment). Each dimension scored the skills as 不合格 or 需改进. |
| GREEN | Added the minimum sections needed: `## 🔁 Delegation Rules`, `## Responsibility` with 3 subsections, `## Red Flags`, `## Rationalization Excuses`, `## Test Scenarios`, `## Success Criteria`. Verified via canonical word-count script. |
| REFACTOR | Trimmed all 5 skills under 500 words by condensing prose, combining table rows, using pattern language over narrative. Reconciled cross-references bidirectionally (e.g., pr-review ↔ gitflow-review). Created parent stress-test file. |

---

## Bidirectional Cross-Reference Matrix

| From \ To | pr | create | review | inline | apply |
|-----------|----|--------|--------|--------|-------|
| **pr** | — | ✓ | ✓ | ✓ | ✓ |
| **create** | ✓ | — | ✓ | ✓ | ✓ |
| **review** | ✓ | ✓ | — | ✓ | ✓ |
| **inline** | ✓ | ✓ | ✓ | — | ✓ |
| **apply** | ✓ | ✓ | ✓ | ✓ | — |

All 5 skills form a fully connected cluster with bidirectional links.

---

## Files Changed

- `skills/gitflow-pr/SKILL.md` — parent refactor (already committed in 92ace82)
- `skills/gitflow-pr-create/SKILL.md` — delegation table + trim
- `skills/gitflow-pr-review/SKILL.md` — delegation table + cross-ref + trim
- `skills/gitflow-pr-inline-review/SKILL.md` — delegation table + trim
- `skills/gitflow-pr-apply-feedback/SKILL.md` — delegation table + trim
- `docs/superpowers/tests/skills/gitflow-pr-test.md` — new stress test file (5 scenarios)
- `docs/references/gitflow-pr-params.md` — already existed; confirmed

---

## Commits

- `e0d32b2` refactor(skill): gitflow-pr child skills — add delegation model + word-count compliance
- `92ace82` chore(skill): apply lint-driven sync to gitflow-pr SKILL.md (parent refactor)

---

## Known Issues / Follow-ups

1. The pre-existing `docs/superpowers/tests/skills/gitflow-pr-review-test.md` still has 12 scenarios in the old Chinese format. A future task should migrate it to the standard `Given/When/Then` 5-scenario stress format used by the new `gitflow-pr-test.md`.
