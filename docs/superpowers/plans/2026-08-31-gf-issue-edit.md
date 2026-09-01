# gf issue edit Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `gf issue edit <number>` supporting partial updates to an Issue's title and/or body across GitHub, GitLab, and GitCode.

**Architecture:** Extend the `IssueProvider` trait with an `edit` method and a new `EditIssueArgs { title: Option<String>, body: Option<String> }` struct. Each of the three platform adapters implements `edit` using its existing CLI wrapper (`gh`, `glab`, the `gitcode` binary), reusing the mutate-then-`view()` pattern already used by `close`/`reopen` (GitLab) and label mutations (GitHub/GitCode). The CLI layer adds an `IssueCommand::Edit` variant, reusing the existing `resolve_body()` helper.

**Tech Stack:** Rust 2024, `async-trait`, `clap` (derive), `miette`, `tokio::process` via the shared `CommandRunner` abstraction, `serde`/`serde_json`.

**Spec:** `docs/superpowers/specs/2026-08-31-gf-issue-edit-design.md`

## Global Constraints

- Never use `unwrap()`/`expect()` in production code (test code may use `expect()` with a message).
- Return `Result<T>` for fallible operations.
- `#[must_use]` on pure value-returning functions.
- Reuse existing error paths: `parse_gh_error` / `parse_glab_error` / `parse_gitcode_error`.
- No new REST/PATCH calls — reuse each platform's existing `issue edit`/`issue update` CLI subcommand.
- Test naming: `test_should_<expected_behavior>`.
- Run `make fmt` and `make clippy` after each task's GREEN step, before commit.

---

### Task 1: Core — `EditIssueArgs` + `IssueProvider::edit`

**Files:**
- Modify: `crates/core/src/issue.rs`

**Interfaces:**
- Produces: `pub struct EditIssueArgs { pub title: Option<String>, pub body: Option<String> }` (derives `Debug, Clone, Default`); `async fn edit(&self, number: u64, args: EditIssueArgs) -> Result<IssueData>` on `IssueProvider`.

- [ ] **Step 1: Write the failing test**

Add to the `#[cfg(test)] mod tests` block in `crates/core/src/issue.rs`:

```rust
    #[test]
    fn test_edit_issue_args_default_is_empty() {
        let args = EditIssueArgs::default();
        assert!(args.title.is_none());
        assert!(args.body.is_none());
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-core test_edit_issue_args_default_is_empty`
Expected: FAIL with "cannot find type `EditIssueArgs` in this scope"

- [ ] **Step 3: Write minimal implementation**

Add after `CreateIssueArgs` (around line 60) in `crates/core/src/issue.rs`:

```rust
/// 编辑 Issue 所需参数（部分更新）。
///
/// 未设置的字段（`None`）在调用 [`IssueProvider::edit`] 时保持当前值不变。
#[derive(Debug, Clone, Default)]
pub struct EditIssueArgs {
    /// 新标题（不修改时为 `None`）。
    pub title: Option<String>,
    /// 新正文（不修改时为 `None`）。
    pub body: Option<String>,
}
```

Add to the `IssueProvider` trait (after `create`, before `list`):

```rust
    /// 编辑 Issue 的标题和/或正文（部分更新）。
    ///
    /// 未在 `args` 中设置的字段保持 Issue 当前值不变。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在或平台 API 调用失败时返回错误。
    async fn edit(&self, number: u64, args: EditIssueArgs) -> Result<IssueData>;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p gitflow-core test_edit_issue_args_default_is_empty`
