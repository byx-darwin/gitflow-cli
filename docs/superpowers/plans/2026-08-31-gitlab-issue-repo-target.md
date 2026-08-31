# GitLab Issue `--repo` 目标修复 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复 `gf issue add-label`/`remove-label`（及所有 GitLab `issue` 动词）在自建 GitLab 实例上因 `--repo` 裸 `owner/repo` 缺失 host 信息导致的执行失败，并让 `glab` 失败的真实 stderr 可被日志观察到。

**Architecture:** `GitLabIssueProvider` 新增 `repo_target` 字段，专门承载传给 `glab issue ...` 子命令 `--repo` 参数的值；当调用方能提供完整 git remote URL 时，`repo_target` 使用该 URL（`glab` 官方文档确认 `--repo` 接受完整 URL/Git URL 形式），否则回退到裸 `owner/repo`（向后兼容）。原 `repo` 字段保留，继续供 REST notes API 路径编码使用。CLI 层的 `resolve_platform()` 把已经在手头的 remote URL 一并返回并透传到 `commands::issue::handle()`，仅在 GitLab 平台且用户未通过 `--repo` 覆盖仓库时使用它。

**Tech Stack:** Rust 2024 workspace；`gitflow-gitlab`（通过 `glab` CLI）、`gitflow-core`、`apps/cli`。

**Spec:** `docs/superpowers/specs/2026-08-31-gitlab-issue-repo-target-design.md`

## Global Constraints

- 不改动 `deny.toml` / `.pre-commit-config.yaml` / `rust-toolchain.toml`。
- 不引入新依赖（stderr 日志验证不新增 `tracing-subscriber`/`tracing-test`，此项目现有测试中没有先例覆盖 `tracing::debug!` 输出，遵循现状）。
- `#![forbid(unsafe_code)]`、禁止 `unwrap()`/`expect()` 用于生产代码路径（测试代码中的 `expect()` 遵循既有惯例）。
- 所有新增 public 方法需要文档注释（含 `# Errors`/`# Panics` 视情况）。
- 每个任务完成后运行 `make test` 与 `make lint`（或等价的 `cargo test -p <crate>` / `cargo clippy --all-targets --all-features -- -D warnings`）确认绿色再提交。

---

### Task 1: `GitLabIssueProvider` 新增 `repo_target` 字段与测试专用构造器

**Files:**
- Modify: `crates/gitlab/src/issue.rs:42-119`（struct 定义 + 三个既有构造器）
- Test: `crates/gitlab/src/issue.rs`（`mod tests` 内新增用例）

**Interfaces:**
- Consumes: 无（本任务是最内层改动）
- Produces:
  - `GitLabIssueProvider<R>.repo_target: String`（私有字段）
  - `GitLabIssueProvider<R: CommandRunner>::with_runner_and_repo_target(repo: impl Into<String>, repo_target: impl Into<String>, runner: R) -> Self`（测试用，Task 2+ 的测试会用到）
  - `GitLabIssueProvider<RealCommandRunner>::with_remote_url(repo: impl Into<String>, remote_url: impl Into<String>) -> Self`（生产用，Task 5 会用到）
  - 既有 `new()` / `with_session()` / `with_runner()` 行为不变（`repo_target` 默认等于 `repo`）

- [ ] **Step 1: 写失败测试**，验证新增的测试专用构造器存在且能让 `--repo` 使用与 `repo` 不同的目标值（先针对 `add_labels`，其余动词在后续任务改造后自然满足）：

在 `crates/gitlab/src/issue.rs` 的 `mod tests` 内、`test_should_call_issue_update_with_label_flag_for_add_labels` 测试之后插入：

