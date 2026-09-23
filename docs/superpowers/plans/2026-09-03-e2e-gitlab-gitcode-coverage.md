# e2e-gitlab / e2e-gitcode Coverage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `crates/e2e-gitlab` and `crates/e2e-gitcode`, matching `crates/e2e-github`'s
`auth`/`issue`/`noauth`/`pr` coverage depth, by extending `e2e-core`'s shared harness and
wiring two new CI jobs into `.github/workflows/e2e-tests.yml`.

**Architecture:** `e2e-core` gains GitLab/GitCode analogues of its existing GitHub-only
helpers (`gl_env()`/`gitcode_env()` mirroring `gh_env()`, `gitlab_mode()`/`gitcode_mode()`
mirroring `mode()`) plus two net-new primitives — `TtyRunner::dir()` (working-directory
override) and `scratch_repo_dir()` (a throwaway git checkout whose `origin` remote points at
a real platform repo) — needed because, unlike GitHub, this repo's own `git remote origin`
never points at GitLab/GitCode. Each new crate replicates `e2e-github`'s four test files
1:1, swapping provider-specific env vars/binaries.

**Tech Stack:** Rust 2024, tokio (async test runtime), `tempfile` (scratch checkouts),
`serde_json` (schema assertions), `cargo-nextest` (CI runner), `glab`/`gc` CLI (real
subprocess dependencies for GitLab/GitCode auth flows).

**Spec:** `docs/superpowers/specs/2026-09-03-e2e-gitlab-gitcode-coverage-design.md`

## Global Constraints

- No production CLI code changes (`apps/cli`, `crates/gitlab`, `crates/gitcode` are
  untouched) — this is a test-infrastructure-only change.
- No new GitHub Secrets — `E2E_GITLAB_TOKEN`/`E2E_GITCODE_TOKEN`/`E2E_TEST_REPO_GITLAB`/
  `E2E_TEST_REPO_GITCODE` stay unconfigured; every new real-credential/real-repo test path
  must skip gracefully when they are absent (mirrors the existing `E2E_GITHUB_TOKEN`
  skip convention in `crates/e2e-github/tests/auth.rs`).
- `e2e-core`'s existing GitHub-facing API (`mode()`, `gh_env()`, `has_github_auth()`,
  `TestConfig::from_env()`/`from_env_lenient()` GitHub fields) must stay backward
  compatible — `crates/e2e-github` tests must keep passing unmodified.
- New test functions are named `test_should_<expected_behavior>` (repo TDD convention).
- Every fallible new function (`scratch_repo_dir`, the new `TestConfig` methods) needs both
  a success-path and a failure/edge-path unit test in `#[cfg(test)] mod tests` in the same
  file, per repo convention.
- `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic` must pass
  for every touched crate. `std::process::Command` is clippy-disallowed
  (`clippy.toml::disallowed-types`) — use `tokio::process::Command` throughout (this is
  why `scratch_repo_dir` is `async`).
- `#![forbid(unsafe_code)]` stays intact in `e2e-core::lib.rs`.

---

### Task 1: `e2e-core::TestConfig` — GitLab/GitCode fields and accessors

**Files:**
- Modify: `crates/e2e-core/src/config.rs` (whole file — see current content below)

**Interfaces:**
- Consumes: nothing new (pure additive change to `TestConfig`)
- Produces (used by Tasks 4-7):
  - `TestConfig::gitlab_test_repo: Option<String>` (field, read from `E2E_TEST_REPO_GITLAB`)
  - `TestConfig::gitcode_test_repo: Option<String>` (field, read from `E2E_TEST_REPO_GITCODE`)
  - `TestConfig::gl_env(&self) -> Vec<(String, String)>`
  - `TestConfig::gitcode_env(&self) -> Vec<(String, String)>`
  - `TestConfig::has_gitlab_auth(&self) -> bool`
  - `TestConfig::has_gitcode_auth(&self) -> bool`
  - `TestConfig::gitlab_mode(&self) -> TestMode`
  - `TestConfig::gitcode_mode(&self) -> TestMode`

- [ ] **Step 1: Write the failing tests**

Append to the `#[cfg(test)] mod tests` block in `crates/e2e-core/src/config.rs` (after the
existing `test_should_emit_empty_env_when_unauthenticated` test):

```rust
    fn config_with_gitlab_token() -> TestConfig {
        TestConfig {
            test_repo: "owner/repo".to_string(),
            github_token: None,
            gitcode_token: None,
            gitlab_token: Some("gl-token".to_string()),
            gitlab_test_repo: Some("group/project".to_string()),
            gitcode_test_repo: None,
        }
    }

    fn config_with_gitcode_token() -> TestConfig {
        TestConfig {
            test_repo: "owner/repo".to_string(),
            github_token: None,
            gitcode_token: Some("gc-token".to_string()),
            gitlab_token: None,
            gitlab_test_repo: None,
            gitcode_test_repo: Some("group/project".to_string()),
        }
    }

    #[test]
    fn test_should_derive_gitlab_authenticated_mode_when_token_present() {
        assert_eq!(config_with_gitlab_token().gitlab_mode(), TestMode::Authenticated);
    }

    #[test]
    fn test_should_derive_gitlab_unauthenticated_mode_when_no_token() {
        assert_eq!(config_without_token().gitlab_mode(), TestMode::Unauthenticated);
    }

    #[test]
    fn test_should_derive_gitcode_authenticated_mode_when_token_present() {
        assert_eq!(config_with_gitcode_token().gitcode_mode(), TestMode::Authenticated);
    }

    #[test]
    fn test_should_derive_gitcode_unauthenticated_mode_when_no_token() {
        assert_eq!(config_without_token().gitcode_mode(), TestMode::Unauthenticated);
    }

    #[test]
    fn test_should_report_gitlab_auth_presence() {
        assert!(config_with_gitlab_token().has_gitlab_auth());
        assert!(!config_without_token().has_gitlab_auth());
    }

    #[test]
    fn test_should_report_gitcode_auth_presence() {
        assert!(config_with_gitcode_token().has_gitcode_auth());
        assert!(!config_without_token().has_gitcode_auth());
    }

    #[test]
    fn test_should_emit_gl_token_env_pair_when_authenticated() {
        assert_eq!(
            config_with_gitlab_token().gl_env(),
            vec![("GL_TOKEN".to_string(), "gl-token".to_string())]
        );
    }

    #[test]
    fn test_should_emit_empty_gl_env_when_unauthenticated() {
        assert!(config_without_token().gl_env().is_empty());
    }

    #[test]
    fn test_should_emit_gitcode_token_env_pair_when_authenticated() {
        assert_eq!(
            config_with_gitcode_token().gitcode_env(),
            vec![("GITCODE_TOKEN".to_string(), "gc-token".to_string())]
        );
    }

    #[test]
    fn test_should_emit_empty_gitcode_env_when_unauthenticated() {
        assert!(config_without_token().gitcode_env().is_empty());
    }
```

