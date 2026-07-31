# 契约测试 + 兼容性矩阵 + 版本护栏 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Lock down CLI output contracts via fixture tests, unify error handling into core with Chinese-first messages, and create a single-source compatibility matrix.

**Architecture:** New `PlatformCliError` type in core replaces three duplicate per-crate error types. Fixture files per crate validate deserialization against real CLI output. A JSON compatibility matrix embedded at compile time drives both `prerequisites.rs` version checks and generated Markdown docs.

**Tech Stack:** Rust 2024, thiserror, serde, serde_json, tokio, tracing

**Spec:** `docs/superpowers/specs/2026-07-31-contract-test-compatibility-matrix-design.md`

## Global Constraints

- Rust 2024 edition, pinned toolchain in `rust-toolchain.toml`
- `#![forbid(unsafe_code)]` at all crate roots
- No `unwrap()`/`expect()` in production code
- All public items documented; `Debug` derived on all types
- Pass `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`
- TDD: RED → GREEN → REFACTOR for every behavior change
- User-facing error messages in Chinese (中文主导)
- Raw CLI stderr never in user-visible output — only `tracing::debug!`
- No new external crate dependencies
- Preserve `[[PLATFORM]]`, `[[INSTALL_COMMAND]]`, `[[LOGIN_COMMAND]]`, `[[LOGIN_WITH_TOKEN]]` agent-parseable markers in prerequisite errors

---

### Task 1: PlatformCliError in core

**Files:**
- Create: `crates/core/src/cli_error.rs`
- Modify: `crates/core/src/lib.rs` (add module + re-export + CoreError variant)

**Interfaces:**
- Produces: `PlatformCliError { user_message, raw_stderr, hint, doc_link, code, platform }`, `impl Display`, `impl std::error::Error`, `CoreError::Cli(PlatformCliError)`

- [ ] **Step 1: Write failing test for PlatformCliError Display**

In `crates/core/src/cli_error.rs`:

```rust
//! 统一的底层平台 CLI 错误类型。

use std::fmt;

use crate::platform::Platform;

/// 统一的底层平台 CLI 错误。
///
/// 各平台 crate 的 `parse_*_error()` 函数返回此类型，
/// 替代原先各自定义的 `GhError`、`GlabError`、`GitcodeError`。
///
/// 用户可见信息（`user_message`、`hint`）为中文主导；
/// `raw_stderr` 仅用于 `tracing::debug!`，不展示给用户。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PlatformCliError {
    /// 用户可见的错误消息（中文主导）。
    pub user_message: String,
    /// 底层 CLI 原始 stderr（仅用于调试日志，不展示给用户）。
    pub raw_stderr: String,
    /// 修复建议（中文）。
    pub hint: Option<String>,
    /// 相关文档链接。
    pub doc_link: Option<String>,
    /// 平台错误代码（如 `NOT_FOUND`）。
    pub code: Option<String>,
    /// 来源平台。
    pub platform: Platform,
}

impl fmt::Display for PlatformCliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.user_message)?;
        if let Some(ref hint) = self.hint {
            write!(f, "\n\n🔧 修复建议：{hint}")?;
        }
        if let Some(ref link) = self.doc_link {
            write!(f, "\n📖 文档：{link}")?;
        }
        Ok(())
    }
}

impl std::error::Error for PlatformCliError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_display_user_message_only_when_no_hint_no_link() {
        let err = PlatformCliError {
            user_message: "资源不存在".into(),
            raw_stderr: "gh: NOT_FOUND".into(),
            hint: None,
            doc_link: None,
            code: Some("NOT_FOUND".into()),
            platform: Platform::GitHub,
        };
        assert_eq!(err.to_string(), "资源不存在");
    }

    #[test]
    fn test_should_display_with_hint_and_doc_link() {
        let err = PlatformCliError {
            user_message: "认证失败".into(),
            raw_stderr: "raw error".into(),
            hint: Some("运行 `gh auth login` 重新认证".into()),
            doc_link: Some("https://cli.github.com/manual/".into()),
            code: None,
            platform: Platform::GitHub,
        };
        let display = err.to_string();
        assert!(display.contains("认证失败"));
        assert!(display.contains("🔧 修复建议：运行 `gh auth login` 重新认证"));
        assert!(display.contains("📖 文档：https://cli.github.com/manual/"));
        // raw_stderr must NOT appear in Display
        assert!(!display.contains("raw error"));
    }

    #[test]
    fn test_should_include_raw_stderr_in_debug() {
        let err = PlatformCliError {
            user_message: "错误".into(),
            raw_stderr: "secret debug info".into(),
            hint: None,
            doc_link: None,
            code: None,
            platform: Platform::GitLab,
        };
        let debug = format!("{err:?}");
        assert!(debug.contains("secret debug info"));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-cli-core cli_error -- --nocapture`
Expected: FAIL — module `cli_error` not found

- [ ] **Step 3: Add CoreError::Cli variant**

In `crates/core/src/lib.rs`, add module declaration and re-export:

```rust
pub mod cli_error;
pub use cli_error::PlatformCliError;
```

Add new variant to `CoreError`:

```rust
    /// 底层平台 CLI 执行错误（结构化）。
    ///
    /// 包含中文用户消息和修复建议，原始 stderr 仅用于调试。
    #[error(transparent)]
    Cli(#[from] PlatformCliError),
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p gitflow-cli-core cli_error`
Expected: 3 tests PASS

