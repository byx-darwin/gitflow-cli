# Code Review Report — Issue #333 (Post-Delivery Diff Audit)

**Title:** feat(gf-quality/gf-pr-review): evidence grading backfill
**Merge commit:** `ecd533a521e401e154032aaf83d14a75338edb47` (merge of `feat/333-evidence-grading` into `dev`, parents `7bd3126` + `7becc8f`)
**Issue:** #333
**Workflow:** gf-workflow, post-delivery check (Phase 4)
**Delivery method:** `local_merge` — **no PR exists** for this change; this report reviews the diff directly, using the `gf-pr-review` skill's 6-dimension checklist as a rubric only (no verdict is submitted, since there is no PR to submit against)
**Review date:** 2026-09-19
**Review scope:** `git diff 7bd3126...ecd533a` — the full content the merge brought into `dev`. Documentation-only change: two skill `SKILL.md` files, one `Makefile` target, one `specs/index.md` index line. No Rust source, no `Cargo.toml`, no CI workflow files changed.

## Summary

The change backfills the `Measured`/`Inferred`/`Unverified` three-tier evidence-grading vocabulary (established in Issue #329's `gf-walkthrough` skill) into `gf-quality`'s gate report and `gf-pr-review`'s per-dimension assessment, plus a failing-test commit-ancestry table in `gf-quality` to stop tests being dismissed as "unrelated" without proof. A new `make check-quality-review-evidence-skill` target grep-asserts the acceptance criteria. All of this is consistent with the stated goal and the implementation plan at `docs/superpowers/plans/2026-09-19-quality-review-evidence-grading.md`.

**Verdict (for audit trail only, not a submitted PR verdict): 1 Important finding.** The evidence-grading vocabulary itself is reused correctly and verbatim from `gf-walkthrough` (verified byte-for-byte below). The defect is in how the failing-test ancestry command was inlined into `gf-quality`'s report *templates*: it was pasted as bare, unfenced shell text directly inside the `\`\`\`markdown` fence that represents the literal, copy-paste Quality Report output — the same class of bug the immediately preceding commit (`7becc8f`, "unnest bash fence inside markdown report template") had just fixed, only reintroduced in a different spot in the same file, in the same merge.

## Scope of Change

| File | Change |
|---|---|
| `Makefile` | +37: new `check-quality-review-evidence-skill` target (8 grep-based hard-constraint checks), `.PHONY` line updated |
| `skills/gf-pr-review/SKILL.md` | +18/-2: `## Evidence Grading` section, per-dimension tier requirement in Step 2/3, one new common-mistake row, one new red flag, one new anti-pattern bullet |
| `skills/gf-quality/SKILL.md` | ~83 changed lines: `## Evidence Grading` section, `Evidence Tier` column added to the gate table and both report templates, new `### Failing Tests` table + ancestry-check guidance in both templates, three new mistake/red-flag bullets |
| `specs/index.md` | +1: indexes the implementation plan |

Total: 4 files, 112 insertions, 27 deletions. No `deny.toml`, `.pre-commit-config.yaml`, `rust-toolchain.toml`, or `.github/workflows/` changes.

## Methodology

1. Diffed `7bd3126...ecd533a` and read each changed file's full diff.
2. Confirmed the merge source branch `feat/333-evidence-grading` still exists locally (not yet deleted).
3. Ran `make check-quality-review-evidence-skill` from `dev` — the change's own acceptance test.
4. Ran `make check-agent-sync` — required for any skill/CLAUDE.md edit per project instructions.
5. Ran `cargo build` as a minimal sanity check (not required for a doc-only change, but cheap and confirms nothing else regressed).
6. Traced the "reused verbatim from `gf-walkthrough`" claims against `skills/gf-walkthrough/SKILL.md` and `docs/superpowers/templates/walkthrough-report-template.md` to verify the reuse is faithful, including *how* the source material structures the ancestry-check command (fenced vs. inline).
7. Applied the 6-dimension checklist from `docs/references/pr-review-checklist.md`, weighting Documentation and Correctness heaviest since the change *is* agent-facing documentation whose "code" is the instructions and templates themselves.

## Findings

### 1. [Important] Ancestry-check command and prose instructions are embedded unfenced inside the literal report-output template, contradicting the pattern the immediately-prior commit fixed

**Location:** `skills/gf-quality/SKILL.md:163-194` (Single-Language Report) and `skills/gf-quality/SKILL.md:198-256` (Multi-Language Aggregate Report).

The `### Failing Tests` sub-sections were added *inside* the outer ```` ```markdown ... ``` ```` fence that both report templates use to show the literal, copy-paste-able Quality Report output:

```
163  ```markdown
...
179  ### Failing Tests (Gate 2 only)
...
184  Never write "unrelated" / "与本次改动无关" without the commit hash and
185  ancestry check below (reused from `gf-walkthrough`):
186
187  H=$(git log -1 --format=%H -- "<test file>")
188  git merge-base --is-ancestor "$H" "$BASE" && echo "先于本次交付存在" || echo "本次引入"
189
190  ### Result
...
194  ```
```

Two problems:

- **Instructional prose leaks into the output template.** The sentence "Never write \"unrelated\" ... (reused from `gf-walkthrough`)" is guidance for the *agent authoring* the report, not part of the report itself, yet it sits inside the fence that represents what the delivered report should literally contain. An agent following the template mechanically risks emitting this English/Chinese mixed instructional sentence as literal report body text.
- **The ancestry command is unfenced, unlike its source.** The commit message of the immediately preceding commit in this same branch, `7becc8f "fix(gf-quality): unnest bash fence inside markdown report template"`, specifically addressed a nested-fence rendering problem in this same report-template area. Here the same two-line shell snippet (`H=$(git log -1 ...)` / `git merge-base --is-ancestor ...`) is pasted as *bare unfenced text* rather than being kept out of the outer fence or referenced externally. Compare the source this content claims to reuse "verbatim": `docs/superpowers/templates/walkthrough-report-template.md:33-36` wraps the identical two-line command in its own ` ```bash ` fence, **outside** of any report-body fence — `gf-walkthrough`'s own `SKILL.md:51-53` doesn't inline the command at all; it only cites the external template file by path. Neither of the two established patterns (external fenced reference, or inline properly-fenced-and-separated) was followed here.

**Why this matters:** This is the report template that anchors the entire feature just delivered (evidence-graded, ancestry-proven failing-test reporting). If the template itself is malformed such that an agent following it literally could paste raw shell commands and instructional sentences into a "Quality Gate Report" markdown block, the delivered reports become less trustworthy — directly undermining the evidence-integrity goal Issue #333 exists to serve. It did not trip the new `make check-quality-review-evidence-skill` target because that target only greps for presence of key strings/headers, not for template well-formedness.

**Suggested fix (not applied — audit only):** Move the ancestry-check command and its explanatory sentence out of the outer ` ```markdown ` fence (either as its own ` ```bash ` block immediately before/after the template fence, mirroring `walkthrough-report-template.md:33-36`, or by referencing an external template the way `gf-walkthrough/SKILL.md:51-53` does), in both the Single-Language (`:163-194`) and Multi-Language (`:198-256`) report sections.

**Severity: Important** — not Critical (doc-only, no build/runtime break, the grep-based acceptance gate still passes), but not cosmetic either: it's a structural defect in the feature's own primary deliverable, reproducing a bug class fixed one commit earlier in the same file.

## Checklist Pass (6 dimensions, audit-trail only — no verdict submitted, no PR exists)

| # | Dimension | Result | Evidence Tier |
|---|---|---|---|
| 1 | Correctness | ⚠️ see Finding 1 | `Inferred` — read `skills/gf-quality/SKILL.md:163-256` against `docs/superpowers/templates/walkthrough-report-template.md:33-36` and `skills/gf-walkthrough/SKILL.md:51-53` |
| 2 | Security | ✅ no findings — no secrets, no user-input handling, no new attack surface | `Inferred` — doc/Makefile-only diff, no I/O boundary touched |
| 3 | Performance | ✅ N/A — no runtime code | `Inferred` |
| 4 | Maintainability | ✅ new Makefile target follows the existing `check-walkthrough-skill` pattern; sections are additive and localized | `Inferred` — diffed against sibling target `check-walkthrough-skill` |
| 5 | Test coverage | ✅ the change's own acceptance test (`make check-quality-review-evidence-skill`) passes 7/7 checks, exit 0 | `Measured` — ran the command, see Verification |
| 6 | Documentation | ⚠️ see Finding 1; otherwise `specs/index.md` correctly indexes the plan, and the "reused verbatim" claim for the *vocabulary table* (not the ancestry command) is confirmed byte-identical | `Measured` for the byte-identical check (command run below); `Inferred` for the rest |

## Verification

```
$ make check-quality-review-evidence-skill
✓ #1 skills/gf-quality/SKILL.md 三档标记齐备
✓ #1 skills/gf-pr-review/SKILL.md 三档标记齐备
✓ #3 gf-quality 失败测试三列齐备
✓ #4 gf-quality 禁用词规则已声明
✓ #5 skills/gf-quality/SKILL.md 声明复用 gf-walkthrough 词汇
✓ #5 skills/gf-pr-review/SKILL.md 声明复用 gf-walkthrough 词汇
✓ #1 gf-pr-review 逐维度声明证据等级
全部硬约束通过
$ echo $?
0
```

```
$ make check-agent-sync
✅ All skills have 'When NOT to Use' section
Mismatches:      0
PASSED: All skill command references are valid.
✅ All 27 skill doc reference(s) resolve
```

```
$ cargo build
    Finished `dev` profile [unoptimized + debuginfo] target(s) in ...
$ echo $?
0
```

```
$ grep -m1 '| `Measured`' skills/gf-quality/SKILL.md skills/gf-pr-review/SKILL.md skills/gf-walkthrough/SKILL.md | \
  awk -F: '{print $1}' | while read f; do grep -m1 '| `Measured`' "$f" | tr -d '[:space:]'; done
|`Measured`|Commandrunthissession|Nooutput→downgradeto`Inferred`.|
|`Measured`|Commandrunthissession|Nooutput→downgradeto`Inferred`.|
|`Measured`|Commandrunthissession|Nooutput→downgradeto`Inferred`.|
```
The three-tier vocabulary table rows are byte-identical across `gf-quality`, `gf-pr-review`, and `gf-walkthrough` (also directly enforced by the Makefile target's own `QLINE`/`WLINE` comparison). The "reused verbatim" claim holds for the vocabulary; it does not hold for how the ancestry-check command was embedded (Finding 1).

```
$ git branch --list feat/333-evidence-grading
  feat/333-evidence-grading
```
The source branch still exists locally; it was not deleted after the `local_merge`.

## Verdict

**Not an approval/request-changes submission** — no PR exists for this `local_merge` delivery, so no verdict is filed via `gf-review`/`gf-pr-review`. For the record: **1 Important finding** (Finding 1, template fencing defect in `skills/gf-quality/SKILL.md`), 0 Critical findings. The acceptance gate (`make check-quality-review-evidence-skill`) passes because it checks string presence, not template well-formedness, so it did not catch Finding 1.

### Non-blocking notes (outside this report's mandate)

- `feat/333-evidence-grading` is still present locally post-merge; branch cleanup was not part of this audit's scope.
- `docs/` now holds 8 `code-review-report-*.md` files (after this one), above the "more than 5" archiving threshold documented in `docs/index.md`. This report does not perform that archival move.
- The reviewed skills' own newly-added `Evidence Grading` vocabulary is used in this report's own checklist table above, per the task's suggestion — this report's own `Measured` rows are backed by the command output shown in Verification; its `Inferred` rows cite the `path:line` ranges read during the diff review.