```rust
    #[tokio::test]
    async fn test_should_use_explicit_repo_target_for_add_labels() {
        let runner = MockCommandRunner::success("");
        let provider = GitLabIssueProvider::with_runner_and_repo_target(
            "owner/repo",
            "https://192.168.230.23/iproost/proxy/api-src.git",
            runner.clone(),
        );

        let result = provider.add_labels(42, &["priority:medium".to_string()]).await;

        assert!(result.is_ok(), "expected Ok, got {result:?}");
        assert_eq!(
            runner.recorded_calls()[0].1,
            vec![
                "issue",
                "update",
                "42",
                "--repo",
                "https://192.168.230.23/iproost/proxy/api-src.git",
                "--label",
                "priority:medium",
            ]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>()
        );
    }
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p gitflow-gitlab test_should_use_explicit_repo_target_for_add_labels`
Expected: 编译失败——`with_runner_and_repo_target` 不存在。

- [ ] **Step 3: 实现最小改动**

在 `crates/gitlab/src/issue.rs:42-48` 将 struct 定义改为：

```rust
#[derive(Debug, Clone)]
pub struct GitLabIssueProvider<R: CommandRunner = RealCommandRunner> {
    /// GitLab `namespace/project`，如 `"gitlab-org/gitlab"`。
    ///
    /// 供 REST notes API 路径编码（[`encode_project_path`]）使用，始终是裸
    /// `owner/repo` 形式，不受 [`repo_target`](Self::repo_target) 影响。
    repo: String,
    /// 传给 `glab issue ...` 子命令 `--repo` 参数的目标字符串。
    ///
    /// 默认等于 `repo`；通过 [`with_remote_url`](GitLabIssueProvider::with_remote_url)
    /// 构造时为完整 git remote URL，用于在自建 GitLab 实例上显式锁定 host，
    /// 避免仅传裸 `OWNER/REPO` 时 `glab` 的 host 探测歧义
    /// （参见 <https://gitlab.com/gitlab-org/cli/-/issues/1370>）。
    repo_target: String,
    /// 用于执行 `glab` CLI 命令的 runner。
    runner: R,
}
```

将 `impl GitLabIssueProvider<RealCommandRunner>` 块（原 50-72 行）中的 `new` 与 `with_session` 改为：

```rust
impl GitLabIssueProvider<RealCommandRunner> {
    /// 创建新的 GitLab Issue 提供者，使用真实的进程执行器。
    ///
    /// `repo` 格式为 `namespace/project`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> Self {
        let repo = repo.into();
        Self {
            repo_target: repo.clone(),
            repo,
            runner: RealCommandRunner,
        }
    }

    /// Create a new provider from a shared [`Session`].
    ///
    /// This enables state reuse across multiple operations in workflow chains.
    #[must_use]
    pub fn with_session(session: &gitflow_core::Session) -> Self {
        let repo = session.repo.clone();
        Self {
            repo_target: repo.clone(),
            repo,
            runner: RealCommandRunner,
        }
    }

    /// 使用完整 git remote URL 作为 `glab issue ...` 的 `--repo` 目标创建提供者。
    ///
    /// `repo` 仍为裸 `namespace/project`（供 REST notes API 路径编码使用），
    /// `remote_url` 为完整 git remote URL。`glab` 官方文档确认 `--repo` 接受
    /// 完整 URL/Git URL 形式，借此在自建 GitLab 实例上显式锁定 host。
    #[must_use]
    pub fn with_remote_url(repo: impl Into<String>, remote_url: impl Into<String>) -> Self {
        Self {
            repo: repo.into(),
            repo_target: remote_url.into(),
            runner: RealCommandRunner,
        }
    }
}
```

将 `impl<R: CommandRunner> GitLabIssueProvider<R>` 块中的 `with_runner`（原 74-85 行）改为：

