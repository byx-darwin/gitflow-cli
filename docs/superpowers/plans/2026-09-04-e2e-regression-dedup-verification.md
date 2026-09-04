# e2e-regression Dedup Verification Follow-up Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **Batching note (gf-workflow):** complexity score = 0 (1 file × 1 = 1 point, no module-boundary/API/migration risk, capped well under 4) → **simple/batch**. Documentation-only, no Rust code, no tests. Single pass.

**Goal:** Record the Issue #310 spike's verification outcome (GitHub search reliably matches the CJK dedup title) as a permanent annotation on `docs/code-review-report-pr309-2026-09-03.md`, and close Issue #310.

**Architecture:** No architecture — this is a documentation append + an Issue-close action. No production code, no CI workflow change (the spike found the dedup logic already works correctly, so `.github/workflows/e2e-tests.yml` needs no change per Issue #310's AC branch 3).

**Tech Stack:** Markdown, `gf` CLI (issue close).

**Spec:** `docs/superpowers/specs/2026-09-04-e2e-regression-dedup-verification-design.md`

## Global Constraints

- No change to `.github/workflows/e2e-tests.yml` — the spike's finding is "verified OK," not "needs a fix."
- The annotation must reference the concrete verification method (test Issue #319, 3 repeated search hits) so a future reader can trust the record without re-running the spike.

---

### Task 1: Annotate `docs/code-review-report-pr309-2026-09-03.md`

**Files:**
- Modify: `docs/code-review-report-pr309-2026-09-03.md` (append a new section after "## Process Note", the file's last section, ending at line 146)

**Interfaces:**
- Consumes: verification result from `docs/superpowers/specs/2026-09-04-e2e-regression-dedup-verification-design.md`
- Produces: nothing consumed by Task 2 — independent

- [ ] **Step 1: Append a new "## Follow-up: Dedup Verification (Issue #310)" section**

Append this exact content at the end of the file (after line 146, the last line of "## Process Note"):

```markdown

## Follow-up: Dedup Verification (Issue #310)

Finding #1 above (CJK title search reliability) was verified on 2026-09-04 per Issue #310's
acceptance criteria. Method: created a throwaway test Issue (#319) in this repo with the exact
production title `定时 E2E 回归失败` and label `e2e-regression`, then ran the exact query
`.github/workflows/e2e-tests.yml`'s `notify-on-schedule-failure` job uses:

```bash
gh issue list --label "e2e-regression" --state open \
  --search "in:title 定时 E2E 回归失败" --json number --jq '.[0].number // empty'
```

Ran 3 times (including with short delays to rule out search-index propagation lag) — all 3
attempts correctly returned the test issue's number. **No false negative observed.**

**Conclusion:** the dedup logic works as designed; GitHub's full-text search reliably matches
this specific CJK title. Per Issue #310's acceptance criteria (the "search hits" branch), **no
change to `.github/workflows/e2e-tests.yml` is needed.** The client-side exact-match fallback
described in this report's Verdict section and in Issue #310's acceptance criteria remains
documented here as the fix to reach for *if* a future title change or GitHub search behavior
change causes this to regress — but is not applied now, since there is nothing to fix.

Test Issue #319 was closed immediately after verification; see #310 for the full spike record
and `docs/superpowers/specs/2026-09-04-e2e-regression-dedup-verification-design.md` for the design
note.

Correction to this report's own framing: this section's parent finding (and Issue #310) described
`upstream-patrol.yml`'s reused search pattern as "ASCII + space-tokenized." On closer inspection,
`upstream-patrol.yml` also contains CJK-heavy search terms (e.g. `in:title upstream CLI 破坏 gh`),
so that comparison wasn't fully accurate. Doesn't change the verification outcome above.
```

- [ ] **Step 2: Verify the file renders correctly**

Run: `cat docs/code-review-report-pr309-2026-09-03.md | tail -30`
Expected: the new section appears at the end, valid Markdown (no broken code fences).

- [ ] **Step 3: Commit**

```bash
git add docs/code-review-report-pr309-2026-09-03.md
git commit -m "docs: record e2e-regression dedup verification outcome (issue #310)"
```

---

### Task 2: Close Issue #310

**Files:**
- None — Issue-tracker action only.

**Interfaces:**
- Consumes: Task 1's commit (referenced in the close comment so the closing record points at the permanent annotation)
- Produces: nothing (terminal task)

- [ ] **Step 1: Close Issue #310 with a summary comment**

Run (after Task 1's commit is pushed and its commit SHA is known):

```bash
gf issue comment 310 --body "验证完成：GitHub 搜索对 \`定时 E2E 回归失败\` 标题命中稳定（3 次重试全部命中测试 Issue #319），去重逻辑按设计工作，e2e-tests.yml 无需改动。详见 docs/code-review-report-pr309-2026-09-03.md 的 Follow-up 小节。"
gf issue close 310
```

- [ ] **Step 2: Verify closure**

Run: `gf issue view 310`
Expected: `state: closed`
