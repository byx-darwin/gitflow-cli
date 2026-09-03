# pr create 默认分支检测 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `gf pr create` 未显式传 `--base` 时，查询仓库真实默认分支（GitHub/GitLab 走各自 CLI），而不是硬编码 `"main"`；查询失败（含 GitCode 平台能力缺失）时回退 `"main"`。

**Architecture:** 在 `PrProvider` trait 新增 `default_branch()` 方法，三个平台 provider（GitHub/GitLab/GitCode）各自实现；GitHub/GitLab 通过各自 CLI 的 `repo view --json` 查询，GitCode 直接返回 `CoreError::Platform`（无 CLI 能力，先例：`merge --auto`）。命令层新增一个纯函数 `resolve_default_branch` 处理"查询结果 → 实际 base"的 fallback 逻辑，未传 `--base` 时才调用 provider 查询。

**Tech Stack:** Rust 2024, `async-trait`, `serde`/`serde_json`, `tracing`, 现有 `CommandRunner`/`MockCommandRunner` 测试基础设施。

**Spec:** `docs/superpowers/specs/2026-09-03-pr-create-default-branch-design.md`

## Global Constraints

- 显式传 `--base` 时行为完全不变，不得触发任何 provider 查询。
- 禁止 `unwrap()`/`expect()` 于生产代码；错误一律通过 `Result<T>` 传播。
- 新公共方法（trait 方法、辅助函数）需 `# Errors` 文档；测试命名遵循 `test_should_<expected_behavior>`。
- 完成后运行 `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`。

---

## Task 1: `PrProvider` trait 新增 `default_branch()` 方法

**Files:**
- Modify: `crates/core/src/pr.rs`（trait 定义 + 编译期检查测试）

**Interfaces:**
- Produces: `async fn default_branch(&self) -> Result<String>`（`PrProvider` trait 方法，供 Task 2/3/4 实现，Task 5 调用）

- [ ] **Step 1: 在 `PrProvider` trait 末尾（`patch` 方法之后）新增方法**

在 `crates/core/src/pr.rs` 找到：

```rust
    async fn patch(&self, number: u64) -> Result<String>;
}
```

改为：

```rust
    async fn patch(&self, number: u64) -> Result<String>;

    /// 查询仓库配置的默认分支（如 `main`、`dev`）。
    ///
    /// 用于 `pr create` 在未显式指定 `--base` 时探测目标分支，避免硬编码
    /// `"main"` 导致默认分支非 `main` 的仓库创建出目标错误的 PR/MR。
    ///
    /// # Errors
    ///
    /// 当平台 API 调用失败或平台不支持该查询（如 GitCode）时返回错误。
    async fn default_branch(&self) -> Result<String>;
}
```

- [ ] **Step 2: 更新编译期检查测试 `test_should_have_diff_and_patch_methods_on_trait`**

该测试通过一个手写 `Check` struct 实现全部 trait 方法来保证 trait 签名可编译。在
`crates/core/src/pr.rs` 找到该测试内的 `impl PrProvider for Check`（`patch` 方法
之后），新增对应实现：

```rust
            async fn patch(&self, _number: u64) -> Result<String> {
                unimplemented!()
            }
            async fn default_branch(&self) -> Result<String> {
                unimplemented!()
            }
        }
```

（把新增方法插入在原有 `patch` 实现之后、`impl` 块的闭合 `}` 之前。）

- [ ] **Step 3: 确认 core crate 编译通过**

Run: `cargo check -p gitflow-core`
Expected: 编译失败 —— `crates/github/src/pr.rs`、`crates/gitlab/src/mr.rs`、
`crates/gitcode/src/pr.rs` 中的 `impl PrProvider for ...` 尚未实现新方法。
**这是预期的失败**：`gitflow-core` 自身（trait 定义 + `Check` 编译检查）应该
编译通过；下游 3 个 crate 的编译失败会在 Task 2-4 逐一修复。

- [ ] **Step 4: 运行 core crate 测试确认 Step 1/2 正确**

Run: `cargo test -p gitflow-core pr::`
Expected: PASS（包括 `test_should_have_diff_and_patch_methods_on_trait`）

- [ ] **Step 5: Commit**

```bash
git add crates/core/src/pr.rs
git commit -m "feat(core): add default_branch() to PrProvider trait"
```

---

## Task 2: GitHub — `GitHubPrProvider::default_branch()`

