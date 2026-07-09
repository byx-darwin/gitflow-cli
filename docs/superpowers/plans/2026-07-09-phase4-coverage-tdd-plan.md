# Phase 4 Coverage TDD — Failure Tests Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add CommandRunner trait abstraction and ~142 failure tests for all 5 core traits across 3 platform implementations.

**Architecture:** Each platform crate (github, gitlab, gitcode) defines its own CommandRunner trait with RealCommandRunner (default) and MockCommandRunner (test-only) implementations. Providers gain a generic parameter `R: CommandRunner` with default `RealCommandRunner`, maintaining backward compatibility. Failure tests use MockCommandRunner to simulate CLI errors and invalid JSON.

**Tech Stack:** Rust 2024, tokio (async tests), serde_json, cross-platform ExitStatus handling

## Global Constraints

- Rust 2024 edition with pinned toolchain
- No `unwrap()` or `expect()` in production code
- All public items require documentation
- Use `matches!()` for error type assertions in tests
- Cross-platform support: Unix (`from_raw(i32)`) and Windows (`from_raw(u32)`)
- `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic` must pass
- All existing tests must continue to pass (backward compatibility)

---

## Task 1: GitHub CommandRunner Infrastructure

**Files:**
- Create: `crates/github/src/runner.rs`
- Modify: `crates/github/src/lib.rs:1` (add `pub mod runner;`)

**Interfaces:**
- Consumes: Nothing (foundation task)
- Produces: `CommandRunner` trait, `RealCommandRunner`, `MockCommandRunner`, `CommandOutput`

- [ ] **Step 1: Create runner.rs with trait and implementations**

```rust
// crates/github/src/runner.rs
//! Command execution abstraction for GitHub CLI (`gh`).

use std::process::ExitStatus;

/// Output from a CLI command execution.
#[derive(Debug, Clone)]
pub struct CommandOutput {
    /// Process exit status.
    pub status: ExitStatus,
    /// Standard output bytes.
    pub stdout: Vec<u8>,
    /// Standard error bytes.
    pub stderr: Vec<u8>,
}

/// Trait for executing CLI commands. Abstracts process spawning for testability.
pub trait CommandRunner: std::fmt::Debug + Send + Sync {
    /// Execute a command with the given program and arguments.
    ///
    /// # Errors
    ///
    /// Returns `std::io::Error` if the command cannot be spawned.
    fn run(&self, program: &str, args: &[&str]) -> std::io::Result<CommandOutput>;
}

/// Default implementation that spawns real processes.
#[derive(Debug, Clone, Default)]
pub struct RealCommandRunner;

impl CommandRunner for RealCommandRunner {
    fn run(&self, program: &str, args: &[&str]) -> std::io::Result<CommandOutput> {
        let output = std::process::Command::new(program).args(args).output()?;
        Ok(CommandOutput {
            status: output.status,
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }
}

/// Mock implementation for testing failure scenarios.
#[cfg(test)]
#[derive(Debug, Clone)]
pub struct MockCommandRunner {
    /// Pre-configured output to return.
    pub output: std::io::Result<CommandOutput>,
}

#[cfg(test)]
impl MockCommandRunner {
    #[cfg(unix)]
    fn make_exit_status(code: i32) -> ExitStatus {
        use std::os::unix::process::ExitStatusExt;
        ExitStatus::from_raw(code)
    }

    #[cfg(windows)]
    fn make_exit_status(code: i32) -> ExitStatus {
        use std::os::windows::process::ExitStatusExt;
        ExitStatus::from_raw(code as u32)
    }

    /// Create a mock that returns success with the given stdout.
    pub fn success(stdout: &str) -> Self {
        Self {
            output: Ok(CommandOutput {
                status: Self::make_exit_status(0),
                stdout: stdout.as_bytes().to_vec(),
                stderr: Vec::new(),
            }),
        }
    }

    /// Create a mock that returns failure with the given stderr and exit code.
    pub fn failure(stderr: &str, code: i32) -> Self {
        Self {
            output: Ok(CommandOutput {
                status: Self::make_exit_status(code),
                stdout: Vec::new(),
                stderr: stderr.as_bytes().to_vec(),
            }),
        }
    }

    /// Create a mock that returns a spawn error.
    pub fn spawn_error() -> Self {
        Self {
            output: Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "command not found",
            )),
        }
    }
}

#[cfg(test)]
impl CommandRunner for MockCommandRunner {
    fn run(&self, _program: &str, _args: &[&str]) -> std::io::Result<CommandOutput> {
        self.output.clone()
    }
}
```

