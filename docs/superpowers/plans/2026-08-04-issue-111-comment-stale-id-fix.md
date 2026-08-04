# Issue #111: Fix GitHub Comment Stale ID Bug — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix `issue comment` and `pr comment` commands so they return the newly created comment's data instead of the first (oldest) comment's data.

**Architecture:** Replace the two-step create-then-fetch pattern (`gh issue comment` + `gh api GET ?per_page=1`) with a single `gh api POST` call that creates the comment and returns the created object directly. Apply the same fix to both `issue.rs` and `pr.rs`.

**Tech Stack:** Rust, `tokio`, `serde_json`, GitHub REST API via `gh` CLI

## Global Constraints

- Must compile with `cargo build --all-targets --all-features`
- Must pass `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`
- Must pass `cargo test`
- No breaking changes to public API (`CommentData` return type unchanged)
- Follow existing error handling patterns (`parse_gh_error`, `CoreError::Serialization`)

---

## File Structure

| Action | File | Responsibility |
|--------|------|----------------|
| Modify | `crates/github/src/issue.rs` | Fix `comment()` method (L351-406) |
| Modify | `crates/github/src/pr.rs` | Fix `comment()` method (L255-310) |
| (no new files) | | Existing types `GitHubCommentApiResponse`, `GitHubUser` already match POST response shape |

---

### Task 1: Fix `issue.rs` `comment()` — TDD

**Files:**
- Modify: `crates/github/src/issue.rs:343-406` (comment method)
- Test: `crates/github/src/issue.rs` (inline `#[cfg(test)] mod tests`)

**Interfaces:**
- Consumes: `CommandRunner` trait (via `self.runner`), `GitHubCommentApiResponse` struct
- Produces: `async fn comment(&self, number: u64, body: &str) -> Result<CommentData>`

- [ ] **Step 1: Write failing test — verify current bug (RED)**

Add this test to `crates/github/src/issue.rs` inside the `mod tests` block, after the existing comment tests (around line 834):

```rust
#[tokio::test]
async fn test_should_return_newly_created_comment_not_stale() {
    use crate::runner::MockCommandRunner;

    // Mock `gh api POST` response — the comment just created
    let post_response_json = r#"{
        "id": 9999999999,
        "body": "This is the NEW comment",
        "user": {"login": "testuser", "id": 42},
        "created_at": "2026-08-04T12:00:00Z"
    }"#;

    let runner = MockCommandRunner::success(post_response_json);
    let provider = GitHubIssueProvider::with_runner("owner/repo", runner);

    let result = provider.comment(42, "This is the NEW comment").await.unwrap();

    // The returned id must be the NEW comment's id, not a stale one
    assert_eq!(result.id, 9999999999);
    assert_eq!(result.body, "This is the NEW comment");
    assert_eq!(result.author.login, "testuser");
}
```

- [ ] **Step 2: Run test to verify it FAILS**

```bash
cargo test -p gitflow-cli-github test_should_return_newly_created_comment_not_stale -- --nocapture
```

Expected: FAIL — current implementation calls `gh api GET` and parses array, not single object. The mock returns a single JSON object but the code tries to deserialize it as `Vec<GitHubCommentApiResponse>`, which will fail with a serialization error.

- [ ] **Step 3: Implement the fix**

Replace the entire `comment()` method in `crates/github/src/issue.rs` (lines 343-406) with:

```rust
    /// 在指定 Issue 上添加评论。
    ///
    /// 调用 `gh api repos/{repo}/issues/{number}/comments -X POST` 创建评论，
    /// 直接从 POST 响应中解析新创建的评论数据。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在、`body` 为空或 `gh` CLI 调用失败时返回错误。
    async fn comment(&self, number: u64, body: &str) -> Result<CommentData> {
        debug!(repo = %self.repo, number, "spawning `gh api` POST to create issue comment");

        let api_path = format!(
            "repos/{repo}/issues/{number}/comments",
            repo = self.repo,
            number = number
        );

        let body_field = format!("body={body}");
        let output = self
            .runner
            .run(
                "gh",
                &["api", &api_path, "-X", "POST", "-f", &body_field],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh api: {e}")))?;

        if !output.status.success() {
            return Err(parse_gh_error(&output.stderr).into());
        }

        let comment: GitHubCommentApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(comment.into())
    }
```

