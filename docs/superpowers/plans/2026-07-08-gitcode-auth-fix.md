# GitCode 认证状态检查修复与重构实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复 GitCode 平台认证检查 bug，统一认证检查接口，增强错误信息和调试日志。

**Architecture:** 新增 `AuthChecker` trait（同步版本）用于前置检查，各平台实现该 trait。重构 `prerequisites.rs` 使用 `AuthChecker` 替代分散的认证逻辑。

**Tech Stack:** Rust 2024, thiserror, temp-env (测试), assert_cmd (集成测试)

## Global Constraints

- Rust 2024 edition, toolchain 1.96.0+
- 代码覆盖率 > 80%
- 所有公共 API 需要文档注释
- 遵循项目现有代码风格（snake_case, Result<T> 返回）
- 不使用 `unwrap()` 或 `expect()`，使用 `thiserror` 错误处理
- TDD 循环：RED → GREEN → REFACTOR

---

## 文件结构

### 新增文件
- `crates/core/src/auth_checker.rs` — `AuthChecker` trait 和 `AuthCheckResult` 定义
- `apps/cli/tests/prerequisites_integration.rs` — 前置检查集成测试

### 修改文件
- `crates/core/src/lib.rs` — 导出 `AuthChecker` 和 `AuthCheckResult`
- `crates/gitcode/src/auth.rs` — 实现 `AuthChecker` for `GitCodeAuthProvider`
- `crates/github/src/auth.rs` — 实现 `AuthChecker` for `GitHubAuthProvider`
- `crates/gitlab/src/auth.rs` — 实现 `AuthChecker` for `GitLabAuthProvider`
- `apps/cli/src/commands/prerequisites.rs` — 重构使用 `AuthChecker`，删除 `is_authenticated` 函数
- `Cargo.toml` (workspace) — 添加 `temp-env` 依赖（如未存在）

---

## Task 1: 定义 AuthChecker trait 和 AuthCheckResult 结构体

**Files:**
- Create: `crates/core/src/auth_checker.rs`
- Modify: `crates/core/src/lib.rs:1-50`
- Test: `crates/core/src/auth_checker.rs:80-120`

**Interfaces:**
- Consumes: 无（首个任务）
- Produces:
  - `pub trait AuthChecker { fn is_authenticated(&self) -> bool; fn check_status(&self) -> AuthCheckResult; }`
  - `pub struct AuthCheckResult { pub authenticated: bool, pub user: Option<String>, pub reason: Option<String>, pub hint: Option<String> }`

---

- [ ] **Step 1: 写失败测试**

创建 `crates/core/src/auth_checker.rs`，在文件末尾添加测试模块：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_create_authenticated_result() {
        let result = AuthCheckResult {
            authenticated: true,
            user: Some("testuser".to_string()),
            reason: None,
            hint: None,
        };
        assert!(result.authenticated);
        assert_eq!(result.user, Some("testuser".to_string()));
    }

    #[test]
    fn test_should_create_not_authenticated_result() {
        let result = AuthCheckResult {
            authenticated: false,
            user: None,
            reason: Some("Not logged in".to_string()),
            hint: Some("Run `gitcode auth login`".to_string()),
        };
        assert!(!result.authenticated);
        assert!(result.reason.is_some());
        assert!(result.hint.is_some());
    }

    #[test]
    fn test_auth_checker_trait_is_object_safe() {
        // 验证 trait 是 object-safe 的
        fn _takes_checker(_checker: &dyn AuthChecker) {}
    }
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p gitflow-cli-core --lib auth_checker::tests --no-run`
Expected: FAIL with "AuthCheckResult not found", "AuthChecker not found"

- [ ] **Step 3: 写最小实现**

在 `crates/core/src/auth_checker.rs` 文件开头添加定义：

```rust
//! 认证检查器 trait 和类型定义。
//!
//! 提供同步版本的认证检查接口，用于 CLI 前置检查。

