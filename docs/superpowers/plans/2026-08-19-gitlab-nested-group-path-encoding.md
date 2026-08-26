# GitLab 嵌套 group 路径编码修复 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix the GitLab nested-group repo path `%2F` encoding bug by reusing the existing `encode_project_path` helper at all 5 hand-built `glab api` path sites, so `gf issue comment/comments`, MR notes, review notes, and pipeline jobs work on 3+-segment paths like `group/subgroup/project`.

**Architecture:** The shared helper `encode_project_path(repo: &str) -> String` already exists at `crates/gitlab/src/commit.rs:190` (`pub(crate)`, implementation `repo.replace('/', "%2F")`). The bug is that 5 sites hand-roll `split_once('/')` + partial `%2F` encoding, which only encodes the first `/`. Each site is replaced with `encode_project_path(&self.repo)`, producing the fully-encoded single URL segment GitLab requires.

**Tech Stack:** Rust 2024, `gitflow-gitlab` crate, `glab` CLI, `tokio`, `serde`, `MockCommandRunner`/`SequencedMockCommandRunner` test doubles.

**Spec:** `docs/superpowers/specs/2026-08-19-gitlab-nested-group-path-encoding-design.md`

## Global Constraints

- Rust 2024 edition, pinned toolchain (`rust-toolchain.toml`).
- No `unwrap()` / `expect()` in production code.
- Public items need doc comments; `encode_project_path` stays `pub(crate)`.
- Pass `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`.
- Verification: `cargo test -p gitflow-gitlab`, `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`, `cargo +nightly fmt --check`.
- No changes to `deny.toml`, `.pre-commit-config.yaml`, or `rust-toolchain.toml`.

---

### Task 1: Fix `issue.rs` — `comment()` and `list_comments()` (reported bug)

**Files:**
- Modify: `crates/gitlab/src/issue.rs:21-24` (import block)
- Modify: `crates/gitlab/src/issue.rs:439-468` (`comment()`)
- Modify: `crates/gitlab/src/issue.rs:479-504` (`list_comments()`)
- Test: `crates/gitlab/src/issue.rs` (in `mod tests`)

**Interfaces:**
- Consumes: `crate::commit::encode_project_path(&self.repo) -> String` (exists, `pub(crate)`)
- Produces: `glab api` paths of form `/projects/<repo-with-%-2F>/issues/<n>/notes` where the repo is fully encoded (`group/subgroup/project` → `group%2Fsubgroup%2Fproject`)

- [ ] **Step 1: Write the failing tests (RED)**

Append inside the existing `mod tests` in `crates/gitlab/src/issue.rs`:

```rust
#[tokio::test]
async fn test_should_encode_nested_group_repo_path_for_comment() {
    let runner = MockCommandRunner::success(
        r#"{"id":77,"body":"hello","author":{"username":"alice","id":1},"created_at":"2026-08-18T00:00:00Z"}"#,
    );
    let provider = GitLabIssueProvider::with_runner("group/subgroup/project", runner.clone());

    let comment = provider.comment(42, "hello").await.expect("should post");

    assert_eq!(comment.id, 77);
    assert_eq!(
        runner.recorded_calls()[0].1,
        vec![
            "api",
            "--method",
            "POST",
            "/projects/group%2Fsubgroup%2Fproject/issues/42/notes",
            "-f",
            "body=hello",
        ]
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn test_should_encode_nested_group_repo_path_for_list_comments() {
    let runner = MockCommandRunner::success(
        r#"[{"id":77,"body":"hello","author":{"username":"alice","id":1},"created_at":"2026-08-18T00:00:00Z"}]"#,
    );
    let provider = GitLabIssueProvider::with_runner("group/subgroup/project", runner.clone());

    let comments = provider.list_comments(42).await.expect("should list");

    assert_eq!(comments.len(), 1);
    assert_eq!(
        runner.recorded_calls()[0].1,
        vec!["api", "/projects/group%2Fsubgroup%2Fproject/issues/42/notes"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>()
    );
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p gitflow-gitlab test_should_encode_nested_group_repo_path_for_comment test_should_encode_nested_group_repo_path_for_list_comments`
Expected: FAIL — actual argv contains `/projects/group%2Fsubgroup/project/issues/42/notes` (inner `/` not encoded)

- [ ] **Step 3: Implement the minimal fix**

In `crates/gitlab/src/issue.rs` import block (currently `use crate::{ error::parse_glab_error, runner::{CommandRunner, RealCommandRunner}, };`), add the `commit` import:

```rust
use crate::{
    commit::encode_project_path,
    error::parse_glab_error,
    runner::{CommandRunner, RealCommandRunner},
};
```

Replace the body of `comment()` (remove the `split_once` validation + partial encoding):

```rust
async fn comment(&self, number: u64, body: &str) -> Result<CommentData> {
    debug!(repo = %self.repo, number, "spawning `glab api` POST issue note");

    let encoded_path = encode_project_path(&self.repo);
    let api_path = format!("/projects/{encoded_path}/issues/{number}/notes");
    let body_arg = format!("body={body}");
    // ... rest unchanged
}
```

