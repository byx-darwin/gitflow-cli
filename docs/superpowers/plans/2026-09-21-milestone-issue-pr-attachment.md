# Milestone Issue/PR Attachment Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let `gf issue create`/`edit`/`list` and `gf pr create` attach/detach/filter by milestone, across GitHub, GitLab, and GitCode, so `gf milestone list`'s `openIssues`/`closedIssues` counters reflect real data instead of always being 0.

**Architecture:** `gitflow-core` gains a lightweight `MilestoneRef{number, title}` type (not the full `MilestoneData`), new `milestone` fields on `IssueData`/`PrData`/`CreateIssueArgs`/`CreatePrArgs`/`ListIssueArgs`, a three-state `EditIssueArgs.milestone: Option<Option<String>>` for set/unset/no-change, and a new `resolve_milestone_identifier()` helper that turns a user-supplied `--milestone <NUMBER|TITLE>` into a canonical `MilestoneRef` by listing the repo's milestones once and matching by number then by exact title. Each platform adapter calls this resolver, then passes whichever native form the underlying CLI needs: GitHub/GitLab use `title` (GitLab's own `number` is a project-scoped `iid`, not the global `id` its `--milestone` flag's "ID" actually means — passing title sidesteps this entirely), GitCode uses `number`. GitCode's `pr create` has no milestone flag at all, so milestone attachment there is a create-then-`pr edit` two-step, with a clear error (not silence) if the second step fails.

**Tech Stack:** Rust 2024, existing per-platform `CommandRunner` abstraction (`gh`/`glab`/`gitcode` CLI wrappers).

**Spec:** `docs/superpowers/specs/2026-09-21-milestone-issue-pr-attachment-design.md` (see §3.3 for a mid-design scope correction: no `EditPrArgs`, no PR list filter — AC only requires `gf pr create`, and `gf pr edit` doesn't exist in this codebase at all).

## Global Constraints

- PR side is **create-only**: no `EditPrArgs`, no PR list milestone filter. Do not add a `gf pr edit` command — out of scope, would be unrequested new surface area.
- `MilestoneRef` is `{number: u64, title: String}` only — never embed the full `MilestoneData` (due date, progress counters) into `IssueData`/`PrData`.
- `IssueData.milestone`/`PrData.milestone` are `Option<MilestoneRef>` with `#[serde(skip_serializing_if = "Option::is_none")]` — omit when unset, matching the existing convention for `name`/`body`/`author`/`published_at` in these same structs.
- GitHub and GitLab adapters pass **title** to the underlying CLI's `--milestone` flag, never a number (GitLab's `--milestone` "ID" is the global `id`, not the project-scoped `iid` this codebase's `MilestoneData.number` holds — passing a number would silently target the wrong milestone or fail).
- GitCode adapters pass **number**.
- `EditIssueArgs.milestone: Option<Option<String>>` — `None` = don't touch, `Some(None)` = unassign, `Some(Some(identifier))` = set to that milestone.
- Never silently ignore a platform limitation — if GitCode's `issue edit` turns out not to support unassignment, return a clear error naming the limitation, don't pretend it worked.
- Run `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic` on every touched crate; run full `make test` at the end of the last task.

---

### Task 1: `gitflow-core` — types and resolver

**Files:**
- Modify: `crates/core/src/types.rs` (add `MilestoneRef`)
- Modify: `crates/core/src/issue.rs` (`IssueData`, `CreateIssueArgs`, `EditIssueArgs`, `ListIssueArgs`)
- Modify: `crates/core/src/pr.rs` (`PrData`, `CreatePrArgs`)
- Modify: `crates/core/src/label.rs` (add `resolve_milestone_identifier`)
- Test: same files, `#[cfg(test)] mod tests`

**Interfaces:**
- Produces: `gitflow_core::types::MilestoneRef { number: u64, title: String }` (Debug, Clone, PartialEq, Eq, Serialize, Deserialize, camelCase).
- Produces: `gitflow_core::label::resolve_milestone_identifier(provider: &dyn MilestoneProvider, identifier: &str) -> Result<MilestoneRef>` — every later task (GitHub/GitLab/GitCode issue.rs/pr.rs) calls this exact signature.
- Produces: `IssueData.milestone: Option<MilestoneRef>`, `CreateIssueArgs.milestone: Option<String>`, `EditIssueArgs.milestone: Option<Option<String>>`, `ListIssueArgs.milestone: Option<String>`, `PrData.milestone: Option<MilestoneRef>`, `CreatePrArgs.milestone: Option<String>`.

- [ ] **Step 1: Write the failing test for `MilestoneRef`**

Add to `crates/core/src/types.rs`'s test module:

```rust
#[test]
fn test_should_serialize_milestone_ref_camel_case() {
    let m = MilestoneRef { number: 5, title: "v1.0".into() };
    let json = serde_json::to_string(&m).expect("serialize");
    assert!(json.contains("\"number\":5"));
    assert!(json.contains("\"title\":\"v1.0\""));
}

#[test]
fn test_should_roundtrip_milestone_ref() {
    let json = r#"{"number": 5, "title": "v1.0"}"#;
    let m: MilestoneRef = serde_json::from_str(json).expect("deserialize");
    assert_eq!(m.number, 5);
    assert_eq!(m.title, "v1.0");
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p gitflow-core types::tests::test_should_serialize_milestone_ref_camel_case types::tests::test_should_roundtrip_milestone_ref`
Expected: FAIL to compile — `MilestoneRef` doesn't exist yet.

- [ ] **Step 3: Add `MilestoneRef` to `crates/core/src/types.rs`**

Add near the other small shared types (e.g. next to `UserSummary`):

```rust
/// A lightweight reference to a milestone attached to an Issue or PR.
///
/// Deliberately does not embed the full `MilestoneData` (due date, progress
/// counters) — that would duplicate data already served by `gf milestone
/// list`/`view` and cost an extra API call on every issue/PR fetch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MilestoneRef {
    /// The milestone's number (platform-native numbering).
    pub number: u64,
    /// The milestone's title.
    pub title: String,
}
```

Ensure `Serialize`/`Deserialize` are already imported in this file (they are, used by other types in `types.rs`).

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p gitflow-core types::`
Expected: PASS.

