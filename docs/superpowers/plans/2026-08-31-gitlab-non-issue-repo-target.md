# GitLab 非 issue 命令族 `--repo`/`--project` 目标修复 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 把 #267/PR #274 已验证的 `repo`/`repo_target` 拆分模式复用到 `GitLabMrProvider`、`GitLabReleaseProvider`、`GitLabPipelineProvider`、`GitLabLabelProvider`、`GitLabMilestoneProvider`（milestone 用 `project_target`，因为对应 flag 是 `--project`），修复这些 provider 在自建 GitLab 实例上同样存在的裸 `--repo`/`--project` host 歧义问题。`GitLabCommitProvider`（纯 REST，从不传 `--repo`）与 `GitLabReviewProvider`（`glab mr approve/revoke` 本来就不传 `--repo`）经调查确认不受影响，不在本计划范围内。

**Architecture:** 每个受影响 provider struct 新增 `repo_target`（或 `project_target`）字段，默认等于 `repo`；新增 `with_remote_url()`（生产用）和 `with_runner_and_repo_target()`/`with_runner_and_project_target()`（测试用）构造器；所有 `--repo`/`--project` CLI 传参改用新字段，`repo` 字段保留给 `encode_project_path` 等 REST 路径编码场景。CLI 层：`apps/cli/src/main.rs::router()` 已有的 `remote_url: &str` 参数透传给 `pr::handle`/`release::handle`/`pipeline::handle`/`label::handle_label`/`label::handle_milestone`（`commit::handle`/`review::handle` 不动）；各 handler 在 GitLab 分支里，`remote_url` 非空时用 `with_remote_url`，否则退回 `new()`。

**Tech Stack:** Rust 2024 workspace；`gitflow-gitlab`（通过 `glab` CLI）、`apps/cli`。

**Spec:** `docs/superpowers/specs/2026-08-31-gitlab-non-issue-repo-target-design.md`

## Global Constraints

- 不改动 `deny.toml` / `.pre-commit-config.yaml` / `rust-toolchain.toml`。
- 不引入新依赖。
- `#![forbid(unsafe_code)]`、生产代码禁止 `unwrap()`/`expect()`。
- 所有新增 public 方法需要文档注释。
- `crates/gitlab/src/commit.rs`、`crates/gitlab/src/review.rs`、`crates/gitlab/src/error.rs` 不在本计划范围内，不要改动。
- 每个任务完成后运行 `cargo test -p gitflow-gitlab`（provider 任务）或 `cargo test -p gitflow-cli`（CLI 任务）确认绿色再提交；全部任务完成后跑 Task 11 的完整验证。

---

### Task 1: `GitLabMrProvider` 新增 `repo_target` + 全部 `--repo` 站点接线

**Files:**
- Modify: `crates/gitlab/src/mr.rs`（struct L39-45、构造器 L47-82、`run_mr_update` L91-117、`create` L234-267、`list` L289-308、`view` L326-344、`close` L357-365、`reopen` L374-382、`merge` L420-429、`checkout` L466-475、`sync_branch` L498-505）
- Test: 同文件 `mod tests`

**Interfaces:**
- Consumes: 无
- Produces: `GitLabMrProvider<R>.repo_target: String`；`GitLabMrProvider<RealCommandRunner>::with_remote_url(repo, remote_url) -> Self`；`GitLabMrProvider<R: CommandRunner>::with_runner_and_repo_target(repo, repo_target, runner) -> Self`

- [ ] **Step 1: 写失败测试**

在 `mod tests` 内、`test_should_call_mr_close_with_repo_flag`（或就近的 close 相关测试）之后插入：

```rust
    #[tokio::test]
    async fn test_should_use_explicit_repo_target_for_close() {
        let runner = MockCommandRunner::success(
            r#"{"iid":42,"title":"Fix","state":"opened","source_branch":"a","target_branch":"main"}"#,
        );
        let provider = GitLabMrProvider::with_runner_and_repo_target(
            "owner/repo",
            "https://192.168.230.23/iproost/proxy/api-src.git",
            runner.clone(),
        );

        let pr = provider.close(42).await.expect("close should succeed");

        assert_eq!(pr.number, 42);
        assert_eq!(
            runner.recorded_calls()[0].1,
            vec![
                "mr",
                "close",
                "42",
                "--repo",
                "https://192.168.230.23/iproost/proxy/api-src.git",
            ]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>()
        );
    }
```

（若 `close` 附近没有现成的成功 JSON 断言测试可参照，直接在 `mod tests` 末尾追加即可；关键是 `with_runner_and_repo_target` 尚不存在，本步骤只是让它编译失败。）

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p gitflow-gitlab test_should_use_explicit_repo_target_for_close`
Expected: 编译失败——`with_runner_and_repo_target` 不存在。

- [ ] **Step 3: 实现**

struct（原 39-45 行）：

```rust
#[derive(Debug, Clone)]
pub struct GitLabMrProvider<R: CommandRunner = RealCommandRunner> {
    /// GitLab `namespace/project`。供 REST notes API 路径编码使用，始终是裸
    /// `owner/repo` 形式，不受 [`repo_target`](Self::repo_target) 影响。
    repo: String,
    /// 传给 `glab mr ...` 子命令 `--repo` 参数的目标字符串。默认等于 `repo`；
    /// 通过 [`with_remote_url`](GitLabMrProvider::with_remote_url) 构造时为完整
    /// git remote URL，用于在自建 GitLab 实例上显式锁定 host。
    repo_target: String,
    /// 用于执行 `glab` CLI 命令的 runner。
    runner: R,
}
```

`impl GitLabMrProvider<RealCommandRunner>`（原 47-69 行）：

```rust
impl GitLabMrProvider<RealCommandRunner> {
    /// 创建新的 GitLab MR 提供者，使用真实的进程执行器。
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