```rust
impl<R: CommandRunner> GitLabIssueProvider<R> {
    /// 使用自定义 [`CommandRunner`] 创建提供者。
    ///
    /// 主要用于测试，可注入模拟 runner 以控制 `glab` CLI 的输出。
    /// `repo` 格式为 `namespace/project`。
    #[must_use]
    pub fn with_runner(repo: impl Into<String>, runner: R) -> Self {
        let repo = repo.into();
        Self {
            repo_target: repo.clone(),
            repo,
            runner,
        }
    }

    /// 使用自定义 [`CommandRunner`] 并显式指定 `--repo` 目标创建提供者。
    ///
    /// 主要用于测试，验证 `repo_target`（如完整 remote URL）被正确传给 `glab`，
    /// 无需引入真实进程执行器。
    #[must_use]
    pub fn with_runner_and_repo_target(
        repo: impl Into<String>,
        repo_target: impl Into<String>,
        runner: R,
    ) -> Self {
        Self {
            repo: repo.into(),
            repo_target: repo_target.into(),
            runner,
        }
    }

    // ... 既有 ensure_label_exists 保持不变（本任务不改）
```

此时代码仍不会通过编译（`add_labels` 内部仍用 `&self.repo` 传 `--repo`），因此测试仍会失败，但失败原因变为断言不匹配（预期完整 URL，实际是 `owner/repo`）——这正是 Step 2 之后的正确 RED 状态。

- [ ] **Step 4: 运行测试确认仍失败（预期的中间态）**

Run: `cargo test -p gitflow-gitlab test_should_use_explicit_repo_target_for_add_labels`
Expected: 编译通过，断言失败：`--repo` 实际值是 `"owner/repo"` 而非完整 URL。

- [ ] **Step 5: 提交（中间态，仅新增构造器，尚未接线）**

```bash
git add crates/gitlab/src/issue.rs
git commit -m "feat(gitlab): add repo_target field and test-only constructor to GitLabIssueProvider"
```

---

### Task 2: 把 `add_labels` / `remove_label` 的 `--repo` 改为使用 `repo_target`

**Files:**
- Modify: `crates/gitlab/src/issue.rs:561-653`（`add_labels`、`remove_label`）
- Test: 同文件 `mod tests`

**Interfaces:**
- Consumes: Task 1 的 `repo_target` 字段、`with_runner_and_repo_target`
- Produces: `add_labels`/`remove_label` 的 `--repo` 参数值等于 `self.repo_target`

- [ ] **Step 1: 确认 Task 1 留下的测试当前失败**（已在 Task 1 Step 4 验证，此处直接进入实现）

- [ ] **Step 2: 实现**

在 `add_labels`（原 569-579 行）：

```rust
        let labels_joined = labels.join(",");
        let number_str = number.to_string();
        let cmd_args: Vec<&str> = vec![
            "issue",
            "update",
            &number_str,
            "--repo",
            &self.repo_target,
            "--label",
            &labels_joined,
        ];
```

在 `remove_label`（原 630-644 行）：

```rust
        let number_str = number.to_string();
        let output = self
            .runner
            .run(
                "glab",
                &[
                    "issue",
                    "update",
                    &number_str,
                    "--repo",
                    &self.repo_target,
                    "--unlabel",
                    label,
                ],
            )
```

同时把 `ensure_label_exists`（原 94-104 行）里 `label create` 的 `--repo` 也改为 `&self.repo_target`（标签自动创建同样应该走同一目标，否则重试路径又会绕回裸 repo 引发 host 歧义）：

```rust
    async fn ensure_label_exists(&self, name: &str) -> Result<()> {
        debug!(repo = %self.repo, name, "auto-creating missing label via `glab label create`");

        let output = self
            .runner
            .run(
                "glab",
                &[
                    "label", "create", "--name", name, "--color", "ededed", "--repo",
                    &self.repo_target,
                ],
            )
```

- [ ] **Step 3: 运行测试确认通过**

Run: `cargo test -p gitflow-gitlab issue::tests`
Expected: 全部 PASS，包括 Task 1 新增的 `test_should_use_explicit_repo_target_for_add_labels`，以及既有的 `test_should_call_issue_update_with_label_flag_for_add_labels`（`repo_target` 默认等于 `repo`，回归不受影响）、`test_should_call_issue_update_with_unlabel_flag_for_remove_label`、`test_should_auto_create_label_and_retry_on_add_labels_glab` 等。