- [ ] **Step 5: Add `milestone` fields to `IssueData`/`CreateIssueArgs`/`EditIssueArgs`/`ListIssueArgs` in `crates/core/src/issue.rs`**

In `IssueData` (after the `url` field or near `author`, doesn't matter, keep it near other optional metadata like `assignees`):

```rust
    /// The milestone this issue is attached to, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub milestone: Option<crate::types::MilestoneRef>,
```

In `CreateIssueArgs`:

```rust
    /// Milestone identifier to attach on creation (`NUMBER` or exact `TITLE`), if any.
    pub milestone: Option<String>,
```

In `EditIssueArgs`:

```rust
    /// Milestone change: `None` = don't touch; `Some(None)` = unassign;
    /// `Some(Some(identifier))` = set to this milestone (`NUMBER` or `TITLE`).
    pub milestone: Option<Option<String>>,
```

In `ListIssueArgs`:

```rust
    /// Filter by milestone identifier (`NUMBER` or exact `TITLE`), if any.
    pub milestone: Option<String>,
```

Because `EditIssueArgs` derives `Default`, `Option<Option<String>>` defaults to `None` automatically — no manual `Default` impl needed.

- [ ] **Step 6: Add `milestone` fields to `PrData`/`CreatePrArgs` in `crates/core/src/pr.rs`**

In `PrData`:

```rust
    /// The milestone this PR is attached to, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub milestone: Option<crate::types::MilestoneRef>,
```

In `CreatePrArgs`:

```rust
    /// Milestone identifier to attach on creation (`NUMBER` or exact `TITLE`), if any.
    pub milestone: Option<String>,
```

Do **not** add `EditPrArgs` or a `milestone` field to `ListPrArgs` — out of scope (see Global Constraints).

- [ ] **Step 7: Fix compile errors from struct-literal construction sites**

`cargo build --workspace` will now fail everywhere a `CreateIssueArgs`/`EditIssueArgs`/`ListIssueArgs`/`CreatePrArgs`/`IssueData`/`PrData` struct literal is built without the new field (tests in every platform crate, plus `apps/cli`). Run:

```bash
cargo build --workspace 2>&1 | grep "missing field\|missing structure fields"
```

For every test/production struct-literal site reported, add `milestone: None` (for the `Option`/`Option<Option>` fields) or `milestone: Vec::new()`/etc. as appropriate — always `None`/absent, since these are pre-existing call sites that predate milestone support. Do not guess at values.

- [ ] **Step 8: Write the resolver's failing tests**

Add to `crates/core/src/label.rs`'s test module. You'll need a minimal mock `MilestoneProvider` — check if one already exists in this file's tests (search for `impl MilestoneProvider for` in test code); if not, define one:

```rust
struct MockMilestoneProvider {
    milestones: Vec<MilestoneData>,
}

#[async_trait::async_trait]
impl MilestoneProvider for MockMilestoneProvider {
    async fn create(&self, _args: CreateMilestoneArgs) -> Result<MilestoneData> {
        unimplemented!("not used by resolver tests")
    }
    async fn list(&self, _limit: Option<u32>) -> Result<Paged<MilestoneData>> {
        Ok(Paged::new(self.milestones.clone(), crate::paging::PageMeta::default()))
    }
    async fn edit(&self, _number: u64, _args: CreateMilestoneArgs) -> Result<MilestoneData> {
        unimplemented!("not used by resolver tests")
    }
    async fn close(&self, _number: u64) -> Result<MilestoneData> {
        unimplemented!("not used by resolver tests")
    }
    async fn reopen(&self, _number: u64) -> Result<MilestoneData> {
        unimplemented!("not used by resolver tests")
    }
}

fn sample_milestone(number: u64, title: &str) -> MilestoneData {
    MilestoneData {
        number,
        title: title.to_string(),
        description: None,
        state: State::Open,
        due_on: None,
        closed_issues: 0,
        open_issues: 0,
    }
}

#[tokio::test]
async fn test_should_resolve_milestone_by_number() {
    let provider = MockMilestoneProvider {
        milestones: vec![sample_milestone(1, "v1.0"), sample_milestone(2, "v2.0")],
    };
    let resolved = resolve_milestone_identifier(&provider, "2").await.expect("resolve");
    assert_eq!(resolved, MilestoneRef { number: 2, title: "v2.0".into() });
}

#[tokio::test]
async fn test_should_resolve_milestone_by_title() {
    let provider = MockMilestoneProvider {
        milestones: vec![sample_milestone(1, "v1.0"), sample_milestone(2, "v2.0")],
    };
    let resolved = resolve_milestone_identifier(&provider, "v1.0").await.expect("resolve");
    assert_eq!(resolved, MilestoneRef { number: 1, title: "v1.0".into() });
}

#[tokio::test]
async fn test_should_prefer_number_match_over_title_match() {
    // Identifier "2" happens to also be a different milestone's title.
    let provider = MockMilestoneProvider {
        milestones: vec![sample_milestone(1, "2"), sample_milestone(2, "v2.0")],
    };
    let resolved = resolve_milestone_identifier(&provider, "2").await.expect("resolve");
    assert_eq!(resolved.number, 2, "numeric identifier must match by number first, not by a milestone whose title happens to equal the same string");
}

#[tokio::test]
async fn test_should_error_when_milestone_identifier_not_found() {
    let provider = MockMilestoneProvider {
        milestones: vec![sample_milestone(1, "v1.0")],
    };
    let result = resolve_milestone_identifier(&provider, "does-not-exist").await;
    assert!(result.is_err());
}
```

(Adjust `Paged::new`/`PageMeta::default()` construction to match this codebase's actual API if it differs — check `crates/core/src/paging.rs` for the real constructor before writing this; the plan's exact call shown here is a best-effort based on the pattern used elsewhere, verify before using.)

- [ ] **Step 9: Run to verify they fail**

Run: `cargo test -p gitflow-core label::tests::test_should_resolve_milestone`
Expected: FAIL to compile — `resolve_milestone_identifier` doesn't exist yet.

- [ ] **Step 10: Implement `resolve_milestone_identifier` in `crates/core/src/label.rs`**

```rust
/// Resolves a user-supplied milestone identifier (`--milestone <NUMBER|TITLE>`)
/// against the repository's milestone list, matching by number first (if the
/// identifier parses as `u64`) then by exact title.
///
/// # Errors
///
/// Returns an error if no milestone matches by either number or exact title.
pub async fn resolve_milestone_identifier(
    provider: &dyn MilestoneProvider,
    identifier: &str,
) -> crate::Result<crate::types::MilestoneRef> {
    let milestones = provider.list(None).await?;
    if let Ok(number) = identifier.parse::<u64>() {
        if let Some(m) = milestones.items.iter().find(|m| m.number == number) {
            return Ok(crate::types::MilestoneRef { number: m.number, title: m.title.clone() });
        }
    }
    milestones
        .items
        .iter()
        .find(|m| m.title == identifier)
        .map(|m| crate::types::MilestoneRef { number: m.number, title: m.title.clone() })
        .ok_or_else(|| {
            crate::CoreError::Platform(format!(
                "milestone '{identifier}' not found (matched by neither number nor exact title)"
            ))
        })
}
```

Check the exact field name for `Paged`'s inner list (`items` is used elsewhere in this codebase, e.g. `crates/gitcode/src/release.rs`'s tests reference `paged.items`) before finalizing — confirm via `crates/core/src/paging.rs`.