Also update `config_with_token()` and `config_without_token()` (the two existing helper
functions in that `mod tests` block) to set `gitlab_test_repo: None, gitcode_test_repo: None`
— they will fail to compile once the struct grows two fields, since they use struct-literal
syntax with every field named.

- [ ] **Step 2: Run tests to verify they fail (compile error — new fields/methods don't exist yet)**

Run: `cargo test -p e2e-core`
Expected: FAIL — `no field \`gitlab_test_repo\`` / `no method named \`gitlab_mode\`` etc.

- [ ] **Step 3: Implement the minimal code**

Replace the `TestConfig` struct definition and `impl TestConfig` block in
`crates/e2e-core/src/config.rs` with:

```rust
/// 测试配置
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// 测试仓库（格式：owner/repo）
    pub test_repo: String,
    /// GitHub 令牌
    pub github_token: Option<String>,
    /// `GitCode` 令牌
    pub gitcode_token: Option<String>,
    /// GitLab 令牌
    pub gitlab_token: Option<String>,
    /// GitLab 测试仓库（格式：group/project），用于 `e2e-gitlab` 的 issue/pr 实测
    pub gitlab_test_repo: Option<String>,
    /// GitCode 测试仓库（格式：group/project），用于 `e2e-gitcode` 的 issue/pr 实测
    pub gitcode_test_repo: Option<String>,
}

impl TestConfig {
    /// 从环境变量加载配置
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::MissingEnvVar` if `E2E_TEST_REPO` is not set.
    pub fn from_env() -> Result<Self, ConfigError> {
        let test_repo = std::env::var("E2E_TEST_REPO")
            .map_err(|_| ConfigError::MissingEnvVar("E2E_TEST_REPO".to_string()))?;

        Ok(Self {
            test_repo,
            github_token: std::env::var("E2E_GITHUB_TOKEN").ok(),
            gitcode_token: std::env::var("E2E_GITCODE_TOKEN").ok(),
            gitlab_token: std::env::var("E2E_GITLAB_TOKEN").ok(),
            gitlab_test_repo: std::env::var("E2E_TEST_REPO_GITLAB").ok(),
            gitcode_test_repo: std::env::var("E2E_TEST_REPO_GITCODE").ok(),
        })
    }

    /// 从环境变量加载配置(宽松版:`E2E_TEST_REPO` 可缺省)
    ///
    /// 用于不依赖测试仓库的实测(如 `auth status`),fork PR 中
    /// secrets 为空时也能构造配置。
    #[must_use]
    pub fn from_env_lenient() -> Self {
        Self {
            test_repo: std::env::var("E2E_TEST_REPO").unwrap_or_default(),
            github_token: std::env::var("E2E_GITHUB_TOKEN").ok(),
            gitcode_token: std::env::var("E2E_GITCODE_TOKEN").ok(),
            gitlab_token: std::env::var("E2E_GITLAB_TOKEN").ok(),
            gitlab_test_repo: std::env::var("E2E_TEST_REPO_GITLAB").ok(),
            gitcode_test_repo: std::env::var("E2E_TEST_REPO_GITCODE").ok(),
        }
    }

    /// 派生测试模式:有 GitHub 令牌即 `Authenticated`
    #[must_use]
    pub fn mode(&self) -> TestMode {
        if self.has_github_auth() {
            TestMode::Authenticated
        } else {
            TestMode::Unauthenticated
        }
    }

    /// 派生 GitLab 测试模式:有 GitLab 令牌即 `Authenticated`
    #[must_use]
    pub fn gitlab_mode(&self) -> TestMode {
        if self.has_gitlab_auth() {
            TestMode::Authenticated
        } else {
            TestMode::Unauthenticated
        }
    }

    /// 派生 GitCode 测试模式:有 GitCode 令牌即 `Authenticated`
    #[must_use]
    pub fn gitcode_mode(&self) -> TestMode {
        if self.has_gitcode_auth() {
            TestMode::Authenticated
        } else {
            TestMode::Unauthenticated
        }
    }

    /// 是否具备 GitHub 凭据
    #[must_use]
    pub fn has_github_auth(&self) -> bool {
        self.github_token.is_some()
    }

    /// 是否具备 GitLab 凭据
    #[must_use]
    pub fn has_gitlab_auth(&self) -> bool {
        self.gitlab_token.is_some()
    }

    /// 是否具备 GitCode 凭据
    #[must_use]
    pub fn has_gitcode_auth(&self) -> bool {
        self.gitcode_token.is_some()
    }

    /// 需要注入 `gh` 子进程的环境变量;未认证时为空
    ///
    /// 修复凭据从未传递给底层 `gh` 子进程的问题——调用方应将
    /// 返回值逐个传入 `TtyRunner::env`。
    #[must_use]
    pub fn gh_env(&self) -> Vec<(String, String)> {
        self.github_token.as_ref().map_or_else(Vec::new, |token| {
            vec![("GH_TOKEN".to_string(), token.clone())]
        })
    }

    /// 需要注入 `glab` 子进程的环境变量;未认证时为空
    #[must_use]
    pub fn gl_env(&self) -> Vec<(String, String)> {
        self.gitlab_token.as_ref().map_or_else(Vec::new, |token| {
            vec![("GL_TOKEN".to_string(), token.clone())]
        })
    }

    /// 需要注入 `gc`/`gitcode` 子进程的环境变量;未认证时为空
    #[must_use]
    pub fn gitcode_env(&self) -> Vec<(String, String)> {
        self.gitcode_token
            .as_ref()
            .map_or_else(Vec::new, |token| {
                vec![("GITCODE_TOKEN".to_string(), token.clone())]
            })
    }
}
```

Update `config_with_token()`/`config_without_token()` in the same file's `mod tests` to add
the two new fields (`gitlab_test_repo: None, gitcode_test_repo: None`).

**Also fix a second, easy-to-miss call site**: `crates/e2e-core/src/fixture.rs` has its own
`offline_config()` test helper using the same `TestConfig { .. }` struct-literal syntax
(all fields named, no `..Default::default()`). It will also fail to compile once
`TestConfig` gains two fields. Add `gitlab_test_repo: None, gitcode_test_repo: None` to it
too — grep to make sure no other struct-literal construction of `TestConfig` was missed:

```bash
grep -rn "TestConfig {" crates/e2e-core/src crates/e2e-github crates/e2e-gitlab crates/e2e-gitcode 2>/dev/null
```

(The last two paths don't exist yet at this point in the plan — the grep is here so this
step's instructions are self-contained once those crates exist too; re-run it after Tasks
4-7 as a final check in Task 8's verification if desired.)

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p e2e-core`
Expected: PASS — all `config` tests green, including the 10 new ones from Step 1.

