# Temp-File Test Isolation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **Batching note (gf-workflow):** complexity score = 4 (4 files × 1, no module-boundary/API/migration risk) → **simple/batch**. Implement all 4 tasks in one pass in the main agent, then a single review pass — not one subagent per task.

**Goal:** Replace the shared-fixed-filename temp file pattern in 4 unit tests with `tempfile::NamedTempFile`, removing the shared-path collision risk that plausibly caused a single Windows CI flake (issue #301, follow-up of #289).

**Architecture:** No architectural change — this is a mechanical, identical substitution applied to 4 independent test functions in 4 different files. No production code, no new types, no new dependencies (`tempfile` is already a workspace dependency).

**Tech Stack:** Rust, `tempfile::NamedTempFile` (already in `apps/cli/Cargo.toml` under `[dependencies]` and `[dev-dependencies]`), `cargo nextest`.

**Spec:** `docs/superpowers/specs/2026-09-04-temp-file-test-isolation-design.md`

## Global Constraints

- No behavior change to `resolve_comment_body` / `resolve_body` / `SafePath` production logic — test-only change.
- Preserve existing assertions and expected values exactly — only the temp-file setup/teardown mechanism changes.
- `tempfile::NamedTempFile` auto-deletes on `Drop` — do not add a manual `remove_file` call back in.

---

### Task 1: `apps/cli/src/commands/commit.rs`

**Files:**
- Modify: `apps/cli/src/commands/commit.rs:238-247` (`test_should_resolve_comment_body_from_file`)

**Interfaces:**
- Consumes: `resolve_comment_body(None, Option<String>) -> Result<String, _>` (unchanged, already defined in this file)
- Produces: nothing consumed by later tasks — each task is independent

- [ ] **Step 1: Confirm current test content (already read)**

Current test (for reference, no action needed — this is the starting state):

```rust
#[test]
fn test_should_resolve_comment_body_from_file() {
    let dir = std::env::temp_dir();
    let path = dir.join("gitflow_test_commit_comment.md");
    std::fs::write(&path, "commit comment from file").expect("write temp file");
    let result = resolve_comment_body(None, Some(path.to_string_lossy().into_owned()));
    let _ = std::fs::remove_file(&path);
    assert!(result.is_ok());
    assert_eq!(result.expect("already checked"), "commit comment from file");
}
```

- [ ] **Step 2: Replace with `NamedTempFile`-based version**

```rust
#[test]
fn test_should_resolve_comment_body_from_file() {
    let file = tempfile::NamedTempFile::new().expect("create temp file");
    std::fs::write(file.path(), "commit comment from file").expect("write temp file");
    let result = resolve_comment_body(
        None,
        Some(file.path().to_string_lossy().into_owned()),
    );
    assert!(result.is_ok());
    assert_eq!(result.expect("already checked"), "commit comment from file");
}
```

- [ ] **Step 3: Run the test to verify it still passes**

Run: `cargo nextest run -p gitflow-cli commands::commit::tests::test_should_resolve_comment_body_from_file`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add apps/cli/src/commands/commit.rs
git commit -m "test(commit): use NamedTempFile instead of fixed shared temp filename"
```

---

### Task 2: `apps/cli/src/commands/issue.rs`

**Files:**
- Modify: `apps/cli/src/commands/issue.rs:448-459` (`test_should_resolve_body_from_file`)

**Interfaces:**
- Consumes: `resolve_body(None, Option<String>) -> Result<Option<String>, _>` (unchanged, already defined in this file)
- Produces: nothing consumed by later tasks — each task is independent

- [ ] **Step 1: Confirm current test content (already read)**

```rust
#[test]
fn test_should_resolve_body_from_file() {
    let dir = std::env::temp_dir();
    let path = dir.join("gitflow_test_body.md");
    std::fs::write(&path, "file content here").expect("write temp file");
    let result = resolve_body(None, Some(path.to_string_lossy().into_owned()));
    let _ = std::fs::remove_file(&path);
    assert!(result.is_ok());
    assert_eq!(
        result.expect("already checked"),
        Some("file content here".into())
    );
}
```

- [ ] **Step 2: Replace with `NamedTempFile`-based version**

```rust
#[test]
fn test_should_resolve_body_from_file() {
    let file = tempfile::NamedTempFile::new().expect("create temp file");
    std::fs::write(file.path(), "file content here").expect("write temp file");
    let result = resolve_body(None, Some(file.path().to_string_lossy().into_owned()));
    assert!(result.is_ok());
    assert_eq!(
        result.expect("already checked"),
        Some("file content here".into())
    );
}
```

- [ ] **Step 3: Run the test to verify it still passes**

Run: `cargo nextest run -p gitflow-cli commands::issue::tests::test_should_resolve_body_from_file`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add apps/cli/src/commands/issue.rs
git commit -m "test(issue): use NamedTempFile instead of fixed shared temp filename"
```

---

### Task 3: `apps/cli/src/commands/pr.rs`

**Files:**
- Modify: `apps/cli/src/commands/pr.rs:587-598` (`test_should_resolve_body_from_file`)

**Interfaces:**
- Consumes: `resolve_body(None, Option<String>) -> Result<Option<String>, _>` (unchanged, already defined in this file — a separate local definition from `issue.rs`'s, same signature/behavior)
- Produces: nothing consumed by later tasks — each task is independent

- [ ] **Step 1: Confirm current test content (already read)**

```rust
#[test]
fn test_should_resolve_body_from_file() {
    let dir = std::env::temp_dir();
    let path = dir.join("gitflow_test_pr_body.md");
    std::fs::write(&path, "pr body from file").expect("write temp file");
    let result = resolve_body(None, Some(path.to_string_lossy().into_owned()));
    let _ = std::fs::remove_file(&path);
    assert!(result.is_ok());
    assert_eq!(
        result.expect("already checked"),
        Some("pr body from file".into())
    );
}
```

- [ ] **Step 2: Replace with `NamedTempFile`-based version**

```rust
#[test]
fn test_should_resolve_body_from_file() {
    let file = tempfile::NamedTempFile::new().expect("create temp file");
    std::fs::write(file.path(), "pr body from file").expect("write temp file");
    let result = resolve_body(None, Some(file.path().to_string_lossy().into_owned()));
    assert!(result.is_ok());
    assert_eq!(
        result.expect("already checked"),
        Some("pr body from file".into())
    );
}
```

- [ ] **Step 3: Run the test to verify it still passes**

Run: `cargo nextest run -p gitflow-cli commands::pr::tests::test_should_resolve_body_from_file`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add apps/cli/src/commands/pr.rs
git commit -m "test(pr): use NamedTempFile instead of fixed shared temp filename"
```

---

### Task 4: `apps/cli/src/commands/release.rs`

**Files:**
- Modify: `apps/cli/src/commands/release.rs:368-379` (`test_should_resolve_body_from_file`)

**Interfaces:**
- Consumes: `resolve_body(None, Option<String>) -> Result<Option<String>, _>` (unchanged, already defined in this file — a separate local definition, same signature/behavior)
- Produces: nothing consumed by later tasks — each task is independent

- [ ] **Step 1: Confirm current test content (already read)**

```rust
#[test]
fn test_should_resolve_body_from_file() {
    let dir = std::env::temp_dir();
    let path = dir.join("gitflow_release_body.md");
    std::fs::write(&path, "release body content").expect("write temp file");
    let result = resolve_body(None, Some(path.to_string_lossy().into_owned()));
    let _ = std::fs::remove_file(&path);
    assert!(result.is_ok());
    assert_eq!(
        result.expect("already checked"),
        Some("release body content".into())
    );
}
```

- [ ] **Step 2: Replace with `NamedTempFile`-based version**

```rust
#[test]
fn test_should_resolve_body_from_file() {
    let file = tempfile::NamedTempFile::new().expect("create temp file");
    std::fs::write(file.path(), "release body content").expect("write temp file");
    let result = resolve_body(None, Some(file.path().to_string_lossy().into_owned()));
    assert!(result.is_ok());
    assert_eq!(
        result.expect("already checked"),
        Some("release body content".into())
    );
}
```

- [ ] **Step 3: Run the test to verify it still passes**

Run: `cargo nextest run -p gitflow-cli commands::release::tests::test_should_resolve_body_from_file`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add apps/cli/src/commands/release.rs
git commit -m "test(release): use NamedTempFile instead of fixed shared temp filename"
```

---

### Task 5: Full-suite verification

**Files:**
- None modified — verification only.

**Interfaces:**
- Consumes: all 4 modified tests from Tasks 1-4
- Produces: nothing (terminal task)

- [ ] **Step 1: Run the full workspace test suite**

Run: `make test` (equivalent to `cargo nextest run --all-features`)
Expected: all tests pass, including the 4 modified ones; no regressions elsewhere.

- [ ] **Step 2: Run lint**

Run: `make lint` (fmt + clippy pedantic)
Expected: clean — no new warnings from the 4 modified files.

- [ ] **Step 3: Confirm no stray temp files or manual `remove_file` calls remain**

Run: `grep -n "gitflow_test_commit_comment.md\|gitflow_test_body.md\|gitflow_test_pr_body.md\|gitflow_release_body.md\|std::env::temp_dir()" apps/cli/src/commands/{commit,issue,pr,release}.rs`
Expected: no matches (all 4 fixed-filename patterns removed).