/// 认证检查器 trait（同步版本，用于前置检查）。
///
/// # Examples
///
/// ```ignore
/// use gitflow_cli_core::AuthChecker;
///
/// fn check_auth(checker: &dyn AuthChecker) {
///     if checker.is_authenticated() {
///         println!("User is authenticated");
///     }
/// }
/// ```
pub trait AuthChecker: Send + Sync {
    /// 快速检查是否已认证（不查询 API，仅检查本地凭据）。
    ///
    /// # Returns
    ///
    /// 如果已认证返回 `true`，否则返回 `false`。
    fn is_authenticated(&self) -> bool;

    /// 获取认证检查的详细状态。
    ///
    /// # Returns
    ///
    /// 包含认证状态、用户名、失败原因和修复建议的 [`AuthCheckResult`]。
    fn check_status(&self) -> AuthCheckResult;
}

/// 认证检查结果。
///
/// 包含认证状态的详细信息，用于生成用户友好的错误消息。
#[derive(Debug, Clone)]
pub struct AuthCheckResult {
    /// 是否已认证。
    pub authenticated: bool,
    /// 用户名（如果已认证）。
    pub user: Option<String>,
    /// 失败原因（如果未认证）。
    pub reason: Option<String>,
    /// 修复建议。
    pub hint: Option<String>,
}
```

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo test -p gitflow-cli-core --lib auth_checker::tests`
Expected: PASS (3 tests)

- [ ] **Step 5: 在 lib.rs 中导出**

修改 `crates/core/src/lib.rs`，在 `pub mod auth;` 之后添加：

```rust
pub mod auth_checker;
pub use auth_checker::{AuthCheckResult, AuthChecker};
```

- [ ] **Step 6: 运行完整测试确认无破坏**

Run: `cargo test -p gitflow-cli-core`
Expected: PASS (所有现有测试 + 新增 3 个测试)

- [ ] **Step 7: 提交**

```bash
git add crates/core/src/auth_checker.rs crates/core/src/lib.rs
git commit -m "feat(core): add AuthChecker trait and AuthCheckResult struct

Define synchronous authentication checker interface for CLI prerequisites.
Includes comprehensive unit tests and documentation.

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 2: 实现 GitCode 的 AuthChecker

**Files:**
- Modify: `crates/gitcode/src/auth.rs:1-225`
- Test: `crates/gitcode/src/auth.rs:225-350`

**Interfaces:**
- Consumes: `AuthChecker` trait, `AuthCheckResult` from `gitflow_cli_core`
- Produces: `impl AuthChecker for GitCodeAuthProvider`

---

- [ ] **Step 1: 写失败测试**

在 `crates/gitcode/src/auth.rs` 的 `mod tests` 模块中添加测试：

```rust
#[test]
fn test_should_parse_user_from_status_new_format() {
    let status = "Logged in as alice";
    assert_eq!(parse_user_from_status(status), Some("alice".to_string()));
}

#[test]
fn test_should_parse_user_from_status_new_format_with_suffix() {
    let status = "Logged in as bob (oauth_token)";
    assert_eq!(parse_user_from_status(status), Some("bob".to_string()));
}

#[test]
fn test_auth_checker_is_authenticated_with_env_var() {
    use crate::auth::AuthChecker;
    temp_env::with_var("GITCODE_TOKEN", Some("test_token"), || {
        let provider = GitCodeAuthProvider::new();
        assert!(provider.is_authenticated());
    });
}

