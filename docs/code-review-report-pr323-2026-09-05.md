# Code Review Report — PR #323 (Post-Merge Audit)

**Title:** fix(gf-workflow): compute worktree symlink depth dynamically instead of hardcoding `../../`
**Branch:** `feat/322-worktree-symlink-depth-fix` → `dev`
**Issue:** Closes #322
**Author:** byx-darwin
**Reviewed by:** post-merge audit (equivalent rigor to `gf-review`; PR merged before a
`gf-review` verdict could be dispatched — see "Note on Review Mechanism" below)
**Review date:** 2026-09-05
**PR state at review time:** merged (mergedAt 2026-09-05T09:36:43Z, merge commit `bf91af5`)

## Note on Review Mechanism

This PR was already `MERGED` (not just closed) by the time this review was dispatched.
`gf review approve 323 --body "..."` was attempted and confirmed to fail against the live
GitHub API (`Failed to approve PR #323: GitHub CLI 执行失败`), consistent with GitHub not
accepting new review events on a merged PR through this path. No `gf` review event was
submitted. This report performs the same 6-dimension analysis `gf-pr-review`/`gf-review`
would apply, independently against the merged diff
(`git diff 87ad8d8...bf91af5`, verified via `gh pr diff 323` and by reading the actual merged
file contents at `origin/dev`), and records a **verdict-equivalent conclusion** here instead.

## Summary

`gf-workflow`'s Worktree Preflight step (`SKILL.md` Phase 3 Step 1 / `references.md` Worktree
Preflight example) previously hardcoded the relative symlink depth for
`.cache/workflows`/`.claude` as `../../`. Issue #322 reports this breaks whenever
`worktree_path` has more than one path segment — which is the routine case, since branch
names follow `feat/<issue-number>-<short-description>` and worktrees live at
`.worktree/<branch-name>`, i.e. `.worktree/feat/89-desc` (3 segments). The design doc goes
further: empirically reproducing the symlink resolution with real `mkdir`+`ln -s` shows the
old `../../` was *already* wrong even for the single-segment case it was presumably written
for (it resolves to `.worktree/`, one level short of the repo root, in every case) — the
slash-containing branch name only made the pre-existing bug easier to hit.

This is a pure documentation/skill-process change (four files, no Rust/CI/config touched):
`SKILL.md` line 344 and `references.md`'s worked example both replace the hardcoded value with
a `segs`/`ups`/`rel`-based computation (`ups = segments-in-worktree_path + 1`) plus a
post-creation `test -d` existence self-check, and `references.md` gains a new
"Why the Symlink Depth Is Computed, Not Hardcoded" subsection with an empirically-verified
two-row table.

## Scope of Change

Four files, +? / -? (per `gh pr view --json files`): `docs/superpowers/plans/2026-09-04-worktree-symlink-depth-fix.md` (new, +250), `docs/superpowers/specs/2026-09-04-worktree-symlink-depth-fix-design.md` (new, +108), `skills/gf-workflow/SKILL.md` (+1/-1), `skills/gf-workflow/references.md` (+61/-11). No changes to `deny.toml`, `.pre-commit-config.yaml`, `rust-toolchain.toml`, `Cargo.toml`, any `.rs` file, or `.github/workflows/`.

## Review Dimensions

1. **Correctness — formula.** Verified independently, not just re-derived from the doc's own
   claim: a relative symlink at `$WORKTREE_PATH/.cache/workflows` resolves starting from its
   *containing directory* `$WORKTREE_PATH/.cache/`, which is one segment deeper than
   `$WORKTREE_PATH`. For `.worktree/foo` (2 segments via `awk -F/ '{print NF}'`), the
   containing dir is 3 segments deep, so `ups=3` (`../../../`) is required to reach the repo
   root — matches the table row exactly (up1→`.worktree/foo/`, up2→`.worktree/`,
   up3→repo root). For `.worktree/feat/89-desc` (3 segments), containing dir is 4 segments
   deep, `ups=4` — matches the second row (up1→`.worktree/feat/89-desc/`,
   up2→`.worktree/feat/`, up3→`.worktree/`, up4→repo root). The `ups = segs + 1` formula is
   correct and both worked examples check out arithmetically.
