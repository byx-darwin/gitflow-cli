# Design: Harden temp-file-path tests against shared fixed filenames

**Issue:** [#301](https://github.com/byx-darwin/gitflow-cli/issues/301) (follow-up of #289)
**Classification:** Bounded (per `superpowers:brainstorming`) — existing test flow, no production behavior change, single approved approach.

## Problem

Four unit tests share the same fragile pattern: they write a **fixed-name** file into the shared OS temp directory (`std::env::temp_dir().join("<fixed name>.md")`), read it back through the function under test, then manually `remove_file` it.

```rust
let dir = std::env::temp_dir();
let path = dir.join("gitflow_test_commit_comment.md");
std::fs::write(&path, "commit comment from file").expect("write temp file");
let result = resolve_comment_body(None, Some(path.to_string_lossy().into_owned()));
let _ = std::fs::remove_file(&path);
assert!(result.is_ok());
```

Affected tests (all in `apps/cli/src/commands/`):

| File | Test | Fixed filename |
|------|------|-----------------|
| `commit.rs:238` | `test_should_resolve_comment_body_from_file` | `gitflow_test_commit_comment.md` |
| `issue.rs:448` | `test_should_resolve_body_from_file` | `gitflow_test_body.md` |
| `pr.rs:587` | `test_should_resolve_body_from_file` | `gitflow_test_pr_body.md` |
| `release.rs:368` | `test_should_resolve_body_from_file` | `gitflow_release_body.md` |

A single occurrence of `test_should_resolve_comment_body_from_file` failing on `Test (windows-latest)` (run [33346653353](https://github.com/byx-darwin/gitflow-cli/actions/runs/33346653353)) with unchanged production code is consistent with — though not proof of — a shared-path collision (stale file from a prior crashed run, antivirus lock, concurrent test-binary invocation on the same filename).

## Approach

Replace the manual `temp_dir().join(<fixed name>)` + `remove_file` pattern with `tempfile::NamedTempFile` (already a workspace dependency, already used elsewhere in the codebase — `apps/cli/src/commands/skills.rs`, `workflow.rs`, `crates/release-signer/src/main.rs`, `crates/e2e-core/src/scratch.rs`):

```rust
let file = tempfile::NamedTempFile::new().expect("create temp file");
std::fs::write(file.path(), "commit comment from file").expect("write temp file");
let result = resolve_comment_body(None, Some(file.path().to_string_lossy().into_owned()));
assert!(result.is_ok());
assert_eq!(result.expect("already checked"), "commit comment from file");
// `file` drops here, auto-deleting the temp path — no manual remove_file needed.
```

Each `NamedTempFile::new()` call gets a unique OS-generated path, eliminating the shared-name collision risk regardless of whether the original failure was environmental or a genuine race. `Drop` cleans up automatically, so the manual `remove_file` call is removed too — one less thing that can silently fail (the existing code already discards its `Result` with `let _ =`).

No alternative approaches were considered: this is the same crate/pattern already used elsewhere in the codebase, so introducing something else (manual UUID suffixing, a different tempfile crate) would add inconsistency for no benefit. Discussed and approved in chat during Phase 1 brainstorming (bounded path — no separate design doc was required by the brainstorming skill itself; this file exists to satisfy `gf-workflow`'s Gate 1→2 evidence requirement).

## Scope

- **In scope:** the 4 tests listed above, in the 4 files listed above.
- **Out of scope:** `resolve_comment_body` / `resolve_body` / `SafePath` production logic — no behavior change.
- **Out of scope:** any other test in the codebase using `std::env::temp_dir()` outside this specific fixed-filename-read-back pattern.

## Testing

- `make test` (or `cargo nextest run`) must stay green locally.
- CI `Test (windows-latest)` job must stay green — this is the job the original flake was observed on.
- No new tests are needed; the 4 existing tests are modified in place, same assertions.

## Acceptance Criteria (from Issue #301)

- [ ] `test_should_resolve_comment_body_from_file` (and the equivalent tests in `issue.rs`/`pr.rs`/`release.rs`) use a uniquely-named temp path per test run instead of a fixed filename.
- [ ] `make test` and the Windows CI job stay green.
- [ ] No behavior change to `resolve_comment_body` / `SafePath` — this is a test-isolation hardening only.