Expected: PASS (crate will not compile yet — that's expected until Tasks 2-4 implement the trait method on each provider; run `cargo check -p gitflow-core` to confirm only `gitflow-core` itself compiles clean at this point, other crates will fail until later tasks)

- [ ] **Step 5: Commit**

```bash
git add crates/core/src/issue.rs
git commit -m "feat(core): add EditIssueArgs and IssueProvider::edit"
```

---

### Task 2: GitHub adapter — `edit`

**Files:**
- Modify: `crates/github/src/issue.rs`

**Interfaces:**
- Consumes: `EditIssueArgs { title: Option<String>, body: Option<String> }` from `gitflow_core::issue`; `parse_gh_error(&[u8]) -> PlatformCliError` (existing); `self.view(number).await -> Result<IssueData>` (existing).
- Produces: `GitHubIssueProvider::edit` satisfying `IssueProvider::edit`.

- [ ] **Step 1: Write the failing test**

Add to `mod tests` in `crates/github/src/issue.rs` (near the other edit-adjacent tests, after `test_should_return_platform_error_when_gh_fails_for_add_labels`):

```rust
    #[tokio::test]
    async fn test_should_edit_issue_title_and_view_result() {
        // Sequence: 1. `gh issue edit` → succeeds (empty stdout)
        //           2. `gh issue view --json ...` → returns updated issue JSON
        let runner = SequencedMockCommandRunner::from_results(&[
            (true, ""),
            (
                true,
                r#"{"number":42,"title":"New title","body":"orig","state":"open","labels":[],"author":{"login":"octocat","id":"1"},"assignees":[],"createdAt":"2026-01-01T00:00:00Z","updatedAt":"2026-01-02T00:00:00Z","url":"https://github.com/owner/repo/issues/42"}"#,
            ),
        ]);
        let provider = GitHubIssueProvider::with_runner("owner/repo", runner);

        let issue = provider
            .edit(
                42,
                gitflow_core::issue::EditIssueArgs {
                    title: Some("New title".to_string()),
                    body: None,
                },
            )
            .await
            .expect("edit should succeed");

        assert_eq!(issue.number, 42);
        assert_eq!(issue.title, "New title");
    }

    #[tokio::test]
    async fn test_should_send_only_provided_fields_for_edit() {
        let runner = MockCommandRunner::success("");
        let provider = GitHubIssueProvider::with_runner("owner/repo", runner.clone());

        // title-only: view() call will fail to deserialize empty stdout, that's fine —
        // we only assert on the recorded `gh issue edit` invocation (the first call).
        let _ = provider
            .edit(
                7,
                gitflow_core::issue::EditIssueArgs {
                    title: Some("T".to_string()),
                    body: None,
                },
            )
            .await;

        assert_eq!(
            runner.recorded_calls()[0].1,
            vec!["issue", "edit", "7", "--repo", "owner/repo", "--title", "T"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gh_fails_for_edit() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not found"}"#, 256);
        let provider = GitHubIssueProvider::with_runner("owner/repo", runner);

        let result = provider
            .edit(42, gitflow_core::issue::EditIssueArgs::default())
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-github test_should_edit_issue_title_and_view_result`
Expected: FAIL with "no method named `edit` found" (trait not yet implemented on this provider)

- [ ] **Step 3: Write minimal implementation**

Add to `impl<R: CommandRunner + 'static> IssueProvider for GitHubIssueProvider<R>` in `crates/github/src/issue.rs`, after `create` and before `list`:

```rust
    /// 编辑 Issue 的标题和/或正文。
    ///
    /// 调用 `gh issue edit <number> --repo <repo> [--title T] [--body B]`，
    /// 成功后通过 [`view`](Self::view) 重新拉取最新数据并返回。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在或 `gh` CLI 调用失败时返回错误。
    async fn edit(&self, number: u64, args: gitflow_core::issue::EditIssueArgs) -> Result<IssueData> {
        debug!(repo = %self.repo, number, "spawning `gh issue edit`");

        let number_str = number.to_string();
        let mut cmd_args: Vec<&str> = vec!["issue", "edit", &number_str, "--repo", &self.repo];

        if let Some(title) = &args.title {
            cmd_args.push("--title");
            cmd_args.push(title);
        }
        if let Some(body) = &args.body {
            cmd_args.push("--body");
            cmd_args.push(body);
        }

        let output = self
            .runner
            .run("gh", &cmd_args)
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            return Err(parse_gh_error(&output.stderr).into());
        }

        self.view(number).await
    }
```

Update the `use gitflow_core::{...}` import at the top of the file to include `EditIssueArgs`:

```rust
use gitflow_core::{
    CoreError, Result,
    issue::{CreateIssueArgs, EditIssueArgs, IssueData, IssueProvider, ListIssueArgs},
    types::{CommentData, Label, State, UserSummary},
};
```

Then simplify the method signature to use the imported type directly: `async fn edit(&self, number: u64, args: EditIssueArgs) -> Result<IssueData>`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p gitflow-github issue::`
Expected: PASS (all issue.rs tests, including the 3 new ones)

- [ ] **Step 5: Commit**

```bash
git add crates/github/src/issue.rs
git commit -m "feat(github): implement IssueProvider::edit via gh issue edit"
```

---

### Task 3: GitLab adapter — `edit`

**Files:**
- Modify: `crates/gitlab/src/issue.rs`

**Interfaces:**
- Consumes: `EditIssueArgs` from `gitflow_core::issue`; `parse_glab_error`; `self.view(number).await`.
- Produces: `GitLabIssueProvider::edit`.

**Note:** `glab` has no `issue edit` subcommand (verified against `glab 1.115.0` — running `glab issue edit --help` silently falls through to the top-level `glab issue` help with exit 0). Title/body/description live under `glab issue update <id>`, flags `-t/--title` and `-d/--description` (confirmed via `glab issue update --help`). This differs from the existing `add_labels`/`remove_label` methods in this same file, which call `glab issue edit --add-label`/`--remove-label` — those calls are silently no-ops against real `glab` today. That is a **pre-existing bug**, out of scope for this task; do not fix it here. Note it for a follow-up Issue after this plan ships.

- [ ] **Step 1: Write the failing test**

Add to `mod tests` in `crates/gitlab/src/issue.rs`, after `test_should_return_platform_error_when_glab_fails_for_remove_label`:

```rust
    #[tokio::test]
    async fn test_should_edit_issue_via_update_and_view_result() {
        // Sequence: 1. `glab issue update` → succeeds
        //           2. `glab issue view --output json` → returns updated issue JSON
        let runner = SequencedMockCommandRunner::from_results(&[
            (true, ""),
            (
                true,
                r#"{"iid":42,"title":"New title","description":"orig","state":"opened","labels":[],"author":{"username":"admin","id":1},"assignees":[],"created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-02T00:00:00Z","web_url":"https://gitlab.com/owner/repo/-/issues/42"}"#,
            ),
        ]);
        let provider = GitLabIssueProvider::with_runner("owner/repo", runner);

        let issue = provider
            .edit(
                42,
                gitflow_core::issue::EditIssueArgs {
                    title: Some("New title".to_string()),
                    body: None,
                },
            )
            .await
            .expect("edit should succeed");

        assert_eq!(issue.number, 42);
        assert_eq!(issue.title, "New title");
    }

    #[tokio::test]
    async fn test_should_send_title_and_description_flags_for_edit() {
        let runner = MockCommandRunner::success("");
        let provider = GitLabIssueProvider::with_runner("owner/repo", runner.clone());

        let _ = provider
            .edit(
                7,
                gitflow_core::issue::EditIssueArgs {
                    title: Some("T".to_string()),
                    body: Some("B".to_string()),
                },
            )
            .await;

        assert_eq!(
            runner.recorded_calls()[0].1,
            vec![
                "issue", "update", "7", "--repo", "owner/repo", "--title", "T", "--description",
                "B"
            ]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_glab_fails_for_edit() {
        let runner = MockCommandRunner::failure(r#"{"message": "Not found"}"#, 256);
        let provider = GitLabIssueProvider::with_runner("owner/repo", runner);

        let result = provider
            .edit(42, gitflow_core::issue::EditIssueArgs::default())
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-gitlab test_should_edit_issue_via_update_and_view_result`
Expected: FAIL with "no method named `edit` found"

- [ ] **Step 3: Write minimal implementation**

Update the import at the top of `crates/gitlab/src/issue.rs`:

```rust
use gitflow_core::{
    CoreError, Result,
    issue::{CreateIssueArgs, EditIssueArgs, IssueData, IssueProvider, ListIssueArgs},
    types::{CommentData, Label, State, UserSummary},
};
```

Add to `impl<R: CommandRunner + 'static> IssueProvider for GitLabIssueProvider<R>`, after `create` and before `list`:

```rust
    /// 编辑 Issue 的标题和/或正文。
    ///
    /// 调用 `glab issue update <number> --repo <repo> [--title T] [--description D]`
    /// （`glab` 没有 `issue edit` 子命令，标题/正文变更走 `issue update`），
    /// 成功后通过 [`view`](Self::view) 重新拉取最新数据并返回。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在或 `glab` CLI 调用失败时返回错误。
    async fn edit(&self, number: u64, args: EditIssueArgs) -> Result<IssueData> {
        debug!(repo = %self.repo, number, "spawning `glab issue update`");

        let number_str = number.to_string();
        let mut cmd_args: Vec<&str> = vec!["issue", "update", &number_str, "--repo", &self.repo];

        if let Some(title) = &args.title {
            cmd_args.push("--title");
            cmd_args.push(title);
        }
        if let Some(body) = &args.body {
            cmd_args.push("--description");
            cmd_args.push(body);
        }

        let output = self
            .runner
            .run("glab", &cmd_args)
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
        }

        self.view(number).await
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p gitflow-gitlab issue::`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/gitlab/src/issue.rs
git commit -m "feat(gitlab): implement IssueProvider::edit via glab issue update"
```

---

### Task 4: GitCode adapter — `edit`

**Files:**
- Modify: `crates/gitcode/src/issue.rs`

**Interfaces:**
- Consumes: `EditIssueArgs` from `gitflow_core::issue`; `parse_gitcode_error`; `self.view(number).await`; `crate::gitcode_binary()`.
- Produces: `GitCodeIssueProvider::edit`.

- [ ] **Step 1: Write the failing test**

Add to `mod tests` in `crates/gitcode/src/issue.rs`, after `test_should_return_platform_error_when_gc_fails_for_remove_label`:

```rust
    #[tokio::test]
    async fn test_should_edit_issue_and_view_result() {
        // Sequence: 1. `gitcode issue edit` → succeeds
        //           2. `gitcode issue view --json` → returns updated issue JSON
        let runner = SequencedMockCommandRunner::from_results(&[
            (true, ""),
            (
                true,
                r#"{"number":42,"title":"New title","body":"orig","state":"open","labels":[],"author":{"login":"octocat","id":"1"},"assignees":[],"createdAt":"2026-01-01T00:00:00Z","updatedAt":"2026-01-02T00:00:00Z","url":"https://gitcode.com/owner/repo/issues/42"}"#,
            ),
        ]);
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner);

        let issue = provider
            .edit(
                42,
                gitflow_core::issue::EditIssueArgs {
                    title: Some("New title".to_string()),
                    body: None,
                },
            )
            .await
            .expect("edit should succeed");

        assert_eq!(issue.number, 42);
        assert_eq!(issue.title, "New title");
    }

    #[tokio::test]
    async fn test_should_invoke_issue_edit_subcommand_with_provided_fields() {
        let runner = RecordingMockRunner::success("");
        let provider = GitCodeIssueProvider::with_runner("o/r", runner.clone());

        // view() call afterward will fail on empty stdout — irrelevant to this test,
        // which only asserts the first (edit) call's argv.
        let _ = provider
            .edit(
                54,
                gitflow_core::issue::EditIssueArgs {
                    title: Some("T".to_string()),
                    body: Some("B".to_string()),
                },
            )
            .await;

        assert_eq!(
            runner.calls()[0],
            vec![
                "issue", "edit", "54", "-R", "o/r", "--title", "T", "--body", "B"
            ]
        );
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_edit() {
        let runner = MockCommandRunner::failure("not found", 256);
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner);

        let result = provider
            .edit(42, gitflow_core::issue::EditIssueArgs::default())
            .await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-gitcode test_should_edit_issue_and_view_result`
Expected: FAIL with "no method named `edit` found"

- [ ] **Step 3: Write minimal implementation**

Update the import at the top of `crates/gitcode/src/issue.rs`:

```rust
use gitflow_core::{
    CoreError, Result, Session,
    issue::{CreateIssueArgs, EditIssueArgs, IssueData, IssueProvider, ListIssueArgs},
    types::{CommentData, Label, State, UserSummary},
};
```

Add to `impl<R: CommandRunner + 'static> IssueProvider for GitCodeIssueProvider<R>`, after `create` and before `list`:

```rust
    /// 编辑 Issue 的标题和/或正文。
    ///
    /// 调用 `<gitcode_binary> issue edit <number> -R <repo> [--title T] [--body B]`，
    /// 成功后通过 [`view`](Self::view) 重新拉取最新数据并返回（不解析 `edit` 自身的
    /// stdout，避免依赖未经验证的响应结构）。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在或 `gitcode` CLI 调用失败时返回错误。
    async fn edit(&self, number: u64, args: EditIssueArgs) -> Result<IssueData> {
        let binary = crate::gitcode_binary();
        let number_str = number.to_string();
        debug!(repo = %self.repo, number, "spawning gitcode issue edit");

        let mut cmd_args: Vec<&str> = vec!["issue", "edit", &number_str, "-R", &self.repo];
        if let Some(title) = &args.title {
            cmd_args.push("--title");
            cmd_args.push(title);
        }
        if let Some(body) = &args.body {
            cmd_args.push("--body");
            cmd_args.push(body);
        }

        let output = self
            .runner
            .run(&binary, &cmd_args)
            .await
            .map_err(|e| CoreError::Platform(format!("{e}")))?;

        if !output.status.success() {
            return Err(parse_gitcode_error(&output.stderr).into());
        }

        self.view(number).await
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p gitflow-gitcode issue::`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/gitcode/src/issue.rs
git commit -m "feat(gitcode): implement IssueProvider::edit via gitcode issue edit"
```

---

### Task 5: CLI — `gf issue edit` subcommand

**Files:**
- Modify: `apps/cli/src/commands/issue.rs`

**Interfaces:**
- Consumes: `IssueProvider::edit` (Tasks 2-4), `EditIssueArgs` from `gitflow_core::issue`, `resolve_body()` (existing, in this file).
- Produces: `IssueCommand::Edit { number: u64, title: Option<String>, body: Option<String>, body_file: Option<String> }`; a private `ensure_edit_has_changes(&Option<String>, &Option<String>) -> miette::Result<()>` helper.

- [ ] **Step 1: Write the failing test**

Add to `mod tests` in `apps/cli/src/commands/issue.rs`, after `test_should_reject_both_body_and_body_file`:

```rust
    #[test]
    fn test_should_error_when_edit_has_no_changes() {
        let result = ensure_edit_has_changes(&None, &None);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Nothing to edit"));
    }

    #[test]
    fn test_should_allow_edit_with_title_only() {
        let result = ensure_edit_has_changes(&Some("T".into()), &None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_should_allow_edit_with_body_only() {
        let result = ensure_edit_has_changes(&None, &Some("B".into()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_should_allow_edit_with_both_title_and_body() {
        let result = ensure_edit_has_changes(&Some("T".into()), &Some("B".into()));
        assert!(result.is_ok());
    }
```

And add to the `test_should_parse_issue_*` group near `test_should_parse_issue_view`:

```rust
    #[test]
    fn test_should_parse_issue_edit_with_title() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "gitflow", "issue", "edit", "42", "--title", "New title",
        ])
        .expect("parse");
        match cli.command {
            crate::Commands::Issue(IssueCommand::Edit {
                number,
                title,
                body,
                body_file,
            }) => {
                assert_eq!(number, 42);
                assert_eq!(title, Some("New title".to_string()));
                assert!(body.is_none());
                assert!(body_file.is_none());
            }
            _ => panic!("Expected IssueCommand::Edit"),
        }
    }

    #[test]
    fn test_should_parse_issue_edit_with_body() {
        use clap::Parser;
        let cli =
            crate::Cli::try_parse_from(["gitflow", "issue", "edit", "42", "--body", "New body"])
                .expect("parse");
        match cli.command {
            crate::Commands::Issue(IssueCommand::Edit { number, body, .. }) => {
                assert_eq!(number, 42);
                assert_eq!(body, Some("New body".to_string()));
            }
            _ => panic!("Expected IssueCommand::Edit"),
        }
    }

    #[test]
    fn test_should_parse_issue_edit_with_body_file() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "gitflow",
            "issue",
            "edit",
            "42",
            "--body-file",
            "/tmp/body.md",
        ])
        .expect("parse");
        match cli.command {
            crate::Commands::Issue(IssueCommand::Edit { body_file, .. }) => {
                assert_eq!(body_file, Some("/tmp/body.md".to_string()));
            }
            _ => panic!("Expected IssueCommand::Edit"),
        }
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-cli test_should_error_when_edit_has_no_changes`
Expected: FAIL with "cannot find function `ensure_edit_has_changes`" (and the parse tests fail with "no variant `Edit`")

- [ ] **Step 3: Write minimal implementation**

Update the import block at the top of `apps/cli/src/commands/issue.rs`:

```rust
use gitflow_core::{
    CliOutput,
    issue::{CreateIssueArgs, EditIssueArgs, IssueProvider, ListIssueArgs},
    types::State,
};
```

Add the `Edit` variant to `IssueCommand`, after `Create` and before `List`:

```rust
    /// 编辑 Issue 的标题和/或正文（部分更新）。
    Edit {
        /// Issue 编号。
        number: u64,

        /// 新标题（可选）。
        #[arg(long)]
        title: Option<String>,

        /// 新正文（可选，与 `--body-file` 二选一）。
        #[arg(long)]
        body: Option<String>,

        /// 从文件读取新正文（可选）。
        #[arg(long = "body-file")]
        body_file: Option<String>,
    },
```

Add a match arm in `handle()`, after the `IssueCommand::Create { .. }` arm and before `IssueCommand::List { .. }`:

```rust
        IssueCommand::Edit {
            number,
            title,
            body,
            body_file,
        } => {
            let resolved_body = resolve_body(body, body_file)?;
            ensure_edit_has_changes(&title, &resolved_body)?;
            let args = EditIssueArgs {
                title,
                body: resolved_body,
            };
            let issue = provider
                .edit(number, args)
                .await
                .map_err(|e| miette::miette!("Failed to edit issue #{number}: {e}"))?;
            let output = CliOutput::success(issue, platform, "issue edit");
            print_output(&output, &output_format)?;
        }
```

Add the helper function, after `resolve_comment_body`:

```rust
/// 校验编辑参数：`title` 与 `body` 至少提供一个。
///
/// # Errors
///
/// 当两者都为 `None` 时返回错误。
fn ensure_edit_has_changes(title: &Option<String>, body: &Option<String>) -> miette::Result<()> {
    if title.is_none() && body.is_none() {
        return Err(miette::miette!(
            "Nothing to edit. Provide --title and/or --body/--body-file."
        ));
    }
    Ok(())
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p gitflow-cli issue::`
Expected: PASS (all tests in `apps/cli/src/commands/issue.rs`, including the 7 new ones)

- [ ] **Step 5: Run the full workspace gate**

Run: `make build && make test && cargo +nightly fmt && cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: all green, no warnings

- [ ] **Step 6: Commit**

```bash
git add apps/cli/src/commands/issue.rs
git commit -m "feat(cli): add gf issue edit subcommand"
```

---

## Post-Plan Follow-Up (not part of this plan)

- File a separate Issue: GitLab's `add_labels`/`remove_label` call `glab issue edit --add-label`/`--remove-label`, but `glab` (verified at v1.115.0) has no `issue edit` subcommand — the call silently falls through to help output with exit 0, so these two operations are currently no-ops against real GitLab. Fix belongs in its own Issue/PR, not bundled into this one.