- [ ] **Step 11: Run to verify they pass**

Run: `cargo test -p gitflow-core label::`
Expected: PASS, all 4 new tests green.

- [ ] **Step 12: Run clippy and full core test suite**

Run: `cargo clippy -p gitflow-core --all-targets --all-features -- -D warnings -W clippy::pedantic`
Run: `cargo test -p gitflow-core`
Expected: both clean.

- [ ] **Step 13: Commit**

```bash
git add crates/core/src/types.rs crates/core/src/issue.rs crates/core/src/pr.rs crates/core/src/label.rs
git commit -m "feat(core): add MilestoneRef and resolve_milestone_identifier for issue/PR milestone attachment

Refs #357"
```

---

### Task 2: `gitflow-github` — issue and PR milestone support

**Files:**
- Modify: `crates/github/src/issue.rs` (`ISSUE_FIELDS` constant, `IssueApiResponse`-equivalent — this crate deserializes `gh`'s JSON directly into `IssueData`, check whether there's an intermediate struct or not before assuming; `create()`, `edit()`, `list_impl()`)
- Modify: `crates/github/src/pr.rs` (`PR_FIELDS` constant, `create()`)
- Test: same files

**Interfaces:**
- Consumes: `gitflow_core::label::resolve_milestone_identifier`, `MilestoneRef` from Task 1. `GitHubMilestoneProvider` (from `crates/github/src/label.rs`) must already exist — construct one with `GitHubMilestoneProvider::new(&self.repo)` (check the exact constructor name/signature in that file before using it).

- [ ] **Step 1: Investigate — confirm `gh issue view --json milestone` shape**

Before writing any code, check how `IssueData` currently gets populated from `gh`'s JSON in this crate: search `crates/github/src/issue.rs` for whether `IssueData` is deserialized directly (like `crates/github/src/release.rs` does for `ReleaseData`) or via an intermediate struct. `gh`'s `milestone` JSON field, when present, is an object with (at least) `number` and `title` keys matching this plan's `MilestoneRef` field names exactly — if `IssueData` is deserialized directly, adding `milestone: Option<MilestoneRef>` to the struct (done in Task 1) is enough with no extra mapping code, since `serde`'s camelCase rename plus matching field names does the work. Confirm this by writing the failing test in Step 2 before assuming.

- [ ] **Step 2: Write the failing test**

Add to `crates/github/src/issue.rs`'s test module (near `test_should_deserialize_release_data_from_json`-style tests — find the closest analog, likely near issue deserialization tests):

```rust
#[test]
fn test_should_deserialize_issue_with_milestone() {
    let gh_json = br#"{
        "number": 42,
        "title": "Test",
        "body": null,
        "state": "OPEN",
        "labels": [],
        "author": {"login": "octocat", "id": "1"},
        "assignees": [],
        "createdAt": "2026-01-01T00:00:00Z",
        "updatedAt": "2026-01-01T00:00:00Z",
        "milestone": {"number": 3, "title": "v2.0"},
        "url": "https://github.com/octocat/hello-world/issues/42"
    }"#;
    let issue: IssueData = serde_json::from_slice(gh_json).expect("valid IssueData JSON");
    assert_eq!(
        issue.milestone,
        Some(gitflow_core::types::MilestoneRef { number: 3, title: "v2.0".into() })
    );
}

#[test]
fn test_should_deserialize_issue_with_no_milestone() {
    let gh_json = br#"{
        "number": 42,
        "title": "Test",
        "body": null,
        "state": "OPEN",
        "labels": [],
        "author": {"login": "octocat", "id": "1"},
        "assignees": [],
        "createdAt": "2026-01-01T00:00:00Z",
        "updatedAt": "2026-01-01T00:00:00Z",
        "url": "https://github.com/octocat/hello-world/issues/42"
    }"#;
    let issue: IssueData = serde_json::from_slice(gh_json).expect("valid IssueData JSON");
    assert!(issue.milestone.is_none());
}
```