#[test]
fn test_auth_checker_check_status_with_env_var() {
    use crate::auth::AuthChecker;
    temp_env::with_var("GITCODE_TOKEN", Some("test_token"), || {
        let provider = GitCodeAuthProvider::new();
        let result = provider.check_status();
        assert!(result.authenticated);
        assert!(result.reason.is_none());
    });
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p gitflow-cli-gitcode --lib auth::tests::test_should_parse_user_from_status_new_format`
Expected: FAIL with "AuthChecker not found" or test not found

- [ ] **Step 3: 实现 AuthChecker for GitCodeAuthProvider**

在 `crates/gitcode/src/auth.rs` 中，在 `impl AuthProvider for GitCodeAuthProvider` 之后添加：

```rust
impl gitflow_cli_core::AuthChecker for GitCodeAuthProvider {
    fn is_authenticated(&self) -> bool {
        // 1. 优先检查环境变量
        if std::env::var("GITCODE_TOKEN").is_ok() {
            return true;
        }

        // 2. 检查 gitcode CLI 是否可用
        let binary = crate::gitcode_binary();
        tracing::debug!(binary = %binary, "checking gitcode authentication");

        let output = std::process::Command::new(&binary)
            .args(["auth", "status"])
            .output();

        match output {
            Ok(out) => {
                tracing::debug!(
                    exit_code = %out.status,
                    stdout = %String::from_utf8_lossy(&out.stdout),
                    stderr = %String::from_utf8_lossy(&out.stderr),
                    "gitcode auth status result"
                );
                out.status.success()
            }
            Err(e) => {
                tracing::debug!(error = %e, "failed to execute gitcode auth status");
                false
            }
        }
    }

    fn check_status(&self) -> gitflow_cli_core::AuthCheckResult {
        // 1. 检查环境变量
        if std::env::var("GITCODE_TOKEN").is_ok() {
            return gitflow_cli_core::AuthCheckResult {
                authenticated: true,
                user: None,
                reason: None,
                hint: None,
            };
        }

        // 2. 执行 gitcode auth status
        let binary = crate::gitcode_binary();
        let output = match std::process::Command::new(&binary)
            .args(["auth", "status"])
            .output()
        {
            Ok(out) => out,
            Err(e) => {
                return gitflow_cli_core::AuthCheckResult {
                    authenticated: false,
                    user: None,
                    reason: Some(format!("Failed to execute gitcode: {e}")),
                    hint: Some("Ensure gitcode CLI is installed: pip install gitcode-cli".into()),
                };
            }
        };

        // 3. 解析结果
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let user = parse_user_from_status(&stdout);

            gitflow_cli_core::AuthCheckResult {
                authenticated: true,
                user,
                reason: None,
                hint: None,
            }
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);

            gitflow_cli_core::AuthCheckResult {
                authenticated: false,
                user: None,
                reason: Some(stderr.to_string()),
                hint: Some("Run `gitcode auth login` to authenticate".into()),
            }
        }
    }
}
```

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo test -p gitflow-cli-gitcode --lib auth::tests`
Expected: PASS (所有测试，包括新增的 4 个)

- [ ] **Step 5: 更新注释说明支持两种格式**

修改 `parse_user_from_status` 函数的注释（第 202 行附近）：

```rust
/// 从 `gc auth status` 的输出中解析用户名。
///
/// 支持两种格式：
/// - 旧格式: `"Logged in to gitcode.com as <username>"`
/// - 新格式: `"Logged in as <username>"`
fn parse_user_from_status(output: &str) -> Option<String> {
    // 匹配模式：查找 " as " 后跟用户名
    for line in output.lines() {
        if let Some(pos) = line.find(" as ") {
            let after_as = &line[pos + 4..];
            // 用户名后面可能跟 " (" 或空格或其他
            if let Some(end) = after_as.find(' ') {
                let user = &after_as[..end];
                if !user.is_empty() {
                    return Some(user.to_string());
                }
            } else if !after_as.is_empty() {
                return Some(after_as.trim().to_string());
            }
        }
    }
    None
}
```

- [ ] **Step 6: 运行 clippy 检查**

Run: `cargo clippy -p gitflow-cli-gitcode --all-targets -- -D warnings`
Expected: PASS (无警告)

- [ ] **Step 7: 提交**