    /// 使用完整 git remote URL 作为 `glab mr ...` 的 `--repo` 目标创建提供者。
    ///
    /// `repo` 仍为裸 `namespace/project`（供 REST notes API 路径编码使用），
    /// `remote_url` 为完整 git remote URL，`glab` 官方文档确认 `--repo` 接受该形式。
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

`impl<R: CommandRunner> GitLabMrProvider<R>`（原 71-82 行的 `with_runner`，新增一个方法）：

```rust
impl<R: CommandRunner> GitLabMrProvider<R> {
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
    /// 主要用于测试，验证 `repo_target`（如完整 remote URL）被正确传给 `glab`。
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

    // ... run_mr_update 保持原位置，Step 3b 单独改
```

`run_mr_update`（原 91-117 行）第 106-107 行的 `"--repo", &self.repo,` 改为 `"--repo", &self.repo_target,`：

```rust
    async fn run_mr_update(&self, number: u64, draft: bool) -> Result<()> {
        let number_str = number.to_string();
        let draft_flag = if draft {
            "--draft=true"
        } else {
            "--draft=false"
        };
        let output = self
            .runner
            .run(
                "glab",
                &[
                    "mr",
                    "update",
                    &number_str,
                    "--repo",
                    &self.repo_target,
                    draft_flag,
                ],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab mr update: {e}")))?;
        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
        }
        Ok(())
    }
}
```

`create`（原 234-247 行）——注意用户显式 `--repo` 覆盖（`args.repo`）优先级仍高于 `repo_target`：

```rust
    async fn create(&self, args: CreatePrArgs) -> Result<PrData> {
        let repo = args.repo.as_deref().unwrap_or(&self.repo_target);
        let mut cmd_args: Vec<&str> = vec![
            "mr",
            "create",
            "--repo",
            repo,
            "--title",
            &args.title,
            "--source-branch",
            &args.head,
            "--target-branch",
            &args.base,
        ];
```

`list`（原 289-290 行）：

```rust
    async fn list(&self, args: ListPrArgs) -> Result<Vec<PrData>> {
        let mut cmd_args: Vec<&str> =
            vec!["mr", "list", "--repo", &self.repo_target, "--output", "json"];
```

`view`（原 326-343 行）：

```rust
    async fn view(&self, number: u64) -> Result<PrData> {
        debug!(repo = %self.repo, number, "spawning `glab mr view`");

        let number_str = number.to_string();
        let output = self
            .runner
            .run(
                "glab",
                &[
                    "mr",
                    "view",
                    &number_str,
                    "--repo",
                    &self.repo_target,
                    "--output",
                    "json",
                ],
            )
            .await
```

`close`（原 363 行）：

```rust
            .run("glab", &["mr", "close", &number_str, "--repo", &self.repo_target])
```

`reopen`（原 380 行）：

```rust
            .run("glab", &["mr", "reopen", &number_str, "--repo", &self.repo_target])
```

`merge`（原 429 行）：

```rust
        let mut cmd_args: Vec<&str> = vec!["mr", "merge", &number_str, "--repo", &self.repo_target];
```

`checkout`（原 474 行）：

```rust
                &["mr", "checkout", &number_str, "--repo", &self.repo_target],
```

`sync_branch`（原 504 行）：

```rust
            .run("glab", &["mr", "rebase", &number_str, "--repo", &self.repo_target])
```

- [ ] **Step 4: 运行测试确认通过且无回归**

Run: `cargo test -p gitflow-gitlab mr::tests`
Expected: 全部 PASS，包括新增的 `test_should_use_explicit_repo_target_for_close`，以及既有断言 `"owner/repo"` 的用例（`with_runner`/`new` 路径 `repo_target` 默认等于 `repo`）。

- [ ] **Step 5: 提交**

```bash
git add crates/gitlab/src/mr.rs
git commit -m "fix(gitlab): route GitLabMrProvider --repo through repo_target"
```

---

### Task 2: `GitLabReleaseProvider` 新增 `repo_target` + 全部 `--repo` 站点接线

**Files:**
- Modify: `crates/gitlab/src/release.rs`（struct L38-44、构造器 L46-82、`create`×2 L150-155/L240-245、`list` L198-202、`view` L220-225、`upload` L293-298、`download` L333-348、`delete` L370-375）
- Test: 同文件 `mod tests`

**Interfaces:**
- Consumes: 无
- Produces: `GitLabReleaseProvider<R>.repo_target: String`；`with_remote_url()`；`with_runner_and_repo_target()`

- [ ] **Step 1: 写失败测试**

在 `mod tests` 末尾追加：

```rust
    #[tokio::test]
    async fn test_should_use_explicit_repo_target_for_list() {
        let runner = MockCommandRunner::success("[]");
        let provider = GitLabReleaseProvider::with_runner_and_repo_target(
            "owner/repo",
            "https://192.168.230.23/iproost/proxy/api-src.git",
            runner.clone(),
        );

        let result = provider.list().await;

        assert!(result.is_ok(), "expected Ok, got {result:?}");
        assert_eq!(
            runner.recorded_calls()[0].1,
            vec![
                "release",
                "list",
                "--repo",
                "https://192.168.230.23/iproost/proxy/api-src.git",
                "--output",
                "json",
            ]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>()
        );
    }
```

（若 `ReleaseProvider::list()` 签名带参数，以 `crates/core/src/release.rs` 中 trait 定义为准调整调用形式；核心断言点是 `recorded_calls()[0].1` 的 `--repo` 值。）

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p gitflow-gitlab test_should_use_explicit_repo_target_for_list`
Expected: 编译失败——`with_runner_and_repo_target` 不存在。

- [ ] **Step 3: 实现**

struct + 构造器（原 38-82 行），与 Task 1 完全同构：

```rust
#[derive(Debug, Clone)]
pub struct GitLabReleaseProvider<R: CommandRunner = RealCommandRunner> {
    /// GitLab `namespace/project`。
    repo: String,
    /// 传给 `glab release ...` 子命令 `--repo` 参数的目标字符串，见
    /// [`GitLabMrProvider::repo_target`](crate::GitLabMrProvider) 的同款设计说明。
    repo_target: String,
    /// 用于执行 `glab` CLI 命令的 runner。
    runner: R,
}

impl GitLabReleaseProvider<RealCommandRunner> {
    /// 创建新的 GitLab Release 提供者，使用真实的进程执行器。
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
    #[must_use]
    pub fn with_session(session: &gitflow_core::Session) -> Self {
        let repo = session.repo.clone();
        Self {
            repo_target: repo.clone(),
            repo,
            runner: RealCommandRunner,
        }
    }

