# Code Review Report — gf-workflow `wf-2026-09-17-001` (Post-Merge Audit)

**Title:** fix(quality): unify coverage tooling on `cargo-llvm-cov`, correct Gate 3's
incremental-vs-total claim, add missing `references/ruby.md`, remove auto-fix-authorising
wording, add skill-link validator (+ unrelated `gc`/GitCode-shadowing fix)
**Branch:** `fix/340-coverage-metric-unification` → `dev` (local `--no-ff` merge, no PR)
**Merge commit:** `eb14dc0` (12 commits, base `d4bde58`)
**Milestone:** #2 — closes #340, #348, #354
**Author:** baoyuexing
**Reviewed by:** post-merge audit (equivalent rigor to `gf-review`; no PR exists because delivery
was a local merge — see "Note on Review Mechanism")
**Review date:** 2026-09-17

## Note on Review Mechanism

There is no PR for this delivery: `fix/340-coverage-metric-unification` was merged locally into
`dev` with `--no-ff` as `eb14dc0`, so `gf review approve/request-changes` has no target — GitHub
never saw a pull request to attach a verdict to. This report is the audit-trail record Phase 4
of `gf-workflow` requires in that situation, following `docs/index.md`'s "Ad-hoc PR Reviews"
convention (`docs/reviews/`) since the filename does not fit the `code-review-report-pr<N>-*`
pattern (no PR number exists).

This diff already went through two prior review rounds on the most capable model available (a
whole-branch review that raised 8 Important findings, and a scoped re-review of the fix wave
that closed all 8). This report does not re-derive those — its value is (1) recording the
delivery for audit trail, (2) checking the **post-merge state of `dev`**, which no prior
diff-scoped review saw, and (3) checking for interactions with files the branch did not touch.

## Summary