- [ ] **Step 2: Add runner module to lib.rs**

```rust
// Add to top of crates/github/src/lib.rs
pub mod runner;
```

- [ ] **Step 3: Run cargo build to verify**

```bash
cargo build -p gitflow-cli-github
```

Expected: Build succeeds with no errors.

- [ ] **Step 4: Commit**

```bash
git add crates/github/src/runner.rs crates/github/src/lib.rs
git commit -m "feat(github): add CommandRunner trait abstraction"
```

---

## Task 2: GitLab CommandRunner Infrastructure

**Files:**
- Create: `crates/gitlab/src/runner.rs`
- Modify: `crates/gitlab/src/lib.rs:1` (add `pub mod runner;`)

**Interfaces:**
- Consumes: Nothing (foundation task)
- Produces: `CommandRunner` trait, `RealCommandRunner`, `MockCommandRunner`, `CommandOutput` (identical to GitHub)

- [ ] **Step 1: Create runner.rs (identical structure to GitHub)**

```rust
// crates/gitlab/src/runner.rs
//! Command execution abstraction for GitLab CLI (`glab`).

use std::process::ExitStatus;

/// Output from a CLI command execution.
#[derive(Debug, Clone)]
pub struct CommandOutput {
    /// Process exit status.
    pub status: ExitStatus,
    /// Standard output bytes.
    pub stdout: Vec<u8>,
    /// Standard error bytes.
    pub stderr: Vec<u8>,
}

/// Trait for executing CLI commands. Abstracts process spawning for testability.
pub trait CommandRunner: std::fmt::Debug + Send + Sync {
    /// Execute a command with the given program and arguments.
    ///
    /// # Errors
    ///
    /// Returns `std::io::Error` if the command cannot be spawned.
    fn run(&self, program: &str, args: &[&str]) -> std::io::Result<CommandOutput>;
}

/// Default implementation that spawns real processes.
#[derive(Debug, Clone, Default)]
pub struct RealCommandRunner;

impl CommandRunner for RealCommandRunner {
    fn run(&self, program: &str, args: &[&str]) -> std::io::Result<CommandOutput> {
        let output = std::process::Command::new(program).args(args).output()?;
        Ok(CommandOutput {
            status: output.status,
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }
}

/// Mock implementation for testing failure scenarios.
#[cfg(test)]
#[derive(Debug, Clone)]
pub struct MockCommandRunner {
    /// Pre-configured output to return.
    pub output: std::io::Result<CommandOutput>,
}

#[cfg(test)]
impl MockCommandRunner {
    #[cfg(unix)]
    fn make_exit_status(code: i32) -> ExitStatus {
        use std::os::unix::process::ExitStatusExt;
        ExitStatus::from_raw(code)
    }

    #[cfg(windows)]
    fn make_exit_status(code: i32) -> ExitStatus {
        use std::os::windows::process::ExitStatusExt;
        ExitStatus::from_raw(code as u32)
    }

    /// Create a mock that returns success with the given stdout.
    pub fn success(stdout: &str) -> Self {
        Self {
            output: Ok(CommandOutput {
                status: Self::make_exit_status(0),
                stdout: stdout.as_bytes().to_vec(),
                stderr: Vec::new(),
            }),
        }
    }

    /// Create a mock that returns failure with the given stderr and exit code.
    pub fn failure(stderr: &str, code: i32) -> Self {
        Self {
            output: Ok(CommandOutput {
                status: Self::make_exit_status(code),
                stdout: Vec::new(),
                stderr: stderr.as_bytes().to_vec(),
            }),
        }
    }

    /// Create a mock that returns a spawn error.
    pub fn spawn_error() -> Self {
        Self {
            output: Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "command not found",
            )),
        }
    }
}

#[cfg(test)]
impl CommandRunner for MockCommandRunner {
    fn run(&self, _program: &str, _args: &[&str]) -> std::io::Result<CommandOutput> {
        self.output.clone()
    }
}
```

