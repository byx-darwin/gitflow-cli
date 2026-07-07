# TASK-18 Report: Refactor gitflow-pr-create

**ID:** TASK-18
**Issue:** #27
**Status:** ✅ Complete
**Effort:** 3-4h (actual: ~1.5h)
**Commit:** `07ad0e3` on `refactor/skills-superpowers`

## Summary

Refactored `skills/gitflow-pr-create/SKILL.md` from an 8-step tutorial narrative into a structured Superpowers skill conforming to `docs/superpowers/templates/skill-template.md`. Added all seven P0 sections (boundaries, prohibition list, red flags, rationalization, keywords, cross-refs, testability hooks) and five P1 sections (structured template, error handling, preconditions, rationalization table, Quick Reference). Replaced narrative prose with pattern-language throughout. Prose word count is now 480 (well within 500-word budget).

## P0 Items (All ✅ Done)

| # | Item | Notes |
|---|------|-------|
| P0-1 | Description rewritten as trigger conditions | Bilingual `Use when...` + `当...时使用` form |
| P0-2 | Boundaries | Responsibility ✅ / ❌ / 🚫 sections; 5 out-of-scope links + 6 prohibitions |
| P0-3 | Prohibition list | 6 ❌ items — protected branch, missing upstream, merge-on-create, unauthorized labels, unconfirmed force-push, cross-repo batch |
| P0-4 | Red Flags | 4 🚩 items + canonical CLI-failure-improvisation defense |
| P0-5 | Keywords + cross-refs | 4 trigger keywords; `See Also` links 5 related skills + template conventions |
| P0-6 | Token compression | No externalization needed; within budget via pattern language |
| P0-7 | Testability hooks | 5 Test Scenarios (Happy, Negative, Boundary, Error ×2) + 6-item Success Criteria |

## P1 Items (All ✅ Done)

| # | Item | Notes |
|---|------|-------|
| P1-1 | Structured template | When to Use + Core Pattern + Quick Reference + Implementation + Error Handling + Flowchart |
| P1-2 | Error Handling | 6-row table with explicit "No improvisation" defense |
| P1-3 | Preconditions | Git repo + CLI + auth status |
| P1-4 | Rationalization table | 3 rows countering likely overstep impulses |
| P1-5 | Quick Reference | 5-row command cheat-sheet (Create / Draft / Ready / Push / Rebase) |

## Files Changed

| File | Change |
|------|--------|
| `skills/gitflow-pr-create/SKILL.md` | Full rewrite to template (~203 lines) |
| `.claude/skills/gitflow-pr-create/SKILL.md` | Synced copy |

## Token Budget

Metric | Count | Status
-------|-------|-------
Prose words (excl code + frontmatter) | 480 | ✅ ≤ 500
Inline code tokens | 62 | excluded
Lines | 203 | reasonable for full template

Measured via canonical convention script: `perl -0 -ne 's/^---\n.*?^---\n//ms; s/` `` `.*?` `` `//gs; s/`[^`]+`//g; @w = /\p{L}+/g; print scalar(@w), "\n"' SKILL.md`

## Self-Review / Observations

- ✅ All P0 + P1 items from the brief addressed
- ✅ Template frontmatter uses `|` multi-line description with bilingual trigger conditions
- ✅ Description avoids functional description — only trigger condition
- ✅ 6-item Responsibility section with In / Out / Do Not subsections
- ✅ 6-item Error Handling table with "No improvisation" defense
- ✅ Mermaid Flowchart covers all decision branches (protected, upstream, base, confirm, success)
- ✅ Test Scenarios: Happy Path, Negative (redirect to `gitflow-pr-review`), Boundary (merge after creation), Error (base outdated + no upstream)
- ✅ Success Criteria checklist — 6 independently-verifiable items
- ✅ Cross-references bidirectional — all 4 related skills (gitflow-pr, gitflow-pr-review, gitflow-pr-inline-review, gitflow-pr-apply-feedback) already reference gitflow-pr-create in their `See Also`
- ✅ No `cargo` / `unwrap` / narrative walkthroughs / fictional data
- ✅ Word count (480) passes the 500-word convention limit

## Key Decisions

1. **Reduced nine tutorial steps to 4 implementation steps + Error Handling table + Flowchart.** The Flowchart carries the decision logic; the prose steps only narrate the failure paths.
2. **Removed embedded realistic command examples.** Original had three full invocations (feature PR, draft PR, fix PR) with full `--body` strings. Moved into Quick Reference table + Core Pattern bash block. The 500-word budget cannot sustain three multi-line commands.
3. **Condensed conventional-commit prefix table into inline list.** The original's 2-column EN/ZH usage table was folded into parenthetical `(feat:, fix:, docs:, refactor:, chore:, test:, perf:)` inside Step 3. Loses table readability but preserves content at ~1/5 token cost.
4. **Did not extract any content to `docs/references/`.** The original had no large parameter tables, no long schema definitions, no compliance checklists that justified externalization.

## Deliverables

- [x] Refactored skill conforming to Superpowers template
- [x] Synced to `.claude/skills/gitflow-pr-create/SKILL.md`
- [x] Committed (`07ad0e3`)
- [x] Self-reviewed against conventions checklist (18/18)
- [x] Report written
