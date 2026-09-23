# IssueData/PrData/ReviewData/CommentData/PipelineStatus created_at Optional Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Change 5 core type timestamp fields (`IssueData.created_at`/`updated_at`, `PrData.created_at`/`updated_at`, `ReviewData.submitted_at`, `CommentData.created_at`, `PipelineStatus.created_at`/`updated_at`) from `DateTime<Utc>` to `Option<DateTime<Utc>>` across `gitflow-core` and its three platform adapters, so a record with no real timestamp is shown as absent instead of silently faked as "just now" — the same fix Issue #366 already applied to `ReleaseData.created_at`.

**Architecture:** Each core type's timestamp field becomes `Option<DateTime<Utc>>` with `#[serde(skip_serializing_if = "Option::is_none")]`, mirroring `ReleaseData.created_at`/`published_at` (#366) and `JobData.started_at`/`completed_at` exactly. GitLab and GitCode each stop calling `.unwrap_or(now)`/`.unwrap_or_else(Utc::now)` in their `From<XxxApiResponse> for YyyData` impls and pass the already-optional API value straight through. GitHub needs no change for `PrData` (direct fail-loud deserialization), but its `IssueData`/`CommentData`/`ReviewData`/`PipelineStatus` conversions go through a different, already-known bug (Category C, parse-failure fallback, split to Issue #401) that still returns a concrete `DateTime<Utc>` — those 4 assignment sites need a minimal `Some(...)` wrap purely for type compatibility, without touching the fallback value itself. `apps/cli` needs no change: its generic `serde_json::Value`-based renderer already omits missing JSON keys correctly.

**Tech Stack:** Rust 2024, `chrono::DateTime<Utc>`, `serde`.

**Spec:** `docs/superpowers/specs/2026-09-22-issue-pr-review-pipeline-created-at-optional-design.md`

## Global Constraints

- 5 core type fields become `Option<DateTime<Utc>>` with `#[serde(skip_serializing_if = "Option::is_none")]` — no exceptions, no different serialization treatment between the 5 (design §3.1).
- `ReviewCommentData.created_at` (`crates/core/src/review.rs:75`) is explicitly OUT OF SCOPE — never touch it, it is unused by any platform crate (design §6).
- `PrData.merged_at` is explicitly OUT OF SCOPE — already `Option<DateTime<Utc>>` and already correct on both platforms (design §6).
- `crates/gitlab/src/review.rs`'s `approve()` (lines 153-191) gets a COMMENT ONLY, never a logic change (design §3.3, Category B: `Utc::now()` there is a genuine synchronous action timestamp, not a missing-data fabrication).
- Category C sites' underlying fallback VALUE is never changed in this plan (still `parse_api_datetime`'s `UNIX_EPOCH`+warn, still `.unwrap_or_else(|_| chrono::Utc::now())` in `github/review.rs`/`github/pipeline.rs`) — only wrapped in `Some(...)` for type compatibility. The behavior fix is Issue #401's job.
- No manual `Cargo.toml` version bump in any task's commit. Task 9 (the final task)'s commit message must include a `BREAKING CHANGE:` paragraph, worded exactly:
  `BREAKING CHANGE: IssueData.created_at/updated_at, PrData.created_at/updated_at, ReviewData.submitted_at, CommentData.created_at, and PipelineStatus.created_at/updated_at are now Option<DateTime<Utc>> instead of DateTime<Utc>. Callers that pattern-match or construct these types directly must handle the Option.`
  No other task's commit message includes this paragraph.
- `apps/cli/` gets zero changes (design §2.4 — confirmed no downstream consumer there).
- Run `cargo clippy -p <crate> --all-targets --all-features -- -D warnings -W clippy::pedantic` at the end of every task that touches that crate's production code. Run the full workspace gate (`make test` + `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic` + `cargo +nightly fmt -- --check`) only in the final task (Task 9), since this is a public core type change with workspace-wide blast radius.
- Task ordering follows crate dependency direction: `gitflow-core` (Task 1) has no dependency on the platform crates; `gitflow-gitlab` (Tasks 2-5), `gitflow-gitcode` (Tasks 6-7), and `gitflow-github` (Task 8) all depend on `gitflow-core` and will NOT compile once Task 1 lands until their own task fixes their fallback/wrap code. This is expected and is called out in each task's Step 2 so the implementer isn't surprised by a compile error where a runtime test failure was expected.

---

### Task 1: `gitflow-core` — make all 5 timestamp fields optional

**Files:**
- Modify: `crates/core/src/issue.rs:45-48` (struct field), no test changes needed (see Step 4)
- Modify: `crates/core/src/pr.rs:47-50` (struct field), no test changes needed
- Modify: `crates/core/src/review.rs:49-50` (struct field; do NOT touch `ReviewCommentData` at lines 56-76)
- Modify: `crates/core/src/pipeline.rs:62-65` (struct field) and its EXISTING test module at line 154 (add one new test — do not create a second `mod tests`)
- Modify: `crates/core/src/types.rs:146-147` (struct field) and its test module: `test_should_serialize_comment_data_to_camel_case_json` (line ~369-383), `test_should_roundtrip_comment_data_via_serde` (line ~385-402), `test_should_derive_debug_for_comment_data` (line ~404-418) — each directly constructs a `CommentData` literal with `created_at: "...".parse().expect("valid date")` and must wrap it in `Some(...)`

**Interfaces:**
- Produces: `IssueData.created_at: Option<DateTime<Utc>>`, `IssueData.updated_at: Option<DateTime<Utc>>`, `PrData.created_at: Option<DateTime<Utc>>`, `PrData.updated_at: Option<DateTime<Utc>>`, `ReviewData.submitted_at: Option<DateTime<Utc>>`, `CommentData.created_at: Option<DateTime<Utc>>`, `PipelineStatus.created_at: Option<DateTime<Utc>>`, `PipelineStatus.updated_at: Option<DateTime<Utc>>` (all were `DateTime<Utc>`) — every downstream crate (gitlab, gitcode, github) constructs or reads these fields and must be updated in later tasks to match.

- [ ] **Step 1: Write the failing tests — missing fields deserialize to `None`, not an error**

Add to `crates/core/src/types.rs`'s `#[cfg(test)] mod tests` block, near the other `CommentData` tests:

```rust
#[test]
fn test_should_deserialize_comment_with_missing_created_at_as_none() {
    let json = r#"{
        "id": 5,
        "body": "no timestamp",
        "author": {"login": "u", "id": "1"}
    }"#;
    let comment: CommentData = serde_json::from_str(json).expect("deserialize");
    assert!(
        comment.created_at.is_none(),
        "missing createdAt must deserialize to None, not error or a fabricated timestamp"
    );
}
```

Add to `crates/core/src/pipeline.rs`'s EXISTING `#[cfg(test)] mod tests` block (this module already exists at line 154, containing `test_should_serialize_pipeline_status_enum_to_snake_case` and others — add the new test as another `#[test]` fn inside that same block, do not create a second module):

```rust
    #[test]
    fn test_should_deserialize_pipeline_status_with_missing_timestamps_as_none() {
        let json = r#"{
            "id": 1,
            "refName": "main",
            "status": "success",
            "url": "https://example.com/pipelines/1"
        }"#;
        let status: PipelineStatus = serde_json::from_str(json).expect("deserialize");
        assert!(status.created_at.is_none());
        assert!(status.updated_at.is_none());
    }
```

- [ ] **Step 2: Run the new tests to verify they fail**

Run: `cargo test -p gitflow-core types::tests::test_should_deserialize_comment_with_missing_created_at_as_none`
Expected: FAIL — `created_at: DateTime<Utc>` is currently a required field, so `serde_json::from_str` returns an error and `.expect("deserialize")` panics.

Run: `cargo test -p gitflow-core pipeline::tests::test_should_deserialize_pipeline_status_with_missing_timestamps_as_none`
Expected: FAIL — same reason, `PipelineStatus.created_at`/`updated_at` are currently required fields.

- [ ] **Step 3: Change the 5 field types**

In `crates/core/src/issue.rs`, change:

```rust
    /// 创建时间（UTC）。
    pub created_at: DateTime<Utc>,
    /// 最近更新时间（UTC）。
    pub updated_at: DateTime<Utc>,
```

to:

```rust
    /// 创建时间（UTC）。API 未返回该字段时为 `None`——绝不用当前时间伪造。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    /// 最近更新时间（UTC）。API 未返回该字段时为 `None`——绝不用当前时间伪造。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
```

In `crates/core/src/pr.rs`, change the identical two lines the identical way (same doc comment wording).

In `crates/core/src/review.rs`, change only the `ReviewData` struct's:

```rust
    /// 提交时间（UTC）。
    pub submitted_at: DateTime<Utc>,
```

to:

```rust
    /// 提交时间（UTC）。API 未返回该字段时为 `None`——绝不用当前时间伪造。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submitted_at: Option<DateTime<Utc>>,
```

Do NOT touch `ReviewCommentData.created_at` (a few lines further down in the same file) — it is a different struct, out of scope per design §6.

In `crates/core/src/pipeline.rs`, change:

```rust
    /// 创建时间（UTC）。
    pub created_at: DateTime<Utc>,
    /// 最近更新时间（UTC）。
    pub updated_at: DateTime<Utc>,
```

to:

```rust
    /// 创建时间（UTC）。API 未返回该字段时为 `None`——绝不用当前时间伪造。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    /// 最近更新时间（UTC）。API 未返回该字段时为 `None`——绝不用当前时间伪造。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
```

In `crates/core/src/types.rs`, change:

```rust
    /// When the comment was created (UTC).
    pub created_at: DateTime<Utc>,
```

to:

```rust
    /// When the comment was created (UTC). `None` when the platform API omits
    /// it — never fabricated as the current time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
```

- [ ] **Step 4: Fix the 3 direct-construction tests in `crates/core/src/types.rs`**

Change `test_should_serialize_comment_data_to_camel_case_json`'s construction from:

```rust
            created_at: "2026-01-01T00:00:00Z".parse().expect("valid date"),
```