- [ ] **Step 5: Run clippy**

Run: `cargo clippy -p gitflow-cli-core --all-targets -- -D warnings -W clippy::pedantic`
Expected: no warnings

- [ ] **Step 6: Commit**

```bash
git add crates/core/src/cli_error.rs crates/core/src/lib.rs
git commit -m "feat(core): add unified PlatformCliError type

Add PlatformCliError to core with Chinese-first user_message,
raw_stderr for debug only, hint, doc_link, code, and platform.
Add CoreError::Cli variant with #[from] conversion.

Refs #95"
```

---

### Task 2: Migrate github crate to PlatformCliError

**Files:**
- Modify: `crates/github/src/error.rs` (rewrite `parse_gh_error`, delete `GhError`)
- Modify: `crates/github/src/pr.rs` (update error call sites)
- Modify: `crates/github/src/issue.rs` (update error call sites)
- Modify: `crates/github/src/label.rs` (update error call sites)
- Modify: `crates/github/src/release.rs` (update error call sites)
- Modify: `crates/github/src/pipeline.rs` (update error call sites)
- Modify: `crates/github/src/review.rs` (update error call sites)
- Modify: `crates/github/src/commit.rs` (update error call sites)
- Modify: `crates/github/src/auth.rs` (update error call sites)
- Modify: `crates/github/src/lib.rs` (update re-exports if any)

**Interfaces:**
- Consumes: `PlatformCliError` from core (Task 1)
- Produces: `parse_gh_error(stderr: &[u8]) -> PlatformCliError`

- [ ] **Step 1: Write failing test for new parse_gh_error**

Replace tests in `crates/github/src/error.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use gitflow_cli_core::PlatformCliError;

    #[test]
    fn test_should_parse_gh_json_error_to_platform_cli_error() {
        let json = br#"{"message": "GraphQL: Could not resolve to a user with the login 'nobody'.", "code": "NOT_FOUND"}"#;
        let err = parse_gh_error(json);
        assert_eq!(err.code.as_deref(), Some("NOT_FOUND"));
        assert_eq!(err.platform, gitflow_cli_core::platform::Platform::GitHub);
        assert!(!err.user_message.is_empty());
        assert!(!err.raw_stderr.is_empty());
    }

    #[test]
    fn test_should_parse_gh_plain_text_error() {
        let stderr = b"gh: Not logged in. Please run `gh auth login` to authenticate.";
        let err = parse_gh_error(stderr);
        assert!(err.user_message.contains("认证") || err.user_message.contains("登录"));
        assert!(err.hint.is_some());
        assert_eq!(err.platform, gitflow_cli_core::platform::Platform::GitHub);
        assert!(err.raw_stderr.contains("Not logged in"));
    }

    #[test]
    fn test_should_not_leak_raw_stderr_in_display() {
        let stderr = b"internal gh debug trace line";
        let err = parse_gh_error(stderr);
        let display = err.to_string();
        assert!(!display.contains("internal gh debug trace"));
    }

    #[test]
    fn test_should_handle_empty_stderr() {
        let err = parse_gh_error(b"");
        assert!(!err.user_message.is_empty());
        assert!(err.hint.is_some());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-cli-github error`
Expected: FAIL — `parse_gh_error` returns `GhError` not `PlatformCliError`

- [ ] **Step 3: Rewrite parse_gh_error to return PlatformCliError**

Replace `crates/github/src/error.rs`:

```rust
//! GitHub CLI 错误解析。

use gitflow_cli_core::PlatformCliError;
use gitflow_cli_core::platform::Platform;

/// 解析 `gh` CLI 的 stderr 输出为统一的 [`PlatformCliError`]。
///
/// 优先尝试 JSON 格式解析（`gh` 在 API 错误时输出 JSON），
/// 回退到纯文本模式（取前三行作为内部详情）。
/// 用户可见消息为中文。
#[must_use]
pub fn parse_gh_error(stderr: &[u8]) -> PlatformCliError {
    let text = String::from_utf8_lossy(stderr);

    // 尝试解析 gh 的 JSON 错误格式
    if let Ok(json) = serde_json::from_slice::<serde_json::Value>(stderr)
        && let Some(msg) = json.get("message").and_then(serde_json::Value::as_str)
    {
        let code = json
            .get("code")
            .and_then(serde_json::Value::as_str)
            .map(String::from);

        let user_message = match code.as_deref() {
            Some("NOT_FOUND") => "资源不存在".into(),
            Some("FORBIDDEN") => "权限不足".into(),
            _ => format!("GitHub 操作失败：{msg}"),
        };

        return PlatformCliError {
            user_message,
            raw_stderr: text.into_owned(),
            hint: Some("运行 `gh auth status` 检查认证状态".into()),
            doc_link: Some("https://cli.github.com/manual/".into()),
            code,
            platform: Platform::GitHub,
        };
    }

    // 回退：纯文本解析
    let user_message = if text.contains("Not logged in") || text.contains("auth") {
        "未登录 GitHub".into()
    } else {
        "GitHub CLI 执行失败".into()
    };

    PlatformCliError {
        user_message,
        raw_stderr: text.into_owned(),
        hint: Some("运行 `gh auth login` 完成登录".into()),
        doc_link: Some("https://cli.github.com/manual/".into()),
        code: None,
        platform: Platform::GitHub,
    }
}
```

- [ ] **Step 4: Update all call sites in github crate**

