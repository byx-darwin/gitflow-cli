# gf issue edit — Design

- **Issue:** #266
- **Classification:** bounded (extends an existing flow already present in this repo)
- **Date:** 2026-08-31

## Problem

`gf issue` has `create` / `list` / `view` / `close` / `reopen` / `comment` / `comments` /
`add-label` / `remove-label`, but no way to edit an existing Issue's title or body.
Today the only workaround is appending a "revision" comment, which leaves the body
stale and revision history scattered across comments.

## Goal

Add `gf issue edit <number>` supporting partial updates:
- `--title <TITLE>` — new title
- `--body <BODY>` / `--body-file <FILE>` — new body (overwrite), same semantics as
  `gf issue create`'s existing `--body`/`--body-file` pair
- Any field not passed keeps its current value

## Approach

Each platform adapter already wraps a CLI (`gh`, `glab`, the `gitcode` binary) rather
than calling REST directly for most operations, and each already has an `issue edit`
subprocess call for label mutations (`add_labels`/`remove_label`). This design extends
that existing call with `--title`/`--body` flags instead of introducing new REST calls
(unlike `close`/`reopen`, which moved to `gh api ... PATCH` specifically to dodge
`gh issue view`'s GraphQL field drift — `gh issue edit --title --body` is a stable,
purpose-built subcommand, so no such workaround is needed here).

## Changes

1. **`crates/core/src/issue.rs`**
   - `EditIssueArgs { title: Option<String>, body: Option<String> }`
   - `IssueProvider::edit(&self, number: u64, args: EditIssueArgs) -> Result<IssueData>`

2. **`crates/github/src/issue.rs`**
   - `gh issue edit <number> --repo <repo> [--title T] [--body B]`, then `self.view(number)`
     for canonical data — same pattern as `add_labels`/`remove_label`.

3. **`crates/gitlab/src/issue.rs`**
   - `glab issue update <number> --repo <repo> [--title T] [--description D]` — glab
     reserves `edit` for label ops; title/body live under `update`. Exact flag name
     (`--description` vs `--body`) to be confirmed against `glab issue update --help`
     during implementation.

4. **`crates/gitcode/src/issue.rs`**
   - `<gitcode_binary> issue edit <number> -R <repo> [--title T] [--body B] --json`,
     parsed via the existing `IssueApiResponse`/`CloseApiResponse`-style struct already
     used by `close`.

5. **`apps/cli/src/commands/issue.rs`**
   - New `IssueCommand::Edit { number, title: Option<String>, body: Option<String>, body_file: Option<String> }`,
     reusing `resolve_body()`.
   - Error if neither `--title` nor `--body`/`--body-file` is provided (no-op edit).

## Testing

TDD per platform, matching each `issue.rs`'s existing style:
- CLI arg-parsing tests (clap) for `Edit`.
- Provider tests via `MockCommandRunner`/`SequencedMockCommandRunner`: success, platform
  failure, and partial-update paths (title-only, body-only, both).

## Out of scope

- No changes to `create`/`view`/`list`/`close`/`reopen`/label commands.
- No new error types — reuses `parse_gh_error`/`parse_gitcode_error`/GitLab's existing
  error path.