to:

```rust
            created_at: Some("2026-01-01T00:00:00Z".parse().expect("valid date")),
```

Change `test_should_roundtrip_comment_data_via_serde`'s construction from:

```rust
            created_at: "2026-03-15T10:00:00Z".parse().expect("valid date"),
```

to:

```rust
            created_at: Some("2026-03-15T10:00:00Z".parse().expect("valid date")),
```

Change `test_should_derive_debug_for_comment_data`'s construction from:

```rust
            created_at: "2026-01-01T00:00:00Z".parse().expect("valid date"),
```

to:

```rust
            created_at: Some("2026-01-01T00:00:00Z".parse().expect("valid date")),
```

No other test in `crates/core/src/{issue,pr,review}.rs` needs changes: `test_should_roundtrip_issue_data_via_serde` (issue.rs), `test_should_roundtrip_pr_data_via_serde` (pr.rs), and `test_should_roundtrip_review_data_via_serde` (review.rs) all compare a round-tripped value against the original via `assert_eq!` — both sides are the same `Option<DateTime<Utc>>` after this change, and `Option<T>: PartialEq` when `T: PartialEq`, so these keep passing unmodified. `test_should_serialize_review_comment_skips_null_fields` and the `ReviewCommentData` tests are untouched (different, out-of-scope struct).

- [ ] **Step 5: Run all core tests to verify they pass**

Run: `cargo test -p gitflow-core`
Expected: PASS — all tests in `issue.rs`, `pr.rs`, `review.rs`, `pipeline.rs`, `types.rs`, including the 2 new tests from Step 1 and the 3 fixed tests from Step 4.

- [ ] **Step 6: Run clippy**

Run: `cargo clippy -p gitflow-core --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: clean, no new warnings.

- [ ] **Step 7: Commit**

```bash
git add crates/core/src/issue.rs crates/core/src/pr.rs crates/core/src/review.rs crates/core/src/pipeline.rs crates/core/src/types.rs
git commit -m "refactor(core): make issue/pr/review/comment/pipeline timestamps optional

IssueData.created_at/updated_at, PrData.created_at/updated_at,
ReviewData.submitted_at, CommentData.created_at, and
PipelineStatus.created_at/updated_at were all DateTime<Utc>. GitLab
and GitCode adapters fall back to Utc::now() when the platform API
omits these fields, displaying a record with an unknown creation time
as \"created today\" — the same class of silent-data-fabrication bug
already fixed for ReleaseData.created_at in #366. Each field now
mirrors that fix and the existing JobData.started_at/completed_at
pattern: Option<DateTime<Utc>> with skip_serializing_if, so None omits
the field from JSON entirely instead of faking a timestamp.

Refs #380"
```

---

### Task 2: `gitflow-gitlab` — `issue.rs`: stop faking `IssueData`/`CommentData` timestamps

**Files:**
- Modify: `crates/gitlab/src/issue.rs:242-280` (`From<IssueApiResponse> for IssueData`)
- Modify: `crates/gitlab/src/issue.rs:294-310` (`From<CommentApiResponse> for CommentData`)
- Modify: `crates/gitlab/src/issue.rs:994-1013` (`test_should_handle_missing_author_with_fallback`)
- Test: same file

**Interfaces:**
- Consumes: `IssueData.created_at`/`updated_at: Option<DateTime<Utc>>`, `CommentData.created_at: Option<DateTime<Utc>>` from Task 1.
- `IssueApiResponse.created_at`/`updated_at` (this file, lines 233-235) and `CommentApiResponse.created_at` (line 291) are already `Option<DateTime<Utc>>` — no change needed to the intermediate structs.

- [ ] **Step 1: Update the existing test's assertions (RED)**

In `crates/gitlab/src/issue.rs`, change `test_should_handle_missing_author_with_fallback` from:

```rust
        let api: IssueApiResponse = serde_json::from_slice(json).expect("valid IssueApiResponse");
        let issue: IssueData = api.into();
        assert_eq!(issue.author.login, "unknown");
        assert_eq!(issue.author.id, "0");
    }
```

to:

```rust
        let api: IssueApiResponse = serde_json::from_slice(json).expect("valid IssueApiResponse");
        let issue: IssueData = api.into();
        assert_eq!(issue.author.login, "unknown");
        assert_eq!(issue.author.id, "0");
        assert!(
            issue.created_at.is_none(),
            "missing created_at must stay None, not fall back to Utc::now()"
        );
        assert!(
            issue.updated_at.is_none(),
            "missing updated_at must stay None, not fall back to Utc::now()"
        );
    }
```

Also add a new test for the comment conversion, right after `test_should_deserialize_comment_api_response`:

```rust
#[test]
fn test_should_keep_comment_created_at_none_when_api_omits_it() {
    let json = br#"{
        "id": 1002,
        "body": "No timestamp provided.",
        "author": {"username": "maintainer", "id": 42}
    }"#;

    let api: CommentApiResponse = serde_json::from_slice(json).expect("valid CommentApiResponse");
    let comment: CommentData = api.into();
    assert!(
        comment.created_at.is_none(),
        "missing created_at must stay None, not fall back to Utc::now()"
    );
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p gitflow-gitlab issue::tests::test_should_handle_missing_author_with_fallback issue::tests::test_should_keep_comment_created_at_none_when_api_omits_it`
Expected: neither compiles yet — Task 1 already changed `IssueData`/`CommentData`'s field types to `Option`, but this file's `From` impls still assign a bare `DateTime<Utc>` (`api.created_at.unwrap_or(now)`) to what is now an `Option<DateTime<Utc>>` field, so `cargo build -p gitflow-gitlab` itself fails with a type mismatch before any test can run. Confirm the compiler error names `created_at`/`updated_at` in `IssueData`/`CommentData` before proceeding to Step 3.

- [ ] **Step 3: Remove the fallbacks in both `From` impls**

Change:

```rust
impl From<IssueApiResponse> for IssueData {
    fn from(api: IssueApiResponse) -> Self {
        let now = Utc::now();
        let labels: Vec<Label> = api
            .labels
            .into_iter()
            .map(|name| Label {
                name,
                color: None,
                description: None,
            })
            .collect();
        let author = api.author.as_ref().map_or_else(
            || UserSummary {
                login: "unknown".into(),
                id: "0".to_string(),
            },
            UserSummary::from,
        );

        Self {
            number: api.iid,
            title: api.title,
            body: api.description,
            state: if api.state == "closed" {
                State::Closed
            } else {
                State::Open
            },
            labels,
            author,
            assignees: api.assignees.iter().map(UserSummary::from).collect(),
            created_at: api.created_at.unwrap_or(now),
            updated_at: api.updated_at.unwrap_or(now),
            url: api.web_url.unwrap_or_default(),
            milestone: api.milestone.map(Into::into),
        }
    }
}
```

to:

```rust
impl From<IssueApiResponse> for IssueData {
    fn from(api: IssueApiResponse) -> Self {
        let labels: Vec<Label> = api
            .labels
            .into_iter()
            .map(|name| Label {
                name,
                color: None,
                description: None,
            })
            .collect();
        let author = api.author.as_ref().map_or_else(
            || UserSummary {
                login: "unknown".into(),
                id: "0".to_string(),
            },
            UserSummary::from,
        );

        Self {
            number: api.iid,
            title: api.title,
            body: api.description,
            state: if api.state == "closed" {
                State::Closed
            } else {
                State::Open
            },
            labels,
            author,
            assignees: api.assignees.iter().map(UserSummary::from).collect(),
            created_at: api.created_at,
            updated_at: api.updated_at,
            url: api.web_url.unwrap_or_default(),
            milestone: api.milestone.map(Into::into),
        }
    }
}
```

(The local `let now = Utc::now();` binding is removed — it has no other use in this function.)

Change:

```rust
impl From<CommentApiResponse> for CommentData {
    fn from(api: CommentApiResponse) -> Self {
        let author = api.author.as_ref().map_or_else(
            || UserSummary {
                login: "unknown".into(),
                id: "0".to_string(),
            },
            UserSummary::from,
        );
        Self {
            id: api.id,
            body: api.body,
            author,
            created_at: api.created_at.unwrap_or_else(Utc::now),
        }
    }
}
```

to:

```rust
impl From<CommentApiResponse> for CommentData {
    fn from(api: CommentApiResponse) -> Self {
        let author = api.author.as_ref().map_or_else(
            || UserSummary {
                login: "unknown".into(),
                id: "0".to_string(),
            },
            UserSummary::from,
        );
        Self {
            id: api.id,
            body: api.body,
            author,
            created_at: api.created_at,
        }
    }
}
```

- [ ] **Step 4: Run all issue.rs tests to verify they pass**

Run: `cargo test -p gitflow-gitlab issue::`
Expected: PASS — all tests, including the 2 from Step 1 and every other existing test that provides a present `created_at`/`updated_at` (unaffected, since the value was present and is now wrapped in `Some` automatically by the already-`Option` intermediate field).

- [ ] **Step 5: Run clippy**

Run: `cargo clippy -p gitflow-gitlab --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: clean. (Removing the unused `now` binding avoids a new unused-variable warning — verify none appears.)

- [ ] **Step 6: Commit**

```bash
git add crates/gitlab/src/issue.rs
git commit -m "refactor(gitlab): stop faking issue/comment created_at with Utc::now()

Refs #380"
```

---

### Task 3: `gitflow-gitlab` — `mr.rs`: stop faking `PrData`/`CommentData` timestamps

**Files:**
- Modify: `crates/gitlab/src/mr.rs:295-327` (`From<MrApiResponse> for PrData`)
- Modify: `crates/gitlab/src/mr.rs:341-357` (`From<CommentApiResponse> for CommentData`)
- Modify: `crates/gitlab/src/mr.rs:890-909` (`test_should_handle_missing_author_with_fallback`)
- Test: same file

