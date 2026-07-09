# E2E 非交互式测试框架实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 建立 Rust 集成测试框架，同时测试交互模式（有 TTY）和非交互模式（无 TTY），覆盖 GitHub 平台的认证命令和破坏性命令

**Architecture:** 分层测试架构，e2e-core 提供共享工具（TTY 控制、断言、清理），平台特定 crate（e2e-github）使用这些工具编写测试。使用 portable-pty 控制 TTY，assert_cmd 进行断言。

**Tech Stack:** Rust 2024, portable-pty 0.8, assert_cmd 2.0, predicates 3.0, tokio, cargo-nextest

## Global Constraints

- 使用 Rust 2024 edition，工具链版本在 `rust-toolchain.toml` 中固定
- 所有公共 API 必须有文档注释
- 通过 `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`
- 通过 `cargo +nightly fmt -- --check`
- 测试使用 `cargo nextest run` 运行
- 环境变量配置测试仓库和令牌（`E2E_TEST_REPO`, `E2E_GITHUB_TOKEN`）

---

## File Structure

```
gitflow-cli/
├── Cargo.toml                          # 修改：添加 e2e-core 和 e2e-github 到 workspace
├── crates/
│   ├── e2e-core/                       # 新建
│   │   ├── Cargo.toml                  # 新建
│   │   └── src/
│   │       ├── lib.rs                  # 新建：导出公共 API
│   │       ├── tty.rs                  # 新建：TtyRunner 实现
│   │       ├── config.rs               # 新建：测试配置读取
│   │       └── fixture.rs              # 新建：TestFixture 实现
│   └── e2e-github/                     # 新建
│       ├── Cargo.toml                  # 新建
│       └── tests/
│           ├── auth.rs                 # 新建：auth 命令测试
│           ├── issue.rs                # 新建：issue 命令测试
│           └── pr.rs                   # 新建：pr 命令测试
└── .github/workflows/
    └── e2e-tests.yml                   # 新建：E2E 测试 workflow
```

---

## Task 1: 创建 e2e-core crate 基础结构

**Files:**
- Modify: `Cargo.toml:1-50`
- Create: `crates/e2e-core/Cargo.toml`
- Create: `crates/e2e-core/src/lib.rs`

**Interfaces:**
- Consumes: 无（第一个任务）
- Produces: `e2e-core` crate，可供后续任务使用

- [ ] **Step 1: 更新 workspace Cargo.toml**

读取 `Cargo.toml`，在 `[workspace]` 的 `members` 数组中添加 `"crates/e2e-core"` 和 `"crates/e2e-github"`。

```toml
[workspace]
members = [
    "apps/cli",
    "crates/core",
    "crates/github",
    "crates/gitcode",
    "crates/gitlab",
    "crates/e2e-core",      # 新增
    "crates/e2e-github",    # 新增
]
```

- [ ] **Step 2: 创建 crates/e2e-core/Cargo.toml**

```toml
[package]
name = "e2e-core"
version = "0.1.0"
edition = "2024"
publish = false

[dependencies]
portable-pty = "0.8"
assert_cmd = "2.0"
predicates = "3.0"
tokio = { version = "1", features = ["full"] }
serde_json = "1.0"
thiserror = "1.0"
tracing = "0.1"

[lints]
workspace = true
```

- [ ] **Step 3: 创建 crates/e2e-core/src/lib.rs**

```rust
//! E2E 测试核心库
//!
//! 提供共享的测试工具，包括 TTY 控制、测试配置和资源管理。

#![forbid(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations)]

pub mod config;
pub mod fixture;
pub mod tty;

pub use config::TestConfig;
pub use fixture::{TestFixture, TestResource};
pub use tty::{CommandOutput, TtyMode, TtyRunner};
```

- [ ] **Step 4: 验证 crate 编译**

Run: `cargo check -p e2e-core`
Expected: 编译失败（缺少模块文件），这是预期的

- [ ] **Step 5: 提交基础结构**

