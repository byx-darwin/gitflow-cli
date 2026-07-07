# Task 12 Report: Refactor gitflow-autoreport-bug (Issue #38)

## Status: Complete

## What Was Implemented

Refactored `skills/gitflow-autoreport-bug/SKILL.md` to conform to the Superpowers
skill template and conventions. This was the project's best skill (only one with
boundary declarations), so changes focused on P0/P1 gaps identified in the analysis.

### P0 Changes

1. **Description rewritten as trigger-only** — was a full workflow description
   ("自动分析 CLI 错误报告，auth cache 加速认证检查..."), now starts with
   "Use when a CLI command fails, a pending bug report needs filing, or a Stop
   Hook detects pending.json." (bilingual).

2. **Added `## Red Flags`** — 6 entries covering canonical patterns plus
   skill-specific ("Fix it while reporting", "Reading `src/`").

3. **Added `## Rationalization Excuses`** — 6 entries countering common Claude
   overstep justifications ("Just looking at src/", "User said fix it", "P0
   urgent, skip validation", "Tech Lead said skip dedup", etc.).

4. **Added `## Test Scenarios`** — 5 scenarios (Happy Path, Negative, Boundary,
   Error, Dedup Hit) with Given/When/Then structure.

### P1 Changes

5. **Structured template** — added `## When to Use`, `## Core Pattern`,
   `## Quick Reference`, `## Common Mistakes`, `## Success Criteria`.

6. **Token compression** — removed inline bash scripts, verbose schema docs,
   and narrative mechanism descriptions. Externalized nothing (all content fits
   within budget).

7. **Added `## See Also`** — 4 cross-references (`/gitflow-workflow`,
   `/gitflow-issue`, `/gitflow-auth`, conventions doc).

## Word Count

- Before: ~680 words (excluded code blocks/frontmatter)
- After: 470 words (within 500 limit)

## Self-Review Checklist (16 items)

| # | Item | Status |
|---|------|--------|
| 1 | `description` matches `/^Use when/i` | Pass |
| 2 | Contains overview paragraph | Pass |
| 3 | Contains `## When to Use` with EN+ZH keywords | Pass |
| 4 | Contains `## Core Pattern` | Pass |
| 5 | Contains `## Quick Reference` | Pass |
| 6 | Contains `## Implementation` | Pass |
| 7 | Contains `## Common Mistakes` | Pass |
| 8 | Contains `## Responsibility` (3 subsections) | Pass |
| 9 | Contains `## Red Flags` | Pass |
| 10 | Contains trigger keywords | Pass (in When to Use table) |
| 11 | Contains `## See Also` (>= 2) | Pass (4) |
| 12 | Contains `## Test Scenarios` (>= 4, 1 negative) | Pass (5, 1 negative) |
| 13 | Contains `## Success Criteria` | Pass |
| 14 | Word count <= 500 | Pass (470) |
| 15 | No fictional data | Pass |
| 16 | No narrative examples | Pass |

### Bidirectional Cross-Reference Note

Forward references to `/gitflow-workflow`, `/gitflow-issue`, `/gitflow-auth`
are in place. Those peer skills do not yet reference back — they are being
refactored in parallel batches. Bidirectional consistency will be verified
during Cluster Coordination (plan Section 5).

## Files Changed

- `skills/gitflow-autoreport-bug/SKILL.md` (+125, -149)

## Commit

```
14c0944 refactor(skill): gitflow-autoreport-bug — conform to Superpowers template
```

## Validation

- Pre-commit hooks: all passed (byte-order-marker, case-conflict, merge-conflict,
  end-of-file, trailing-whitespace, typos, gitleaks)
- No Rust code changes, so no cargo build/test/clippy needed
- Word count verified via `perl` one-liner from conventions doc
