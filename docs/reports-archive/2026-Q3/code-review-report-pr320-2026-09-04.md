# Code Review Report — PR #320

**Title:** ci(e2e): verify e2e-regression alert CJK title dedup reliability
**Branch:** `feat/310-e2e-regression-dedup-verification` → `dev`
**Issue:** Closes #310
**Author:** byx-darwin
**Reviewed by:** gf-review (Phase 4 post-delivery check, gf-workflow standard mode)
**Review date:** 2026-09-04
**PR state at review time:** merged (created and merged 2026-09-04T06:21:06Z / 06:21:24Z)

## Summary

This PR is the follow-up spike record for #292/PR #309's code-review finding #1
(`docs/code-review-report-pr309-2026-09-03.md`): the `notify-on-schedule-failure` job in
`.github/workflows/e2e-tests.yml` dedups scheduled-failure alerts by searching for a fixed,
almost-entirely-CJK title (`定时 E2E 回归失败`) via `gh issue list --search "in:title ..."`.
GitHub's full-text search has known unreliable tokenization for CJK text without word-boundary
spaces, so Issue #310 asked whether this search actually finds the existing Issue in practice
(risk: silent dedup failure, Issue pile-up on every scheduled failure).

The author ran the spike: created a throwaway test Issue (#319) with the exact production title
and label, then ran the exact query the workflow uses, 3 times (including delays to rule out
search-index propagation lag). All 3 attempts correctly matched. Per Issue #310's acceptance
criteria ("若命中正常" / search-hits branch), the conclusion is **no code change needed** — this
PR is documentation-only, recording that outcome and closing the loop.

**No Rust code, no CI workflow files, no production behavior changed.**

## Scope of Change

Three files, all documentation, +175/-0 lines, no deletions:

| File | Change |
|---|---|
| `docs/code-review-report-pr309-2026-09-03.md` | Appends a new "Follow-up: Dedup Verification (Issue #310)" section (31 lines) recording method, result, conclusion, and a correction to the report's own earlier framing. |
| `docs/superpowers/plans/2026-09-04-e2e-regression-dedup-verification.md` | New implementation plan (104 lines) for the 2-task spike close-out (annotate report, close Issue #310). |
| `docs/superpowers/specs/2026-09-04-e2e-regression-dedup-verification-design.md` | New spike design note (40 lines): question, method, result, conclusion, scope. |

Verified via `gh pr diff 320` — confirmed no hunks outside these three files, no changes under
`.github/workflows/` or any `.rs`/`Cargo.toml` path.

## Review Dimensions

1. **Correctness of the verification method** — The spike replicates the exact production
   input: test Issue #319 was created with the literal title `定时 E2E 回归失败` and label
   `e2e-regression` (confirmed via `gf issue view 319`), and the query run
   (`gh issue list --label "e2e-regression" --state open --search "in:title 定时 E2E 回归失败" ...`)
   matches the search shape actually used by `notify-on-schedule-failure` in
   `.github/workflows/e2e-tests.yml`. Repeating 3 times with delays is a reasonable, if modest,
   sample size to rule out both immediate false negatives and search-index propagation lag —
   appropriate for a low-cost spike where the fallback (client-side exact match) is already
   designed and documented, not something that needs inventing under this PR.
2. **Conclusion follows from evidence** — Issue #310's acceptance criteria explicitly branch on
   outcome: "若命中正常，关闭本 Issue 并在 PR #309 的 review 报告里记一笔已验证" (if it hits
   reliably, close the Issue and record "verified" in the PR #309 review report). 3/3 matches is
   exactly that branch; the PR's actions (append verified note, close #310, no workflow change)
   are the literal prescribed action, not a discretionary call.
3. **Test-artifact hygiene** — Test Issue #319 was commented and closed immediately after the
   3 queries completed (confirmed: `gf issue view 319` shows `state: closed`, updated 1 minute
   after creation). No lingering test state in the tracker beyond the closed Issue itself, which
   is intentionally kept as the verification record's evidence trail.
4. **Documentation accuracy / self-correction** — The appended section also corrects a minor
   factual inaccuracy in the PR #309 report and in Issue #310's own body: `upstream-patrol.yml`'s
   reused search pattern was described as "ASCII + space-tokenized," but on inspection it also
   contains CJK-heavy terms (e.g. `in:title upstream CLI 破坏 gh`). This is a good-faith
   correction that doesn't change the verification outcome — worth noting as evidence of careful
   review rather than a defect.
5. **Markdown structural correctness** — The plan doc
   (`docs/superpowers/plans/2026-09-04-e2e-regression-dedup-verification.md`) embeds the exact
   text to be appended to the review report, which itself contains a ```` ```bash ```` fence —
   requiring an outer 4-backtick fence to avoid premature closure. Fence-balance check on the
   merged content (```` `grep -n '^```'` ````) confirms all fences pair up correctly in all three
   touched files (plan: 4 pairs including the nested block; report and design note: 1 pair each).
   This was in fact caught as a real defect earlier in Phase 3 by the inline `/code-review` pass
   (nested fence incorrectly closing in the plan's own example block) and fixed in commit `e3055c1`
   ("docs: fix nested code fence in dedup verification plan (code review finding)") before the
   final push — confirmed present in the branch history (`176c2ad..e3055c1`: `ddc7ca7` → `7f14a1e`
   → `e3055c1`).
6. **Traceability / conventions** — PR body links Issue #310, PR #309's report, and the design
   note; `Closes #310` is used correctly (not duplicated in the body text, consistent with this
   repo's `gf pr create --closes` convention). Naming and location of the new spike/plan docs
   follow `docs/superpowers/specs/` and `docs/superpowers/plans/` conventions used elsewhere in
   the repo.
7. **Regression risk** — None. No code, config, or workflow file is touched; the change is
   additive documentation plus an Issue-tracker close action.

## Findings

None. No correctness, safety, consistency, or scope issues identified in this independent pass.

## Verification Evidence (independently checked, not solely from PR description)

- `gh pr diff 320` — confirmed diff is limited to the three documentation files listed above,
  no code or workflow changes.
- `git log 176c2ad..e3055c1 --oneline` — confirmed the fence-fix commit (`e3055c1`) exists on
  the branch, following the two content commits (`ddc7ca7`, `7f14a1e`), consistent with the PR
  description's claim that the Phase 3 inline `/code-review` finding was fixed before push.
- Fence-balance check (`grep -n '^```'` on each touched file's content in the merge commit) —
  all three files have evenly paired fences; no unclosed or incorrectly nested blocks.
- `gf issue view 310` — `state: closed`, matching the PR's `Closes #310`.
- `gf issue view 319` — `state: closed`, title and label match the production dedup query
  exactly as claimed.
- PR-reported validation (not independently re-run, but consistent with the above): pre-commit
  clean on touched files; 3/3 real `gh issue list --search` queries against Issue #319 matched.

## Decision

**Approve.**

Rationale: documentation-only change with zero production/CI impact, a verification method that
correctly replicates the production dedup query, a conclusion that follows the exact branch
prescribed by Issue #310's acceptance criteria, clean test-artifact hygiene (test Issue closed
immediately), and independently confirmed fence-balance / diff-scope correctness. The one issue
raised during this PR's lifecycle (nested code-fence incorrectly closing) was caught by the Phase 3
inline review and fixed in a dedicated commit prior to push — no residual trace of it in the
merged content. No blocking or non-blocking findings remain.

## Note on PR State

At the time this formal review was submitted, PR #320 had already been merged into `dev`
(merged ~18 seconds after creation, consistent with a passing auto-merge gate on a documentation-
only, pre-vetted change with zero findings). GitHub permits review submission against merged
PRs, so this review was submitted post-merge; it documents the formal Phase 4 sign-off for the
gf-workflow record on Issue #310 and does not gate any further merge action.

## Note on Submission Mechanism

`gf review approve 320` was expected to fail with a GitHub 422 (GitHub rejects
`approve`/`request-changes` review events from a PR's own author; the authenticated `gf`/`gh`
identity, `byx-darwin`, is also PR #320's author — the same platform restriction documented in
`docs/code-review-report-pr317-2026-09-04.md`). The APPROVE verdict above was recorded via
`gf review comment 320` (a `comment`-type GitHub review event carrying an explicit APPROVE
decision in its body), consistent with the precedent set for PR #317.