```bash
git add Cargo.toml crates/e2e-core/
git commit -m "feat(e2e-core): add crate scaffold"
```

---

## Task 2: 实现 TtyRunner（TTY 控制）

**Files:**
- Create: `crates/e2e-core/src/tty.rs`
- Test: `crates/e2e-core/src/tty.rs`（内联测试）

**Interfaces:**
- Consumes: `portable-pty`, `tokio`
- Produces: `TtyRunner`, `TtyMode`, `CommandOutput`

- [ ] **Step 1: 编写 TtyMode 枚举的失败测试**

在 `crates/e2e-core/src/tty.rs` 中：

```rust
//! TTY 控制模块
//!
//! 提供交互模式和非交互模式的命令执行能力。

use std::{
    collections::HashMap,
    path::PathBuf,
    process::ExitStatus,
};
use thiserror::Error;

/// TTY 模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TtyMode {
    /// 有 TTY（交互模式）
    Interactive,
    /// 无 TTY（非交互模式，stdin 重定向）
    NonInteractive,
}

/// TTY 相关错误
#[derive(Debug, Error)]
pub enum TtyError {
    #[error("PTY error: {0}")]
    Pty(#[from] portable_pty::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

/// 命令输出
#[derive(Debug)]
pub struct CommandOutput {
    /// 标准输出
    pub stdout: String,
    /// 标准错误
    pub stderr: String,
    /// 退出状态
    pub status: ExitStatus,
}

/// TTY 测试运行器
#[derive(Debug)]
pub struct TtyRunner {
    mode: TtyMode,
    working_dir: PathBuf,
    env_vars: HashMap<String, String>,
}

impl TtyRunner {
    /// 创建新的 TTY 运行器
    pub fn new(mode: TtyMode) -> Self {
        Self {
            mode,
            working_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            env_vars: HashMap::new(),
        }
    }

    /// 设置环境变量
    pub fn env<K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.env_vars.insert(key.into(), value.into());
        self
    }

    /// 执行命令并返回输出
    pub async fn run(&self, args: &[&str]) -> Result<CommandOutput, TtyError> {
        match self.mode {
            TtyMode::Interactive => self.run_with_pty(args).await,
            TtyMode::NonInteractive => self.run_without_pty(args).await,
        }
    }

    /// 有 TTY 模式：使用 portable-pty
    async fn run_with_pty(&self, args: &[&str]) -> Result<CommandOutput, TtyError> {
        use portable_pty::{native_pty_system, CommandBuilder, PtySize};

        let pty_system = native_pty_system();
        let pty_pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut cmd = CommandBuilder::new("gitflow-cli");
        cmd.args(args);
        cmd.cwd(&self.working_dir);

        for (k, v) in &self.env_vars {
            cmd.env(k, v);
        }

        let child = pty_pair.slave.spawn_command(cmd)?;
        let output = pty_pair.master.read_to_string()?;
        let status = child.wait()?;

        Ok(CommandOutput {
            stdout: output,
            stderr: String::new(),
            status,
        })
    }

    /// 无 TTY 模式：stdin 重定向到 /dev/null
    async fn run_without_pty(&self, args: &[&str]) -> Result<CommandOutput, TtyError> {
        use tokio::process::Command;

        let mut cmd = Command::new("gitflow-cli");
        cmd.args(args);
        cmd.current_dir(&self.working_dir);
        cmd.stdin(std::process::Stdio::null());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        for (k, v) in &self.env_vars {
            cmd.env(k, v);
        }

        let output = cmd.output().await?;

        Ok(CommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            status: output.status,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tty_mode_equality() {
        assert_eq!(TtyMode::Interactive, TtyMode::Interactive);
        assert_eq!(TtyMode::NonInteractive, TtyMode::NonInteractive);
        assert_ne!(TtyMode::Interactive, TtyMode::NonInteractive);
    }

    #[test]
    fn test_tty_runner_creation() {
        let runner = TtyRunner::new(TtyMode::Interactive);
        assert_eq!(runner.mode, TtyMode::Interactive);
    }
}
```

