# ReleaseData.created_at Optional Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Change `ReleaseData.created_at` from `DateTime<Utc>` to `Option<DateTime<Utc>>` across `gitflow-core` and its three platform adapters, so a release with no real creation timestamp is shown as absent instead of silently faked as "created today".

**Architecture:** `gitflow-core`'s `ReleaseData.created_at` becomes `Option<DateTime<Utc>>` with `#[serde(skip_serializing_if = "Option::is_none")]`, mirroring the existing `published_at` field in the same struct exactly. `gitflow-gitlab` and `gitflow-gitcode` each stop calling `.unwrap_or(now)` / `.unwrap_or_else(Utc::now)` in their `From<XxxApiResponse> for ReleaseData` impl and pass the already-`Option` value straight through. `gitflow-github` needs no production code change — it deserializes `gh` CLI JSON directly into `ReleaseData` with no intermediate struct — but gets a new regression test proving the type change doesn't break deserialization when `createdAt` is absent. `apps/cli` needs no change: its generic `serde_json::Value`-based renderer already omits missing JSON keys correctly (same as it already does for the sibling `publishedAt` field).

**Tech Stack:** Rust 2024, `chrono::DateTime<Utc>`, `serde`.

**Spec:** `docs/superpowers/specs/2026-09-21-release-created-at-optional-design.md`

## Global Constraints

- No change to `apps/cli/` (design §2, confirmed the generic output renderer already handles missing keys correctly).
- No change to `crates/gitlab/src/mr.rs`'s unrelated same-named `created_at` field (merge requests, different feature — design §6).
- No `Cargo.toml` version bump in this branch (design §3.2) — instead, the commit that introduces the breaking type change (Task 1) must include a `BREAKING CHANGE:` footer in its commit message, worded exactly:
  `BREAKING CHANGE: ReleaseData.created_at is now Option<DateTime<Utc>> instead of DateTime<Utc>. Callers that pattern-match or construct ReleaseData directly must handle the Option.`
- `None` serializes by omitting the field entirely (`skip_serializing_if`), not as an explicit `null` — mirrors `published_at` (design §3.1).
- Run `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic` on every touched crate; run full `make test` at the end of the last task since this is a public core type change with workspace-wide blast radius.

---

### Task 1: `gitflow-core` — make `created_at` optional

**Files:**
- Modify: `crates/core/src/release.rs:19-48` (struct), `crates/core/src/release.rs` test module (add one test)
- Test: `crates/core/src/release.rs` (same file, `#[cfg(test)] mod tests`)

**Interfaces:**
- Produces: `ReleaseData.created_at: Option<DateTime<Utc>>` (was `DateTime<Utc>`) — every downstream crate (gitlab, gitcode, github) constructs or reads this field and must be updated in later tasks to match.

- [ ] **Step 1: Write the failing test — missing `createdAt` deserializes to `None`, not an error**

Add to the `#[cfg(test)] mod tests` block in `crates/core/src/release.rs` (near the other `ReleaseData` deserialization tests, after `test_should_omit_none_fields_on_serialize`):

```rust
#[test]
fn test_should_deserialize_release_with_missing_created_at_as_none() {
    let json = r#"{
        "id": 7,
        "tagName": "v0.5.0",
        "draft": false,
        "prerelease": false,
        "url": "https://example.com/releases/7"
    }"#;
    let release: ReleaseData = serde_json::from_str(json).expect("deserialize");
    assert!(
        release.created_at.is_none(),
        "missing createdAt must deserialize to None, not error or a fabricated timestamp"
    );
}

#[test]
fn test_should_omit_created_at_when_none_on_serialize() {
    let json = sample_release_json();
    let mut release: ReleaseData = serde_json::from_str(json).expect("deserialize");
    release.created_at = None;
    let serialized = serde_json::to_string(&release).expect("serialize");
    assert!(!serialized.contains("\"createdAt\""));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p gitflow-core release::tests::test_should_deserialize_release_with_missing_created_at_as_none release::tests::test_should_omit_created_at_when_none_on_serialize`
