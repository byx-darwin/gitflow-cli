# GitHub gh 2.97 Label Edit 假失败修复 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix `gf label edit` 假失败 by replacing the nonexistent `gh label view` call with `gh api`, while also fixing the misleading auth-login hint (P2) and missing label-list pagination (P3).

**Architecture:** Refactor `GitHubLabelProvider` to use the existing `CommandRunner` abstraction (mirroring `GitHubAuthProvider` and the #199 GitLab fix), so command args are unit-testable. `fetch_label` switches from `gh label view --json` (removed in gh 2.97) to the REST endpoint `gh api repos/{owner}/{repo}/labels/{name}` with RFC 3986 path-segment encoding. `parse_gh_error` only suggests `gh auth login` on genuine auth failures.

**Tech Stack:** Rust 2024, tokio, serde_json, async-trait, tracing. No new dependencies — URL encoding is a private helper.

**Spec:** `docs/superpowers/specs/2026-08-18-github-gh-label-edit-design.md`

## Global Constraints

- Rust 2024 edition, pinned toolchain in `rust-toolchain.toml`.
- Must pass `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`, `cargo +nightly fmt`, and the full `make test`.
- No `unwrap()`/`expect()` in production code; use `Result<T>` and `CoreError`.
- All public items need docs; `#[must_use]` on pure value-returning functions.
- Do NOT use `gh label view` anywhere — gh 2.97 has no such subcommand.
- Do NOT touch `crates/gitcode/src/label.rs` or the milestone parts of `crates/github/src/label.rs` (out of scope).
- Keep `GitHubLabelProvider::new(repo)` working — `apps/cli/src/commands/label.rs:138` calls it; default type parameter must preserve the call site.

---
---

### Task 1: Add `recorded_calls` to GitHub `MockCommandRunner`

**Files:**
- Modify: `crates/github/src/runner.rs` (the `#[cfg(test)]` mock section, lines ~95-181)

**Interfaces:**
- Consumes: nothing (standalone test-infra change).
- Produces: `MockCommandRunner::recorded_calls(&self) -> Vec<(String, Vec<String>)>` — the sequence of `(program, args)` for every `run`/`run_with_stdin` call, in order. Task 2's tests depend on this to assert exact `gh` args.

Why: the GitHub `MockCommandRunner` currently returns canned output but does **not** record the args it was called with. The P1 regression tests in Task 2 must assert that `fetch_label` invokes `gh api ...` and *never* `gh label view`. GitLab's runner already has this API (see `crates/gitlab/src/runner.rs:201`) — this task mirrors it.

- [ ] **Step 1: Write the failing test**

Add to the `#[cfg(test)] mod tests` block at the bottom of `crates/github/src/runner.rs`:

```rust
#[tokio::test]
async fn test_should_record_calls() {
    let runner = MockCommandRunner::success("ok");
    runner.run("gh", &["label", "list"]).await.expect("should run");
    runner
        .run_with_stdin("gh", &["api", "repos/o/r"], b"data")
        .await
        .expect("should run with stdin");
    let calls = runner.recorded_calls();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].0, "gh");
    assert_eq!(calls[0].1, vec!["label", "list"]);
    assert_eq!(calls[1].0, "gh");
    assert_eq!(calls[1].1, vec!["api", "repos/o/r"]);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-github test_should_record_calls`
Expected: compile error — `no method named recorded_calls` on `MockCommandRunner`.

- [ ] **Step 3: Implement the recording**

Modify the mock section of `crates/github/src/runner.rs`:

Add type aliases near the top of the mock section (after the `MockResult` enum):

```rust
/// A single recorded command invocation: `(program, args)`.
type RecordedCall = (String, Vec<String>);

/// All recorded command invocations in execution order.
type RecordedCalls = Vec<RecordedCall>;
```

Change the struct:

```rust
#[cfg(test)]
#[derive(Debug, Clone)]
pub struct MockCommandRunner {
    result: MockResult,
    /// Recorded `(program, args)` sequences for every `run` call.
    recorded: std::sync::Arc<std::sync::Mutex<RecordedCalls>>,
}
```

Add the `recorded` initializer to each of the three constructors (`success`, `failure`, `spawn_error`):

```rust
        Self {
            result: MockResult::Output(CommandOutput {
                status: Self::make_exit_status(0),
                stdout: stdout.as_bytes().to_vec(),
                stderr: Vec::new(),
            }),
            recorded: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
```

(failure: `result: MockResult::Output(...)` with `stderr: stderr.as_bytes().to_vec()`; spawn_error: `result: MockResult::Error(...)`.)

Add the accessor method in `impl MockCommandRunner`:

```rust
    /// Return the recorded `(program, args)` sequences from every executed call.
    ///
    /// # Panics
    ///
    /// Panics if the internal recording mutex is poisoned (a prior panic while
    /// holding the lock).
    #[must_use]
    pub fn recorded_calls(&self) -> Vec<(String, Vec<String>)> {
        self.recorded.lock().expect("mock mutex poisoned").clone()
    }
```

Modify the `run` impl to push before returning:

```rust
    async fn run(&self, program: &str, args: &[&str]) -> std::io::Result<CommandOutput> {
        self.recorded.lock().expect("mock mutex poisoned").push((
            program.to_string(),
            args.iter().map(|s| (*s).to_string()).collect(),
        ));
        match &self.result {
            MockResult::Output(output) => Ok(output.clone()),
            MockResult::Error(kind, message) => Err(std::io::Error::new(*kind, message.clone())),
        }
    }
```

`run_with_stdin` already delegates to `self.run(...)`, so it records through the same path — no change needed there.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p gitflow-github`
Expected: all pass, including the existing `test_should_clone_mock_runner` (Clone shares the `Arc` recording, matching GitLab semantics).

- [ ] **Step 5: Commit**

```bash
git add crates/github/src/runner.rs
git commit -m "test(github): record gh command args in MockCommandRunner"
```

---
---

### Task 2: `label.rs` — runner refactor, `fetch_label` via `gh api`, URL encoding, list pagination (P1 + P3)

**Files:**
- Modify: `crates/github/src/label.rs` (entire `GitHubLabelProvider` — struct, constructors, trait impl, `fetch_label`, plus `encode_path_segment` helper and tests)

**Interfaces:**
- Consumes: `CommandRunner`, `RealCommandRunner`, `MockCommandRunner` (with `recorded_calls` from Task 1) from `crate::runner`; `parse_gh_error` from `crate::error`; `LabelProvider`, `LabelData`, `CreateLabelArgs` from `gitflow_core::label`.
- Produces:
  - `GitHubLabelProvider<R: CommandRunner = RealCommandRunner>` — generic struct with `new(repo)` (default runner) and `with_runner(repo, runner)`.
  - `encode_path_segment(value: &str) -> String` — module-private RFC 3986 path-segment encoder.
  - `fetch_label(&self, name: &str) -> Result<LabelData>` — now calls `gh api repos/{owner}/{repo}/labels/{name}`.

- [ ] **Step 1: Write the failing tests**

Add to the `#[cfg(test)] mod tests` block in `crates/github/src/label.rs`, and add the imports at the top of the tests module:

```rust
    use crate::runner::MockCommandRunner;
```

New tests:

```rust
    #[tokio::test]
    async fn test_should_fetch_label_via_gh_api() {
        let runner = MockCommandRunner::success(
            r#"{"name":"bug","color":"d73a4a","description":"Something isn't working"}"#,
        );
        let provider = GitHubLabelProvider::with_runner("octocat/hello-world", runner.clone());

        let label = provider.fetch_label("bug").await.expect("should fetch");

        assert_eq!(label.name, "bug");
        assert_eq!(label.color.as_deref(), Some("d73a4a"));
        assert_eq!(
            runner.recorded_calls()[0].1,
            vec!["api", "repos/octocat/hello-world/labels/bug"]
        );
    }

    #[tokio::test]
    async fn test_should_encode_label_name_in_api_path() {
        let runner = MockCommandRunner::success(r#"{"name":"good first issue","color":"7057ff"}"#);
        let provider = GitHubLabelProvider::with_runner("octocat/hello-world", runner.clone());

        let label = provider
            .fetch_label("good first issue")
            .await
            .expect("should fetch");

        assert_eq!(label.name, "good first issue");
        assert_eq!(
            runner.recorded_calls()[0].1,
            vec!["api", "repos/octocat/hello-world/labels/good%20first%20issue"]
        );
    }

    #[tokio::test]
    async fn test_should_edit_label_and_refetch_via_gh_api() {
        let runner = MockCommandRunner::success(
            r#"{"name":"bug","color":"3344ff","description":"probe2"}"#,
        );
        let provider = GitHubLabelProvider::with_runner("byx-darwin/gitflow-cli", runner.clone());

        let args = CreateLabelArgs {
            name: "bug".to_string(),
            color: "3344ff".to_string(),
            description: Some("probe2".to_string()),
        };
        let label = provider.edit("bug", args).await.expect("should edit");

        assert_eq!(label.color.as_deref(), Some("3344ff"));
        let calls = runner.recorded_calls();
        assert_eq!(calls.len(), 2);
        assert_eq!(
            calls[0].1,
            vec![
                "label",
                "edit",
                "bug",
                "--repo",
                "byx-darwin/gitflow-cli",
                "--color",
                "3344ff",
                "--description",
                "probe2"
            ]
        );
        // P1 regression: second call must be `gh api`, NEVER `gh label view`.
        assert_eq!(calls[1].1, vec!["api", "repos/byx-darwin/gitflow-cli/labels/bug"]);
        assert!(
            calls.iter().all(|(_, args)| args.first().is_some_and(|a| a != "view")),
            "no `gh label view` call may remain"
        );
    }

    #[tokio::test]
    async fn test_should_fail_when_fetch_label_api_fails() {
        let runner = MockCommandRunner::failure("HTTP 404", 1);
        let provider = GitHubLabelProvider::with_runner("owner/repo", runner);

        let err = provider.fetch_label("missing").await.expect_err("should fail");
        assert!(err.to_string().contains("GitHub") || err.to_string().contains("执行失败"));
    }

    #[tokio::test]
    async fn test_should_list_labels_with_limit_flag() {
        let runner = MockCommandRunner::success("[]");
        let provider = GitHubLabelProvider::with_runner("owner/repo", runner.clone());

        let labels = provider.list().await.expect("should list");

        assert!(labels.is_empty());
        assert_eq!(
            runner.recorded_calls()[0].1,
            vec![
                "label",
                "list",
                "--repo",
                "owner/repo",
                "--json",
                "name,color,description",
                "--limit",
                "100"
            ]
        );
    }

    #[tokio::test]
    async fn test_should_create_label_via_runner() {
        let runner = MockCommandRunner::success("");
        let provider = GitHubLabelProvider::with_runner("owner/repo", runner.clone());

        let args = CreateLabelArgs {
            name: "bug".to_string(),
            color: "d73a4a".to_string(),
            description: None,
        };
        let label = provider.create(args).await.expect("should create");

        assert_eq!(label.name, "bug");
        assert_eq!(label.color.as_deref(), Some("d73a4a"));
        assert_eq!(
            runner.recorded_calls()[0].1,
            vec![
                "label",
                "create",
                "bug",
                "--color",
                "d73a4a",
                "--repo",
                "owner/repo"
            ]
        );
    }

    #[tokio::test]
    async fn test_should_delete_label_via_runner() {
        let runner = MockCommandRunner::success("");
        let provider = GitHubLabelProvider::with_runner("owner/repo", runner.clone());

        provider.delete("bug").await.expect("should delete");

        assert_eq!(
            runner.recorded_calls()[0].1,
            vec!["label", "delete", "bug", "--yes", "--repo", "owner/repo"]
        );
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p gitflow-github label`
Expected: compile errors — `no method named with_runner`, `no method named fetch_label` on the non-generic type (fetch_label is currently in a non-generic `impl`; tests can't construct with a runner).

- [ ] **Step 3: Implement the refactor**

Modify `crates/github/src/label.rs`:

**(a)** Change the imports at the top of the file:

```rust
use crate::{
    error::parse_gh_error,
    runner::{CommandRunner, RealCommandRunner},
};
```

(remove the direct `use crate::error::parse_gh_error;` line — fold it into the `use crate::{...}` block.)

**(b)** Replace the struct + constructors:

```rust
/// GitHub Label 提供者，通过 `gh` CLI 管理仓库标签。
///
/// # Examples
///
/// ```no_run
/// use gitflow_github::GitHubLabelProvider;
///
/// let provider = GitHubLabelProvider::new("octocat/hello-world");
/// ```
#[derive(Debug, Clone)]
pub struct GitHubLabelProvider<R: CommandRunner = RealCommandRunner> {
    /// GitHub `owner/repo`。
    repo: String,
    /// 用于执行 `gh` CLI 命令的 runner。
    runner: R,
}

impl GitHubLabelProvider<RealCommandRunner> {
    /// 创建新的 GitHub Label 提供者。
    ///
    /// `repo` 格式为 `owner/repo`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> Self {
        Self {
            repo: repo.into(),
            runner: RealCommandRunner,
        }
    }
}

impl<R: CommandRunner> GitHubLabelProvider<R> {
    /// 使用自定义 [`CommandRunner`] 创建提供者。
    ///
    /// 主要用于测试，可注入模拟 runner 以控制 `gh` CLI 的输出。
    /// `repo` 格式为 `owner/repo`。
    #[must_use]
    pub fn with_runner(repo: impl Into<String>, runner: R) -> Self {
        Self {
            repo: repo.into(),
            runner,
        }
    }
}
```

**(c)** Change the trait impl signature:

```rust
#[async_trait]
impl<R: CommandRunner + 'static> LabelProvider for GitHubLabelProvider<R> {
```

**(d)** `create` — replace the direct `Command` with the runner:

```rust
    async fn create(&self, args: CreateLabelArgs) -> Result<LabelData> {
        debug!(
            repo = %self.repo,
            name = %args.name,
            color = %args.color,
            "spawning `gh label create`"
        );

        let mut cmd_args: Vec<&str> = vec![
            "label",
            "create",
            &args.name,
            "--color",
            &args.color,
            "--repo",
            &self.repo,
        ];

        if let Some(ref desc) = args.description {
            cmd_args.push("--description");
            cmd_args.push(desc);
        }

        let output = self
            .runner
            .run("gh", &cmd_args)
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh label create: {e}")))?;

        if !output.status.success() {
            return Err(parse_gh_error(&output.stderr).into());
        }

        // gh label create doesn't return JSON, construct the response manually
        Ok(LabelData {
            name: args.name,
            color: Some(args.color),
            description: args.description,
        })
    }
```

**(e)** `list` — use the runner and add `--limit 100` (P3):

```rust
    async fn list(&self) -> Result<Vec<LabelData>> {
        debug!(repo = %self.repo, "spawning `gh label list`");

        let output = self
            .runner
            .run(
                "gh",
                &[
                    "label",
                    "list",
                    "--repo",
                    &self.repo,
                    "--json",
                    LABEL_FIELDS,
                    "--limit",
                    "100",
                ],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh label list: {e}")))?;

        if !output.status.success() {
            return Err(parse_gh_error(&output.stderr).into());
        }

        let labels: Vec<LabelData> =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(labels)
    }
```

**(f)** `edit` — use the runner:

```rust
    async fn edit(&self, name: &str, args: CreateLabelArgs) -> Result<LabelData> {
        debug!(repo = %self.repo, name, "spawning `gh label edit`");

        let mut cmd_args: Vec<&str> = vec![
            "label",
            "edit",
            name,
            "--repo",
            &self.repo,
            "--color",
            &args.color,
        ];

        if let Some(ref desc) = args.description {
            cmd_args.push("--description");
            cmd_args.push(desc);
        }

        let output = self
            .runner
            .run("gh", &cmd_args)
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh label edit: {e}")))?;

        if !output.status.success() {
            return Err(parse_gh_error(&output.stderr).into());
        }

        // gh label edit 不返回 JSON，重新 fetch 获取最新数据
        self.fetch_label(name).await
    }
```

**(g)** `delete` — use the runner:

```rust
    async fn delete(&self, name: &str) -> Result<()> {
        debug!(repo = %self.repo, name, "spawning `gh label delete`");

        let output = self
            .runner
            .run("gh", &["label", "delete", name, "--yes", "--repo", &self.repo])
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh label delete: {e}")))?;

        if !output.status.success() {
            return Err(parse_gh_error(&output.stderr).into());
        }

        Ok(())
    }
```

**(h)** Move `fetch_label` into the generic impl block and switch it to `gh api` (P1), and add the URL encoder:

```rust
impl<R: CommandRunner> GitHubLabelProvider<R> {
    /// 获取指定名称的标签数据（内部辅助方法）。
    ///
    /// 调用 `gh api` REST 端点 `repos/{owner}/{repo}/labels/{name}` 重新拉取。
    /// gh 2.97 没有 `label view` 子命令，故不能用 `gh label view --json`。
    async fn fetch_label(&self, name: &str) -> Result<LabelData> {
        let api_path = format!(
            "repos/{repo}/labels/{name}",
            repo = self.repo,
            name = encode_path_segment(name)
        );

        let output = self
            .runner
            .run("gh", &["api", &api_path])
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh api label: {e}")))?;

        if !output.status.success() {
            return Err(parse_gh_error(&output.stderr).into());
        }

        let label: LabelData =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(label)
    }
}
```

Add the module-level helper function (place it right after the `GitHubLabelProvider` impl blocks, before the milestone section):

```rust
/// RFC 3986 路径段编码：仅保留 unreserved 字符，其余按字节百分号编码（大写十六进制）。
///
/// 用于在 `gh api repos/{owner}/{repo}/labels/{name}` 路径中编码标签名，
/// 标签名可能包含空格或特殊字符（如 `good first issue`）。
#[must_use]
fn encode_path_segment(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for b in value.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(char::from(b));
            }
            _ => {
                out.push('%');
                out.push_str(&format!("{b:02X}"));
            }
        }
    }
    out
}
```

Note: the `LABEL_FIELDS` const stays unchanged.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p gitflow-github`
Expected: all pass — existing constructor/debug/clone tests (they call `GitHubLabelProvider::new(...)`), deserialization tests, and the 7 new tests.

