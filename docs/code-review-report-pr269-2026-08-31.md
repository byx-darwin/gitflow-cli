# Code Review Report — PR #269

- **PR**: feat(issue): add `gf issue edit` subcommand
- **Branch**: `feat/266-gf-issue-edit` → `dev`
- **Closes**: #266
- **Date**: 2026-08-31
- **Reviewer (attempted)**: gf-review skill, via automated agent

## Outcome: Review NOT submitted — blocked by preconditions

Verification via `gf pr view 269` and `gh pr view 269` surfaced two blockers
before any verdict could be formed or submitted:

1. **PR is merged, not open.**
   `gf pr view 269` returns `"state": "closed"`; cross-checking with
   `gh pr view 269 --json state,mergedAt` confirms `"state": "MERGED"`,
   `"mergedAt": "2026-08-31T06:20:33Z"`. `git fetch origin dev` further shows
   the merge commit `209c15e` ("Merge pull request #269 from
   byx-darwin/feat/266-gf-issue-edit") is already an ancestor of
   `origin/dev`. The `gf-review` skill's preconditions require the target PR
   to be open ("Confirm open, not draft/merged"; "PR not found / closed →
   Stop. Check number."). A merged PR cannot receive a new review verdict
   through this flow.

2. **Self-review conflict.**
   `gf auth status` shows the authenticated account is `byx-darwin` (GitHub
   login), and PR #269's `author.login` is also `byx-darwin`. The underlying
   commits (`c292446`, `3468569`) are authored by `baoyuexing
   <baoyuexing@vmos.cn>`, the same git identity active in this session. The
   `gf-review` skill explicitly prohibits this: "Reviewing your own PR — This
   skill prohibits self-review... refuse self-review requests."

Either blocker alone is sufficient to stop the flow; both are present. No
`gf review approve/request-changes/comment` call was made.

## Diff verification performed (informational only)

Despite not submitting a verdict, the diff (`dev...feat/266-gf-issue-edit`)
was inspected for correctness, error handling, test coverage, and
convention consistency, per the request not to trust the PR description
blindly.

**Files touched** (495 insertions, 4 deletions, 5 files):
- `crates/core/src/issue.rs` — `EditIssueArgs { title, body }` +
  `IssueProvider::edit` trait method.
- `crates/github/src/issue.rs` — `gh issue edit --title/--body`, then
  `view()`.
- `crates/gitlab/src/issue.rs` — `glab issue update --title/--description`
  (documented rationale: `glab` has no `issue edit` subcommand for
  title/body; `edit` is reserved for label ops), then `view()`.
- `crates/gitcode/src/issue.rs` — `<gitcode_binary> issue edit --title/--body`,
  then `view()`.
- `apps/cli/src/commands/issue.rs` — `IssueCommand::Edit`, reusing the
  existing `resolve_body()` helper; new `ensure_edit_has_changes()` rejects
  edits with neither `--title` nor `--body`/`--body-file` set.

**Correctness / conventions:**
- No `unwrap()`/`expect()` introduced; all fallible paths return
  `Result<T>` and propagate through `CoreError` (`Platform`/`Cli`
  variants), consistent with sibling methods (`create`, `add_label`, etc.)
  in the same files.
- Each provider follows the existing "mutate then re-`view()` for canonical
  data" pattern already used by other partial-update operations in this
  codebase — consistent, not a new idiom.
- Doc comments include `# Errors` sections per public trait/method, matching
  `CLAUDE.md`'s public-API documentation requirement.
- Partial-update semantics (only pushing `--title`/`--body`/`--description`
  flags that are `Some`) are implemented identically across all three
  platform crates, verified with argv-assertion tests
  (`recorded_calls()[0].1` / `runner.calls()[0]`).
- CLI-level guard `ensure_edit_has_changes` correctly rejects the
  no-op-edit case (`--title` and `--body`/`--body-file` both absent) with a
  clear error message; tests cover title-only, body-only, both, and
  neither.

**Observations (non-blocking, informational only — no verdict is being
formed):**
- `IssueProvider::edit` itself does not enforce "at least one field set" —
  it is enforced only at the `apps/cli` layer via
  `ensure_edit_has_changes`. This is defensible (a provider-level partial
  update API is naturally permissive) but means any other future caller of
  the trait must re-implement the same guard. Each provider's own
  `EditIssueArgs::default()` error-path test exercises this permissive
  behavior only for CLI-failure simulation, not as a design statement.
- `--body-file` (via the pre-existing, reused `resolve_body()` helper) reads
  an arbitrary user-supplied path with `std::fs::read_to_string` and does
  not route through `SafePath` as `CLAUDE.md` requires for externally
  supplied file paths. This is pre-existing behavior shared with the
  `create`/`comment` subcommands (not introduced by this PR), so it is not
  a regression, but the `edit` subcommand extends usage of a helper that
  already diverges from the `SafePath` convention.

No correctness issues were found in the code introduced by this PR. Test
coverage (success path with view-refresh, argv-shape assertion, and
CLI-failure mapping) is present and symmetric across all three platform
crates plus CLI arg-parsing and validation tests.

## Required next step

A human reviewer (i.e. not the PR author) must either:
- Confirm no further review action is needed since the PR is already merged
  into `dev` (merge commit `209c15e`), and archive this report as
  informational only, or
- If a retroactive verdict is still desired, have a different account
  record it directly on the merged PR through the GitHub/GitLab/GitCode UI
  or API — outside the scope of what the `gf-review` skill permits an
  automated agent authenticated as the PR's own author to do.

Separately, the pre-existing `resolve_body()` / `--body-file` path-handling
gap (no `SafePath` validation) noted above may be worth a follow-up issue if
the team wants CLI file-path arguments brought in line with `CLAUDE.md`'s
input-boundary rules; it was not introduced by this PR.