- [ ] **Step 4: Run test to verify it PASSES (GREEN)**

```bash
cargo test -p gitflow-cli-github test_should_return_newly_created_comment_not_stale -- --nocapture
```

Expected: PASS

- [ ] **Step 5: Add error path test**

Add this test right after the previous one:

```rust
#[tokio::test]
async fn test_should_return_error_when_gh_api_post_fails() {
    use crate::runner::MockCommandRunner;

    let runner = MockCommandRunner::failure(
        r#"{"message": "Issue not found", "documentation_url": "https://docs.github.com"}"#,
        1,
    );
    let provider = GitHubIssueProvider::with_runner("owner/repo", runner);

    let result = provider.comment(999, "test").await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, CoreError::Platform(_)),
        "expected Platform error, got: {err:?}"
    );
}
```

- [ ] **Step 6: Run all github crate tests**

```bash
cargo test -p gitflow-cli-github
```

Expected: All tests PASS

- [ ] **Step 7: Commit**

```bash
git add crates/github/src/issue.rs
git commit -m "fix(github): use gh api POST for issue comment (#111)

Replace two-step create-then-fetch with single gh api POST call.
Previous implementation used per_page=1 which returned oldest comment
instead of newly created one.

Closes #111"
```

---

### Task 2: Fix `pr.rs` `comment()` — TDD

**Files:**
- Modify: `crates/github/src/pr.rs:247-310` (comment method)
- Test: `crates/github/src/pr.rs` (inline `#[cfg(test)] mod tests`)

**Interfaces:**
- Consumes: `CommandRunner` trait, `crate::issue::GitHubCommentApiResponse`
- Produces: `async fn comment(&self, number: u64, body: &str) -> Result<CommentData>`

- [ ] **Step 1: Check existing PR tests**

```bash
grep -n "#\[tokio::test\]" crates/github/src/pr.rs | head -10
```

Find the test module location. If no `#[cfg(test)]` block exists, check if tests are in `issue.rs` or a separate test file.

- [ ] **Step 2: Write failing test for PR comment (RED)**

Find the test module in `crates/github/src/pr.rs` (or create one if absent following the pattern from `issue.rs`). Add:

```rust
#[tokio::test]
async fn test_should_return_newly_created_pr_comment_not_stale() {
    use crate::runner::MockCommandRunner;

    let post_response_json = r#"{
        "id": 8888888888,
        "body": "NEW PR comment",
        "user": {"login": "reviewer", "id": 99},
        "created_at": "2026-08-04T13:00:00Z"
    }"#;

    let runner = MockCommandRunner::success(post_response_json);
    let provider = GitHubPrProvider::with_runner("owner/repo", runner);

    let result = provider.comment(7, "NEW PR comment").await.unwrap();

    assert_eq!(result.id, 8888888888);
    assert_eq!(result.body, "NEW PR comment");
    assert_eq!(result.author.login, "reviewer");
}
```

- [ ] **Step 3: Run test to verify it FAILS**

```bash
cargo test -p gitflow-cli-github test_should_return_newly_created_pr_comment_not_stale -- --nocapture
```

Expected: FAIL — same serialization mismatch as Task 1.

- [ ] **Step 4: Implement the fix**

Replace the entire `comment()` method in `crates/github/src/pr.rs` (lines 247-310) with:

```rust
    /// 在指定 PR 上添加评论。
    ///
    /// 调用 `gh api repos/{repo}/issues/{number}/comments -X POST` 创建评论，
    /// 直接从 POST 响应中解析新创建的评论数据。
    /// （PR 评论使用与 Issue 评论相同的 API 端点。）
    ///
    /// # Errors
    ///
    /// 当 PR 不存在、`body` 为空或 `gh` CLI 调用失败时返回错误。
    async fn comment(&self, number: u64, body: &str) -> Result<CommentData> {
        debug!(repo = %self.repo, number, "spawning `gh api` POST to create PR comment");

        let api_path = format!(
            "repos/{repo}/issues/{number}/comments",
            repo = self.repo,
            number = number
        );

        let body_field = format!("body={body}");
        let output = self
            .runner
            .run(
                "gh",
                &["api", &api_path, "-X", "POST", "-f", &body_field],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh api: {e}")))?;

        if !output.status.success() {
            return Err(parse_gh_error(&output.stderr).into());
        }

        let comment: crate::issue::GitHubCommentApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(comment.into())
    }
```