**Interfaces:**
- Consumes: `PrData.created_at`/`updated_at: Option<DateTime<Utc>>`, `CommentData.created_at: Option<DateTime<Utc>>` from Task 1. `PrData.merged_at` is untouched (already `Option`, out of scope).
- `MrApiResponse.created_at`/`updated_at` and `CommentApiResponse.created_at` (this file) are already `Option<DateTime<Utc>>` — no change needed to the intermediate structs.

- [ ] **Step 1: Update the existing test's assertions (RED)**

In `crates/gitlab/src/mr.rs`, change `test_should_handle_missing_author_with_fallback` from:

```rust
        let api: MrApiResponse = serde_json::from_slice(json).expect("valid MrApiResponse");
        let pr: PrData = api.into();
        assert_eq!(pr.author.login, "unknown");
    }
```

to:

```rust
        let api: MrApiResponse = serde_json::from_slice(json).expect("valid MrApiResponse");
        let pr: PrData = api.into();
        assert_eq!(pr.author.login, "unknown");
        assert!(
            pr.created_at.is_none(),
            "missing created_at must stay None, not fall back to Utc::now()"
        );
        assert!(
            pr.updated_at.is_none(),
            "missing updated_at must stay None, not fall back to Utc::now()"
        );
    }
```

Add a new test after the file's existing `CommentApiResponse` deserialization test (search for `fn test_should_deserialize_comment_api_response` in this file if present, otherwise place it directly after the `From<CommentApiResponse> for CommentData` impl's own test block):

```rust
#[test]
fn test_should_keep_mr_comment_created_at_none_when_api_omits_it() {
    let json = br#"{
        "id": 1003,
        "body": "No timestamp provided.",
        "author": {"username": "maintainer", "id": 42}
    }"#;

    let api: CommentApiResponse = serde_json::from_slice(json).expect("valid CommentApiResponse");
    let comment: CommentData = api.into();
    assert!(
        comment.created_at.is_none(),
        "missing created_at must stay None, not fall back to Utc::now()"
    );
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p gitflow-gitlab mr::tests::test_should_handle_missing_author_with_fallback mr::tests::test_should_keep_mr_comment_created_at_none_when_api_omits_it`
Expected: does not compile — same reason as Task 2 Step 2 (this file's `From` impls still assign bare `DateTime<Utc>` to now-`Option` fields).

- [ ] **Step 3: Remove the fallbacks in both `From` impls**

Change:

```rust
impl From<MrApiResponse> for PrData {
    fn from(api: MrApiResponse) -> Self {
        let now = Utc::now();
        let state = if api.state == "closed" || api.state == "merged" {
            State::Closed
        } else {
            State::Open
        };
        let author = api.author.as_ref().map_or_else(
            || UserSummary {
                login: "unknown".into(),
                id: "0".to_string(),
            },
            UserSummary::from,
        );

        Self {
            number: api.iid,
            title: api.title,
            body: api.description,
            state,
            draft: api.draft,
            author,
            base_branch: api.target_branch,
            head_branch: api.source_branch,
            created_at: api.created_at.unwrap_or(now),
            updated_at: api.updated_at.unwrap_or(now),
            merged_at: api.merged_at,
            url: api.web_url.unwrap_or_default(),
            milestone: api.milestone.map(Into::into),
        }
    }
}
```

to:

```rust
impl From<MrApiResponse> for PrData {
    fn from(api: MrApiResponse) -> Self {
        let state = if api.state == "closed" || api.state == "merged" {
            State::Closed
        } else {
            State::Open
        };
        let author = api.author.as_ref().map_or_else(
            || UserSummary {
                login: "unknown".into(),
                id: "0".to_string(),
            },
            UserSummary::from,
        );

        Self {
            number: api.iid,
            title: api.title,
            body: api.description,
            state,
            draft: api.draft,
            author,
            base_branch: api.target_branch,
            head_branch: api.source_branch,
            created_at: api.created_at,
            updated_at: api.updated_at,
            merged_at: api.merged_at,
            url: api.web_url.unwrap_or_default(),
            milestone: api.milestone.map(Into::into),
        }
    }
}
```

Change:

```rust
impl From<CommentApiResponse> for CommentData {
    fn from(api: CommentApiResponse) -> Self {
        let author = api.author.as_ref().map_or_else(
            || UserSummary {
                login: "unknown".into(),
                id: "0".to_string(),
            },
            UserSummary::from,
        );
        Self {
            id: api.id,
            body: api.body,
            author,
            created_at: api.created_at.unwrap_or_else(Utc::now),
        }
    }
}
```

to:

```rust
impl From<CommentApiResponse> for CommentData {
    fn from(api: CommentApiResponse) -> Self {
        let author = api.author.as_ref().map_or_else(
            || UserSummary {
                login: "unknown".into(),
                id: "0".to_string(),
            },
            UserSummary::from,
        );
        Self {
            id: api.id,
            body: api.body,
            author,
            created_at: api.created_at,
        }
    }
}
```

- [ ] **Step 4: Run all mr.rs tests to verify they pass**

Run: `cargo test -p gitflow-gitlab mr::`
Expected: PASS.

- [ ] **Step 5: Run clippy**

Run: `cargo clippy -p gitflow-gitlab --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: clean.

- [ ] **Step 6: Commit**

```bash
git add crates/gitlab/src/mr.rs
git commit -m "refactor(gitlab): stop faking mr/comment created_at with Utc::now()

Refs #380"
```

---

### Task 4: `gitflow-gitlab` — `review.rs`: stop faking `ReviewData.submitted_at` (2 sites), document the legitimate one

**Files:**
- Modify: `crates/gitlab/src/review.rs:141-148` (`request_changes()`)
- Modify: `crates/gitlab/src/review.rs:206-215` (`submit_review()`'s underlying comment path — actually the `comment()` method at lines 131-148 per current line numbers; see exact code below)
- Modify: `crates/gitlab/src/review.rs:180-189` (`approve()`) — COMMENT ONLY, no logic change
- Modify: `crates/gitlab/src/review.rs:358-385` (`test_should_convert_note_to_review_data`)
- Test: same file

**Interfaces:**
- Consumes: `ReviewData.submitted_at: Option<DateTime<Utc>>` from Task 1.
- `NoteApiResponse.created_at` (this file, line 97) is already `Option<DateTime<Utc>>` — no change needed to the intermediate struct.

- [ ] **Step 1: Fix the existing test that directly constructs `ReviewData` (RED)**

In `crates/gitlab/src/review.rs`, `test_should_convert_note_to_review_data` currently ends with:

```rust
        let review = ReviewData {
            id: note.id,
            state: ReviewState::Commented,
            body: Some(note.body),
            author,
            submitted_at: note.created_at.expect("has date"),
        };

        assert_eq!(review.id, 100);
        assert_eq!(review.state, ReviewState::Commented);
        assert_eq!(review.author.login, "reviewer");
    }
```

Change the construction to drop `.expect("has date")` (both sides are now `Option<DateTime<Utc>>`, so no unwrap is needed) and add an assertion:

```rust
        let review = ReviewData {
            id: note.id,
            state: ReviewState::Commented,
            body: Some(note.body),
            author,
            submitted_at: note.created_at,
        };

        assert_eq!(review.id, 100);
        assert_eq!(review.state, ReviewState::Commented);
        assert_eq!(review.author.login, "reviewer");
        assert!(review.submitted_at.is_some());
    }
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p gitflow-gitlab review::tests::test_should_convert_note_to_review_data`
Expected: does not compile yet — `submitted_at: note.created_at.expect("has date")` still returns a bare `DateTime<Utc>`, but Task 1 already made `ReviewData.submitted_at: Option<DateTime<Utc>>`, so even the OLD code (before this step's edit) already fails to compile the moment Task 1 lands. This step's edit is what fixes the type; run the test only after making the Step 1 edit to confirm it passes, or run it beforehand to confirm the type-mismatch compiler error names `submitted_at` if you want to see the RED state explicitly.

- [ ] **Step 3: Remove the fallbacks in `request_changes()` and `comment()`**

`crates/gitlab/src/review.rs` has two call sites using `note.created_at.unwrap_or_else(Utc::now)`. Change the `comment()` method from:

```rust
    async fn comment(&self, pr_number: u64, body: &str) -> Result<ReviewData> {
        let note = self.post_note(pr_number, body).await?;
        let author = note.author.as_ref().map_or_else(
            || UserSummary {
                login: "unknown".into(),
                id: "0".to_string(),
            },
            UserSummary::from,
        );

        Ok(ReviewData {
            id: note.id,
            state: ReviewState::Commented,
            body: Some(note.body),
            author,
            submitted_at: note.created_at.unwrap_or_else(Utc::now),
        })
    }
```

to:

```rust
    async fn comment(&self, pr_number: u64, body: &str) -> Result<ReviewData> {
        let note = self.post_note(pr_number, body).await?;
        let author = note.author.as_ref().map_or_else(
            || UserSummary {
                login: "unknown".into(),
                id: "0".to_string(),
            },
            UserSummary::from,
        );

        Ok(ReviewData {
            id: note.id,
            state: ReviewState::Commented,
            body: Some(note.body),
            author,
            submitted_at: note.created_at,
        })
    }
```

Change `request_changes()` from:

```rust
    async fn request_changes(&self, pr_number: u64, body: &str) -> Result<ReviewData> {
        let changes_body = format!("Changes requested:\n\n{body}");
        let note = self.post_note(pr_number, &changes_body).await?;
        let author = note.author.as_ref().map_or_else(
            || UserSummary {
                login: "unknown".into(),
                id: "0".to_string(),
            },
            UserSummary::from,
        );

        Ok(ReviewData {
            id: note.id,
            state: ReviewState::ChangesRequested,
            body: Some(note.body),
            author,
            submitted_at: note.created_at.unwrap_or_else(Utc::now),
        })
    }