(Adjust field names/casing to match this file's actual existing sample JSON conventions — copy from a nearby existing test's fixture rather than retyping from scratch, only adding the `milestone` key.)

- [ ] **Step 3: Run to verify (may already pass or fail depending on Step 1's finding)**

Run: `cargo test -p gitflow-github issue::tests::test_should_deserialize_issue_with_milestone issue::tests::test_should_deserialize_issue_with_no_milestone`

If `IssueData` deserializes directly from `gh`'s JSON (no intermediate struct), this likely already PASSES since Task 1 added the field with matching name/casing — that's fine, it's still a real regression-pinning test, not wasted effort. If there IS an intermediate struct, this FAILS until you add a `milestone` field there and to its `From` impl — do that now, mirroring how `author`/`assignees` are already mapped in that same struct.

- [ ] **Step 4: Add `milestone` to `ISSUE_FIELDS` and `PR_FIELDS` constants**

In `crates/github/src/issue.rs`:

```rust
const ISSUE_FIELDS: &str =
    "number,title,body,state,labels,author,assignees,createdAt,updatedAt,milestone,url";
```

In `crates/github/src/pr.rs`, find the `PR_FIELDS` constant (currently `"number,title,body,state,isDraft,author,baseRefName,headRefName,createdAt,..."`, spans two lines) and add `milestone` to the list in the same way.

- [ ] **Step 5: Wire `--milestone` into `issue create`**

In `crates/github/src/issue.rs`'s `create()` (around line 178), after resolving the body/labels/assignees section and before spawning `gh`:

```rust
        let resolved_milestone_title;
        if let Some(identifier) = &args.milestone {
            let milestone_provider = crate::GitHubMilestoneProvider::new(&self.repo);
            let resolved = gitflow_core::label::resolve_milestone_identifier(
                &milestone_provider,
                identifier,
            )
            .await?;
            resolved_milestone_title = resolved.title;
            cmd_args.push("--milestone");
            cmd_args.push(&resolved_milestone_title);
        }
```

Place this block so `resolved_milestone_title` outlives `cmd_args` (declare it before the `if let`, as shown, matching the existing `labels_joined`/`assignees_joined` pattern in this same function — check the exact current variable declaration order before inserting, since `cmd_args` is built as a `Vec<&str>` borrowing from locals that must not be dropped before `self.runner.run(...)` is called). Verify the exact constructor name for `GitHubMilestoneProvider` in `crates/github/src/label.rs` before using it — likely `::new(repo: impl Into<String>)` matching every other provider in this codebase, but confirm.

- [ ] **Step 6: Wire `--milestone`/`--remove-milestone` into `issue edit`**

In `crates/github/src/issue.rs`'s `edit()` (around line 264), after the `title`/`body` handling:

```rust
        let resolved_milestone_title;
        match &args.milestone {
            None => {}
            Some(None) => {
                cmd_args.push("--remove-milestone");
            }
            Some(Some(identifier)) => {
                let milestone_provider = crate::GitHubMilestoneProvider::new(&self.repo);
                let resolved = gitflow_core::label::resolve_milestone_identifier(
                    &milestone_provider,
                    identifier,
                )
                .await?;
                resolved_milestone_title = resolved.title;
                cmd_args.push("--milestone");
                cmd_args.push(&resolved_milestone_title);
            }
        }
```

- [ ] **Step 7: Wire `--milestone` filter into `issue list`**

In `crates/github/src/issue.rs`'s `list_impl()`, alongside the existing `--label`/`--search` handling:

```rust
        let resolved_milestone_title;
        if let Some(identifier) = &args.milestone {
            let milestone_provider = crate::GitHubMilestoneProvider::new(repo);
            let resolved = gitflow_core::label::resolve_milestone_identifier(
                &milestone_provider,
                identifier,
            )
            .await?;
            resolved_milestone_title = resolved.title;
            cmd_args.push("--milestone");
            cmd_args.push(&resolved_milestone_title);
        }
```

(Match this to the exact variable name used for the repo in `list_impl` — it's `repo`, a parameter, per the existing code at line ~139; confirm before inserting.)

- [ ] **Step 8: Wire `--milestone` into `pr create`**

In `crates/github/src/pr.rs`'s `create()` (around line 144), same pattern as Step 5 but using a `GitHubMilestoneProvider` for `repo` (the effective repo variable already computed in this function, not `self.repo` — check the existing code, it uses `let repo = args.repo.as_deref().unwrap_or(&self.repo);`).

- [ ] **Step 9: Run all tests to verify GREEN**

Run: `cargo test -p gitflow-github issue:: pr::`
Expected: PASS, including the two new tests from Step 2 and any new create/edit/list milestone tests you add mirroring the existing test style for `--label`/`--search` wiring in this file (add at least one test per wired command proving the right `--milestone`/`--remove-milestone` argv is produced, using the `MockCommandRunner` pattern already used throughout this file).

- [ ] **Step 10: Run clippy**

Run: `cargo clippy -p gitflow-github --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: clean.

- [ ] **Step 11: Commit**

```bash
git add crates/github/src/issue.rs crates/github/src/pr.rs
git commit -m "feat(github): support --milestone on issue create/edit/list and pr create

Refs #357"
```

---

### Task 3: `gitflow-gitlab` — issue and MR milestone support

**Files:**
- Modify: `crates/gitlab/src/issue.rs` (`IssueApiResponse` struct, `create()`, `edit()`, `list_impl()`)
- Modify: `crates/gitlab/src/mr.rs` (`create()`, and its own API response struct if the milestone needs to appear there too — check if `PrData` round-trips through an intermediate struct here the way `ReleaseApiResponse` does elsewhere in this crate)
- Test: same files

**Interfaces:**
- Consumes: `resolve_milestone_identifier`, `MilestoneRef` from Task 1. Construct `GitLabMilestoneProvider` (check exact name/constructor in `crates/gitlab/src/label.rs`) for the same repo.
- All milestone identifiers passed to `glab` are **titles**, never numbers (Global Constraints — GitLab's own `--milestone` "ID" means the global `id`, not this codebase's `MilestoneData.number`, which is `iid`).

- [ ] **Step 1: Investigate real milestone JSON shape from `glab issue view --output json`**

This codebase's `IssueApiResponse` (in `crates/gitlab/src/issue.rs`, around line 197) is a snake_case intermediate struct with `iid`, `title`, `description`, etc. GitLab's embedded milestone object shape on an issue is NOT yet confirmed in this codebase's own real-data testing. Before writing the mapping, use the GitLab test repo noted in this session's memory (`/Users/xs/Documents/workspce/xiaosuan/iproost/iproost-docs`, remote `192.168.230.23`) to: create a test milestone, attach it to a test issue via `glab issue update <n> --milestone <title>`, then run `glab issue view <n> --output json` and inspect the real `milestone` key's shape. Record the exact fields (likely `id`, `iid`, `title`, `state`, ... — GitLab's REST milestone object). Clean up the test milestone/issue afterward.

- [ ] **Step 2: Write the failing test using the real shape found in Step 1**

Add to `crates/gitlab/src/issue.rs`'s test module, using the ACTUAL captured JSON shape from Step 1 (not a guess):

```rust
#[test]
fn test_should_deserialize_issue_with_milestone() {
    // Real shape captured from `glab issue view --output json` against
    // <test repo/issue from Step 1> on 2026-09-21.
    let json = r#"{
        "iid": 42,
        "title": "Test",
        "description": null,
        "state": "opened",
        "labels": [],
        "milestone": {"id": 999, "iid": 3, "title": "v2.0", "state": "active"},
        "web_url": "http://192.168.230.23/iproost/iproost-docs/-/issues/42"
    }"#;
    let api: IssueApiResponse = serde_json::from_str(json).expect("deserialize");
    let issue: IssueData = api.into();
    assert_eq!(
        issue.milestone,
        Some(gitflow_core::types::MilestoneRef { number: 3, title: "v2.0".into() })
    );
}
```

(Replace the JSON literal with what Step 1 actually captured — this plan's version is a placeholder shape based on GitLab's public REST API docs, not yet verified against this codebase's real CLI output; do not skip Step 1.)

- [ ] **Step 3: Run to verify it fails**

Run: `cargo test -p gitflow-gitlab issue::tests::test_should_deserialize_issue_with_milestone`
Expected: FAIL — `IssueApiResponse` has no `milestone` field yet.

- [ ] **Step 4: Add `milestone` to `IssueApiResponse` and its `From` impl**

Following the `number: api.iid.unwrap_or(api.id)` convention from `crates/gitlab/src/label.rs:524` (Task 1's investigation), add a small nested struct and field:

```rust
#[derive(Debug, Clone, Deserialize)]
struct MilestoneRefApi {
    #[serde(default)]
    id: u64,
    #[serde(default)]
    iid: Option<u64>,
    title: String,
}

impl From<MilestoneRefApi> for gitflow_core::types::MilestoneRef {
    fn from(api: MilestoneRefApi) -> Self {
        Self {
            number: api.iid.unwrap_or(api.id),
            title: api.title,
        }
    }
}
```

Add to `IssueApiResponse`:

```rust
    #[serde(default)]
    milestone: Option<MilestoneRefApi>,
```

And in its `From<IssueApiResponse> for IssueData` impl, add:

```rust
            milestone: api.milestone.map(Into::into),
```

- [ ] **Step 5: Run to verify it passes**

Run: `cargo test -p gitflow-gitlab issue::tests::test_should_deserialize_issue_with_milestone`
Expected: PASS.

- [ ] **Step 6: Wire `--milestone` into `issue create`**

In `crates/gitlab/src/issue.rs`'s `create()` (around line 360), same pattern as GitHub Task 2 Step 5, but:
- Use `GitLabMilestoneProvider` (check exact constructor in `crates/gitlab/src/label.rs`, likely takes `&self.repo_target` given this crate's pattern of using `repo_target` rather than `repo` for the actual API-facing identifier — confirm before use)
- Push `--milestone` with the resolved **title**

- [ ] **Step 7: Wire `--milestone`/unassign into `issue edit`**

In `crates/gitlab/src/issue.rs`'s `edit()` (around line 445, which calls `glab issue update`), same three-way match as GitHub Task 2 Step 6, but:
- For unassignment, this crate's doc comment (line ~439) already notes `glab` has no `issue edit`, only `issue update` — confirm via `glab issue update --help` whether passing `--milestone ""` or `--milestone 0` is the correct unassign incantation (the earlier platform-capability research found: `-m --milestone Title of the milestone to assign Set to "" or 0 to unassign.`). Use `cmd_args.push("--milestone"); cmd_args.push("");` for the `Some(None)` case.

- [ ] **Step 8: Wire `--milestone` filter into `issue list`**

In `crates/gitlab/src/issue.rs`'s `list_impl()` (around line 291), same pattern as GitHub Task 2 Step 7. Note: `glab issue list --milestone` per the platform research only documented accepting an **ID** ("Filter issue by milestone <id>") — investigate during implementation whether it also accepts title (test empirically against the GitLab test repo); if it truly only accepts a numeric ID, this is a case where the resolver's `title`-preference (Global Constraints) doesn't apply for LIST filtering specifically — if so, document this exception clearly in a code comment explaining why list filtering uses a different form than create/edit, rather than silently doing something inconsistent. Do not guess — verify against the real `192.168.230.23` test repo.

- [ ] **Step 9: Wire `--milestone` into `mr create`**

In `crates/gitlab/src/mr.rs`'s `create()` (around line 337), same pattern as GitHub Task 2 Step 8, using `GitLabMilestoneProvider` and pushing the resolved title.

- [ ] **Step 10: Run all tests to verify GREEN**

Run: `cargo test -p gitflow-gitlab issue:: mr::`
Expected: PASS, including new argv-shape tests for create/edit/list/mr-create mirroring this crate's existing `MockCommandRunner`-based test style.

- [ ] **Step 11: Run clippy**

Run: `cargo clippy -p gitflow-gitlab --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: clean.

- [ ] **Step 12: Commit**

```bash
git add crates/gitlab/src/issue.rs crates/gitlab/src/mr.rs
git commit -m "feat(gitlab): support --milestone on issue create/edit/list and mr create

Refs #357"
```

---

### Task 4: `gitflow-gitcode` — issue and PR milestone support (including the create+edit two-step for PRs)

**Files:**
- Modify: `crates/gitcode/src/issue.rs` (`IssueApiResponse` struct, `create()`, `edit()`, `list_impl()`)
- Modify: `crates/gitcode/src/pr.rs` (`PrApiResponse` struct, `create()` — two-step create+edit)
- Test: same files

**Interfaces:**
- Consumes: `resolve_milestone_identifier`, `MilestoneRef` from Task 1. Construct `GitCodeMilestoneProvider` (check exact name in `crates/gitcode/src/label.rs`).
- All milestone identifiers passed to `gitcode` are **numbers** (Global Constraints).

- [ ] **Step 1: Investigate real milestone JSON shape and issue-edit-unassign semantics**

Using this session's GitCode test repo (`byx-darwin/NexaTrade`, already authenticated via `gitcode`/`gc` CLI at `~/Library/Python/3.14/bin/`): create a test milestone, attach it to a test issue via `gitcode issue create --milestone <number>` or `gitcode issue edit --milestone <number>`, then run with `--json` to inspect the real embedded milestone shape. **Also** specifically test whether `gitcode issue edit --milestone 0` (or any other value) unassigns a milestone — the platform capability research could not confirm this. If there is no way to unassign via this CLI, this is a real platform limitation: implement `create`/`edit`-with-a-value fully, but for the `Some(None)` (unassign) case in `edit()`, return a clear `CoreError::Platform` explaining GitCode's CLI has no unassign capability — do not fake success. Clean up test artifacts afterward (delete the test milestone/issue if the CLI supports it, or note if it can't be deleted, same as the release-delete-405 finding from Issue #368's dogfooding).

- [ ] **Step 2: Write the failing test using the real shape found in Step 1**

Add to `crates/gitcode/src/issue.rs`'s test module, using the actual captured shape:

```rust
#[test]
fn test_should_deserialize_issue_with_milestone() {
    // Real shape captured from `gitcode issue view --json` on <test repo> 2026-09-21.
    let json = br#"{
        "number": "42",
        "title": "Test",
        "body": null,
        "state": "open",
        "html_url": "https://gitcode.com/owner/repo/issues/42",
        "milestone": {"number": 3, "title": "v2.0"}
    }"#;
    let api: IssueApiResponse = serde_json::from_slice(json).expect("deserialize");
    let issue: IssueData = api.into();
    assert_eq!(
        issue.milestone,
        Some(gitflow_core::types::MilestoneRef { number: 3, title: "v2.0".into() })
    );
}
```

(Replace with the real captured shape from Step 1 — GitCode's field naming has repeatedly diverged from assumptions in this codebase's history, e.g. #368's `id` being completely absent rather than `null`; do not skip the real verification.)

- [ ] **Step 3: Run to verify it fails, then add `milestone` to `IssueApiResponse`**

Add a nested struct mirroring the shape found in Step 1 (structure depends on findings — if GitCode's milestone number is a bare integer like other numeric IDs in this file, or a string like `IssueApiResponse.number` itself is, verify which), and wire it into the `From<IssueApiResponse> for IssueData` impl the same way `labels`/`assignees` are handled in this file (around line 68-73).

- [ ] **Step 4: Wire `--milestone` into `issue create`**

In `crates/gitcode/src/issue.rs`'s `create()` (around line 354), same pattern as GitHub/GitLab, using `GitCodeMilestoneProvider` and pushing the resolved **number** (as a `.to_string()`, kept alive alongside the existing `binary` local, matching this function's existing borrow lifetime pattern for `cmd_args: Vec<&str>`).

- [ ] **Step 5: Wire `--milestone` into `issue edit`, with the unassign limitation from Step 1**

In `crates/gitcode/src/issue.rs`'s `edit()` (around line 426), three-way match like the other platforms. If Step 1 found no unassign capability, the `Some(None)` arm returns an error immediately (before spawning any command):

```rust
            Some(None) => {
                return Err(gitflow_core::CoreError::Platform(
                    "GitCode CLI does not support unassigning a milestone from an issue".into(),
                ));
            }
```

(Only write this if Step 1's investigation actually confirms the limitation — if `--milestone 0` or similar does work, wire it properly instead and skip this error path.)

- [ ] **Step 6: Wire `--milestone` filter into `issue list`**

In `crates/gitcode/src/issue.rs`'s `list_impl()` (around line 313), same pattern, using the resolved number (the earlier platform capability research found `gitcode issue list --milestone` accepts both title and number natively — but per this plan's Global Constraint of "always resolve then use the platform's Global-Constraint-designated native form" for consistency, still resolve first and pass the number).

- [ ] **Step 7: Implement the create+edit two-step for `pr create`**

In `crates/gitcode/src/pr.rs`'s `create()` (around line 281), after the existing PR creation succeeds and `api.into()` produces the `PrData`, but before returning:

```rust
        let mut pr_data: PrData = api.into();

        if let Some(identifier) = &args.milestone {
            let milestone_provider = crate::GitCodeMilestoneProvider::new(&self.repo);
            let resolved = gitflow_core::label::resolve_milestone_identifier(
                &milestone_provider,
                identifier,
            )
            .await?;
            let number_str = pr_data.number.to_string();
            let milestone_number_str = resolved.number.to_string();
            let edit_output = self
                .runner
                .run(
                    &binary,
                    &[
                        "pr",
                        "edit",
                        &number_str,
                        "--repo",
                        args.repo.as_deref().unwrap_or(&self.repo),
                        "--milestone",
                        &milestone_number_str,
                    ],
                )
                .await
                .map_err(|e| {
                    CoreError::Platform(format!(
                        "PR #{} created, but attaching milestone failed to spawn: {e}",
                        pr_data.number
                    ))
                })?;
            if !edit_output.status.success() {
                return Err(CoreError::Platform(format!(
                    "PR #{} created, but attaching milestone failed: {}",
                    pr_data.number,
                    String::from_utf8_lossy(&edit_output.stderr)
                )));
            }
            pr_data.milestone = Some(resolved);
        }

        Ok(pr_data)
```

Adjust variable names to match the exact existing code around line 326-330 (the plan's earlier investigation showed `let api: PrApiResponse = serde_json::from_slice(&output.stdout)...; Ok(api.into())` — restructure this tail to bind `pr_data` instead of returning directly, as shown above).

- [ ] **Step 8: Run all tests to verify GREEN**

Run: `cargo test -p gitflow-gitcode issue:: pr::`
Expected: PASS, including a new test proving the two-step create+edit sequence for PR milestone attachment (use `SequencedMockCommandRunner`, already used elsewhere in this crate's tests per Issue #368's work, to assert both the `pr create` and `pr edit` calls happen with the right argv in order) and a test proving milestone-attach failure surfaces a clear error without losing the created PR's data.

- [ ] **Step 9: Run clippy**

Run: `cargo clippy -p gitflow-gitcode --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: clean.

- [ ] **Step 10: Commit**

```bash
git add crates/gitcode/src/issue.rs crates/gitcode/src/pr.rs
git commit -m "feat(gitcode): support --milestone on issue create/edit/list, and pr create via create+edit two-step

GitCode's \`pr create\` has no --milestone flag at all, so milestone
attachment on PR creation is a create-then-edit sequence; a failure in
the second step surfaces a clear error naming the already-created PR
number rather than silently dropping the milestone request.

Refs #357"
```

---

### Task 5: `apps/cli` — wire `--milestone`/`--remove-milestone` CLI flags

**Files:**
- Modify: `apps/cli/src/commands/issue.rs` (`IssueCommand::Create`, `IssueCommand::Edit`, `IssueCommand::List` variants + their dispatch in `handle()`, `ensure_edit_has_changes`)
- Modify: `apps/cli/src/commands/pr.rs` (`PrCommand::Create` variant + dispatch)
- Test: same files

**Interfaces:**
- Consumes: `CreateIssueArgs.milestone`, `EditIssueArgs.milestone` (three-state), `ListIssueArgs.milestone`, `CreatePrArgs.milestone` from Task 1 (already implemented by platform adapters in Tasks 2-4).

- [ ] **Step 1: Add `--milestone` to `IssueCommand::Create`**

In `apps/cli/src/commands/issue.rs`, add to the `Create` variant (near the existing `label`/`assignee` fields, around line 42-48):

```rust
        /// 挂载的 milestone（编号或标题，可选）。
        #[arg(long)]
        milestone: Option<String>,
```

Update the `Create` match arm in `handle()` (around line 219-233) to destructure and pass it:

```rust
        IssueCommand::Create {
            title,
            body,
            body_file,
            label,
            assignee,
            milestone,
            repo: _,
        } => {
            let resolved_body = resolve_body(body, body_file)?;
            let args = CreateIssueArgs {
                title,
                body: resolved_body,
                labels: label,
                assignees: assignee,
                milestone,
            };
```

- [ ] **Step 2: Add `--milestone`/`--remove-milestone` to `IssueCommand::Edit`**

Add both flags to the `Edit` variant (around line 56-71):

```rust
        /// 设置 milestone（编号或标题，可选，与 `--remove-milestone` 二选一）。
        #[arg(long)]
        milestone: Option<String>,

        /// 取消 milestone 挂载（可选，与 `--milestone` 二选一）。
        #[arg(long = "remove-milestone")]
        remove_milestone: bool,
```

Update the `Edit` match arm (around line 241-259):

```rust
        IssueCommand::Edit {
            number,
            title,
            body,
            body_file,
            milestone,
            remove_milestone,
        } => {
            let resolved_body = resolve_body(body, body_file)?;
            let resolved_milestone = match (milestone, remove_milestone) {
                (Some(_), true) => {
                    return Err(miette::miette!(
                        "--milestone 与 --remove-milestone 不能同时指定"
                    ));
                }
                (Some(m), false) => Some(Some(m)),
                (None, true) => Some(None),
                (None, false) => None,
            };
            ensure_edit_has_changes(title.as_ref(), resolved_body.as_ref(), resolved_milestone.as_ref())?;
            let args = EditIssueArgs {
                title,
                body: resolved_body,
                milestone: resolved_milestone,
            };
```

- [ ] **Step 3: Update `ensure_edit_has_changes` to account for milestone**

Change the signature and body (around line 412-419):

```rust
fn ensure_edit_has_changes(
    title: Option<&String>,
    body: Option<&String>,
    milestone: Option<&Option<String>>,
) -> miette::Result<()> {
    if title.is_none() && body.is_none() && milestone.is_none() {
        return Err(miette::miette!(
            "Nothing to edit. Provide --title, --body/--body-file, --milestone, or --remove-milestone."
        ));
    }
    Ok(())
}
```

- [ ] **Step 4: Add `--milestone` to `IssueCommand::List`**

Add to the `List` variant (around line 74-90):

```rust
        /// 按 milestone 过滤（编号或标题，可选）。
        #[arg(long)]
        milestone: Option<String>,
```

Update the `List` match arm (around line 260-284) to destructure and pass `milestone` into `ListIssueArgs`.

- [ ] **Step 5: Add `--milestone` to `PrCommand::Create`**

In `apps/cli/src/commands/pr.rs`, add to the `Create` variant (near `closes`, check its exact location):

```rust
        /// 挂载的 milestone（编号或标题，可选）。
        #[arg(long)]
        milestone: Option<String>,
```

Update the `Create` match arm (around line 243-274) to pass `milestone` into `CreatePrArgs`.

- [ ] **Step 6: Fix any remaining compile errors from struct literal / clap derive test sites**

Run `cargo build -p gitflow-cli 2>&1 | grep "missing field"` and fix every reported test/production call site the same way as Task 1 Step 7.

- [ ] **Step 7: Run existing tests, then add new ones**

Run: `cargo test -p gitflow-cli issue:: pr::`
Expected: existing tests pass once compile errors are fixed. Add new tests mirroring this file's existing clap-parsing test style (e.g. `test_should_parse_pr_create_minimal`) for: `issue create --milestone`, `issue edit --milestone`, `issue edit --remove-milestone`, `issue edit --milestone X --remove-milestone` (must error), `issue list --milestone`, `pr create --milestone`.

- [ ] **Step 8: Run clippy and fmt**

Run: `cargo clippy -p gitflow-cli --all-targets --all-features -- -D warnings -W clippy::pedantic`
Run: `cargo +nightly fmt -- --check`
Expected: both clean.

- [ ] **Step 9: Commit**

```bash
git add apps/cli/src/commands/issue.rs apps/cli/src/commands/pr.rs
git commit -m "feat(cli): add --milestone/--remove-milestone flags to issue create/edit/list and pr create

Refs #357"
```

---

### Task 6: Full workspace validation + real cross-platform verification

**Files:** none (validation + manual verification task)

- [ ] **Step 1: Full workspace validation**

Run: `make test` — expect all pass, workspace-wide.
Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic` — expect clean.
Run: `cargo +nightly fmt -- --check` — expect no diff.

- [ ] **Step 2: Real end-to-end verification per platform (AC's hard requirement — "仅加参数而计数仍为 0 则为假")**

For each of the three platforms, using the test repos already established this session (GitHub: this repo itself or a scratch branch; GitLab: `192.168.230.23`/`iproost/iproost-docs`; GitCode: `byx-darwin/NexaTrade`):

1. `gf milestone create --title "dogfood-357-<platform>"` — note the returned number.
2. `gf issue create --title "milestone attach test" --milestone <number-or-title>` — confirm it succeeds and the returned `IssueData.milestone` is populated.
3. `gf milestone list --platform <platform> --output json | jq '.data[] | select(.title == "dogfood-357-<platform>")'` — confirm `openIssues` is now `1`, not `0`.
4. Clean up: close/delete the test issue, close/delete the test milestone (note if deletion isn't supported per platform, same as prior sessions' findings — don't force it).

Record the actual JSON output of step 3 for each platform as evidence — this is the concrete proof the AC's core complaint ("加了参数但计数仍为 0 则为假") is resolved, not just that the code compiles.

- [ ] **Step 3: Record findings**

If any platform's counter still doesn't update (e.g. due to API propagation delay, or a platform-specific quirk not caught by unit tests), stop and report rather than declaring the task done — this AC item exists specifically because "just adding the parameter" was called out in the Issue body as insufficient proof.

## Self-Review Notes

- **Spec coverage:** Design §3.1 (`MilestoneRef`) → Task 1. §3.2 (resolver) → Task 1. §3.3 (three-state edit, PR create-only, no `EditPrArgs`/list filter) → Tasks 1, 5. §3.4 (GitCode PR two-step) → Task 4. §3.5 (GitCode edit-unassign uncertainty) → Task 4 Step 1 (explicit live investigation before coding). §4 file table → Tasks 1-5 map 1:1. §5 (regression tests + real end-to-end verification) → each task's own tests plus Task 6.
- **Placeholder scan:** Tasks 3 and 4 contain explicitly-flagged "investigate first, then use the real shape" steps for GitLab's and GitCode's embedded milestone JSON — this is not a placeholder in the forbidden sense (vague "add appropriate handling"); it's an accurate reflection that this codebase's own established discipline (Issues #368, #377, #366) is to verify platform API shapes empirically rather than guess, and prior guesses in this codebase's history (Issue #360, #365) caused real bugs. The plan gives the reader exact tools (which test repo, which CLI, which command) to do that verification, and a placeholder JSON shape to replace with real findings — this is the correct way to plan work whose exact shape is genuinely unknown until tested, not a plan-writing shortcut.
- **Type consistency:** `MilestoneRef{number: u64, title: String}` from Task 1 is used identically (construction, comparison) across Tasks 2-5. `EditIssueArgs.milestone: Option<Option<String>>` and its three-arm match are used identically in Tasks 2-3-4's `edit()` wiring and Task 5's CLI translation.
- **Task ordering:** Task 1 (core) must land first (all others depend on its types). Tasks 2-4 (per-platform) are independent of each other and could theoretically run in parallel, but are listed sequentially since a single implementer stream processes them one at a time; a subagent-driven-development executor MAY run them as independent parallel dispatches if it judges the coordination overhead worth it. Task 5 (CLI) depends on all of Tasks 2-4 (needs every platform's `milestone` field wired for its own struct literals to compile). Task 6 depends on everything.