    /// 使用完整 git remote URL 作为 `glab release ...` 的 `--repo` 目标创建提供者。
    #[must_use]
    pub fn with_remote_url(repo: impl Into<String>, remote_url: impl Into<String>) -> Self {
        Self {
            repo: repo.into(),
            repo_target: remote_url.into(),
            runner: RealCommandRunner,
        }
    }
}

impl<R: CommandRunner> GitLabReleaseProvider<R> {
    /// 使用自定义 [`CommandRunner`] 创建提供者。
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
}
```

各 `--repo` 站点，把 `&self.repo` 改为 `&self.repo_target`：

- 第一个 `create`（原 155 行）：`vec!["release", "create", &args.tag_name, "--repo", &self.repo_target];`
- `list`（原 202 行）：`&["release", "list", "--repo", &self.repo_target, "--output", "json"],`
- `view`（原 225 行）：`"release", "view", tag_name, "--repo", &self.repo_target, "--output", "json",`
- 第二个 `create`（原 245 行）：`let mut cmd_args: Vec<&str> = vec!["release", "create", tag_name, "--repo", &self.repo_target];`
- `upload`（原 298 行）：`"release", "upload", tag_name, file_path, "--repo", &self.repo_target,`
- `download`（原 333-348 行区块）：

```rust
        let output = self
            .runner
            .run(
                "glab",
                &[
                    "release",
                    "download",
                    tag_name,
                    "--repo",
                    &self.repo_target,
                    "--pattern",
                    asset_name,
                    "--dir",
                    &parent_str,
                ],
            )
```

- `delete`（原 375 行）：`&["release", "delete", tag_name, "--repo", &self.repo_target, "--yes"],`

- [ ] **Step 4: 运行测试确认通过且无回归**

Run: `cargo test -p gitflow-gitlab release::tests`
Expected: 全部 PASS。

- [ ] **Step 5: 提交**

```bash
git add crates/gitlab/src/release.rs
git commit -m "fix(gitlab): route GitLabReleaseProvider --repo through repo_target"
```

---

### Task 3: `GitLabPipelineProvider` 新增 `repo_target` + 2 处 `--repo` 站点接线

**Files:**
- Modify: `crates/gitlab/src/pipeline.rs`（struct L41-47、构造器 L49-84、`list`/`status` 附近 L221、`trace`/logs 附近 L249）
- Test: 同文件 `mod tests`

**Interfaces:**
- Consumes: 无
- Produces: `GitLabPipelineProvider<R>.repo_target: String`；`with_remote_url()`；`with_runner_and_repo_target()`

- [ ] **Step 1: 写失败测试**

```rust
    #[tokio::test]
    async fn test_should_use_explicit_repo_target_for_trace() {
        let runner = MockCommandRunner::success("log output");
        let provider = GitLabPipelineProvider::with_runner_and_repo_target(
            "owner/repo",
            "https://192.168.230.23/iproost/proxy/api-src.git",
            runner.clone(),
        );

        let _ = provider.trace(99).await;

        assert_eq!(
            runner.recorded_calls()[0].1,
            vec![
                "ci",
                "trace",
                "99",
                "--repo",
                "https://192.168.230.23/iproost/proxy/api-src.git",
            ]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>()
        );
    }
```

（方法名 `trace` 以 `crates/core/src/pipeline.rs` 中 `PipelineProvider` trait 的实际方法名为准核对，行为断言重点是 `--repo` 参数值。）

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p gitflow-gitlab test_should_use_explicit_repo_target_for_trace`
Expected: 编译失败。

- [ ] **Step 3: 实现**

struct + 构造器（原 41-84 行），与 Task 1 同构（类型名替换为 `GitLabPipelineProvider`）：

```rust
#[derive(Debug, Clone)]
pub struct GitLabPipelineProvider<R: CommandRunner = RealCommandRunner> {
    /// GitLab `namespace/project`。
    repo: String,
    /// 传给 `glab ci ...` 子命令 `--repo` 参数的目标字符串。
    repo_target: String,
    /// 用于执行 `glab` CLI 命令的 runner。
    runner: R,
}

impl GitLabPipelineProvider<RealCommandRunner> {
    /// 创建新的 GitLab Pipeline 提供者，使用真实的进程执行器。
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
    #[must_use]
    pub fn with_session(session: &gitflow_core::Session) -> Self {
        let repo = session.repo.clone();
        Self {
            repo_target: repo.clone(),
            repo,
            runner: RealCommandRunner,
        }
    }

    /// 使用完整 git remote URL 作为 `glab ci ...` 的 `--repo` 目标创建提供者。
    #[must_use]
    pub fn with_remote_url(repo: impl Into<String>, remote_url: impl Into<String>) -> Self {
        Self {
            repo: repo.into(),
            repo_target: remote_url.into(),
            runner: RealCommandRunner,
        }
    }
}

impl<R: CommandRunner> GitLabPipelineProvider<R> {
    /// 使用自定义 [`CommandRunner`] 创建提供者。
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
}
```

两个调用站点：

- 原 221 行区块：`"ci", "list", "--repo", &self.repo_target, "--ref", branch, "--output", "json",`
- 原 249 行：`.run("glab", &["ci", "trace", &id_str, "--repo", &self.repo_target])`

- [ ] **Step 4: 运行测试确认通过且无回归**

Run: `cargo test -p gitflow-gitlab pipeline::tests`
Expected: 全部 PASS。

- [ ] **Step 5: 提交**

```bash
git add crates/gitlab/src/pipeline.rs
git commit -m "fix(gitlab): route GitLabPipelineProvider --repo through repo_target"
```

---

### Task 4: `GitLabLabelProvider` 新增 `repo_target` + 4 处 `--repo` 站点接线

**Files:**
- Modify: `crates/gitlab/src/label.rs`（`GitLabLabelProvider` struct L33-39、构造器 L41-76、`list_api` L83-96、`create` L124-141、`edit` L188-199、`delete` L221-234）
- Test: 同文件 `mod tests`（`GitLabLabelProvider` 部分）

**Interfaces:**
- Consumes: 无
- Produces: `GitLabLabelProvider<R>.repo_target: String`；`with_remote_url()`；`with_runner_and_repo_target()`

- [ ] **Step 1: 写失败测试**

在 `mod tests` 内 `--- GitLabLabelProvider tests ---` 分节下新增：

```rust
    #[tokio::test]
    async fn test_should_use_explicit_repo_target_for_delete() {
        let runner = MockCommandRunner::success("");
        let provider = GitLabLabelProvider::with_runner_and_repo_target(
            "owner/repo",
            "https://192.168.230.23/iproost/proxy/api-src.git",
            runner.clone(),
        );

        let result = provider.delete("bug").await;

        assert!(result.is_ok(), "expected Ok, got {result:?}");
        assert_eq!(
            runner.recorded_calls()[0].1,
            vec![
                "label",
                "delete",
                "bug",
                "--repo",
                "https://192.168.230.23/iproost/proxy/api-src.git",
            ]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>()
        );
    }
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p gitflow-gitlab test_should_use_explicit_repo_target_for_delete`
Expected: 编译失败。

- [ ] **Step 3: 实现**

struct（原 33-39 行）+ 构造器（原 41-76 行），同构：

```rust
#[derive(Debug, Clone)]
pub struct GitLabLabelProvider<R: CommandRunner = RealCommandRunner> {
    /// GitLab `namespace/project`。
    repo: String,
    /// 传给 `glab label ...` 子命令 `--repo` 参数的目标字符串。
    repo_target: String,
    /// 用于执行 `glab` CLI 命令的 runner。
    runner: R,
}

impl GitLabLabelProvider<RealCommandRunner> {
    /// 创建新的 GitLab Label 提供者。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> GitLabLabelProvider<RealCommandRunner> {
        let repo = repo.into();
        GitLabLabelProvider {
            repo_target: repo.clone(),
            repo,
            runner: RealCommandRunner,
        }
    }

    /// Create a new provider from a shared [`Session`].
    #[must_use]
    pub fn with_session(session: &gitflow_core::Session) -> Self {
        let repo = session.repo.clone();
        Self {
            repo_target: repo.clone(),
            repo,
            runner: RealCommandRunner,
        }
    }

    /// 使用完整 git remote URL 作为 `glab label ...` 的 `--repo` 目标创建提供者。
    #[must_use]
    pub fn with_remote_url(repo: impl Into<String>, remote_url: impl Into<String>) -> Self {
        Self {
            repo: repo.into(),
            repo_target: remote_url.into(),
            runner: RealCommandRunner,
        }
    }
}

impl<R: CommandRunner> GitLabLabelProvider<R> {
    /// 使用自定义 [`CommandRunner`] 创建提供者。
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

    /// 通过 `glab label list --output json` 获取原始 label API 响应。
    async fn list_api(&self) -> Result<Vec<LabelApiResponse>> {
        let output = self
            .runner
            .run(
                "glab",
                &["label", "list", "--repo", &self.repo_target, "--output", "json"],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab label list: {e}")))?;
        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
        }
        serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)
    }
}
```

`create`（原 132-141 行）：

```rust
        let mut cmd_args: Vec<&str> = vec![
            "label",
            "create",
            "--name",
            &args.name,
            "--color",
            &args.color,
            "--repo",
            &self.repo_target,
        ];
```

`edit`（原 188-199 行）：

```rust
        let mut cmd_args: Vec<&str> = vec![
            "label",
            "edit",
            "--label-id",
            &id_str,
            "--repo",
            &self.repo_target,
            "--new-name",
            &args.name,
            "--color",
            &args.color,
        ];
```

`delete`（原 226 行）：

```rust
            .run("glab", &["label", "delete", name, "--repo", &self.repo_target])
```

- [ ] **Step 4: 运行测试确认通过且无回归**

Run: `cargo test -p gitflow-gitlab label::tests`
Expected: 全部 PASS（本任务只涉及 `GitLabLabelProvider` 部分；`GitLabMilestoneProvider` 部分留给 Task 5，此时仍是旧代码，编译应仍然通过，因为两个 provider 相互独立）。

- [ ] **Step 5: 提交**

```bash
git add crates/gitlab/src/label.rs
git commit -m "fix(gitlab): route GitLabLabelProvider --repo through repo_target"
```

---

### Task 5: `GitLabMilestoneProvider` 新增 `project_target` + 5 处 `--project` 站点接线

**Files:**
- Modify: `crates/gitlab/src/label.rs`（`GitLabMilestoneProvider` struct L249-255、构造器 L257-293、`create` L355-370、`list` L397-412、`edit` L428-445、`close` L472-493、`reopen` L510-531）
- Test: 同文件 `mod tests`（`GitLabMilestoneProvider` 部分）

**Interfaces:**
- Consumes: 无
- Produces: `GitLabMilestoneProvider<R>.project_target: String`（注意字段名与其余 provider 不同，因为对应 flag 是 `--project`）；`with_remote_url()`；`with_runner_and_project_target()`

- [ ] **Step 1: 写失败测试**

```rust
    #[tokio::test]
    async fn test_should_use_explicit_project_target_for_close() {
        let runner = MockCommandRunner::success(
            r#"[{"id":1,"iid":1,"title":"v1","state":"closed"}]"#,
        );
        let provider = GitLabMilestoneProvider::with_runner_and_project_target(
            "owner/repo",
            "https://192.168.230.23/iproost/proxy/api-src.git",
            runner.clone(),
        );

        let result = provider.close(1).await;

        assert!(result.is_ok(), "expected Ok, got {result:?}");
        assert_eq!(
            runner.recorded_calls()[0].1,
            vec![
                "milestone",
                "edit",
                "1",
                "--state",
                "close",
                "--project",
                "https://192.168.230.23/iproost/proxy/api-src.git",
            ]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>()
        );
    }
```

（`close` 内部会在 `edit` 调用成功后再调 `self.list()` 校验，`MockCommandRunner::success` 对所有调用返回同一段 JSON——`recorded_calls()[0]` 取的是第一次即 `edit` 调用，断言仍然有效；若 provider 用的是 `SequencedMockCommandRunner` 风格，以文件里既有的相似测试写法为准调整。）

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p gitflow-gitlab test_should_use_explicit_project_target_for_close`
Expected: 编译失败。

- [ ] **Step 3: 实现**

struct（原 249-255 行）+ 构造器（原 257-293 行）：

```rust
#[derive(Debug, Clone)]
pub struct GitLabMilestoneProvider<R: CommandRunner = RealCommandRunner> {
    /// GitLab `namespace/project`。
    repo: String,
    /// 传给 `glab milestone ...` 子命令 `--project` 参数的目标字符串
    /// （`glab milestone` 用 `--project` 而非 `--repo`，语义与其余 provider
    /// 的 `repo_target` 一致，仅 flag 名不同）。
    project_target: String,
    /// 用于执行 `glab` CLI 命令的 runner。
    runner: R,
}

impl GitLabMilestoneProvider<RealCommandRunner> {
    /// 创建新的 GitLab Milestone 提供者。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> GitLabMilestoneProvider<RealCommandRunner> {
        let repo = repo.into();
        GitLabMilestoneProvider {
            project_target: repo.clone(),
            repo,
            runner: RealCommandRunner,
        }
    }