2. **Correctness — "old value fails on the single-segment case too" claim.** Verified: for
   `.worktree/foo`, `../../` from the symlink's containing dir (`.worktree/foo/.cache/`) only
   climbs to `.worktree/` (up1→`.worktree/foo/`, up2→`.worktree/`), one level short of repo
   root — exactly matching the table's "Hardcoded `../../` resolves to: `.worktree/` — not the
   repo root" claim for that row. This corroborates the PR's headline claim that the bug wasn't
   solely a "branch name contains `/`" edge case.
3. **Cross-file consistency (`SKILL.md` ↔ `references.md`).** Both snippets use identical
   variable names `segs`, `ups`, `rel` and identical logic
   (`segs=$(awk -F/ '{print NF}' <<< ...)`; `ups=$((segs + 1))`;
   `rel=$(printf '../%.0s' $(seq 1 "$ups"))`). `SKILL.md` uses the pre-existing `<worktree-path>`
   angle-bracket placeholder convention already used elsewhere in that same table row (e.g.
   `mkdir -p <worktree-path>/.cache`), while `references.md`'s worked example uses the concrete
   shell variable `$WORKTREE_PATH` — this is a deliberate, pre-existing stylistic split (prose
   template vs. runnable example), not an inconsistency. Both self-check clauses
   (`test -d ... || { ...; exit 1; }`) produce the same diagnostic shape
   (`segs=$segs ups=$ups`).
4. **GFM table pipe-escaping.** `SKILL.md`'s new command is embedded as one inline code span
   inside a `|`-delimited table cell (Phase 3 table, row "1"), and its self-check clause
   contains a `||` shell operator, written as `\|\|` in the source. Checked directly against
   the merged file content (not just the diff) with a pipe-aware cell splitter: this table row
   parses to exactly the expected number of cells with the escape in place. This matches GFM's
   documented (if CommonMark-code-span-contradicting) special-case rule that a table's raw-text
   cell-splitting happens *before* code-span parsing, so a literal `|` inside a code span within
   a table cell must still be written as `\|` — and GitHub's reference renderer (cmark-gfm)
   correctly strips the backslash and displays a bare `|`. The escaping here is both necessary
   and correctly applied. (Pre-existing, unmodified content in the *same* row — the
   `grep -qxF '.cache/workflows' "$EF" || echo ...` `info/exclude` clause, present since commit
   `656970b`, predating this PR — uses an un-escaped `||` inside the same table cell, which by
   the same rule would already split that row incorrectly; this is out of this PR's diff scope and not
   introduced or worsened by it, but is flagged here as a pre-existing latent issue worth a
   separate follow-up.)
5. **Rationale subsection self-consistency.** The new "Why the Symlink Depth Is Computed, Not
   Hardcoded" subsection's formula (`ups = (number of "/"-separated segments in worktree_path) + 1`)
   and its two-row empirical table are internally consistent with each other and with the design
   doc's own Chinese-language verification table
   (`docs/superpowers/specs/2026-09-04-worktree-symlink-depth-fix-design.md`), which shows the
   same segment/ups pairs (2→3, 3→4) and the same resolved-path traces
   (`../../../.cache/workflows`, `../../../../.cache/workflows` → repo root). No numeric or
   narrative contradiction found between the two documents or within either one.
6. **Structural integrity.** `grep -n '\.\./\.\.' ` on the merged files shows only intentional,
   contextual leftover mentions of `../../` (discussing the *old*, now-replaced value, or the
   unrelated pre-existing "Why These Symlinks Must Never Reach the Main Branch" section) — no
   stray un-migrated hardcoded depth remains. The new heading appears exactly once, immediately
   before the pre-existing "Why These Symlinks Must Never Reach the Main Branch" heading, per
   the plan. Fenced-code-block count in `references.md` is even (30) — no unclosed block
   introduced by the worked-example replacement.