- [ ] **Step 4: 补充 `remove_label` 的显式 `repo_target` 用例**

在 `test_should_call_issue_update_with_unlabel_flag_for_remove_label` 之后插入：

```rust
    #[tokio::test]
    async fn test_should_use_explicit_repo_target_for_remove_label() {
        let runner = MockCommandRunner::success("");
        let provider = GitLabIssueProvider::with_runner_and_repo_target(
            "owner/repo",
            "https://192.168.230.23/iproost/proxy/api-src.git",
            runner.clone(),
        );

        let result = provider.remove_label(42, "priority:medium").await;

        assert!(result.is_ok(), "expected Ok, got {result:?}");
        assert_eq!(
            runner.recorded_calls()[0].1,
            vec![
                "issue",
                "update",
                "42",
                "--repo",
                "https://192.168.230.23/iproost/proxy/api-src.git",
                "--unlabel",
                "priority:medium",
            ]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>()
        );
    }
```

Run: `cargo test -p gitflow-gitlab test_should_use_explicit_repo_target_for_remove_label`
Expected: PASS immediately (implementation already done in Step 2).

- [ ] **Step 5: 提交**

```bash
git add crates/gitlab/src/issue.rs
git commit -m "fix(gitlab): route add_labels/remove_label --repo through repo_target"
```

---

### Task 3: 把其余 GitLab issue 动词的 `--repo` 也统一改为 `repo_target`

**Files:**
- Modify: `crates/gitlab/src/issue.rs`（`create` L239-246、`edit` L326、`list` L352、`view` L399-409、`close` L440、`reopen` L468）
- Test: 同文件 `mod tests`

**Interfaces:**
- Consumes: Task 1/2 的 `repo_target` 字段
- Produces: 全部 `glab issue ...` 子命令的 `--repo` 参数统一来自 `self.repo_target`；`glab api ...`（`comment`/`list_comments`）不受影响，继续用 `self.repo`

- [ ] **Step 1: 写失败测试**（以 `view` 为代表，验证统一改造后其余动词也生效；`close`/`reopen` 已有断言完整调用参数的既有测试，改造后需同步更新其断言仓库值保持 `owner/repo`——不需要新增，只需保证不回归）

在 `mod tests` 内、`test_should_reopen_issue_without_output_json_flag_and_refetch_via_view` 之后插入：

```rust
    #[tokio::test]
    async fn test_should_use_explicit_repo_target_for_view() {
        let runner = MockCommandRunner::success(
            r#"{"iid":42,"title":"Fix","state":"opened","description":null,"labels":[]}"#,
        );
        let provider = GitLabIssueProvider::with_runner_and_repo_target(
            "owner/repo",
            "https://192.168.230.23/iproost/proxy/api-src.git",
            runner.clone(),
        );

        let issue = provider.view(42).await.expect("view should succeed");

        assert_eq!(issue.number, 42);
        assert!(
            runner.recorded_calls()[0]
                .1
                .contains(&"https://192.168.230.23/iproost/proxy/api-src.git".to_string())
        );
    }
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p gitflow-gitlab test_should_use_explicit_repo_target_for_view`
Expected: FAIL — 调用参数里仍是 `"owner/repo"`。

- [ ] **Step 3: 实现——把以下 6 处 `--repo` 引用的 `&self.repo` 改为 `&self.repo_target`**

`create`（原 239-246 行）：

```rust
        let mut cmd_args: Vec<&str> = vec![
            "issue",
            "create",
            "--repo",
            &self.repo_target,
            "--title",
            &args.title,
        ];
```

`edit`（原 326 行）：

```rust
        let mut cmd_args: Vec<&str> = vec!["issue", "update", &number_str, "--repo", &self.repo_target];
```

`list`（原 351-352 行）：

```rust
        let mut cmd_args: Vec<&str> =
            vec!["issue", "list", "--repo", &self.repo_target, "--output", "json"];
```

`view`（原 399-409 行）：