**Files:**
- Modify: `crates/github/src/pr.rs`

**Interfaces:**
- Consumes: `self.repo: String`（`owner/repo`）、`self.runner: R where R: CommandRunner`、
  `parse_gh_error(&[u8]) -> PlatformCliError`（已有，来自 `crate::error`）
- Produces: `default_branch()` 实现（`PrProvider` trait 方法）

- [ ] **Step 1: 写失败测试**

在 `crates/github/src/pr.rs` 的 `#[cfg(test)] mod tests` 内新增：

```rust
    #[tokio::test]
    async fn test_should_return_default_branch_on_success() {
        let runner = MockCommandRunner::success(
            r#"{"defaultBranchRef":{"name":"dev"}}"#,
        );
        let provider = GitHubPrProvider::with_runner("octocat/hello-world", runner);

        let result = provider.default_branch().await;

        assert!(result.is_ok());
        assert_eq!(result.expect("already checked"), "dev");
    }

    #[tokio::test]
    async fn test_should_send_expected_argv_for_default_branch() {
        let runner = MockCommandRunner::success(
            r#"{"defaultBranchRef":{"name":"dev"}}"#,
        );
        let provider = GitHubPrProvider::with_runner("octocat/hello-world", runner);

        let _ = provider.default_branch().await;

        let calls = provider.recorded_calls();
        assert_eq!(calls[0].0, "gh");
        assert_eq!(
            calls[0].1,
            vec![
                "repo",
                "view",
                "--repo",
                "octocat/hello-world",
                "--json",
                "defaultBranchRef",
            ]
        );
    }

    #[tokio::test]
    async fn test_should_return_error_when_repo_view_fails() {
        let runner = MockCommandRunner::failure("gh: Not Found (HTTP 404)", 1);
        let provider = GitHubPrProvider::with_runner("octocat/nonexistent", runner);

        let result = provider.default_branch().await;

        assert!(result.is_err());
    }
```

> 若 `GitHubPrProvider` 尚无 `recorded_calls()` 便捷方法（只有 `runner.recorded_calls()`），
> 改用 `MockCommandRunner` 的引用而非把它 move 进 `with_runner`，参照本文件已有测试
> （如 `test_should_create_provider_with_different_repos` 附近）中获取 runner 调用记录
> 的写法，保持一致即可——若已有 clone-and-keep-handle 模式，直接复用。

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p gitflow-github default_branch -- --nocapture`
Expected: FAIL — `default_branch` 方法不存在（编译错误）

- [ ] **Step 3: 实现 `default_branch()`**

在 `crates/github/src/pr.rs` 顶部（`PR_FIELDS` 常量附近）新增响应结构：

```rust
/// `gh repo view --json defaultBranchRef` 的响应类型。
#[derive(Debug, Deserialize)]
struct RepoViewResponse {
    #[serde(rename = "defaultBranchRef")]
    default_branch_ref: DefaultBranchRef,
}

/// `defaultBranchRef` 对象，仅取 `name` 字段。
#[derive(Debug, Deserialize)]
struct DefaultBranchRef {
    name: String,
}
```

并在文件顶部 `use` 区新增：

```rust
use serde::Deserialize;
```

在 `impl<R: CommandRunner + 'static> PrProvider for GitHubPrProvider<R>` 块内，
`patch` 方法之后新增：

```rust
    async fn default_branch(&self) -> Result<String> {
        debug!(repo = %self.repo, "spawning `gh repo view`");

        let output = self
            .runner
            .run(
                "gh",
                &[
                    "repo",
                    "view",
                    "--repo",
                    &self.repo,
                    "--json",
                    "defaultBranchRef",
                ],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            return Err(parse_gh_error(&output.stderr).into());
        }

        let resp: RepoViewResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(resp.default_branch_ref.name)
    }
```

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo test -p gitflow-github default_branch`
Expected: PASS（3 个新测试全部通过）

- [ ] **Step 5: Commit**

```bash
git add crates/github/src/pr.rs
git commit -m "feat(github): implement default_branch() via gh repo view"
```

---

## Task 3: GitLab — `GitLabMrProvider::default_branch()`

**Files:**
- Modify: `crates/gitlab/src/mr.rs`

**Interfaces:**
- Consumes: `self.repo_target: String`、`self.runner: R`、`parse_glab_error`（已有）
- Produces: `default_branch()` 实现

- [ ] **Step 1: 写失败测试**

在 `crates/gitlab/src/mr.rs` 的 `#[cfg(test)] mod tests` 内新增：