- [ ] **Step 5: Lint**

Run: `cargo clippy -p e2e-core --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: no warnings.

- [ ] **Step 6: Commit**

```bash
git add crates/e2e-core/src/config.rs
git commit -m "feat(e2e-core): add gitlab/gitcode fields and accessors to TestConfig"
```

---

### Task 2: `e2e-core::TtyRunner` — working-directory override

**Files:**
- Modify: `crates/e2e-core/src/tty.rs`

**Interfaces:**
- Consumes: nothing new
- Produces (used by Task 3 wiring and Tasks 6/7's `issue.rs`/`pr.rs`):
  `TtyRunner::dir(&mut self, path: impl Into<PathBuf>) -> &mut Self`

- [ ] **Step 1: Write the failing test**

Add to the `#[cfg(test)] mod tests` block in `crates/e2e-core/src/tty.rs`:

```rust
    #[test]
    fn test_should_override_working_dir_when_dir_is_set() {
        let mut runner = TtyRunner::new(TtyMode::NonInteractive);
        let custom = PathBuf::from("/tmp/e2e-core-dir-test");
        runner.dir(custom.clone());
        assert_eq!(runner.working_dir, custom);
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p e2e-core test_should_override_working_dir_when_dir_is_set`
Expected: FAIL — `no method named \`dir\` found for struct \`TtyRunner\``

- [ ] **Step 3: Implement**

In `impl TtyRunner` in `crates/e2e-core/src/tty.rs`, add (after the existing `env_remove`
method, before `run`):

```rust
    /// 覆盖执行时的工作目录(默认取进程自身 cwd)。
    ///
    /// 用于让测试在指定目录(例如 [`crate::scratch_repo_dir`] 创建的临时仓库)中
    /// 执行 `gf`,绕过 `gf` 仅从 `git remote get-url origin` 解析仓库路径、
    /// 部分子命令无 `--repo` 覆盖的限制。
    pub fn dir(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.working_dir = path.into();
        self
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p e2e-core test_should_override_working_dir_when_dir_is_set`
Expected: PASS

- [ ] **Step 5: Full crate test + lint**

Run: `cargo test -p e2e-core && cargo clippy -p e2e-core --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: all green, no warnings.

- [ ] **Step 6: Commit**

```bash
git add crates/e2e-core/src/tty.rs
git commit -m "feat(e2e-core): add TtyRunner::dir() working-directory override"
```

---

### Task 3: `e2e-core::scratch_repo_dir` — scratch git checkout helper

**Files:**
- Create: `crates/e2e-core/src/scratch.rs`
- Modify: `crates/e2e-core/src/lib.rs` (register module + re-export)
- Modify: `crates/e2e-core/src/fixture.rs` (extend `FixtureError` with `Io`/`Git` variants)
- Modify: `crates/e2e-core/Cargo.toml` (add `tempfile` dependency)

**Interfaces:**
- Consumes: `crate::fixture::FixtureError` (extended in this task)
- Produces (used by Tasks 6/7's `issue.rs`/`pr.rs`):
  `pub async fn scratch_repo_dir(remote_url: &str) -> Result<tempfile::TempDir, FixtureError>`

- [ ] **Step 1: Add the `tempfile` dependency**

`tempfile = "3"` is already a `[workspace.dependencies]` entry (`Cargo.toml:33`), so this is
a version-consistent addition, not a new external dependency to vet:

```bash
cargo add tempfile -p e2e-core
```

Verify `crates/e2e-core/Cargo.toml` now has `tempfile = "3"` under `[dependencies]` (not
`[dev-dependencies]` — `scratch_repo_dir` is a `pub fn` consumed by `e2e-gitlab`/
`e2e-gitcode` as a *runtime* dependency of their test binaries, not merely e2e-core's own
internal test code).

- [ ] **Step 2: Extend `FixtureError` (write the failing test first)**

Add to the `#[cfg(test)] mod tests` block in `crates/e2e-core/src/fixture.rs`:

```rust
    #[test]
    fn test_should_wrap_io_error_as_fixture_error() {
        let io_err = std::io::Error::other("boom");
        let err: FixtureError = io_err.into();
        assert!(matches!(err, FixtureError::Io(_)));
    }

    #[test]
    fn test_should_format_git_error_message() {
        let err = FixtureError::Git("git init failed: fatal error".to_string());
        assert_eq!(
            err.to_string(),
            "git command failed: git init failed: fatal error"
        );
    }
```

Run: `cargo test -p e2e-core test_should_wrap_io_error_as_fixture_error`
Expected: FAIL — `the trait \`From<std::io::Error>\` is not implemented for \`FixtureError\``

- [ ] **Step 3: Implement the `FixtureError` extension**

In `crates/e2e-core/src/fixture.rs`, replace the `FixtureError` enum with:

```rust
/// 固件错误
#[derive(Debug, Error)]
pub enum FixtureError {
    /// TTY error
    #[error("TTY error: {0}")]
    Tty(#[from] crate::tty::TtyError),
    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    /// Config error
    #[error("Config error: {0}")]
    Config(#[from] crate::config::ConfigError),
    /// IO error(如临时目录创建失败)
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// git 命令执行失败(非零退出码)
    #[error("git command failed: {0}")]
    Git(String),
}
```

Run: `cargo test -p e2e-core` — Expected: PASS (both new tests + all pre-existing).

- [ ] **Step 4: Write the failing tests for `scratch_repo_dir`**

Create `crates/e2e-core/src/scratch.rs`:

```rust
//! 临时 git 仓库构造模块
//!
//! 为需要"remote 指向真实目标仓库"的实测(GitLab/GitCode 的 issue/pr 场景)
//! 提供一次性的临时 git 检出目录。

use crate::fixture::FixtureError;

/// 创建一个临时 git 仓库,`origin` 指向 `remote_url`。
///
/// 用于让 [`crate::TtyRunner`] 在一个"remote 指向目标平台仓库"的工作目录中执行
/// `gf`,绕过 `gf` 仅从 `git remote get-url origin` 解析仓库路径、`list` 类命令
/// 无 `--repo` 覆盖的限制(详见设计文档
/// `docs/superpowers/specs/2026-09-03-e2e-gitlab-gitcode-coverage-design.md`)。
///
/// 返回的 [`tempfile::TempDir`] 在析构时自动清理临时目录。
///
/// # Errors
///
/// 当临时目录创建失败,或 `git init`/`git remote add` 命令执行失败
/// (非零退出码)时返回 `FixtureError::Io` 或 `FixtureError::Git`。
pub async fn scratch_repo_dir(remote_url: &str) -> Result<tempfile::TempDir, FixtureError> {
    let dir = tempfile::tempdir()?;

    let init = tokio::process::Command::new("git")
        .args(["init", "--quiet"])
        .current_dir(dir.path())
        .output()
        .await?;
    if !init.status.success() {
        return Err(FixtureError::Git(format!(
            "git init failed: {}",
            String::from_utf8_lossy(&init.stderr)
        )));
    }

    let remote = tokio::process::Command::new("git")
        .args(["remote", "add", "origin", remote_url])
        .current_dir(dir.path())
        .output()
        .await?;
    if !remote.status.success() {
        return Err(FixtureError::Git(format!(
            "git remote add failed: {}",
            String::from_utf8_lossy(&remote.stderr)
        )));
    }

    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_should_create_scratch_repo_with_origin_remote() {
        let dir = scratch_repo_dir("https://gitlab.com/example/project.git")
            .await
            .expect("scratch repo creation must succeed");

        let output = tokio::process::Command::new("git")
            .args(["remote", "get-url", "origin"])
            .current_dir(dir.path())
            .output()
            .await
            .expect("git remote get-url must run");
        assert!(output.status.success());
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            "https://gitlab.com/example/project.git"
        );
    }

    #[tokio::test]
    async fn test_should_fail_when_remote_url_looks_like_a_flag() {
        // Empty strings are silently accepted by `git remote add` (verified locally:
        // `git remote add origin ""` exits 0 and stores an empty URL) — not a usable
        // failure case. A leading-`--` string, however, git parses as an unknown
        // option and rejects with a non-zero exit, which is what we need to exercise
        // the `FixtureError::Git` branch.
        let result = scratch_repo_dir("--bogus-flag").await;
        assert!(matches!(result, Err(FixtureError::Git(_))));
    }
}
```

- [ ] **Step 5: Register the module**

In `crates/e2e-core/src/lib.rs`, change:

```rust
pub mod config;
pub mod fixture;
pub mod tty;

pub use config::{TestConfig, TestMode};
pub use fixture::{TestFixture, TestResource};
pub use tty::{CommandOutput, TtyError, TtyMode, TtyRunner};
```

to:

```rust
pub mod config;
pub mod fixture;
pub mod scratch;
pub mod tty;

pub use config::{TestConfig, TestMode};
pub use fixture::{TestFixture, TestResource};
pub use scratch::scratch_repo_dir;
pub use tty::{CommandOutput, TtyError, TtyMode, TtyRunner};
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test -p e2e-core`
Expected: PASS — including the two new `scratch` tests. (These two tests only invoke local
`git`, no network access, so they run in any environment including CI.)

- [ ] **Step 7: Lint**

Run: `cargo clippy -p e2e-core --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: no warnings.

- [ ] **Step 8: Commit**

```bash
git add crates/e2e-core/src/scratch.rs crates/e2e-core/src/lib.rs crates/e2e-core/src/fixture.rs crates/e2e-core/Cargo.toml Cargo.lock
git commit -m "feat(e2e-core): add scratch_repo_dir() for remote-pointed test checkouts"
```

---

### Task 4: `crates/e2e-gitlab` scaffold + `auth.rs` + `noauth.rs`

**Files:**
- Create: `crates/e2e-gitlab/Cargo.toml`
- Create: `crates/e2e-gitlab/tests/auth.rs`
- Create: `crates/e2e-gitlab/tests/noauth.rs`

**Interfaces:**
- Consumes: `e2e_core::{TestConfig, TestMode, TtyMode, TtyRunner}` (from Task 1/2)
- Produces: nothing consumed by later tasks (test binaries are leaves)

- [ ] **Step 1: Scaffold the crate**

The workspace's `members = ["crates/*", "apps/*"]` (`Cargo.toml:2`) is a glob — no explicit
member-list edit is needed once a valid `Cargo.toml` exists under `crates/e2e-gitlab/`.

Create `crates/e2e-gitlab/Cargo.toml`:

```toml
[package]
name = "e2e-gitlab"
version.workspace = true
edition.workspace = true
publish = false
release = false
license = "MIT"

[dev-dependencies]
e2e-core = { path = "../e2e-core" }
serde_json = "1.0"
tokio = { version = "1", features = ["full", "test-util"] }

[lints]
workspace = true
```

Run: `cargo metadata --no-deps --format-version 1 | jq '.packages[].name'` — confirm
`"e2e-gitlab"` appears in the list (proves the workspace picked up the new member).

- [ ] **Step 2: Write `auth.rs` (RED — will fail to compile until Step 1's crate exists, which it now does; the test itself passes immediately since it self-skips without credentials)**

Create `crates/e2e-gitlab/tests/auth.rs`:

```rust
//! GitLab auth 命令 E2E 实测(真实凭据,严格断言)
//!
//! 无 `E2E_GITLAB_TOKEN` 时自动 skip。

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "Test code uses unwrap/expect/indexing for simplicity"
)]

use e2e_core::{TestConfig, TestMode, TtyMode, TtyRunner};

#[tokio::test]
async fn test_should_report_logged_in_with_real_credentials() {
    let config = TestConfig::from_env_lenient();
    if config.gitlab_mode() != TestMode::Authenticated {
        eprintln!("skipped: E2E_GITLAB_TOKEN not set");
        return;
    }

    for tty_mode in [TtyMode::Interactive, TtyMode::NonInteractive] {
        let mut runner = TtyRunner::new(tty_mode);
        for (key, value) in config.gl_env() {
            runner.env(key, value);
        }

        let output = runner
            .run(&["auth", "status", "--platform", "gitlab", "--output", "json"])
            .await
            .unwrap();

        assert!(
            output.status.success(),
            "mode {tty_mode:?}: stderr: {}",
            output.stderr
        );
        let parsed: serde_json::Value =
            serde_json::from_str(&output.stdout).expect("stdout must be a JSON envelope");
        assert_eq!(
            parsed["success"],
            serde_json::json!(true),
            "mode {tty_mode:?}: stdout: {}",
            output.stdout
        );
        assert_eq!(
            parsed["data"]["loggedIn"],
            serde_json::json!(true),
            "mode {tty_mode:?}: expected logged-in, stdout: {}",
            output.stdout
        );
    }
}
```

- [ ] **Step 3: Run to verify it passes in the current (unauthenticated) environment**

Run: `cargo test -p e2e-gitlab --test auth`
Expected: PASS, with `skipped: E2E_GITLAB_TOKEN not set` printed to stderr (visible with
`-- --nocapture`).

- [ ] **Step 4: Write `noauth.rs`**

Create `crates/e2e-gitlab/tests/noauth.rs`:

```rust
//! 未认证错误路径 E2E 测试(无需凭据,前提是运行环境已安装 `glab` CLI)
//!
//! 通过 `env_remove` 清除继承的 `GL_TOKEN`,构造确定性的未认证环境。
//! `GitLabAuthProvider` 是纯 env-var 短路 + 真实 `glab` 子进程读取
//! (见 `crates/gitlab/src/auth.rs`),没有 `gh` 那种本地 `hosts.yml` 状态,
//! 因此 `env_remove` 单独即可保证确定性,无需额外的空目录隔离。