```rust
            .run(
                "glab",
                &[
                    "issue",
                    "view",
                    &number_str,
                    "--repo",
                    &self.repo_target,
                    "--output",
                    "json",
                ],
            )
```

`close`（原 440 行）：

```rust
                &["issue", "close", &number_str, "--repo", &self.repo_target],
```

`reopen`（原 468 行）：

```rust
                &["issue", "reopen", &number_str, "--repo", &self.repo_target],
```

- [ ] **Step 4: 运行完整 issue 模块测试确认通过且无回归**

Run: `cargo test -p gitflow-gitlab issue::tests`
Expected: 全部 PASS。既有的 `test_should_close_issue_without_output_json_flag_and_refetch_via_view`、`test_should_reopen_issue_without_output_json_flag_and_refetch_via_view`、`test_should_edit_issue_via_update_and_view_result` 等断言 `"owner/repo"` 的用例继续通过——因为它们全部通过 `with_runner("owner/repo", ...)` 构造，`repo_target` 默认等于 `repo`。

- [ ] **Step 5: 提交**

```bash
git add crates/gitlab/src/issue.rs
git commit -m "fix(gitlab): route remaining issue verbs' --repo through repo_target"
```

---

### Task 4: `parse_glab_error` 记录真实 stderr 到日志

**Files:**
- Modify: `crates/gitlab/src/error.rs:11-13`（`parse_glab_error` 入口）

**Interfaces:**
- Consumes: 无
- Produces: 无新增公共接口；行为变化——`glab` 调用失败时，原始 stderr 通过 `tracing::debug!` 可被观察（`RUST_LOG=debug` 或 `gf --output json` 场景下的调试日志）

**说明：** 本项目当前没有对 `tracing::debug!` 输出做单元测试的先例（`grep` 全仓库确认），引入 `tracing-subscriber`/`tracing-test` 仅为覆盖这一行日志不符合“最小依赖”的约束。此步骤按既有 `raw_stderr` 字段已有测试覆盖（`crates/gitlab/src/error.rs` 现有的 `test_should_parse_glab_json_error_to_platform_cli_error` 等已断言 `raw_stderr` 非空）间接验证数据源正确，日志行为本身通过代码走读确认，不新增测试基础设施。

- [ ] **Step 1: 修改**

在 `crates/gitlab/src/error.rs` 的 `parse_glab_error` 函数体最前面（`let text = String::from_utf8_lossy(stderr);` 之后）新增：

```rust
pub fn parse_glab_error(stderr: &[u8]) -> PlatformCliError {
    let text = String::from_utf8_lossy(stderr);
    tracing::debug!(raw_stderr = %text, "glab command failed");

    let is_auth_failure = |t: &str| {
```

- [ ] **Step 2: 运行既有测试确认无回归**

Run: `cargo test -p gitflow-gitlab error::tests`
Expected: 全部 PASS（本改动不影响任何断言值，只新增一行日志副作用）。

- [ ] **Step 3: 提交**

```bash
git add crates/gitlab/src/error.rs
git commit -m "fix(gitlab): log raw glab stderr on CLI failure for diagnosability"
```

---

### Task 5: `resolve_platform` 返回 remote URL 并透传到 `async_main`/`router`

**Files:**
- Modify: `apps/cli/src/main.rs:96-135, 178-224, 269-315`

**Interfaces:**
- Consumes: 无
- Produces: `resolve_platform(...) -> miette::Result<(String, String, String)>`（新增第三个返回值 `remote_url`）；`async_main(cli, platform, repo, remote_url)`；`router(command, platform, repo, remote_url, output)`

- [ ] **Step 1: 修改 `resolve_platform` 签名与返回值**（原 269-315 行）