```rust
    #[tokio::test]
    async fn test_should_return_default_branch_on_success() {
        let runner = MockCommandRunner::success(r#"{"default_branch":"dev"}"#);
        let provider = GitLabMrProvider::with_runner("group/project", runner);

        let result = provider.default_branch().await;

        assert!(result.is_ok());
        assert_eq!(result.expect("already checked"), "dev");
    }

    #[tokio::test]
    async fn test_should_use_repo_target_for_default_branch() {
        let runner = MockCommandRunner::success(r#"{"default_branch":"dev"}"#);
        let provider = GitLabMrProvider::with_runner_and_repo_target(
            "group/project",
            "https://gitlab.example.com/group/project.git",
            runner,
        );

        let _ = provider.default_branch().await;

        let calls = provider.recorded_calls();
        assert_eq!(calls[0].0, "glab");
        assert_eq!(
            calls[0].1,
            vec![
                "repo",
                "view",
                "--repo",
                "https://gitlab.example.com/group/project.git",
                "--output",
                "json",
            ]
        );
    }

    #[tokio::test]
    async fn test_should_return_error_when_repo_view_fails() {
        let runner = MockCommandRunner::failure("glab: 404 Not Found", 1);
        let provider = GitLabMrProvider::with_runner("group/nonexistent", runner);

        let result = provider.default_branch().await;

        assert!(result.is_err());
    }
```

> 按本文件已有测试的实际取值/获取 recorded_calls 的方式对齐（如
> `test_should_use_explicit_repo_target_for_view` 一类测试），若辅助构造函数签名
> 与上面示例不完全一致，以文件内实际签名为准，只调整调用方式，不改变测试意图。

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p gitflow-gitlab default_branch -- --nocapture`
Expected: FAIL — 方法不存在

- [ ] **Step 3: 实现 `default_branch()`**

在 `crates/gitlab/src/mr.rs` 顶部新增响应结构（放在已有 `MrApiResponse` 附近）：

```rust
/// `glab repo view --output json` 的响应类型（仅取需要的字段）。
#[derive(Debug, Deserialize)]
struct RepoViewResponse {
    default_branch: String,
}
```

在 `impl<R: CommandRunner + 'static> PrProvider for GitLabMrProvider<R>` 块内新增：

```rust
    async fn default_branch(&self) -> Result<String> {
        debug!(repo = %self.repo, "spawning `glab repo view`");

        let output = self
            .runner
            .run(
                "glab",
                &["repo", "view", "--repo", &self.repo_target, "--output", "json"],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
        }

        let resp: RepoViewResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(resp.default_branch)
    }
```

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo test -p gitflow-gitlab default_branch`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/gitlab/src/mr.rs
git commit -m "feat(gitlab): implement default_branch() via glab repo view"
```

---

## Task 4: GitCode — `GitCodePrProvider::default_branch()`（无 CLI 能力）

**Files:**
- Modify: `crates/gitcode/src/pr.rs`

**Interfaces:**
- Produces: `default_branch()` 实现，直接返回 `CoreError::Platform`，不发起任何 CLI 调用

- [ ] **Step 1: 写失败测试**

在 `crates/gitcode/src/pr.rs` 的 `#[cfg(test)] mod tests` 内新增：

```rust
    #[tokio::test]
    async fn test_should_error_without_cli_call_for_default_branch() {
        let runner = MockCommandRunner::success("should not be called");
        let provider = GitCodePrProvider::with_runner("group/project", runner);

        let result = provider.default_branch().await;

        assert!(result.is_err());
        assert!(provider.recorded_calls().is_empty());
    }
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p gitflow-gitcode default_branch -- --nocapture`
Expected: FAIL — 方法不存在

- [ ] **Step 3: 实现 `default_branch()`**

在 `impl<R: CommandRunner + 'static> PrProvider for GitCodePrProvider<R>` 块内新增
（参照同文件 `merge()` 方法中 `auto` 分支的既有写法，`crates/gitcode/src/pr.rs:475-480`）：

```rust
    async fn default_branch(&self) -> Result<String> {
        Err(CoreError::Platform(
            "GitCode CLI 不支持查询仓库默认分支。请显式传入 --base，\
             或改用 GitHub/GitLab。"
                .into(),
        ))
    }
```

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo test -p gitflow-gitcode default_branch`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/gitcode/src/pr.rs
git commit -m "feat(gitcode): default_branch() returns Platform error (unsupported)"
```