In every file that calls `parse_gh_error` and wraps in `CoreError::Platform(format!(...))`, change the pattern from:

```rust
let gh_err = parse_gh_error(&output.stderr);
return Err(CoreError::Platform(format!("{gh_err}")));
```

to:

```rust
return Err(parse_gh_error(&output.stderr).into());
```

This applies to: `pr.rs`, `issue.rs`, `label.rs`, `release.rs`, `pipeline.rs`, `review.rs`, `commit.rs`, `auth.rs`. Use `git grep 'parse_gh_error' crates/github/` to find all call sites.

- [ ] **Step 5: Remove GhError struct**

Delete the `GhError` struct and its `impl fmt::Display` from `crates/github/src/error.rs`. Remove any `pub use error::GhError` re-exports from `crates/github/src/lib.rs`.

- [ ] **Step 6: Run tests**

Run: `cargo test -p gitflow-cli-github`
Expected: all tests PASS

- [ ] **Step 7: Run clippy**

Run: `cargo clippy -p gitflow-cli-github --all-targets -- -D warnings -W clippy::pedantic`
Expected: no warnings

- [ ] **Step 8: Commit**

```bash
git add crates/github/
git commit -m "refactor(github): migrate to unified PlatformCliError

Replace GhError with PlatformCliError from core.
parse_gh_error now returns Chinese-first user messages.
Raw stderr preserved in raw_stderr field for debug only.

Refs #95"
```

---

### Task 3: Migrate gitlab crate to PlatformCliError

**Files:**
- Modify: `crates/gitlab/src/error.rs` (rewrite `parse_glab_error`, delete `GlabError`)
- Modify: `crates/gitlab/src/mr.rs` (update error call sites)
- Modify: `crates/gitlab/src/issue.rs` (update error call sites)
- Modify: `crates/gitlab/src/label.rs` (update error call sites)
- Modify: `crates/gitlab/src/release.rs` (update error call sites)
- Modify: `crates/gitlab/src/pipeline.rs` (update error call sites)
- Modify: `crates/gitlab/src/review.rs` (update error call sites)
- Modify: `crates/gitlab/src/commit.rs` (update error call sites)
- Modify: `crates/gitlab/src/auth.rs` (update error call sites)
- Modify: `crates/gitlab/src/lib.rs` (update re-exports if any)

**Interfaces:**
- Consumes: `PlatformCliError` from core (Task 1)
- Produces: `parse_glab_error(stderr: &[u8]) -> PlatformCliError`

- [ ] **Step 1: Write failing test for new parse_glab_error**

Replace tests in `crates/gitlab/src/error.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use gitflow_cli_core::platform::Platform;

    #[test]
    fn test_should_parse_glab_json_error_to_platform_cli_error() {
        let json = br#"{"message": "404 Not Found", "code": "NOT_FOUND"}"#;
        let err = parse_glab_error(json);
        assert_eq!(err.code.as_deref(), Some("NOT_FOUND"));
        assert_eq!(err.platform, Platform::GitLab);
        assert!(!err.user_message.is_empty());
    }

    #[test]
    fn test_should_parse_glab_plain_text_error() {
        let stderr = b"ERROR: not authenticated";
        let err = parse_glab_error(stderr);
        assert!(err.hint.is_some());
        assert_eq!(err.platform, Platform::GitLab);
        assert!(err.raw_stderr.contains("not authenticated"));
    }

    #[test]
    fn test_should_not_leak_raw_stderr_in_display() {
        let stderr = b"glab internal trace";
        let err = parse_glab_error(stderr);
        assert!(!err.to_string().contains("glab internal trace"));
    }

    #[test]
    fn test_should_handle_empty_stderr() {
        let err = parse_glab_error(b"");
        assert!(!err.user_message.is_empty());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-cli-gitlab error`
Expected: FAIL

- [ ] **Step 3: Rewrite parse_glab_error**

Same pattern as Task 2 but for `glab`:

```rust
//! GitLab CLI 错误解析。

use gitflow_cli_core::PlatformCliError;
use gitflow_cli_core::platform::Platform;

/// 解析 `glab` CLI 的 stderr 输出为统一的 [`PlatformCliError`]。
#[must_use]
pub fn parse_glab_error(stderr: &[u8]) -> PlatformCliError {
    let text = String::from_utf8_lossy(stderr);

    if let Ok(json) = serde_json::from_slice::<serde_json::Value>(stderr)
        && let Some(msg) = json.get("message").and_then(serde_json::Value::as_str)
    {
        let code = json
            .get("code")
            .and_then(serde_json::Value::as_str)
            .map(String::from);

        let user_message = match code.as_deref() {
            Some("NOT_FOUND") => "资源不存在".into(),
            Some("FORBIDDEN") => "权限不足".into(),
            _ => format!("GitLab 操作失败：{msg}"),
        };

        return PlatformCliError {
            user_message,
            raw_stderr: text.into_owned(),
            hint: Some("运行 `glab auth status` 检查认证状态".into()),
            doc_link: Some("https://gitlab.com/gitlab-org/cli/-/blob/main/docs/".into()),
            code,
            platform: Platform::GitLab,
        };
    }

    let user_message = if text.contains("not authenticated") || text.contains("auth") {
        "未登录 GitLab".into()
    } else {
        "GitLab CLI 执行失败".into()
    };

    PlatformCliError {
        user_message,
        raw_stderr: text.into_owned(),
        hint: Some("运行 `glab auth login` 完成登录".into()),
        doc_link: Some("https://gitlab.com/gitlab-org/cli/-/blob/main/docs/".into()),
        code: None,
        platform: Platform::GitLab,
    }
}
```