```rust
fn resolve_platform(cli_platform: Option<PlatformArg>) -> miette::Result<(String, String, String)> {
    // Get git remote URL (sync — see doc comment above).
    let output = std::process::Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .map_err(|e| {
            miette::miette!("Failed to get git remote URL: {e}\nAre you in a git repository?")
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(miette::miette!(
            "git remote get-url origin failed: {}\nAre you in a git repository?",
            stderr.trim()
        ));
    }

    let remote_url = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Determine platform.
    let platform = if let Some(p) = cli_platform {
        match p {
            PlatformArg::Github => "github",
            PlatformArg::Gitlab => "gitlab",
            PlatformArg::Gitcode => "gitcode",
        }
        .to_string()
    } else {
        let detection = gitflow_core::platform::Platform::detect_from_remote_url(&remote_url);
        if !detection.is_explicit() {
            eprintln!(
                "warning: unrecognized domain in remote URL: {remote_url}\nDefaulting to GitLab \
                 adapter. Use --platform to specify explicitly."
            );
        }
        format!("{:?}", detection.platform).to_lowercase()
    };

    // Extract owner/repo
    let repo = extract_repo_from_url(&remote_url)
        .ok_or_else(|| miette::miette!("Unable to parse owner/repo from URL: {remote_url}"))?;

    Ok((platform, repo, remote_url))
}
```

（唯一变化：函数签名的返回类型多一个 `String`，末尾 `Ok((platform, repo))` 改为 `Ok((platform, repo, remote_url))`；`remote_url` 变量此前就存在，本次不再丢弃。）

- [ ] **Step 2: 更新调用点**（原 96-135 行区域）

把：

```rust
    let (platform, repo) = if platform_needed {
        match resolve_platform(cli.platform.clone()) {
            Ok(pr) => pr,
            Err(e) => {
                report_error_noninteractive(
                    &command_name,
                    "unknown",
                    &e.to_string(),
                    "PLATFORM_ERROR",
                );
                eprintln!("{e:?}");
                return std::process::ExitCode::from(1);
            }
        }
    } else {
        ("unknown".to_string(), String::new())
    };

    // Block on the async main, handling graceful shutdown signals
    match rt.block_on(async_main(cli, &platform, &repo)) {
```

改为：

```rust
    let (platform, repo, remote_url) = if platform_needed {
        match resolve_platform(cli.platform.clone()) {
            Ok(pr) => pr,
            Err(e) => {
                report_error_noninteractive(
                    &command_name,
                    "unknown",
                    &e.to_string(),
                    "PLATFORM_ERROR",
                );
                eprintln!("{e:?}");
                return std::process::ExitCode::from(1);
            }
        }
    } else {
        ("unknown".to_string(), String::new(), String::new())
    };

    // Block on the async main, handling graceful shutdown signals
    match rt.block_on(async_main(cli, &platform, &repo, &remote_url)) {
```

- [ ] **Step 3: 扩展 `async_main` 与 `router` 签名**（原 178-224 行）

```rust
async fn async_main(cli: Cli, platform: &str, repo: &str, remote_url: &str) -> miette::Result<()> {
    // Skills/Completions/Workflow/Update don't need native CLI — skip prerequisite check
    if !matches!(
        cli.command,
        Commands::Skills(_)
            | Commands::Completions(_)
            | Commands::Workflow(_)
            | Commands::Update(_)
            | Commands::Doctor(_)
    ) {
        commands::prerequisites::check(platform).map_err(|e| miette::miette!("{e}"))?;
    }

    tokio::select! {
        result = router(cli.command, platform, repo, remote_url, cli.output) => result,
        () = async {
            match tokio::signal::ctrl_c().await {
                Ok(()) => tracing::info!("Received shutdown signal, exiting gracefully"),
                Err(e) => tracing::warn!("Failed to install Ctrl+C handler: {e}"),
            }
        } => {
            Ok(())
        }
    }
}

/// Dispatch a subcommand to the appropriate handler.
async fn router(
    command: Commands,
    platform: &str,
    repo: &str,
    remote_url: &str,
    output: OutputFormat,
) -> miette::Result<()> {
    match command {
        Commands::Issue(cmd) => commands::issue::handle(cmd, platform, repo, remote_url, output).await,
        Commands::Pr(cmd) => commands::pr::handle(cmd, platform, repo, output).await,
        Commands::Release(cmd) => commands::release::handle(cmd, platform, repo, output).await,
        Commands::Review(cmd) => commands::review::handle(cmd, platform, repo, output).await,
        Commands::Auth(cmd) => commands::auth::handle(cmd, platform, repo, output).await,
        Commands::Label(cmd) => commands::label::handle_label(cmd, platform, repo, output).await,
        Commands::Milestone(cmd) => {
            commands::label::handle_milestone(cmd, platform, repo, output).await
        }
        Commands::Commit(cmd) => commands::commit::handle(cmd, platform, repo, output).await,
        Commands::Pipeline(cmd) => commands::pipeline::handle(cmd, platform, repo, output).await,
        // ...其余分支保持不变（只有 Issue 分支多传 remote_url）
```