#![allow(clippy::unwrap_used, reason = "Test code uses unwrap for simplicity")]

use e2e_core::{TtyMode, TtyRunner};

fn scrubbed_runner() -> TtyRunner {
    let mut runner = TtyRunner::new(TtyMode::NonInteractive);
    runner.env_remove("GL_TOKEN");
    runner
}

#[tokio::test]
async fn test_should_fail_with_login_guidance_when_status_checked_unauthenticated() {
    let runner = scrubbed_runner();
    let output = runner
        .run(&["auth", "status", "--platform", "gitlab", "--output", "json"])
        .await
        .unwrap();

    assert!(
        !output.status.success(),
        "unauthenticated auth status must exit non-zero, stdout: {}, stderr: {}",
        output.stdout,
        output.stderr
    );
    let combined = format!("{}{}", output.stdout, output.stderr).to_lowercase();
    assert!(
        combined.contains("auth login") || combined.contains("login"),
        "expected login guidance in output, got: {combined}"
    );
}

#[tokio::test]
async fn test_should_fail_with_login_guidance_when_listing_issues_unauthenticated() {
    let runner = scrubbed_runner();
    let output = runner
        .run(&["issue", "list", "--platform", "gitlab", "--output", "json"])
        .await
        .unwrap();

    assert!(
        !output.status.success(),
        "unauthenticated issue list must exit non-zero, stdout: {}, stderr: {}",
        output.stdout,
        output.stderr
    );
    let combined = format!("{}{}", output.stdout, output.stderr).to_lowercase();
    assert!(
        combined.contains("auth login") || combined.contains("login"),
        "expected login guidance in output, got: {combined}"
    );
}
```

- [ ] **Step 5: Run to verify current behavior**

Run: `cargo test -p e2e-gitlab --test noauth`
Expected (local machine without `glab` installed): **FAIL** — the CLI reports
`Failed to spawn glab auth status: No such file or directory`, which is a non-zero exit
(first assertion passes) but the combined output won't contain "login" (second assertion
fails). This is the expected RED state locally — it turns GREEN once `glab` is installed
(Task 8 installs it in CI; a local run requires installing `glab` manually, noted in the
design doc's Open Questions). **Do not weaken the assertion to hide this** — install `glab`
locally to verify GREEN before moving on, per repo TDD discipline:

```bash
go install gitlab.com/gitlab-org/cli/cmd/glab@latest
export PATH="$(go env GOPATH)/bin:$PATH"
cargo test -p e2e-gitlab --test noauth
```

Expected after installing `glab`: PASS.

- [ ] **Step 6: Lint**

Run: `cargo clippy -p e2e-gitlab --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: no warnings.

- [ ] **Step 7: Commit**

```bash
git add crates/e2e-gitlab
git commit -m "feat(e2e-gitlab): add crate scaffold with auth/noauth E2E tests"
```

---

### Task 5: `crates/e2e-gitlab` — `issue.rs` + `pr.rs`

**Files:**
- Create: `crates/e2e-gitlab/tests/issue.rs`
- Create: `crates/e2e-gitlab/tests/pr.rs`

**Interfaces:**
- Consumes: `e2e_core::{TestConfig, TestMode, TtyMode, TtyRunner, scratch_repo_dir}` (Tasks 1-3)

- [ ] **Step 1: Write `issue.rs`**

Create `crates/e2e-gitlab/tests/issue.rs`:

```rust
//! GitLab issue 命令 E2E 实测(真实凭据 + 真实仓库,严格 schema 断言)
//!
//! 无 `E2E_GITLAB_TOKEN` 或 `E2E_TEST_REPO_GITLAB` 时自动 skip(真实测试仓库/凭据
//! 基础设施留待后续 Issue 配置)。

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "Test code uses unwrap/expect/indexing for simplicity"
)]

use e2e_core::{TestConfig, TestMode, TtyMode, TtyRunner, scratch_repo_dir};

#[tokio::test]
async fn test_should_list_open_issues_with_valid_schema() {
    let config = TestConfig::from_env_lenient();
    let Some(repo) = config.gitlab_test_repo.clone() else {
        eprintln!("skipped: E2E_TEST_REPO_GITLAB not set");
        return;
    };
    if config.gitlab_mode() != TestMode::Authenticated {
        eprintln!("skipped: E2E_GITLAB_TOKEN not set");
        return;
    }

    let scratch = scratch_repo_dir(&format!("https://gitlab.com/{repo}.git"))
        .await
        .expect("scratch repo setup must succeed");

    let mut runner = TtyRunner::new(TtyMode::NonInteractive);
    runner.dir(scratch.path().to_path_buf());
    for (key, value) in config.gl_env() {
        runner.env(key, value);
    }

    let output = runner
        .run(&[
            "issue", "list", "--platform", "gitlab", "--state", "open", "--output", "json",
        ])
        .await
        .unwrap();

    assert!(output.status.success(), "stderr: {}", output.stderr);
    let parsed: serde_json::Value =
        serde_json::from_str(&output.stdout).expect("stdout must be a JSON envelope");
    assert_eq!(
        parsed["success"],
        serde_json::json!(true),
        "stdout: {}",
        output.stdout
    );

    let items = parsed["data"]
        .as_array()
        .expect("data must be an array of issues");
    for item in items {
        assert!(
            item["number"].as_u64().is_some(),
            "number must be an unsigned integer: {item}"
        );
        assert!(
            item["title"].as_str().is_some_and(|t| !t.is_empty()),
            "title must be a non-empty string: {item}"
        );
    }
}
```

- [ ] **Step 2: Write `pr.rs`**

Create `crates/e2e-gitlab/tests/pr.rs`:

```rust
//! GitLab pr(mr)命令 E2E 实测(真实凭据 + 真实仓库,严格 schema 断言)
//!
//! 无 `E2E_GITLAB_TOKEN` 或 `E2E_TEST_REPO_GITLAB` 时自动 skip。
//!
//! 与 `e2e-github` 的差异:`e2e-github` 断言 `closed` 列表非空(利用本仓库自身
//! 已有已合并 PR 的确定性)。GitLab 测试仓库在基础设施到位前身份未知(可能是全新
//! 空仓库),因此本测试放宽为"若 `items` 非空则逐项校验 schema",不强制非空。

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "Test code uses unwrap/expect/indexing for simplicity"
)]

use e2e_core::{TestConfig, TestMode, TtyMode, TtyRunner, scratch_repo_dir};

#[tokio::test]
async fn test_should_list_closed_prs_with_valid_schema() {
    let config = TestConfig::from_env_lenient();
    let Some(repo) = config.gitlab_test_repo.clone() else {
        eprintln!("skipped: E2E_TEST_REPO_GITLAB not set");
        return;
    };
    if config.gitlab_mode() != TestMode::Authenticated {
        eprintln!("skipped: E2E_GITLAB_TOKEN not set");
        return;
    }

    let scratch = scratch_repo_dir(&format!("https://gitlab.com/{repo}.git"))
        .await
        .expect("scratch repo setup must succeed");

    let mut runner = TtyRunner::new(TtyMode::NonInteractive);
    runner.dir(scratch.path().to_path_buf());
    for (key, value) in config.gl_env() {
        runner.env(key, value);
    }

    let output = runner
        .run(&[
            "pr", "list", "--platform", "gitlab", "--state", "closed", "--output", "json",
        ])
        .await
        .unwrap();

    assert!(output.status.success(), "stderr: {}", output.stderr);
    let parsed: serde_json::Value =
        serde_json::from_str(&output.stdout).expect("stdout must be a JSON envelope");
    assert_eq!(
        parsed["success"],
        serde_json::json!(true),
        "stdout: {}",
        output.stdout
    );

    let items = parsed["data"]
        .as_array()
        .expect("data must be an array of pull/merge requests");
    for item in items {
        assert!(
            item["number"].as_u64().is_some(),
            "number must be an unsigned integer: {item}"
        );
        assert!(
            item["title"].as_str().is_some_and(|t| !t.is_empty()),
            "title must be a non-empty string: {item}"
        );
    }
}
```

- [ ] **Step 3: Run to verify both pass in the current (no test-repo) environment**

Run: `cargo test -p e2e-gitlab`
Expected: PASS — `issue`/`pr` print `skipped: E2E_TEST_REPO_GITLAB not set`; `auth` prints
`skipped: E2E_GITLAB_TOKEN not set`; `noauth` passes for real (once `glab` is installed per
Task 4 Step 5).

- [ ] **Step 4: Lint**

Run: `cargo clippy -p e2e-gitlab --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: no warnings.

- [ ] **Step 5: Commit**

```bash
git add crates/e2e-gitlab/tests/issue.rs crates/e2e-gitlab/tests/pr.rs
git commit -m "feat(e2e-gitlab): add issue/pr E2E tests via scratch_repo_dir"
```

---

### Task 6: `crates/e2e-gitcode` scaffold + `auth.rs` + `noauth.rs`

**Files:**
- Create: `crates/e2e-gitcode/Cargo.toml`
- Create: `crates/e2e-gitcode/tests/auth.rs`
- Create: `crates/e2e-gitcode/tests/noauth.rs`

**Interfaces:**
- Consumes: `e2e_core::{TestConfig, TestMode, TtyMode, TtyRunner}` (from Task 1/2)
- Produces: nothing consumed by later tasks

Mirror Task 4 exactly, substituting GitCode identifiers throughout (`gitcode_mode()`,
`gitcode_env()`, `--platform gitcode`, `E2E_GITCODE_TOKEN`, `GITCODE_TOKEN`).

- [ ] **Step 1: Scaffold the crate**

Create `crates/e2e-gitcode/Cargo.toml`:

```toml
[package]
name = "e2e-gitcode"
version.workspace = true
edition.workspace = true
publish = false
release = false
license = "MIT"

[dev-dependencies]
e2e-core = { path = "../e2e-core" }
serde_json = "1.0"
tokio = { version = "1", features = ["full", "test-util"] }

[lints]
workspace = true
```

- [ ] **Step 2: Write `auth.rs`**

Create `crates/e2e-gitcode/tests/auth.rs`:

```rust
//! GitCode auth 命令 E2E 实测(真实凭据,严格断言)
//!
//! 无 `E2E_GITCODE_TOKEN` 时自动 skip。

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "Test code uses unwrap/expect/indexing for simplicity"
)]

use e2e_core::{TestConfig, TestMode, TtyMode, TtyRunner};

#[tokio::test]
async fn test_should_report_logged_in_with_real_credentials() {
    let config = TestConfig::from_env_lenient();
    if config.gitcode_mode() != TestMode::Authenticated {
        eprintln!("skipped: E2E_GITCODE_TOKEN not set");
        return;
    }

    for tty_mode in [TtyMode::Interactive, TtyMode::NonInteractive] {
        let mut runner = TtyRunner::new(tty_mode);
        for (key, value) in config.gitcode_env() {
            runner.env(key, value);
        }

        let output = runner
            .run(&["auth", "status", "--platform", "gitcode", "--output", "json"])
            .await
            .unwrap();

        assert!(
            output.status.success(),
            "mode {tty_mode:?}: stderr: {}",
            output.stderr
        );
        let parsed: serde_json::Value =
            serde_json::from_str(&output.stdout).expect("stdout must be a JSON envelope");
        assert_eq!(
            parsed["success"],
            serde_json::json!(true),
            "mode {tty_mode:?}: stdout: {}",
            output.stdout
        );
        assert_eq!(
            parsed["data"]["loggedIn"],
            serde_json::json!(true),
            "mode {tty_mode:?}: expected logged-in, stdout: {}",
            output.stdout
        );
    }
}
```

- [ ] **Step 3: Write `noauth.rs`**

Create `crates/e2e-gitcode/tests/noauth.rs`:

```rust
//! 未认证错误路径 E2E 测试(无需凭据,前提是运行环境已安装 `gc`/`gitcode` CLI)
//!
//! 通过 `env_remove` 清除继承的 `GITCODE_TOKEN`,构造确定性的未认证环境。
//! `GitCodeAuthProvider` 是纯 env-var 短路 + 真实 `gc` 子进程读取
//! (见 `crates/gitcode/src/auth.rs`),没有本地配置文件状态,`env_remove` 单独
//! 即可保证确定性。

#![allow(clippy::unwrap_used, reason = "Test code uses unwrap for simplicity")]

use e2e_core::{TtyMode, TtyRunner};

fn scrubbed_runner() -> TtyRunner {
    let mut runner = TtyRunner::new(TtyMode::NonInteractive);
    runner.env_remove("GITCODE_TOKEN");
    runner
}

