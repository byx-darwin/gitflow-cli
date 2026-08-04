# Issue #111: Fix GitHub Comment Stale ID Bug

**Date**: 2026-08-04
**Issue**: [#111](https://github.com/byx-darwin/gitflow-cli/issues/111)
**Status**: Approved
**Related**: #113 (v1.0.0 release blocker)

## Problem Statement

`gf issue comment <n> --body ...` 成功创建评论后，返回的 `data.id` 和 `data.createdAt` 是该 Issue 的**第一条评论**（最旧），而非刚创建的新评论。

同样的 bug 也存在于 `gf pr comment` 命令。

## Root Cause Analysis

### Current Implementation (Buggy)

**Location**:
- `crates/github/src/issue.rs:351-406` — `comment()` method
- `crates/github/src/pr.rs:255-310` — `comment()` method

**Flow**:
```
1. gh issue comment <number> --body "..."  (创建评论，不返回 JSON)
2. gh api repos/{repo}/issues/{number}/comments?per_page=1  (获取评论列表)
3. comments.into_iter().next()  (取第一个元素)
```

**Bug**:
- `per_page=1` 返回第一页的 1 条记录 = **最旧的评论**
- 代码注释写"取最后一个"但实际用了 `.next()` 取第一个
- 结果：返回 Issue 的第一条评论，而非刚创建的评论

### Comparison with GitLab (Correct)

GitLab 实现直接使用 `glab issue note --output json`，该命令返回刚创建的评论对象：

```rust
let output = self.runner.run("glab", &[
    "issue", "note", &number_str,
    "--repo", &self.repo,
    "--body", body,
    "--output", "json",  // 直接返回创建的评论
]);
let api_response: CommentApiResponse = serde_json::from_slice(&output.stdout)?;
```

## Solution Design

### Approach: Use `gh api` POST Directly

Instead of two-step process (create via `gh issue comment` + fetch via `gh api` GET), use single `gh api` POST that creates and returns the comment in one call.

**Rationale**:
1. **Consistency**: Matches pattern used by other operations (`close`/`reopen` use `gh api`)
2. **Efficiency**: One API call instead of two
3. **Correctness**: POST response directly returns created object, no ambiguity
4. **Testability**: Single API call easier to mock

### Implementation

#### GitHub Issue Comment

**Before**:
```rust
async fn comment(&self, number: u64, body: &str) -> Result<CommentData> {
    // Step 1: Create comment (no JSON output)
    let output = self.runner.run("gh", &[
        "issue", "comment", &number_str,
        "--repo", &self.repo,
        "--body", body,
    ]).await?;

    // Step 2: Fetch comments (BUG: gets oldest)
    let api_path = format!("repos/{repo}/issues/{number}/comments?per_page=1", ...);
    let api_output = self.runner.run("gh", &["api", &api_path]).await?;

    // Step 3: Parse and take first (WRONG!)
    let comments: Vec<GitHubCommentApiResponse> = serde_json::from_slice(&api_output.stdout)?;
    let comment = comments.into_iter().next().ok_or(...)?;

    Ok(comment.into())
}
```

**After**:
```rust
async fn comment(&self, number: u64, body: &str) -> Result<CommentData> {
    debug!(repo = %self.repo, number, "spawning `gh api` POST to create comment");

    // Single API call: POST to create comment, returns created object
    let api_path = format!(
        "repos/{repo}/issues/{number}/comments",
        repo = self.repo,
        number = number
    );

    let output = self.runner.run(
        "gh",
        &[
            "api", &api_path,
            "-X", "POST",
            "-f", &format!("body={body}"),
        ],
    ).await.map_err(|e| CoreError::Platform(format!("Failed to spawn gh api: {e}")))?;

    if !output.status.success() {
        return Err(parse_gh_error(&output.stderr).into());
    }

    // Parse the created comment directly from POST response
    let comment: GitHubCommentApiResponse =
        serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

    Ok(comment.into())
}
```

#### GitHub PR Comment

Same pattern for `crates/github/src/pr.rs:255-310`. Reuses `GitHubCommentApiResponse` from issue module.

### API Contract

**Request**:
```bash
gh api repos/{owner}/{repo}/issues/{number}/comments \
  -X POST \
  -f body="{comment_body}"
```

**Response** (GitHub REST API standard shape):
```json
{
  "id": 1234567890,
  "body": "comment text",
  "user": {
    "login": "username",
    "id": 12345
  },
  "created_at": "2026-08-04T00:00:00Z",
  "html_url": "https://github.com/owner/repo/issues/1#issuecomment-1234567890",
  ...
}
```

This matches existing `GitHubCommentApiResponse` structure:
```rust
pub struct GitHubCommentApiResponse {
    pub id: u64,
    pub body: String,
    pub user: GitHubUser,
    pub created_at: String,
}
```

### Error Handling

Maintain existing error patterns:
- `gh api` failure → `parse_gh_error(&output.stderr)` → `CoreError::Platform`
- JSON parse failure → `CoreError::Serialization`
- Empty response → `CoreError::Platform("No comment returned from gh api")`

## Testing Strategy

### Unit Tests

Add test in `crates/github/src/issue.rs`:

```rust
#[test]
fn test_should_create_comment_via_gh_api_post() {
    // Mock gh api POST response
    let mock_response = br#"{
        "id": 5141132300,
        "body": "New comment",
        "user": {"login": "testuser", "id": 123},
        "created_at": "2026-08-04T00:00:00Z"
    }"#;

    // Verify:
    // 1. gh api called with correct POST path
    // 2. Response parsed correctly
    // 3. Returned CommentData has correct id (not stale)
}
```

### Integration Tests

E2E test workflow:
1. Create comment via `gf issue comment`
2. Verify returned `id` matches actual new comment (via `gh api` query)
3. Verify `id` does NOT match first comment

### Regression Prevention

The bug was discovered in workflow `wf-2026-07-31-003` Phase 1 when `issue-review` step created second comment but got first comment's id. After fix:
- Workflow contract `evidence.comment_id` will be correct
- `issue-review` step can reference correct comment

## Impact Analysis

### Affected Commands
- `gf issue comment <number> --body <text>`
- `gf pr comment <number> --body <text>`

### Affected Workflows
- `gf-workflow` Phase 1 `issue-review` step (creates review comments)
- Any automation relying on returned `comment_id`

### Backward Compatibility
- API contract unchanged (still returns `CommentData`)
- Only the implementation changes (bug fix)
- No breaking changes for users

## Delivery Criteria

- [ ] `crates/github/src/issue.rs` `comment()` uses `gh api POST`
- [ ] `crates/github/src/pr.rs` `comment()` uses `gh api POST`
- [ ] Unit tests pass for both methods
- [ ] E2E test: create comment, verify returned id is correct
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `cargo test` passes

## Related Issues

- **#113** (v1.0.0 release): This bug blocks release. Issue #113 explicitly states "建议先修复 #111"
- **#96** (workflow skill): Bug discovered during Phase 1 of workflow `wf-2026-07-31-003`

## References

- GitHub REST API: [Create an issue comment](https://docs.github.com/en/rest/issues/comments?apiVersion=2022-11-28#create-an-issue-comment)
- GitHub REST API: [Create a commit comment](https://docs.github.com/en/rest/commits/comments?apiVersion=2022-11-28#create-a-commit-comment) (PR comments use same endpoint)