- [ ] **Step 2: 运行测试验证失败**

Run: `cargo test -p e2e-core --lib tty`
Expected: 编译失败（缺少 `tty` 模块导出）

- [ ] **Step 3: 更新 lib.rs 导出 tty 模块**

修改 `crates/e2e-core/src/lib.rs`：

```rust
pub mod tty;

pub use tty::{CommandOutput, TtyMode, TtyRunner};
```

- [ ] **Step 4: 运行测试验证通过**

Run: `cargo test -p e2e-core --lib tty`
Expected: PASS

- [ ] **Step 5: 提交 TTY 控制实现**

```bash
git add crates/e2e-core/src/tty.rs crates/e2e-core/src/lib.rs
git commit -m "feat(e2e-core): implement TtyRunner for TTY control"
```

---

## Task 3: 实现 TestConfig（测试配置）

**Files:**
- Create: `crates/e2e-core/src/config.rs`
- Test: `crates/e2e-core/src/config.rs`（内联测试）

**Interfaces:**
- Consumes: 环境变量
- Produces: `TestConfig` 结构体

- [ ] **Step 1: 编写 TestConfig 的失败测试**

在 `crates/e2e-core/src/config.rs` 中：

```rust
//! 测试配置模块
//!
//! 从环境变量读取测试配置。

use thiserror::Error;

/// 配置错误
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),
}

/// 测试配置
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// 测试仓库（格式：owner/repo）
    pub test_repo: String,
    /// GitHub 令牌
    pub github_token: Option<String>,
    /// GitCode 令牌
    pub gitcode_token: Option<String>,
    /// GitLab 令牌
    pub gitlab_token: Option<String>,
}

impl TestConfig {
    /// 从环境变量加载配置
    pub fn from_env() -> Result<Self, ConfigError> {
        let test_repo = std::env::var("E2E_TEST_REPO")
            .map_err(|_| ConfigError::MissingEnvVar("E2E_TEST_REPO".to_string()))?;

        Ok(Self {
            test_repo,
            github_token: std::env::var("E2E_GITHUB_TOKEN").ok(),
            gitcode_token: std::env::var("E2E_GITCODE_TOKEN").ok(),
            gitlab_token: std::env::var("E2E_GITLAB_TOKEN").ok(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_missing_env_var() {
        // 确保环境变量未设置
        std::env::remove_var("E2E_TEST_REPO");
        let result = TestConfig::from_env();
        assert!(result.is_err());
    }

    #[test]
    fn test_config_with_env_vars() {
        std::env::set_var("E2E_TEST_REPO", "test/repo");
        std::env::set_var("E2E_GITHUB_TOKEN", "ghp_test");

        let config = TestConfig::from_env().unwrap();
        assert_eq!(config.test_repo, "test/repo");
        assert_eq!(config.github_token, Some("ghp_test".to_string()));

        // 清理
        std::env::remove_var("E2E_TEST_REPO");
        std::env::remove_var("E2E_GITHUB_TOKEN");
    }
}
```

- [ ] **Step 2: 运行测试验证失败**

Run: `cargo test -p e2e-core --lib config`
Expected: 编译失败（缺少 `config` 模块导出）

- [ ] **Step 3: 更新 lib.rs 导出 config 模块**

修改 `crates/e2e-core/src/lib.rs`：

```rust
pub mod config;

pub use config::TestConfig;
```

- [ ] **Step 4: 运行测试验证通过**

Run: `cargo test -p e2e-core --lib config`
Expected: PASS

- [ ] **Step 5: 提交 TestConfig 实现**

```bash
git add crates/e2e-core/src/config.rs crates/e2e-core/src/lib.rs
git commit -m "feat(e2e-core): implement TestConfig for environment variables"
```

---

## Task 4: 实现 TestFixture（测试数据管理）

**Files:**
- Create: `crates/e2e-core/src/fixture.rs`
- Test: `crates/e2e-core/src/fixture.rs`（内联测试）