**注意：** 只修改 `Commands::Issue` 这一分支的调用（追加 `remote_url` 实参），其余分支原样保留，不要整体重写这个 `match`——先用 `Read` 工具核对 `router` 函数当前的完整分支列表（本计划编写时看到的分支到 `Pipeline` 为止，后面可能还有别的命令），逐个分支对照后只改 `Issue` 那一行。

- [ ] **Step 4: 编译检查（无新增单元测试——`resolve_platform` 本身依赖真实 `git` 进程，既有测试套件里从未对它做单元测试，保持现状；`extract_repo_from_url` 的既有测试不受影响）**

Run: `cargo build -p gitflow-cli`
Expected: 编译成功。

Run: `cargo test -p gitflow-cli`
Expected: 既有测试全部 PASS（`extract_repo_from_url` 相关用例不受影响）。

- [ ] **Step 5: 提交**

```bash
git add apps/cli/src/main.rs
git commit -m "feat(cli): thread git remote URL through resolve_platform/async_main/router"
```

---

### Task 6: `commands/issue.rs::handle` 在 GitLab 平台上使用 `with_remote_url`

**Files:**
- Modify: `apps/cli/src/commands/issue.rs:166-190`
- Test: 同文件 `mod tests`

**Interfaces:**
- Consumes: Task 5 的 `router` 传入的 `remote_url: &str`；Task 1 的 `GitLabIssueProvider::with_remote_url`
- Produces: `pub async fn handle(command: IssueCommand, platform: &str, repo: &str, remote_url: &str, output_format: OutputFormat) -> miette::Result<()>`；纯函数 `fn should_use_remote_url_for_gitlab(command: &IssueCommand) -> bool`

- [ ] **Step 1: 写失败测试**——针对新增的纯判断函数（可测，不依赖 I/O）

在 `mod tests` 内、`test_should_error_when_edit_has_no_changes` 之后插入：

```rust
    #[test]
    fn test_should_use_remote_url_when_no_repo_override() {
        let command = IssueCommand::View { number: 42 };
        assert!(should_use_remote_url_for_gitlab(&command));
    }

    #[test]
    fn test_should_not_use_remote_url_when_create_has_repo_override() {
        let command = IssueCommand::Create {
            title: "t".into(),
            body: None,
            body_file: None,
            label: vec![],
            assignee: vec![],
            repo: Some("other/repo".into()),
        };
        assert!(!should_use_remote_url_for_gitlab(&command));
    }

    #[test]
    fn test_should_use_remote_url_when_create_has_no_repo_override() {
        let command = IssueCommand::Create {
            title: "t".into(),
            body: None,
            body_file: None,
            label: vec![],
            assignee: vec![],
            repo: None,
        };
        assert!(should_use_remote_url_for_gitlab(&command));
    }
```

