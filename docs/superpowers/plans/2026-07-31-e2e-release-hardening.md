# e2e 实化 + 发布流水线加固 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让 e2e-tests 流水线不再空转(双层模式真实测试 + 每周定时回归),新增上游 CLI nightly 巡检预警,并加固发布流程杜绝 `v{{version}}` 类模板事故复发。

**Architecture:** 测试层承载模式判定(`TestConfig::mode()` 从凭据派生 Authenticated/Unauthenticated,workflow 保持单 job 无分支);CI 层新增独立 nightly patrol workflow(API 查询上游最新版 + 对比 compatibility-matrix.json + 去重 Issue 预警);发布层在 `release.sh` 中加入纯函数事后校验闸门 + 无条件 dry-run + `--rehearse` 演练模式。

**Tech Stack:** Rust 2024(tokio/serde_json/thiserror)、GitHub Actions、bash(jq/gh/curl)、cargo-release/git-cliff。

## Global Constraints

- Rust 2024 edition,pinned toolchain(`rust-toolchain.toml` 禁改);`#![forbid(unsafe_code)]` 全工作区生效 → 测试中**不得**用 `std::env::set_var/remove_var`(edition 2024 中为 unsafe)
- 生产代码禁 `unwrap()`/`expect()`;测试文件顶部沿用现有模式 `#![allow(clippy::unwrap_used, reason = "Test code uses unwrap for simplicity")]`
- 所有新增 public item 必须有文档注释;纯函数加 `#[must_use]`;可失败函数文档含 `# Errors`
- 通过 `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic` 与 `cargo +nightly fmt -- --check`
- 测试命名 `test_should_<expected_behavior>`;TDD 循环 RED → GREEN → REFACTOR;每个任务末尾独立提交(conventional commits)
- 禁改:`deny.toml` 策略、`.pre-commit-config.yaml`、`rust-toolchain.toml`
- 本地运行 e2e 测试的前置:`cargo build --release && export PATH="$PWD/target/release:$PATH"`(二进制依赖型测试未构建时自动 skip,不 fail)

## ⚠️ Deviations from Design Doc(相对 `2026-07-31-e2e-release-hardening-design.md`)

计划制定期间验证 CLI 实际接口,发现设计文档两处假设与代码不符,本计划已修正:

1. **`issue list` / `pr list` 没有 `--repo` flag**(仅 `--platform`/`--state`/`--output`/`--limit`,作用于当前仓库 remote)。设计 §4.2 的 `--repo $E2E_TEST_REPO` 机制不可行 → **实测测试对当前仓库运行**(CI checkout 的 gitflow-cli 仓库 / 开发者本地 clone),断言升级为 **JSON schema 严格校验**(实测捕获的形状:`{success, data: [{number, title, state, ...}], platform, command}`)。e2e-test-repo fixture seed 机制废弃;`E2E_TEST_REPO` 从 workflow env 移除(secret 保留不引用)。
2. **e2e-core 需小幅 API 扩展**支撑 hermetic 测试:新增 `TestConfig::from_env_lenient()`、`TtyRunner::env_remove()`、`TestFixture::with_config()`、lib.rs 补充导出 `TestMode`/`TtyError`。设计 §4.1 的"fixture 生命周期(依赖 env 操控)"改为 `with_config` + 空清理测试(Rust 2024 forbid unsafe 无法在测试内操控环境变量)。

另:实测确认未认证时 `auth status` 与 `issue list` 均 **exit 1**,输出 miette 诊断(含 `gh auth login` 引导);并发现两个潜在 bug(Phase 4 triage):① `issue comment` 返回 stale comment id;② 未认证诊断中 `[[PLATFORM]]` 占位符未替换。
3. **设计 §7.3 的"逐项 y/N 确认清单"→ 改为自动闸门 + 演练报告**:preflight 各项(分支/工作区/测试/clippy)本就应是自动强制的检查,交互式 y/N 只是仪式;本计划中失败即中止(`preflight_checks` 现状)+ `--rehearse` 输出 ✅ 清单报告,强制力高于逐项确认。

## File Structure

| 文件 | 动作 | 职责 |
|------|------|------|
| `crates/e2e-core/src/config.rs` | Modify | `TestMode` 枚举 + `mode()`/`has_github_auth()`/`gh_env()`/`from_env_lenient()` + 单元测试 |
| `crates/e2e-core/src/tty.rs` | Modify | `env_remove()` + 单元测试 |
| `crates/e2e-core/src/fixture.rs` | Modify | `with_config()` 构造器 + 单元测试 |
| `crates/e2e-core/src/lib.rs` | Modify | 补充导出 `TestMode`、`TtyError` |
| `crates/e2e-core/tests/harness.rs` | Create | harness 自测:二进制发现、错误传播 |
| `crates/e2e-github/Cargo.toml` | Modify | dev-deps 增加 `serde_json` |
| `crates/e2e-github/tests/auth.rs` | Rewrite | 严格实测:loggedIn==true(凭据门控) |
| `crates/e2e-github/tests/issue.rs` | Rewrite | 严格实测:issue list JSON schema |
| `crates/e2e-github/tests/pr.rs` | Rewrite | 严格实测:pr list JSON schema |
| `crates/e2e-github/tests/noauth.rs` | Create | 无凭据错误路径(全环境可跑) |
| `.github/workflows/e2e-tests.yml` | Rewrite | 每周 cron + 双包测试 + mode 汇报 |
| `.github/workflows/upstream-patrol.yml` | Create | nightly 巡检双 job + Issue 预警 |
| `release.toml` | Modify | `{{version}}` → `{version}`(根因修复) |
| `scripts/release.sh` | Modify | 校验函数 + `--self-test` + 闸门接入 + 无条件 dry-run + `--rehearse` |
| `Makefile` | Modify | `release-rehearse` target + `.PHONY` |
| `docs/release-workflow.md` | Modify | 事故复盘 + 校验闸门 + 演练流程 |
| `docs/e2e-test-setup-guide.md` | Modify | secrets 表更新 + 周计划与双层模式说明 |

---

### Task 1: e2e-core — TestMode 与 TestConfig 凭据派生 API

**Files:**
- Modify: `crates/e2e-core/src/config.rs`
- Modify: `crates/e2e-core/src/lib.rs`(导出)

**Interfaces:**
- Consumes: 无
- Produces: `TestMode { Authenticated, Unauthenticated }`(derive `Debug, Clone, Copy, PartialEq, Eq`);`TestConfig::mode(&self) -> TestMode`;`TestConfig::has_github_auth(&self) -> bool`;`TestConfig::gh_env(&self) -> Vec<(String, String)>`;`TestConfig::from_env_lenient() -> Self`。Task 5/6/7 的 e2e-github 测试依赖这组 API。

- [ ] **Step 1: Write the failing tests**

替换 `crates/e2e-core/src/config.rs` 末尾的空 `#[cfg(test)] mod tests`(现有注释说 env 测试因 unsafe 被跳过——改为直接构造值的 hermetic 测试,保留该注释说明):