```bash
git add crates/gitcode/src/auth.rs
git commit -m "feat(gitcode): implement AuthChecker for GitCodeAuthProvider

- Add is_authenticated() with env var and CLI check
- Add check_status() with detailed error info
- Support both old and new gitcode CLI output formats
- Add comprehensive unit tests

Fixes authentication check bug where wrong binary path was used.

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 3: 实现 GitHub 的 AuthChecker

**Files:**
- Modify: `crates/github/src/auth.rs:1-150`
- Test: `crates/github/src/auth.rs:150-250`

**Interfaces:**
- Consumes: `AuthChecker` trait, `AuthCheckResult` from `gitflow_cli_core`
- Produces: `impl AuthChecker for GitHubAuthProvider`

---

- [ ] **Step 1: 写失败测试**

在 `crates/github/src/auth.rs` 的 `mod tests` 模块中添加：

```rust
#[test]
fn test_auth_checker_is_authenticated_with_env_var() {
    use crate::auth::AuthChecker;
    temp_env::with_var("GH_TOKEN", Some("test_token"), || {
        let provider = GitHubAuthProvider::new();
        assert!(provider.is_authenticated());
    });
}

#[test]
fn test_auth_checker_check_status_with_env_var() {
    use crate::auth::AuthChecker;
    temp_env::with_var("GH_TOKEN", Some("test_token"), || {
        let provider = GitHubAuthProvider::new();
        let result = provider.check_status();
        assert!(result.authenticated);
        assert!(result.reason.is_none());
    });
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p gitflow-cli-github --lib auth::tests::test_auth_checker_is_authenticated_with_env_var`
Expected: FAIL with "AuthChecker not found"

- [ ] **Step 3: 实现 AuthChecker for GitHubAuthProvider**

在 `crates/github/src/auth.rs` 中，在 `impl AuthProvider for GitHubAuthProvider` 之后添加：

```rust
impl gitflow_cli_core::AuthChecker for GitHubAuthProvider {
    fn is_authenticated(&self) -> bool {
        if std::env::var("GH_TOKEN").is_ok() {
            return true;
        }

        let output = std::process::Command::new("gh")
            .args(["auth", "status"])
            .output();

        matches!(output, Ok(out) if out.status.success())
    }

    fn check_status(&self) -> gitflow_cli_core::AuthCheckResult {
        // 1. 检查环境变量
        if std::env::var("GH_TOKEN").is_ok() {
            return gitflow_cli_core::AuthCheckResult {
                authenticated: true,
                user: None,
                reason: None,
                hint: None,
            };
        }

        // 2. 执行 gh auth status
        let output = match std::process::Command::new("gh")
            .args(["auth", "status"])
            .output()
        {
            Ok(out) => out,
            Err(e) => {
                return gitflow_cli_core::AuthCheckResult {
                    authenticated: false,
                    user: None,
                    reason: Some(format!("Failed to execute gh: {e}")),
                    hint: Some("Install GitHub CLI: https://cli.github.com".into()),
                };
            }
        };

        // 3. 解析结果
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let user = parse_github_user_from_status(&stdout);

            gitflow_cli_core::AuthCheckResult {
                authenticated: true,
                user,
                reason: None,
                hint: None,
            }
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);

            gitflow_cli_core::AuthCheckResult {
                authenticated: false,
                user: None,
                reason: Some(stderr.to_string()),
                hint: Some("Run `gh auth login` to authenticate".into()),
            }
        }
    }
}

/// 从 `gh auth status` 的输出中解析用户名。
fn parse_github_user_from_status(output: &str) -> Option<String> {
    for line in output.lines() {
        if let Some(pos) = line.find(" as ") {
            let after_as = &line[pos + 4..];
            if let Some(end) = after_as.find(' ') {
                let user = &after_as[..end];
                if !user.is_empty() {
                    return Some(user.to_string());
                }
            } else if !after_as.is_empty() {
                return Some(after_as.trim().to_string());
            }
        }
    }
    None
}
```

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo test -p gitflow-cli-github --lib auth::tests`
Expected: PASS (所有测试)

- [ ] **Step 5: 运行 clippy 检查**

Run: `cargo clippy -p gitflow-cli-github --all-targets -- -D warnings`
Expected: PASS

- [ ] **Step 6: 提交**