- [ ] **Step 2: Add runner module to lib.rs**

```rust
// Add to top of crates/gitlab/src/lib.rs
pub mod runner;
```

- [ ] **Step 3: Run cargo build to verify**

```bash
cargo build -p gitflow-cli-gitlab
```

Expected: Build succeeds with no errors.

- [ ] **Step 4: Commit**

```bash
git add crates/gitlab/src/runner.rs crates/gitlab/src/lib.rs
git commit -m "feat(gitlab): add CommandRunner trait abstraction"
```

---

## Task 3: GitCode CommandRunner Infrastructure

**Files:**
- Create: `crates/gitcode/src/runner.rs`
- Modify: `crates/gitcode/src/lib.rs:1` (add `pub mod runner;`)

**Interfaces:**
- Consumes: Nothing (foundation task)
- Produces: `CommandRunner` trait, `RealCommandRunner`, `MockCommandRunner`, `CommandOutput` (identical structure)

- [ ] **Step 1: Create runner.rs (identical structure to GitHub/GitLab)**

```rust
// crates/gitcode/src/runner.rs
//! Command execution abstraction for GitCode CLI (`gitcode`/`gc`).

use std::process::ExitStatus;

/// Output from a CLI command execution.
#[derive(Debug, Clone)]
pub struct CommandOutput {
    /// Process exit status.
    pub status: ExitStatus,
    /// Standard output bytes.
    pub stdout: Vec<u8>,
    /// Standard error bytes.
    pub stderr: Vec<u8>,
}

/// Trait for executing CLI commands. Abstracts process spawning for testability.
pub trait CommandRunner: std::fmt::Debug + Send + Sync {
    /// Execute a command with the given program and arguments.
    ///
    /// # Errors
    ///
    /// Returns `std::io::Error` if the command cannot be spawned.
    fn run(&self, program: &str, args: &[&str]) -> std::io::Result<CommandOutput>;
}

/// Default implementation that spawns real processes.
#[derive(Debug, Clone, Default)]
pub struct RealCommandRunner;

impl CommandRunner for RealCommandRunner {
    fn run(&self, program: &str, args: &[&str]) -> std::io::Result<CommandOutput> {
        let output = std::process::Command::new(program).args(args).output()?;
        Ok(CommandOutput {
            status: output.status,
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }
}

/// Mock implementation for testing failure scenarios.
#[cfg(test)]
#[derive(Debug, Clone)]
pub struct MockCommandRunner {
    /// Pre-configured output to return.
    pub output: std::io::Result<CommandOutput>,
}

#[cfg(test)]
impl MockCommandRunner {
    #[cfg(unix)]
    fn make_exit_status(code: i32) -> ExitStatus {
        use std::os::unix::process::ExitStatusExt;
        ExitStatus::from_raw(code)
    }

    #[cfg(windows)]
    fn make_exit_status(code: i32) -> ExitStatus {
        use std::os::windows::process::ExitStatusExt;
        ExitStatus::from_raw(code as u32)
    }

    /// Create a mock that returns success with the given stdout.
    pub fn success(stdout: &str) -> Self {
        Self {
            output: Ok(CommandOutput {
                status: Self::make_exit_status(0),
                stdout: stdout.as_bytes().to_vec(),
                stderr: Vec::new(),
            }),
        }
    }

    /// Create a mock that returns failure with the given stderr and exit code.
    pub fn failure(stderr: &str, code: i32) -> Self {
        Self {
            output: Ok(CommandOutput {
                status: Self::make_exit_status(code),
                stdout: Vec::new(),
                stderr: stderr.as_bytes().to_vec(),
            }),
        }
    }

    /// Create a mock that returns a spawn error.
    pub fn spawn_error() -> Self {
        Self {
            output: Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "command not found",
            )),
        }
    }
}

#[cfg(test)]
impl CommandRunner for MockCommandRunner {
    fn run(&self, _program: &str, _args: &[&str]) -> std::io::Result<CommandOutput> {
        self.output.clone()
    }
}
```