---

## Task 5: 命令层接入 — `apps/cli/src/commands/pr.rs`

**Files:**
- Modify: `apps/cli/src/commands/pr.rs:251`（`PrCommand::Create` 分支）

**Interfaces:**
- Consumes: `PrProvider::default_branch(&self) -> Result<String>`（Task 1-4 产出）
- Produces: `fn resolve_default_branch(detected: gitflow_core::Result<String>) -> String`
  （模块私有辅助函数，供本文件内测试使用，风格对齐既有的 `resolve_body`/`resolve_head`）

- [ ] **Step 1: 写失败测试**

在 `apps/cli/src/commands/pr.rs` 的 `mod tests` 内（`test_should_resolve_head_with_explicit_value`
测试之后）新增：

```rust
    #[test]
    fn test_should_use_detected_branch_on_success() {
        let result = resolve_default_branch(Ok("dev".to_string()));
        assert_eq!(result, "dev");
    }

    #[test]
    fn test_should_fallback_to_main_on_detection_failure() {
        let result = resolve_default_branch(Err(gitflow_core::CoreError::Platform(
            "unsupported".to_string(),
        )));
        assert_eq!(result, "main");
    }
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p gitflow-cli resolve_default_branch -- --nocapture`
Expected: FAIL — 函数不存在（编译错误）

- [ ] **Step 3: 实现 `resolve_default_branch` 辅助函数并接入调用点**

在 `apps/cli/src/commands/pr.rs` 中，紧邻已有的 `resolve_head`/`resolve_body`
辅助函数（文件内 `fn resolve_head` 附近）新增：

```rust
/// 将 provider 查询到的默认分支结果落地为最终使用的 base 分支名。
///
/// 查询失败（含平台不支持，如 GitCode）时回退 `"main"`，不中断 `pr create` 流程。
fn resolve_default_branch(detected: gitflow_core::Result<String>) -> String {
    detected.unwrap_or_else(|e| {
        tracing::debug!(error = %e, "default_branch query failed, falling back to \"main\"");
        "main".to_string()
    })
}
```

把 `crates/core/src/pr.rs:251` 附近的：

```rust
            let resolved_base = base.unwrap_or_else(|| "main".to_string());
```

改为：

```rust
            let resolved_base = match base {
                Some(b) => b,
                None => resolve_default_branch(provider.default_branch().await),
            };
```

（显式传 `--base` 时走 `Some(b)` 分支，完全不触发 `provider.default_branch()` 调用。）

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo test -p gitflow-cli resolve_default_branch`
Expected: PASS

- [ ] **Step 5: 全量测试 + lint**

Run: `make test && make clippy`
Expected: 全部通过（新增改动不引入 clippy pedantic 警告）

- [ ] **Step 6: Commit**

```bash
git add apps/cli/src/commands/pr.rs
git commit -m "fix(pr): detect repo default branch instead of hardcoding \"main\""
```

---

## Self-Review Notes（供执行者参考，无需重复）

- **Spec 覆盖**：Task 1-4 覆盖 spec 中「GitHub/GitLab 走 API、GitCode 无能力」的方案；
  Task 5 覆盖调用点改造 + fallback 逻辑 + 「显式 `--base` 不变」的验收标准。
- **验收标准对应**：
  - 「未传 `--base` 命中真实默认分支」→ Task 2/3 的成功路径测试 + Task 5 调用点改造
  - 「非 `main` 默认分支仓库验证」→ Task 2/3 测试用 `"dev"` 作为 mock 返回值，即覆盖此场景
  - 「显式 `--base` 行为不变」→ Task 5 `match base { Some(b) => b, ... }` 结构保证，
    且不触发 provider 调用
- **命令层集成测试范围说明**：`apps/cli/src/commands/pr.rs::handle()` 直接构造具体
  provider（无依赖注入），无法在不重构的前提下对 `handle()` 做端到端 mock 测试。
  Task 5 因此将 fallback 逻辑抽成纯函数 `resolve_default_branch` 单独测试（对齐本文件
  `resolve_body`/`resolve_head` 的既有测试模式），调用点本身的正确性由代码审查 +
  `cargo build` 类型检查保证，属合理的范围裁剪（YAGNI），不属于遗漏。