```bash
git add crates/github/src/auth.rs
git commit -m "feat(github): implement AuthChecker for GitHubAuthProvider

- Add is_authenticated() with env var and CLI check
- Add check_status() with detailed error info
- Add parse_github_user_from_status() helper
- Add unit tests for env var authentication

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 4: 实现 GitLab 的 AuthChecker

**Files:**
- Modify: `crates/gitlab/src/auth.rs:1-150`
- Test: `crates/gitlab/src/auth.rs:150-250`

**Interfaces:**
- Consumes: `AuthChecker` trait, `AuthCheckResult` from `gitflow_cli_core`
- Produces: `impl AuthChecker for GitLabAuthProvider`

---

- [ ] **Step 1: 写失败测试**

在 `crates/gitlab/src/auth.rs` 的 `mod tests` 模块中添加：

```rust
#[test]
fn test_auth_checker_is_authenticated_with_env_var() {
    use crate::auth::AuthChecker;
    temp_env::with_var("GL_TOKEN", Some("test_token"), || {
        let provider = GitLabAuthProvider::new();
        assert!(provider.is_authenticated());
    });
}

#[test]
fn test_auth_checker_check_status_with_env_var() {
    use crate::auth::AuthChecker;
    temp_env::with_var("GL_TOKEN", Some("test_token"), || {
        let provider = GitLabAuthProvider::new();
        let result = provider.check_status();
        assert!(result.authenticated);
        assert!(result.reason.is_none());
    });
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p gitflow-cli-gitlab --lib auth::tests::test_auth_checker_is_authenticated_with_env_var`
Expected: FAIL with "AuthChecker not found"

- [ ] **Step 3: 实现 AuthChecker for GitLabAuthProvider**

在 `crates/gitlab/src/auth.rs` 中，在 `impl AuthProvider for GitLabAuthProvider` 之后添加：

```rust
impl gitflow_cli_core::AuthChecker for GitLabAuthProvider {
    fn is_authenticated(&self) -> bool {
        if std::env::var("GL_TOKEN").is_ok() {
            return true;
        }

        let output = std::process::Command::new("glab")
            .args(["auth", "status"])
            .output();

        matches!(output, Ok(out) if out.status.success())
    }

    fn check_status(&self) -> gitflow_cli_core::AuthCheckResult {
        // 1. 检查环境变量
        if std::env::var("GL_TOKEN").is_ok() {
            return gitflow_cli_core::AuthCheckResult {
                authenticated: true,
                user: None,
                reason: None,
                hint: None,
            };
        }

        // 2. 执行 glab auth status
        let output = match std::process::Command::new("glab")
            .args(["auth", "status"])
            .output()
        {
            Ok(out) => out,
            Err(e) => {
                return gitflow_cli_core::AuthCheckResult {
                    authenticated: false,
                    user: None,
                    reason: Some(format!("Failed to execute glab: {e}")),
                    hint: Some("Install GitLab CLI: brew install glab".into()),
                };
            }
        };

        // 3. 解析结果
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let user = parse_gitlab_user_from_status(&stdout);

            gitflow_cli_core::AuthCheckResult {
                authenticated: true,
                user,
                reason: None,
                hint: None,
            }
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);

            gitflow_cli_core::AuthCheckResult {
                authenticated: false,
                user: None,
                reason: Some(stderr.to_string()),
                hint: Some("Run `glab auth login` to authenticate".into()),
            }
        }
    }
}

/// 从 `glab auth status` 的输出中解析用户名。
fn parse_gitlab_user_from_status(output: &str) -> Option<String> {
    for line in output.lines() {
        if let Some(pos) = line.find(" as ") {
            let after_as = &line[pos + 4..];
            if let Some(end) = after_as.find(' ') {
                let user = &after_as[..end];
                if !user.is_empty() {
                    return Some(user.to_string());
                }
            } else if !after_as.is_empty() {
                return Some(after_as.trim().to_string());
            }
        }
    }
    None
}
```

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo test -p gitflow-cli-gitlab --lib auth::tests`
Expected: PASS (所有测试)