    /// Create a new provider from a shared [`Session`].
    #[must_use]
    pub fn with_session(session: &gitflow_core::Session) -> Self {
        let repo = session.repo.clone();
        Self {
            project_target: repo.clone(),
            repo,
            runner: RealCommandRunner,
        }
    }

    /// 使用完整 git remote URL 作为 `glab milestone ...` 的 `--project` 目标创建提供者。
    #[must_use]
    pub fn with_remote_url(repo: impl Into<String>, remote_url: impl Into<String>) -> Self {
        Self {
            repo: repo.into(),
            project_target: remote_url.into(),
            runner: RealCommandRunner,
        }
    }
}

impl<R: CommandRunner> GitLabMilestoneProvider<R> {
    /// 使用自定义 [`CommandRunner`] 创建提供者。
    #[must_use]
    pub fn with_runner(repo: impl Into<String>, runner: R) -> Self {
        let repo = repo.into();
        Self {
            project_target: repo.clone(),
            repo,
            runner,
        }
    }

    /// 使用自定义 [`CommandRunner`] 并显式指定 `--project` 目标创建提供者。
    #[must_use]
    pub fn with_runner_and_project_target(
        repo: impl Into<String>,
        project_target: impl Into<String>,
        runner: R,
    ) -> Self {
        Self {
            repo: repo.into(),
            project_target: project_target.into(),
            runner,
        }
    }
}
```

`create`（原 363-370 行）：

```rust
        let mut cmd_args: Vec<&str> = vec![
            "milestone",
            "create",
            "--title",
            &args.title,
            "--project",
            &self.project_target,
        ];