- [ ] **Step 2: Add runner module to lib.rs**

```rust
// Add to top of crates/gitcode/src/lib.rs
pub mod runner;
```

- [ ] **Step 3: Run cargo build to verify**

```bash
cargo build -p gitflow-cli-gitcode
```

Expected: Build succeeds with no errors.

- [ ] **Step 4: Commit**

```bash
git add crates/gitcode/src/runner.rs crates/gitcode/src/lib.rs
git commit -m "feat(gitcode): add CommandRunner trait abstraction"
```

---

## Task 4: GitHub IssueProvider Refactor + Failure Tests

**Files:**
- Modify: `crates/github/src/issue.rs` (add generic parameter, replace Command::new with runner.run)
- Test: `crates/github/src/issue.rs` (add failure tests in existing `#[cfg(test)]` module)

**Interfaces:**
- Consumes: `CommandRunner` trait from `crate::runner`
- Produces: `GitHubIssueProvider<R: CommandRunner>` with backward-compatible `new()` and test-friendly `with_runner()`

- [ ] **Step 1: Write failing test for CLI failure scenario**

Add to the existing `#[cfg(test)] mod tests` in `crates/github/src/issue.rs`:

```rust
use crate::runner::MockCommandRunner;

#[tokio::test]
async fn test_should_return_platform_error_when_gh_fails_for_view() {
    let runner = MockCommandRunner::failure(
        r#"{"message": "Issue not found"}"#,
        256,
    );
    let provider = super::GitHubIssueProvider::with_runner("owner/repo".into(), runner);

    let result = provider.view(999).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), gitflow_cli_core::CoreError::Platform(_)));
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test -p gitflow-cli-github test_should_return_platform_error_when_gh_fails_for_view
```

Expected: FAIL — `with_runner` method doesn't exist yet.

- [ ] **Step 3: Refactor GitHubIssueProvider to use CommandRunner**

Modify `crates/github/src/issue.rs`:

```rust
use crate::runner::{CommandRunner, RealCommandRunner};

#[derive(Debug, Clone)]
pub struct GitHubIssueProvider<R: CommandRunner = RealCommandRunner> {
    repo: String,
    runner: R,
}

impl GitHubIssueProvider<RealCommandRunner> {
    #[must_use]
    pub fn new(repo: String) -> Self {
        Self {
            repo,
            runner: RealCommandRunner,
        }
    }
}

impl<R: CommandRunner> GitHubIssueProvider<R> {
    #[must_use]
    pub fn with_runner(repo: String, runner: R) -> Self {
        Self { repo, runner }
    }
}

#[async_trait]
impl<R: CommandRunner + 'static> IssueProvider for GitHubIssueProvider<R> {
    async fn view(&self, number: u64) -> Result<IssueData> {
        let output = self
            .runner
            .run("gh", &["issue", "view", &number.to_string(), "--repo", &self.repo, "--json", "number,title,state,body,url"])
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            return Err(CoreError::Platform(parse_gh_error(&output.stderr)));
        }

        serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)
    }

    // ... refactor all other methods similarly, replacing Command::new with self.runner.run
}
```

- [ ] **Step 4: Run test to verify it passes**

```bash
cargo test -p gitflow-cli-github test_should_return_platform_error_when_gh_fails_for_view
```

Expected: PASS

- [ ] **Step 5: Add remaining failure tests for IssueProvider**

Add to `#[cfg(test)] mod tests`:

```rust
#[tokio::test]
async fn test_should_return_serialization_error_on_invalid_json_for_view() {
    let runner = MockCommandRunner::success("not valid json");
    let provider = super::GitHubIssueProvider::with_runner("owner/repo".into(), runner);

    let result = provider.view(1).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), gitflow_cli_core::CoreError::Serialization(_)));
}

#[tokio::test]
async fn test_should_return_platform_error_when_gh_fails_for_list() {
    let runner = MockCommandRunner::failure(r#"{"message": "Forbidden"}"#, 256);
    let provider = super::GitHubIssueProvider::with_runner("owner/repo".into(), runner);

    let result = provider.list(gitflow_cli_core::ListIssueArgs::default()).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), gitflow_cli_core::CoreError::Platform(_)));
}

#[tokio::test]
async fn test_should_return_serialization_error_on_invalid_json_for_list() {
    let runner = MockCommandRunner::success("invalid");
    let provider = super::GitHubIssueProvider::with_runner("owner/repo".into(), runner);

    let result = provider.list(gitflow_cli_core::ListIssueArgs::default()).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), gitflow_cli_core::CoreError::Serialization(_)));
}

// Add tests for: create, close, reopen, comment, add_labels, remove_label
// Pattern: test_should_return_platform_error_when_gh_fails_for_{method}
//          test_should_return_serialization_error_on_invalid_json_for_{method} (if parses JSON)
```

- [ ] **Step 6: Run all GitHub issue tests**

```bash
cargo test -p gitflow-cli-github issue
```

Expected: All tests pass (existing + new failure tests).

- [ ] **Step 7: Run cargo clippy**

```bash
cargo clippy -p gitflow-cli-github --all-targets --all-features -- -D warnings -W clippy::pedantic
```

Expected: No warnings.

- [ ] **Step 8: Commit**

```bash
git add crates/github/src/issue.rs
git commit -m "test(github): add failure tests for IssueProvider"
```

---

## Task 5: GitHub PrProvider Refactor + Failure Tests

**Files:**
- Modify: `crates/github/src/pr.rs`
- Test: `crates/github/src/pr.rs`

**Interfaces:**
- Consumes: `CommandRunner` trait
- Produces: `GitHubPrProvider<R: CommandRunner>`

- [ ] **Step 1-8: Follow same pattern as Task 4**

Refactor `GitHubPrProvider` to use generic `R: CommandRunner`, then add failure tests for:
- `create`: CLI failure, URL parse failure
- `list`: CLI failure, invalid JSON
- `view`: CLI failure, invalid JSON
- `close`, `reopen`: CLI failure
- `merge`: CLI failure, invalid JSON
- `checkout`, `sync_branch`, `mark_ready`, `mark_wip`, `comment`: CLI failure

Estimated: ~14 failure tests

- [ ] **Commit**

```bash
git add crates/github/src/pr.rs
git commit -m "test(github): add failure tests for PrProvider"
```

---

## Task 6: GitHub ReleaseProvider Refactor + Failure Tests

**Files:**
- Modify: `crates/github/src/release.rs`
- Test: `crates/github/src/release.rs`

**Interfaces:**
- Consumes: `CommandRunner` trait
- Produces: `GitHubReleaseProvider<R: CommandRunner>`

- [ ] **Step 1-8: Follow same pattern**

Refactor and add failure tests for:
- `create`: CLI failure, invalid JSON
- `list`: CLI failure, invalid JSON
- `view`: CLI failure, invalid JSON
- `edit`: CLI failure, invalid JSON
- `delete`: CLI failure
- `upload_asset`, `download_asset`: CLI failure

Estimated: ~10 failure tests

- [ ] **Commit**

```bash
git add crates/github/src/release.rs
git commit -m "test(github): add failure tests for ReleaseProvider"
```

---

## Task 7: GitHub AuthProvider Refactor + Failure Tests

**Files:**
- Modify: `crates/github/src/auth.rs`
- Test: `crates/github/src/auth.rs`

**Interfaces:**
- Consumes: `CommandRunner` trait
- Produces: `GitHubAuthProvider<R: CommandRunner>`

- [ ] **Step 1-8: Follow same pattern**

Refactor and add failure tests for:
- `login`: CLI failure
- `logout`: CLI failure
- `status`: CLI failure, not-logged-in returns `AuthStatus { logged_in: false }` (not error)
- `token`: CLI failure, empty stdout

Estimated: ~5 failure tests

- [ ] **Commit**

```bash
git add crates/github/src/auth.rs
git commit -m "test(github): add failure tests for AuthProvider"
```