- [ ] **Step 5: 运行 clippy 检查**

Run: `cargo clippy -p gitflow-cli-gitlab --all-targets -- -D warnings`
Expected: PASS

- [ ] **Step 6: 提交**

```bash
git add crates/gitlab/src/auth.rs
git commit -m "feat(gitlab): implement AuthChecker for GitLabAuthProvider

- Add is_authenticated() with env var and CLI check
- Add check_status() with detailed error info
- Add parse_gitlab_user_from_status() helper
- Add unit tests for env var authentication

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 5: 重构 prerequisites.rs 使用 AuthChecker

**Files:**
- Modify: `apps/cli/src/commands/prerequisites.rs:138-210`
- Delete: `apps/cli/src/commands/prerequisites.rs:193-210` (is_authenticated 函数)

**Interfaces:**
- Consumes: `AuthChecker` trait, `AuthCheckResult` from `gitflow_cli_core`
- Produces: 重构后的 `check()` 函数，使用 `AuthChecker`

---

- [ ] **Step 1: 写失败测试**

在 `apps/cli/src/commands/prerequisites.rs` 的 `mod tests` 模块中添加：

```rust
#[test]
fn test_prerequisites_check_fails_with_clear_error_when_not_authenticated() {
    // 清除环境变量，确保未认证
    temp_env::with_var_unset("GITCODE_TOKEN", || {
        // Mock gitcode CLI 不存在（使用空 PATH）
        let temp_dir = tempfile::tempdir().unwrap();
        let original_path = std::env::var("PATH").unwrap_or_default();
        std::env::set_var("PATH", temp_dir.path());

        let result = super::check("gitcode");

        // 恢复 PATH
        std::env::set_var("PATH", original_path);

        // 应该返回 NotFound 错误（因为 CLI 不存在）
        assert!(result.is_err());
    });
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p gitflow-cli --bin gitflow-cli prerequisites::tests::test_prerequisites_check_fails_with_clear_error_when_not_authenticated`
Expected: FAIL (测试可能通过，但我们需要重构代码)

- [ ] **Step 3: 重构 check() 函数**

修改 `apps/cli/src/commands/prerequisites.rs` 的 `check()` 函数（第 138-191 行）：

```rust
/// 前置条件检查。
///
/// # Errors
///
/// 当 CLI 未安装、版本过低或未认证时返回错误。
pub fn check(platform: &str) -> Result<(), PrerequisiteError> {
    let requirement = requirement_for(platform).ok_or_else(|| {
        PrerequisiteError::UnsupportedPlatform {
            platform: platform.into(),
        }
    })?;

    // 1. 查找 CLI
    let path = find_cli_binary(requirement.binary)?;

    // 2. 检查版本
    let version = get_version(&path)?;
    check_version(&version, requirement.min_version)?;

    // 3. 认证检查（使用 AuthChecker）
    let auth_checker = create_auth_checker(platform);
    if !auth_checker.is_authenticated() {
        let result = auth_checker.check_status();
        return Err(PrerequisiteError::NotAuthenticated {
            binary: requirement.binary.into(),
            platform: platform.into(),
            reason: result.reason.unwrap_or_else(|| "Unknown reason".into()),
            hint: result.hint.unwrap_or_else(|| requirement.login_cmd.into()),
        });
    }

    Ok(())
}

/// 创建平台特定的认证检查器。
fn create_auth_checker(platform: &str) -> Box<dyn gitflow_cli_core::AuthChecker> {
    match platform {
        "github" => Box::new(gitflow_cli_github::GitHubAuthProvider::new()),
        "gitlab" => Box::new(gitflow_cli_gitlab::GitLabAuthProvider::new()),
        "gitcode" => Box::new(gitflow_cli_gitcode::GitCodeAuthProvider::new()),
        _ => unreachable!("Platform already validated by requirement_for"),
    }
}
```

- [ ] **Step 4: 删除旧的 is_authenticated 函数**

删除 `apps/cli/src/commands/prerequisites.rs` 中的 `is_authenticated` 函数（第 193-210 行）。

- [ ] **Step 5: 更新 PrerequisiteError 枚举**

修改 `PrerequisiteError` 枚举，更新 `NotAuthenticated` 变体并添加 `UnsupportedPlatform`：

```rust
/// 前置检查失败错误。
#[derive(Debug, thiserror::Error)]
pub enum PrerequisiteError {
    #[error(
        "[[PLATFORM]] {binary} is not installed.\n\n📦 Install: {install_cmd}\n\nFull options:\n{install_hint}\n\n🌐 Official: {install_url}"
    )]
    NotFound {
        binary: String,
        platform: String,
        install_hint: String,
        install_url: String,
        install_cmd: String,
    },

    #[error("[[PLATFORM]] {binary} v{found} is too old (need v{required}+).\n\n📦 Upgrade: {install_cmd}")]
    VersionTooLow {
        binary: String,
        platform: String,
        found: String,
        required: String,
        install_cmd: String,
    },

    #[error("[[PLATFORM]] `{binary}` was found but `--version` failed.")]
    VersionCheckFailed {
        binary: String,
        platform: String,
    },

    #[error("[[PLATFORM]] Not authenticated.\n\n🔍 Reason: {reason}\n\n🔧 Fix: {hint}")]
    NotAuthenticated {
        binary: String,
        platform: String,
        reason: String,
        hint: String,
    },

    #[error("[[PLATFORM]] Unsupported platform: {platform}")]
    UnsupportedPlatform {
        platform: String,
    },
}
```

- [ ] **Step 6: 运行测试确认通过**

Run: `cargo test -p gitflow-cli --bin gitflow-cli prerequisites`
Expected: PASS (所有测试)

- [ ] **Step 7: 运行完整构建和测试**

Run: `cargo build --workspace`
Expected: PASS

Run: `cargo test --workspace`
Expected: PASS (所有测试)

- [ ] **Step 8: 提交**

```bash
git add apps/cli/src/commands/prerequisites.rs
git commit -m "refactor(cli): use AuthChecker in prerequisites check