```

`list`（原 400-412 行）：

```rust
        let output = self
            .runner
            .run(
                "glab",
                &[
                    "milestone",
                    "list",
                    "--project",
                    &self.project_target,
                    "--output",
                    "json",
                ],
            )
```

`edit`（原 437-445 行）：

```rust
        let mut cmd_args: Vec<&str> = vec![
            "milestone",
            "edit",
            &number_str,
            "--project",
            &self.project_target,
            "--title",
            &args.title,
        ];
```

`close`（原 480-493 行）：

```rust
        let output = self
            .runner
            .run(
                "glab",
                &[
                    "milestone",
                    "edit",
                    &number_str,
                    "--state",
                    "close",
                    "--project",
                    &self.project_target,
                ],
            )
```

`reopen`（原 518-531 行）：

```rust
        let output = self
            .runner
            .run(
                "glab",
                &[
                    "milestone",
                    "edit",
                    &number_str,
                    "--state",
                    "activate",
                    "--project",
                    &self.project_target,
                ],
            )
```

- [ ] **Step 4: 运行测试确认通过且无回归**

Run: `cargo test -p gitflow-gitlab label::tests`
Expected: 全部 PASS（含 Task 4 + Task 5 的 `GitLabLabelProvider`/`GitLabMilestoneProvider` 测试）。

- [ ] **Step 5: 提交**

```bash
git add crates/gitlab/src/label.rs
git commit -m "fix(gitlab): route GitLabMilestoneProvider --project through project_target"
```

---

### Task 6: `apps/cli/src/main.rs::router()` 把 `remote_url` 透传给 5 个受影响 handler

**Files:**
- Modify: `apps/cli/src/main.rs`（`router()` 函数，原 204-227 行区域）

**Interfaces:**
- Consumes: Task 1-5 的 provider 变更；`router()` 已有的 `remote_url: &str` 参数（PR #274 引入）
- Produces: `pr::handle`/`release::handle`/`pipeline::handle`/`label::handle_label`/`label::handle_milestone` 的调用点新增 `remote_url` 实参

- [ ] **Step 1: 编译检查当前状态（作为基线）**

Run: `cargo build -p gitflow-cli`
Expected: 当前应仍能编译（Task 1-5 只改了 provider 内部字段，未改 `new()` 签名，CLI 侧尚未调用新增的 `with_remote_url`）。

- [ ] **Step 2: 实现**

把 `router()`（原 204-227 行）中以下 5 行：

```rust
        Commands::Pr(cmd) => commands::pr::handle(cmd, platform, repo, output).await,
        Commands::Release(cmd) => commands::release::handle(cmd, platform, repo, output).await,
        ...
        Commands::Label(cmd) => commands::label::handle_label(cmd, platform, repo, output).await,
        Commands::Milestone(cmd) => {
            commands::label::handle_milestone(cmd, platform, repo, output).await
        }
        ...
        Commands::Pipeline(cmd) => commands::pipeline::handle(cmd, platform, repo, output).await,