Replace the body of `list_comments()` likewise:

```rust
async fn list_comments(&self, number: u64) -> Result<Vec<CommentData>> {
    debug!(repo = %self.repo, number, "spawning `glab api` GET issue notes");

    let encoded_path = encode_project_path(&self.repo);
    let api_path = format!("/projects/{encoded_path}/issues/{number}/notes");
    // ... rest unchanged
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p gitflow-gitlab test_should_encode_nested_group_repo_path_for_comment test_should_encode_nested_group_repo_path_for_list_comments`
Expected: PASS

- [ ] **Step 5: Run full issue module tests**

Run: `cargo test -p gitflow-gitlab issue::`
Expected: all pass (existing 2-segment tests like `test_should_post_issue_note_via_glab_api_with_message_field` still assert `/projects/owner%2Frepo/...`)

- [ ] **Step 6: Commit**

```bash
git add crates/gitlab/src/issue.rs
git commit -m "fix(gitlab): issue comment/comments 嵌套 group 项目路径全量 %2F 编码"
```

---

### Task 2: Fix `mr.rs` — `comment()` MR note path

**Files:**
- Modify: `crates/gitlab/src/mr.rs:372-402` (`comment()`)
- Test: `crates/gitlab/src/mr.rs` (in `mod tests`)

**Interfaces:**
- Consumes: `crate::commit::encode_project_path` (already imported at `mr.rs:22`)
- Produces: fully-encoded `/projects/<repo-with-%-2F>/merge_requests/<n>/notes`

- [ ] **Step 1: Write the failing test (RED)**

Append inside the existing `mod tests` in `crates/gitlab/src/mr.rs`:

```rust
#[tokio::test]
async fn test_should_encode_nested_group_repo_path_for_mr_note() {
    let runner = MockCommandRunner::success(
        r#"{"id":88,"body":"lgtm","author":{"username":"bob","id":2},"created_at":"2026-08-18T00:00:00Z"}"#,
    );
    let provider = GitLabMrProvider::with_runner("group/subgroup/project", runner.clone());

    let comment = provider.comment(7, "lgtm").await.expect("should post");

    assert_eq!(comment.id, 88);
    assert_eq!(
        runner.recorded_calls()[0].1,
        vec![
            "api",
            "--method",
            "POST",
            "/projects/group%2Fsubgroup%2Fproject/merge_requests/7/notes",
            "-f",
            "body=lgtm",
        ]
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>()
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-gitlab test_should_encode_nested_group_repo_path_for_mr_note`
Expected: FAIL — argv has `/projects/group%2Fsubgroup/project/merge_requests/7/notes`

- [ ] **Step 3: Implement the minimal fix**

In `crates/gitlab/src/mr.rs` `comment()` (currently has `let (owner, project) = self.repo.split_once('/')...;` + `format!("/projects/{owner}%2F{project}/merge_requests/{number}/notes")`):

```rust
let encoded_path = encode_project_path(&self.repo);
let api_path = format!("/projects/{encoded_path}/merge_requests/{number}/notes");
let body_arg = format!("body={body}");
```