- Replace is_authenticated() with AuthChecker trait
- Add create_auth_checker() factory function
- Update PrerequisiteError with detailed reason and hint
- Add UnsupportedPlatform variant
- Delete obsolete is_authenticated() function

Fixes bug where wrong binary path was used for authentication check.

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 6: 编写集成测试

**Files:**
- Create: `apps/cli/tests/prerequisites_integration.rs`

**Interfaces:**
- Consumes: `prerequisites::check()` 函数
- Produces: 集成测试覆盖完整流程

---

- [ ] **Step 1: 创建集成测试文件**

创建 `apps/cli/tests/prerequisites_integration.rs`：

```rust
//! 前置检查集成测试。
//!
//! 测试完整的 prerequisites::check() 流程，包括：
//! - CLI 不存在
//! - CLI 版本过低
//! - 未认证
//! - 认证成功

use assert_cmd::Command;
use predicates::prelude::*;
use temp_env;

#[test]
fn test_prerequisites_check_success_with_env_var() {
    // 使用环境变量模拟已认证状态
    temp_env::with_var("GITCODE_TOKEN", Some("test_token"), || {
        let mut cmd = Command::cargo_bin("gitflow-cli").unwrap();
        cmd.arg("auth")
            .arg("status")
            .arg("--platform")
            .arg("gitcode");

        // 应该成功（因为设置了环境变量）
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("logged in").or(predicate::str::contains("success")));
    });
}

#[test]
fn test_prerequisites_check_cli_not_found() {
    // 使用空 PATH 模拟 CLI 不存在
    let temp_dir = tempfile::tempdir().unwrap();
    temp_env::with_var("PATH", Some(temp_dir.path().to_str().unwrap()), || {
        let mut cmd = Command::cargo_bin("gitflow-cli").unwrap();
        cmd.arg("issue").arg("list").arg("--platform").arg("gitcode");

        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("is not installed"))
            .stderr(predicate::str::contains("Install:"));
    });
}

#[test]
fn test_prerequisites_check_unsupported_platform() {
    let mut cmd = Command::cargo_bin("gitflow-cli").unwrap();
    cmd.arg("issue").arg("list").arg("--platform").arg("unsupported");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Unsupported platform"));
}
```