```

to:

```rust
    async fn request_changes(&self, pr_number: u64, body: &str) -> Result<ReviewData> {
        let changes_body = format!("Changes requested:\n\n{body}");
        let note = self.post_note(pr_number, &changes_body).await?;
        let author = note.author.as_ref().map_or_else(
            || UserSummary {
                login: "unknown".into(),
                id: "0".to_string(),
            },
            UserSummary::from,
        );

        Ok(ReviewData {
            id: note.id,
            state: ReviewState::ChangesRequested,
            body: Some(note.body),
            author,
            submitted_at: note.created_at,
        })
    }
```

- [ ] **Step 4: Add the explanatory comment to `approve()` (Category B, no logic change)**

Change:

```rust
        Ok(ReviewData {
            id: 0,
            state: ReviewState::Approved,
            body: if message.is_empty() {
                body.map(String::from)
            } else {
                Some(message)
            },
            author,
            submitted_at: Utc::now(),
        })
```

to:

```rust
        Ok(ReviewData {
            id: 0,
            state: ReviewState::Approved,
            body: if message.is_empty() {
                body.map(String::from)
            } else {
                Some(message)
            },
            author,
            // `glab mr approve` returns no timestamp of its own, but approval
            // is a synchronous action happening at this exact instant — this
            // is NOT backfilling an unknown historical value (the #380/#366
            // bug class), so Utc::now() is correct here and intentionally
            // stays a plain value, not Option (#380 design §3.3).
            submitted_at: Some(Utc::now()),
        })
```

Note: `submitted_at` is now `Option<DateTime<Utc>>` (from Task 1), so even this legitimate `Utc::now()` call must be wrapped in `Some(...)` to compile — only the *value* stays `Utc::now()` unconditionally; wrapping it in `Some` is the same type-compatibility requirement as everywhere else, not a change to this site's semantics.

- [ ] **Step 5: Run all review.rs tests to verify they pass**

Run: `cargo test -p gitflow-gitlab review::`
Expected: PASS — including `test_should_approve_mr_without_repo_flag` and `test_should_approve_nested_group_mr_without_repo_flag`, which check `review.state`/`review.author.login` but not `submitted_at`, so they are unaffected by the `Some(...)` wrap.

- [ ] **Step 6: Run clippy**

Run: `cargo clippy -p gitflow-gitlab --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: clean.

- [ ] **Step 7: Commit**

```bash
git add crates/gitlab/src/review.rs
git commit -m "refactor(gitlab): stop faking review submitted_at with Utc::now()

comment() and request_changes() both fell back to Utc::now() when the
underlying MR note lacked a created_at. approve()'s own Utc::now() is
left unchanged with an explanatory comment: it represents the approval
action happening synchronously right now, not a fabricated historical
timestamp — a different situation from the bug being fixed here.

Refs #380"
```

---

### Task 5: `gitflow-gitlab` — `pipeline.rs`: stop faking `PipelineStatus` timestamps, adjust `report()`'s aggregate logic

**Files:**
- Modify: `crates/gitlab/src/pipeline.rs:171-194` (`From<PipelineApiResponse> for PipelineStatus`)
- Modify: `crates/gitlab/src/pipeline.rs:335-419` (`report()` — cutoff filter and duration calculation)
- Test: same file

**Interfaces:**
- Consumes: `PipelineStatus.created_at`/`updated_at: Option<DateTime<Utc>>` from Task 1.
- `PipelineApiResponse.created_at`/`updated_at` (this file, lines 142-144) are already `Option<DateTime<Utc>>` — no change needed to the intermediate struct.

- [ ] **Step 1: Write the failing tests (RED)**

Add to `crates/gitlab/src/pipeline.rs`'s `#[cfg(test)] mod tests` block, after `test_should_convert_pipeline_to_status`:

```rust
#[test]
fn test_should_keep_pipeline_timestamps_none_when_api_omits_them() {
    let api = PipelineApiResponse {
        id: 101,
        ref_name: Some("main".into()),
        git_ref: None,
        status: "success".into(),
        created_at: None,
        updated_at: None,
        web_url: None,
        sha: None,
    };

    let status: PipelineStatus = api.into();
    assert!(
        status.created_at.is_none(),
        "missing created_at must stay None, not fall back to Utc::now()"
    );
    assert!(
        status.updated_at.is_none(),
        "missing updated_at must stay None, not fall back to Utc::now()"
    );
}
```

Add two more tests, after `test_should_zero_report_when_all_pipelines_are_running`, exercising the design §3.2 "exclude, don't guess" aggregate semantics:

```rust
#[tokio::test]
async fn test_should_exclude_pipelines_with_missing_created_at_from_report_cutoff() {
    // One pipeline has a real, in-window created_at; the other omits it
    // entirely. The one with no created_at must not count toward
    // total_runs — we cannot know whether it falls in the requested
    // window, so we exclude it rather than guess.
    let now = Utc::now();
    let ts = |offset_secs: i64| (now - chrono::Duration::seconds(offset_secs)).to_rfc3339();

    let json = format!(
        r#"[
            {{"id": 1, "ref_name": "main", "status": "success", "created_at": "{}", "updated_at": "{}"}},
            {{"id": 2, "ref_name": "main", "status": "success"}}
        ]"#,
        ts(600),
        ts(300),
    );

    let runner = MockCommandRunner::success(&json);
    let provider = GitLabPipelineProvider::with_runner("owner/repo", runner);

    let report = provider
        .report("main", 7)
        .await
        .expect("report should succeed");

    assert_eq!(
        report.total_runs, 1,
        "pipeline #2 has no created_at and must be excluded from the report, not counted as in-window"
    );
}

#[tokio::test]
async fn test_should_exclude_pipeline_with_missing_timestamp_from_duration_average() {
    // Pipeline #1 has both timestamps present (contributes a real duration
    // sample). Pipeline #2 has created_at but no updated_at — it must be
    // excluded from the average, not treated as a 0-second duration.
    let now = Utc::now();
    let ts = |offset_secs: i64| (now - chrono::Duration::seconds(offset_secs)).to_rfc3339();

    let json = format!(
        r#"[
            {{"id": 1, "ref_name": "main", "status": "success", "created_at": "{}", "updated_at": "{}"}},
            {{"id": 2, "ref_name": "main", "status": "success", "created_at": "{}"}}
        ]"#,
        ts(600),
        ts(300),
        ts(500),
    );

    let runner = MockCommandRunner::success(&json);
    let provider = GitLabPipelineProvider::with_runner("owner/repo", runner);

    let report = provider
        .report("main", 7)
        .await
        .expect("report should succeed");

    // Only pipeline #1 contributes a duration sample (300s); pipeline #2's
    // missing updated_at must not be treated as 0, which would drag the
    // average down incorrectly.
    assert!(
        (report.avg_duration_secs - 300.0).abs() < 1.0,
        "expected ~300s average from the one complete sample, got {}",
        report.avg_duration_secs
    );
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p gitflow-gitlab pipeline::tests::test_should_keep_pipeline_timestamps_none_when_api_omits_them`
Expected: does not compile — `From<PipelineApiResponse> for PipelineStatus` still assigns bare `DateTime<Utc>` via `.unwrap_or_else(Utc::now)` to what is now an `Option<DateTime<Utc>>` field.

Run: `cargo test -p gitflow-gitlab pipeline::tests::test_should_exclude_pipelines_with_missing_created_at_from_report_cutoff pipeline::tests::test_should_exclude_pipeline_with_missing_timestamp_from_duration_average`
Expected: also does not compile yet (same crate, same compile error blocks all tests in the crate) — fix Step 3 before re-running.

- [ ] **Step 3: Remove the fallback in the `From` impl**

Change:

```rust
impl From<PipelineApiResponse> for PipelineStatus {
    fn from(api: PipelineApiResponse) -> Self {
        let status_enum = parse_pipeline_status(&api.status);
        let conclusion = if api.status == "success" {
            Some("success".into())
        } else if api.status == "failed" {
            Some("failure".into())
        } else if api.status == "canceled" || api.status == "cancelled" {
            Some("cancelled".into())
        } else {
            None
        };

        Self {
            id: api.id,
            ref_name: api.effective_ref(),
            status: status_enum,
            conclusion,
            created_at: api.created_at.unwrap_or_else(Utc::now),
            updated_at: api.updated_at.unwrap_or_else(Utc::now),
            url: api.web_url.unwrap_or_default(),
        }
    }
}
```

to:

```rust
impl From<PipelineApiResponse> for PipelineStatus {
    fn from(api: PipelineApiResponse) -> Self {
        let status_enum = parse_pipeline_status(&api.status);
        let conclusion = if api.status == "success" {
            Some("success".into())
        } else if api.status == "failed" {
            Some("failure".into())
        } else if api.status == "canceled" || api.status == "cancelled" {
            Some("cancelled".into())
        } else {
            None
        };

        Self {
            id: api.id,
            ref_name: api.effective_ref(),
            status: status_enum,
            conclusion,
            created_at: api.created_at,
            updated_at: api.updated_at,
            url: api.web_url.unwrap_or_default(),
        }
    }
}
```

- [ ] **Step 4: Adjust `report()`'s cutoff filter and duration calculation**

Change:

```rust
        // Filter by date range
        let cutoff = Utc::now() - chrono::Duration::days(i64::from(days));
        let recent: Vec<&PipelineStatus> = pipelines
            .iter()
            .filter(|p| p.created_at >= cutoff)
            .collect();
```

to:

```rust
        // Filter by date range. A pipeline with no created_at has an unknown
        // creation time and must be excluded, not guessed into or out of the
        // window (#380 design §3.2).
        let cutoff = Utc::now() - chrono::Duration::days(i64::from(days));
        let recent: Vec<&PipelineStatus> = pipelines
            .iter()
            .filter(|p| p.created_at.is_some_and(|c| c >= cutoff))
            .collect();
```

Change:

```rust
        // Calculate average duration
        #[allow(
            clippy::cast_precision_loss,
            reason = "Duration values never exceed f64 precision"
        )]
        let durations: Vec<f64> = recent
            .iter()
            .map(|p| (p.updated_at - p.created_at).num_seconds().max(0) as f64)
            .filter(|d| *d > 0.0)
            .collect();
```

to:

```rust
        // Calculate average duration. A pipeline missing either timestamp
        // cannot have its duration computed and must be excluded from the
        // sample set entirely, not treated as a 0-second run (#380 design
        // §3.2).
        #[allow(
            clippy::cast_precision_loss,
            reason = "Duration values never exceed f64 precision"
        )]
        let durations: Vec<f64> = recent
            .iter()
            .filter_map(|p| {
                let created = p.created_at?;
                let updated = p.updated_at?;
                Some((updated - created).num_seconds().max(0) as f64)
            })
            .filter(|d| *d > 0.0)
            .collect();
```

- [ ] **Step 5: Run all pipeline.rs tests to verify they pass**

Run: `cargo test -p gitflow-gitlab pipeline::`
Expected: PASS — all tests including the 3 new ones and the pre-existing `test_should_exclude_non_terminal_pipelines_from_report_total_runs`/`test_should_zero_report_when_all_pipelines_are_running` (both use always-present timestamps, so `Some(c) >= cutoff` behaves identically to the old bare comparison and these keep passing unmodified).

- [ ] **Step 6: Run clippy**

Run: `cargo clippy -p gitflow-gitlab --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: clean.

- [ ] **Step 7: Commit**

```bash
git add crates/gitlab/src/pipeline.rs
git commit -m "refactor(gitlab): stop faking pipeline created_at/updated_at with Utc::now()

Also updates report()'s cutoff filter and average-duration calculation
to exclude pipelines with an unknown timestamp instead of guessing
they fall in-window or have a 0-second duration.

Refs #380"
```

---

### Task 6: `gitflow-gitcode` — `issue.rs`: stop faking `IssueData`/`CommentData` timestamps

**Files:**
- Modify: `crates/gitcode/src/issue.rs:85-134` (`From<IssueApiResponse> for IssueData`)
- Modify: `crates/gitcode/src/issue.rs:172-197` (`From<CommentApiResponse> for CommentData`)
- Modify: `crates/gitcode/src/issue.rs:954-969` (`test_should_roundtrip_comment_data_via_serde`)
- Modify: `crates/gitcode/src/issue.rs:1101` (assertion inside `test_should_close_then_view_to_get_full_issue_with_milestone`)
- Test: same file

**Interfaces:**
- Consumes: `IssueData.created_at`/`updated_at: Option<DateTime<Utc>>`, `CommentData.created_at: Option<DateTime<Utc>>` from Task 1.
- `IssueApiResponse.created_at`/`updated_at` and `CommentApiResponse.created_at` (this file) are `Option<String>` (raw, unparsed) — these stay `Option<String>`; only the parsing/fallback logic inside the `From` impls changes.

**Important correction to this Issue's own history:** `From<IssueApiResponse> for IssueData` in this file was NOT fixed by #395 (that fix only removed a dead `From<CloseApiResponse> for IssueData` impl used by the old close/reopen path, which now delegates to `view()` — but `view()` itself goes through this same `From<IssueApiResponse>` impl, which still has the `Utc::now()` fallback today). Treat it as a live, unfixed site.

- [ ] **Step 1: Write the failing tests (RED)**

Add to `crates/gitcode/src/issue.rs`'s `#[cfg(test)] mod tests` block, after `test_should_handle_missing_author_with_fallback` if one exists in this file, otherwise directly after `test_should_deserialize_issue_data_from_gc_output`:

```rust
#[test]
fn test_should_keep_issue_created_at_none_when_gc_api_omits_it() {
    let gc_json = br#"{
        "number": 15,
        "title": "No timestamps",
        "body": null,
        "state": "open",
        "labels": [],
        "author": {"login": "dev", "id": "5"},
        "assignees": [],
        "html_url": "https://gitcode.com/octocat/hello-world/issues/15"
    }"#;

    let api: IssueApiResponse = serde_json::from_slice(gc_json).expect("valid IssueApiResponse");
    let issue: IssueData = api.into();
    assert!(
        issue.created_at.is_none(),
        "missing created_at must stay None, not fall back to Utc::now()"
    );
    assert!(
        issue.updated_at.is_none(),
        "missing updated_at must stay None, not fall back to Utc::now()"
    );
}

#[test]
fn test_should_keep_comment_created_at_none_when_gc_api_omits_it() {
    let gc_json = br#"{
        "id": "1002",
        "body": "No timestamp provided.",
        "author": "maintainer"
    }"#;

    let api: CommentApiResponse =
        serde_json::from_slice(gc_json).expect("valid CommentApiResponse");
    let comment = CommentData::from(api);
    assert!(
        comment.created_at.is_none(),
        "missing created_at must stay None, not fall back to Utc::now()"
    );
}
```

- [ ] **Step 2: Update the two exact-value assertions that will otherwise fail to compile**

Change `test_should_roundtrip_comment_data_via_serde` from:

```rust
    fn test_should_roundtrip_comment_data_via_serde() {
        let comment = CommentData {
            id: 77,
            body: "reviewed".into(),
            author: UserSummary {
                login: "alice".into(),
                id: "3".to_string(),
            },
            created_at: "2026-05-01T00:00:00Z".parse().expect("valid date"),
        };
```

to:

```rust
    fn test_should_roundtrip_comment_data_via_serde() {
        let comment = CommentData {
            id: 77,
            body: "reviewed".into(),
            author: UserSummary {
                login: "alice".into(),
                id: "3".to_string(),
            },
            created_at: Some("2026-05-01T00:00:00Z".parse().expect("valid date")),
        };
```

Change `test_should_close_then_view_to_get_full_issue_with_milestone`'s assertion from:

```rust
        assert_eq!(issue.created_at.to_rfc3339(), "2026-01-01T00:00:00+00:00");
```

to:

```rust
        assert_eq!(
            issue.created_at.map(|dt| dt.to_rfc3339()),
            Some("2026-01-01T00:00:00+00:00".to_string())
        );
```

- [ ] **Step 3: Run the new/changed tests to verify they fail (RED)**

Run: `cargo test -p gitflow-gitcode issue::tests::test_should_keep_issue_created_at_none_when_gc_api_omits_it issue::tests::test_should_keep_comment_created_at_none_when_gc_api_omits_it`
Expected: does not compile — both `From` impls in this file still call `.unwrap_or_else(Utc::now)`/`map_or_else(Utc::now, ...)` producing a bare `DateTime<Utc>`, which no longer fits the now-`Option` field.

- [ ] **Step 4: Remove the fallback in `From<IssueApiResponse> for IssueData`**

Change:

```rust
impl From<IssueApiResponse> for IssueData {
    fn from(api: IssueApiResponse) -> Self {
        Self {
            number: api.number.parse().unwrap_or(0),
            title: api.title,
            body: api.body,
            state: match api.state.as_str() {
                "closed" => State::Closed,
                _ => State::Open,
            },
            labels: api
                .labels
                .unwrap_or_default()
                .into_iter()
                .map(Label::from)
                .collect(),
            author: api.user.map_or(
                UserSummary {
                    login: "unknown".into(),
                    id: String::new(),
                },
                UserSummary::from,
            ),
            assignees: api
                .assignees
                .unwrap_or_default()
                .into_iter()
                .map(UserSummary::from)
                .collect(),
            created_at: api
                .created_at
                .and_then(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .ok()
                        .map(|d| d.with_timezone(&Utc))
                })
                .unwrap_or_else(Utc::now),
            updated_at: api
                .updated_at
                .and_then(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .ok()
                        .map(|d| d.with_timezone(&Utc))
                })
                .unwrap_or_else(Utc::now),
            url: api.html_url,
            milestone: api.milestone.map(Into::into),
        }
    }
}
```

to:

```rust
impl From<IssueApiResponse> for IssueData {
    fn from(api: IssueApiResponse) -> Self {
        Self {
            number: api.number.parse().unwrap_or(0),
            title: api.title,
            body: api.body,
            state: match api.state.as_str() {
                "closed" => State::Closed,
                _ => State::Open,
            },
            labels: api
                .labels
                .unwrap_or_default()
                .into_iter()
                .map(Label::from)
                .collect(),
            author: api.user.map_or(
                UserSummary {
                    login: "unknown".into(),
                    id: String::new(),
                },
                UserSummary::from,
            ),
            assignees: api
                .assignees
                .unwrap_or_default()
                .into_iter()
                .map(UserSummary::from)
                .collect(),
            created_at: api.created_at.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|d| d.with_timezone(&Utc))
            }),
            updated_at: api.updated_at.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|d| d.with_timezone(&Utc))
            }),
            url: api.html_url,
            milestone: api.milestone.map(Into::into),
        }
    }
}
```

- [ ] **Step 5: Remove the fallback in `From<CommentApiResponse> for CommentData`**

Change:

```rust
impl From<CommentApiResponse> for CommentData {
    fn from(api: CommentApiResponse) -> Self {
        let author = api.user.map_or_else(
            || UserSummary {
                login: api.author.unwrap_or_else(|| "unknown".into()),
                id: String::new(),
            },
            UserSummary::from,
        );
        let created_at = api.created_at.as_deref().map_or_else(Utc::now, |s| {
            DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.with_timezone(&Utc))
                .or_else(|_| {
                    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                        .map(|ndt| ndt.and_utc())
                })
                .unwrap_or_else(|_| Utc::now())
        });
        Self {
            id: api.id,
            body: api.body,
            author,
            created_at,
        }
    }
}
```

to:

```rust
impl From<CommentApiResponse> for CommentData {
    fn from(api: CommentApiResponse) -> Self {
        let author = api.user.map_or_else(
            || UserSummary {
                login: api.author.unwrap_or_else(|| "unknown".into()),
                id: String::new(),
            },
            UserSummary::from,
        );
        let created_at = api.created_at.as_deref().and_then(|s| {
            DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.with_timezone(&Utc))
                .or_else(|_| {
                    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                        .map(|ndt| ndt.and_utc())
                })
                .ok()
        });
        Self {
            id: api.id,
            body: api.body,
            author,
            created_at,
        }
    }
}
```