```

改为：

```rust
        Commands::Pr(cmd) => commands::pr::handle(cmd, platform, repo, remote_url, output).await,
        Commands::Release(cmd) => {
            commands::release::handle(cmd, platform, repo, remote_url, output).await
        }
        ...
        Commands::Label(cmd) => {
            commands::label::handle_label(cmd, platform, repo, remote_url, output).await
        }
        Commands::Milestone(cmd) => {
            commands::label::handle_milestone(cmd, platform, repo, remote_url, output).await
        }
        ...
        Commands::Pipeline(cmd) => {
            commands::pipeline::handle(cmd, platform, repo, remote_url, output).await
        }
```

（省略号 `...` 处的其余分支——`Review`/`Auth`/`Commit`/`Workflow`/`Doctor`/`Skills`/`Update`/`Run`/`Completions`——保持原样不动，先用 `Read` 核对 `router()` 当前完整分支列表再逐条替换，不要整体重写整个 `match`。）

- [ ] **Step 3: 编译检查（预期失败，Task 7-9 会补上 handler 签名）**

Run: `cargo build -p gitflow-cli`
Expected: 5 个 `E0061`（参数数量不匹配）错误，分别指向 `commands::pr::handle`/`commands::release::handle`/`commands::pipeline::handle`/`commands::label::handle_label`/`commands::label::handle_milestone`。这是预期的中间态，Task 7-9 补完后消失。

- [ ] **Step 4: 提交（中间态，与 Task 7 合并提交更合理——见 Task 7 Step 5 的说明）**

本任务不单独提交，改动随 Task 7 一起提交（`main.rs` 的改动如果不搭配至少一个 handler 更新会导致编译失败，参照 #267 计划里 Task 5+6 合并提交的先例）。

---

### Task 7: `commands::pr::handle` 接线 `remote_url`

**Files:**
- Modify: `apps/cli/src/commands/pr.rs`（`handle()` 签名 L214-219、provider 构造 L220-229）
- Test: 同文件 `mod tests`（如有）

**Interfaces:**
- Consumes: Task 6 的 `router()` 调用点；Task 1 的 `GitLabMrProvider::with_remote_url`
- Produces: `pub async fn handle(command: PrCommand, platform: &str, repo: &str, remote_url: &str, output_format: OutputFormat) -> miette::Result<()>`

- [ ] **Step 1: 实现**

把 `handle()` 签名（原 214-219 行）改为：

```rust
pub async fn handle(
    command: PrCommand,
    platform: &str,
    repo: &str,
    remote_url: &str,
    output_format: OutputFormat,
) -> miette::Result<()> {
```

把 provider 构造（原 220-229 行）中 GitLab 分支改为：

```rust
    let provider: Box<dyn PrProvider> = match platform {
        "github" => Box::new(GitHubPrProvider::new(repo)),
        "gitlab" => {
            if remote_url.is_empty() {
                Box::new(GitLabMrProvider::new(repo))
            } else {
                Box::new(GitLabMrProvider::with_remote_url(repo, remote_url))
            }
        }
        "gitcode" => Box::new(GitCodePrProvider::new(repo)),
        other => {
            return Err(miette::miette!(
                "Platform '{other}' not yet supported for pr commands"
            ));
        }
    };
```

**注意**：`pr create` 自身的 `--repo` 覆盖（`PrCommand::Create { repo: target_repo, .. }`）已经在 Task 1 的 `GitLabMrProvider::create()` 内部通过 `args.repo.as_deref().unwrap_or(&self.repo_target)` 处理，构造阶段不需要像 `commands/issue.rs` 那样区分"是否有仓库覆盖"——`pr.rs` 这里始终优先用 `with_remote_url`（当 `remote_url` 非空时），create 时如果用户传了 `--repo` 会在 provider 内部覆盖它。

- [ ] **Step 2: 编译并运行测试**

Run: `cargo build -p gitflow-cli && cargo test -p gitflow-cli`
Expected: 编译通过（`Commands::Pr` 分支已在 Task 6 更新，实参数量匹配）；既有测试全部 PASS。

- [ ] **Step 3: 提交（含 Task 6 的 `main.rs` 改动）**

```bash
git add apps/cli/src/main.rs apps/cli/src/commands/pr.rs
git commit -m "fix(cli): use git remote URL as GitLab --repo target for pr commands"
```

---

### Task 8: `commands::release::handle` 接线 `remote_url`

**Files:**
- Modify: `apps/cli/src/commands/release.rs`（`handle()` 签名 L138-143、provider 构造 L144-153 附近）

**Interfaces:**
- Consumes: Task 6 的 `router()` 调用点；Task 2 的 `GitLabReleaseProvider::with_remote_url`
- Produces: `pub async fn handle(command: ReleaseCommand, platform: &str, repo: &str, remote_url: &str, output_format: OutputFormat) -> miette::Result<()>`

- [ ] **Step 1: 实现**

签名（原 138-143 行）：

```rust
pub async fn handle(
    command: ReleaseCommand,
    platform: &str,
    repo: &str,
    remote_url: &str,
    output_format: OutputFormat,
) -> miette::Result<()> {
```

provider 构造（原 144-147 行区域）GitLab 分支：

```rust
    let provider: Box<dyn ReleaseProvider> = match platform {
        "github" => Box::new(GitHubReleaseProvider::new(repo)),
        "gitlab" => {
            if remote_url.is_empty() {
                Box::new(GitLabReleaseProvider::new(repo))
            } else {
                Box::new(GitLabReleaseProvider::with_remote_url(repo, remote_url))
            }
        }
        "gitcode" => Box::new(GitCodeReleaseProvider::new(repo)),
```

（保留原有 `other => { ... }` 兜底分支不动，只替换 `"gitlab"` 这一行为上面的块。）

- [ ] **Step 2: 编译并运行测试**

Run: `cargo build -p gitflow-cli && cargo test -p gitflow-cli`
Expected: 编译通过，测试全部 PASS。

- [ ] **Step 3: 提交**

```bash
git add apps/cli/src/commands/release.rs
git commit -m "fix(cli): use git remote URL as GitLab --repo target for release commands"
```

---

### Task 9: `commands::pipeline::handle` 接线 `remote_url`

**Files:**
- Modify: `apps/cli/src/commands/pipeline.rs`（`handle()` 签名 L64-69、provider 构造 L70-73 附近）

**Interfaces:**
- Consumes: Task 6 的 `router()` 调用点；Task 3 的 `GitLabPipelineProvider::with_remote_url`
- Produces: `pub async fn handle(command: PipelineCommand, platform: &str, repo: &str, remote_url: &str, output_format: OutputFormat) -> miette::Result<()>`

- [ ] **Step 1: 实现**

签名（原 64-69 行）：

```rust
pub async fn handle(
    command: PipelineCommand,
    platform: &str,
    repo: &str,
    remote_url: &str,
    output_format: OutputFormat,
) -> miette::Result<()> {
```

provider 构造（原 70-73 行区域）GitLab 分支：

```rust
    let provider: Box<dyn PipelineProvider> = match platform {
        "github" => Box::new(GitHubPipelineProvider::new(repo)),
        "gitlab" => {
            if remote_url.is_empty() {
                Box::new(GitLabPipelineProvider::new(repo))
            } else {
                Box::new(GitLabPipelineProvider::with_remote_url(repo, remote_url))
            }
        }
        "gitcode" => Box::new(GitCodePipelineProvider::new(repo)),
```

- [ ] **Step 2: 编译并运行测试**

Run: `cargo build -p gitflow-cli && cargo test -p gitflow-cli`
Expected: 编译通过，测试全部 PASS。

- [ ] **Step 3: 提交**

```bash
git add apps/cli/src/commands/pipeline.rs
git commit -m "fix(cli): use git remote URL as GitLab --repo target for pipeline commands"
```

---

### Task 10: `commands::label::handle_label` + `handle_milestone` 接线 `remote_url`

**Files:**
- Modify: `apps/cli/src/commands/label.rs`（`handle_label()` 签名 L131-136、provider 构造 L137-140 附近；`handle_milestone()` 签名 L253-258、provider 构造 L259-262 附近）

**Interfaces:**
- Consumes: Task 6 的 `router()` 调用点；Task 4 的 `GitLabLabelProvider::with_remote_url`、Task 5 的 `GitLabMilestoneProvider::with_remote_url`
- Produces: `pub async fn handle_label(command: LabelCommand, platform: &str, repo: &str, remote_url: &str, output_format: OutputFormat) -> miette::Result<()>`；`pub async fn handle_milestone(command: MilestoneCommand, platform: &str, repo: &str, remote_url: &str, output_format: OutputFormat) -> miette::Result<()>`

- [ ] **Step 1: 实现 `handle_label`**

签名（原 131-136 行）：

```rust
pub async fn handle_label(
    command: LabelCommand,
    platform: &str,
    repo: &str,
    remote_url: &str,
    output_format: OutputFormat,
) -> miette::Result<()> {
```

provider 构造（原 137-140 行区域）GitLab 分支：

```rust
    let provider: Box<dyn LabelProvider> = match platform {
        "github" => Box::new(GitHubLabelProvider::new(repo)),
        "gitlab" => {
            if remote_url.is_empty() {
                Box::new(GitLabLabelProvider::new(repo))
            } else {
                Box::new(GitLabLabelProvider::with_remote_url(repo, remote_url))
            }
        }
        "gitcode" => Box::new(GitCodeLabelProvider::new(repo)),
```

- [ ] **Step 2: 实现 `handle_milestone`**

签名（原 253-258 行）：

```rust
pub async fn handle_milestone(
    command: MilestoneCommand,
    platform: &str,
    repo: &str,
    remote_url: &str,
    output_format: OutputFormat,
) -> miette::Result<()> {
```

provider 构造（原 259-262 行区域）GitLab 分支：

```rust
    let provider: Box<dyn MilestoneProvider> = match platform {
        "github" => Box::new(GitHubMilestoneProvider::new(repo)),
        "gitlab" => {
            if remote_url.is_empty() {
                Box::new(GitLabMilestoneProvider::new(repo))
            } else {
                Box::new(GitLabMilestoneProvider::with_remote_url(repo, remote_url))
            }
        }
        "gitcode" => Box::new(GitCodeMilestoneProvider::new(repo)),
```

- [ ] **Step 3: 编译并运行测试**

Run: `cargo build -p gitflow-cli && cargo test -p gitflow-cli`
Expected: 编译通过（此时 Task 6-10 全部完成，`router()` 里全部 5 个调用点都已匹配对应 handler 的新签名）；测试全部 PASS。

- [ ] **Step 4: 提交**

```bash
git add apps/cli/src/commands/label.rs
git commit -m "fix(cli): use git remote URL as GitLab --repo/--project target for label/milestone commands"
```

---

### Task 11: 全量验证

**Files:** 无新增/修改（纯验证任务）

**Interfaces:**
- Consumes: Task 1-10 的全部改动
- Produces: 验证证据，用于 Phase 2 quality gate 与 Phase 4 交付前确认

- [ ] **Step 1: 运行受影响 crate 的完整测试**

Run: `cargo test -p gitflow-gitlab -p gitflow-cli`
Expected: 全部 PASS，无回归（若 `-p gitflow-gitlab -p gitflow-cli` 合并调用出现 `auth::tests` 相关的偶发失败，那是已知的 `cargo test` 跨包并行环境变量竞争假象——见 #267 处理先例——改用 `make test`（nextest）复核）。

- [ ] **Step 2: `make test` 复核**

Run: `make test`
Expected: 全部 PASS。

- [ ] **Step 3: 运行 clippy pedantic**

Run: `cargo clippy -p gitflow-gitlab -p gitflow-cli --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: 无警告。

- [ ] **Step 4: 运行 rustfmt 检查**

Run: `cargo +nightly fmt --check`
Expected: 无 diff。

- [ ] **Step 5: 记录验证结果，准备进入 Gate 2→3**

无需提交；把命令输出摘要带入 `gf-quality` 阶段闸门检查。