#[tokio::test]
async fn test_should_fail_with_login_guidance_when_status_checked_unauthenticated() {
    let runner = scrubbed_runner();
    let output = runner
        .run(&["auth", "status", "--platform", "gitcode", "--output", "json"])
        .await
        .unwrap();

    assert!(
        !output.status.success(),
        "unauthenticated auth status must exit non-zero, stdout: {}, stderr: {}",
        output.stdout,
        output.stderr
    );
    let combined = format!("{}{}", output.stdout, output.stderr).to_lowercase();
    assert!(
        combined.contains("auth login") || combined.contains("login"),
        "expected login guidance in output, got: {combined}"
    );
}

#[tokio::test]
async fn test_should_fail_with_login_guidance_when_listing_issues_unauthenticated() {
    let runner = scrubbed_runner();
    let output = runner
        .run(&["issue", "list", "--platform", "gitcode", "--output", "json"])
        .await
        .unwrap();

    assert!(
        !output.status.success(),
        "unauthenticated issue list must exit non-zero, stdout: {}, stderr: {}",
        output.stdout,
        output.stderr
    );
    let combined = format!("{}{}", output.stdout, output.stderr).to_lowercase();
    assert!(
        combined.contains("auth login") || combined.contains("login"),
        "expected login guidance in output, got: {combined}"
    );
}
```

- [ ] **Step 4: Run to verify current behavior, install `gc` locally to reach GREEN**

Run: `cargo test -p e2e-gitcode --test auth` — Expected: PASS (skips, no token).

Run: `cargo test -p e2e-gitcode --test noauth` — Expected RED locally without `gc` installed
(same "binary not found" mismatch as Task 4 Step 5). Install and re-verify:

```bash
pip install gitcode-cli
cargo test -p e2e-gitcode --test noauth
```

Expected after installing: PASS.

- [ ] **Step 5: Lint**

Run: `cargo clippy -p e2e-gitcode --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: no warnings.

- [ ] **Step 6: Commit**

```bash
git add crates/e2e-gitcode/Cargo.toml crates/e2e-gitcode/tests/auth.rs crates/e2e-gitcode/tests/noauth.rs
git commit -m "feat(e2e-gitcode): add crate scaffold with auth/noauth E2E tests"
```

---

### Task 7: `crates/e2e-gitcode` — `issue.rs` + `pr.rs`

**Files:**
- Create: `crates/e2e-gitcode/tests/issue.rs`
- Create: `crates/e2e-gitcode/tests/pr.rs`

**Interfaces:**
- Consumes: `e2e_core::{TestConfig, TestMode, TtyMode, TtyRunner, scratch_repo_dir}` (Tasks 1-3)

- [ ] **Step 1: Write `issue.rs`**

Create `crates/e2e-gitcode/tests/issue.rs`:

```rust
//! GitCode issue 命令 E2E 实测(真实凭据 + 真实仓库,严格 schema 断言)
//!
//! 无 `E2E_GITCODE_TOKEN` 或 `E2E_TEST_REPO_GITCODE` 时自动 skip(真实测试仓库/凭据
//! 基础设施留待后续 Issue 配置)。

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "Test code uses unwrap/expect/indexing for simplicity"
)]

use e2e_core::{TestConfig, TestMode, TtyMode, TtyRunner, scratch_repo_dir};

#[tokio::test]
async fn test_should_list_open_issues_with_valid_schema() {
    let config = TestConfig::from_env_lenient();
    let Some(repo) = config.gitcode_test_repo.clone() else {
        eprintln!("skipped: E2E_TEST_REPO_GITCODE not set");
        return;
    };
    if config.gitcode_mode() != TestMode::Authenticated {
        eprintln!("skipped: E2E_GITCODE_TOKEN not set");
        return;
    }

    let scratch = scratch_repo_dir(&format!("https://gitcode.com/{repo}.git"))
        .await
        .expect("scratch repo setup must succeed");

    let mut runner = TtyRunner::new(TtyMode::NonInteractive);
    runner.dir(scratch.path().to_path_buf());
    for (key, value) in config.gitcode_env() {
        runner.env(key, value);
    }

    let output = runner
        .run(&[
            "issue", "list", "--platform", "gitcode", "--state", "open", "--output", "json",
        ])
        .await
        .unwrap();

    assert!(output.status.success(), "stderr: {}", output.stderr);
    let parsed: serde_json::Value =
        serde_json::from_str(&output.stdout).expect("stdout must be a JSON envelope");
    assert_eq!(
        parsed["success"],
        serde_json::json!(true),
        "stdout: {}",
        output.stdout
    );

    let items = parsed["data"]
        .as_array()
        .expect("data must be an array of issues");
    for item in items {
        assert!(
            item["number"].as_u64().is_some(),
            "number must be an unsigned integer: {item}"
        );
        assert!(
            item["title"].as_str().is_some_and(|t| !t.is_empty()),
            "title must be a non-empty string: {item}"
        );
    }
}
```

- [ ] **Step 2: Write `pr.rs`**

Create `crates/e2e-gitcode/tests/pr.rs`:

```rust
//! GitCode pr 命令 E2E 实测(真实凭据 + 真实仓库,严格 schema 断言)
//!
//! 无 `E2E_GITCODE_TOKEN` 或 `E2E_TEST_REPO_GITCODE` 时自动 skip。
//!
//! 与 `e2e-github` 的差异:见 `crates/e2e-gitlab/tests/pr.rs` 顶部说明——测试仓库
//! 身份未知,非空断言放宽为"若非空则校验 schema"。

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "Test code uses unwrap/expect/indexing for simplicity"
)]

use e2e_core::{TestConfig, TestMode, TtyMode, TtyRunner, scratch_repo_dir};

#[tokio::test]
async fn test_should_list_closed_prs_with_valid_schema() {
    let config = TestConfig::from_env_lenient();
    let Some(repo) = config.gitcode_test_repo.clone() else {
        eprintln!("skipped: E2E_TEST_REPO_GITCODE not set");
        return;
    };
    if config.gitcode_mode() != TestMode::Authenticated {
        eprintln!("skipped: E2E_GITCODE_TOKEN not set");
        return;
    }

    let scratch = scratch_repo_dir(&format!("https://gitcode.com/{repo}.git"))
        .await
        .expect("scratch repo setup must succeed");

    let mut runner = TtyRunner::new(TtyMode::NonInteractive);
    runner.dir(scratch.path().to_path_buf());
    for (key, value) in config.gitcode_env() {
        runner.env(key, value);
    }

    let output = runner
        .run(&[
            "pr", "list", "--platform", "gitcode", "--state", "closed", "--output", "json",
        ])
        .await
        .unwrap();

    assert!(output.status.success(), "stderr: {}", output.stderr);
    let parsed: serde_json::Value =
        serde_json::from_str(&output.stdout).expect("stdout must be a JSON envelope");
    assert_eq!(
        parsed["success"],
        serde_json::json!(true),
        "stdout: {}",
        output.stdout
    );

    let items = parsed["data"]
        .as_array()
        .expect("data must be an array of pull requests");
    for item in items {
        assert!(
            item["number"].as_u64().is_some(),
            "number must be an unsigned integer: {item}"
        );
        assert!(
            item["title"].as_str().is_some_and(|t| !t.is_empty()),
            "title must be a non-empty string: {item}"
        );
    }
}
```