(`map_or_else(Utc::now, |s| ... .unwrap_or_else(|_| Utc::now()))` becomes `and_then(|s| ... .ok())`: a missing `created_at` string short-circuits to `None` via `and_then`, and a present-but-unparsable string now yields `None` via `.ok()` instead of a fabricated `Utc::now()` — both the missing-field and parse-failure cases inside this one field now correctly produce `None`.)

- [ ] **Step 6: Run all issue.rs tests to verify they pass**

Run: `cargo test -p gitflow-gitcode issue::`
Expected: PASS — all tests, including the 2 new ones and the 2 fixed exact-value assertions.

- [ ] **Step 7: Run clippy**

Run: `cargo clippy -p gitflow-gitcode --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: clean.

- [ ] **Step 8: Commit**

```bash
git add crates/gitcode/src/issue.rs
git commit -m "refactor(gitcode): stop faking issue/comment created_at with Utc::now()

From<IssueApiResponse> for IssueData was not actually fixed by #395
(that change only removed a dead From<CloseApiResponse> impl; view(),
which close()/reopen() now delegate to, goes through this same
IssueApiResponse conversion and still had the Utc::now() fallback).

Refs #380"
```

---

### Task 7: `gitflow-gitcode` — `pr.rs`: stop faking `PrData`/`CommentData` timestamps

**Files:**
- Modify: `crates/gitcode/src/pr.rs:156-194` (`From<PrApiResponse> for PrData`, the `parse_time`/`parse_opt_time` closures)
- Modify: `crates/gitcode/src/pr.rs:125-153` (`From<PrCommentApiResponse> for CommentData`)
- Modify: `crates/gitcode/src/pr.rs:872-891` (`test_should_map_real_gitcode_pr_response_to_pr_data`)
- Modify: `crates/gitcode/src/pr.rs:893-915` (`test_should_map_open_pr_with_minimal_gitcode_fields`)
- Modify: `crates/gitcode/src/pr.rs:1776-1783` (`test_should_parse_comment_with_legacy_string_author`)
- Test: same file

**Interfaces:**
- Consumes: `PrData.created_at`/`updated_at: Option<DateTime<Utc>>`, `CommentData.created_at: Option<DateTime<Utc>>` from Task 1. `PrData.merged_at` is untouched (already correctly `Option` via the existing `parse_opt_time` closure — see Step 3, this closure is the template being generalized).

- [ ] **Step 1: Add a missing-timestamp assertion to the existing minimal-fields test (RED)**

In `crates/gitcode/src/pr.rs`, change `test_should_map_open_pr_with_minimal_gitcode_fields` from:

```rust
        let api: PrApiResponse = serde_json::from_str(json).expect(r"minimal gitcode PR JSON");
        let pr: PrData = api.into();

        assert_eq!(pr.state, State::Open);
        assert!(pr.draft);
        assert_eq!(pr.body, None);
        assert_eq!(pr.head_branch, "feature/x");
        assert_eq!(pr.base_branch, "main");
        assert_eq!(pr.author.login, "dev");
    }
```

to:

```rust
        let api: PrApiResponse = serde_json::from_str(json).expect(r"minimal gitcode PR JSON");
        let pr: PrData = api.into();

        assert_eq!(pr.state, State::Open);
        assert!(pr.draft);
        assert_eq!(pr.body, None);
        assert_eq!(pr.head_branch, "feature/x");
        assert_eq!(pr.base_branch, "main");
        assert_eq!(pr.author.login, "dev");
        assert!(
            pr.created_at.is_none(),
            "missing created_at must stay None, not fall back to Utc::now()"
        );
        assert!(
            pr.updated_at.is_none(),
            "missing updated_at must stay None, not fall back to Utc::now()"
        );
    }
```

Add a new test after `test_should_deserialize_comment_data_from_gc_pr_comment_output`:

```rust
#[test]
fn test_should_keep_pr_comment_created_at_none_when_gc_api_omits_it() {
    let gc_json = br#"{
        "id": 2003,
        "body": "No timestamp provided.",
        "author": {"login": "reviewer", "id": "88"}
    }"#;

    let api: PrCommentApiResponse =
        serde_json::from_slice(gc_json).expect("valid PrCommentApiResponse");
    let comment = CommentData::from(api);
    assert!(
        comment.created_at.is_none(),
        "missing created_at must stay None, not fall back to Utc::now()"
    );
}
```

- [ ] **Step 2: Update the two exact-value assertions that will otherwise fail to compile**

Change `test_should_map_real_gitcode_pr_response_to_pr_data`'s final assertion from:

```rust
        assert_eq!(pr.created_at.to_rfc3339(), "2026-07-30T04:40:46+00:00");
```

to:

```rust
        assert_eq!(
            pr.created_at.map(|dt| dt.to_rfc3339()),
            Some("2026-07-30T04:40:46+00:00".to_string())
        );
```

Change `test_should_parse_comment_with_legacy_string_author`'s assertion from:

```rust
        assert_eq!(comment.created_at.to_rfc3339(), "2026-07-07T10:40:20+00:00");
```

to:

```rust
        assert_eq!(
            comment.created_at.map(|dt| dt.to_rfc3339()),
            Some("2026-07-07T10:40:20+00:00".to_string())
        );
```

- [ ] **Step 3: Run the tests to verify they fail (RED)**

Run: `cargo test -p gitflow-gitcode pr::tests::test_should_map_open_pr_with_minimal_gitcode_fields pr::tests::test_should_keep_pr_comment_created_at_none_when_gc_api_omits_it`
Expected: does not compile — `parse_time` and the comment conversion's inline fallback both still produce bare `DateTime<Utc>`, no longer assignable to the now-`Option` fields.

- [ ] **Step 4: Collapse `parse_time`/`parse_opt_time` into one `Option`-returning closure**

Change:

```rust
impl From<PrApiResponse> for PrData {
    fn from(api: PrApiResponse) -> Self {
        let parse_time = |s: Option<String>| {
            s.and_then(|v| DateTime::parse_from_rfc3339(&v).ok())
                .map_or_else(Utc::now, |dt| dt.with_timezone(&Utc))
        };
        // 不能复用 parse_time：缺失会被填成「现在」，那等于谎报一次合并。
        let parse_opt_time = |s: Option<String>| {
            s.and_then(|v| DateTime::parse_from_rfc3339(&v).ok())
                .map(|dt| dt.with_timezone(&Utc))
        };
        Self {
            number: api.number,
            title: api.title,
            body: api.body,
            state: match api.state.as_deref() {
                Some("closed" | "merged") => State::Closed,
                _ => State::Open,
            },
            draft: api.draft,
            author: api.user.map_or(
                UserSummary {
                    login: "unknown".into(),
                    id: String::new(),
                },
                |u| UserSummary {
                    login: u.login,
                    id: u.id.unwrap_or_default(),
                },
            ),
            base_branch: api.base.map_or_else(String::new, |b| b.branch_ref),
            head_branch: api.head.map_or_else(String::new, |h| h.branch_ref),
            created_at: parse_time(api.created_at),
            updated_at: parse_time(api.updated_at),
            merged_at: parse_opt_time(api.merged_at),
            url: api.html_url.unwrap_or_default(),
            milestone: api.milestone.map(Into::into),
        }
    }
}
```

to:

```rust
impl From<PrApiResponse> for PrData {
    fn from(api: PrApiResponse) -> Self {
        // A missing timestamp stays None — never filled in as "now", which
        // would misreport an unknown creation/update/merge time (#380).
        let parse_time = |s: Option<String>| {
            s.and_then(|v| DateTime::parse_from_rfc3339(&v).ok())
                .map(|dt| dt.with_timezone(&Utc))
        };
        Self {
            number: api.number,
            title: api.title,
            body: api.body,
            state: match api.state.as_deref() {
                Some("closed" | "merged") => State::Closed,
                _ => State::Open,
            },
            draft: api.draft,
            author: api.user.map_or(
                UserSummary {
                    login: "unknown".into(),
                    id: String::new(),
                },
                |u| UserSummary {
                    login: u.login,
                    id: u.id.unwrap_or_default(),
                },
            ),
            base_branch: api.base.map_or_else(String::new, |b| b.branch_ref),
            head_branch: api.head.map_or_else(String::new, |h| h.branch_ref),
            created_at: parse_time(api.created_at),
            updated_at: parse_time(api.updated_at),
            merged_at: parse_time(api.merged_at),
            url: api.html_url.unwrap_or_default(),
            milestone: api.milestone.map(Into::into),
        }
    }
}
```

(The two closures are now identical, so `parse_opt_time` is deleted and every call site — including `merged_at`, which was already correct — uses the single renamed `parse_time`.)

- [ ] **Step 5: Remove the fallback in `From<PrCommentApiResponse> for CommentData`**

Change:

```rust
impl From<PrCommentApiResponse> for CommentData {
    fn from(api: PrCommentApiResponse) -> Self {
        let author = api.user.map_or_else(
            || UserSummary {
                login: api.author.unwrap_or_else(|| "unknown".into()),
                id: String::new(),
            },
            |u| UserSummary {
                login: u.login,
                id: u.id.unwrap_or_default(),
            },
        );
        let created_at = api.created_at.as_deref().map_or_else(Utc::now, |s| {
            DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.with_timezone(&Utc))
                .or_else(|_| {
                    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                        .map(|ndt| ndt.and_utc())
                })
                .unwrap_or_else(|_| Utc::now())
        });
        Self {
            id: api.id,
            body: api.body,
            author,
            created_at,
        }
    }
}
```

to:

```rust
impl From<PrCommentApiResponse> for CommentData {
    fn from(api: PrCommentApiResponse) -> Self {
        let author = api.user.map_or_else(
            || UserSummary {
                login: api.author.unwrap_or_else(|| "unknown".into()),
                id: String::new(),
            },
            |u| UserSummary {
                login: u.login,
                id: u.id.unwrap_or_default(),
            },
        );
        let created_at = api.created_at.as_deref().and_then(|s| {
            DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.with_timezone(&Utc))
                .or_else(|_| {
                    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                        .map(|ndt| ndt.and_utc())
                })
                .ok()
        });
        Self {
            id: api.id,
            body: api.body,
            author,
            created_at,
        }
    }
}
```

- [ ] **Step 6: Run all pr.rs tests to verify they pass**

Run: `cargo test -p gitflow-gitcode pr::`
Expected: PASS — all tests, including the 2 new ones and the 2 fixed exact-value assertions.

- [ ] **Step 7: Run clippy**

Run: `cargo clippy -p gitflow-gitcode --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: clean.