- [ ] **Step 2: 运行集成测试**

Run: `cargo test -p gitflow-cli --test prerequisites_integration`
Expected: PASS (3 个测试)

- [ ] **Step 3: 提交**

```bash
git add apps/cli/tests/prerequisites_integration.rs
git commit -m "test(cli): add integration tests for prerequisites check

- Test successful auth with env var
- Test CLI not found error
- Test unsupported platform error

Ensures prerequisites::check() works correctly end-to-end.

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 7: 更新文档和质量检查

**Files:**
- Modify: `docs/superpowers/specs/2026-07-08-gitcode-auth-fix-design.md` (更新状态)
- Modify: `CHANGELOG.md` (添加条目)

**Interfaces:**
- Consumes: 所有已实现的功能
- Produces: 更新的文档和变更日志

---

- [ ] **Step 1: 更新设计文档状态**

修改 `docs/superpowers/specs/2026-07-08-gitcode-auth-fix-design.md` 第 4 行：

```markdown
**状态**: 已实现 ✅
```

- [ ] **Step 2: 更新 CHANGELOG.md**

在 `CHANGELOG.md` 顶部添加：

```markdown
## [Unreleased]

### Fixed

- **auth**: Fix GitCode authentication check using wrong binary path
  - `is_authenticated()` now uses correct `path` instead of `binary` string
  - Support both old and new gitcode CLI output formats
  - Add detailed error messages with reason and fix hints

### Added

- **core**: Add `AuthChecker` trait for unified authentication checking
- **core**: Add `AuthCheckResult` struct for detailed auth status
- **gitcode**: Implement `AuthChecker` for `GitCodeAuthProvider`
- **github**: Implement `AuthChecker` for `GitHubAuthProvider`
- **gitlab**: Implement `AuthChecker` for `GitLabAuthProvider`
- **cli**: Add integration tests for prerequisites check

### Changed

- **cli**: Refactor `prerequisites::check()` to use `AuthChecker` trait
- **cli**: Enhance `PrerequisiteError::NotAuthenticated` with detailed messages
```

- [ ] **Step 3: 运行完整质量检查**

Run: `cargo fmt --all -- --check`
Expected: PASS

Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
Expected: PASS

Run: `cargo test --workspace`
Expected: PASS

Run: `cargo audit`
Expected: PASS (或已知问题)

- [ ] **Step 4: 检查代码覆盖率**

Run: `cargo tarpaulin --workspace --out Html`
Expected: 覆盖率 > 80%

如果覆盖率不足，需要添加更多测试。

- [ ] **Step 5: 提交**

```bash
git add docs/superpowers/specs/2026-07-08-gitcode-auth-fix-design.md CHANGELOG.md
git commit -m "docs: update design spec status and changelog

- Mark design spec as implemented
- Add changelog entries for auth fix and refactor

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## 验收标准检查清单

完成所有任务后，验证以下标准：

- [ ] 所有单元测试通过: `cargo test --workspace`
- [ ] 所有集成测试通过: `cargo test --workspace --test '*'`
- [ ] 代码覆盖率 > 80%: `cargo tarpaulin`
- [ ] `cargo clippy` 无警告: `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo fmt` 格式化通过: `cargo fmt --all -- --check`
- [ ] `cargo audit` 无严重问题
- [ ] 文档更新完成
- [ ] 代码审查通过

---

## 执行选择

计划已保存到 `docs/superpowers/plans/2026-07-08-gitcode-auth-fix.md`。

**两种执行方式：**

**1. Subagent-Driven（推荐）** — 每个任务分派一个 fresh subagent，任务间审查，快速迭代

**2. Inline Execution** — 在当前会话中执行任务，批量执行并设置检查点

**选择哪种方式？**
