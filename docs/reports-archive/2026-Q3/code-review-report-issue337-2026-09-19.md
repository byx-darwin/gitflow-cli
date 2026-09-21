# Code Review Report — Issue #337 (Post-Delivery Diff Audit)

**Title:** gf-workflow-batch dependency-edge topological ordering
**Merge commit:** `30131395dd3a312f63fe21b436721d80a14bb187` (merge of `feat/337-gf-workflow-batch-topo-order` into `dev`, `--no-ff`)
**Diff range reviewed:** `73a3bf1545a1909d473b5804cf35666cd9744a48..434e53b` (the commits the merge brought into `dev`; the merge commit `3013139` itself is a no-op merge of this range)
**Issue:** #337
**Workflow:** gf-workflow, post-delivery check (Phase 4)
**Delivery method:** `local_merge` — **no PR exists** for this change; this report reviews the merged diff directly. It is an independent second pass after a full pre-merge `subagent-driven-development` review cycle (per-task reviews + whole-branch review + one fix wave + scoped re-review, all reported clean).
**Review date:** 2026-09-19
**Review effort:** medium (fresh independent pass, not exhaustive re-litigation of the pre-merge review)

## Scope of Change

| File | Change |
|---|---|
| `skills/gf-workflow-batch/references.md` | +132/-3: adds regex-based `Blocked by:` edge extraction, three-color DFS cycle detection (`find_cycle`), dependency-aware `ready` derivation, and topological dispatch ordering to the Pending Derivation Algorithm |
| `skills/gf-workflow-batch/SKILL.md` | +47/-8 (net, across the diff range): documents the new dependency-resolution behavior, error handling for cycles, and updated flow |
| `skills/gf-issue-decompose/references/dependency-edges.md` | +9/-... : cross-references the new consumer of dependency edges in `gf-workflow-batch` |

Doc/skill-only change — no Rust source, `Cargo.toml`, lockfile, or CI workflow files touched.

## Methodology

Invoked `/code-review --effort medium` against the diff range. The reviewer read the full diff plus surrounding unchanged context in `references.md` and `SKILL.md`, and independently verified two candidate findings against the Rust source before finalizing:

1. Verified whether `ready.sort()` changes dispatch order relative to the existing `pending.sort(by=issue.number)` — confirmed `pending` was already number-sorted upstream of this diff, so the initially suspected behavior-change finding was **withdrawn** as a false positive (sorting a subset of an already-sorted list is redundant, not a regression).
2. Verified the redundant-body-refetch finding against `crates/github/src/issue.rs` (`ISSUE_FIELDS` shared between `list_impl` and `view`) and `crates/core/src/issue.rs:21-48`, confirming GitHub/GitLab/GitCode providers all return the same fields from `list` and `view`, so the finding is real rather than speculative.

## Findings

**Verdict: 0 Critical, 0 Important, 6 Minor/Low-confidence findings** (all in `skills/gf-workflow-batch/references.md`; this is a pseudocode/prose spec for an LLM agent to hand-execute, not compiled/tested code, so all findings here are design/robustness gaps rather than confirmed runtime bugs).

### 1. Redundant per-round issue-body re-fetch (references.md:84)

`bodies = {i.number: gf_issue_view(i.number).body for i in open_issues}` re-fetches every open issue's body with a separate `gf_issue_view` call per issue, even though `open_issues = gf_issue_list_open()` already returns the same fields — confirmed via `crates/github/src/issue.rs` (`ISSUE_FIELDS` shared by `list_impl` and `view`, lines ~22-23, 139, 311) and `crates/core/src/issue.rs:21-48`. Every dispatch-loop round makes N extra `gf issue view` calls purely to re-read `.body`, growing API/rate-limit exposure with backlog size. Fix: `bodies = {i.number: i.body for i in open_issues}`.

### 2. Edge-parsing regex is brittle to plausible authoring variants (references.md:88)