- [ ] **Step 8: Commit**

```bash
git add crates/gitcode/src/pr.rs
git commit -m "refactor(gitcode): stop faking pr/comment created_at with Utc::now()

parse_time and parse_opt_time were two closures doing the same RFC3339
parse, differing only in whether a missing timestamp became Utc::now()
or None — the file's own comment already flagged this as wrong for
merge timestamps (\"缺失会被填成「现在」，那等于谎报一次合并\"). Collapsed
into one Option-returning closure used for created_at/updated_at/merged_at
alike.

Refs #380"
```

---

### Task 8: `gitflow-github` — wrap Category C assignment sites in `Some(...)` for type compatibility, add verification tests

**Files:**
- Modify: `crates/github/src/issue.rs:699-711` (`From<GitHubCommentApiResponse> for CommentData`)
- Modify: `crates/github/src/issue.rs:732-748` (`From<GitHubIssueApiResponse> for IssueData`)
- Modify: `crates/github/src/issue.rs:1030-1045` (`test_should_handle_invalid_date_in_api_response`)
- Modify: `crates/github/src/issue.rs:1075-1085` (`test_should_fall_back_to_epoch_for_invalid_rest_issue_dates`)
- Modify: `crates/github/src/issue.rs:1340` (assertion inside a close-flow test)
- Modify: `crates/github/src/review.rs:248-267` (`From<GitHubReviewApiResponse> for ReviewData`)
- Modify: `crates/github/src/pipeline.rs:53-69` (`GhRun::into_status`)
- No production change: `crates/github/src/pr.rs` (direct fail-loud deserialization, no `From` impl exists)
- Test: all 4 files' `#[cfg(test)] mod tests` blocks

**Interfaces:**
- Consumes: `IssueData.created_at`/`updated_at`, `CommentData.created_at`, `ReviewData.submitted_at`, `PipelineStatus.created_at`/`updated_at`, all now `Option<DateTime<Utc>>`, from Task 1.
- **This task does NOT change what value these Category C sites fall back to on parse failure** (`parse_api_datetime`'s `UNIX_EPOCH` + `tracing::warn!`, `github/review.rs`'s and `github/pipeline.rs`'s bare `Utc::now()`) — that decision belongs to Issue #401. This task only wraps the existing return value in `Some(...)` so the assignment type-checks against the new `Option` field.

- [ ] **Step 1: Update the 3 existing tests whose assertions compare against the old bare type (RED)**

In `crates/github/src/issue.rs`, change `test_should_handle_invalid_date_in_api_response` from:

```rust
        let comment_data: CommentData = api_response.into();
        // Should fall back to UNIX_EPOCH
        assert_eq!(comment_data.created_at, chrono::DateTime::UNIX_EPOCH);
```

to:

```rust
        let comment_data: CommentData = api_response.into();
        // Should fall back to UNIX_EPOCH (still wrapped in Some — the value
        // is unchanged, only the type is now Option; the fallback VALUE
        // itself is Issue #401's decision, not this one's)
        assert_eq!(comment_data.created_at, Some(chrono::DateTime::UNIX_EPOCH));
```

Change `test_should_fall_back_to_epoch_for_invalid_rest_issue_dates` from:

```rust
        assert_eq!(issue.created_at, chrono::DateTime::UNIX_EPOCH);
        assert_eq!(issue.updated_at, chrono::DateTime::UNIX_EPOCH);
```

to:

```rust
        assert_eq!(issue.created_at, Some(chrono::DateTime::UNIX_EPOCH));
        assert_eq!(issue.updated_at, Some(chrono::DateTime::UNIX_EPOCH));
```

Change the assertion inside the close-flow test from:

```rust
        assert_eq!(issue.updated_at.to_rfc3339(), "2026-08-03T09:31:29+00:00");
```

to:

```rust
        assert_eq!(
            issue.updated_at.map(|dt| dt.to_rfc3339()),
            Some("2026-08-03T09:31:29+00:00".to_string())
        );
```

- [ ] **Step 2: Run the tests to verify they fail (RED)**

Run: `cargo test -p gitflow-github issue::tests::test_should_handle_invalid_date_in_api_response issue::tests::test_should_fall_back_to_epoch_for_invalid_rest_issue_dates`
Expected: does not compile — `From<GitHubCommentApiResponse> for CommentData` and `From<GitHubIssueApiResponse> for IssueData` both still assign a bare `DateTime<Utc>` (from `parse_api_datetime`) to what is now an `Option<DateTime<Utc>>` field. Confirm the compiler error names `created_at`/`updated_at` before proceeding.

- [ ] **Step 3: Wrap the `parse_api_datetime` call sites in `Some(...)`**

In `crates/github/src/issue.rs`, change:

```rust
impl From<GitHubCommentApiResponse> for CommentData {
    fn from(api: GitHubCommentApiResponse) -> Self {
        Self {
            id: api.id,
            body: api.body,
            author: gitflow_core::types::UserSummary {
                login: api.user.login,
                id: api.user.id.to_string(),
            },
            created_at: parse_api_datetime(&api.created_at),
        }
    }
}
```

to:

```rust
impl From<GitHubCommentApiResponse> for CommentData {
    fn from(api: GitHubCommentApiResponse) -> Self {
        Self {
            id: api.id,
            body: api.body,
            author: gitflow_core::types::UserSummary {
                login: api.user.login,
                id: api.user.id.to_string(),
            },
            // parse_api_datetime always returns a value (falling back to
            // UNIX_EPOCH on parse failure); wrapped in Some purely to match
            // CommentData.created_at's new Option type (#380). Whether that
            // fallback value itself should change is Issue #401's decision.
            created_at: Some(parse_api_datetime(&api.created_at)),
        }
    }
}
```

Change:

```rust
impl From<GitHubIssueApiResponse> for IssueData {
    fn from(api: GitHubIssueApiResponse) -> Self {
        Self {
            number: api.number,
            title: api.title,
            body: api.body,
            state: api.state,
            labels: api.labels.into_iter().map(Label::from).collect(),
            author: api.user.into(),
            assignees: api.assignees.into_iter().map(UserSummary::from).collect(),
            created_at: parse_api_datetime(&api.created_at),
            updated_at: parse_api_datetime(&api.updated_at),
            url: api.html_url,
            milestone: api.milestone,
        }
    }
}
```

to:

```rust
impl From<GitHubIssueApiResponse> for IssueData {
    fn from(api: GitHubIssueApiResponse) -> Self {
        Self {
            number: api.number,
            title: api.title,
            body: api.body,
            state: api.state,
            labels: api.labels.into_iter().map(Label::from).collect(),
            author: api.user.into(),
            assignees: api.assignees.into_iter().map(UserSummary::from).collect(),
            // Same Some(...) wrap as CommentData above, same reasoning.
            created_at: Some(parse_api_datetime(&api.created_at)),
            updated_at: Some(parse_api_datetime(&api.updated_at)),
            url: api.html_url,
            milestone: api.milestone,
        }
    }
}
```

- [ ] **Step 4: Wrap `github/review.rs`'s fallback**

Change:

```rust
impl From<GitHubReviewApiResponse> for ReviewData {
    fn from(api: GitHubReviewApiResponse) -> Self {
        Self {
            id: api.id,
            state: match api.state.as_str() {
                "APPROVED" => ReviewState::Approved,
                "CHANGES_REQUESTED" => ReviewState::ChangesRequested,
                _ => ReviewState::Commented,
            },
            body: api.body,
            author: gitflow_core::types::UserSummary {
                login: api.user.login,
                id: api.user.id.to_string(),
            },
            submitted_at: api
                .submitted_at
                .parse()
                .unwrap_or_else(|_| chrono::Utc::now()),
        }
    }
}
```

to:

```rust
impl From<GitHubReviewApiResponse> for ReviewData {
    fn from(api: GitHubReviewApiResponse) -> Self {
        Self {
            id: api.id,
            state: match api.state.as_str() {
                "APPROVED" => ReviewState::Approved,
                "CHANGES_REQUESTED" => ReviewState::ChangesRequested,
                _ => ReviewState::Commented,
            },
            body: api.body,
            author: gitflow_core::types::UserSummary {
                login: api.user.login,
                id: api.user.id.to_string(),
            },
            // Wrapped in Some purely for ReviewData.submitted_at's new
            // Option type (#380). This parse-failure fallback is silent
            // (no tracing::warn!, unlike issue.rs's parse_api_datetime) —
            // that inconsistency and the fallback value itself are Issue
            // #401's decision, not this one's.
            submitted_at: Some(
                api.submitted_at
                    .parse()
                    .unwrap_or_else(|_| chrono::Utc::now()),
            ),
        }
    }
}
```

- [ ] **Step 5: Wrap `github/pipeline.rs`'s fallback**

Change:

```rust
impl GhRun {
    fn into_status(self) -> PipelineStatus {
        let created_at = chrono::DateTime::parse_from_rfc3339(&self.created_at)
            .ok()
            .map_or_else(chrono::Utc::now, |dt| dt.with_timezone(&chrono::Utc));
        let updated_at = chrono::DateTime::parse_from_rfc3339(&self.updated_at)
            .ok()
            .map_or_else(chrono::Utc::now, |dt| dt.with_timezone(&chrono::Utc));

        PipelineStatus {
            id: self.database_id,
            ref_name: self.head_branch,
            status: gh_status_to_enum(&self.status, self.conclusion.as_deref()),
            conclusion: self.conclusion,
            created_at,
            updated_at,
            url: self.url,
        }
    }
}
```

to:

```rust
impl GhRun {
    fn into_status(self) -> PipelineStatus {
        let created_at = chrono::DateTime::parse_from_rfc3339(&self.created_at)
            .ok()
            .map_or_else(chrono::Utc::now, |dt| dt.with_timezone(&chrono::Utc));
        let updated_at = chrono::DateTime::parse_from_rfc3339(&self.updated_at)
            .ok()
            .map_or_else(chrono::Utc::now, |dt| dt.with_timezone(&chrono::Utc));

        PipelineStatus {
            id: self.database_id,
            ref_name: self.head_branch,
            status: gh_status_to_enum(&self.status, self.conclusion.as_deref()),
            conclusion: self.conclusion,
            // Wrapped in Some purely for PipelineStatus's new Option type
            // (#380). Fallback value unchanged — Issue #401's decision.
            created_at: Some(created_at),
            updated_at: Some(updated_at),
            url: self.url,
        }
    }
}
```

- [ ] **Step 6: Run the fixed tests to verify they pass**

Run: `cargo test -p gitflow-github issue::`
Expected: PASS — all tests including the 3 fixed in Step 1.

- [ ] **Step 7: Add verification tests to the 4 files**

Add to `crates/github/src/issue.rs`'s test module (proving the `IssueData`/`CommentData` type change is otherwise a no-op for the normal, present-timestamp path — mirrors #366's Task 4 pattern):

