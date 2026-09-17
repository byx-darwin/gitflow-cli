# EnvSource 注入消除测试级进程环境竞态 — 实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 把三个平台 adapter 对进程级环境变量的读取收敛到一个可注入的 `EnvSource` 抽象，使单元测试不再写进程 env，从源头消除 `cargo test` 的并行竞态。

**Architecture:** 在共享 crate `gitflow-cli-adapter-utils` 新增 `EnvSource` trait 与 `RealEnv` 实现（与既有 `CommandRunner` / `RealCommandRunner` 并列），另提供 feature 门控的 `MockEnv`。三个 provider 结构体各增加一个**带默认值**的泛型参数 `E: EnvSource = RealEnv`，因此所有现存调用点无需改动；测试通过新增的 `with_runner_and_env` 注入确定性 env。

**Tech Stack:** Rust 2024 · `async-trait` · `tokio` (test) · cargo workspace features (resolver = "3")

**Spec:** `docs/superpowers/specs/2026-09-17-envsource-injection-design.md`

**Issue:** [#359](https://github.com/byx-darwin/gitflow-cli/issues/359) · **Workflow:** `wf-2026-09-17-002`（standard 模式）

## Global Constraints

- Rust 2024 edition，工具链固定于 `rust-toolchain.toml`，不得改动该文件
- 工作区 lint 策略：`unsafe_code = "forbid"`、`missing_docs = "warn"`、`missing_debug_implementations = "warn"` —— **所有新增 public 项必须有文档注释并派生 `Debug`**
- 生产代码禁止 `unwrap()` / `expect()`；测试中允许（工作区已对 `cfg(test)` 放开）
- 测试命名 `test_should_<expected_behavior>`，单元测试置于同文件 `#[cfg(test)] mod tests`
- 禁止引入 `serial_test`、`--test-threads=1` 或任何串行化手段（Issue #359 验收第 3 条）
- 禁止 `cargo clean`
- 每个 Task 结束时提交；**提交信息用 conventional commits**
- clippy 必须通过 `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`
- `EnvSource::var` 的语义固定为：**返回 `Option<String>`，把 `NotPresent` 与 `NotUnicode` 一并折叠为 `None`**（与被替换的 `std::env::var(..).ok()` 行为一致）

## File Structure

| 文件 | 职责 | 动作 |
|---|---|---|
| `crates/cli-adapter-utils/src/lib.rs` | `EnvSource` / `RealEnv` / `MockEnv` 的唯一定义处；进程 env 读取的唯一实现 | 修改 |
| `crates/cli-adapter-utils/Cargo.toml` | 新增 `test-util` feature | 修改 |
| `crates/gitlab/src/auth.rs` | GitLab provider：4 处 env 读取改为注入 | 修改 |
| `crates/gitlab/Cargo.toml` | dev-dep 开启 `test-util`，移除 `temp-env` | 修改 |
| `crates/github/src/auth.rs` | GitHub provider：2 处 env 读取改为注入 | 修改 |
| `crates/github/Cargo.toml` | 同上 | 修改 |
| `crates/gitcode/src/auth.rs` | GitCode provider：2 处 env 读取改为注入 | 修改 |
| `crates/gitcode/Cargo.toml` | 同上 | 修改 |
| `Cargo.toml`（workspace） | 移除 `temp-env` 依赖定义 | 修改 |
| `apps/cli/Cargo.toml` | 移除零使用的 `temp-env` 死依赖 | 修改 |

不新建文件：`EnvSource` 与 `CommandRunner` 是同一类抽象（进程边界的可替换来源），
`cli-adapter-utils/src/lib.rs` 当前仅 ~100 行，放在一起符合「一起变更的代码放在一起」。

---

### Task 1: EnvSource 抽象与测试替身

**Files:**
- Modify: `crates/cli-adapter-utils/src/lib.rs`（在 `CommandRunner` 定义之后追加）
- Modify: `crates/cli-adapter-utils/Cargo.toml`（新增 `[features]`）
- Test: `crates/cli-adapter-utils/src/lib.rs` 的 `#[cfg(test)] mod tests`

**Interfaces:**
- Consumes: 无（本 Task 是依赖链起点）
- Produces:
  - `pub trait EnvSource: std::fmt::Debug + Send + Sync { fn var(&self, key: &str) -> Option<String>; }`
  - `pub struct RealEnv;`（`Debug + Clone + Default`，实现 `EnvSource`）
  - `pub struct MockEnv;`（feature `test-util`；`Debug + Clone + Default`，实现 `EnvSource`）
  - `MockEnv::empty() -> MockEnv`
  - `MockEnv::with(key: &str, value: &str) -> MockEnv`

- [ ] **Step 1: 写失败测试**

在 `crates/cli-adapter-utils/src/lib.rs` 末尾追加（若已有 `mod tests` 则并入）：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_return_none_from_mock_env_when_key_absent() {
        let env = MockEnv::empty();
        assert_eq!(env.var("GL_TOKEN"), None);
    }

    #[test]
    fn test_should_return_value_from_mock_env_when_key_present() {
        let env = MockEnv::with("GL_TOKEN", "glpat-mock");
        assert_eq!(env.var("GL_TOKEN"), Some("glpat-mock".to_string()));
    }

    #[test]
    fn test_should_isolate_unrelated_keys_in_mock_env() {
        let env = MockEnv::with("GL_TOKEN", "glpat-mock");
        assert_eq!(env.var("GITLAB_HOST"), None);
    }

    #[test]
    fn test_should_read_real_process_env_through_real_env() {
        // PATH 在所有支持的平台上均已设置，且本测试只读不写，
        // 因此不会与并行测试产生竞态。
        let env = RealEnv;
        assert!(env.var("PATH").is_some());
    }

    #[test]
    fn test_should_return_none_from_real_env_for_absent_key() {
        let env = RealEnv;
        assert_eq!(env.var("GF_DEFINITELY_NOT_SET_12345"), None);
    }
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p gitflow-cli-adapter-utils --all-features`
Expected: 编译失败，`cannot find type MockEnv` / `cannot find type RealEnv`

- [ ] **Step 3: 新增 feature**

在 `crates/cli-adapter-utils/Cargo.toml` 的 `[dependencies]` **之前**插入：

```toml
[features]
## Exposes `MockEnv` for downstream crates' unit tests. Never enable in production builds.
test-util = []
```

- [ ] **Step 4: 实现 EnvSource / RealEnv / MockEnv**

在 `crates/cli-adapter-utils/src/lib.rs` 中 `RealCommandRunner` 的实现之后追加：

```rust
/// Read-only access to process environment variables.
///
/// Abstracts environment lookups so tests can inject deterministic values
/// instead of mutating process-global state. Mutating the real environment
/// races with other tests running in parallel threads of the same process.
pub trait EnvSource: std::fmt::Debug + Send + Sync {
    /// Return the value of `key`.
    ///
    /// Returns `None` when the variable is unset or its value is not valid
    /// UTF-8 — both cases are indistinguishable to callers, matching the
    /// `std::env::var(..).ok()` behaviour this trait replaces.
    fn var(&self, key: &str) -> Option<String>;
}

/// Default [`EnvSource`] reading the real process environment.
///
/// Used in production by all platform adapter crates.
#[derive(Debug, Clone, Copy, Default)]
pub struct RealEnv;

impl EnvSource for RealEnv {
    fn var(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }
}

/// In-memory [`EnvSource`] for tests.
///
/// Holds a fixed set of variables and never touches the process environment,
/// so tests using it are immune both to parallel-test interference and to
/// whatever the developer happens to have exported in their shell.
///
/// # Examples
///
/// ```
/// use gitflow_cli_adapter_utils::{EnvSource, MockEnv};
///
/// let env = MockEnv::with("GL_TOKEN", "glpat-example");
/// assert_eq!(env.var("GL_TOKEN"), Some("glpat-example".to_string()));
/// assert_eq!(env.var("GITLAB_HOST"), None);
/// ```
#[cfg(feature = "test-util")]
#[derive(Debug, Clone, Default)]
pub struct MockEnv {
    /// Fixed variable table consulted by [`EnvSource::var`].
    vars: std::collections::HashMap<String, String>,
}

#[cfg(feature = "test-util")]
impl MockEnv {
    /// Create a source where every lookup returns `None`.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Create a source holding exactly one variable.
    #[must_use]
    pub fn with(key: &str, value: &str) -> Self {
        let mut vars = std::collections::HashMap::new();
        vars.insert(key.to_string(), value.to_string());
        Self { vars }
    }
}

#[cfg(feature = "test-util")]
impl EnvSource for MockEnv {
    fn var(&self, key: &str) -> Option<String> {
        self.vars.get(key).cloned()
    }
}
```

- [ ] **Step 5: 运行测试确认通过**

Run: `cargo test -p gitflow-cli-adapter-utils --all-features`
Expected: PASS（5 个新测试全绿；doctest 亦需通过）

- [ ] **Step 6: 确认 test-util 未渗入默认构建**

Run: `cargo build -p gitflow-cli-adapter-utils 2>&1 && cargo tree -p gitflow-cli-adapter-utils -e features | head -20`
Expected: 构建成功；`MockEnv` 未参与默认 feature 集

- [ ] **Step 7: lint**

Run: `cargo clippy -p gitflow-cli-adapter-utils --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: 无警告

- [ ] **Step 8: 提交**

```bash
git add crates/cli-adapter-utils/src/lib.rs crates/cli-adapter-utils/Cargo.toml
git commit -m "feat(adapter-utils): add injectable EnvSource abstraction

Adds EnvSource/RealEnv plus a feature-gated MockEnv so adapter crates can
read environment variables through an injectable source instead of reaching
for std::env directly. Refs #359"
```

---

### Task 2: GitLab provider 注入 env（本 Issue 的核心修复）

**Files:**
- Modify: `crates/gitlab/src/auth.rs`（结构体 :32-35、impl 头 :38/:59/:75-76/:203-204、env 读取 :158/:164/:205/:218、测试 :343-362 与 8 处 `token()` 测试）
- Modify: `crates/gitlab/Cargo.toml`（dev-dependencies）
- Test: `crates/gitlab/src/auth.rs` 的 `#[cfg(test)] mod tests`

**Interfaces:**
- Consumes: Task 1 的 `EnvSource` / `RealEnv` / `MockEnv`（`MockEnv::empty()`、`MockEnv::with(&str, &str)`）
- Produces:
  - `GitLabAuthProvider<R: CommandRunner = RealCommandRunner, E: EnvSource = RealEnv>`
  - `GitLabAuthProvider::with_runner_and_env(runner: R, env: E) -> GitLabAuthProvider<R, E>`
  - 测试内私有 helper `fn provider(runner: MockCommandRunner) -> GitLabAuthProvider<MockCommandRunner, MockEnv>`

- [ ] **Step 1: 开启 dev-dependency feature**

`crates/gitlab/Cargo.toml` 的 `[dev-dependencies]` 改为（**暂不动 `temp-env`**，Task 5 统一清理）：

```toml
[dev-dependencies]
tokio.workspace = true
temp-env.workspace = true
gitflow-cli-adapter-utils = { workspace = true, features = ["test-util"] }
```

- [ ] **Step 2: 写失败测试**

在 `crates/gitlab/src/auth.rs` 的 `mod tests` 内、`use crate::runner::MockCommandRunner;` 之后追加：

```rust
    use gitflow_cli_adapter_utils::MockEnv;

    /// Build a provider whose environment is empty, so tests never depend on
    /// what the host shell exported.
    fn provider(runner: MockCommandRunner) -> GitLabAuthProvider<MockCommandRunner, MockEnv> {
        GitLabAuthProvider::with_runner_and_env(runner, MockEnv::empty())
    }

    #[tokio::test]
    async fn test_should_prefer_runner_over_absent_env_token() {
        let stdout = "  ✓ Token found in operating system keyring: glpat-abcdef\n";
        let runner = MockCommandRunner::success(stdout);
        let provider = GitLabAuthProvider::with_runner_and_env(runner, MockEnv::empty());

        let token = provider.token().await.expect("should get token");

        assert_eq!(token, "glpat-abcdef");
    }

    #[tokio::test]
    async fn test_should_short_circuit_token_when_env_var_present() {
        // 语义回归护栏：GL_TOKEN 命中时必须优先于 CLI 调用。
        let runner = MockCommandRunner::success("  ✓ Token found in keyring: glpat-from-cli\n");
        let provider =
            GitLabAuthProvider::with_runner_and_env(runner.clone(), MockEnv::with("GL_TOKEN", "glpat-from-env"));

        let token = provider.token().await.expect("should get token");

        assert_eq!(token, "glpat-from-env");
        assert!(
            runner.recorded_calls().is_empty(),
            "env short-circuit must not spawn glab"
        );
    }

    #[tokio::test]
    async fn test_should_append_hostname_arg_when_gitlab_host_present() {
        // 语义回归护栏：GITLAB_HOST 命中时追加 --hostname。
        let stdout = "  ✓ Token found in operating system keyring: glpat-abcdef\n";
        let runner = MockCommandRunner::success(stdout);
        let provider = GitLabAuthProvider::with_runner_and_env(
            runner.clone(),
            MockEnv::with("GITLAB_HOST", "gitlab.example.com"),
        );

        provider.token().await.expect("should get token");

        assert_eq!(
            runner.recorded_calls()[0].1,
            vec!["auth", "status", "--show-token", "--hostname", "gitlab.example.com"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );
    }
```

- [ ] **Step 3: 运行测试确认失败**

Run: `cargo test -p gitflow-gitlab --lib`
Expected: 编译失败，`no function or associated item named with_runner_and_env found`

- [ ] **Step 4: 改结构体与构造函数**

`crates/gitlab/src/auth.rs`。先扩展 import（:15-18）：

```rust
use crate::{
    error::parse_glab_error,
    runner::{CommandRunner, RealCommandRunner},
};
use gitflow_cli_adapter_utils::{EnvSource, RealEnv};
```

结构体（:32-35）改为：

```rust
#[derive(Debug, Clone)]
pub struct GitLabAuthProvider<R: CommandRunner = RealCommandRunner, E: EnvSource = RealEnv> {
    /// 用于执行 `glab` CLI 命令的 runner。
    runner: R,
    /// 环境变量来源，生产环境为进程环境，测试可注入。
    env: E,
}
```

`impl GitLabAuthProvider<RealCommandRunner>`（:38）改为显式写全两个参数，并补 `env` 字段：

```rust
impl GitLabAuthProvider<RealCommandRunner, RealEnv> {
    /// 创建新的 GitLab 认证提供者，使用真实的进程执行器与进程环境。
    #[must_use]
    pub fn new() -> Self {
        Self {
            runner: RealCommandRunner,
            env: RealEnv,
        }
    }

    /// Create a new provider from a shared [`Session`].
    ///
    /// `AuthProvider` doesn't use repository context, so session.repo is ignored.
    /// This enables state reuse across multiple operations in workflow chains.
    #[must_use]
    pub fn with_session(_session: &gitflow_core::Session) -> Self {
        Self {
            runner: RealCommandRunner,
            env: RealEnv,
        }
    }
}
```

`impl<R: CommandRunner> GitLabAuthProvider<R>`（:59）拆成两块：

```rust
impl<R: CommandRunner> GitLabAuthProvider<R, RealEnv> {
    /// 使用自定义 [`CommandRunner`] 创建提供者，环境变量仍取自进程环境。
    ///
    /// 主要用于测试，可注入模拟 runner 以控制 `glab` CLI 的输出。
    #[must_use]
    pub fn with_runner(runner: R) -> Self {
        Self {
            runner,
            env: RealEnv,
        }
    }
}

impl<R: CommandRunner, E: EnvSource> GitLabAuthProvider<R, E> {
    /// 同时注入自定义 [`CommandRunner`] 与 [`EnvSource`]。
    ///
    /// 测试应优先使用本构造函数：注入的环境变量来源使测试不依赖进程环境，
    /// 因而既不会被并行测试污染，也不受开发者 shell 中已导出变量的影响。
    #[must_use]
    pub fn with_runner_and_env(runner: R, env: E) -> Self {
        Self { runner, env }
    }
}
```

`impl Default`（:68-72）改为：

```rust
impl Default for GitLabAuthProvider<RealCommandRunner, RealEnv> {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **Step 5: 改两个 trait impl 的泛型头**

`#[async_trait] impl<R: CommandRunner + 'static> AuthProvider for GitLabAuthProvider<R>`（:75-76）改为：

```rust
#[async_trait]
impl<R: CommandRunner + 'static, E: EnvSource + 'static> AuthProvider for GitLabAuthProvider<R, E> {
```

`impl<R: CommandRunner> gitflow_core::AuthChecker for GitLabAuthProvider<R>`（:203-204）改为：

```rust
impl<R: CommandRunner, E: EnvSource> gitflow_core::AuthChecker for GitLabAuthProvider<R, E> {
```

- [ ] **Step 6: 改 4 处 env 读取**

`token()` 内（:158 与 :164）：

```rust
        // 环境变量优先（与 AuthChecker::is_authenticated 一致）
        if let Some(tok) = self.env.var("GL_TOKEN") {
            return Ok(tok);
        }

        debug!("spawning `glab auth status --show-token`");

        let host = self.env.var("GITLAB_HOST");
```

`is_authenticated()` 内（:205）与 `check_status()` 内（:218），两处同形：

```rust
        if self.env.var("GL_TOKEN").is_some() {
```

- [ ] **Step 7: 改造既有测试**

两个 env 测试（:343-362）改为注入式，不再写进程环境：

```rust
    #[test]
    fn test_auth_checker_is_authenticated_with_env_var() {
        use gitflow_core::AuthChecker;
        let provider = GitLabAuthProvider::with_runner_and_env(
            MockCommandRunner::success(""),
            MockEnv::with("GL_TOKEN", "test_token"),
        );
        assert!(provider.is_authenticated());
    }

    #[test]
    fn test_auth_checker_check_status_with_env_var() {
        use gitflow_core::AuthChecker;
        let provider = GitLabAuthProvider::with_runner_and_env(
            MockCommandRunner::success(""),
            MockEnv::with("GL_TOKEN", "test_token"),
        );
        let result = provider.check_status();
        assert!(result.authenticated);
        assert!(result.reason.is_none());
    }
```

随后把 **8 处 `token()` 测试**中的 `GitLabAuthProvider::with_runner(runner)` 替换为
`provider(runner)`（Step 2 定义的 helper），使其注入空 env。涉及的测试函数：

1. `test_should_return_platform_error_when_glab_fails_for_token`
2. `test_should_error_when_stdout_has_no_token_line`
3. `test_should_return_token_successfully`
4. `test_should_trim_whitespace_from_token`
5. `test_should_extract_token_from_auth_status_show_token`（注意此处原为 `with_runner(runner.clone())` → `provider(runner.clone())`）
6. `test_should_extract_token_from_stderr_like_real_glab`
7. `test_should_error_when_no_token_found`
8. `test_should_return_platform_error_when_token_spawn_fails`

其余不调用 `token()` 的测试保持 `with_runner` 不变 —— 它们不读 env，无需改动，
也正好验证 `with_runner` 的向后兼容性未被破坏。

- [ ] **Step 8: 运行测试确认通过**

Run: `cargo test -p gitflow-gitlab --lib`
Expected: PASS，0 failed

- [ ] **Step 9: 主验收 —— 宿主环境注入（确定性判据）**

> **为什么这一步是主验收，而非「连续 5 次全绿」。** Phase 2 闸门实测记录：在**未做任何修复**的
> `dev` 分支上，`cargo test -p gitflow-gitlab --lib` 连跑 5 次全部 `ok`，`cargo llvm-cov`
> 也在不加 `--ignore-run-fail` 的情况下 exit 0（85.78%）—— 尽管同一会话早些时候确实复现过
> `259 passed; 1 failed`。并行竞态是概率性的，**「多次全绿」可以在修复前靠运气达成，
> 因此不能作为主判据**。下面两条注入则在修复前 100% 复现（exit 101）。

分两条独立运行，便于定位：

Run: `GL_TOKEN=host-leak cargo test -p gitflow-gitlab --lib 2>&1 | grep -E '^test result:'`
Expected: `test result: ok.`
修复前基线（实测）：`FAILED. 252 passed; 8 failed` —— 失败的正是本 Task Step 7 列出的 8 个 `token()` 测试

Run: `GITLAB_HOST=h.example.com cargo test -p gitflow-gitlab --lib 2>&1 | grep -E '^test result:'`
Expected: `test result: ok.`
修复前基线（实测）：`FAILED. 259 passed; 1 failed` —— 失败的正是 `test_should_extract_token_from_auth_status_show_token`

- [ ] **Step 10: 辅助验收 —— 连续 5 次全绿**

Run: `for i in 1 2 3 4 5; do cargo test -p gitflow-gitlab --lib 2>&1 | grep -E '^test result:'; done`
Expected: 5 行均为 `test result: ok.`

这条对应 Issue #359 验收第 1 条，仍需执行并记录，但它**只能证伪不能证实** ——
5 次全绿不构成修复成立的证据，Step 9 才是。

- [ ] **Step 11: 验证下游调用点未被破坏**

设计的向后兼容性完全押在「带默认值的类型参数」上。`apps/cli` 有 9 处构造点
（`commands/auth.rs:59-61`、`commands/doctor.rs:164-166`、`commands/prerequisites.rs:301-303`），
全部走 `::new()` 并 box 成 `dyn AuthProvider` / `dyn AuthChecker`。
**在第一个 provider 改完时就验证**，不要拖到 Task 5 才发现假设不成立。

Run: `cargo check -p gitflow-cli --all-targets`
Expected: 编译成功，且 `apps/cli` 无任何改动

- [ ] **Step 12: lint**

Run: `cargo clippy -p gitflow-gitlab --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: 无警告

- [ ] **Step 13: 提交**

```bash
git add crates/gitlab/src/auth.rs crates/gitlab/Cargo.toml
git commit -m "fix(gitlab): inject env source into auth provider

GitLabAuthProvider now reads GL_TOKEN and GITLAB_HOST through an injected
EnvSource instead of std::env, so its tests no longer mutate process-global
state and no longer depend on the developer's exported shell variables.
Refs #359"
```

---

### Task 3: GitHub provider 注入 env

**Files:**
- Modify: `crates/github/src/auth.rs`（结构体 :33-35、impl 头 :38/:59/:69/:75-76/:223、env 读取 :225/:238、测试 :463-481）
- Modify: `crates/github/Cargo.toml`（dev-dependencies）
- Test: `crates/github/src/auth.rs` 的 `#[cfg(test)] mod tests`

**Interfaces:**
- Consumes: Task 1 的 `EnvSource` / `RealEnv` / `MockEnv`
- Produces: `GitHubAuthProvider<R: CommandRunner = RealCommandRunner, E: EnvSource = RealEnv>` 与 `with_runner_and_env(runner: R, env: E)`

**注意**：GitHub 的 `token()` 直接调用 `gh auth token`，**不读 env**，因此本 crate 只有
`is_authenticated()` 与 `check_status()` 两处读取。

- [ ] **Step 1: 开启 dev-dependency feature**

`crates/github/Cargo.toml` 的 `[dev-dependencies]` 追加一行（暂不动 `temp-env`）：

```toml
gitflow-cli-adapter-utils = { workspace = true, features = ["test-util"] }
```

- [ ] **Step 2: 写失败测试**

在 `crates/github/src/auth.rs` 的 `mod tests` 内追加：

**注意**：`AuthChecker::is_authenticated` / `check_status` 的**非 env 分支直接使用
`std::process::Command` 而非注入的 runner**（该 impl 带
`#[allow(clippy::disallowed_types, reason = "AuthChecker is synchronous")]`）。
因此只为 env **命中**分支写测试 —— 未命中分支会真的去 shell 里找 `gh`，
结果取决于本机是否安装，不具确定性，不应写成单元测试。

```rust
    use gitflow_cli_adapter_utils::MockEnv;

    #[test]
    fn test_should_short_circuit_auth_check_when_env_var_present() {
        use gitflow_core::AuthChecker;
        let provider = GitHubAuthProvider::with_runner_and_env(
            MockCommandRunner::success(""),
            MockEnv::with("GH_TOKEN", "gho-from-env"),
        );
        assert!(provider.is_authenticated());
    }

    #[test]
    fn test_should_report_authenticated_status_from_env_without_reason() {
        use gitflow_core::AuthChecker;
        let provider = GitHubAuthProvider::with_runner_and_env(
            MockCommandRunner::success(""),
            MockEnv::with("GH_TOKEN", "gho-from-env"),
        );
        let result = provider.check_status();
        assert!(result.authenticated);
        assert!(result.reason.is_none());
        assert!(result.hint.is_none());
    }
```

- [ ] **Step 3: 运行测试确认失败**

Run: `cargo test -p gitflow-github --lib`
Expected: 编译失败，`no function or associated item named with_runner_and_env found`

- [ ] **Step 4: 改结构体与构造函数**

扩展 import：

```rust
use gitflow_cli_adapter_utils::{EnvSource, RealEnv};
```

结构体（:33-35）：

```rust
#[derive(Debug, Clone)]
pub struct GitHubAuthProvider<R: CommandRunner = RealCommandRunner, E: EnvSource = RealEnv> {
    /// 用于执行 `gh` CLI 命令的 runner。
    runner: R,
    /// 环境变量来源，生产环境为进程环境，测试可注入。
    env: E,
}
```

`impl GitHubAuthProvider<RealCommandRunner>`（:38）→ `impl GitHubAuthProvider<RealCommandRunner, RealEnv>`，
其中 `new()`（:41）与 `with_session()`（:52）的构造体各补 `env: RealEnv,`。

`impl<R: CommandRunner> GitHubAuthProvider<R>`（:59）拆成两块：

```rust
impl<R: CommandRunner> GitHubAuthProvider<R, RealEnv> {
    /// 使用自定义 [`CommandRunner`] 创建提供者，环境变量仍取自进程环境。
    #[must_use]
    pub fn with_runner(runner: R) -> Self {
        Self {
            runner,
            env: RealEnv,
        }
    }
}

impl<R: CommandRunner, E: EnvSource> GitHubAuthProvider<R, E> {
    /// 同时注入自定义 [`CommandRunner`] 与 [`EnvSource`]。
    ///
    /// 测试应优先使用本构造函数：注入的环境变量来源使测试不依赖进程环境。
    #[must_use]
    pub fn with_runner_and_env(runner: R, env: E) -> Self {
        Self { runner, env }
    }
}
```

`impl Default for GitHubAuthProvider`（:69）改为 `impl Default for GitHubAuthProvider<RealCommandRunner, RealEnv>`。

- [ ] **Step 5: 改两个 trait impl 的泛型头**

```rust
#[async_trait]
impl<R: CommandRunner + 'static, E: EnvSource + 'static> AuthProvider for GitHubAuthProvider<R, E> {
```

```rust
impl<R: CommandRunner, E: EnvSource> gitflow_core::AuthChecker for GitHubAuthProvider<R, E> {
```

- [ ] **Step 6: 改 2 处 env 读取**

`is_authenticated()`（:225）与 `check_status()`（:238）两处同形：

```rust
        if self.env.var("GH_TOKEN").is_some() {
```

- [ ] **Step 7: 改造既有的两个 env 测试**

```rust
    #[test]
    fn test_auth_checker_is_authenticated_with_env_var() {
        use gitflow_core::AuthChecker;
        let provider = GitHubAuthProvider::with_runner_and_env(
            MockCommandRunner::success(""),
            MockEnv::with("GH_TOKEN", "test_token"),
        );
        assert!(provider.is_authenticated());
    }

    #[test]
    fn test_auth_checker_check_status_with_env_var() {
        use gitflow_core::AuthChecker;
        let provider = GitHubAuthProvider::with_runner_and_env(
            MockCommandRunner::success(""),
            MockEnv::with("GH_TOKEN", "test_token"),
        );
        let result = provider.check_status();
        assert!(result.authenticated);
        assert!(result.reason.is_none());
    }
```

- [ ] **Step 8: 运行测试确认通过**

Run: `cargo test -p gitflow-github --lib`
Expected: PASS，0 failed

- [ ] **Step 9: 验证对宿主环境免疫**

Run: `GH_TOKEN=host-leak cargo test -p gitflow-github --lib 2>&1 | tail -1`
Expected: `test result: ok.`

- [ ] **Step 10: lint**

Run: `cargo clippy -p gitflow-github --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: 无警告

- [ ] **Step 11: 提交**

```bash
git add crates/github/src/auth.rs crates/github/Cargo.toml
git commit -m "fix(github): inject env source into auth provider

GitHubAuthProvider now reads GH_TOKEN through an injected EnvSource, so its
auth-check tests no longer mutate process-global state. Refs #359"
```

---

### Task 4: GitCode provider 注入 env

**Files:**
- Modify: `crates/gitcode/src/auth.rs`（结构体 :32-34、impl 头 :37/:57/:67/:73-74/:214、env 读取 :217/:248、测试 :394-412）
- Modify: `crates/gitcode/Cargo.toml`（dev-dependencies）
- Test: `crates/gitcode/src/auth.rs` 的 `#[cfg(test)] mod tests`

**Interfaces:**
- Consumes: Task 1 的 `EnvSource` / `RealEnv` / `MockEnv`
- Produces: `GitCodeAuthProvider<R: CommandRunner = RealCommandRunner, E: EnvSource = RealEnv>` 与 `with_runner_and_env(runner: R, env: E)`

**注意**：GitCode 的 `token()` 调用 `gc auth token`，**不读 env**；本 crate 只有
`is_authenticated()` 与 `check_status()` 两处读取。另注意 `with_session` 的参数类型是
`&Session`（该文件已 `use` 了 `Session`），与 gitlab/github 的 `&gitflow_core::Session` 写法不同，**保持原样**。

- [ ] **Step 1: 开启 dev-dependency feature**

`crates/gitcode/Cargo.toml` 的 `[dev-dependencies]` 追加一行（暂不动 `temp-env`）：

```toml
gitflow-cli-adapter-utils = { workspace = true, features = ["test-util"] }
```

- [ ] **Step 2: 写失败测试**

在 `crates/gitcode/src/auth.rs` 的 `mod tests` 内追加：

```rust
    use gitflow_cli_adapter_utils::MockEnv;

    #[test]
    fn test_should_short_circuit_auth_check_when_env_var_present() {
        use gitflow_core::AuthChecker;
        let provider = GitCodeAuthProvider::with_runner_and_env(
            MockCommandRunner::success(""),
            MockEnv::with("GITCODE_TOKEN", "gc-from-env"),
        );
        assert!(provider.is_authenticated());
    }

    #[test]
    fn test_should_report_authenticated_status_from_env_without_reason() {
        use gitflow_core::AuthChecker;
        let provider = GitCodeAuthProvider::with_runner_and_env(
            MockCommandRunner::success(""),
            MockEnv::with("GITCODE_TOKEN", "gc-from-env"),
        );
        let result = provider.check_status();
        assert!(result.authenticated);
        assert!(result.reason.is_none());
        assert!(result.hint.is_none());
    }
```

- [ ] **Step 3: 运行测试确认失败**

Run: `cargo test -p gitflow-gitcode --lib`
Expected: 编译失败，`no function or associated item named with_runner_and_env found`

- [ ] **Step 4: 改结构体与构造函数**

扩展 import：

```rust
use gitflow_cli_adapter_utils::{EnvSource, RealEnv};
```

结构体（:32-34）：

```rust
#[derive(Debug, Clone)]
pub struct GitCodeAuthProvider<R: CommandRunner = RealCommandRunner, E: EnvSource = RealEnv> {
    /// 用于执行 GitCode CLI 命令的 runner。
    runner: R,
    /// 环境变量来源，生产环境为进程环境，测试可注入。
    env: E,
}
```

`impl GitCodeAuthProvider<RealCommandRunner>`（:37）→ `impl GitCodeAuthProvider<RealCommandRunner, RealEnv>`，
其中 `new()`（:40）与 `with_session()`（:50）的构造体各补 `env: RealEnv,`。

`impl<R: CommandRunner> GitCodeAuthProvider<R>`（:57）拆成两块：

```rust
impl<R: CommandRunner> GitCodeAuthProvider<R, RealEnv> {
    /// 使用自定义 [`CommandRunner`] 创建提供者，环境变量仍取自进程环境。
    #[must_use]
    pub fn with_runner(runner: R) -> Self {
        Self {
            runner,
            env: RealEnv,
        }
    }
}

impl<R: CommandRunner, E: EnvSource> GitCodeAuthProvider<R, E> {
    /// 同时注入自定义 [`CommandRunner`] 与 [`EnvSource`]。
    ///
    /// 测试应优先使用本构造函数：注入的环境变量来源使测试不依赖进程环境。
    #[must_use]
    pub fn with_runner_and_env(runner: R, env: E) -> Self {
        Self { runner, env }
    }
}
```

`impl Default for GitCodeAuthProvider<RealCommandRunner>`（:67）→ `impl Default for GitCodeAuthProvider<RealCommandRunner, RealEnv>`。

- [ ] **Step 5: 改两个 trait impl 的泛型头**

```rust
#[async_trait]
impl<R: CommandRunner + 'static, E: EnvSource + 'static> AuthProvider for GitCodeAuthProvider<R, E> {
```

```rust
impl<R: CommandRunner, E: EnvSource> gitflow_core::AuthChecker for GitCodeAuthProvider<R, E> {
```

- [ ] **Step 6: 改 2 处 env 读取**

`is_authenticated()`（:217）与 `check_status()`（:248）两处同形：

```rust
        if self.env.var("GITCODE_TOKEN").is_some() {
```

- [ ] **Step 7: 改造既有的两个 env 测试**

```rust
    #[test]
    fn test_auth_checker_is_authenticated_with_env_var() {
        use gitflow_core::AuthChecker;
        let provider = GitCodeAuthProvider::with_runner_and_env(
            MockCommandRunner::success(""),
            MockEnv::with("GITCODE_TOKEN", "test_token"),
        );
        assert!(provider.is_authenticated());
    }

    #[test]
    fn test_auth_checker_check_status_with_env_var() {
        use gitflow_core::AuthChecker;
        let provider = GitCodeAuthProvider::with_runner_and_env(
            MockCommandRunner::success(""),
            MockEnv::with("GITCODE_TOKEN", "test_token"),
        );
        let result = provider.check_status();
        assert!(result.authenticated);
        assert!(result.reason.is_none());
    }
```

- [ ] **Step 8: 运行测试确认通过**

Run: `cargo test -p gitflow-gitcode --lib`
Expected: PASS，0 failed

- [ ] **Step 9: 验证对宿主环境免疫**

Run: `GITCODE_TOKEN=host-leak cargo test -p gitflow-gitcode --lib 2>&1 | tail -1`
Expected: `test result: ok.`

- [ ] **Step 10: lint**

Run: `cargo clippy -p gitflow-gitcode --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: 无警告

- [ ] **Step 11: 提交**

```bash
git add crates/gitcode/src/auth.rs crates/gitcode/Cargo.toml
git commit -m "fix(gitcode): inject env source into auth provider

GitCodeAuthProvider now reads GITCODE_TOKEN through an injected EnvSource,
so its auth-check tests no longer mutate process-global state. Refs #359"
```

---

### Task 5: 移除 temp-env 死依赖并做全量验收

**Files:**
- Modify: `Cargo.toml`（workspace，:42）
- Modify: `crates/gitlab/Cargo.toml`、`crates/github/Cargo.toml`、`crates/gitcode/Cargo.toml`（各删 `temp-env.workspace = true`）
- Modify: `apps/cli/Cargo.toml`（:64，该处声明后 `apps/cli` 内零使用）

**Interfaces:**
- Consumes: Task 2/3/4 完成后 `temp_env` 在全仓库零引用
- Produces: 无新接口；本 Task 的交付物是「依赖清理 + 全量验收证据」

- [ ] **Step 1: 确认 temp_env 已零引用**

Run: `grep -rn "temp_env" --include="*.rs" . | grep -v "/target/"`
Expected: 无输出。**若有输出则停止**，回到对应 Task 补完改造

- [ ] **Step 2: 删除 5 处声明**

```bash
# workspace 依赖定义
sed -i '' '/^temp-env = "0.3"$/d' Cargo.toml
# 三个 adapter crate 的 dev-dependency
sed -i '' '/^temp-env\.workspace = true$/d' crates/gitlab/Cargo.toml crates/github/Cargo.toml crates/gitcode/Cargo.toml
# apps/cli 的死依赖
sed -i '' '/^temp-env = { workspace = true }$/d' apps/cli/Cargo.toml
```

- [ ] **Step 3: 确认 5 处声明均已移除**

Run: `grep -rn "temp-env" --include="Cargo.toml" . | grep -v "/target/"`
Expected: 无输出（`Cargo.lock` 中的条目由下一步的构建清理）

- [ ] **Step 4: 构建并更新 lockfile**

Run: `cargo build --workspace --all-targets`
Expected: 构建成功，`Cargo.lock` 中 `temp-env` 条目被移除

- [ ] **Step 5: 全量测试**

Run: `cargo test --workspace`
Expected: 全绿

- [ ] **Step 6: 验收 1 —— gitlab 连续 5 次全绿**

Run: `for i in 1 2 3 4 5; do cargo test -p gitflow-gitlab --lib 2>&1 | tail -1; done`
Expected: 5 行均为 `test result: ok.`

- [ ] **Step 7: 验收 2 —— 覆盖率闸门无需 --ignore-run-fail**

Run: `cargo llvm-cov --workspace --fail-under-lines 80 --summary-only`
Expected: 退出码 0，**命令中不含 `--ignore-run-fail`**；总行覆盖率不低于修复前基线 **85.78%**
（Phase 2 闸门实测值，2026-09-17）

> 同 Task 2 Step 9 的警告：该命令在修复前也能 exit 0（已实测），因此它验证的是
> 「没有引入回归」而非「竞态已消除」。后者由 Task 2 Step 9 的确定性判据负责。

- [ ] **Step 8: 验收 6 —— 确认未引入串行化手段**

Run: `grep -rn "serial_test\|test-threads" --include="*.rs" --include="*.toml" --include="Makefile" . | grep -v "/target/"`
Expected: 无输出

- [ ] **Step 9: 全量 lint 与格式化**

Run: `cargo +nightly fmt --all && cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: 无警告

- [ ] **Step 10: 依赖审计（依赖清单发生变更）**

Run: `cargo audit && cargo deny check`
Expected: 通过

- [ ] **Step 11: 提交**

```bash
git add Cargo.toml Cargo.lock crates/gitlab/Cargo.toml crates/github/Cargo.toml crates/gitcode/Cargo.toml apps/cli/Cargo.toml
git commit -m "chore(deps): drop temp-env after EnvSource injection

temp-env lost all call sites once the three auth providers started reading
the environment through an injected EnvSource. Also removes the declaration
in apps/cli, which had zero usages. Closes #359"
```

---

## 验收标准总表

| # | 来源 | 验收项 | 验证步骤 |
|---|---|---|---|
| 1 | Issue #359 | `cargo test -p gitflow-gitlab --lib` 连续 5 次全绿 | Task 2 Step 10、Task 5 Step 6 —— ⚠️ 概率性，仅作辅助 |
| 2 | Issue #359 | `cargo llvm-cov --workspace --fail-under-lines 80` 无需 `--ignore-run-fail` | Task 5 Step 7 —— ⚠️ 概率性，仅作辅助 |
| 3 | Issue #359 | 不依赖 `--test-threads=1` | Task 5 Step 8 |
| 4 | Issue #359 | gitcode / github 同类写法一并处理 | Task 3、Task 4 |
| 5 | issue-review | 三平台 env 短路语义不变且有测试覆盖 | Task 2 Step 2（`test_should_short_circuit_token_when_env_var_present` + `test_should_append_hostname_arg_when_gitlab_host_present`）、Task 3 Step 2、Task 4 Step 2 |
| 6 | issue-review | `temp-env` 失去全部使用点后移除其 5 处声明 | Task 5 Step 1–3 |
| 7 | Phase 2 补充 | 测试对宿主环境免疫（次生缺陷 1 与 2） | **Task 2 Step 9（主判据，修复前 100% 复现）**、Task 3 Step 9、Task 4 Step 9 |

## 任务依赖

```
Task 1 (EnvSource 抽象)
   ├── Task 2 (gitlab) ─┐
   ├── Task 3 (github) ─┼── Task 5 (清理 + 全量验收)
   └── Task 4 (gitcode) ┘
```

Task 2/3/4 相互独立，但都依赖 Task 1；Task 5 依赖 2/3/4 全部完成。