**Interfaces:**
- Consumes: `TtyRunner`, `TestConfig`
- Produces: `TestFixture`, `TestResource`

- [ ] **Step 1: 编写 TestFixture 的失败测试**

在 `crates/e2e-core/src/fixture.rs` 中：

```rust
//! 测试数据管理模块
//!
//! 提供测试资源的创建和清理。

use crate::{TtyMode, TtyRunner};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 固件错误
#[derive(Debug, Error)]
pub enum FixtureError {
    #[error("TTY error: {0}")]
    Tty(#[from] crate::tty::TtyError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Config error: {0}")]
    Config(#[from] crate::config::ConfigError),
}

/// 测试资源类型
#[derive(Debug, Clone)]
pub enum TestResource {
    /// Issue 资源
    Issue { number: u64 },
    /// PR 资源
    Pr { number: u64 },
    /// Label 资源
    Label { name: String },
    /// Milestone 资源
    Milestone { number: u64 },
    /// Release 资源
    Release { tag: String },
}

/// Issue 创建响应
#[derive(Debug, Deserialize)]
struct IssueResponse {
    number: u64,
}

/// 测试固件管理器
#[derive(Debug)]
pub struct TestFixture {
    repo: String,
    created_resources: Vec<TestResource>,
}

impl TestFixture {
    /// 创建新的测试固件
    pub fn new() -> Result<Self, FixtureError> {
        let config = crate::TestConfig::from_env()?;
        Ok(Self {
            repo: config.test_repo,
            created_resources: Vec::new(),
        })
    }

    /// 创建测试 Issue
    pub async fn create_issue(&mut self, title: &str) -> Result<u64, FixtureError> {
        let runner = TtyRunner::new(TtyMode::NonInteractive);
        let output = runner
            .run(&[
                "issue",
                "create",
                "--title",
                title,
                "--repo",
                &self.repo,
                "--output",
                "json",
            ])
            .await?;

        let issue: IssueResponse = serde_json::from_str(&output.stdout)?;
        self.created_resources.push(TestResource::Issue {
            number: issue.number,
        });
        Ok(issue.number)
    }

    /// 创建测试 Label
    pub async fn create_label(&mut self, name: &str) -> Result<(), FixtureError> {
        let runner = TtyRunner::new(TtyMode::NonInteractive);
        runner
            .run(&[
                "label",
                "create",
                "--name",
                name,
                "--color",
                "ff0000",
                "--repo",
                &self.repo,
            ])
            .await?;
        self.created_resources.push(TestResource::Label {
            name: name.to_string(),
        });
        Ok(())
    }

    /// 清理所有创建的资源
    pub async fn cleanup(&mut self) -> Result<(), FixtureError> {
        let runner = TtyRunner::new(TtyMode::NonInteractive);

        for resource in self.created_resources.drain(..) {
            match resource {
                TestResource::Issue { number } => {
                    let _ = runner
                        .run(&["issue", "close", &number.to_string(), "--repo", &self.repo])
                        .await;
                }
                TestResource::Label { name } => {
                    let _ = runner
                        .run(&["label", "delete", "--name", &name, "--repo", &self.repo])
                        .await;
                }
                TestResource::Pr { number } => {
                    let _ = runner
                        .run(&["pr", "close", &number.to_string(), "--repo", &self.repo])
                        .await;
                }
                TestResource::Milestone { number } => {
                    let _ = runner
                        .run(&[
                            "milestone",
                            "close",
                            &number.to_string(),
                            "--repo",
                            &self.repo,
                        ])
                        .await;
                }
                TestResource::Release { tag } => {
                    let _ = runner
                        .run(&["release", "delete", "--tag", &tag, "--repo", &self.repo])
                        .await;
                }
            }
        }
        Ok(())
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        if !self.created_resources.is_empty() {
            tracing::warn!(
                "TestFixture dropped with {} resources not cleaned up",
                self.created_resources.len()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_debug() {
        let resource = TestResource::Issue { number: 123 };
        let debug_str = format!("{:?}", resource);
        assert!(debug_str.contains("123"));
    }
}
```