```rust
#[test]
fn test_should_deserialize_issue_with_present_timestamps_as_some() {
    // Sanity check that the Option type change doesn't break the normal
    // (timestamp present) path through GitHubIssueApiResponse, only the
    // already-covered UNIX_EPOCH fallback path.
    let issue: IssueData = sample_rest_issue_response().into();
    assert!(issue.created_at.is_some());
    assert!(issue.updated_at.is_some());
}
```

Add to `crates/github/src/review.rs`'s test module, near the other `ReviewData` conversion tests:

```rust
#[test]
fn test_should_deserialize_review_with_present_submitted_at_as_some() {
    let api_response = GitHubReviewApiResponse {
        id: 1,
        state: "APPROVED".to_string(),
        body: None,
        user: GitHubUser {
            login: "user".to_string(),
            id: 1,
        },
        submitted_at: "2026-08-03T10:00:00Z".to_string(),
    };
    let review_data: ReviewData = api_response.into();
    assert!(review_data.submitted_at.is_some());
}
```

(Adjust field names/types to match this file's actual `GitHubReviewApiResponse`/`GitHubUser` struct definitions exactly as they appear — read the file's existing `From<GitHubReviewApiResponse>`-based tests near line 390-400 for the precise construction pattern before writing this test, since the exact struct shape must compile.)

Add to `crates/github/src/pipeline.rs`'s test module, near the other `GhRun`/`into_status` tests:

```rust
#[test]
fn test_should_convert_run_with_present_timestamps_to_some() {
    let run = GhRun {
        database_id: 1,
        head_branch: "main".to_string(),
        status: "completed".to_string(),
        conclusion: Some("success".to_string()),
        created_at: "2026-07-01T10:00:00Z".to_string(),
        updated_at: "2026-07-01T10:05:30Z".to_string(),
        url: "https://example.com/runs/1".to_string(),
    };
    let status = run.into_status();
    assert!(status.created_at.is_some());
    assert!(status.updated_at.is_some());
}
```

Add to `crates/github/src/pr.rs`'s test module (test-only, no production change — proving the type change is a no-op for GitHub's fail-loud `PrData` path):

```rust
#[test]
fn test_should_deserialize_pr_with_missing_timestamps_as_none() {
    // gh CLI is not known to ever omit createdAt/updatedAt, but PrData's
    // fields are now Option<DateTime<Utc>> at the core level (#380) — this
    // pins that the github path tolerates their absence gracefully rather
    // than erroring, since this crate has no intermediate struct or
    // fallback logic of its own for PrData.
    let gh_json = br#"{
        "number": 9,
        "title": "No timestamps",
        "state": "open",
        "draft": false,
        "author": {"login": "octocat", "id": "1"},
        "baseBranch": "main",
        "headBranch": "feature/x",
        "mergedAt": null,
        "url": "https://github.com/octocat/hello-world/pull/9"
    }"#;

    let pr: PrData = serde_json::from_slice(gh_json).expect("missing timestamps must not error");
    assert!(pr.created_at.is_none());
    assert!(pr.updated_at.is_none());
}
```

- [ ] **Step 8: Run the new tests to verify they pass**

Run: `cargo test -p gitflow-github issue::tests::test_should_deserialize_issue_with_present_timestamps_as_some review::tests::test_should_deserialize_review_with_present_submitted_at_as_some pipeline::tests::test_should_convert_run_with_present_timestamps_to_some pr::tests::test_should_deserialize_pr_with_missing_timestamps_as_none`
Expected: PASS. If `test_should_deserialize_pr_with_missing_timestamps_as_none` fails, stop and investigate before proceeding — it would mean an undiscovered dependency on `PrData`'s fields being non-optional somewhere in this crate.

- [ ] **Step 9: Run all github crate tests, then clippy**

Run: `cargo test -p gitflow-github`
Expected: PASS, all tests in the crate.

Run: `cargo clippy -p gitflow-github --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: clean.

- [ ] **Step 10: Commit**

```bash
git add crates/github/src/issue.rs crates/github/src/review.rs crates/github/src/pipeline.rs crates/github/src/pr.rs
git commit -m "test(github): wrap Category C fallback sites in Some() for type compatibility

IssueData/CommentData's parse_api_datetime-based fallback (issue.rs),
ReviewData's submitted_at fallback (review.rs), and PipelineStatus's
into_status fallback (pipeline.rs) all still returned a concrete
DateTime<Utc> after #380's core type change made these fields Option.
Wrapped each in Some(...) with no change to the fallback value itself
— that decision belongs to #401, which tracks the parse-failure bug
class separately from #380's missing-field bug class. pr.rs needed no
production change (fail-loud deserialization) but gets a regression
test pinning that a missing timestamp still degrades to None cleanly.

Refs #380"
```

---

### Task 9: Final workspace-wide validation and `BREAKING CHANGE` record

**Files:**
- None modified — validation only, plus this task's own commit carries the `BREAKING CHANGE` note as an empty/no-op marker commit if no further code changes are needed, OR (more likely) this note travels with whichever of Tasks 1-8 already committed the type change. See Step 3 for the exact placement decision.

**Interfaces:**
- Consumes: everything from Tasks 1-8.

- [ ] **Step 1: Full workspace test run**

Run: `make test`
Expected: all tests pass, workspace-wide — this is the real check that no crate/test outside the ones already touched silently depended on any of the 5 fields being non-optional.

- [ ] **Step 2: Full workspace clippy and format check**

Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: clean, no warnings anywhere in the workspace.

Run: `cargo +nightly fmt -- --check`
Expected: no diff.

- [ ] **Step 3: Record the `BREAKING CHANGE` note**

Per this plan's Global Constraints, exactly one commit in this branch's history must carry a `BREAKING CHANGE:` paragraph. Since Task 1's commit (Step 7) is the one that actually changes the 5 public field types, amend that expectation by writing Task 1's commit message to already include this paragraph — go back and verify Task 1's commit message (written in Task 1 Step 7) reads exactly:

```
refactor(core): make issue/pr/review/comment/pipeline timestamps optional

IssueData.created_at/updated_at, PrData.created_at/updated_at,
ReviewData.submitted_at, CommentData.created_at, and
PipelineStatus.created_at/updated_at were all DateTime<Utc>. GitLab
and GitCode adapters fall back to Utc::now() when the platform API
omits these fields, displaying a record with an unknown creation time
as "created today" — the same class of silent-data-fabrication bug
already fixed for ReleaseData.created_at in #366. Each field now
mirrors that fix and the existing JobData.started_at/completed_at
pattern: Option<DateTime<Utc>> with skip_serializing_if, so None omits
the field from JSON entirely instead of faking a timestamp.

BREAKING CHANGE: IssueData.created_at/updated_at, PrData.created_at/updated_at,
ReviewData.submitted_at, CommentData.created_at, and
PipelineStatus.created_at/updated_at are now Option<DateTime<Utc>>
instead of DateTime<Utc>. Callers that pattern-match or construct these
types directly must handle the Option.

Refs #380
```

If Task 1's commit was made without this paragraph, do NOT create a new empty commit — instead use `git commit --amend` on Task 1's commit (this is the one exception to this repo's normal "always create new commits, never amend" rule, permissible only because Task 1's commit has not been pushed or merged yet and amending it here — before any push — does not destroy work, only adds a footer). If any later task's commit was made after Task 1, `git commit --amend` on Task 1 requires no rebase of subsequent commits' content, only their parent SHA changes automatically. Verify with `git log --oneline` that Task 1's commit message now contains `BREAKING CHANGE:` and no other commit does (`git log --grep="BREAKING CHANGE" --oneline` should show exactly one commit).

- [ ] **Step 4: Final review checklist (no code change, verification only)**

Confirm each of the following by inspection:
- [ ] `crates/core/src/review.rs`'s `ReviewCommentData` struct is byte-for-byte unchanged from before this plan started (design §6 exclusion).
- [ ] `crates/core/src/pr.rs`'s `PrData.merged_at` field is unchanged (`Option<DateTime<Utc>>`, no doc comment edit).
- [ ] `apps/cli/` has zero diff (`git diff --stat main...HEAD -- apps/cli/` is empty, assuming `main` is this branch's fork point).
- [ ] `crates/gitlab/src/review.rs`'s `approve()` method's logic (not just its comment) is unchanged — it must still call `Utc::now()` unconditionally, just wrapped in `Some(...)`.
- [ ] No `Cargo.toml` file in the workspace has a modified `version` field.

This is not a step that produces a git diff; it is a manual audit before declaring the plan complete. If any item fails, fix it now before moving to code review.