7. **Documentation-index completeness (finding).** `docs/index.md` was **not** part of this
   PR's file list (confirmed via `gh pr view --json files` and `git show origin/dev:docs/index.md | grep`
   for the new design-doc filename / issue number — zero matches). This project's own
   established convention — visible in `docs/index.md` itself (e.g. the line-30 entry added
   alongside PR #321/#318's sibling fix) and stated explicitly in `CLAUDE.md` ("For docs,
   inspect `docs/`, place new files there, and update `docs/index.md`") — is to add a one-line
   index entry whenever a new `docs/superpowers/specs/*.md` design doc lands. This PR adds
   `2026-09-04-worktree-symlink-depth-fix-design.md` and a companion plan but does not add the
   corresponding index line.
8. **Regression risk.** None. No Rust code, `Cargo.toml`/lockfile, CI workflow, or build
   artifact is touched; the change is agent-consumed documentation with no automated enforcement
   surface, matching the risk profile of the prior sibling fix (PR #321).

## Findings

**1 non-blocking finding:** `docs/index.md` was not updated with an entry for the new design
doc (`docs/superpowers/specs/2026-09-04-worktree-symlink-depth-fix-design.md`), contrary to
this repository's established documentation-indexing convention (see Dimension 7 above).
Suggest a small follow-up commit adding the missing index line, at the author's discretion —
does not require reverting or blocking, since the fix's substantive content is independently
verified correct.

**1 informational note (out of PR scope, not a blocking finding):** the pre-existing,
unmodified `grep -qxF '.cache/workflows' "$EF" || echo ...` clause in the same `SKILL.md` table
row uses an un-escaped `||` inside a table cell's code span (predates this PR, commit
`656970b`). Flagged for awareness only; not introduced or worsened by PR #323.

## Verification Evidence (independently checked, not solely from PR description)

- `gh pr view 323 --json number,state,mergedAt,mergeCommit,baseRefName,headRefName,files` —
  confirmed `MERGED`, merge commit `bf91af5`, base `dev`, head `feat/322-worktree-symlink-depth-fix`,
  and the four-file scope matching the PR's own declared summary.
- `git fetch origin dev` + `git show origin/dev:skills/gf-workflow/SKILL.md` and
  `...references.md` — read the actual **merged file contents** directly (not only the diff) to
  verify shell syntax and table structure in situ.
- Custom pipe-aware Python cell-splitter run against `SKILL.md` line 344 (the modified table
  row) — confirms the new `\|\|` escape is correctly placed and the row's cell count is as
  expected once the pre-existing, unrelated `info/exclude` line's un-escaped `||` (present since
  `656970b`, before this PR) is accounted for separately.
- `grep -n '\.\./\.\.' ` and `grep -n '^### '` and `awk '/^```bash$/{c++} /^```$/{c++} END{print c}'`
  against the merged `references.md` — no stray hardcoded depth, correct single occurrence and
  placement of the new heading, even fence count.
- Manual arithmetic re-derivation of both worked table rows (`.worktree/foo` → segs=2, ups=3;
  `.worktree/feat/89-desc` → segs=3, ups=4) against the stated symlink-resolution mechanics —
  both check out.
- `gh pr view 323 --json files` cross-referenced against `git show origin/dev:docs/index.md | grep -n "worktree-symlink-depth-fix\|322"` (zero matches) — confirms the missing index-entry
  finding.
- `gf review approve 323 --body "..."` attempted and confirmed to fail (merged PR), documented
  under "Note on Review Mechanism" rather than silently skipped.

## Decision (verdict-equivalent, recorded here since no `gf` verdict was submitted)

**Approve, with one non-blocking documentation nit.**

Rationale: the core technical fix — replacing a hardcoded, provably-wrong-in-all-cases relative
symlink depth with a `segs + 1` formula plus a post-creation existence self-check — is
independently re-derived and confirmed correct for both worked examples; the two documents
(`SKILL.md`, `references.md`) are internally consistent in variable naming and logic; the new
rationale subsection's formula and empirical claims are self-consistent with each other and
with the design doc; the required Markdown table pipe-escaping for the new self-check clause is
correctly applied per GFM's code-span-inside-table-cell escaping rule; and no code, CI, or build
surface is touched, so regression risk is zero. The one finding — a missing `docs/index.md`
entry for the new design doc, contrary to this project's own convention — is a documentation-
completeness gap, not a correctness defect, and does not warrant blocking or reverting.
Recommend a small follow-up commit adding the missing index line, at the author's discretion.
