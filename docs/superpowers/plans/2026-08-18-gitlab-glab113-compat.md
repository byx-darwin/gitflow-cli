# GitLab glab 1.113 Compatibility Fix — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让 gf 1.3.0 在 glab 1.113.0 下 GitLab 写操作全部恢复正常（issue/label/mr/milestone/release/note/auth/exit-code）。

**Architecture:** 按「写操作不追加 `--output json`、读操作保留」的原则逐文件修复 `crates/gitlab` 的 glab 调用层；写操作需要返回实体时改用现有 `view`/`list` 重新拉取，评论类改用 `glab api`（返回 JSON、规避 `--output`/`--message` 双问题）；用可注入 `CommandRunner` 统一 label/milestone/review 三个 provider（当前硬编码 `tokio::process::Command`），并给 `MockCommandRunner` 增加参数记录能力以断言精确的 glab 参数序列。

**Tech Stack:** Rust 2024 workspace · tokio · serde · 现有 `CommandRunner` trait（`crates/gitlab/src/runner.rs`）· 现有 `MockCommandRunner` / `SequencedMockCommandRunner`。

**Spec:** `docs/superpowers/specs/2026-08-18-gitlab-glab113-compat-design.md`（本计划唯一论证依据，执行者须先读该文档）。Issue: [#199](https://github.com/byx-darwin/gitflow-cli/issues/199)。

## Global Constraints

- **glab 1.113 写命令无 `--output` flag**（实测）：`issue close/reopen/note`、`mr create/close/reopen/note`、`label create/edit/delete`、`milestone create/edit/close/reopen`、`release create/edit`、`ci view`/`ci trace`。
- **glab 1.113 读命令有 `--output` flag**（实测）：`issue list/view`、`mr list/view`、`label list`、`milestone list`、`ci list`、`release list/view`。
- **回归基线不得破坏**：`issue list/view/comments`、`commit` 系列、`label/milestone/release list`、`pipeline status/report`、`workflow` 全套、`doctor`、`auth status`。
- 不新增依赖；不改 `deny.toml`/`.pre-commit-config.yaml`/`rust-toolchain.toml`；不引入 `unwrap()`/`expect()`。
- 每次提交用 conventional commit 前缀（`fix(gitlab):`）。
- 所有测试用 `#[tokio::test]` + `MockCommandRunner`/`SequencedMockCommandRunner`，遵循现有命名 `test_should_*`。
- 不实现 `gf repo` 子命令（推迟，另行 issue）。

---

### Task 1: 测试基础设施 — MockCommandRunner 增加参数记录

**Files:**
- Modify: `crates/gitlab/src/runner.rs`

**Interfaces:**
- Consumes: 无。
- Produces: `MockCommandRunner::recorded_calls` 与 `SequencedMockCommandRunner::recorded_calls` 均返回 `Vec<(String, Vec<String>)>` — 每次 `run`/`run_with_stdin` 调用记录的 `(program, args)` 序列，供后续所有任务断言精确 glab 参数（含顺序 runner 场景）。

- [ ] **Step 1: Write the failing test**

在 `runner.rs` 的 `mod tests` 中追加（`SequencedMockCommandRunner` 同法补一条）：

```rust
#[tokio::test]
async fn test_should_record_glab_calls() {
    let runner = MockCommandRunner::success("ok");
    runner
        .run("glab", &["issue", "close", "42", "--repo", "owner/repo"])
        .await
        .expect("should succeed");
    let calls = runner.recorded_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "glab");
    assert_eq!(
        calls[0].1,
        vec!["issue", "close", "42", "--repo", "owner/repo"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>()
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-gitlab runner::tests::test_should_record_glab_calls`
Expected: FAIL — `recorded_calls` 不存在。

- [ ] **Step 3: Implement**

`MockCommandRunner` 增加字段（保持 `#[derive(Debug, Clone)]` 兼容）：

```rust
pub struct MockCommandRunner {
    result: MockResult,
    recorded: Arc<std::sync::Mutex<Vec<(String, Vec<String>)>>>,
}
```

在 `success`/`failure`/`spawn_error` 三个构造函数中初始化 `recorded: Arc::new(std::sync::Mutex::new(Vec::new()))`。在 `run` 与 `run_with_stdin` 中记录：

```rust
async fn run(&self, program: &str, args: &[&str]) -> std::io::Result<CommandOutput> {
    self.recorded.lock().expect("mock mutex poisoned").push((
        program.to_string(),
        args.iter().map(|s| (*s).to_string()).collect(),
    ));
    // ...原有 match 逻辑不变
}
```

新增方法：

```rust
#[must_use]
pub fn recorded_calls(&self) -> Vec<(String, Vec<String>)> {
    self.recorded.lock().expect("mock mutex poisoned").clone()
}
```

`SequencedMockCommandRunner` 同法增加 `recorded: Arc<std::sync::Mutex<Vec<(String, Vec<String>)>>>` 字段（`new`/`from_results` 初始化，`run` 中记录，暴露同名 `recorded_calls`）。文件顶部补 `use std::sync::Arc;`。

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p gitflow-gitlab runner`
Expected: PASS（含既有 runner 测试）。

- [ ] **Step 5: Commit**

```bash
git add crates/gitlab/src/runner.rs
git commit -m "test(gitlab): record glab args in MockCommandRunner"
```

---

### Task 2: label / milestone / review 三个 provider 改为可注入 runner

**Files:**
- Modify: `crates/gitlab/src/label.rs`
- Modify: `crates/gitlab/src/review.rs`

**Interfaces:**
- Consumes: `crate::runner::{CommandRunner, RealCommandRunner}`。
- Produces: 三个泛型 struct 的 `with_runner(repo, runner)` 构造器；行为与 `new(repo)` 完全一致（`R: CommandRunner = RealCommandRunner` 默认参数使 `apps/cli` 既有调用点零改动）。

- [ ] **Step 1: Write the failing test**（label.rs `mod tests` 追加）

```rust
#[tokio::test]
async fn test_should_fail_when_label_create_glab_fails() {
    let runner = crate::runner::MockCommandRunner::failure(r#"{"message": "Forbidden"}"#, 256);
    let provider = GitLabLabelProvider::with_runner("owner/repo", runner);
    let args = CreateLabelArgs {
        name: "bug".to_string(),
        color: "#d73a4a".to_string(),
        description: None,
    };
    let result = provider.create(args).await;
    assert!(result.is_err());
}
```

同法补 `GitLabMilestoneProvider::with_runner` 与 `GitLabReviewProvider::with_runner` 的失败路径测试。

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-gitlab label review`
Expected: FAIL — `with_runner` 不存在。

- [ ] **Step 3: Implement**（label.rs）

```rust
use crate::runner::{CommandRunner, RealCommandRunner};

#[derive(Debug, Clone)]
pub struct GitLabLabelProvider<R: CommandRunner = RealCommandRunner> {
    repo: String,
    runner: R,
}

impl GitLabLabelProvider {
    #[must_use]
    pub fn new(repo: impl Into<String>) -> GitLabLabelProvider<RealCommandRunner> {
        GitLabLabelProvider { repo: repo.into(), runner: RealCommandRunner }
    }
}

impl<R: CommandRunner> GitLabLabelProvider<R> {
    #[must_use]
    pub fn with_runner(repo: impl Into<String>, runner: R) -> Self {
        Self { repo: repo.into(), runner }
    }
}
```

将 `create`/`list`/`edit`/`delete` 与 `GitLabMilestoneProvider` 全部方法中的 `tokio::process::Command::new("glab").args(...)` 链式调用改写为 `self.runner.run("glab", &[...])`（参照 `issue.rs` 的调用形态），并在 `impl<R: CommandRunner>` 上实现 trait（`impl<R: CommandRunner> LabelProvider for GitLabLabelProvider<R>`），`#[async_trait]` 不变。`new` 返回类型改为具体默认泛型。同理改写 `review.rs` 的 `GitLabReviewProvider`（含 `comment`/`get_current_user`）。

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p gitflow-gitlab label review`
Expected: PASS；`cargo build -p gitflow-gitlab` 通过（`apps/cli` 调用点 `::new(repo)` 兼容）。

- [ ] **Step 5: Commit**

```bash
git add crates/gitlab/src/label.rs crates/gitlab/src/review.rs
git commit -m "refactor(gitlab): make label/milestone/review providers runner-injectable"
```

---

### Task 3: issue.rs `close`/`reopen` — 去 `--output json`，写后 `view` 重新拉取

**Files:**
- Modify: `crates/gitlab/src/issue.rs`（`close` L381-410、`reopen` L420-449）

**Interfaces:**
- Consumes: `self.view(number) -> Result<IssueData>`。
- Produces: `close/reopen` 不再解析写命令 stdout，改为成功后 `self.view(number)` 返回最新 IssueData。

- [ ] **Step 1: Write the failing test**

更新 `test_should_return_serialization_error_on_invalid_json_for_close`（L986-997）——close 不再直接解析 JSON，改由 view 承担。用记录型 runner 断言参数并验证行为：

```rust
#[tokio::test]
async fn test_should_close_issue_without_output_json_flag_and_refetch_via_view() {
    let runner = MockCommandRunner::success(
        r#"{"iid":42,"title":"Fix","state":"closed","description":null,"labels":[]}"#,
    );
    let provider = GitLabIssueProvider::with_runner("owner/repo", runner);

    let issue = provider.close(42).await.expect("close should succeed");

    assert_eq!(issue.number, 42);
    assert_eq!(issue.state, State::Closed);
    let calls = runner.recorded_calls();
    assert_eq!(
        calls[0].1,
        vec!["issue", "close", "42", "--repo", "owner/repo"]
            .into_iter().map(String::from).collect::<Vec<_>>()
    );
    // 第二次调用是重新拉取 view（带 --output json，读操作保留）
    assert_eq!(calls[1].0, "glab");
    assert!(calls[1].1.contains(&"--output".to_string()));
}
```

`reopen` 同法断言 `["issue", "reopen", ...]`。更新既有失败路径测试为顺序 runner（第一次 `close` 失败 → `Cli` 错误）。

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-gitlab issue::tests`
Expected: FAIL — 参数断言（无 `--output`）与解析行为不符。

- [ ] **Step 3: Implement**

`close`（reopen 同构）：

```rust
async fn close(&self, number: u64) -> Result<IssueData> {
    debug!(repo = %self.repo, number, "spawning `glab issue close`");
    let number_str = number.to_string();
    let output = self
        .runner
        .run("glab", &["issue", "close", &number_str, "--repo", &self.repo])
        .await
        .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;
    if !output.status.success() {
        return Err(parse_glab_error(&output.stderr).into());
    }
    self.view(number).await
}
```

`reopen` 同理，命令改为 `["issue", "reopen", ...]`。同步更新两处 doc comment（不再声称 `--output json`）。

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p gitflow-gitlab issue`
Expected: PASS。

- [ ] **Step 5: Commit**

```bash
git add crates/gitlab/src/issue.rs
git commit -m "fix(gitlab): drop --output json on issue close/reopen, refetch via view"
```

---

### Task 4: issue.rs `comment` — 改用 `glab api` POST 创建 note

**Files:**
- Modify: `crates/gitlab/src/issue.rs`（`comment` L459-490）

**Interfaces:**
- Consumes: `self.repo.split_once('/')`（同 `list_comments` L503 模式）。
- Produces: `comment` 返回 `CommentData`（glab api 直接返回 note JSON，含 author/created_at）。

- [ ] **Step 1: Write the failing test**

```rust
#[tokio::test]
async fn test_should_post_issue_note_via_glab_api_with_message_field() {
    let runner = MockCommandRunner::success(
        r#"{"id":77,"body":"hello","author":{"username":"alice","id":1},"created_at":"2026-08-18T00:00:00Z"}"#,
    );
    let provider = GitLabIssueProvider::with_runner("owner/repo", runner);

    let comment = provider.comment(42, "hello").await.expect("should post");

    assert_eq!(comment.id, 77);
    assert_eq!(comment.author.login, "alice");
    assert_eq!(
        runner.recorded_calls()[0].1,
        vec![
            "api", "--method", "POST",
            "/projects/owner%2Frepo/issues/42/notes",
            "-f", "body=hello",
        ]
        .into_iter().map(String::from).collect::<Vec<_>>()
    );
}
```

更新既有 `test_should_return_serialization_error_on_invalid_json_for_comment` 为上述新参数形态。

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-gitlab issue::tests`
Expected: FAIL — 当前仍走 `issue note --body --output`。

- [ ] **Step 3: Implement**

```rust
async fn comment(&self, number: u64, body: &str) -> Result<CommentData> {
    debug!(repo = %self.repo, number, "spawning `glab api` POST issue note");

    let (owner, project) = self.repo.split_once('/').ok_or_else(|| {
        CoreError::Platform(format!(
            "Invalid repo format '{}', expected 'owner/project'",
            self.repo
        ))
    })?;

    let api_path = format!("/projects/{owner}%2F{project}/issues/{number}/notes");
    let body_arg = format!("body={body}");

    let output = self
        .runner
        .run("glab", &["api", "--method", "POST", &api_path, "-f", &body_arg])
        .await
        .map_err(|e| CoreError::Platform(format!("Failed to spawn glab api: {e}")))?;

    if !output.status.success() {
        return Err(parse_glab_error(&output.stderr).into());
    }

    let api_response: CommentApiResponse =
        serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

    Ok(api_response.into())
}
```

同步更新 doc comment（不再声称 `glab issue note`/`--body`/`--output`）。

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p gitflow-gitlab issue`
Expected: PASS。

- [ ] **Step 5: Commit**

```bash
git add crates/gitlab/src/issue.rs
git commit -m "fix(gitlab): post issue notes via glab api instead of --body/--output"
```

---

### Task 5: issue.rs `parse_issue_iid_from_url` — 兼容 `/work_items/N`

**Files:**
- Modify: `crates/gitlab/src/issue.rs`（`parse_issue_iid_from_url` L672-684）

**Interfaces:**
- Consumes: 无。
- Produces: 同时匹配 `/-/issues/N` 与 `/-/work_items/N` 的 `Option<u64>`。

- [ ] **Step 1: Write the failing test**

在既有 URL 解析测试（L842-870 附近）追加：

```rust
#[test]
fn test_should_parse_work_item_url() {
    assert_eq!(
        parse_issue_iid_from_url(
            "http://192.168.230.23/iproost/iproost-docs/-/work_items/1"
        ),
        Some(1)
    );
}

#[test]
fn test_should_parse_work_item_url_among_lines() {
    let output = "Creating issue...\nhttp://192.168.230.23/iproost/iproost-docs/-/work_items/7\nDone.";
    assert_eq!(parse_issue_iid_from_url(output), Some(7));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-gitlab parse_issue_iid_from_url`
Expected: FAIL — work_items 返回 None。

- [ ] **Step 3: Implement**

```rust
fn parse_issue_iid_from_url(url: &str) -> Option<u64> {
    url.lines().find_map(|line| {
        let line = line.trim();
        for marker in ["/-/issues/", "/-/work_items/"] {
            if line.contains(marker) {
                if let Some(id) = line
                    .rsplit(marker)
                    .next()
                    .and_then(|s| s.split('/').next())
                    .and_then(|s| s.parse().ok())
                {
                    return Some(id);
                }
            }
        }
        None
    })
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p gitflow-gitlab issue`
Expected: PASS（含既有 `/issues/` 用例）。

- [ ] **Step 5: Commit**

```bash
git add crates/gitlab/src/issue.rs
git commit -m "fix(gitlab): parse /-/work_items/N URLs on issue create"
```

---

### Task 6: mr.rs `create`/`close`/`reopen` — 去 `--output json`，写后重新拉取

**Files:**
- Modify: `crates/gitlab/src/mr.rs`（`create` L183-231、`close` L299-329、`reopen` L330-360）

**Interfaces:**
- Consumes: 新增私有 `fn parse_mr_iid_from_url(&str) -> Option<u64>`（模式仿 `parse_issue_iid_from_url`，匹配 `/-/merge_requests/N`）；`self.view(number)`。
- Produces: `create` 成功后解析 MR URL → `self.view`；`close`/`reopen` 成功后 `self.view(number)`。

- [ ] **Step 1: Write the failing test**

追加：

```rust
#[tokio::test]
async fn test_should_create_mr_without_output_json_and_refetch_via_view() {
    let runner = MockCommandRunner::success(
        r#"{"iid":12,"title":"Feat","state":"opened","source_branch":"feat/x","target_branch":"main"}"#,
    );
    let provider = GitLabMrProvider::with_runner("owner/repo", runner);
    let args = CreatePrArgs {
        title: "Feat".to_string(),
        head: "feat/x".to_string(),
        base: "main".to_string(),
        body: None,
        draft: false,
        repo: None,
    };
    let pr = provider.create(args).await.expect("should create");
    assert_eq!(pr.number, 12);
    let calls = runner.recorded_calls();
    assert!(!calls[0].1.contains(&"--output".to_string()));
    // 第二次调用是 mr view（保留 --output json）
    assert!(calls[1].1.contains(&"--output".to_string()));
}
```

`close`/`reopen` 同 Task 3 模式（记录型 runner + 无 `--output` + view 断言）。

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-gitlab mr::tests`
Expected: FAIL。

- [ ] **Step 3: Implement**

`create`：移除 `--output json`；成功后：

```rust
let stdout = String::from_utf8_lossy(&output.stdout);
let mr_iid = parse_mr_iid_from_url(&stdout).ok_or_else(|| {
    CoreError::Platform(format!("Failed to parse MR URL from output: {stdout}"))
})?;
self.view(mr_iid).await
```

新增解析函数（放文件内，`mr.rs` 私有）：

```rust
fn parse_mr_iid_from_url(url: &str) -> Option<u64> {
    url.lines().find_map(|line| {
        let line = line.trim();
        if line.contains("/-/merge_requests/") {
            line.rsplit("/-/merge_requests/")
                .next()
                .and_then(|s| s.split('/').next())
                .and_then(|s| s.parse().ok())
        } else {
            None
        }
    })
}
```

`close`：改为 `["mr", "close", &number_str, "--repo", &self.repo]` → 成功后 `self.view(number).await`。`reopen` 同理。

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p gitflow-gitlab mr`
Expected: PASS。更新受影响的既有 `create` 失败/序列化测试（用顺序 runner 或新 URL 形态）。

- [ ] **Step 5: Commit**

```bash
git add crates/gitlab/src/mr.rs
git commit -m "fix(gitlab): drop --output json on mr create/close/reopen, refetch via view"
```

---

### Task 7: mr.rs `comment` — 改用 `glab api` POST 创建 note

**Files:**
- Modify: `crates/gitlab/src/mr.rs`（`comment` L361-392）

**Interfaces:**
- Consumes: `CommentApiResponse`（L151 已存在）、`self.repo.split_once('/')`。
- Produces: `comment` 返回 `CommentData`。

- [ ] **Step 1: Write the failing test**

```rust
#[tokio::test]
async fn test_should_post_mr_note_via_glab_api() {
    let runner = MockCommandRunner::success(
        r#"{"id":88,"body":"lgtm","author":{"username":"bob","id":2},"created_at":"2026-08-18T00:00:00Z"}"#,
    );
    let provider = GitLabMrProvider::with_runner("owner/repo", runner);

    let comment = provider.comment(7, "lgtm").await.expect("should post");

    assert_eq!(comment.id, 88);
    assert_eq!(
        runner.recorded_calls()[0].1,
        vec![
            "api", "--method", "POST",
            "/projects/owner%2Frepo/merge_requests/7/notes",
            "-f", "body=lgtm",
        ]
        .into_iter().map(String::from).collect::<Vec<_>>()
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-gitlab mr::tests`
Expected: FAIL。

- [ ] **Step 3: Implement**

仿 Task 4，仅路径改为 `/projects/{owner}%2F{project}/merge_requests/{number}/notes`，返回 `CommentData`。

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p gitflow-gitlab mr`
Expected: PASS。

- [ ] **Step 5: Commit**

```bash
git add crates/gitlab/src/mr.rs
git commit -m "fix(gitlab): post mr notes via glab api instead of --body/--output"
```

---

### Task 8: mr.rs `mark_ready`/`mark_wip` — 改用 `mr update --draft=false/true`

**Files:**
- Modify: `crates/gitlab/src/mr.rs`（`mark_ready` L444-459、`mark_wip` L461-476）

**Interfaces:**
- Consumes: `self.view(number)`（保持不变）。
- Produces: `mark_ready` 调用 `glab mr update <n> --draft=false`；`mark_wip` 调用 `--draft=true`。

- [ ] **Step 1: Write the failing test**

> 实现拆分为私有 `async fn run_mr_update(&self, number, draft: bool) -> Result<()>`（见 Step 3），`mark_ready`/`mark_wip` 在其成功后调用 `view`。测试直接对 `run_mr_update` 断言精确参数：

```rust
#[tokio::test]
async fn test_should_mark_ready_with_mr_update_draft_false() {
    let runner = MockCommandRunner::success("");
    let provider = GitLabMrProvider::with_runner("owner/repo", runner);
    provider.run_mr_update(5, false).await.expect("should succeed");
    assert_eq!(
        runner.recorded_calls()[0].1,
        vec!["mr", "update", "5", "--repo", "owner/repo", "--draft=false"]
            .into_iter().map(String::from).collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn test_should_mark_wip_with_mr_update_draft_true() {
    let runner = MockCommandRunner::success("");
    let provider = GitLabMrProvider::with_runner("owner/repo", runner);
    provider.run_mr_update(5, true).await.expect("should succeed");
    assert_eq!(
        runner.recorded_calls()[0].1,
        vec!["mr", "update", "5", "--repo", "owner/repo", "--draft=true"]
            .into_iter().map(String::from).collect::<Vec<_>>()
    );
}
```

并各补一条失败路径：`MockCommandRunner::failure(..., 256)` → `run_mr_update` 返回 `Cli` 错误。

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-gitlab mr::tests`
Expected: FAIL — 当前仍走 `mr ready`/`mr draft`。

- [ ] **Step 3: Implement**

```rust
async fn run_mr_update(&self, number: u64, draft: bool) -> Result<()> {
    let number_str = number.to_string();
    let draft_flag = if draft { "--draft=true" } else { "--draft=false" };
    let output = self
        .runner
        .run("glab", &["mr", "update", &number_str, "--repo", &self.repo, draft_flag])
        .await
        .map_err(|e| CoreError::Platform(format!("Failed to spawn glab mr update: {e}")))?;
    if !output.status.success() {
        return Err(parse_glab_error(&output.stderr).into());
    }
    Ok(())
}

async fn mark_ready(&self, number: u64) -> Result<PrData> {
    debug!(repo = %self.repo, number, "spawning `glab mr update --draft=false`");
    self.run_mr_update(number, false).await?;
    self.view(number).await
}

async fn mark_wip(&self, number: u64) -> Result<PrData> {
    debug!(repo = %self.repo, number, "spawning `glab mr update --draft=true`");
    self.run_mr_update(number, true).await?;
    self.view(number).await
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p gitflow-gitlab mr`
Expected: PASS。

- [ ] **Step 5: Commit**

```bash
git add crates/gitlab/src/mr.rs
git commit -m "fix(gitlab): use mr update --draft for ready/wip"
```

---

### Task 9: mr.rs `merge` — 移除 `--merge`

**Files:**
- Modify: `crates/gitlab/src/mr.rs`（`merge` L394-422）

**Interfaces:**
- Consumes: 无。
- Produces: `MergeStrategy::Merge` 或 `None` 时不追加任何 strategy flag；`Squash`→`--squash`、`Rebase`→`--rebase` 保留。

- [ ] **Step 1: Write the failing test**

```rust
#[tokio::test]
async fn test_should_merge_without_merge_flag() {
    let runner = MockCommandRunner::success("Merged!");
    let provider = GitLabMrProvider::with_runner("owner/repo", runner);

    let _ = provider.merge(9, Some(MergeStrategy::Merge)).await.expect("should merge");

    assert_eq!(
        runner.recorded_calls()[0].1,
        vec!["mr", "merge", "9", "--repo", "owner/repo"]
            .into_iter().map(String::from).collect::<Vec<_>>()
    );
}
```

同法补 `None` 分支用例（也不带 flag）。

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-gitlab mr::tests`
Expected: FAIL — 当前包含 `--merge`。

- [ ] **Step 3: Implement**

```rust
match strategy {
    Some(MergeStrategy::Squash) => cmd_args.push("--squash"),
    Some(MergeStrategy::Rebase) => cmd_args.push("--rebase"),
    Some(MergeStrategy::Merge) | None => {}
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p gitflow-gitlab mr`
Expected: PASS。

- [ ] **Step 5: Commit**

```bash
git add crates/gitlab/src/mr.rs
git commit -m "fix(gitlab): drop unknown --merge flag on mr merge"
```

---

### Task 10: label.rs label `create`/`edit`/`delete`

**Files:**
- Modify: `crates/gitlab/src/label.rs`（`create` L69-105、`list` L107-128、`edit` L130-161、`delete` L163-181；`LabelApiResponse` L48-55）

**Interfaces:**
- Consumes: 新增私有 `async fn list_api(&self) -> Result<Vec<LabelApiResponse>>`（`list()` 改为调用它并映射 `LabelData`）；`LabelApiResponse` 增加 `#[serde(default)] id: u64`。
- Produces: `edit` 先用 `list_api` 解析 label-id，再 `glab label edit --label-id <id> --new-name <name> --repo R --color C [--description D]`；`create` 写后经 `list` 按 name 找回；`delete` 去 `--yes`。

- [ ] **Step 1: Write the failing test**

```rust
#[tokio::test]
async fn test_should_edit_label_with_label_id() {
    let list_json = r#"[{"id":101,"name":"bug","color":"#d73a4a"}]"#;
    let edited_json = r#"{"id":101,"name":"critical","color":"#d73a4a"}"#;
    let runner = SequencedMockCommandRunner::from_results(&[
        (true, list_json),   // list_api 解析 id
        (true, edited_json), // label edit 成功（stdout 为纯文本，忽略）
        (true, list_json),   // 再次 list 找回
    ]);
    let provider = GitLabLabelProvider::with_runner("owner/repo", runner);
    let args = CreateLabelArgs {
        name: "critical".to_string(),
        color: "#d73a4a".to_string(),
        description: None,
    };

    let label = provider.edit("bug", args).await.expect("should edit");

    assert_eq!(label.name, "critical");
}
```

`delete` 用 `recorded_calls` 断言无 `--yes`：

```rust
#[tokio::test]
async fn test_should_delete_label_without_yes_flag() {
    let runner = MockCommandRunner::success("");
    let provider = GitLabLabelProvider::with_runner("owner/repo", runner);
    provider.delete("bug").await.expect("should delete");
    assert_eq!(
        runner.recorded_calls()[0].1,
        vec!["label", "delete", "bug", "--repo", "owner/repo"]
            .into_iter().map(String::from).collect::<Vec<_>>()
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-gitlab label::tests`
Expected: FAIL。

- [ ] **Step 3: Implement**

`LabelApiResponse` 增加 `#[serde(default)] id: u64`。新增：

```rust
async fn list_api(&self) -> Result<Vec<LabelApiResponse>> {
    let output = self
        .runner
        .run("glab", &["label", "list", "--repo", &self.repo, "--output", "json"])
        .await
        .map_err(|e| CoreError::Platform(format!("Failed to spawn glab label list: {e}")))?;
    if !output.status.success() {
        return Err(parse_glab_error(&output.stderr).into());
    }
    serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)
}
```

`list()` 改为 `self.list_api().await.map(|v| v.into_iter().map(LabelData::from).collect())`。

`create`：移除 `--output json`，成功后 `let labels = self.list().await?; labels.into_iter().find(|l| l.name == args.name).ok_or_else(|| CoreError::Platform(format!("Label '{}' not found after create", args.name)))`。

`edit`：

```rust
async fn edit(&self, name: &str, args: CreateLabelArgs) -> Result<LabelData> {
    let api_labels = self.list_api().await?;
    let label_id = api_labels
        .iter()
        .find(|l| l.name == name)
        .map(|l| l.id)
        .ok_or_else(|| CoreError::Platform(format!("Label '{name}' not found")))?;

    let id_str = label_id.to_string();
    let mut cmd_args: Vec<&str> = vec![
        "label", "edit", "--label-id", &id_str, "--repo", &self.repo,
        "--new-name", &args.name, "--color", &args.color,
    ];
    if let Some(ref desc) = args.description {
        cmd_args.push("--description");
        cmd_args.push(desc);
    }
    let output = self.runner.run("glab", &cmd_args).await.map_err(|e| {
        CoreError::Platform(format!("Failed to spawn glab label edit: {e}"))
    })?;
    if !output.status.success() {
        return Err(parse_glab_error(&output.stderr).into());
    }

    let labels = self.list().await?;
    labels
        .into_iter()
        .find(|l| l.name == args.name)
        .ok_or_else(|| CoreError::Platform(format!("Label '{}' not found after edit", args.name)))
}
```

`delete`：`["label", "delete", name, "--repo", &self.repo]`（去 `--yes`）。

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p gitflow-gitlab label`
Expected: PASS。

- [ ] **Step 5: Commit**

```bash
git add crates/gitlab/src/label.rs
git commit -m "fix(gitlab): label create/edit/delete compatible with glab 1.113"
```

---

### Task 11: label.rs milestone `create`/`edit`/`close`/`reopen` — 去 `--output json`

**Files:**
- Modify: `crates/gitlab/src/label.rs`（milestone 部分 L269-419）

**Interfaces:**
- Consumes: `self.list()`（milestone）返回 `Vec<MilestoneData>`（含 `number`/`title`）。
- Produces: `create` 写后按 title 找回；`edit`/`close`/`reopen` 写后按 `number` 找回。

- [ ] **Step 1: Write the failing test**

```rust
#[tokio::test]
async fn test_should_close_milestone_without_output_json_and_refetch() {
    let runner = SequencedMockCommandRunner::from_results(&[
        (true, ""),
        (true, r#"[{"id":1,"iid":3,"title":"v1.0","description":null,"state":"closed","due_date":null,"created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"}]"#),
    ]);
    let provider = GitLabMilestoneProvider::with_runner("owner/repo", runner);

    let ms = provider.close(3).await.expect("should close");

    assert_eq!(ms.number, 3);
    assert_eq!(ms.state, State::Closed);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-gitlab label::tests`
Expected: FAIL。

- [ ] **Step 3: Implement**

四个方法统一：命令改为无 `--output` 形态（`["milestone", "create", "--title", t, "--project", repo, ...]` 等），成功后：

```rust
let milestones = self.list().await?;
milestones
    .into_iter()
    .find(|m| m.title == args.title || m.number == number)
    .ok_or_else(|| CoreError::Platform("Milestone not found after write".into()))
```

（`create`/`edit` 用 `title` 匹配，`close`/`reopen` 用 `number` 匹配——两者分别以各自输入的字段为准。）

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p gitflow-gitlab label`
Expected: PASS。

- [ ] **Step 5: Commit**

```bash
git add crates/gitlab/src/label.rs
git commit -m "fix(gitlab): milestone create/edit/close/reopen without --output json"
```

---

### Task 12: release.rs `create`/`edit` — 去 `--output json`，写后 `view` 重新拉取

**Files:**
- Modify: `crates/gitlab/src/release.rs`（`create` L142-196、`edit` L244-`~290`）

**Interfaces:**
- Consumes: `self.view(tag_name)`（L220-242 已有，`release view --output json` 受支持）。
- Produces: `create`/`edit` 成功后 `self.view(&args.tag_name)`。

- [ ] **Step 1: Write the failing test**

```rust
#[tokio::test]
async fn test_should_create_release_without_output_json_and_refetch_via_view() {
    let runner = SequencedMockCommandRunner::from_results(&[
        (true, ""), // release create 成功
        (true, r#"{"tag_name":"v1.0.0","name":"v1.0.0","description":"notes"}"#),
    ]);
    let provider = GitLabReleaseProvider::with_runner("owner/repo", runner);
    let args = CreateReleaseArgs {
        tag_name: "v1.0.0".to_string(),
        name: None,
        body: Some("notes".to_string()),
        draft: false,
        prerelease: false,
        target_commitish: None,
    };

    let rel = provider.create(args).await.expect("should create");

    assert_eq!(rel.tag_name, "v1.0.0");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-gitlab release::tests`
Expected: FAIL。

- [ ] **Step 3: Implement**

`create`：移除 `--output json`，成功后 `self.view(&args.tag_name).await`。`edit` 同理。同步更新 doc comment。

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p gitflow-gitlab release`
Expected: PASS。

- [ ] **Step 5: Commit**

```bash
git add crates/gitlab/src/release.rs
git commit -m "fix(gitlab): release create/edit without --output json, refetch via view"
```

---

### Task 13: pipeline.rs `jobs` — 改用 `glab api` GET pipeline jobs

**Files:**
- Modify: `crates/gitlab/src/pipeline.rs`（`jobs` L259-288）

**Interfaces:**
- Consumes: `self.repo.split_once('/')`；`JobApiResponse` / `JobData`（已存在）。
- Produces: `glab api /projects/{owner}%2F{project}/pipelines/{id}/jobs` → `Vec<JobData>`。

> 依据：`glab ci view` 无 `--output` 且输出纯文本；`logs`（`ci trace`）本就无 `--output` 且直接返回文本，**不改**。

- [ ] **Step 1: Write the failing test**

```rust
#[tokio::test]
async fn test_should_fetch_jobs_via_glab_api() {
    let runner = MockCommandRunner::success(
        r#"[{"id":1,"name":"build","status":"success"},{"id":2,"name":"test","status":"running"}]"#,
    );
    let provider = GitLabPipelineProvider::with_runner("owner/repo", runner);

    let jobs = provider.jobs(5).await.expect("should fetch");

    assert_eq!(jobs.len(), 2);
    assert_eq!(
        runner.recorded_calls()[0].1,
        vec!["api", "/projects/owner%2Frepo/pipelines/5/jobs"]
            .into_iter().map(String::from).collect::<Vec<_>>()
    );
}
```

> 若 `JobApiResponse` 字段与 GitLab API 返回（`id`/`name`/`status`）不完全一致，测试期据实际结构调整 `JobApiResponse`（serde `#[serde(default)]` 或 `rename`），不得改动共享 `JobData`。

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-gitlab pipeline::tests`
Expected: FAIL。

- [ ] **Step 3: Implement**

```rust
async fn jobs(&self, pipeline_id: u64) -> Result<Vec<JobData>> {
    debug!(repo = %self.repo, pipeline_id, "spawning `glab api` GET pipeline jobs");

    let (owner, project) = self.repo.split_once('/').ok_or_else(|| {
        CoreError::Platform(format!(
            "Invalid repo format '{}', expected 'owner/project'",
            self.repo
        ))
    })?;

    let api_path = format!("/projects/{owner}%2F{project}/pipelines/{pipeline_id}/jobs");

    let output = self
        .runner
        .run("glab", &["api", &api_path])
        .await
        .map_err(|e| CoreError::Platform(format!("Failed to spawn glab api: {e}")))?;

    if !output.status.success() {
        return Err(parse_glab_error(&output.stderr).into());
    }

    let jobs: Vec<JobApiResponse> =
        serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;
    Ok(jobs.into_iter().map(JobData::from).collect())
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p gitflow-gitlab pipeline`
Expected: PASS。

- [ ] **Step 5: Commit**

```bash
git add crates/gitlab/src/pipeline.rs
git commit -m "fix(gitlab): fetch pipeline jobs via glab api instead of ci view --output"
```

---

### Task 14: auth.rs `token` — 改用 `glab auth status --show-token`

**Files:**
- Modify: `crates/gitlab/src/auth.rs`（`token` L145-160）

**Interfaces:**
- Consumes: `std::env::var("GITLAB_HOST")`（可选）。
- Produces: `token()` 返回真实 token 字符串；解析 `auth status --show-token` 输出中的 `Token found ...: <token>` 行。

- [ ] **Step 1: Write the failing test**

```rust
#[tokio::test]
async fn test_should_extract_token_from_auth_status_show_token() {
    let stdout = "192.168.230.23\n  ✓ Logged in to 192.168.230.23 as baoyuexing (keyring)\n  ✓ Token found in operating system keyring: glpat-abcdef\n";
    let runner = MockCommandRunner::success(stdout);
    let provider = GitLabAuthProvider::with_runner(runner);

    let token = provider.token().await.expect("should get token");

    assert_eq!(token, "glpat-abcdef");
    assert_eq!(
        runner.recorded_calls()[0].1,
        vec!["auth", "status", "--show-token"]
            .into_iter().map(String::from).collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn test_should_error_when_no_token_found() {
    let runner = MockCommandRunner::success("  ! No token found (checked config file, keyring, and environment variables).\n");
    let provider = GitLabAuthProvider::with_runner(runner);

    let result = provider.token().await;

    assert!(result.is_err());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-gitlab auth::tests`
Expected: FAIL — 当前仍跑 `glab auth token` 并整段 trim。

- [ ] **Step 3: Implement**

```rust
async fn token(&self) -> Result<String> {
    // 环境变量优先（与 AuthChecker::is_authenticated 一致）
    if let Ok(tok) = std::env::var("GL_TOKEN") {
        return Ok(tok);
    }

    debug!("spawning `glab auth status --show-token`");

    let mut args: Vec<&str> = vec!["auth", "status", "--show-token"];
    if let Ok(host) = std::env::var("GITLAB_HOST") {
        args.push("--hostname");
        args.push(&host);
    }

    let output = self
        .runner
        .run("glab", &args)
        .await
        .map_err(|e| CoreError::Platform(format!("Failed to spawn glab auth status: {e}")))?;

    if !output.status.success() {
        return Err(parse_glab_error(&output.stderr).into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .find_map(|line| {
            if line.contains("Token found") {
                line.rsplit_once(": ").map(|(_, t)| t.trim().to_string())
            } else {
                None
            }
        })
        .filter(|t| !t.is_empty())
        .ok_or_else(|| CoreError::Platform("No GitLab token found (run `glab auth login`)".into()))
}
```

> 说明：`glab auth status --show-token` 对每个实例输出一行 `Token found in ... : <token>`；多实例时 `--hostname`（来自 `GITLAB_HOST`）定位目标实例。无 token 时返回真实错误而非帮助文本。

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p gitflow-gitlab auth`
Expected: PASS。

- [ ] **Step 5: Commit**

```bash
git add crates/gitlab/src/auth.rs
git commit -m "fix(gitlab): read auth token from glab auth status --show-token"
```

---

### Task 15: review.rs `comment`/`request_changes` — 改用 `glab api` POST（依赖 Task 2）

**Files:**
- Modify: `crates/gitlab/src/review.rs`（`comment` L98-130、`request_changes` L186-230）

**Interfaces:**
- Consumes: `NoteApiResponse`（L56 已有，含 author）、`self.repo.split_once('/')`、`self.runner`（Task 2 注入）。
- Produces: `comment`/`request_changes` 返回 `ReviewData`（note author 来自 glab api 响应）。`ReviewData` 字段为 `{ id, state, body: Option<String>, author, submitted_at }`；`approve` 保持现状（无 `--output`）。

- [ ] **Step 1: Write the failing test**

```rust
#[tokio::test]
async fn test_should_post_review_note_via_glab_api() {
    let runner = MockCommandRunner::success(
        r#"{"id":99,"body":"fix this","author":{"username":"alice","id":1},"created_at":"2026-08-18T00:00:00Z"}"#,
    );
    let provider = GitLabReviewProvider::with_runner("owner/repo", runner);

    let review = provider.comment(7, "fix this").await.expect("should post");

    assert_eq!(review.id, 99);
    assert_eq!(review.author.login, "alice");
    assert_eq!(review.body.as_deref(), Some("fix this"));
    assert_eq!(
        runner.recorded_calls()[0].1,
        vec![
            "api", "--method", "POST",
            "/projects/owner%2Frepo/merge_requests/7/notes",
            "-f", "body=fix this",
        ]
        .into_iter().map(String::from).collect::<Vec<_>>()
    );
}
```

`request_changes` 同法断言（`changes_body` 为 `"Changes requested:\n\n{body}"`，路径同 `merge_requests/{n}/notes`）。

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-gitlab review::tests`
Expected: FAIL。

- [ ] **Step 3: Implement**

新增私有辅助（两个方法共用）：

```rust
async fn post_note(&self, pr_number: u64, body: &str) -> Result<NoteApiResponse> {
    debug!(repo = %self.repo, number = pr_number, "spawning `glab api` POST mr note");

    let (owner, project) = self.repo.split_once('/').ok_or_else(|| {
        CoreError::Platform(format!(
            "Invalid repo format '{}', expected 'owner/project'",
            self.repo
        ))
    })?;

    let api_path = format!("/projects/{owner}%2F{project}/merge_requests/{pr_number}/notes");
    let body_arg = format!("body={body}");

    let output = self
        .runner
        .run("glab", &["api", "--method", "POST", &api_path, "-f", &body_arg])
        .await
        .map_err(|e| CoreError::Platform(format!("Failed to spawn glab api: {e}")))?;

    if !output.status.success() {
        return Err(parse_glab_error(&output.stderr).into());
    }

    serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)
}
```

`comment`：

```rust
async fn comment(&self, pr_number: u64, body: &str) -> Result<ReviewData> {
    let note = self.post_note(pr_number, body).await?;
    let author = note.author.as_ref().map_or_else(
        || UserSummary { login: "unknown".into(), id: "0".to_string() },
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

`request_changes`：

```rust
async fn request_changes(&self, pr_number: u64, body: &str) -> Result<ReviewData> {
    let changes_body = format!("Changes requested:\n\n{body}");
    let note = self.post_note(pr_number, &changes_body).await?;
    let author = note.author.as_ref().map_or_else(
        || UserSummary { login: "unknown".into(), id: "0".to_string() },
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

同步更新两个方法的 doc comment。`get_current_user` 保留（仍被 `approve` 使用）。

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p gitflow-gitlab review`
Expected: PASS。

- [ ] **Step 5: Commit**

```bash
git add crates/gitlab/src/review.rs
git commit -m "fix(gitlab): post review notes via glab api instead of --body/--output"
```

---

### Task 16: error.rs — 区分认证与非认证错误提示

**Files:**
- Modify: `crates/gitlab/src/error.rs`（`parse_glab_error` L11-56）

**Interfaces:**
- Consumes: 无。
- Produces: JSON 分支默认 hint 不再指向认证；纯文本回退仅认证类错误提示 `glab auth login`。

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn test_should_not_hint_auth_login_on_unknown_flag_error() {
    let err = parse_glab_error(b"ERROR: Unknown flag: --output");
    assert!(!err.hint.as_deref().unwrap_or("").contains("glab auth login"));
    assert!(err.user_message.contains("执行失败") || !err.user_message.contains("未登录"));
}

#[test]
fn test_should_hint_auth_login_on_not_authenticated_error() {
    let err = parse_glab_error(b"ERROR: not authenticated");
    assert!(err.hint.as_deref().unwrap_or("").contains("glab auth login"));
}

#[test]
fn test_should_not_hint_auth_login_on_not_found_json_error() {
    let err = parse_glab_error(br#"{"message": "404 Not Found", "code": "NOT_FOUND"}"#);
    assert!(!err.hint.as_deref().unwrap_or("").contains("glab auth"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-gitlab error`
Expected: FAIL — 当前全部回退都提示 `glab auth login`，JSON `NOT_FOUND` 也提示认证。

- [ ] **Step 3: Implement**

```rust
let is_auth_failure = |text: &str| {
    let lower = text.to_ascii_lowercase();
    lower.contains("not authenticated")
        || lower.contains("unauthorized")
        || lower.contains("401")
        || lower.contains("token")
};

// JSON 分支
let hint = match code.as_deref() {
    Some("UNAUTHORIZED") => Some("运行 `glab auth login` 完成登录".into()),
    Some("RATE_LIMITED") => Some("等待几分钟后重试".into()),
    Some("VALIDATION_FAILED") => Some("检查请求参数格式是否正确".into()),
    Some("CONFLICT") => Some("运行 `git pull --rebase` 解决冲突后重试".into()),
    Some("NOT_FOUND") => Some("检查资源编号或项目路径是否正确".into()),
    Some("FORBIDDEN") => Some("检查当前账号对该资源的权限".into()),
    _ if is_auth_failure(&text) => Some("运行 `glab auth login` 完成登录".into()),
    _ => None,
};
```

纯文本回退分支：

```rust
let user_message: String = if is_auth_failure(&text) {
    "未登录 GitLab".into()
} else {
    "GitLab CLI 执行失败".into()
};

let mut err = PlatformCliError::new(user_message, text.into_owned(), Platform::GitLab);
if is_auth_failure(&text) {
    err.hint = Some("运行 `glab auth login` 完成登录".into());
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p gitflow-gitlab error`
Expected: PASS。

- [ ] **Step 5: Commit**

```bash
git add crates/gitlab/src/error.rs
git commit -m "fix(gitlab): distinguish auth vs non-auth errors in parse_glab_error"
```

---

### Task 17: CLI `issue/pr list --state all` → `--all`

**Files:**
- Modify: `crates/core/src/types.rs`（State 枚举 L107-113）
- Modify: `crates/gitlab/src/issue.rs`（`list` L301-305 区）
- Modify: `crates/gitlab/src/mr.rs`（`list` L233-243 区）
- Modify: `crates/github/src/issue.rs` / `crates/github/src/pr.rs`（state match 加 `All` 臂）
- Modify: `crates/gitcode/src/issue.rs` / `crates/gitcode/src/pr.rs`（state match 加 `All` 臂）
- Modify: `apps/cli/src/commands/issue.rs`（L203-212）、`apps/cli/src/commands/pr.rs`（L252-261）

**Interfaces:**
- Consumes: `gitflow_core::types::State`。
- Produces: `State::All` 新枚举值；CLI 解析 `all` → `State::All`；gitlab `list` 对 `State::All` 传 `--all`，github/gitcode 传 `--state all`（`gh`/`gitcode` 均支持 `all`）。

> **注意**：`State` 是跨平台共享枚举，新增变体后 `crates/{github,gitcode}/src/{issue,pr}.rs` 中无通配臂的 `match state` 必须同步加 `State::All => "all"`，否则编译失败。这是 Task 17 的必备部分。

- [ ] **Step 1: Write the failing test**

`crates/core/src/types.rs` 的 State 枚举增加 `All`。gitlab 侧测试：

```rust
#[tokio::test]
async fn test_should_list_all_issues_with_all_flag() {
    let runner = MockCommandRunner::success("[]");
    let provider = GitLabIssueProvider::with_runner("owner/repo", runner);

    let _ = provider
        .list(ListIssueArgs { state: Some(State::All), ..Default::default() })
        .await;

    assert_eq!(
        runner.recorded_calls()[0].1,
        vec!["issue", "list", "--repo", "owner/repo", "--output", "json", "--all"]
            .into_iter().map(String::from).collect::<Vec<_>>()
    );
}
```

`mr.rs` 同构（`["mr", "list", "--repo", ..., "--output", "json", "--all"]`）。CLI 侧在 `apps/cli/src/commands/pr.rs` 增加解析测试：

```rust
#[test]
fn test_should_accept_state_all_for_pr_list() {
    let cli = crate::Cli::try_parse_from(["gf", "pr", "list", "--state", "all"]);
    assert!(cli.is_ok());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-gitlab issue mr` 与 `cargo test -p gitflow-cli commands::pr`
Expected: FAIL — `State::All` 不存在 / CLI 拒绝 `all`。

- [ ] **Step 3: Implement**

`crates/core/src/types.rs`：

```rust
pub enum State {
    Open,
    Closed,
    All,
}
```

gitlab `issue::list`：

```rust
if let Some(state) = &args.state {
    match state {
        State::Closed => cmd_args.push("--closed"),
        State::All => cmd_args.push("--all"),
        State::Open => {}
    }
}
```

`mr::list` 同构。github `issue::list` / `pr::list` 与 gitcode 对应 match 增加：

```rust
State::All => "all",
```

CLI `issue.rs` / `pr.rs` 状态解析增加分支：

```rust
"all" => Ok(State::All),
```

并更新错误文案为 `Expected 'open', 'closed', or 'all'.`。

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p gitflow-gitlab issue mr && cargo test -p gitflow-github issue pr && cargo test -p gitflow-gitcode issue pr && cargo test -p gitflow-cli commands::pr`
Expected: PASS。

- [ ] **Step 5: Commit**

```bash
git add crates/core/src/types.rs crates/gitlab/src/issue.rs crates/gitlab/src/mr.rs crates/github/src/issue.rs crates/github/src/pr.rs crates/gitcode/src/issue.rs crates/gitcode/src/pr.rs apps/cli/src/commands/issue.rs apps/cli/src/commands/pr.rs
git commit -m "feat(core): accept --state all on issue/pr list"
```

---

### Task 18: 全量质量闸 + 本地真实 GitLab 冒烟

**Files:**
- None（验证）。

**Interfaces:**
- Consumes: 全部 Task 1-17 产物。

- [x] **Step 1: 全量测试与静态检查**

```bash
make build
make test
make fmt
make clippy
```

Expected: 全部通过；若有失败项按 `CLAUDE.md` 修复后重跑。

- [x] **Step 2: 真实 GitLab 冒烟（192.168.230.23，已登录 `baoyuexing`）**

在可写测试仓库逐项验证（用独立测试 label/release 名，测后清理）：

```bash
# 写操作
gf issue close <n> && gf issue reopen <n> && gf issue comment <n> --body "smoke"
gf label create --name smoke-test --color "#ff0000"
gf label edit smoke-test --name smoke-edited --color "#00ff00"
gf label delete smoke-edited
gf release create --tag smoke-v0 --notes "smoke"
gf release delete smoke-v0
gf pr ready <n> && gf pr wip <n>
gf auth token   # 断言返回 glpat- 前缀真实 token
```

> 若仓库无权限/无现成 MR，用 `glab` 直连同样验证 `glab label create --output json` 等不再报 Unknown flag。

**Expected:** 所有命令成功；失败路径（如 `gf issue close 99999`）退出码非零且错误信息不含「未登录」误导文案。

- [x] **Step 3: 回归基线抽查**

```bash
gf issue list --state open && gf issue view <n> && gf issue comments <n>
gf commit view HEAD && gf workflow status wf-2026-08-18-001 && gf doctor
gf label list && gf release list
```

Expected: 全部正常（多走 `glab api` 的路径不受影响）。

- [x] **Step 4: 对照验收标准自查**

逐一核对 Issue #199 验收清单 9 项（见 Spec §6 映射表），输出一份冒烟结果记录到 `docs/superpowers/plans/2026-08-18-gitlab-glab113-compat.md` 的验收小节或独立报告。

- [ ] **Step 5: Commit（若有测试/文档改动）**

```bash
git add -A
git commit -m "test(gitlab): smoke-test glab 1.113 compatibility against self-hosted GitLab"
```

---

## Task 18 冒烟结果记录（2026-08-18，执行引擎实测）

**环境：** `192.168.230.23`（自建 GitLab），用户 `baoyuexing`（keyring 已登录），测试仓库
`iproost/iproost-docs`（默认分支 `master`）。gf 二进制为工作树 `target/debug/gf`
（分支 `feat/199-gitlab-glab113-compat`）。`GITLAB_HOST=192.168.230.23` 定位实例。

### 写操作冒烟（全部成功）

| 操作 | 结果 | 备注 |
|---|---|---|
| `gf issue create --title ... --body ...` | ✅ exit 0 | 返回 `/-/work_items/3`，`parse_issue_iid_from_url` 新版 URL 解析生效 |
| `gf issue close 3` / `reopen 3` | ✅ exit 0 | 无 `--output json`，写后 `view` 回拉 |
| `gf issue comment 3 --body "smoke comment via gf"` | ✅ exit 0 | 走 `glab api ... notes` |
| `gf label create smoke-test --color "#ff0000"` | ✅ exit 0 | 位置参数 `<NAME>`（计划中 `--name` 写法与 CLI 不符） |
| `gf label edit smoke-test --color "#00ff00"` | ✅ exit 0 | CLI 无 `--new-name`，仅 `--color`/`--description`（计划偏差） |
| `gf label delete smoke-test` | ✅ exit 0 | 无 `--yes`，实际直接删除 |
| `gf release create --tag-name smoke-v0 --body "smoke"` | ✅ exit 0 | CLI 用 `--body`，计划中 `--notes` 不存在 |
| `gf release delete smoke-v0` | ✅ exit 0 | 测后清理 |
| `gf pr create --title ... --head smoke/pr-113-compat --base master` | ✅ exit 0 | 新建 MR !1，`parse_mr_iid_from_url` 生效 |
| `gf pr ready 1` / `gf pr wip 1` | ✅ exit 0 | `mr update --draft=false/true`，`pr view` 确认 draft 状态切换 |
| `gf auth token` | ✅ exit 0 | 返回 `glpat-` 前缀真实 token |

**冒烟中发现并修复的真实 bug：** `glab auth status --show-token` 将整个状态块（含
`Token found ...` 行）写到 **stderr**（exit 0）。原 `token()` 只解析 stdout → 找不到
token。已按 TDD 修复：新增 `MockCommandRunner::success_with_stderr` + 回归测试
（RED → GREEN），`token()` 改为合并解析 stdout+stderr。
Commit: `94f1fa6 fix(gitlab): parse auth token from stderr in auth status --show-token`。

### 失败路径

`gf issue close 99999` → **exit 1**，错误信息为「GitLab CLI 执行失败」+ 文档链接，
不含「未登录/请登录」误导文案（已验证输出无 `登录|login|not authenticated|unauthorized|401|token`）。

### 回归基线抽查

| 命令 | 结果 |
|---|---|
| `gf issue list --state open` / `--state all` | ✅ exit 0（`State::All` 映射 `--all` 生效） |
| `gf issue view 3` / `gf issue comments 3` | ✅ exit 0 |
| `gf commit view HEAD`（GitLab 仓库上下文） | ✅ exit 0 |
| `gf label list` / `gf release list` | ✅ exit 0 |
| `gf doctor` | ✅ exit 0（9 项：4✅ 3⚠️ 2❌，均为本工作树 skills/hook 环境项，与本次改动无关） |
| `gf workflow status wf-2026-08-18-001` | ❌ 预存在缺陷：CLI `WorkflowMode` 仅支持 `full`/`fast`，本合同 `"mode": "standard"`（contract schema v1.1 允许）→ `unknown variant 'standard'`。**与 Issue #199 无关，不在本次范围，未修复**，需另行处理 |

### 清理

- MR !1 已关闭，`smoke/pr-113-compat` 分支已删除，smoke issue #3 已删除，smoke label / smoke-v0 release 已删除。仓库恢复无残留测试数据。

### 全量质量闸（含 auth 修复后）

`make build` ✅ · `make test` ✅（1299/1299，nextest）· `make fmt` ✅ · `make clippy` ✅
