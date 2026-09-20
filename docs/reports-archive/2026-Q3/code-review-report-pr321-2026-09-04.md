# Code Review Report — PR #321 (Post-Merge Audit)

**Title:** fix(gf-workflow): guard worktree shared symlinks against accidental commit
**Branch:** `feat/318-worktree-symlink-exclude-guard` → `dev`
**Issue:** Closes #318
**Author:** byx-darwin
**Reviewed by:** post-merge audit (equivalent rigor to `gf-review`; PR was already merged
before review could be dispatched — see "Note on Review Mechanism" below)
**Review date:** 2026-09-04
**PR state at review time:** merged (mergedAt 2026-09-04T07:09:31Z, merge commit `d9483cd`)

## Note on Review Mechanism

This PR merged (via auto-merge queue) before a `gf-review` verdict could be submitted. The
`gf-review` skill's normal flow requires an open PR to attach a verdict to via `gf`; submitting
against a closed/merged PR is not attempted here. Instead, this report performs the same
correctness / reuse / consistency analysis `gf-review` would apply, independently against the
merged diff (`git diff 4322027...d9483cd`), and records a **verdict-equivalent conclusion**
directly in this document instead of via `gf`. No PR review event was submitted.

## Summary

`gf-workflow`'s Worktree Preflight creates two shared symlinks in every worktree
(`.cache/workflows`, `.claude`) so workflow contracts and Claude config remain accessible.
Neither was previously excluded from git tracking, so a broad `git add -A` / `git commit -a`
during the mandatory TDD loop could sweep them into a commit. Issue #318 records a verified
real-world incident (downstream project `iproost/proxy/api-src`, commit `e7f4254`): both got
committed as `120000` symlink entries, and — resolved from that repo's own root — pointed
outside the repository into a directory shared by other checkouts, causing cross-repo/cross-
session workflow-contract read/write collisions.

This PR is a docs/skill-process-only change (no Rust code, no `Cargo.toml`/lockfile touched):
it makes the Worktree Preflight write both symlink paths to the shared `.git/info/exclude`
immediately after creation, adds a pre-delivery `git diff --summary | grep 'create mode 120000'`
guard as a belt-and-suspenders check before PR/merge, and documents the failure mode and the
verified `info/exclude`-sharing mechanism.

## Scope of Change

Five files, all documentation/skill-process, +503/-2 lines:

| File | Change |
|---|---|
| `skills/gf-workflow/SKILL.md` | Phase 3 Step 1: inline symlink-creation command now also writes to shared `info/exclude`. Phase 3 Step 3: new pre-delivery symlink-commit guard. |
| `skills/gf-workflow/references.md` | Worktree Path Convention example updated with the same `info/exclude` write; new subsection "Why These Symlinks Must Never Reach the Main Branch". |
| `docs/superpowers/specs/2026-09-04-worktree-symlink-exclude-guard-design.md` | New design note (109 lines). |
| `docs/superpowers/plans/2026-09-04-worktree-symlink-exclude-guard.md` | New implementation plan (343 lines), including a scope correction noting `SKILL.md`'s own inline command also needed the fix, not just `references.md`. |
| `docs/index.md` | One new index line for the design doc. |

Verified via `git diff 4322027...d9483cd -- skills/gf-workflow/SKILL.md skills/gf-workflow/references.md docs/index.md` and `git show --stat d9483cd` — matches the PR's declared file list exactly; no changes outside these five files, none under `.github/workflows/`, `Cargo.toml`, or any `.rs` path.

## Review Dimensions

1. **Technical correctness of the `info/exclude` claim** — Independently verified with a
   throwaway `git init` + `git worktree add` sandbox: (a) `git rev-parse --git-common-dir` run
   from inside a linked worktree resolves to the absolute path of the *main* repo's `.git`, not
   a per-worktree copy; (b) a line appended to that shared `info/exclude` from the main tree is
   honored by `git status --ignored` inside a **sibling** worktree created afterward (tested:
   `.claude/` created inside worktree `y` after the main tree wrote `.claude` to
   `.git/info/exclude` shows as `!!` ignored, not untracked). The doc's claim — "worktrees share
   one `info/exclude` with the main tree and all sibling worktrees, there is no per-worktree
   copy" — holds exactly as stated.
2. **Correctness of the pre-delivery guard command** — Independently verified
   `git diff --summary <base>...<head> | grep 'create mode 120000'`: a real symlink committed on
   a feature branch produces the line `create mode 120000 <path>` in `--summary` output, so the
   grep reliably matches. No false-negative risk from the command shape itself (e.g., no
   `--stat`-vs-`--summary` confusion — `--summary` is the correct flag that actually emits mode
   lines).
3. **Cross-file consistency (SKILL.md ↔ references.md)** — Both files' `info/exclude`-write
   snippets are functionally equivalent (idempotent `grep -qxF ... || echo ... >>` pattern) and
   both correctly reference the shared common-dir. `SKILL.md` Step 1/Step 3 point to
   `references.md` → Worktree Preflight for full rationale, and that section (and a dedicated
   "Why These Symlinks Must Never Reach the Main Branch" subsection) exists there as promised —
   confirmed by direct read, not just by the PR description's claim.
