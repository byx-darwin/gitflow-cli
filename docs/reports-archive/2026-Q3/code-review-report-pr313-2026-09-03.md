# Code Review Report — PR #313 (post-merge review)

- **PR**: fix(pr): detect repo default branch instead of hardcoding "main"
- **URL**: https://github.com/byx-darwin/gitflow-cli/pull/313
- **Base / Head**: `dev` ← `feat/305-pr-create-default-branch`
- **Status**: Merged (merge commit `dbf68f1`, `mergedAt` `2026-09-03T10:04:17Z`) — already closed by the
  time this review was dispatched (`gf pr view 313` returned `"state": "closed"` with `mergedAt` set).
- **Closes**: #305
- **Review type**: Formal review requested against an already-merged PR, using `gf pr view 313` plus a
  local diff of the merged commits (`git diff f4308a8..7bdfedf`) as the source of truth — not `gh`, per the
  task's instruction. Consistent with the precedent in `docs/code-review-report-pr309-2026-09-03.md` /
  `docs/code-review-report-pr281-2026-09-02.md`: no live `gf review approve/request-changes/comment` call
  was submitted against GitHub for an already-merged PR by the same author; this report is the
  retrospective/formal review record.
- **Reviewer**: independent automated agent review — code was actually built, tested, and linted, not
  just read.
- **Self-review check**: PR author `byx-darwin` (same as local git user `baoyuexing`). No `gf review`
  verdict was submitted for this reason, consistent with prior reports' precedent.

## Scope Verification

Rust source diff matches the PR description exactly: `crates/core/src/pr.rs` (+13, new
`PrProvider::default_branch()` trait method + test-mock impl), `crates/github/src/pr.rs` (+95, `gh repo
view --json defaultBranchRef` impl + 3 tests), `crates/gitlab/src/mr.rs` (+91, `glab repo view --output
json` impl + 3 tests), `crates/gitcode/src/pr.rs` (+29, `CoreError::Platform` stub + 1 test),
`apps/cli/src/commands/pr.rs` (+28/-1, wiring + `resolve_default_branch()` helper + 2 tests). Two
docs-only additions (`docs/superpowers/plans/...`, `docs/superpowers/specs/...`) are out of scope for a
Rust-code review.

## What Was Checked

1. **Trait design** — `default_branch(&self) -> Result<String>` added to `PrProvider`
   (`crates/core/src/pr.rs`). Doc comment includes `# Errors` section as required by this repo's CLAUDE.md.
   Verified all four implementers were updated: `GitHubPrProvider`, `GitLabMrProvider`,
   `GitCodePrProvider`, and the in-crate test mock `Check` in `crates/core/src/pr.rs` — confirmed via
   `grep -rn "impl.*PrProvider for"` across the workspace; no other implementer exists, so this is not a
   breaking change for any downstream code.

2. **GitHub implementation** (`crates/github/src/pr.rs`) — spawns `gh repo view --repo <repo> --json
   defaultBranchRef`, deserializes into a private `RepoViewResponse { default_branch_ref:
   DefaultBranchRef { name: String } }`. Non-zero exit routed through the existing `parse_gh_error`
   helper (reused, not reinvented); JSON parse failure mapped through `CoreError::Serialization`. Matches
   the error-handling shape of every other method in this file.