- [ ] **Step 3: Run full workspace test suite**

Run: `cargo test -p e2e-core -p e2e-github -p e2e-gitlab -p e2e-gitcode`
Expected: PASS across all four crates (GitHub path unaffected; GitLab/GitCode `issue`/`pr`/
`auth` skip, `noauth` passes given `glab`/`gc` installed per Tasks 4/6).

- [ ] **Step 4: Lint**

Run: `cargo clippy -p e2e-gitcode --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: no warnings.

- [ ] **Step 5: Commit**

```bash
git add crates/e2e-gitcode/tests/issue.rs crates/e2e-gitcode/tests/pr.rs
git commit -m "feat(e2e-gitcode): add issue/pr E2E tests via scratch_repo_dir"
```

---

### Task 8: CI — `e2e-gitlab`/`e2e-gitcode` jobs in `e2e-tests.yml`

**Files:**
- Modify: `.github/workflows/e2e-tests.yml`

**Interfaces:**
- Consumes: crates from Tasks 4-7 (`-p e2e-gitlab`, `-p e2e-gitcode` must build)
- Produces: two new CI jobs; nothing consumed by later tasks

- [ ] **Step 1: Add the `e2e-gitlab` job**

In `.github/workflows/e2e-tests.yml`, after the closing of the existing `e2e-github:` job
block (after its `Upload test results` step, still inside `jobs:`), insert:

```yaml
  e2e-gitlab:
    name: E2E Tests (GitLab)
    runs-on: ubuntu-latest
    timeout-minutes: 30

    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Install cargo-nextest
        uses: taiki-e/install-action@cargo-nextest

      - name: Install glab CLI
        run: |
          go install gitlab.com/gitlab-org/cli/cmd/glab@latest
          echo "$(go env GOPATH)/bin" >> "$GITHUB_PATH"

      - name: Build release binary
        run: cargo build --release

      - name: Add to PATH
        run: echo "${{ github.workspace }}/target/release" >> $GITHUB_PATH

      # 模式判定由测试层承担:无凭据/无测试仓库时实测自动 skip,
      # 无凭据错误路径(noauth)不受影响,正常运行(依赖上一步安装的 glab)。
      - name: Run E2E tests
        env:
          E2E_GITLAB_TOKEN: ${{ secrets.E2E_GITLAB_TOKEN }}
          E2E_TEST_REPO_GITLAB: ${{ secrets.E2E_TEST_REPO_GITLAB }}
          RUST_LOG: info
        run: |
          cargo nextest run -p e2e-core -p e2e-gitlab --all-features

      - name: Report run mode
        if: always()
        env:
          E2E_GITLAB_TOKEN: ${{ secrets.E2E_GITLAB_TOKEN }}
        run: |
          if [ -n "$E2E_GITLAB_TOKEN" ]; then
            echo "E2E mode: authenticated (real-credential scenarios executed)" >> "$GITHUB_STEP_SUMMARY"
          else
            echo "E2E mode: unauthenticated (error paths + harness self-tests only)" >> "$GITHUB_STEP_SUMMARY"
          fi

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: e2e-results-gitlab
          path: target/nextest/
          retention-days: 7
```

- [ ] **Step 2: Add the `e2e-gitcode` job**

Immediately after the `e2e-gitlab:` block, still inside `jobs:`, insert:

```yaml
  e2e-gitcode:
    name: E2E Tests (GitCode)
    runs-on: ubuntu-latest
    timeout-minutes: 30

    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Install cargo-nextest
        uses: taiki-e/install-action@cargo-nextest

      - name: Install gitcode CLI
        run: pip install gitcode-cli

      - name: Build release binary
        run: cargo build --release

      - name: Add to PATH
        run: echo "${{ github.workspace }}/target/release" >> $GITHUB_PATH

      - name: Run E2E tests
        env:
          E2E_GITCODE_TOKEN: ${{ secrets.E2E_GITCODE_TOKEN }}
          E2E_TEST_REPO_GITCODE: ${{ secrets.E2E_TEST_REPO_GITCODE }}
          RUST_LOG: info
        run: |
          cargo nextest run -p e2e-core -p e2e-gitcode --all-features

      - name: Report run mode
        if: always()
        env:
          E2E_GITCODE_TOKEN: ${{ secrets.E2E_GITCODE_TOKEN }}
        run: |
          if [ -n "$E2E_GITCODE_TOKEN" ]; then
            echo "E2E mode: authenticated (real-credential scenarios executed)" >> "$GITHUB_STEP_SUMMARY"
          else
            echo "E2E mode: unauthenticated (error paths + harness self-tests only)" >> "$GITHUB_STEP_SUMMARY"
          fi

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: e2e-results-gitcode
          path: target/nextest/
          retention-days: 7
```

- [ ] **Step 3: Validate YAML syntax**

Run: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/e2e-tests.yml'))"`
Expected: no exception (proves valid YAML — this repo has no `act`/local GH Actions runner,
so full job execution can only be verified after push; syntax validation is the local
pre-flight check).

- [ ] **Step 4: Commit**

```bash
git add .github/workflows/e2e-tests.yml
git commit -m "ci: add e2e-gitlab and e2e-gitcode jobs to e2e-tests.yml"
```

---

## Post-Plan Verification (run once, after all tasks complete)

- [ ] `cargo build --workspace` — confirms `e2e-gitlab`/`e2e-gitcode` are picked up by the
  workspace glob and compile cleanly alongside everything else.
- [ ] `cargo test -p e2e-core -p e2e-github -p e2e-gitlab -p e2e-gitcode` — full regression
  across all four e2e crates.
- [ ] `cargo +nightly fmt --check` — formatting.
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic`
  — full workspace lint (catches any cross-crate issue individual `-p` runs missed).
- [ ] `make lint` (if it wraps the above; otherwise the explicit commands above suffice).
- [ ] Re-read the design doc's Acceptance Criteria (`docs/superpowers/specs/2026-09-03-e2e-gitlab-gitcode-coverage-design.md`) and Issue #291's checklist — confirm all four boxes are
  satisfiable by what was built (issue/pr real-repo scenarios satisfy the checklist via
  "graceful skip + complete code path", per the confirmed scope note added as an Issue
  comment).