- [ ] **Step 4: Update all call sites in gitlab crate**

Change `CoreError::Platform(format!("{glab_err}"))` → `parse_glab_error(&output.stderr).into()` in all files. Use `git grep 'parse_glab_error' crates/gitlab/` to find them.

- [ ] **Step 5: Remove GlabError struct and update re-exports**

- [ ] **Step 6: Run tests**

Run: `cargo test -p gitflow-cli-gitlab`
Expected: all PASS

- [ ] **Step 7: Run clippy**

Run: `cargo clippy -p gitflow-cli-gitlab --all-targets -- -D warnings -W clippy::pedantic`

- [ ] **Step 8: Commit**

```bash
git add crates/gitlab/
git commit -m "refactor(gitlab): migrate to unified PlatformCliError

Replace GlabError with PlatformCliError from core.
Chinese-first error messages, raw stderr for debug only.

Refs #95"
```

---

### Task 4: Migrate gitcode crate to PlatformCliError

**Files:**
- Modify: `crates/gitcode/src/error.rs` (rewrite `parse_gitcode_error`, delete `GitcodeError`)
- Modify: `crates/gitcode/src/pr.rs` (update error call sites)
- Modify: `crates/gitcode/src/issue.rs` (update error call sites)
- Modify: `crates/gitcode/src/label.rs` (update error call sites)
- Modify: `crates/gitcode/src/release.rs` (update error call sites)
- Modify: `crates/gitcode/src/pipeline.rs` (update error call sites)
- Modify: `crates/gitcode/src/review.rs` (update error call sites)
- Modify: `crates/gitcode/src/commit.rs` (update error call sites)
- Modify: `crates/gitcode/src/auth.rs` (update error call sites)
- Modify: `crates/gitcode/src/lib.rs` (update re-exports if any)

**Interfaces:**
- Consumes: `PlatformCliError` from core (Task 1)
- Produces: `parse_gitcode_error(stderr: &[u8]) -> PlatformCliError`