3. **GitLab implementation** (`crates/gitlab/src/mr.rs`) — spawns `glab repo view --repo <repo_target>
   --output json`, deserializes `default_branch: String` directly (GitLab's JSON is flatter than
   GitHub's). Correctly uses `self.repo_target` (not `self.repo`) for the `--repo` argument, matching the
   convention every other method in this file already follows (`repo_target` carries remote-URL overrides;
   `repo` is the canonical `owner/repo` used only for logging/matching). Verified via `grep -n
   "repo_target\|self.repo\\b"`.

4. **GitCode implementation** (`crates/gitcode/src/pr.rs`) — returns `CoreError::Platform("GitCode CLI
   不支持查询仓库默认分支...")` unconditionally, with **no CLI process spawned**. Verified this claim is
   true, not just documented: `test_should_error_without_cli_call_for_default_branch` asserts
   `runner.calls().is_empty()` using `RecordingMockRunner`, which would fail if a spawn were attempted.
   The error message and pattern exactly mirror the existing `merge --auto` unsupported-capability stub
   (same crate, same file), so this is consistent with prior art rather than a one-off.

5. **CLI wiring** (`apps/cli/src/commands/pr.rs`) — `resolved_base` changed from
   `base.unwrap_or_else(|| "main".to_string())` to a `match` that only calls `provider.default_branch()`
   when `base` is `None`; an explicit `--base` never triggers the query at all (by construction of the
   `match`, not by a runtime check), so it cannot regress or slow down the explicit-base path. On query
   failure (any error variant, any platform including GitCode's constant `CoreError::Platform`), the new
   `resolve_default_branch()` helper falls back to `"main"` and logs the error via `tracing::debug!`
   (structured `error = %e` field, matching this repo's logging conventions — no `println!`/`dbg!`
   introduced). This means behavior for GitCode is unchanged from before the PR (always falls back to
   `"main"`), while GitHub/GitLab repos with a non-`main` default branch (e.g. `dev`) now correctly detect
   it — exactly the bug described in Issue #305.

6. **Build/lint/test verification** (re-run independently in this review, not taken on faith from the PR
   body):
   - `cargo test -p gitflow-cli --bin gf commands::pr::tests` — 40 passed, including the two new
     `resolve_default_branch` tests (success + fallback-to-main-on-`CoreError::Platform`).
   - `cargo test -p gitflow-github -p gitflow-gitlab -p gitflow-gitcode` (filtered to the new tests) — all
     7 new provider-level tests pass (success path, exact-argv assertion, and CLI-failure path for
     GitHub/GitLab; no-CLI-call assertion for GitCode).
   - `cargo clippy -p gitflow-core -p gitflow-github -p gitflow-gitlab -p gitflow-gitcode -p gitflow-cli
     --all-targets --all-features -- -D warnings -W clippy::pedantic` — clean, no warnings.
   - `cargo +nightly fmt --check` (same five crates) — clean.

## Findings

| # | Severity | Area | Finding |
|---|----------|------|---------|
| 1 | Nitpick | Doc accuracy | `apps/cli/src/commands/pr.rs:35` — the `--base` clap help text still reads `目标分支（可选，默认为 \`main\`）` ("optional, defaults to `main`"), which is now only half true: the actual behavior is "detect the repo's configured default branch, falling back to `main` only if that query fails or the platform doesn't support it." Purely a `--help`-output/doc-comment staleness issue — no functional impact, and not a violation of the "all public items documented" rule since the doc exists and is not wrong so much as no-longer-complete. Worth a one-line follow-up edit but does not block or warrant reverting the merged PR. |

No correctness bugs, no missing error-path test coverage (every fallible new function has both a success
and a failure test, per this repo's CLAUDE.md testing convention), no `unwrap()`/`expect()` introduced in
production code, no secrets or user input reaching a shell/SQL boundary, and no clippy::pedantic violations.

## Verdict

**Approve.** The fix is minimal, correctly scoped, and directly resolves Issue #305's root cause (hardcoded
`"main"` silently targeting a nonexistent branch on repos whose default branch differs). The three-platform
split (GitHub live query / GitLab live query / GitCode static unsupported-error) is implemented consistently
with each provider's existing conventions and with the prior `merge --auto` precedent for
platform-unsupported capabilities. Test coverage is complete for every new fallible path (success, CLI
failure, and — for GitCode — no-spawn-at-all), and independently re-running `cargo test`, `cargo clippy
--all-targets --all-features -- -D warnings -W clippy::pedantic`, and `cargo +nightly fmt --check` against
the merged code confirms the PR body's stated verification claims rather than just trusting them. The one
finding (stale `--base` help text) is a cosmetic nitpick, not a blocker.

## Process Note

PR #313 was already merged (merge commit `dbf68f1`, `mergedAt` `2026-09-03T10:04:17Z`) by the time this
review was dispatched. Consistent with the precedent recorded in `docs/code-review-report-pr309-2026-09-03.md`
and `docs/code-review-report-pr281-2026-09-02.md`, no `gf review` call (`approve`/`request-changes`/
`comment`) was submitted against GitHub for PR #313, both because the PR is already merged and because the
author (`byx-darwin`) matches the local git user (`baoyuexing`) — self-review is out of scope per the
`gf-review` skill's explicit prohibition. This report is the formal review record. Review was conducted via
`gf pr view 313` plus a local `git diff` of the merged commit range against the actual worktree checkout
(`.worktree/feat/305-pr-create-default-branch`), and all quality-gate commands (`cargo test`, `cargo
clippy`, `cargo +nightly fmt --check`) were re-run independently rather than trusting the PR body's
checklist.