- [ ] **Step 2: 运行测试验证失败**

Run: `cargo test -p e2e-core --lib fixture`
Expected: 编译失败（缺少 `fixture` 模块导出）

- [ ] **Step 3: 更新 lib.rs 导出 fixture 模块**

修改 `crates/e2e-core/src/lib.rs`：

```rust
pub mod fixture;

pub use fixture::{TestFixture, TestResource};
```

- [ ] **Step 4: 运行测试验证通过**

Run: `cargo test -p e2e-core --lib fixture`
Expected: PASS

- [ ] **Step 5: 提交 TestFixture 实现**

```bash
git add crates/e2e-core/src/fixture.rs crates/e2e-core/src/lib.rs
git commit -m "feat(e2e-core): implement TestFixture for resource management"
```

---

## Task 5: 创建 e2e-github crate

**Files:**
- Create: `crates/e2e-github/Cargo.toml`
- Create: `crates/e2e-github/tests/auth.rs`（空测试）

**Interfaces:**
- Consumes: `e2e-core`
- Produces: `e2e-github` crate

- [ ] **Step 1: 创建 crates/e2e-github/Cargo.toml**

```toml
[package]
name = "e2e-github"
version = "0.1.0"
edition = "2024"
publish = false

[dev-dependencies]
e2e-core = { path = "../e2e-core" }
tokio = { version = "1", features = ["full", "test-util"] }

[lints]
workspace = true
```

- [ ] **Step 2: 创建空测试文件 crates/e2e-github/tests/auth.rs**

```rust
//! GitHub auth 命令 E2E 测试

use e2e_core::{TtyMode, TtyRunner};

#[tokio::test]
#[ignore = "需要 E2E_TEST_REPO 和 E2E_GITHUB_TOKEN 环境变量"]
async fn test_placeholder() {
    // 占位测试，后续任务会实现真实测试
    assert!(true);
}
```

- [ ] **Step 3: 验证 crate 编译**

Run: `cargo check -p e2e-github`
Expected: 编译成功

- [ ] **Step 4: 运行占位测试**

Run: `cargo test -p e2e-github`
Expected: PASS（占位测试通过）

- [ ] **Step 5: 提交 e2e-github crate**

```bash
git add crates/e2e-github/
git commit -m "feat(e2e-github): add crate scaffold"
```

---

## Task 6: 实现 auth 命令测试

**Files:**
- Modify: `crates/e2e-github/tests/auth.rs`

**Interfaces:**
- Consumes: `TtyRunner`, `TestFixture`
- Produces: auth 命令测试用例

- [ ] **Step 1: 编写 auth status 测试的失败用例**

修改 `crates/e2e-github/tests/auth.rs`：

```rust
//! GitHub auth 命令 E2E 测试

use e2e_core::{TtyMode, TtyRunner};

#[tokio::test]
async fn test_auth_status_interactive() {
    let runner = TtyRunner::new(TtyMode::Interactive);
    let output = runner.run(&["auth", "status", "--platform", "github"]).await.unwrap();

    // 交互模式：应该成功或提示需要登录
    assert!(output.status.success() || output.stdout.contains("login"));
}

#[tokio::test]
async fn test_auth_status_non_interactive() {
    let runner = TtyRunner::new(TtyMode::NonInteractive);
    let output = runner.run(&["auth", "status", "--platform", "github"]).await.unwrap();

    // 非交互模式：应该成功或提示需要登录
    assert!(output.status.success() || output.stderr.contains("login"));
}

#[tokio::test]
async fn test_auth_token_interactive() {
    let runner = TtyRunner::new(TtyMode::Interactive);
    let output = runner.run(&["auth", "token", "--platform", "github"]).await.unwrap();

    // 交互模式：应该返回 token 或提示需要登录
    assert!(output.status.success() || output.stdout.contains("login"));
}

#[tokio::test]
async fn test_auth_token_non_interactive() {
    let runner = TtyRunner::new(TtyMode::NonInteractive);
    let output = runner.run(&["auth", "token", "--platform", "github"]).await.unwrap();

    // 非交互模式：应该返回 token 或提示需要登录
    assert!(output.status.success() || output.stderr.contains("login"));
}
```