Milestone #2 fixes a coverage-tooling split: `Makefile:158` ran `cargo-llvm-cov` while
`skills/gf-quality/references/rust.md` documented `cargo-tarpaulin` in six places (#340), and
`SKILL.md` claimed Gate 3 measured incremental coverage when every language's actual command
measured total coverage, with #354 wrongly assuming `cargo tarpaulin --diff` exists (#348/#354).
The design doc (`docs/superpowers/specs/2026-09-17-coverage-metric-unification-design.md`)
independently re-derives that tarpaulin's historically-cited 37.55% baseline undercounts on
Apple Silicon (ptrace-based instrumentation), vs. llvm-cov's 85.78% on the same codebase —
corroborating that the tool split was not cosmetic. The fix: standardize all five language
references on total-coverage semantics, add an explicit `N/A` (no source file of that language
in the change set) distinct from `SKIPPED` (tool/config absent), create the previously-missing
`references/ruby.md` that `detector.md` had always pointed `Gemfile` at, and strip wording in
several references that had allowed auto-fixing, which contradicted `SKILL.md`'s "Report only.
No auto-fix." A new `scripts/validate-skill-links.sh` (+ fixture tests) is wired into
`make check-agent-sync` to prevent skill cross-reference rot going forward. One commit
(`8f74699`) is outside the milestone: a macOS-specific fix where Graphviz's `gc` binary shadowed
the GitCode CLI's own `gc` name.

## Scope of Change

19 files, +1938/-70 (`git diff --stat d4bde58..eb14dc0`):
`Makefile` (+1), `crates/gitcode/src/lib.rs` (+253 net, mostly new), `docs/index.md` (+1),
`docs/integration-guide.md`, `docs/references/gf-quality-params.md`, two new
`docs/superpowers/{plans,specs}/2026-09-17-coverage-metric-unification*.md` design artifacts,
`docs/superpowers/tests/skills/gf-quality-test.md`, two new
`scripts/{validate-skill-links.sh,tests/validate-skill-links-test.sh}`, and eight
`skills/gf-quality/**` files (`SKILL.md` + `references/{go,java,node,python,ruby,rust}.md` +
sibling `gf-label-stats/SKILL.md`, `gf-pr-review/SKILL.md` for unrelated link-path fixes bundled
in the same wave). No `deny.toml`, `.pre-commit-config.yaml`, `rust-toolchain.toml`, or CI
workflow file touched.

## Review Dimensions

1. **Post-merge validation gates on `dev` (not previously run against the merged tree).**
   - `make check-agent-sync` — all 26 skills confirmed to have "When NOT to Use" sections, 78
     CLI commands / 26 files / 185 refs cross-checked with 0 mismatches, and the new "Skill Doc
     Link Summary" reports all 24 skill doc references resolve. Clean.
   - `bash scripts/tests/validate-skill-links-test.sh` — 5/5 fixture cases pass (resolving link,
     broken load target, broken markdown link, absolute/placeholder/glob/command paths skipped,
     load-target-resolves-against-skill-root).
   - `cargo clippy -p gitflow-gitcode --all-targets --all-features -- -D warnings
     -W clippy::pedantic` — zero warnings.
   - `cargo test -p gitflow-gitcode --lib` — 212/212 pass, including the 8 new binary-discovery
     tests from `8f74699`.

2. **Live verification of the `gc`/GitCode-shadowing fix (`8f74699`) against this machine's
   actual environment** — something a diff-scoped reviewer could reason about only from the
   commit's test doubles, not from the real binary. This machine has Graphviz's `gc` at
   `/opt/homebrew/bin/gc` and no `gitcode` on PATH. Ran `/opt/homebrew/bin/gc --help` directly:
   exit code 1, stdout is Graphviz's usage text, stderr is `gc: option -- unrecognized`. This
   independently confirms the commit's documented premise (`--help` on the wrong binary fails
   non-zero) and validates that `is_gitcode_help_output`'s `success &&
   contains("gitcode")` conjunction correctly rejects this exact real-world candidate — not just
   the mocked one in `crates/gitcode/src/lib.rs`'s `test_should_reject_graphviz_gc_help_output`.

3. **Normative-vs-historical `tarpaulin` classification, checked repo-wide, not just in the
   diff.** The design doc's own acceptance criterion is
   `grep -rn tarpaulin skills/ docs/integration-guide.md docs/references/ docs/superpowers/tests/`
   → empty. Reproduced independently on `dev`: **NORMATIVE CLEAN**, 0 hits. The remaining ~30
   `tarpaulin` mentions repo-wide are all in `docs/reports-archive/`, `docs/superpowers/plans/`,
   `docs/superpowers/specs/2026-07-06-*`, `docs/research/`, and
   `docs/issue-triage-report-2026-09-16.md` — historical records the design doc explicitly
   decided not to rewrite (D6: "rewriting archived reports would make the record not match what
   actually happened at the time"). No normative leakage found.

4. **"incremental" residue check** — grepped normative files for the word "incremental" to make
   sure no stray claim of incremental coverage survived the Gate 3 rewrite. Two hits, both in
   `rust.md` and `java.md`, both about **incremental compilation** (`profile.dev.incremental`,
   Gradle build caching) under "Performance Tips" — unrelated to coverage semantics, not a
   leftover of the removed incremental-coverage claim.

5. **`detector.md` ↔ `ruby.md` cross-reference.** `detector.md:26` maps `Gemfile` → Ruby →
   `references/ruby.md`; the file now exists (138 lines) with Gate Commands, Tool Installation,
   Environment Variables, Forbidden Actions, Makefile-First Rule, Configuration, and
   Troubleshooting sections — structurally parallel to `python.md`'s section set (both have the
   same top-level headings in the same order; `ruby.md` additionally splits Environment
   Variables into a top-level and a nested subsection, a superset, not a contradiction). Gate 3
   in `ruby.md` states "exit 0 (total line coverage ≥ threshold); N/A if no `.rb` in change set"
   — matches `SKILL.md`'s N/A-vs-SKIPPED distinction verbatim.

6. **Auto-fix wording consistency, checked across every reference file, not just the ones the
   fix wave touched.** `grep -rn "auto-fix"` across `skills/gf-quality/` shows every reference
   (`go.md`, `node.md`, `python.md`, `rust.md`, `ruby.md`) and `SKILL.md` itself now uniformly
   state "Never auto-fix ... report only" / "Report only. No auto-fix." No remaining reference
   authorizes auto-fixing, and no file contradicts `SKILL.md`'s absolute stance.

7. **Interaction with an untouched file:** `docs/references/gf-quality-params.md` (not modified
   by this branch) contains its own multi-language coverage-command table (Node/Python/Go/Java —
   no Rust, no Ruby row). This does not contradict the new `ruby.md`: that table was already
   partial before this branch (Rust, the primary/default language, was never in it either), and
   nothing in `gf-quality-params.md` makes a total-vs-incremental coverage claim that the branch
   would now contradict. No finding.

8. **`docs/index.md` bookkeeping.** The one line the branch adds
   (`docs/index.md:37`) links `2026-09-17-coverage-metric-unification-design.md`, correctly
   summarizes the milestone (#340/#348/#354), and correctly states the llvm-cov-vs-tarpaulin
   delta (85.78% vs. 37.55%) matching the design doc's own numbers. No index entry is missing
   for the new `docs/superpowers/plans/2026-09-17-coverage-metric-unification.md` companion
   plan — `docs/index.md` already references it via the same line's markdown link. Unlike PR
   #323's audit finding, no index gap here.

9. **Regression risk.** No CI workflow, `deny.toml`, `.pre-commit-config.yaml`, or
   `rust-toolchain.toml` touched. The one Rust production-code change
   (`crates/gitcode/src/lib.rs`) is covered by 8 new unit tests plus a live-environment
   reproduction (Dimension 2) and passes pedantic clippy clean. The remaining 18 files are
   skill/doc/script content with no runtime execution path outside `gf-quality`'s own gates and
   the new `make check-agent-sync` validator, both of which pass against the merged tree.

## Findings

**No findings.** All three post-merge checks requested (`make check-agent-sync`,
`validate-skill-links-test.sh`, `cargo clippy -p gitflow-gitcode`) pass clean against `dev` as
merged. The normative/historical `tarpaulin` split holds repo-wide, the auto-fix wording is
consistent across every reference file (not just the ones the branch touched), the new
`ruby.md` is structurally and semantically consistent with its siblings and with `detector.md`'s
pre-existing mapping, and the out-of-milestone `gc`-shadowing fix was independently verified
against this machine's real Graphviz `gc` binary (not just its test double) and behaves as
documented.

## Verification Evidence

- `git log --oneline d4bde58..eb14dc0` — confirmed 12 commits, `eb14dc0` is the `--no-ff` merge
  commit; `git diff --stat d4bde58..eb14dc0` — confirmed 19 files, +1938/-70.
- `make check-agent-sync` — 26/26 skills pass "When NOT to Use" check, 0/185 reference
  mismatches, 24/24 skill doc links resolve.
- `bash scripts/tests/validate-skill-links-test.sh` — 5/5 fixture cases pass.
- `cargo clippy -p gitflow-gitcode --all-targets --all-features -- -D warnings
  -W clippy::pedantic` — 0 warnings.
- `cargo test -p gitflow-gitcode --lib` — 212 passed, 0 failed.
- `/opt/homebrew/bin/gc --help` run directly on this machine (separate stdout/stderr capture) —
  exit 1, stderr `gc: option -- unrecognized`, no `gitcode` marker in either stream — independent
  real-binary confirmation of `8f74699`'s probe logic, not just its mocked unit test.
- `grep -rn tarpaulin skills/ docs/integration-guide.md docs/references/
  docs/superpowers/tests/` — 0 hits (design doc's own acceptance criterion), confirming no
  normative tarpaulin residue.
- `grep -rn tarpaulin .` repo-wide minus `.git/` — all remaining ~30 hits classified: archived
  reports, historical plans/specs, research notes, and one triage report — all pre-decided as
  intentionally unrewritten (D6 in the design doc).
- `grep -rn "incremental"` across `skills/gf-quality/references/` normative files — 2 hits, both
  "incremental compilation" (build performance), unrelated to the removed incremental-coverage
  claim.
- `grep -n "^##" skills/gf-quality/references/{ruby,python}.md` — structural comparison
  confirming `ruby.md` is a superset-consistent sibling, not a divergent outlier.
- `grep -rn "auto-fix"` across `skills/gf-quality/` — confirmed uniform "report only, never
  auto-fix" wording in every reference file and `SKILL.md`.
- Read `docs/index.md:37` and cross-checked its claimed milestone/numbers against
  `docs/superpowers/specs/2026-09-17-coverage-metric-unification-design.md`.

## Decision (verdict-equivalent, recorded here since no `gf` verdict exists)

**Approve — the merged state of `dev` matches the branch's intent, no new findings.**

Rationale: this delivery already survived two review rounds that closed 8 Important findings
before merge; this audit's job was to check what those rounds structurally could not — the
actual post-merge tree — and it comes back clean on all three requested gates plus targeted
cross-file consistency checks the branch's own diff could not self-verify (normative/historical
tarpaulin classification repo-wide, auto-fix wording repo-wide, and a live-machine reproduction
of the `gc`-shadowing fix against a real, unmocked Graphviz binary). No blocking or
non-blocking findings to report. No follow-up required.