```rust
#[cfg(test)]
mod tests {
    // Note: Environment variable tests are skipped because `std::env::set_var`
    // and `std::env::remove_var` are unsafe in Rust 2024, and this crate
    // forbids unsafe code with `#![forbid(unsafe_code)]`.
    // All logic is tested via directly constructed `TestConfig` values instead.

    use super::*;

    fn config_with_token() -> TestConfig {
        TestConfig {
            test_repo: "owner/repo".to_string(),
            github_token: Some("test-token".to_string()),
            gitcode_token: None,
            gitlab_token: None,
        }
    }

    fn config_without_token() -> TestConfig {
        TestConfig {
            test_repo: "owner/repo".to_string(),
            github_token: None,
            gitcode_token: None,
            gitlab_token: None,
        }
    }

    #[test]
    fn test_should_derive_authenticated_mode_when_github_token_present() {
        assert_eq!(config_with_token().mode(), TestMode::Authenticated);
    }

    #[test]
    fn test_should_derive_unauthenticated_mode_when_no_github_token() {
        assert_eq!(config_without_token().mode(), TestMode::Unauthenticated);
    }

    #[test]
    fn test_should_report_github_auth_presence() {
        assert!(config_with_token().has_github_auth());
        assert!(!config_without_token().has_github_auth());
    }

    #[test]
    fn test_should_emit_gh_token_env_pair_when_authenticated() {
        assert_eq!(
            config_with_token().gh_env(),
            vec![("GH_TOKEN".to_string(), "test-token".to_string())]
        );
    }

    #[test]
    fn test_should_emit_empty_env_when_unauthenticated() {
        assert!(config_without_token().gh_env().is_empty());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo nextest run -p e2e-core config`
Expected: FAIL — `TestMode` / `mode` / `has_github_auth` / `gh_env` 未定义(编译错误即 RED)

- [ ] **Step 3: Write minimal implementation**

在 `crates/e2e-core/src/config.rs` 的 `TestConfig` 定义之前加入枚举,并在 `impl TestConfig` 中追加方法:

```rust
/// 测试模式(由凭据可用性派生)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestMode {
    /// 已认证:可运行真实平台实测场景
    Authenticated,
    /// 未认证:仅运行错误路径与 harness 自测
    Unauthenticated,
}
```

```rust
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

    /// 是否具备 GitHub 凭据
    #[must_use]
    pub fn has_github_auth(&self) -> bool {
        self.github_token.is_some()
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
```

- [ ] **Step 4: Export new types from lib.rs**

修改 `crates/e2e-core/src/lib.rs` 的重导出:

```rust
pub use config::{TestConfig, TestMode};
pub use tty::{CommandOutput, TtyError, TtyMode, TtyRunner};
```

(`TtyError` 供 Task 4 的 harness 测试匹配 `NotFound`;`TestMode` 供 e2e-github 门控。)

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo nextest run -p e2e-core config`
Expected: PASS(5 个测试全绿)

- [ ] **Step 6: Lint**

Run: `cargo clippy -p e2e-core --all-targets -- -D warnings -W clippy::pedantic`
Expected: 无警告(`map_or_else` 形式已符合 pedantic)

- [ ] **Step 7: Commit**

```bash
git add crates/e2e-core/src/config.rs crates/e2e-core/src/lib.rs
git commit -m "feat(e2e-core): add TestMode and credential-derived config API

- TestMode { Authenticated, Unauthenticated } derived from github_token
- gh_env() injects GH_TOKEN into gh subprocesses (fixes credentials
  never reaching the underlying CLI)
- from_env_lenient() for tests that do not need E2E_TEST_REPO

Refs #96"
```

---

### Task 2: e2e-core — TtyRunner::env_remove

**Files:**
- Modify: `crates/e2e-core/src/tty.rs`

**Interfaces:**
- Consumes: `TtyRunner`(现有)
- Produces: `TtyRunner::env_remove<K: Into<String>>(&mut self, key: K) -> &mut Self`。Task 7 无凭据测试依赖它移除继承的 `GH_TOKEN`/`GITHUB_TOKEN`。

- [ ] **Step 1: Write the failing test**

在 `crates/e2e-core/src/tty.rs` 的 `#[cfg(test)] mod tests` 中追加(同模块可访问私有字段):

```rust
    #[test]
    fn test_should_record_env_removals_in_order() {
        let mut runner = TtyRunner::new(TtyMode::NonInteractive);
        runner
            .env_remove("GH_TOKEN")
            .env_remove("GITHUB_TOKEN")
            .env("E2E_PROBE", "1");
        assert_eq!(
            runner.env_removals,
            vec!["GH_TOKEN".to_string(), "GITHUB_TOKEN".to_string()]
        );
        assert_eq!(
            runner.env_vars.get("E2E_PROBE"),
            Some(&"1".to_string())
        );
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo nextest run -p e2e-core tty`
Expected: FAIL — `env_removals` 字段与 `env_remove` 方法不存在

- [ ] **Step 3: Write minimal implementation**

`TtyRunner` 结构体新增字段:

```rust
    env_removals: Vec<String>,
```

`TtyRunner::new` 中初始化 `env_removals: Vec::new(),`。在 `env` 方法后新增:

```rust
    /// 从子进程环境中移除变量(如清除继承的 `GH_TOKEN` 以测试未认证路径)
    pub fn env_remove<K>(&mut self, key: K) -> &mut Self
    where
        K: Into<String>,
    {
        self.env_removals.push(key.into());
        self
    }
```

`run` 方法中 `for (k, v) in &self.env_vars { cmd.env(k, v); }` 之后加入:

```rust
        for key in &self.env_removals {
            cmd.env_remove(key);
        }
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo nextest run -p e2e-core tty`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/e2e-core/src/tty.rs
git commit -m "feat(e2e-core): add TtyRunner::env_remove for hermetic no-auth tests

Refs #96"
```

---

### Task 3: e2e-core — TestFixture::with_config

**Files:**
- Modify: `crates/e2e-core/src/fixture.rs`

**Interfaces:**
- Consumes: `TestConfig`(Task 1)
- Produces: `TestFixture::with_config(config: &TestConfig) -> Self`;`new()` 委托给它。

- [ ] **Step 1: Write the failing tests**

在 `crates/e2e-core/src/fixture.rs` 末尾新增:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::TestConfig;

    fn offline_config() -> TestConfig {
        TestConfig {
            test_repo: "owner/repo".to_string(),
            github_token: None,
            gitcode_token: None,
            gitlab_token: None,
        }
    }

    #[test]
    fn test_should_build_fixture_from_config_without_env_access() {
        let fixture = TestFixture::with_config(&offline_config());
        assert_eq!(fixture.repo, "owner/repo");
        assert!(fixture.created_resources.is_empty());
    }

    #[tokio::test]
    async fn test_should_cleanup_empty_fixture_without_side_effects() {
        let mut fixture = TestFixture::with_config(&offline_config());
        assert!(fixture.cleanup().await.is_ok());
    }

    #[test]
    fn test_should_not_panic_when_dropping_empty_fixture() {
        let fixture = TestFixture::with_config(&offline_config());
        drop(fixture);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo nextest run -p e2e-core fixture`
Expected: FAIL — `with_config` 不存在;`repo`/`created_resources` 私有但同模块可访问(编译期仅缺方法)

- [ ] **Step 3: Write minimal implementation**

在 `impl TestFixture` 中,将 `new()` 改为委托并新增 `with_config`:

```rust
    /// 从显式配置创建测试固件(不访问环境变量,便于 hermetic 测试)
    #[must_use]
    pub fn with_config(config: &crate::TestConfig) -> Self {
        Self {
            repo: config.test_repo.clone(),
            created_resources: Vec::new(),
        }
    }

    /// 创建新的测试固件
    ///
    /// # Errors
    ///
    /// Returns `FixtureError::Config` if `E2E_TEST_REPO` is not set.
    pub fn new() -> Result<Self, FixtureError> {
        let config = crate::TestConfig::from_env()?;
        Ok(Self::with_config(&config))
    }
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo nextest run -p e2e-core fixture`
Expected: PASS(3 个)

- [ ] **Step 5: Commit**

```bash
git add crates/e2e-core/src/fixture.rs
git commit -m "feat(e2e-core): add TestFixture::with_config for env-free construction

Refs #96"
```

---

### Task 4: e2e-core — harness 集成测试

**Files:**
- Create: `crates/e2e-core/tests/harness.rs`

**Interfaces:**
- Consumes: `TtyRunner`、`TtyError`(Task 1 导出)
- Produces: 2 个集成测试(二进制不在 PATH 时 skip 而非 fail)

- [ ] **Step 1: Write the test file**

```rust
//! E2E harness 自测:二进制发现与错误传播。
//!
//! 需要 `gitflow-cli` 在 PATH 中:`cargo build --release` 后
//! `export PATH="$PWD/target/release:$PATH"`;CI 由 e2e-tests.yml
//! 的构建步骤保证。二进制缺失时测试 skip(不 fail)。

#![allow(clippy::unwrap_used, reason = "Test code uses unwrap for simplicity")]

use e2e_core::{TtyError, TtyMode, TtyRunner};

fn is_missing_binary(err: &TtyError) -> bool {
    matches!(err, TtyError::Io(e) if e.kind() == std::io::ErrorKind::NotFound)
}

#[tokio::test]
async fn test_should_run_help_successfully_in_both_tty_modes() {
    for mode in [TtyMode::Interactive, TtyMode::NonInteractive] {
        let runner = TtyRunner::new(mode);
        let output = match runner.run(&["--help"]).await {
            Ok(output) => output,
            Err(e) if is_missing_binary(&e) => {
                eprintln!("skipped: gitflow-cli not in PATH (cargo build --release first)");
                return;
            }
            Err(e) => panic!("unexpected runner error: {e}"),
        };
        assert!(
            output.status.success(),
            "mode {mode:?}: exit {:?}, stderr: {}",
            output.status,
            output.stderr
        );
        assert!(
            output.stdout.contains("gitflow"),
            "mode {mode:?}: stdout missing product name: {}",
            output.stdout
        );
    }
}

#[tokio::test]
async fn test_should_propagate_nonzero_exit_for_unknown_subcommand() {
    let runner = TtyRunner::new(TtyMode::NonInteractive);
    let output = match runner.run(&["definitely-not-a-real-subcommand"]).await {
        Ok(output) => output,
        Err(e) if is_missing_binary(&e) => {
            eprintln!("skipped: gitflow-cli not in PATH (cargo build --release first)");
            return;
        }
        Err(e) => panic!("unexpected runner error: {e}"),
    };
    assert!(!output.status.success(), "unknown subcommand must fail");
    assert!(
        !output.stderr.is_empty(),
        "stderr should contain a clap usage error"
    );
}
```

- [ ] **Step 2: Verify tests skip without the binary (RED-equivalent for harness tests)**

Run: `env PATH="/usr/bin:/bin" cargo nextest run -p e2e-core --test harness`
Expected: 2 tests 通过且输出 `skipped: gitflow-cli not in PATH`(证明门控逻辑;若 PATH 中恰好有 gitflow-cli 则直接 PASS,同样合格)

- [ ] **Step 3: Build the binary and verify tests pass (GREEN)**

Run:
```bash
cargo build --release
env PATH="$PWD/target/release:$PATH" cargo nextest run -p e2e-core --test harness
```
Expected: PASS(2 个,无 skip 输出)

- [ ] **Step 4: Lint**

Run: `cargo clippy -p e2e-core --all-targets -- -D warnings -W clippy::pedantic`
Expected: 无警告

- [ ] **Step 5: Commit**

```bash
git add crates/e2e-core/tests/harness.rs
git commit -m "test(e2e-core): add harness self-tests for binary discovery and error propagation

Refs #96"
```

---

### Task 5: e2e-github — 实化 auth 实测(凭据门控 + 严格断言)

**Files:**
- Modify: `crates/e2e-github/Cargo.toml`
- Rewrite: `crates/e2e-github/tests/auth.rs`

**Interfaces:**
- Consumes: `TestConfig::from_env_lenient/mode/gh_env`、`TestMode`、`TtyRunner::env`(Task 1)
- Produces: 严格实测样板(凭据门控 + `GH_TOKEN` 注入 + JSON 断言),Task 6 复用同一模式

- [ ] **Step 1: Add serde_json dev-dependency**

`crates/e2e-github/Cargo.toml` 的 `[dev-dependencies]` 追加:

```toml
serde_json = "1.0"
```

- [ ] **Step 2: Rewrite tests/auth.rs**

完整替换 `crates/e2e-github/tests/auth.rs`(删除 4 个空泛断言测试):

```rust
//! GitHub auth 命令 E2E 实测(真实凭据,严格断言)
//!
//! 无 `E2E_GITHUB_TOKEN` 时自动 skip(fork PR 路径)。

#![allow(clippy::unwrap_used, reason = "Test code uses unwrap for simplicity")]

use e2e_core::{TestConfig, TestMode, TtyMode, TtyRunner};

#[tokio::test]
async fn test_should_report_logged_in_with_real_credentials() {
    let config = TestConfig::from_env_lenient();
    if config.mode() != TestMode::Authenticated {
        eprintln!("skipped: E2E_GITHUB_TOKEN not set");
        return;
    }

    for tty_mode in [TtyMode::Interactive, TtyMode::NonInteractive] {
        let mut runner = TtyRunner::new(tty_mode);
        for (key, value) in config.gh_env() {
            runner.env(key, value);
        }

        let output = runner
            .run(&["auth", "status", "--platform", "github", "--output", "json"])
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

- [ ] **Step 3: Run without credentials — verify skip**

Run: `env -u E2E_GITHUB_TOKEN cargo nextest run -p e2e-github --test auth`
Expected: PASS 且输出 `skipped: E2E_GITHUB_TOKEN not set`

- [ ] **Step 4: Run with credentials — verify strict assertions (GREEN)**

Run(本地已 `gh auth login` 或有令牌):
```bash
env E2E_GITHUB_TOKEN="$(gh auth token)" cargo nextest run -p e2e-github --test auth
```
Expected: PASS,无 skip;若 `loggedIn != true` 说明 `GH_TOKEN` 注入链路有问题(这正是本任务要修复的根因)

- [ ] **Step 5: Lint + fmt**

Run: `cargo clippy -p e2e-github --all-targets -- -D warnings -W clippy::pedantic && cargo +nightly fmt -- --check`
Expected: 无警告

- [ ] **Step 6: Commit**

```bash
git add crates/e2e-github/Cargo.toml crates/e2e-github/tests/auth.rs
git commit -m "test(e2e-github): materialize auth e2e with strict JSON assertions

Replaces vacuous always-pass assertions with credential-gated real
checks: GH_TOKEN injected into the gh subprocess, loggedIn==true
asserted on the JSON envelope.

Refs #96"
```

---

### Task 6: e2e-github — 实化 issue/pr 实测(JSON schema 严格断言)

**Files:**
- Rewrite: `crates/e2e-github/tests/issue.rs`
- Rewrite: `crates/e2e-github/tests/pr.rs`

**Interfaces:**
- Consumes: Task 5 的模式(`from_env_lenient` + `gh_env` 注入)
- Produces: 对**当前仓库**的 issue/pr 只读实测(实测捕获的 JSON 形状:`data` 为数组,元素含 `number: u64`、`title: string`)

- [ ] **Step 1: Rewrite tests/issue.rs**

```rust
//! GitHub issue 命令 E2E 实测(真实凭据,严格 schema 断言)
//!
//! 对当前仓库(CI checkout / 本地 clone)运行;无凭据时 skip。

#![allow(clippy::unwrap_used, reason = "Test code uses unwrap for simplicity")]

use e2e_core::{TestConfig, TestMode, TtyMode, TtyRunner};

#[tokio::test]
async fn test_should_list_open_issues_with_valid_schema() {
    let config = TestConfig::from_env_lenient();
    if config.mode() != TestMode::Authenticated {
        eprintln!("skipped: E2E_GITHUB_TOKEN not set");
        return;
    }

    let mut runner = TtyRunner::new(TtyMode::NonInteractive);
    for (key, value) in config.gh_env() {
        runner.env(key, value);
    }

    let output = runner
        .run(&[
            "issue", "list", "--platform", "github", "--state", "open", "--output", "json",
        ])
        .await
        .unwrap();

    assert!(output.status.success(), "stderr: {}", output.stderr);
    let parsed: serde_json::Value =
        serde_json::from_str(&output.stdout).expect("stdout must be a JSON envelope");
    assert_eq!(parsed["success"], serde_json::json!(true), "stdout: {}", output.stdout);

    let items = parsed["data"]
        .as_array()
        .expect("data must be an array of issues");
    assert!(
        !items.is_empty(),
        "this repository should have at least one open issue"
    );
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

- [ ] **Step 2: Rewrite tests/pr.rs**

```rust
//! GitHub pr 命令 E2E 实测(真实凭据,严格 schema 断言)
//!
//! 查询 closed 状态以保证结果非空(已合并 PR 不会消失)。

#![allow(clippy::unwrap_used, reason = "Test code uses unwrap for simplicity")]

use e2e_core::{TestConfig, TestMode, TtyMode, TtyRunner};

#[tokio::test]
async fn test_should_list_closed_prs_with_valid_schema() {
    let config = TestConfig::from_env_lenient();
    if config.mode() != TestMode::Authenticated {
        eprintln!("skipped: E2E_GITHUB_TOKEN not set");
        return;
    }

    let mut runner = TtyRunner::new(TtyMode::NonInteractive);
    for (key, value) in config.gh_env() {
        runner.env(key, value);
    }

    let output = runner
        .run(&[
            "pr", "list", "--platform", "github", "--state", "closed", "--output", "json",
        ])
        .await
        .unwrap();

    assert!(output.status.success(), "stderr: {}", output.stderr);
    let parsed: serde_json::Value =
        serde_json::from_str(&output.stdout).expect("stdout must be a JSON envelope");
    assert_eq!(parsed["success"], serde_json::json!(true), "stdout: {}", output.stdout);

    let items = parsed["data"]
        .as_array()
        .expect("data must be an array of pull requests");
    assert!(
        !items.is_empty(),
        "this repository should have at least one closed PR"
    );
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

- [ ] **Step 3: Run without credentials — verify skip**

Run: `env -u E2E_GITHUB_TOKEN cargo nextest run -p e2e-github --test issue --test pr`
Expected: PASS,两条 skip

- [ ] **Step 4: Run with credentials — verify strict assertions (GREEN)**

Run:
```bash
env E2E_GITHUB_TOKEN="$(gh auth token)" cargo nextest run -p e2e-github --test issue --test pr
```
Expected: PASS;若 schema 断言失败,对照 `gitflow-cli issue list --output json | head -30` 的实际形状修正(本计划已按实测形状编写:`data` 数组 + `number`/`title` 字段)

- [ ] **Step 5: Lint + fmt**

Run: `cargo clippy -p e2e-github --all-targets -- -D warnings -W clippy::pedantic && cargo +nightly fmt -- --check`
Expected: 无警告

- [ ] **Step 6: Commit**

```bash
git add crates/e2e-github/tests/issue.rs crates/e2e-github/tests/pr.rs
git commit -m "test(e2e-github): materialize issue/pr e2e with JSON schema assertions

Tests run against the current repository (CI checkout / local clone)
and validate the deserialization contract: data array with u64 number
and non-empty title on every item.

Refs #96"
```

---

### Task 7: e2e-github — 无凭据错误路径测试

**Files:**
- Create: `crates/e2e-github/tests/noauth.rs`

**Interfaces:**
- Consumes: `TtyRunner::env_remove`(Task 2)
- Produces: 全环境可跑的未认证契约测试(实测契约:exit 1 + miette 诊断含 `gh auth login`)

- [ ] **Step 1: Write the test file**

```rust
//! 未认证错误路径 E2E 测试(无需凭据,任何环境均可运行)
//!
//! 通过 `env_remove` 清除继承的令牌、`GH_CONFIG_DIR` 指向空目录
//! 屏蔽 `gh` 的 hosts.yml,构造确定性的未认证环境。

#![allow(clippy::unwrap_used, reason = "Test code uses unwrap for simplicity")]

use e2e_core::{TtyMode, TtyRunner};

fn scrubbed_runner() -> TtyRunner {
    let mut runner = TtyRunner::new(TtyMode::NonInteractive);
    for var in ["GH_TOKEN", "GITHUB_TOKEN", "GH_ENTERPRISE_TOKEN"] {
        runner.env_remove(var);
    }
    let empty_config = std::env::temp_dir().join(format!("e2e-noauth-{}", std::process::id()));
    std::fs::create_dir_all(&empty_config).unwrap();
    runner.env("GH_CONFIG_DIR", empty_config.to_string_lossy().to_string());
    runner
}

#[tokio::test]
async fn test_should_fail_with_login_guidance_when_status_checked_unauthenticated() {
    let runner = scrubbed_runner();
    let output = runner
        .run(&["auth", "status", "--platform", "github", "--output", "json"])
        .await
        .unwrap();

    assert!(
        !output.status.success(),
        "unauthenticated auth status must exit non-zero"
    );
    let combined = format!("{}{}", output.stdout, output.stderr);
    assert!(
        combined.contains("gh auth login"),
        "expected login guidance in output, got: {combined}"
    );
}

#[tokio::test]
async fn test_should_fail_with_login_guidance_when_listing_issues_unauthenticated() {
    let runner = scrubbed_runner();
    let output = runner
        .run(&["issue", "list", "--platform", "github", "--output", "json"])
        .await
        .unwrap();

    assert!(
        !output.status.success(),
        "unauthenticated issue list must exit non-zero"
    );
    let combined = format!("{}{}", output.stdout, output.stderr);
    assert!(
        combined.contains("gh auth login"),
        "expected login guidance in output, got: {combined}"
    );
}
```

- [ ] **Step 2: Run tests (these exercise the real binary — build first)**

Run:
```bash
cargo build --release
env PATH="$PWD/target/release:$PATH" cargo nextest run -p e2e-github --test noauth
```
Expected: PASS(2 个);这两个测试在 fork PR 上同样运行,是 PR 路径的核心信号

- [ ] **Step 3: Lint + fmt**

Run: `cargo clippy -p e2e-github --all-targets -- -D warnings -W clippy::pedantic && cargo +nightly fmt -- --check`
Expected: 无警告

- [ ] **Step 4: Commit**

```bash
git add crates/e2e-github/tests/noauth.rs
git commit -m "test(e2e-github): add unauthenticated error path e2e tests

Hermetic in any environment: inherited tokens removed via env_remove,
gh hosts.yml masked via GH_CONFIG_DIR. Verifies the exit-1 + login
guidance contract that runs on fork PRs without secrets.

Refs #96"
```

---

### Task 8: e2e-tests.yml — 每周定时 + 双包 + mode 汇报

**Files:**
- Rewrite: `.github/workflows/e2e-tests.yml`

**Interfaces:**
- Consumes: Task 1-7 的测试
- Produces: 每周一 02:00 UTC 定时回归;模式判定完全由测试层门控(workflow 无 if 分支)

- [ ] **Step 1: Rewrite the workflow**

完整替换 `.github/workflows/e2e-tests.yml`:

```yaml
name: E2E Tests

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
    paths:
      - 'crates/**'
      - 'apps/**'
      - '.github/workflows/e2e-tests.yml'
  schedule:
    - cron: '0 2 * * 1'  # 每周一 02:00 UTC 定时回归(真实凭据)
  workflow_dispatch:

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

permissions:
  contents: read

jobs:
  e2e-github:
    name: E2E Tests (GitHub)
    runs-on: ubuntu-latest
    timeout-minutes: 30

    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Install cargo-nextest
        uses: taiki-e/install-action@cargo-nextest

      - name: Build release binary
        run: cargo build --release

      - name: Add to PATH
        run: echo "${{ github.workspace }}/target/release" >> $GITHUB_PATH

      # 模式判定由测试层承担:fork PR 取不到 secrets,
      # E2E_GITHUB_TOKEN 为空 → 实测自动 skip,无凭据错误路径正常运行。
      - name: Run E2E tests
        env:
          E2E_GITHUB_TOKEN: ${{ secrets.E2E_GITHUB_TOKEN }}
          RUST_LOG: info
        run: |
          cargo nextest run -p e2e-core -p e2e-github --all-features

      - name: Report run mode
        if: always()
        env:
          E2E_GITHUB_TOKEN: ${{ secrets.E2E_GITHUB_TOKEN }}
        run: |
          if [ -n "$E2E_GITHUB_TOKEN" ]; then
            echo "E2E mode: authenticated (real-credential scenarios executed)" >> "$GITHUB_STEP_SUMMARY"
          else
            echo "E2E mode: unauthenticated (error paths + harness self-tests only)" >> "$GITHUB_STEP_SUMMARY"
          fi

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: e2e-results-github
          path: target/nextest/
          retention-days: 7
```

(变更点:cron 每日 → 每周一;测试命令加 `-p e2e-core`;移除无人消费的 `E2E_TEST_REPO`;新增 mode 汇报。)

- [ ] **Step 2: Validate YAML syntax**

Run: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/e2e-tests.yml'))" && echo OK`
Expected: `OK`;若本地有 `actionlint` 则额外运行 `actionlint .github/workflows/e2e-tests.yml`(无输出即通过)

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/e2e-tests.yml
git commit -m "ci: weekly scheduled e2e regression with dual-mode test layer

- cron moved from daily to weekly (Monday 02:00 UTC)
- e2e-core harness self-tests added to the run
- mode detection delegated to tests (fork PRs run unauthenticated
  error paths; schedule/main/dispatch run real-credential scenarios)
- unused E2E_TEST_REPO env dropped; run mode reported to step summary

Refs #96"
```

---

### Task 9: upstream-patrol.yml — nightly 巡检 + upstream-drift 标签

**Files:**
- Create: `.github/workflows/upstream-patrol.yml`
- Runtime: 仓库标签 `upstream-drift`(本地用 gitflow-cli 创建,dogfooding)

**Interfaces:**
- Consumes: `docs/compatibility-matrix.json`(`min_version`/`tested_versions`,jq 解析)、`scripts/smoke-test.sh --platform github --read-only`(优先使用 `./target/release/gitflow-cli`)
- Produces: nightly 双 job 巡检;`upstream-drift` 标签下的去重预警 Issue

- [ ] **Step 1: Create the upstream-drift label (dogfooding)**

Run(在仓库根目录,已登录 gh):
```bash
gitflow-cli label create upstream-drift --color "FBCA04"
```
Expected: 成功创建(若已存在会报错,忽略)。验证:`gitflow-cli label list | grep upstream-drift`

- [ ] **Step 2: Write the workflow**

创建 `.github/workflows/upstream-patrol.yml`:

```yaml
name: Upstream CLI Patrol

on:
  schedule:
    - cron: '0 3 * * *'  # 每天 03:00 UTC 巡检上游 CLI 新版本
  workflow_dispatch:

permissions:
  contents: read
  issues: write

jobs:
  version-check:
    name: Version Check (no credentials)
    runs-on: ubuntu-latest
    timeout-minutes: 15

    steps:
      - uses: actions/checkout@v4

      - name: Query upstream latest versions
        id: latest
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          set -euo pipefail
          # gh: GitHub API(ubuntu runner 预装 gh,公共仓库匿名可读)
          GH_LATEST=$(gh api repos/cli/cli/releases/latest --jq '.tag_name' | sed 's/^v//')
          echo "gh=$GH_LATEST" >> "$GITHUB_OUTPUT"

          # glab: GitLab 公共 API
          GLAB_LATEST=$(curl -fsSL --max-time 30 \
            "https://gitlab.com/api/v4/projects/gitlab-org%2Fcli/repository/tags?per_page=1" \
            | jq -r '.[0].name' | sed 's/^v//') || GLAB_LATEST=""
          if [ -n "$GLAB_LATEST" ]; then
            echo "glab=$GLAB_LATEST" >> "$GITHUB_OUTPUT"
          else
            echo "::warning::Could not query latest glab version; skipping glab check"
          fi

          # gitcode: 尽力而为,失败仅告警(访问不稳定)
          GITCODE_LATEST=$(curl -fsSL --max-time 20 \
            "https://gitcode.com/api/v5/repos/gitcode-cli/cli/releases/latest" \
            | jq -r '.tag_name' | sed 's/^v//') || GITCODE_LATEST=""
          if [ -n "$GITCODE_LATEST" ]; then
            echo "gitcode=$GITCODE_LATEST" >> "$GITHUB_OUTPUT"
          else
            echo "::warning::Could not query latest gitcode version; skipping gitcode check"
          fi

      - name: Compare against compatibility matrix and alert
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          set -euo pipefail
          MATRIX="docs/compatibility-matrix.json"
          ALERT_FOUND=false

          check_platform() {
            local binary="$1" latest="$2"
            [ -z "$latest" ] && return 0
            local tested_max
            tested_max=$(jq -r --arg b "$binary" \
              '.platforms[] | select(.cli_binary == $b) | .tested_versions | sort_by(split(".") | map(tonumber)) | last' \
              "$MATRIX")
            # 若 latest 比 tested_max 新,则有漂移
            local newest
            newest=$(printf '%s\n' "$tested_max" "$latest" | sort -V | tail -1)
            if [ "$newest" != "$tested_max" ]; then
              ALERT_FOUND=true
              local title="upstream CLI 新版本: ${binary} ${latest}"
              local existing
              existing=$(gh issue list --label upstream-drift --state open \
                --search "in:title upstream CLI ${binary}" --json number --jq '.[0].number // empty')
              local body_file
              body_file=$(mktemp)
              {
                printf '巡检发现 %s 发布了新版本。\n\n' "$binary"
                printf -- '- **最新版本**: %s\n' "$latest"
                printf -- '- **兼容性矩阵 tested_versions 上限**: %s\n' "$tested_max"
                printf -- '- **矩阵文件**: `%s`\n\n' "$MATRIX"
                printf '## 建议动作\n\n'
                printf '1. 本地安装 %s %s,运行 `make smoke-test` 与契约测试验证\n' "$binary" "$latest"
                printf '2. 若验证通过,更新 `%s` 的 `tested_versions`\n' "$MATRIX"
                printf '3. 若存在破坏性变更,评估是否提升 `min_version` 并补充契约 fixture\n'
              } > "$body_file"
              if [ -n "$existing" ]; then
                gh issue comment "$existing" --body "巡检复核(${binary} ${latest} 仍为最新): $(date -u +%F)"
              else
                gh issue create --title "$title" --label upstream-drift --body-file "$body_file"
              fi
              rm -f "$body_file"
            fi
          }

          check_platform "gh" "${{ steps.latest.outputs.gh }}"
          check_platform "glab" "${{ steps.latest.outputs.glab }}"
          check_platform "gitcode" "${{ steps.latest.outputs.gitcode }}"

          if [ "$ALERT_FOUND" = true ]; then
            echo "Drift detected — issue(s) created or updated."
          else
            echo "All platforms within tested versions. No alert needed."
          fi

  github-live-check:
    name: GitHub Live Smoke (latest gh)
    runs-on: ubuntu-latest
    timeout-minutes: 30
    if: ${{ secrets.E2E_GITHUB_TOKEN != '' }}

    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Install latest gh
        run: |
          set -euo pipefail
          LATEST=$(gh api repos/cli/cli/releases/latest --jq '.tag_name' | sed 's/^v//')
          curl -fsSL "https://github.com/cli/cli/releases/download/v${LATEST}/gh_${LATEST}_linux_amd64.tar.gz" -o gh.tar.gz
          tar -xzf gh.tar.gz
          sudo cp "gh_${LATEST}_linux_amd64/bin/gh" /usr/local/bin/gh
          gh --version

      - name: Build gitflow-cli
        run: cargo build --release

      - name: Run read-only smoke test against latest gh
        id: smoke
        env:
          GH_TOKEN: ${{ secrets.E2E_GITHUB_TOKEN }}
        run: |
          if bash scripts/smoke-test.sh --platform github --read-only; then
            echo "result=pass" >> "$GITHUB_OUTPUT"
          else
            echo "result=fail" >> "$GITHUB_OUTPUT"
          fi

      - name: Alert on breakage
        if: steps.smoke.outputs.result == 'fail'
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          set -euo pipefail
          GH_VERSION=$(gh --version | head -1)
          TITLE="upstream CLI 破坏: gh (${GH_VERSION}) smoke 失败"
          EXISTING=$(gh issue list --label upstream-drift --state open \
            --search "in:title upstream CLI 破坏 gh" --json number --jq '.[0].number // empty')
          BODY="nightly live smoke 在最新 gh(\`${GH_VERSION}\`)上失败,疑似上游破坏性变更。

          请查看本次 workflow run 日志定位失败命令,并评估:
          1. 修复适配器或锁定行为
          2. 必要时提升 \`docs/compatibility-matrix.json\` 的 \`min_version\`"
          if [ -n "$EXISTING" ]; then
            gh issue comment "$EXISTING" --body "live smoke 复核仍失败(${GH_VERSION}): $(date -u +%F)"
          else
            gh issue create --title "$TITLE" --label upstream-drift --body "$BODY"
          fi
```

- [ ] **Step 3: Validate YAML + shell blocks**

Run: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/upstream-patrol.yml'))" && echo OK`
Expected: `OK`;有 `actionlint` 则运行之。另将两个 run 块分别粘贴到临时 `.sh` 文件跑 `bash -n` 语法检查。

- [ ] **Step 4: Commit**

```bash
git add .github/workflows/upstream-patrol.yml
git commit -m "ci: add nightly upstream CLI patrol with issue alerting

- version-check: queries latest gh/glab/gitcode releases, compares
  against compatibility-matrix.json tested_versions, opens/updates
  deduplicated upstream-drift issues (gitcode failures warn-only)
- github-live-check: runs read-only smoke test against the latest gh
  with the bot token, alerts on breakage

Refs #96"
```

---

### Task 10: release.toml — 模板语法根因修复

**Files:**
- Modify: `release.toml`

**Interfaces:**
- Consumes: 无
- Produces: cargo-release ≥0.25 兼容的单括号模板;Task 12 的演练依赖它产生正确 tag

- [ ] **Step 1: Fix the three placeholders**

`release.toml` 中:

```toml
tag-name = "v{version}"
tag-message = "Release v{version}"
pre-release-commit-message = "chore: release v{version}"
```

- [ ] **Step 2: Verify no residue remains**

Run: `grep -n '{{version}}' release.toml; echo "exit=$?"`
Expected: `exit=1`(无匹配)

- [ ] **Step 3: Verify cargo-release resolves the config**

Run: `cargo release config 2>/dev/null | grep -E 'tag-name|pre-release-commit' || cargo release version patch --dry-run --workspace 2>&1 | head -10`
Expected: 配置解析无错误(若 `cargo release config` 子命令不可用,以 dry-run 无报错为准)

- [ ] **Step 4: Commit**

```bash
git add release.toml
git commit -m "fix(release): use cargo-release single-brace {version} templates

Root cause of the 'chore: release v{{version}}' incident (commits
9331bfa/0b0e9d7): cargo-release >= 0.25 does not substitute the
legacy double-brace syntax, leaving literal placeholders in commit
messages and tag names.

Refs #96"
```

---

### Task 11: release.sh — 事后校验函数 + --self-test(bash RED→GREEN)

**Files:**
- Modify: `scripts/release.sh`

**Interfaces:**
- Consumes: 无
- Produces: `validate_commit_subject <subject>`、`validate_tag_name <tag>`、`validate_no_template_residue <file>`(成功返回 0,失败打印原因返回 1);`run_self_test`;`--self-test` 入口。Task 12 在发布链路中接入。

- [ ] **Step 1: RED — write self-test with stub validators**

在 `scripts/release.sh` 的 `check_prerequisites()` 函数**之前**插入(先写测试与错误实现):

```bash
# ---------------------------------------------------------------------------
# Release artifact validation (pure functions; testable via --self-test)
# ---------------------------------------------------------------------------

# 未被替换的模板变量,如 {{version}}
TEMPLATE_RESIDUE_PATTERN='\{\{[a-zA-Z_]+\}\}'
# 合法 tag:vX.Y.Z 或 vX.Y.Z-<prerelease>
VERSION_TAG_PATTERN='^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$'
# 合法发布提交主题
RELEASE_COMMIT_PATTERN='^chore: release v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$'

validate_commit_subject() {
    # STUB(RED 阶段:恒成功,令 expect_fail 用例暴露测试有效性)
    return 0
}

validate_tag_name() {
    return 0
}

validate_no_template_residue() {
    return 0
}

run_self_test() {
    local failures=0

    expect_pass() {
        local desc="$1"; shift
        if "$@" >/dev/null 2>&1; then
            log_success "$desc"
        else
            log_error "$desc (expected pass, got fail)"
            failures=$((failures + 1))
        fi
    }

    expect_fail() {
        local desc="$1"; shift
        if "$@" >/dev/null 2>&1; then
            log_error "$desc (expected fail, got pass)"
            failures=$((failures + 1))
        else
            log_success "$desc"
        fi
    }

    echo ""
    log_info "Running release validation self-test..."

    expect_pass "commit subject: well-formed" validate_commit_subject "chore: release v1.0.0"
    expect_pass "commit subject: prerelease" validate_commit_subject "chore: release v1.0.0-rc.1"
    expect_fail "commit subject: template residue" validate_commit_subject "chore: release v{{version}}"
    expect_fail "commit subject: malformed" validate_commit_subject "release 1.0.0"

    expect_pass "tag: well-formed" validate_tag_name "v1.0.0"
    expect_pass "tag: prerelease" validate_tag_name "v1.0.0-rc.1"
    expect_fail "tag: template residue" validate_tag_name "v{{version}}"
    expect_fail "tag: missing v prefix" validate_tag_name "1.0.0"

    local tmp
    tmp=$(mktemp)
    printf '## v{{version}}\n' > "$tmp"
    expect_fail "changelog: template residue" validate_no_template_residue "$tmp"
    printf '## 1.0.0 - 2026-07-31\n' > "$tmp"
    expect_pass "changelog: clean" validate_no_template_residue "$tmp"
    rm -f "$tmp"

    echo ""
    if [ "$failures" -eq 0 ]; then
        log_success "Self-test passed"
        return 0
    fi
    log_error "Self-test failed: $failures case(s)"
    return 1
}
```

并修改文件顶部参数解析(替换现有 `QUICK_MODE` 判定 4 行)与 main 调度(Task 12 会扩展 main;此处先接入 self-test)。将:

```bash
# Quick mode flag
QUICK_MODE=false
if [[ "${1:-}" == "--quick" ]]; then
    QUICK_MODE=true
fi
```

改为:

```bash
# Mode flags
QUICK_MODE=false
REHEARSE_MODE=false
case "${1:-}" in
    --quick) QUICK_MODE=true ;;
    --rehearse) REHEARSE_MODE=true; QUICK_MODE=true ;;
    --self-test)
        # self-test 无需发布前置,直接运行并退出
        QUICK_MODE=true
        ;;
    "") ;;
    *)
        echo "Usage: bash scripts/release.sh [--quick|--rehearse|--self-test]" >&2
        exit 2
        ;;
esac
```

并在 `trap cleanup_on_error EXIT` 之后插入:

```bash
if [[ "${1:-}" == "--self-test" ]]; then
    trap - EXIT
    if run_self_test; then exit 0; else exit 1; fi
fi
```

- [ ] **Step 2: Run self-test to verify it fails (RED)**

Run: `bash scripts/release.sh --self-test`
Expected: FAIL — 6 个 `expect_fail` 用例报 `(expected fail, got pass)`(stub 恒成功证明测试能抓住坏实现),退出码 1

- [ ] **Step 3: GREEN — implement the validators**

替换三个 stub 函数体:

```bash
validate_commit_subject() {
    local subject="$1"
    if [[ "$subject" =~ $TEMPLATE_RESIDUE_PATTERN ]]; then
        log_error "Template residue in commit subject: $subject"
        return 1
    fi
    if [[ ! "$subject" =~ $RELEASE_COMMIT_PATTERN ]]; then
        log_error "Malformed release commit subject: $subject"
        return 1
    fi
    return 0
}

validate_tag_name() {
    local tag="$1"
    if [[ "$tag" =~ $TEMPLATE_RESIDUE_PATTERN ]]; then
        log_error "Template residue in tag name: $tag"
        return 1
    fi
    if [[ ! "$tag" =~ $VERSION_TAG_PATTERN ]]; then
        log_error "Malformed tag name: $tag"
        return 1
    fi
    return 0
}

validate_no_template_residue() {
    local file="$1"
    if grep -qE "$TEMPLATE_RESIDUE_PATTERN" "$file"; then
        log_error "Template residue found in $file:"
        grep -nE "$TEMPLATE_RESIDUE_PATTERN" "$file" | head -5
        return 1
    fi
    return 0
}
```

- [ ] **Step 4: Run self-test to verify it passes (GREEN)**

Run: `bash scripts/release.sh --self-test`
Expected: 全部 ✓,`Self-test passed`,退出码 0

- [ ] **Step 5: Commit**

```bash
git add scripts/release.sh
git commit -m "feat(release): add artifact validation functions with self-test

Pure validators for release commit subjects, tag names, and changelog
template residue, proven by a --self-test harness (RED stubs first,
then real implementations).

Refs #96"
```

---

### Task 12: release.sh — 闸门接入 + 无条件 dry-run + --rehearse + Makefile

**Files:**
- Modify: `scripts/release.sh`
- Modify: `Makefile`

**Interfaces:**
- Consumes: Task 11 的校验函数、Task 10 的模板修复
- Produces: 发布链路三道闸门;`--quick` 不再跳过 dry-run;`--rehearse` 完整 dry 演练报告;`make release-rehearse`

- [ ] **Step 1: Wire gates into execute_release**

在 `execute_release()` 中:

`cargo release commit --execute --no-confirm`(Step 2/6)之后插入:

```bash
    # Gate: validate the release commit subject (blocks v{{version}}-class incidents)
    local commit_subject
    commit_subject=$(git log -1 --pretty=%s)
    if ! validate_commit_subject "$commit_subject"; then
        log_error "Release commit validation failed. Rolling back bump commit."
        git reset --hard HEAD~1
        exit 1
    fi
    log_success "Release commit subject validated"
```

`git commit -m "chore: update CHANGELOG.md for v${RELEASE_VERSION}" || true`(Step 4/6)之后插入:

```bash
    # Gate: no template residue in the generated changelog
    if ! validate_no_template_residue CHANGELOG.md; then
        log_error "CHANGELOG.md contains unsubstituted template variables. Aborting."
        exit 1
    fi
    log_success "CHANGELOG.md validated"
```

`cargo release tag --execute --workspace --no-confirm`(Step 7,函数末尾)之后、`git push origin main --tags` 之前插入:

```bash
    # Gate: validate the created tag before pushing it anywhere
    local created_tag
    created_tag=$(git tag --points-at HEAD | head -1)
    if ! validate_tag_name "$created_tag"; then
        log_error "Tag validation failed. Removing local tag."
        git tag -d "$created_tag"
        exit 1
    fi
    log_success "Tag $created_tag validated"
```

- [ ] **Step 2: Make dry-run unconditional in main()**

将 `main()` 中的:

```bash
    if ! $QUICK_MODE; then
        dry_run
    fi
```

改为:

```bash
    # dry-run 是强制步骤:--quick 仅跳过交互确认,不跳过 dry-run
    dry_run
```

- [ ] **Step 3: Add rehearsal report + dispatch**

在 `post_release()` 函数**之后**新增:

```bash
# Rehearsal report (dry-run drill; prints the mandatory checklist)
print_rehearsal_report() {
    echo ""
    echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║   Release Rehearsal Report (dry-run)   ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"
    echo ""
    echo "  ✅ Prerequisites (cargo / cargo-release / git-cliff)"
    echo "  ✅ On main branch, working tree clean"
    echo "  ✅ Tests passed (cargo nextest)"
    echo "  ✅ Clippy passed"
    echo "  ✅ Version preview: v${RELEASE_VERSION}"
    echo "  ✅ cargo release dry-run succeeded"
    echo "  ✅ Validation self-test:"
    if run_self_test > /dev/null 2>&1; then
        echo "     validators green (commit subject / tag / changelog residue)"
    else
        log_error "Validation self-test failed during rehearsal"
        exit 1
    fi
    echo ""
    log_success "Rehearsal passed. No changes were made (dry-run only)."
    log_info "Run 'bash scripts/release.sh' to perform the actual release."
}
```

并在 `main()` 的 `dry_run` 调用之后、`execute_release` 之前插入:

```bash
    if $REHEARSE_MODE; then
        print_rehearsal_report
        exit 0
    fi
```

(到达此处说明 check_prerequisites / preflight_checks / show_version_preview / preview_changelog / dry_run 全部通过;`QUICK_MODE=true` 使 confirm 自动通过,演练无需交互。trap 在 exit 0 时不打印恢复指引。)

- [ ] **Step 4: Verify --self-test still green and --rehearse parses**

Run: `bash scripts/release.sh --self-test && bash scripts/release.sh --bad-flag; echo "bad-flag exit=$?"`
Expected: self-test 通过;`bad-flag exit=2`

- [ ] **Step 5: Add the Makefile target**

在 `Makefile` 的 `release-quick:` 目标之后插入:

```makefile
release-rehearse: ## Full dry-run release drill (mandatory checklist, no changes)
	@bash scripts/release.sh --rehearse
```

并将 `release-rehearse` 追加到 `.PHONY:` 长列表行(含 `release release-quick` 的那一行)末尾。

- [ ] **Step 6: Syntax-check the script**

Run: `bash -n scripts/release.sh && echo SYNTAX-OK`
Expected: `SYNTAX-OK`

- [ ] **Step 7: Commit**

```bash
git add scripts/release.sh Makefile
git commit -m "feat(release): mandatory validation gates and dry-run rehearsal

- execute_release: validate commit subject, changelog residue, and tag
  name at each mutation point; roll back on failure with recovery hints
- dry-run is now unconditional: --quick only skips interactive prompts
- --rehearse / make release-rehearse: full dry-run drill with checklist
  report, exits non-zero on any failure, never mutates state

Refs #96"
```

---

### Task 13: 文档 — 事故复盘 + 演练流程 + e2e 指南更新

**Files:**
- Modify: `docs/release-workflow.md`
- Modify: `docs/e2e-test-setup-guide.md`

**Interfaces:**
- Consumes: Task 8-12 的既成事实
- Produces: 可检索的事故复盘与演练说明;secrets 表与现实一致

- [ ] **Step 1: Append the incident retrospective + gates + rehearsal to docs/release-workflow.md**

在文档末尾追加:

```markdown
## 事故复盘:`v{{version}}` 模板未替换

**现象**:历史提交 `9331bfa`/`0b0e9d7` 的提交主题字面为 `chore: release v{{version}}`。

**根因**:`release.toml` 使用 cargo-release 旧版双括号语法 `{{version}}`;
cargo-release ≥ 0.25 只替换单括号 `{version}`,旧写法被原样保留进提交主题与 tag 消息。

**修复**:`release.toml` 改为 `v{version}` 单括号语法(Issue #96)。

**防复发闸门**(`scripts/release.sh`):

| 闸门 | 位置 | 失败动作 |
|------|------|----------|
| 提交主题校验 | `cargo release commit` 后 | `git reset --hard HEAD~1` + 中止 |
| CHANGELOG 残留校验 | `git cliff` 后 | 中止 |
| tag 名校验 | `cargo release tag` 后、push 前 | `git tag -d` + 中止 |

校验器为纯函数,`bash scripts/release.sh --self-test` 可随时自测。

**强制 dry-run**:`--quick` 不再跳过 dry-run,仅跳过交互确认。

## 发布演练(1.0 发布前必做)

```bash
make release-rehearse
```

完整 dry 链路:前置检查 → main/干净工作区 → 测试 → clippy → 版本预览 →
`cargo release --dry-run` → 校验器自检。输出 ✅ 清单报告;任一失败退出码非 0;
绝不产生变更。演练输出摘录应附在对应发布 Issue 中作为证据。
```

- [ ] **Step 2: Update docs/e2e-test-setup-guide.md secrets table**

将 §3 secrets 表替换为:

```markdown
   | Name | Value | 用途 |
   |------|-------|------|
   | `E2E_GITHUB_TOKEN` | `ghp_xxxx`(上一步生成的令牌) | 每周定时回归的真实凭据实测(auth/issue/pr) |

   > **注**:`E2E_TEST_REPO` 已不再被 e2e 测试引用(Issue #96 起实测对当前仓库运行,
   > 严格校验 JSON schema)。旧 secret 可保留,不影响运行。
```

并在 §1 概述后追加一节:

```markdown
## 1.5 运行模式与触发路径

e2e-tests.yml 的模式判定完全由测试层承担:

| 触发路径 | secrets 可用性 | 实际运行的测试 |
|----------|----------------|----------------|
| schedule(每周一 02:00 UTC)/ push main / workflow_dispatch | 有 | 真实凭据实测 + 无凭据错误路径 + harness 自测 |
| pull_request(含 fork PR) | 无 | 无凭据错误路径 + harness 自测(实测自动 skip) |

本地运行实测:`cargo build --release && export PATH="$PWD/target/release:$PATH" &&
E2E_GITHUB_TOKEN="$(gh auth token)" cargo nextest run -p e2e-core -p e2e-github`。
```

- [ ] **Step 3: Proofread rendered Markdown**

Run: 人工检查两个文件的标题层级与代码围栏配对;`grep -n '{{' docs/release-workflow.md` 确认复盘段落外的 `{{` 仅出现在引用事故现象处。

- [ ] **Step 4: Commit**

```bash
git add docs/release-workflow.md docs/e2e-test-setup-guide.md
git commit -m "docs: release incident retrospective, rehearsal flow, e2e mode guide

Refs #96"
```

---

## Final Verification(合入前整体校验)

- [ ] `cargo build --release`
- [ ] `env PATH="$PWD/target/release:$PATH" cargo nextest run --all-features`(全工作区;e2e 实测有凭据跑真实、无凭据 skip)
- [ ] `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`
- [ ] `cargo +nightly fmt -- --check`
- [ ] `bash scripts/release.sh --self-test`
- [ ] `make release-rehearse`(退出标准证据:输出 ✅ 清单,退出码 0;摘录贴入 Issue #96)
- [ ] `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/e2e-tests.yml')); yaml.safe_load(open('.github/workflows/upstream-patrol.yml'))"`(+ actionlint,若可用)
- [ ] PR body 含 `Closes #96`