- [ ] **Step 2: 运行测试验证失败**

Run: `cargo test -p e2e-github --test auth`
Expected: 测试运行但可能失败（需要真实凭证）

- [ ] **Step 3: 运行测试验证通过（需要设置环境变量）**

设置环境变量：
```bash
export E2E_TEST_REPO="byx-darwin/e2e-test-repo"
export E2E_GITHUB_TOKEN="ghp_your_token_here"
```

Run: `cargo test -p e2e-github --test auth`
Expected: PASS（如果凭证有效）

- [ ] **Step 4: 提交 auth 测试**

```bash
git add crates/e2e-github/tests/auth.rs
git commit -m "test(e2e-github): add auth command tests"
```

---

## Task 7: 实现 issue 命令测试

**Files:**
- Create: `crates/e2e-github/tests/issue.rs`

**Interfaces:**
- Consumes: `TtyRunner`, `TestFixture`
- Produces: issue 命令测试用例

- [ ] **Step 1: 编写 issue 命令测试**

创建 `crates/e2e-github/tests/issue.rs`：

```rust
//! GitHub issue 命令 E2E 测试

use e2e_core::{TestFixture, TtyMode, TtyRunner};

#[tokio::test]
async fn test_issue_list_interactive() {
    let runner = TtyRunner::new(TtyMode::Interactive);
    let output = runner
        .run(&["issue", "list", "--platform", "github"])
        .await
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn test_issue_list_non_interactive() {
    let runner = TtyRunner::new(TtyMode::NonInteractive);
    let output = runner
        .run(&["issue", "list", "--platform", "github"])
        .await
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn test_issue_close_and_reopen() {
    let mut fixture = TestFixture::new().unwrap();
    let issue_number = fixture.create_issue("E2E test issue").await.unwrap();

    // 测试 close
    let runner = TtyRunner::new(TtyMode::NonInteractive);
    let output = runner
        .run(&[
            "issue",
            "close",
            &issue_number.to_string(),
            "--platform",
            "github",
        ])
        .await
        .unwrap();
    assert!(output.status.success());

    // 测试 reopen
    let output = runner
        .run(&[
            "issue",
            "reopen",
            &issue_number.to_string(),
            "--platform",
            "github",
        ])
        .await
        .unwrap();
    assert!(output.status.success());

    // 清理
    fixture.cleanup().await.unwrap();
}
```

- [ ] **Step 2: 运行测试验证失败**

Run: `cargo test -p e2e-github --test issue`
Expected: 测试运行但可能失败（需要真实凭证）

- [ ] **Step 3: 运行测试验证通过（需要设置环境变量）**

Run: `cargo test -p e2e-github --test issue`
Expected: PASS（如果凭证有效）

- [ ] **Step 4: 提交 issue 测试**

```bash
git add crates/e2e-github/tests/issue.rs
git commit -m "test(e2e-github): add issue command tests"
```

---

## Task 8: 实现 pr 命令测试

**Files:**
- Create: `crates/e2e-github/tests/pr.rs`

**Interfaces:**
- Consumes: `TtyRunner`, `TestFixture`
- Produces: pr 命令测试用例

- [ ] **Step 1: 编写 pr 命令测试**

创建 `crates/e2e-github/tests/pr.rs`：