Delete the `split_once` block. (`encode_project_path` is already imported at `mr.rs:22`.)

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p gitflow-gitlab test_should_encode_nested_group_repo_path_for_mr_note`
Expected: PASS

- [ ] **Step 5: Run full mr module tests**

Run: `cargo test -p gitflow-gitlab mr::`
Expected: all pass

- [ ] **Step 6: Commit**

```bash
git add crates/gitlab/src/mr.rs
git commit -m "fix(gitlab): mr note 嵌套 group 项目路径全量 %2F 编码"
```

---

### Task 3: Fix `review.rs` — `post_note()` MR review note path

**Files:**
- Modify: `crates/gitlab/src/review.rs:25-28` (import block)
- Modify: `crates/gitlab/src/review.rs:240-260` (`post_note()`)
- Test: `crates/gitlab/src/review.rs` (in `mod tests`)

**Interfaces:**
- Consumes: `crate::commit::encode_project_path` (new import)
- Produces: fully-encoded `/projects/<repo-with-%-2F>/merge_requests/<n>/notes`

- [ ] **Step 1: Write the failing test (RED)**

Append inside the existing `mod tests` in `crates/gitlab/src/review.rs`:

```rust
#[tokio::test]
async fn test_should_encode_nested_group_repo_path_for_review_note() {
    let runner = MockCommandRunner::success(
        r#"{"id":99,"body":"fix this","author":{"username":"alice","id":1},"created_at":"2026-08-18T00:00:00Z"}"#,
    );
    let provider = GitLabReviewProvider::with_runner("group/subgroup/project", runner.clone());

    let review = provider.comment(7, "fix this").await.expect("should post");

    assert_eq!(review.id, 99);
    assert_eq!(
        runner.recorded_calls()[0].1,
        vec![
            "api",
            "--method",
            "POST",
            "/projects/group%2Fsubgroup%2Fproject/merge_requests/7/notes",
            "-f",
            "body=fix this",
        ]
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>()
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-gitlab test_should_encode_nested_group_repo_path_for_review_note`
Expected: FAIL — argv has `/projects/group%2Fsubgroup/project/merge_requests/7/notes`

- [ ] **Step 3: Implement the minimal fix**

In `crates/gitlab/src/review.rs` import block (currently `use crate::{ error::parse_glab_error, runner::{CommandRunner, RealCommandRunner}, };`), add:

```rust
use crate::{
    commit::encode_project_path,
    error::parse_glab_error,
    runner::{CommandRunner, RealCommandRunner},
};
```

In `post_note()` (currently has `let (owner, project) = self.repo.split_once('/')...;` + `format!("/projects/{owner}%2F{project}/merge_requests/{pr_number}/notes")`):

```rust
let encoded_path = encode_project_path(&self.repo);
let api_path = format!("/projects/{encoded_path}/merge_requests/{pr_number}/notes");
let body_arg = format!("body={body}");
```

Delete the `split_once` block.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p gitflow-gitlab test_should_encode_nested_group_repo_path_for_review_note`
Expected: PASS

- [ ] **Step 5: Run full review module tests**

Run: `cargo test -p gitflow-gitlab review::`
Expected: all pass

- [ ] **Step 6: Commit**

```bash
git add crates/gitlab/src/review.rs
git commit -m "fix(gitlab): review note 嵌套 group 项目路径全量 %2F 编码"
```

---

### Task 4: Fix `pipeline.rs` — `jobs()` pipeline jobs path

**Files:**
- Modify: `crates/gitlab/src/pipeline.rs:23-27` (import block)
- Modify: `crates/gitlab/src/pipeline.rs:251-267` (`jobs()`)
- Test: `crates/gitlab/src/pipeline.rs` (in `mod tests`)

**Interfaces:**
- Consumes: `crate::commit::encode_project_path` (new import)
- Produces: fully-encoded `/projects/<repo-with-%-2F>/pipelines/<id>/jobs`

- [ ] **Step 1: Write the failing test (RED)**

Append inside the existing `mod tests` in `crates/gitlab/src/pipeline.rs`:

```rust
#[tokio::test]
async fn test_should_encode_nested_group_repo_path_for_pipeline_jobs() {
    let runner = MockCommandRunner::success(
        r#"[{"id":1,"name":"build","status":"success"},{"id":2,"name":"test","status":"running"}]"#,
    );
    let provider = GitLabPipelineProvider::with_runner("group/subgroup/project", runner.clone());

    let jobs = provider.jobs(5).await.expect("should fetch");

    assert_eq!(jobs.len(), 2);
    assert_eq!(
        runner.recorded_calls()[0].1,
        vec!["api", "/projects/group%2Fsubgroup%2Fproject/pipelines/5/jobs"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>()
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-gitlab test_should_encode_nested_group_repo_path_for_pipeline_jobs`
Expected: FAIL — argv has `/projects/group%2Fsubgroup/project/pipelines/5/jobs`

- [ ] **Step 3: Implement the minimal fix**

In `crates/gitlab/src/pipeline.rs` import block (currently `use crate::{ error::parse_glab_error, runner::{CommandRunner, RealCommandRunner}, };`), add:

```rust
use crate::{
    commit::encode_project_path,
    error::parse_glab_error,
    runner::{CommandRunner, RealCommandRunner},
};
```

In `jobs()` (currently has `let (owner, project) = self.repo.split_once('/')...;` + `format!("/projects/{owner}%2F{project}/pipelines/{pipeline_id}/jobs")`):

```rust
let encoded_path = encode_project_path(&self.repo);
let api_path = format!("/projects/{encoded_path}/pipelines/{pipeline_id}/jobs");
```

Delete the `split_once` block.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p gitflow-gitlab test_should_encode_nested_group_repo_path_for_pipeline_jobs`
Expected: PASS

- [ ] **Step 5: Run full pipeline module tests**

Run: `cargo test -p gitflow-gitlab pipeline::`
Expected: all pass

- [ ] **Step 6: Commit**

```bash
git add crates/gitlab/src/pipeline.rs
git commit -m "fix(gitlab): pipeline jobs 嵌套 group 项目路径全量 %2F 编码"
```

---

### Task 5: Full verification gate

**Files:**
- None (verification only)

- [ ] **Step 1: Full crate test suite**

Run: `cargo test -p gitflow-gitlab`
Expected: all pass

- [ ] **Step 2: Workspace clippy (pedantic)**

Run: `cargo clippy -p gitflow-gitlab --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: clean

- [ ] **Step 3: Format check**

Run: `cargo +nightly fmt --check`
Expected: clean (run `cargo +nightly fmt` if not)

- [ ] **Step 4: Workspace test (regression safety)**

Run: `make test`
Expected: all pass

- [ ] **Step 5: Commit any format-only changes**

```bash
git add -A
git commit -m "style: fmt"  # only if Step 3 produced changes
```