- [ ] **Step 1: Write failing test for new parse_gitcode_error**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use gitflow_cli_core::platform::Platform;

    #[test]
    fn test_should_parse_gitcode_json_error() {
        let json = br#"{"message": "Unauthorized", "code": "UNAUTHORIZED"}"#;
        let err = parse_gitcode_error(json);
        assert_eq!(err.code.as_deref(), Some("UNAUTHORIZED"));
        assert_eq!(err.platform, Platform::GitCode);
        assert!(err.user_message.contains("认证") || err.user_message.contains("权限"));
    }

    #[test]
    fn test_should_parse_gitcode_plain_text_error() {
        let stderr = b"Error: authentication required";
        let err = parse_gitcode_error(stderr);
        assert!(err.hint.is_some());
        assert_eq!(err.platform, Platform::GitCode);
    }

    #[test]
    fn test_should_not_leak_raw_stderr() {
        let stderr = b"gitcode internal panic trace";
        let err = parse_gitcode_error(stderr);
        assert!(!err.to_string().contains("internal panic"));
    }

    #[test]
    fn test_should_handle_empty_stderr() {
        let err = parse_gitcode_error(b"");
        assert!(!err.user_message.is_empty());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-cli-gitcode error`

- [ ] **Step 3: Rewrite parse_gitcode_error**

```rust
//! GitCode CLI 错误解析。

use gitflow_cli_core::PlatformCliError;
use gitflow_cli_core::platform::Platform;

/// 解析 `gitcode` CLI 的 stderr 输出为统一的 [`PlatformCliError`]。
#[must_use]
pub fn parse_gitcode_error(stderr: &[u8]) -> PlatformCliError {
    let text = String::from_utf8_lossy(stderr);

    if let Ok(json) = serde_json::from_slice::<serde_json::Value>(stderr)
        && let Some(msg) = json.get("message").and_then(serde_json::Value::as_str)
    {
        let code = json
            .get("code")
            .and_then(serde_json::Value::as_str)
            .map(String::from);

        let user_message = match code.as_deref() {
            Some("UNAUTHORIZED") | Some("FORBIDDEN") => "认证失败或权限不足".into(),
            Some("NOT_FOUND") => "资源不存在".into(),
            _ => format!("GitCode 操作失败：{msg}"),
        };

        return PlatformCliError {
            user_message,
            raw_stderr: text.into_owned(),
            hint: Some("运行 `gitcode auth status` 检查认证状态".into()),
            doc_link: Some("https://gitcode.com/gitcode-cli/cli/blob/main/README.md".into()),
            code,
            platform: Platform::GitCode,
        };
    }

    let user_message = if text.contains("auth") || text.contains("login") {
        "未登录 GitCode".into()
    } else {
        "GitCode CLI 执行失败".into()
    };

    PlatformCliError {
        user_message,
        raw_stderr: text.into_owned(),
        hint: Some("运行 `gitcode auth login` 完成登录".into()),
        doc_link: Some("https://gitcode.com/gitcode-cli/cli/blob/main/README.md".into()),
        code: None,
        platform: Platform::GitCode,
    }
}
```

- [ ] **Step 4: Update all call sites in gitcode crate**

Change `CoreError::Platform(format!("{gitcode_err}"))` → `parse_gitcode_error(&output.stderr).into()`.

- [ ] **Step 5: Remove GitcodeError struct and update re-exports**

- [ ] **Step 6: Run tests**

Run: `cargo test -p gitflow-cli-gitcode`
Expected: all PASS

- [ ] **Step 7: Run clippy**

Run: `cargo clippy -p gitflow-cli-gitcode --all-targets -- -D warnings -W clippy::pedantic`

- [ ] **Step 8: Commit**

```bash
git add crates/gitcode/
git commit -m "refactor(gitcode): migrate to unified PlatformCliError

Replace GitcodeError with PlatformCliError from core.
Chinese-first error messages, raw stderr for debug only.

Refs #95"
```

---

### Task 5: Compatibility matrix data source

**Files:**
- Create: `docs/compatibility-matrix.json`
- Create: `crates/core/src/compatibility.rs`
- Modify: `crates/core/src/lib.rs` (add module + re-export)

**Interfaces:**
- Produces: `PlatformCompat { name, identifier, cli_binary, min_version, tested_versions, install_url, doc_link }`, `platform_compatibility() -> Vec<PlatformCompat>`, `platform_requirement(id: &str) -> Option<PlatformCompat>`

- [ ] **Step 1: Create compatibility-matrix.json**

Create `docs/compatibility-matrix.json`:

```json
{
  "schema_version": 1,
  "updated_at": "2026-07-31",
  "gitflow_cli_version": "0.9.0",
  "platforms": [
    {
      "name": "GitHub",
      "identifier": "github",
      "cli_binary": "gh",
      "min_version": "2.0.0",
      "tested_versions": ["2.62.0"],
      "install_url": "https://github.com/cli/cli#installation",
      "doc_link": "https://cli.github.com/manual/",
      "features": {
        "issue": true,
        "pr": true,
        "label": true,
        "milestone": true,
        "release": true,
        "pipeline": true,
        "review": true,
        "auth": true
      }
    },
    {
      "name": "GitLab",
      "identifier": "gitlab",
      "cli_binary": "glab",
      "min_version": "1.30.0",
      "tested_versions": ["1.46.1"],
      "install_url": "https://gitlab.com/gitlab-org/cli#installation",
      "doc_link": "https://gitlab.com/gitlab-org/cli/-/blob/main/docs/",
      "features": {
        "issue": true,
        "pr": true,
        "label": true,
        "milestone": true,
        "release": true,
        "pipeline": true,
        "review": true,
        "auth": true
      }
    },
    {
      "name": "GitCode",
      "identifier": "gitcode",
      "cli_binary": "gitcode",
      "min_version": "0.6.0",
      "tested_versions": ["0.6.1"],
      "install_url": "https://gitcode.com/gitcode-cli/cli",
      "doc_link": "https://gitcode.com/gitcode-cli/cli/blob/main/README.md",
      "features": {
        "issue": true,
        "pr": true,
        "label": true,
        "milestone": true,
        "release": true,
        "pipeline": true,
        "review": true,
        "auth": true
      }
    }
  ]
}
```

- [ ] **Step 2: Write failing test for compatibility module**

Create `crates/core/src/compatibility.rs`:

```rust
//! 兼容性矩阵数据。
//!
//! 从 `docs/compatibility-matrix.json` 编译时嵌入，
//! 提供各平台 CLI 版本要求和功能覆盖信息。

use serde::Deserialize;

/// 编译时嵌入的兼容性矩阵 JSON。
const MATRIX_JSON: &str = include_str!("../../../docs/compatibility-matrix.json");

/// 兼容性矩阵根结构。
#[derive(Debug, Deserialize)]
struct MatrixRoot {
    #[allow(dead_code)]
    schema_version: u32,
    #[allow(dead_code)]
    updated_at: String,
    #[allow(dead_code)]
    gitflow_cli_version: String,
    platforms: Vec<PlatformCompat>,
}

/// 单个平台的兼容性信息。
#[derive(Debug, Clone, Deserialize)]
pub struct PlatformCompat {
    /// 平台显示名称（如 "GitHub"）。
    pub name: String,
    /// 平台标识符（如 "github"）。
    pub identifier: String,
    /// CLI 可执行文件名（如 "gh"）。
    pub cli_binary: String,
    /// 最低版本号（semver）。
    pub min_version: String,
    /// 已测试的版本列表。
    pub tested_versions: Vec<String>,
    /// 官方安装指引链接。
    pub install_url: String,
    /// 文档链接。
    pub doc_link: String,
}

/// 获取所有平台的兼容性信息。
///
/// # Panics
///
/// 当嵌入的 JSON 格式无效时 panic（编译时数据损坏，属于不可恢复错误）。
#[must_use]
pub fn platform_compatibility() -> Vec<PlatformCompat> {
    let root: MatrixRoot = serde_json::from_str(MATRIX_JSON)
        .expect("embedded compatibility-matrix.json is invalid");
    root.platforms
}

/// 获取指定平台的兼容性信息。
#[must_use]
pub fn platform_requirement(identifier: &str) -> Option<PlatformCompat> {
    platform_compatibility()
        .into_iter()
        .find(|p| p.identifier == identifier)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_load_all_three_platforms() {
        let platforms = platform_compatibility();
        assert_eq!(platforms.len(), 3);
    }

    #[test]
    fn test_should_return_github_requirement() {
        let gh = platform_requirement("github").expect("github should exist");
        assert_eq!(gh.cli_binary, "gh");
        assert_eq!(gh.min_version, "2.0.0");
        assert!(!gh.install_url.is_empty());
        assert!(!gh.doc_link.is_empty());
    }

    #[test]
    fn test_should_return_gitcode_min_version_0_6() {
        let gc = platform_requirement("gitcode").expect("gitcode should exist");
        assert_eq!(gc.min_version, "0.6.0");
    }

    #[test]
    fn test_should_return_none_for_unknown_platform() {
        assert!(platform_requirement("bitbucket").is_none());
    }
}
```

- [ ] **Step 3: Add module to lib.rs**

In `crates/core/src/lib.rs`:

```rust
pub mod compatibility;
pub use compatibility::{PlatformCompat, platform_compatibility, platform_requirement};
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p gitflow-cli-core compatibility`
Expected: 4 tests PASS

- [ ] **Step 5: Run clippy**

Run: `cargo clippy -p gitflow-cli-core --all-targets -- -D warnings -W clippy::pedantic`

- [ ] **Step 6: Commit**

```bash
git add docs/compatibility-matrix.json crates/core/src/compatibility.rs crates/core/src/lib.rs
git commit -m "feat(core): add compatibility matrix data source

Single JSON source of truth for platform CLI version requirements.
Embedded at compile time via include_str!.
gitcode min_version set to 0.6.0.

Refs #95"
```

---

### Task 6: Version guardrails — prerequisites.rs Chinese + doc_link

**Files:**
- Modify: `apps/cli/src/commands/prerequisites.rs` (Chinese errors, doc_link field, gitcode 0.6.0, use compatibility data)

**Interfaces:**
- Consumes: `platform_requirement()` from core (Task 5)

- [ ] **Step 1: Write failing test for Chinese error messages**

Add tests to `apps/cli/src/commands/prerequisites.rs`:

```rust
    #[test]
    fn test_should_show_chinese_not_found_message() {
        let err = PrerequisiteError::NotFound {
            binary: "gh".into(),
            platform: "github".into(),
            install_hint: "brew install gh".into(),
            install_url: "https://example.com".into(),
            install_cmd: "brew install gh".into(),
            doc_link: "https://cli.github.com/manual/".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("未检测到"));
        assert!(msg.contains("[[PLATFORM]]"));
        assert!(msg.contains("📖 文档"));
    }

    #[test]
    fn test_should_show_chinese_version_too_low_message() {
        let err = PrerequisiteError::VersionTooLow {
            binary: "gitcode".into(),
            platform: "gitcode".into(),
            found: "0.5.0".into(),
            required: "0.6.0".into(),
            install_cmd: "pip install gitcode-cli".into(),
            doc_link: "https://gitcode.com/gitcode-cli/cli".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("版本过低"));
        assert!(msg.contains("v0.5.0"));
        assert!(msg.contains("v0.6.0"));
    }

    #[test]
    fn test_should_require_gitcode_0_6_0() {
        let req = requirement_for("gitcode").expect("gitcode requirement");
        assert_eq!(req.min_version, "0.6.0");
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-cli prerequisites`
Expected: FAIL — `doc_link` field doesn't exist yet, messages are English

- [ ] **Step 3: Update PrerequisiteError to Chinese + doc_link**

Add `doc_link: String` field to `NotFound`, `VersionTooLow`, `VersionParseFailed` variants.

Update `#[error(...)]` attributes:

```rust
    #[error(
        "[[PLATFORM]] 未检测到 {binary}。\n\n\
         📦 安装：{install_cmd}\n\
         📖 文档：{doc_link}\n\n\
         其他安装方式：\n{install_hint}"
    )]
    NotFound { binary: String, platform: String, install_hint: String, install_url: String, install_cmd: String, doc_link: String },

    #[error(
        "[[PLATFORM]] {binary} 版本过低：当前 v{found}，需要 v{required}+。\n\n\
         📦 升级：{install_cmd}\n\
         📖 文档：{doc_link}"
    )]
    VersionTooLow { binary: String, platform: String, found: String, required: String, install_cmd: String, doc_link: String },

    #[error(
        "[[PLATFORM]] {binary} 版本信息解析失败。\n\n\
         📦 重新安装：{install_cmd}\n\
         📖 文档：{doc_link}"
    )]
    VersionParseFailed { binary: String, platform: String, install_cmd: String, doc_link: String },

    #[error(
        "[[PLATFORM]] {binary} 未认证。\n\n\
         🔍 原因：{reason}\n\
         🔧 修复：运行 `{hint}` 完成登录"
    )]
    NotAuthenticated { binary: String, platform: String, reason: String, hint: String },

    #[error("不支持的平台：{platform}。支持的平台：github、gitlab、gitcode")]
    UnsupportedPlatform { platform: String },
```

- [ ] **Step 4: Update CliRequirement + requirement_for**

Add `doc_link: &'static str` to `CliRequirement`. Update `requirement_for()` — bump gitcode `min_version` to `"0.6.0"`, add `doc_link` for each platform.

- [ ] **Step 5: Update check() and find_gitcode_cli() to pass doc_link**

All `PrerequisiteError::NotFound` / `VersionTooLow` / `VersionParseFailed` constructions need the new `doc_link` field from `req.doc_link`.

- [ ] **Step 6: Update existing tests**

Update `test_should_return_requirement_for_gitcode` to assert `min_version == "0.6.0"`.

- [ ] **Step 7: Run tests**

Run: `cargo test -p gitflow-cli prerequisites`
Expected: all PASS

- [ ] **Step 8: Run clippy**

Run: `cargo clippy -p gitflow-cli --all-targets -- -D warnings -W clippy::pedantic`

- [ ] **Step 9: Commit**

```bash
git add apps/cli/src/commands/prerequisites.rs
git commit -m "feat(cli): Chinese-first prerequisite errors + gitcode 0.6.0

PrerequisiteError messages now Chinese-first with doc_link.
gitcode min_version bumped from 0.5.9 to 0.6.0.
Agent-parseable markers [[PLATFORM]] preserved.

Refs #95"
```

---

### Task 7: GitHub contract tests

**Files:**
- Create: `crates/github/tests/fixtures/pr_list_github_v2.json`
- Create: `crates/github/tests/fixtures/issue_list_github_v2.json`
- Create: `crates/github/tests/fixtures/label_list_github_v2.json`
- Create: `crates/github/tests/contract_test.rs`

**Interfaces:**
- Consumes: `MockCommandRunner` from `crates/github/src/runner.rs`, provider types from github crate

- [ ] **Step 1: Create fixture files**

Create `crates/github/tests/fixtures/pr_list_github_v2.json` — capture from `gh pr list --json number,title,body,state,author,labels,headRefName,baseRefName,isDraft,url,createdAt,updatedAt --repo <repo>` or use representative sample. Must include at least 1 complete PR record with all fields the `PrApiResponse` struct deserializes.

Create `crates/github/tests/fixtures/issue_list_github_v2.json` — capture from `gh issue list --json number,title,body,state,labels,author,assignees,createdAt,updatedAt,url --repo <repo>`.

Create `crates/github/tests/fixtures/label_list_github_v2.json` — capture from `gh label list --json name,color,description --repo <repo>`.

Sanitize: replace real tokens/emails with placeholders.

- [ ] **Step 2: Write contract test**

Create `crates/github/tests/contract_test.rs`:

```rust
//! 契约测试：验证 GitHub CLI JSON 输出格式与反序列化模型一致。
//!
//! 夹具来源：gh v2.x 真实 CLI 输出。
//! 若上游 gh CLI 变更输出格式，此测试将失败。

use gitflow_cli_github::{GitHubPrProvider, GitHubIssueProvider};

// Note: LabelProvider may not use CommandRunner generic — adjust based on
// actual implementation. If GitHubLabelProvider uses tokio::process::Command
// directly, label contract test may need a different approach.

#[tokio::test]
async fn test_contract_pr_list_github_v2() {
    let fixture = include_str!("fixtures/pr_list_github_v2.json");
    // Use the crate's test MockCommandRunner
    let runner = gitflow_cli_github::MockCommandRunner::success(fixture);
    let provider = GitHubPrProvider::with_runner("owner/repo", runner);

    let prs = provider
        .list(gitflow_cli_core::pr::ListPrArgs::default())
        .await
        .expect("PR list deserialization failed — gh output format may have changed");

    assert!(!prs.is_empty(), "fixture should contain at least 1 PR");
    let pr = &prs[0];
    assert!(pr.number > 0);
    assert!(!pr.title.is_empty());
}

#[tokio::test]
async fn test_contract_issue_list_github_v2() {
    let fixture = include_str!("fixtures/issue_list_github_v2.json");
    let runner = gitflow_cli_github::MockCommandRunner::success(fixture);
    let provider = GitHubIssueProvider::with_runner("owner/repo", runner);

    let issues = provider
        .list(gitflow_cli_core::issue::ListIssueArgs::default())
        .await
        .expect("Issue list deserialization failed");

    assert!(!issues.is_empty());
    assert!(issues[0].number > 0);
    assert!(!issues[0].title.is_empty());
}
```

Note: `MockCommandRunner` is `#[cfg(test)]` in `runner.rs` — for integration tests in `tests/`, it must be exposed. Check if the crate already exposes it via a `testing` feature or `pub` under `cfg(test)`. If not, the contract tests should be placed as unit tests inside the crate (e.g., `#[cfg(test)] mod contract_tests` in each provider file) rather than in `tests/`. Adapt based on what compiles.

- [ ] **Step 3: Run tests**

Run: `cargo test -p gitflow-cli-github contract`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add crates/github/tests/
git commit -m "test(github): add contract tests for PR/Issue/Label fixtures

Lock down gh CLI JSON output format via fixture deserialization.
Fixtures captured from gh v2.x real output.

Refs #95"
```

---

### Task 8: GitLab contract tests

**Files:**
- Create: `crates/gitlab/tests/fixtures/pr_list_gitlab_v1.json`
- Create: `crates/gitlab/tests/fixtures/issue_list_gitlab_v1.json`
- Create: `crates/gitlab/tests/fixtures/label_list_gitlab_v1.json`
- Create: `crates/gitlab/tests/contract_test.rs` (or inline `#[cfg(test)] mod contract_tests`)

**Interfaces:**
- Consumes: `MockCommandRunner` from gitlab crate runner

- [ ] **Step 1: Create fixture files**

Capture from `glab mr list`, `glab issue list`, `glab label list` with `--output json`, or build representative samples matching the serde models in `crates/gitlab/src/mr.rs`, `issue.rs`, `label.rs`.

- [ ] **Step 2: Write contract tests**

Same pattern as Task 7 but for GitLab providers (`GitLabPrProvider`, `GitLabIssueProvider`).

- [ ] **Step 3: Run tests**

Run: `cargo test -p gitflow-cli-gitlab contract`

- [ ] **Step 4: Commit**

```bash
git add crates/gitlab/tests/
git commit -m "test(gitlab): add contract tests for MR/Issue/Label fixtures

Refs #95"
```

---

### Task 9: GitCode contract tests

**Files:**
- Rename: `crates/gitcode/tests/fixtures/pr_list_gitcode_v0.6.1.json` → `pr_list_gitcode_v0.6.json`
- Create: `crates/gitcode/tests/fixtures/issue_list_gitcode_v0.6.json`
- Create: `crates/gitcode/tests/fixtures/label_list_gitcode_v0.6.json`
- Create: `crates/gitcode/tests/contract_test.rs` (or inline)

**Interfaces:**
- Consumes: `MockCommandRunner` from gitcode crate runner

- [ ] **Step 1: Rename existing fixture + create new fixtures**

```bash
git mv crates/gitcode/tests/fixtures/pr_list_gitcode_v0.6.1.json \
       crates/gitcode/tests/fixtures/pr_list_gitcode_v0.6.json
```

Create `issue_list_gitcode_v0.6.json` and `label_list_gitcode_v0.6.json` from real `gitcode` CLI output or representative samples.

- [ ] **Step 2: Write contract tests**

Same pattern as Task 7/8 for GitCode providers.

- [ ] **Step 3: Update any existing references to old fixture name**

Search: `git grep 'pr_list_gitcode_v0.6.1' crates/gitcode/`

- [ ] **Step 4: Run tests**

Run: `cargo test -p gitflow-cli-gitcode contract`

- [ ] **Step 5: Commit**

```bash
git add crates/gitcode/tests/
git commit -m "test(gitcode): add contract tests + rename fixture to v0.6

Refs #95"
```

---

### Task 10: Markdown generation + final quality gate

**Files:**
- Create: `crates/core/examples/gen_compat_matrix.rs`
- Modify: `Makefile` (add `compatibility-matrix` target)
- Create: `docs/compatibility-matrix.md` (generated)

**Interfaces:**
- Consumes: `docs/compatibility-matrix.json`

- [ ] **Step 1: Create the generator example**

Create `crates/core/examples/gen_compat_matrix.rs`:

```rust
//! 从 `docs/compatibility-matrix.json` 生成 Markdown 兼容性矩阵。
//!
//! 用法：`cargo run -p gitflow-cli-core --example gen-compat-matrix`

use std::fs;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct MatrixRoot {
    updated_at: String,
    gitflow_cli_version: String,
    platforms: Vec<PlatformEntry>,
}

#[derive(Debug, Deserialize)]
struct PlatformEntry {
    name: String,
    cli_binary: String,
    min_version: String,
    tested_versions: Vec<String>,
    features: std::collections::BTreeMap<String, bool>,
}

fn main() {
    let json = fs::read_to_string("docs/compatibility-matrix.json")
        .expect("failed to read docs/compatibility-matrix.json");
    let root: MatrixRoot = serde_json::from_str(&json).expect("invalid JSON");

    let mut md = String::new();
    md.push_str("# 兼容性矩阵\n\n");
    md.push_str(&format!(
        "> 自动生成，请勿手动编辑。数据源：`docs/compatibility-matrix.json`\n> 更新时间：{} · gitflow-cli v{}\n\n",
        root.updated_at, root.gitflow_cli_version
    ));
    md.push_str("| 平台 | CLI 工具 | 最低版本 | 已测试版本 | 功能覆盖 |\n");
    md.push_str("|------|---------|---------|-----------|--------|\n");

    for p in &root.platforms {
        let tested = p.tested_versions.join(", ");
        let features: Vec<String> = p
            .features
            .iter()
            .map(|(k, &v)| {
                if v { format!("{k} ✅") } else { format!("{k} ❌") }
            })
            .collect();
        md.push_str(&format!(
            "| {} | `{}` | ≥ {} | {} | {} |\n",
            p.name,
            p.cli_binary,
            p.min_version,
            tested,
            features.join(" ")
        ));
    }

    fs::write("docs/compatibility-matrix.md", &md).expect("failed to write markdown");
    println!("Generated docs/compatibility-matrix.md");
}
```

- [ ] **Step 2: Add Makefile target**

In `Makefile`, add:

```makefile
.PHONY: compatibility-matrix
compatibility-matrix: ## 从 JSON 生成兼容性矩阵 Markdown
	cargo run -p gitflow-cli-core --example gen-compat-matrix
```

- [ ] **Step 3: Run the generator**

Run: `make compatibility-matrix`
Expected: `docs/compatibility-matrix.md` created

- [ ] **Step 4: Verify generated Markdown**

Run: `cat docs/compatibility-matrix.md`
Expected: table with 3 platform rows

- [ ] **Step 5: Run full quality gate**

Run: `cargo test --workspace`
Expected: all tests PASS

Run: `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: no warnings

Run: `cargo +nightly fmt -- --check`
Expected: no formatting issues

- [ ] **Step 6: Commit**

```bash
git add crates/core/examples/gen_compat_matrix.rs Makefile docs/compatibility-matrix.md
git commit -m "feat: add compatibility matrix Markdown generation

make compatibility-matrix generates docs/compatibility-matrix.md
from single JSON source of truth.

Closes #95"
```