（先用 `Read` 工具核对 `IssueCommand::View`/`IssueCommand::Create` 的确切字段列表——本计划基于 `handle()` 内已可见的 `title`/`body`/`body_file`/`label`/`assignee`/`repo` 字段推断，如有出入以 `apps/cli/src/commands/issue.rs` 顶部或 `apps/cli/src/cli.rs` 中的枚举定义为准，逐字段对齐。）

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p gitflow-cli should_use_remote_url`
Expected: 编译失败——`should_use_remote_url_for_gitlab` 不存在。

- [ ] **Step 3: 实现**

在 `apps/cli/src/commands/issue.rs` 中，`pub async fn handle` 函数定义之前新增：

```rust
/// 判断 GitLab 平台是否应使用 `remote_url` 作为 `--repo` 目标。
///
/// 当用户通过 `IssueCommand::Create { repo: Some(_), .. }` 显式覆盖仓库时，
/// 该仓库与当前 git remote 不对应，不应强行拼接 `remote_url`。
#[must_use]
fn should_use_remote_url_for_gitlab(command: &IssueCommand) -> bool {
    !matches!(command, IssueCommand::Create { repo: Some(_), .. })
}
```

把 `pub async fn handle` 签名（原 166-171 行）改为：

```rust
pub async fn handle(
    command: IssueCommand,
    platform: &str,
    repo: &str,
    remote_url: &str,
    output_format: OutputFormat,
) -> miette::Result<()> {
```

把 provider 构造（原 181-190 行）改为：

```rust
    let provider: Box<dyn IssueProvider> = match platform {
        "github" => Box::new(GitHubIssueProvider::new(effective_repo)),
        "gitlab" => {
            if should_use_remote_url_for_gitlab(&command) && !remote_url.is_empty() {
                Box::new(GitLabIssueProvider::with_remote_url(
                    effective_repo,
                    remote_url,
                ))
            } else {
                Box::new(GitLabIssueProvider::new(effective_repo))
            }
        }
        "gitcode" => Box::new(GitCodeIssueProvider::new(effective_repo)),
        other => {
            return Err(miette::miette!(
                "Platform '{other}' not yet supported for issue commands"
            ));
        }
    };
```

- [ ] **Step 4: 更新调用点**——`apps/cli/src/main.rs` 的 `router` 函数（Task 5 已改好签名，此处确认实参顺序一致：`commands::issue::handle(cmd, platform, repo, remote_url, output).await`）。

- [ ] **Step 5: 运行测试确认通过**

Run: `cargo test -p gitflow-cli`
Expected: 全部 PASS，包括 Step 1 新增的三个用例。

- [ ] **Step 6: 提交**

```bash
git add apps/cli/src/commands/issue.rs apps/cli/src/main.rs
git commit -m "fix(cli): use git remote URL as GitLab --repo target for issue commands"
```

---

### Task 7: 全量验证

**Files:** 无新增/修改（纯验证任务）

**Interfaces:**
- Consumes: Task 1-6 的全部改动
- Produces: 验证证据（命令输出），用于 Phase 2 quality gate 与 Phase 3 交付前确认

- [ ] **Step 1: 运行受影响 crate 的完整测试**

Run: `cargo test -p gitflow-gitlab -p gitflow-cli`
Expected: 全部 PASS，无回归。

- [ ] **Step 2: 运行 clippy pedantic**

Run: `cargo clippy -p gitflow-gitlab -p gitflow-cli --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: 无警告。若有 pedantic 提示（如 `&self.repo_target` 相关的 borrow 建议），按 CLAUDE.md 的 Clippy Pedantic Alignment 规则修正。

- [ ] **Step 3: 运行 rustfmt 检查**

Run: `cargo +nightly fmt --check`
Expected: 无需改动，或按需 `cargo +nightly fmt` 后重新提交格式化 diff。

- [ ] **Step 4: 运行文档测试（新增 doc comment 中如有代码示例）**

Run: `cargo test -p gitflow-gitlab --doc`
Expected: PASS（本次改动未新增 doctest 示例，仅确认既有 `new()` 的 `no_run` 示例仍编译通过）。

- [ ] **Step 5: 记录验证结果，准备进入 Gate 2→3**

无需提交（验证任务不产生代码变更）；把命令输出摘要带入 `gf-quality` 阶段闸门检查。
