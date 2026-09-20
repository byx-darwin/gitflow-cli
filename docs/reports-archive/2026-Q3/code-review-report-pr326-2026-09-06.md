# Code Review Report — PR #326

**Title:** docs(gf-workflow): remove Mode ① background agent execution mode
**Branch:** `feat/325-remove-mode1-background-agent` → `dev`
**Closes:** #325
**Reviewed:** 2026-09-06 (Phase 4 of gf-workflow run `wf-2026-09-06-001`)
**Status at review time:** merged (`1ea04a2`, into `origin/dev`)
**Scope:** docs/skill-only — no Rust code, `Cargo.toml`, or `contract.schema.json` changes

## Summary

Removes gf-workflow's Phase 3 "① Background agent" execution mode
(`isolation: worktree` + `run_in_background`) from three skill files and
renumbers the GO gate menu from a three-option to a two-option choice:
① manual new window (new default) / ② same-session (explicit request only).
The removal is backed by an investigation doc
(`specs/gf-workflow-mode1-removal-design.md`) documenting two empirically
confirmed structural defects in the background-agent mode:

1. A worktree-isolated agent's git operations are hard-restricted to its own
   harness-managed worktree — it cannot target gf-workflow's fixed
   `.worktree/<branch-name>` path.
2. That harness-managed worktree forks from `origin/<default-branch>`, not
   from the orchestrator's `base_branch` (typically `dev`), so any resulting
   branch would silently miss `base_branch`-only commits.

## Files Changed

- `skills/gf-workflow/references.md` — Execution Modes table trimmed to 2
  rows + a "Why no background-executor mode" rationale paragraph citing the
  design doc.
- `skills/gf-workflow/SKILL.md` — GO gate description, Worktree Preflight
  clause, Execution engine clause, and two rationalization-table rows
  updated to reflect the 2-option menu.
- `skills/gf-workflow/gates.md` — GO 闸门 (gate) description trimmed from
  三选一 to 二选一, drops the mattpocock-specific menu-trimming note (no
  longer needed since both `skill_source` values now share an identical
  menu).
- `specs/gf-workflow-mode1-removal-design.md` (new) — investigation +
  design rationale.
- `docs/superpowers/plans/2026-09-06-gf-workflow-mode1-removal.md` (new) —
  implementation plan.
- `specs/index.md` — indexes the two new docs.

## Verification Performed

- `git diff` of all three edited skill files against pre-change `dev`,
  read in full.
- `make check-agent-sync` — exit 0 (clean).
- `grep -rn -i "后台代理\|background agent\|isolation: worktree\|run_in_background"`
  across the three edited files — the only remaining hits are the
  intentional historical/removal-notice mentions (rationalization-table
  entry explaining *why* the mode was removed, and the "Why no
  background-executor mode" note in `references.md`); no active-feature
  references to the removed mode remain.
- `grep -n "③"` across the three edited files — no hits; renumbering from
  three options to two is consistent everywhere (no stray "mode ③"
  left referring to what is now mode ②).
- Cross-checked the two rationalization-table rows in `SKILL.md`
  ("Dispatch a background agent to run /implement" and "Gate 2→3 already
  checked the tree") — both correctly updated to reference the new 2-mode
  reality and cite Issue #325.
- Confirmed `specs/index.md` and the implementation-plan doc are correctly
  cross-linked and match the actual diff.
- No Rust/Cargo/schema files touched — confirmed via `git diff --stat`.

## Findings

None. The change is internally consistent:

- Menu numbering (①②) is renumbered coherently across all three files —
  the old Mode ② (manual new window) becomes the new default Mode ①, the
  old Mode ③ (same-session) becomes Mode ②, with no leftover references to
  a third option or to the old numbering.
- The rationale for removal is well-documented and traceable
  (`specs/gf-workflow-mode1-removal-design.md`), and the two structural
  defects cited are irreparable via documentation alone (the Agent tool's
  `isolation: worktree` exposes no controllable base-ref parameter),
  justifying outright removal over a documented workaround.
- `contract.schema.json`'s `executor` field is correctly identified as a
  free-form string requiring no schema change.
- `specs/index.md` update correctly indexes both new docs per the
  project's `specs/` convention.

## Verdict

**Approve.** Docs/skill-only change, internally consistent, well-supported
by an investigation doc, verified via `make check-agent-sync` and manual
grep-based consistency checks. No code, schema, or CI-relevant changes.

## Note on Review Circumstances

At the time this Phase 4 review step ran, PR #326 had already been merged
into `origin/dev` (merge commit `1ea04a2`) — the review verdict is
submitted as part of gf-workflow's standard Phase 4 delivery-review
audit trail, consistent with prior Phase 4 reports in this repository
(e.g. `docs/code-review-report-pr323-2026-09-05.md`).

`gf review approve 326` was rejected by GitHub (self-approval on one's own
pull request is disallowed by the platform regardless of merge state). The
approve-equivalent verdict above was therefore submitted via
`gf review comment 326` (review id `5123578679`, state `commented`,
2026-09-06T00:39:26Z), carrying the same conclusion text.