`r'Blocked by:\s*((?:#\d+(?:,\s*)?)+)'` only matches the exact literal `Blocked by:` (case-sensitive, colon required, comma-separated). Variants like `Blocked By:`, `Blocked by #12`, or space-joined refs (`Blocked by: #12 #14`) silently fail to match and the edge is dropped with no error — the same "prose edge silently treated as no dependency" failure mode the PR's own `dependency-edges.md` warns against, but caused here by parser strictness rather than author error. Risk: an issue is dispatched before its real blocker completes, with no surfaced warning.

### 3. Cycle detection runs unconditionally before the empty-pending fast path (references.md:201)

`ready = resolve_dependencies(pending)` (full cycle detection over all open issues) runs before the `if pending is empty` check. A stray cyclic/dangling `Blocked by` reference between two open issues that are *not* in `pending` (e.g. already covered by an active/archived contract) will abort every run — including one where `pending` is empty and should proceed straight to Discussion Mode. None of the new test scenarios (7–15) cover "pending empty but a bad edge exists among non-pending open issues."

### 4. DFS cycle detection has no recursion-depth guard (references.md:141)

`find_cycle`'s recursive `dfs(b)` has no depth limit. A sufficiently long `Blocked by` chain (on the order of a few hundred to a thousand issues deep) would hit Python's default recursion limit and raise an unhandled `RecursionError` instead of the intended graceful `WorkflowBatchError` with a cycle path. Low real-world likelihood given typical backlog sizes, but the failure mode contradicts the documented graceful-stop behavior.

### 5. Terminal summary can't distinguish "done" from "stuck behind an unresolved blocker" (references.md:228)

When `ready` is exhausted while issues remain in `pending` (blocked on something nobody is working), the final summary table looks identical to a fully-completed run. No signal is surfaced that manual follow-up (closing the blocker, or manual dispatch) is needed.

### 6. [Altitude] Cycle-detection/edge-parsing logic lives only in markdown pseudocode, not compiled/tested code (references.md:70-219)

The regex parsing and three-color DFS are deterministic, unit-testable graph logic, but are specified only as prose for an LLM agent to hand-simulate each round. No `cargo test`/clippy run can catch drift between this spec and an agent's actual execution of it (e.g. a missed comma-separated ref, or incorrectly tracked GRAY/BLACK coloring) — the logic is invisible to the Rust test suite entirely. This is a design-level observation, not a defect in the current text.

## Notes on Review Process

- One initially reported finding (`ready.sort()` allegedly changing dispatch order vs. "list order") was investigated and **withdrawn** after confirming `pending` was already number-sorted upstream in the unchanged part of the Pending Derivation Algorithm — the sort on `ready` is redundant but not a behavior change. Included here for audit-trail completeness; it is not counted among the 6 findings above.
- Findings 1 and 2 have concrete evidence (source code cross-reference and regex analysis, respectively). Findings 3–6 are scenario-based (plausible edge cases / design observations) rather than reproduced failures — consistent with "medium" effort on doc/pseudocode content where no test harness exists to reproduce them directly.

## Verification

Doc/skill-only change: no Rust build/test/clippy required per project instructions ("Docs-only changes do not require the full Cargo suite"). No `deny.toml`, `.pre-commit-config.yaml`, `rust-toolchain.toml`, or `.github/workflows/` files were touched by this diff.

## Verdict

**Not an approval/request-changes submission** — no PR exists for this `local_merge` delivery, so no verdict is filed via `gf-review`/`gf-pr-review`. For the record: **0 Critical, 0 Important, 6 Minor findings**, all confined to the new dependency-resolution pseudocode in `skills/gf-workflow-batch/references.md`. None block the delivery already made to `dev`; all are candidates for a follow-up hardening pass on the dependency-edge feature (regex robustness, recursion guard, empty-pending fast path ordering, summary-table signal for stuck issues, and the redundant-fetch efficiency fix).