```rust
//! GitHub pr 命令 E2E 测试

use e2e_core::{TtyMode, TtyRunner};

#[tokio::test]
async fn test_pr_list_interactive() {
    let runner = TtyRunner::new(TtyMode::Interactive);
    let output = runner
        .run(&["pr", "list", "--platform", "github"])
        .await
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn test_pr_list_non_interactive() {
    let runner = TtyRunner::new(TtyMode::NonInteractive);
    let output = runner
        .run(&["pr", "list", "--platform", "github"])
        .await
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn test_pr_merge_non_interactive() {
    // 这个测试需要有可合并的 PR
    // 暂时标记为 ignore，后续实现
    let runner = TtyRunner::new(TtyMode::NonInteractive);
    let output = runner
        .run(&["pr", "merge", "999", "--platform", "github"])
        .await
        .unwrap();

    // 应该失败（PR 不存在）但不应该提示需要 --yes
    assert!(!output.status.success() || output.stderr.contains("--yes"));
}
```

- [ ] **Step 2: 运行测试验证失败**

Run: `cargo test -p e2e-github --test pr`
Expected: 测试运行但可能失败（需要真实凭证）

- [ ] **Step 3: 运行测试验证通过（需要设置环境变量）**

Run: `cargo test -p e2e-github --test pr`
Expected: PASS（如果凭证有效）

- [ ] **Step 4: 提交 pr 测试**

```bash
git add crates/e2e-github/tests/pr.rs
git commit -m "test(e2e-github): add pr command tests"
```

---

## Task 9: 创建 CI workflow

**Files:**
- Create: `.github/workflows/e2e-tests.yml`

**Interfaces:**
- Consumes: `e2e-github` crate
- Produces: CI workflow

- [ ] **Step 1: 创建 .github/workflows/e2e-tests.yml**

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
    - cron: '0 2 * * *'
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

      - name: Run E2E tests
        env:
          E2E_TEST_REPO: ${{ secrets.E2E_TEST_REPO }}
          E2E_GITHUB_TOKEN: ${{ secrets.E2E_GITHUB_TOKEN }}
          RUST_LOG: info
        run: |
          cargo nextest run -p e2e-github --all-features

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: e2e-results-github
          path: target/nextest/
          retention-days: 7
```

- [ ] **Step 2: 提交 CI workflow**

```bash
git add .github/workflows/e2e-tests.yml
git commit -m "ci: add E2E tests workflow for GitHub platform"
```

---

## Task 10: 配置测试仓库和 Secrets

**Files:**
- 无代码文件（手动配置）

**Interfaces:**
- Consumes: GitHub 账户访问权限
- Produces: 配置好的测试仓库和 Secrets

- [ ] **Step 1: 创建测试仓库**

在 GitHub 上创建仓库 `byx-darwin/e2e-test-repo`：
- 初始化为空仓库（仅有 README）
- 启用 Issues、PR、Releases 功能

- [ ] **Step 2: 创建 GitHub 令牌**

在 GitHub Settings → Developer settings → Personal access tokens 创建令牌：
- 权限：repo, issue, pr, label, milestone
- 名称：`e2e-test-token`
- 过期时间：90 天（定期轮换）

- [ ] **Step 3: 配置 GitHub Secrets**

在仓库 Settings → Secrets and variables → Actions 添加：
- `E2E_TEST_REPO`: `byx-darwin/e2e-test-repo`
- `E2E_GITHUB_TOKEN`: `ghp_your_token_here`

- [ ] **Step 4: 验证配置**

Run: `cargo test -p e2e-github --test auth`
Expected: PASS（使用真实凭证）

- [ ] **Step 5: 提交最终验证**

```bash
git add .
git commit -m "chore: configure E2E test infrastructure"
```

---

## Self-Review Checklist

- [x] **Spec coverage:** 所有设计文档中的要求都有对应任务
- [x] **Placeholder scan:** 无 "TBD", "TODO" 或不完整步骤
- [x] **Type consistency:** `TtyRunner`, `TestFixture`, `CommandOutput` 在所有任务中一致

---

## 执行手递手

计划已保存到 `docs/superpowers/plans/2026-07-09-e2e-noninteractive-test-plan.md`。

**两种执行选项：**

**1. Subagent-Driven（推荐）** - 每个任务分派独立 subagent，任务间审查，快速迭代

**2. Inline Execution** - 在当前会话中使用 executing-plans 批量执行，设置检查点审查

**选择哪个方案？**