Expected: FAIL — `test_should_deserialize_release_with_missing_created_at_as_none` fails because `created_at: DateTime<Utc>` is currently a required field, so `serde_json::from_str` returns an error and `.expect("deserialize")` panics. `test_should_omit_created_at_when_none_on_serialize` fails to compile (`release.created_at = None` — cannot assign `Option<_>` to `DateTime<Utc>`).

- [ ] **Step 3: Change the field type**

In `crates/core/src/release.rs`, change:

```rust
    /// 创建时间（UTC）。
    pub created_at: DateTime<Utc>,
```

to:

```rust
    /// 创建时间（UTC）。API 未返回该字段时为 `None`——绝不用当前时间伪造。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p gitflow-core release::`
Expected: PASS — all `ReleaseData` tests pass, including the two new ones. `test_should_roundtrip_release_data_via_serde` (line ~190, `assert_eq!(round_tripped.created_at, release.created_at)`) and `test_should_serialize_camel_case_fields` (asserts `serialized.contains("\"createdAt\"")`) need no code change: both compare/check a *present* value from `sample_release_json()` (which includes `"createdAt": "2026-01-01T00:00:00Z"`), and `Option<DateTime<Utc>>` derives `PartialEq`/serializes the same way as the bare type when `Some`.

- [ ] **Step 5: Run clippy**

Run: `cargo clippy -p gitflow-core --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: clean, no new warnings.

- [ ] **Step 6: Commit**

```bash
git add crates/core/src/release.rs
git commit -m "$(cat <<'EOF'
refactor(core): make ReleaseData.created_at optional

A release with no real creation timestamp was being displayed as
"created today" by gitlab/gitcode adapters falling back to Utc::now().
This is the same class of silent-data-fabrication bug as #360/#365,
just in the display layer instead of pagination. created_at now
mirrors the existing Option<DateTime<Utc>> published_at field exactly:
None omits the field from JSON entirely, and apps/cli's generic
output renderer already handles that correctly (verified: no apps/cli
change needed).

BREAKING CHANGE: ReleaseData.created_at is now Option<DateTime<Utc>>
instead of DateTime<Utc>. Callers that pattern-match or construct
ReleaseData directly must handle the Option.

Refs #366
EOF
)"
```

---

### Task 2: `gitflow-gitlab` — stop faking `created_at`

**Files:**
- Modify: `crates/gitlab/src/release.rs:166-184` (`From<ReleaseApiResponse> for ReleaseData` impl)
- Modify: `crates/gitlab/src/release.rs:788-813` (`test_should_convert_minimal_api_response_to_release_data`)
- Test: same file

**Interfaces:**
- Consumes: `ReleaseData.created_at: Option<DateTime<Utc>>` from Task 1.
- `ReleaseApiResponse.created_at` (this file, line 156) is already `Option<DateTime<Utc>>` — no change needed to the intermediate struct.

- [ ] **Step 1: Update the existing test's assertion (this is your RED step — it currently only documents the bug in a comment, without asserting it)**

In `crates/gitlab/src/release.rs`, change `test_should_convert_minimal_api_response_to_release_data` (around line 788) from:

```rust
        assert_eq!(release.url, ""); // defaults to empty string
        // created_at defaults to Utc::now() when None
    }