- [ ] **Step 5: Run test to verify it PASSES (GREEN)**

```bash
cargo test -p gitflow-cli-github test_should_return_newly_created_pr_comment_not_stale -- --nocapture
```

Expected: PASS

- [ ] **Step 6: Add error path test**

```rust
#[tokio::test]
async fn test_should_return_error_when_pr_comment_api_fails() {
    use crate::runner::MockCommandRunner;

    let runner = MockCommandRunner::failure(
        r#"{"message": "Not Found", "documentation_url": "https://docs.github.com"}"#,
        1,
    );
    let provider = GitHubPrProvider::with_runner("owner/repo", runner);

    let result = provider.comment(999, "test").await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, CoreError::Platform(_)),
        "expected Platform error, got: {err:?}"
    );
}
```

- [ ] **Step 7: Run all github crate tests**

```bash
cargo test -p gitflow-cli-github
```

Expected: All tests PASS

- [ ] **Step 8: Commit**

```bash
git add crates/github/src/pr.rs
git commit -m "fix(github): use gh api POST for PR comment (#111)

Same fix as issue comment — replace two-step pattern with single
gh api POST call to avoid returning stale comment data.

Closes #111"
```

---

### Task 3: Quality Gate — Clippy + Full Test Suite

**Files:** (no modifications, validation only)

- [ ] **Step 1: Run clippy with pedantic warnings**

```bash
cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic
```

Expected: No warnings or errors. If any warnings appear in the modified code, fix them inline.

- [ ] **Step 2: Run full test suite**

```bash
cargo test --all-targets --all-features
```

Expected: All tests PASS

- [ ] **Step 3: Run format check**

```bash
cargo +nightly fmt -- --check
```

If formatting issues found:

```bash
cargo +nightly fmt
```

- [ ] **Step 4: Commit any formatting fixes**

```bash
git add -A
git commit -m "style: fix formatting in github crate comment methods"
```

(Only if Step 3 found and fixed issues.)

---

### Task 4: E2E Smoke Test — Verify Fix with Real GitHub API

**Files:** (no code modifications — manual verification)

- [ ] **Step 1: Build the binary**

```bash
cargo build --bin gitflow-cli
```

- [ ] **Step 2: Create a test comment on a known issue**

Pick an issue with multiple existing comments (e.g., #111 itself has several). Run:

```bash
./target/debug/gitflow-cli issue comment 111 --body "E2E verification: this comment tests fix #111 $(date +%s)"
```

- [ ] **Step 3: Verify returned data is correct**

Check the JSON output:
1. `data.id` should be a **new** comment id (large number, not matching any existing comment)
2. `data.createdAt` should be **current timestamp** (2026-08-04 or later)
3. `data.body` should contain the text you just typed

If all three match, the bug is fixed.

- [ ] **Step 4: Cross-verify with `gh api`**

```bash
gh api repos/byx-darwin/gitflow-cli/issues/111/comments --jq '.[-1]'
```

The last comment's `id` should match the `data.id` returned by the CLI.

- [ ] **Step 5: Test PR comment**

```bash
# Find an open PR number
gh pr list --state open --json number --jq '.[0].number'

# If no open PR, use a recently closed one for testing
./target/debug/gitflow-cli pr comment <PR_NUMBER> --body "E2E verification: PR comment fix #111 $(date +%s)"
```

Verify same three checks as Step 3.

- [ ] **Step 6: Commit (if any cleanup needed)**

No commit needed for manual E2E testing. If you created test comments on real issues, note the comment IDs for cleanup if desired.

---

## Self-Review Checklist

1. **Spec coverage:** ✅
   - Issue `comment()` fix → Task 1
   - PR `comment()` fix → Task 2
   - Unit tests → Tasks 1 & 2
   - E2E test → Task 4
   - Clippy + full tests → Task 3

2. **Placeholder scan:** ✅ — No TBD/TODO/placeholders

3. **Type consistency:** ✅
   - `GitHubCommentApiResponse` used consistently
   - `CommentData` return type unchanged
   - `MockCommandRunner` usage consistent with existing tests