---

## Task 8: GitHub PipelineProvider Refactor + Failure Tests

**Files:**
- Modify: `crates/github/src/pipeline.rs`
- Test: `crates/github/src/pipeline.rs`

**Interfaces:**
- Consumes: `CommandRunner` trait
- Produces: `GitHubPipelineProvider<R: CommandRunner>`

- [ ] **Step 1-8: Follow same pattern**

Refactor and add failure tests for:
- `status`: CLI failure, invalid JSON
- `logs`: CLI failure
- `jobs`: CLI failure, invalid JSON
- `report`: CLI failure, invalid JSON

Estimated: ~6 failure tests

- [ ] **Commit**

```bash
git add crates/github/src/pipeline.rs
git commit -m "test(github): add failure tests for PipelineProvider"
```

---

## Task 9-13: GitLab Provider Refactors + Failure Tests

**Files:**
- Modify: `crates/gitlab/src/{issue,mr,release,auth,pipeline}.rs`
- Test: Same files

**Interfaces:**
- Consumes: `CommandRunner` trait from `crate::runner`
- Produces: `GitLabIssueProvider<R>`, `GitLabMrProvider<R>`, etc.

- [ ] **Tasks 9-13: Repeat Tasks 4-8 pattern for GitLab**

Each task:
1. Write failing test
2. Refactor provider to use `R: CommandRunner`
3. Add all failure tests (same matrix as GitHub)
4. Verify tests pass
5. Run clippy
6. Commit

Estimated: ~48 failure tests total for GitLab

- [ ] **Final commit for all GitLab providers**

```bash
git add crates/gitlab/src/
git commit -m "test(gitlab): add failure tests for all providers"
```

---

## Task 14-18: GitCode Provider Refactors + Failure Tests

**Files:**
- Modify: `crates/gitcode/src/{issue,pr,release,auth,pipeline}.rs`
- Test: Same files

**Interfaces:**
- Consumes: `CommandRunner` trait from `crate::runner`
- Produces: `GitCodeIssueProvider<R>`, `GitCodePrProvider<R>`, etc.

**Special Note:** GitCode `PipelineProvider` is a stub that always returns `Platform` error. Tests should verify this behavior.

- [ ] **Tasks 14-18: Repeat Tasks 4-8 pattern for GitCode**

Each task follows the same pattern. GitCode-specific differences:
- CLI binary is `gitcode` or `gc` (use `crate::gitcode_binary()`)
- `PipelineProvider` methods all return `CoreError::Platform("GitCode does not support...")` — test this

Estimated: ~46 failure tests total for GitCode

- [ ] **Final commit for all GitCode providers**

```bash
git add crates/gitcode/src/
git commit -m "test(gitcode): add failure tests for all providers"
```

---

## Task 19: Final Verification and Cleanup

**Files:**
- None (verification task)

- [ ] **Step 1: Run full test suite**

```bash
cargo test --all
```

Expected: All tests pass (existing + ~142 new failure tests).

- [ ] **Step 2: Run clippy on entire workspace**

```bash
cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic
```

Expected: No warnings.

- [ ] **Step 3: Verify backward compatibility**

```bash
# Check that existing code still compiles
cargo build --all
```

Expected: Build succeeds with no changes to calling code.

- [ ] **Step 4: Count new tests**

```bash
cargo test --all -- --list | grep -c "test_should_return_"
```

Expected: ~142 tests matching the naming pattern.

- [ ] **Step 5: Final commit (if any cleanup needed)**

```bash
git add -A
git commit -m "chore: final cleanup for Phase 4 coverage TDD"
```

---

## Summary

**Total Tasks:** 19
**Estimated New Tests:** ~142
**Files Modified:** 15 provider files + 3 runner files
**Backward Compatibility:** Maintained via default generic parameters
**Test Coverage:** All 5 traits × 3 platforms × failure paths

**Execution Strategy:** Complete Tasks 1-3 (infrastructure) first, then Tasks 4-8 (GitHub), then 9-13 (GitLab), then 14-18 (GitCode), then Task 19 (verification).