```

to:

```rust
        assert_eq!(release.url, ""); // defaults to empty string
        assert!(
            release.created_at.is_none(),
            "missing created_at must stay None, not fall back to Utc::now()"
        );
    }
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p gitflow-gitlab release::tests::test_should_convert_minimal_api_response_to_release_data`
Expected: FAIL — `release.created_at` is currently `Utc::now()` (a `Some`-shaped value once Task 1 lands, since the field is now `Option`, `Utc::now()` no longer even compiles as an assignment target — this will actually be a compile error, not a runtime assertion failure, until Step 3 fixes the `From` impl. Either way, the test does not pass yet.)

- [ ] **Step 3: Remove the fallback in the `From` impl**

In `crates/gitlab/src/release.rs`, change:

```rust
impl From<ReleaseApiResponse> for ReleaseData {
    fn from(api: ReleaseApiResponse) -> Self {
        let now = Utc::now();
        let author = api.author.as_ref().map(UserSummary::from);

        Self {
            id: api.id.unwrap_or(0),
            tag_name: api.tag_name,
            name: api.name,
            body: api.description,
            draft: api.draft,
            prerelease: api.prerelease,
            author,
            created_at: api.created_at.unwrap_or(now),
            published_at: api.released_at,
            url: api.url.unwrap_or_default(),
        }
    }
}
```

to:

```rust
impl From<ReleaseApiResponse> for ReleaseData {
    fn from(api: ReleaseApiResponse) -> Self {
        let author = api.author.as_ref().map(UserSummary::from);

        Self {
            id: api.id.unwrap_or(0),
            tag_name: api.tag_name,
            name: api.name,
            body: api.description,
            draft: api.draft,
            prerelease: api.prerelease,
            author,
            created_at: api.created_at,
            published_at: api.released_at,
            url: api.url.unwrap_or_default(),
        }
    }
}
```

(The local `let now = Utc::now();` binding is removed entirely — it has no other use in this function.)

- [ ] **Step 4: Run the test to verify it passes**

Run: `cargo test -p gitflow-gitlab release::`
Expected: PASS — all release tests pass, including `test_should_preserve_all_fields_in_full_api_response_conversion` (a `Some(created_at)` case, unaffected by removing the fallback since the value was present).

- [ ] **Step 5: Run clippy**

Run: `cargo clippy -p gitflow-gitlab --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: clean. (Removing the unused `now` binding avoids a new `clippy::let_and_return`-style or unused-variable warning — verify none appears.)

- [ ] **Step 6: Commit**

```bash
git add crates/gitlab/src/release.rs
git commit -m "refactor(gitlab): stop faking created_at with Utc::now() when API omits it

Refs #366"
```

---

### Task 3: `gitflow-gitcode` — stop faking `created_at`

**Files:**
- Modify: `crates/gitcode/src/release.rs:170-185` (`From<ReleaseApiResponse> for ReleaseData` impl, around line 180)
- Modify: `crates/gitcode/src/release.rs` — three existing tests need their assertions updated for the `Option` type:
  - `test_should_degrade_predictably_for_minimal_api_release_object` (~line 964-979): add a `created_at` assertion
  - `test_should_deserialize_release_from_gitcode_api_response` (~line 894): `assert_eq!(release.created_at.to_rfc3339(), ...)` → needs `Option` handling
  - `test_should_deserialize_real_gitcode_release_api_response` (~line 1058): `assert_eq!(release.created_at, chrono::DateTime::parse_from_rfc3339(...)...)` → needs `Some(...)` wrapping
- Test: same file

**Interfaces:**
- Consumes: `ReleaseData.created_at: Option<DateTime<Utc>>` from Task 1.
- `ReleaseApiResponse.created_at` (this file, line 158) is already `Option<DateTime<Utc>>` — no change needed to the intermediate struct.

- [ ] **Step 1: Add the missing assertion to `test_should_degrade_predictably_for_minimal_api_release_object` (RED)**

In `crates/gitcode/src/release.rs`, find:

```rust
    async fn test_should_degrade_predictably_for_minimal_api_release_object() {
        // 形状不匹配时必须整体退化而非半途失败：所有字段都有 default。
        let runner = MockCommandRunner::success(r#"[{"tag_name": "v0.0.1"}]"#);
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let paged = provider.list(None).await.expect("list should succeed");

        let release = &paged.items[0];
        assert_eq!(release.tag_name, "v0.0.1");
        assert_eq!(release.id, 0);
        assert!(!release.draft);
        assert!(!release.prerelease);
        assert!(release.author.is_none());
        assert!(release.published_at.is_none());
        assert!(release.url.is_empty());
    }
```

Add one line before the closing brace:

```rust
    async fn test_should_degrade_predictably_for_minimal_api_release_object() {
        // 形状不匹配时必须整体退化而非半途失败：所有字段都有 default。
        let runner = MockCommandRunner::success(r#"[{"tag_name": "v0.0.1"}]"#);
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let paged = provider.list(None).await.expect("list should succeed");

        let release = &paged.items[0];
        assert_eq!(release.tag_name, "v0.0.1");
        assert_eq!(release.id, 0);
        assert!(!release.draft);
        assert!(!release.prerelease);
        assert!(release.author.is_none());
        assert!(release.published_at.is_none());
        assert!(release.url.is_empty());
        assert!(
            release.created_at.is_none(),
            "missing created_at must stay None, not fall back to Utc::now()"
        );
    }
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p gitflow-gitcode release::tests::test_should_degrade_predictably_for_minimal_api_release_object`
Expected: FAIL (or compile error — once Task 1's core change lands, `api.created_at.unwrap_or_else(Utc::now)` still compiles fine since it still produces a `DateTime<Utc>`... but assigning that `DateTime<Utc>` to the now-`Option<DateTime<Utc>>` field is a type mismatch, so this crate will not currently compile at all until Step 3. Confirm the compiler error names `created_at` before proceeding.)

- [ ] **Step 3: Remove the fallback in the `From` impl**

In `crates/gitcode/src/release.rs`, change:

```rust
impl From<ReleaseApiResponse> for ReleaseData {
    fn from(api: ReleaseApiResponse) -> Self {
        Self {
            id: api.id.and_then(|s| s.parse().ok()).unwrap_or(0),
            tag_name: api.tag_name.unwrap_or_default(),
            name: api.name,
            body: api.body,
            draft: api.draft.unwrap_or_default(),
            prerelease: api.prerelease.unwrap_or_default(),
            author: api.author.map(UserSummary::from),
            created_at: api.created_at.unwrap_or_else(Utc::now),
            published_at: api.published_at,
            url: api.html_url.or(api.url).unwrap_or_default(),
        }
    }
}
```

to:

```rust
impl From<ReleaseApiResponse> for ReleaseData {
    fn from(api: ReleaseApiResponse) -> Self {
        Self {
            id: api.id.and_then(|s| s.parse().ok()).unwrap_or(0),
            tag_name: api.tag_name.unwrap_or_default(),
            name: api.name,
            body: api.body,
            draft: api.draft.unwrap_or_default(),
            prerelease: api.prerelease.unwrap_or_default(),
            author: api.author.map(UserSummary::from),
            created_at: api.created_at,
            published_at: api.published_at,
            url: api.html_url.or(api.url).unwrap_or_default(),
        }
    }
}
```

- [ ] **Step 4: Fix `test_should_deserialize_release_from_gitcode_api_response`'s assertion**

Change:

```rust
        assert_eq!(release.created_at.to_rfc3339(), "2026-01-01T00:00:00+00:00");
```

to:

```rust
        assert_eq!(
            release.created_at.map(|dt| dt.to_rfc3339()),
            Some("2026-01-01T00:00:00+00:00".to_string())
        );
```

- [ ] **Step 5: Fix `test_should_deserialize_real_gitcode_release_api_response`'s assertion**

Change:

```rust
        assert_eq!(
            release.created_at,
            chrono::DateTime::parse_from_rfc3339("2026-09-20T22:22:44+08:00")
                .expect("valid rfc3339")
                .with_timezone(&Utc)
        );
```

to:

```rust
        assert_eq!(
            release.created_at,
            Some(
                chrono::DateTime::parse_from_rfc3339("2026-09-20T22:22:44+08:00")
                    .expect("valid rfc3339")
                    .with_timezone(&Utc)
            )
        );
```

- [ ] **Step 6: Run all release tests to verify they pass**

Run: `cargo test -p gitflow-gitcode release::`
Expected: PASS — every test in the file, including `test_should_fail_list_when_created_at_is_not_rfc3339` (unaffected: a malformed non-empty string still fails deserialization the same way regardless of `Option`, since `Option<DateTime<Utc>>`'s `Some` branch still runs full RFC3339 parsing).

- [ ] **Step 7: Run clippy**

Run: `cargo clippy -p gitflow-gitcode --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: clean.

- [ ] **Step 8: Commit**

```bash
git add crates/gitcode/src/release.rs
git commit -m "refactor(gitcode): stop faking created_at with Utc::now() when API omits it

Refs #366"
```

---

### Task 4: `gitflow-github` — prove the type change is a no-op here, then full workspace validation

**Files:**
- Modify: `crates/github/src/release.rs` test module (add one test, near `test_should_deserialize_draft_release_from_gh_output`)

**Interfaces:**
- Consumes: `ReleaseData.created_at: Option<DateTime<Utc>>` from Task 1. No production code in this crate references `created_at` directly (it deserializes straight into `ReleaseData`), so this task is test-only.

- [ ] **Step 1: Write the test**

Add to `crates/github/src/release.rs`'s test module, after `test_should_deserialize_draft_release_from_gh_output`:

```rust
#[test]
fn test_should_deserialize_release_with_missing_created_at_from_gh_output() {
    // gh CLI is not known to ever omit createdAt, but ReleaseData.created_at
    // is now Option<DateTime<Utc>> at the core level (#366) — this pins that
    // the github path tolerates it gracefully rather than erroring, since
    // this crate has no intermediate struct or fallback logic of its own.
    let gh_json = br#"{
        "id": 8,
        "tagName": "v0.9.0",
        "draft": false,
        "prerelease": false,
        "url": "https://example.com/releases/8"
    }"#;

    let release: ReleaseData =
        serde_json::from_slice(gh_json).expect("missing createdAt must not error");
    assert!(release.created_at.is_none());
}
```

- [ ] **Step 2: Run the test to verify it passes**

Run: `cargo test -p gitflow-github release::tests::test_should_deserialize_release_with_missing_created_at_from_gh_output`
Expected: PASS immediately (Task 1 already made the field `Option`, and this crate has no fallback code to remove) — this step is verification, not RED→GREEN, since there is no bug to fix here. If it fails, stop and investigate before proceeding — it would mean an undiscovered dependency on the old required-field behavior somewhere in this crate.

- [ ] **Step 3: Run clippy on this crate**

Run: `cargo clippy -p gitflow-github --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: clean.

- [ ] **Step 4: Full workspace validation**

Run: `make test`
Expected: all tests pass, workspace-wide (this is the real check that no other crate/test anywhere silently depended on `created_at` being non-optional — the earlier per-crate steps only proved the touched crates work in isolation).

Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: clean, no warnings anywhere in the workspace.

Run: `cargo +nightly fmt -- --check`
Expected: no diff.

- [ ] **Step 5: Commit**

```bash
git add crates/github/src/release.rs
git commit -m "test(github): pin that missing createdAt degrades to None, not an error

Refs #366"
```

## Self-Review Notes

- **Spec coverage:** §3.1 (omit-on-None) — Task 1. §3.2 (no version bump, BREAKING CHANGE footer) — Task 1's commit message. §4 file table — Task 1 (core), Task 2 (gitlab), Task 3 (gitcode), Task 4 (github, test-only) all map 1:1. §5 test strategy — RED/GREEN sequencing followed in Tasks 2 and 3; Task 4 documents why it isn't a RED→GREEN case (no bug existed there). §6 out-of-scope items (`mr.rs`, `apps/cli`, version bump) — none of the four tasks touch them.
- **Placeholder scan:** no TBD/TODO; every code block is complete, copy-pasteable, and shows full before/after context rather than describing the change in prose.
- **Type consistency:** `Option<DateTime<Utc>>` is the type introduced in Task 1 and consumed identically (via `.is_none()`, `Some(...)`, `.map(...)`) in Tasks 2-4 — no naming or shape drift between tasks.
- **Task ordering rationale:** Tasks are ordered by crate dependency direction (`gitflow-core` has no dependency on the platform crates; `gitflow-gitlab`/`gitflow-gitcode`/`gitflow-github` all depend on `gitflow-core`), so each task's `cargo test -p <crate>` is guaranteed compilable once the prior task lands — no task leaves an intermediate commit where `cargo build --workspace` is broken in a way that blocks the *next* task's isolated crate-level testing (though the full workspace only goes green again after Task 2 and Task 3 both land, since gitlab and gitcode both stop compiling the moment Task 1's core type change lands until their own fallback code is fixed — this is called out explicitly in Task 2 Step 2 and Task 3 Step 2 so the implementer isn't surprised by a compile error where a test failure was expected).
