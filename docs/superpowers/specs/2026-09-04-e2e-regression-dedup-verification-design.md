# Design: e2e-regression alert CJK title dedup verification

**Issue:** [#310](https://github.com/byx-darwin/gitflow-cli/issues/310)
**Classification:** Spike (per `superpowers:brainstorming`) — feasibility question, output is an answer, not kept code.

## Question

Does GitHub's issue full-text search (`gh issue list --search "in:title ..."`) reliably match the fixed CJK title `定时 E2E 回归失败` used by `.github/workflows/e2e-tests.yml`'s `notify-on-schedule-failure` job's dedup logic? If it doesn't, the dedup silently fails and every scheduled regression failure creates a new Issue instead of commenting on an existing one.

## Method

Created a throwaway test Issue (#319) in this repo with the exact title `定时 E2E 回归失败` and label `e2e-regression` — replicating the precise input the workflow's `gh` command would see. Ran the exact query the workflow uses:

```bash
gh issue list --label "e2e-regression" --state open \
  --search "in:title 定时 E2E 回归失败" --json number --jq '.[0].number // empty'
```

Repeated 3 times (including with short delays to rule out search-index propagation lag).

## Result

All 3 attempts returned `319` — the search reliably matched the test issue. No false negative observed.

## Conclusion

Per Issue #310's Acceptance Criteria (branch 3 — "若命中正常"): the dedup logic works as designed. **No change needed to `.github/workflows/e2e-tests.yml`.** Resolution is documentation-only:

1. Close Issue #310 with a summary of this finding.
2. Append a "verified" note to `docs/code-review-report-pr309-2026-09-03.md` (the review report that originally flagged this as finding #1), per AC branch 3's explicit instruction.
3. Correct a minor factual inaccuracy discovered during verification: Issue #310 describes `upstream-patrol.yml`'s reused search pattern as "ASCII + space-tokenized," but it also contains CJK-heavy search terms (e.g. `in:title upstream CLI 破坏 gh`) — noted for completeness, doesn't change the verification outcome.

## Test artifact cleanup

Test Issue #319 was commented on (linking back to this verification) and closed immediately after the search queries completed. No lingering test state.

## Scope

- **In scope:** verifying search reliability; closing #310; annotating the PR #309 review report.
- **Out of scope:** any change to `e2e-tests.yml` (not needed — this is the "verified OK" branch, not the "needs a fix" branch).