- [ ] **Step 5: Run lint + fmt**

Run: `cargo clippy -p gitflow-github --all-targets --all-features -- -D warnings -W clippy::pedantic` and `cargo +nightly fmt --check`
Expected: clean. If clippy flags the `format!("{b:02X}")` in a hot loop or similar, apply the suggested fix and re-run.

- [ ] **Step 6: Commit**

```bash
git add crates/github/src/label.rs
git commit -m "fix(github): fetch_label via gh api, drop nonexistent label view (P1); label list --limit 100 (P3)"
```

---
---

### Task 3: `error.rs` — only hint `gh auth login` on real auth failures (P2)

**Files:**
- Modify: `crates/github/src/error.rs` (add `is_auth_failure` closure, rework the JSON-path default hint and the plain-text fallback)

**Interfaces:**
- Consumes: nothing new — mirrors `crates/gitlab/src/error.rs:14-71` (the #199 fix).
- Produces: unchanged public API; behavior change only. `parse_gh_error` returns hint `None` for non-auth errors.

- [ ] **Step 1: Write the failing tests**

Add to the `#[cfg(test)] mod tests` block in `crates/github/src/error.rs`:

```rust
    #[test]
    fn test_should_not_hint_auth_login_on_unknown_flag_error() {
        let err = parse_gh_error(b"unknown flag: --json\nUsage: gh label <command> [flags]");
        assert!(!err.user_message.contains("未登录"));
        assert!(err.hint.is_none());
    }

    #[test]
    fn test_should_hint_auth_login_on_not_authenticated_error() {
        let err = parse_gh_error(b"gh: Not logged in. Please run `gh auth login` to authenticate.");
        assert!(err.user_message.contains("登录"));
        assert!(err.hint.as_deref().is_some_and(|h| h.contains("gh auth login")));
    }

    #[test]
    fn test_should_not_hint_auth_login_on_generic_json_error() {
        let err = parse_gh_error(br#"{"message": "Something went wrong"}"#);
        assert!(err.hint.is_none());
    }
```

Also **update** the existing `test_should_handle_empty_stderr` — empty stderr is not an auth failure, so its hint must now be `None`:

```rust
    #[test]
    fn test_should_handle_empty_stderr() {
        let err = parse_gh_error(b"");
        assert!(!err.user_message.is_empty());
        assert!(err.hint.is_none());
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p gitflow-github error::tests`
Expected: the new tests fail (current code always sets an auth hint), and the updated `test_should_handle_empty_stderr` fails (`assert!(err.hint.is_none())` fails because hint is `Some`).

- [ ] **Step 3: Implement the fix**

Modify `crates/github/src/error.rs`:

**(a)** Add the auth-failure predicate right after `let text = ...;`:

```rust
    let is_auth_failure = |t: &str| {
        let lower = t.to_ascii_lowercase();
        lower.contains("not authenticated")
            || lower.contains("unauthorized")
            || lower.contains("not logged in")
            || lower.contains("401")
            || lower.contains("token")
    };
```

**(b)** Replace the JSON-path `hint` match arm (the current `_ => Some("运行 `gh auth status` 检查认证状态".into())`) so the default only hints on real auth failure, and add specific code cases mirroring GitLab:

```rust
        let hint = match code.as_deref() {
            Some("UNAUTHORIZED") => Some("运行 `gh auth status` 检查认证状态".into()),
            Some("RATE_LIMITED") => Some("等待几分钟后重试".into()),
            Some("VALIDATION_FAILED") => Some("检查请求参数格式是否正确".into()),
            Some("CONFLICT") => Some("运行 `git pull --rebase` 解决冲突后重试".into()),
            Some("GONE") => Some("确认资源是否存在，可能已被删除或重命名".into()),
            Some("NOT_FOUND") => Some("检查资源名称或编号是否正确".into()),
            Some("FORBIDDEN") => Some("检查当前账号对该资源的权限".into()),
            _ if is_auth_failure(&text) => Some("运行 `gh auth login` 完成登录".into()),
            _ => None,
        };
```

**(c)** Replace the plain-text fallback block:

```rust
    // 回退：纯文本解析
    let is_auth = is_auth_failure(&text);
    let user_message: String = if is_auth {
        "未登录 GitHub".into()
    } else {
        "GitHub CLI 执行失败".into()
    };

    let mut err = PlatformCliError::new(user_message, text.into_owned(), Platform::GitHub);
    if is_auth {
        err.hint = Some("运行 `gh auth login` 完成登录".into());
    }
    err.doc_link = Some("https://cli.github.com/manual/".into());
    err
```

(The old check `text.contains("Not logged in") || text.contains("auth")` is dropped — `"auth"` alone was too broad and produced the false-positive hint described in P2. The `is_auth_failure` closure explicitly covers `not logged in`.)

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p gitflow-github error::tests`
Expected: all pass — the existing `test_should_parse_gh_plain_text_error` still passes because its stderr contains "Not logged in" (auth → hint `Some`), the existing JSON code tests (NOT_FOUND/FORBIDDEN/RATE_LIMITED/VALIDATION_FAILED/CONFLICT/GONE) still pass with their explicit hints, and `test_should_not_leak_raw_stderr_in_display`/`test_should_handle_empty_stderr` pass with the new `None` hints.

- [ ] **Step 5: Run lint + fmt**

Run: `cargo clippy -p gitflow-github --all-targets --all-features -- -D warnings -W clippy::pedantic` and `cargo +nightly fmt --check`
Expected: clean.

- [ ] **Step 6: Commit**

```bash
git add crates/github/src/error.rs
git commit -m "fix(github): only hint gh auth login on real auth failures (P2)"
```

---
---

### Task 4: Full workspace validation + final commit

**Files:**
- None changed — validation and workspace checks only.

**Interfaces:**
- Consumes: all changes from Tasks 1-3.

- [ ] **Step 1: Run the full unit test suite**

Run: `cargo test`
Expected: all workspace tests pass, including `crates/github`, `crates/gitlab` (unchanged), and `crates/core`.

- [ ] **Step 2: Run fmt + clippy on the touched crates**

Run: `cargo +nightly fmt --check`
Run: `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: clean.

- [ ] **Step 3: Confirm no `gh label view` remains**

Run: `grep -rn "label.*view" crates/github/src/`
Expected: no match (the old `label view` / `Failed to spawn gh label view` strings are gone). The gitlab/gitcode crates may still reference their own `label view` — those are out of scope for this workflow (documented in the spec).

- [ ] **Step 4: (Optional) Live probe against real `gh`**

If a GitHub token is available, verify `gh api repos/byx-darwin/gitflow-cli/labels` returns JSON and that a `gitflow` test label (create → edit → delete) flows without the false-failure. This is optional — unit tests are the gate; skip if the repo already has enough real labels to avoid touching shared state.

- [ ] **Step 5: Final commit (if any stray fixes were made)**

```bash
git add -A
git commit -m "test(github): label edit regression coverage for gh api fetch"
```
If nothing changed, skip this step.