4. **Minor finding — variable-casing inconsistency across files (non-blocking).**
   `SKILL.md`'s Phase 3 Step 3 pre-delivery guard is written as:
   `git diff --summary "$base_branch"...HEAD | grep 'create mode 120000'` (lowercase,
   matching the actual `base_branch` contract-field name used everywhere else in `SKILL.md`,
   e.g. line 424's `git checkout $base_branch`). But the "same" command, as documented in
   `references.md` (line 233), the design doc (line 84), and the plan (lines 18, 263, 283), uses
   `"$BASE_BRANCH"` (uppercase) instead. `SKILL.md` explicitly says "Full command + rationale:
   `references.md` → Worktree Preflight," implying the two are meant to be the identical
   command, but the casing differs between the two canonical copies. This is pseudocode for an
   agent to adapt rather than a literal copy-pasted shell script (so it is not a functional bug),
   but it is a real, avoidable inconsistency between the two "source of truth" copies of the same
   guard, and could confuse a future editor who diffs the two commands expecting them to match
   verbatim.
5. **Scope-correction transparency** — The plan document explicitly records and justifies a
   scope expansion beyond the design doc: the design doc said the change was "only
   `references.md`," but planning found `SKILL.md`'s own inline command needed the identical fix
   (since Step 1's handoff text is copied verbatim to background-agent/new-window executors per
   the "must carry the preflight steps verbatim" rule) — otherwise the protection would be
   bypassed in non-same-session execution modes. This is exactly the kind of self-correction a
   reviewer would otherwise have to catch; it was caught and documented before merge.
6. **Iterative self-correction in commit history** — `git log` on the merged range shows five
   feature commits followed by a sixth, `e032282` ("dedupe redundant references.md pointer in
   Phase 3 Step 1"), indicating an inline `/code-review`-style pass caught and fixed a redundant
   pointer before the PR merged — consistent with this repo's established Phase-3 review
   discipline (same pattern seen in PR #320's fence-fix commit).
7. **`docs/index.md` entry accuracy** — The new line's description ("符号链接写入共享
   `info/exclude`（实测：worktree 无独立 exclude，公用主仓库文件）+ Phase 3 交付前新增符号链接
   提交检测") accurately summarizes both changes and matches the actual content of the linked
   design doc. Format (bullet, Issue number, one-line Chinese summary, link) is consistent with
   the surrounding entries.
8. **Regression risk** — None. No code, config, CI workflow, or build artifact is touched. The
   change is documentation/process guidance consumed only by an agent executing `gf-workflow`;
   there is no automated enforcement mechanism that could silently fail (the guard itself is
   agent-executed prose, not a hook or CI check), which is a known characteristic of this
   skill's design (not something this PR needed to fix) rather than a defect introduced here.

## Findings

**1 non-blocking finding** (see Dimension 4 above): `$base_branch` (SKILL.md) vs `$BASE_BRANCH`
(references.md / design doc / plan) casing mismatch between the two copies of the pre-delivery
symlink guard command. Suggest normalizing to one casing (likely lowercase `$base_branch`, to
match the contract-field name already used elsewhere in `SKILL.md`, e.g. line 424) in a future
edit; does not block or require immediate action given the surrounding text is pseudocode for an
agent, not an executable script.

## Verification Evidence (independently checked, not solely from PR description)

- `gh pr view 321 --json mergeCommit,mergedAt,files` — confirmed merge commit `d9483cd`, merge
  time, and file list matching the PR's own declared scope.
- `git diff 4322027...d9483cd -- skills/gf-workflow/SKILL.md skills/gf-workflow/references.md docs/index.md`
  — read in full; matches the summary above.
- Fence-balance check (`grep -c '^```'` per touched/added file) — all even (SKILL.md: 8,
  references.md: 28, design doc: 6, plan: 36); no unclosed or mismatched code blocks.
- Sandbox test (`/tmp/testrepo`, `git init` + `git worktree add`): reproduced the exact
  `git diff --summary <base>...<head> | grep 'create mode 120000'` guard against a real
  committed symlink — correctly matched (`create mode 120000 link1`).
- Sandbox test (continued): wrote `.claude` to the main tree's `.git/info/exclude`, created an
  untracked `.claude/` directory inside a **separately created sibling worktree**, and confirmed
  `git status --porcelain --ignored` from inside that sibling worktree reports it as ignored
  (`!! .claude/`) rather than untracked — independently confirms the doc's central technical
  claim about shared `info/exclude` scope.
- `grep -n '\$base_branch\|\$BASE_BRANCH'` across all four touched/added files — surfaced the
  casing inconsistency recorded as the one finding above.

## Decision (verdict-equivalent, recorded here since no `gf` verdict was submitted)

**Approve, with one non-blocking documentation nit.**

Rationale: the core technical claim (shared `info/exclude` across a clone's worktrees) is
independently verified true; the pre-delivery guard command is independently verified correct;
the scope correction (extending the fix to `SKILL.md`'s own inline command, not just
`references.md`) shows the plan caught a real gap the design doc would have missed; commit
history shows an inline review pass already cleaned up a redundant pointer before merge; and no
code, CI, or build surface is touched, so regression risk is zero. The one finding — a variable-
casing mismatch between two prose copies of the same guard command — is cosmetic (both files are
agent-consumed pseudocode, not executable scripts) and does not warrant blocking or reverting.
Recommend a small follow-up edit to `SKILL.md` line 346 (or the three uppercase occurrences) to
normalize casing, at the author's discretion.
