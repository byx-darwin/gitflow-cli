# Design: Phase 4 Coverage TDD — Failure Tests for All Trait Methods

**Date**: 2026-07-09
**Issue**: #72
**Status**: Draft

## Context

Current test coverage (~440 tests) focuses almost exclusively on happy paths: construction, deserialization, URL parsing. No tests cover failure scenarios where CLI commands fail or return invalid output. This leaves critical error-handling code untested.

## Goal

Add comprehensive failure tests for all 5 core traits across all 3 platform implementations:
- **IssueProvider**: create/list/view/close/reopen/comment/add_labels/remove_label
- **PrProvider**: create/list/view/close/reopen/merge/checkout/mark_ready/mark_wip/sync_branch/comment
- **ReleaseProvider**: create/list/view/edit/delete/upload_asset/download_asset
- **AuthProvider**: login/logout/status/token
- **PipelineProvider**: status/logs/jobs/report

Total: **15 provider implementations × ~10 methods = ~142 failure tests**

## Design

### 1. CommandRunner Trait Architecture

Each platform crate defines its own `CommandRunner` trait independently:

```
crates/github/src/runner.rs   → GhCommandRunner trait
crates/gitlab/src/runner.rs   → GlabCommandRunner trait
crates/gitcode/src/runner.rs  → GcCommandRunner trait
```

#### Trait Definition

```rust
#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub status: std::process::ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

pub trait CommandRunner: std::fmt::Debug + Send + Sync {
    fn run(&self, program: &str, args: &[&str]) -> std::io::Result<CommandOutput>;
}
```

#### Default Implementation

```rust
#[derive(Debug, Clone, Default)]
pub struct RealCommandRunner;

impl CommandRunner for RealCommandRunner {
    fn run(&self, program: &str, args: &[&str]) -> std::io::Result<CommandOutput> {
        let output = std::process::Command::new(program)
            .args(args)
            .output()?;
        Ok(CommandOutput {
            status: output.status,
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }
}
```

#### Mock Implementation (test-only, cross-platform)

```rust
#[cfg(test)]
#[derive(Debug, Clone)]
pub struct MockCommandRunner {
    pub output: std::io::Result<CommandOutput>,
}

#[cfg(test)]
impl MockCommandRunner {
    #[cfg(unix)]
    fn make_exit_status(code: i32) -> std::process::ExitStatus {
        use std::os::unix::process::ExitStatusExt;
        ExitStatus::from_raw(code)
    }

    #[cfg(windows)]
    fn make_exit_status(code: i32) -> std::process::ExitStatus {
        use std::os::windows::process::ExitStatusExt;
        ExitStatus::from_raw(code as u32)
    }

    pub fn success(stdout: &str) -> Self {
        Self {
            output: Ok(CommandOutput {
                status: Self::make_exit_status(0),
                stdout: stdout.as_bytes().to_vec(),
                stderr: Vec::new(),
            }),
        }
    }

    pub fn failure(stderr: &str, code: i32) -> Self {
        Self {
            output: Ok(CommandOutput {
                status: Self::make_exit_status(code),
                stdout: Vec::new(),
                stderr: stderr.as_bytes().to_vec(),
            }),
        }
    }

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

#### ExitStatus Handling

Cross-platform construction using conditional compilation:
- **Unix**: `std::os::unix::process::ExitStatusExt::from_raw(code)`
- **Windows**: `std::os::windows::process::ExitStatusExt::from_raw(code as u32)`
- **Semantics**: `0` = success, non-zero = failure (e.g., 256 = exit code 1, 512 = exit code 2)

### 2. Provider Refactoring Pattern

Each Provider struct gains a generic parameter `R: CommandRunner`:

**Before:**
```rust
#[derive(Debug, Clone)]
pub struct GitHubIssueProvider {
    repo: String,
}

impl GitHubIssueProvider {
    pub fn new(repo: String) -> Self {
        Self { repo }
    }
}
```

**After:**
```rust
#[derive(Debug, Clone)]
pub struct GitHubIssueProvider<R: CommandRunner = RealCommandRunner> {
    repo: String,
    runner: R,
}

impl GitHubIssueProvider<RealCommandRunner> {
    pub fn new(repo: String) -> Self {
        Self {
            repo,
            runner: RealCommandRunner,
        }
    }
}

impl<R: CommandRunner> GitHubIssueProvider<R> {
    pub fn with_runner(repo: String, runner: R) -> Self {
        Self { repo, runner }
    }
}

impl<R: CommandRunner + 'static> IssueProvider for GitHubIssueProvider<R> {
    async fn create(&self, args: CreateIssueArgs) -> Result<IssueData> {
        let output = self.runner
            .run("gh", &["issue", "create", ...])
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

        if !output.status.success() {
            return Err(CoreError::Platform(parse_gh_error(&output.stderr)));
        }

        let data: IssueData = serde_json::from_slice(&output.stdout)
            .map_err(CoreError::Serialization)?;
        Ok(data)
    }
}
```

#### Key Changes

1. **Default generic parameter** `R = RealCommandRunner`: Existing code `GitHubIssueProvider::new(repo)` works unchanged
2. **`with_runner` constructor**: Tests use `GitHubIssueProvider::with_runner(repo, mock_runner)`
3. **All `std::process::Command::new()` calls** replaced with `self.runner.run()`
4. **Error handling unchanged**: `map_err(CoreError::Platform)`, `map_err(CoreError::Serialization)` remain the same

#### Refactoring Scope

| Platform | Files to Refactor | Provider Count |
|----------|-------------------|----------------|
| GitHub   | issue.rs, pr.rs, release.rs, auth.rs, pipeline.rs | 5 |
| GitLab   | issue.rs, mr.rs, release.rs, auth.rs, pipeline.rs | 5 |
| GitCode  | issue.rs, pr.rs, release.rs, auth.rs, pipeline.rs | 5 |

Total: **15 providers** need generic parameter added.

#### Backward Compatibility

- `GitHubIssueProvider::new(repo)` still works (uses default generic)
- Existing code `let provider = GitHubIssueProvider::new(repo);` **requires no changes**
- Only tests explicitly use `with_runner`

### 3. Test Patterns

All failure tests follow a unified pattern:

```rust
#[cfg(test)]
mod failure_tests {
    use super::*;
    use crate::runner::{MockCommandRunner, CommandOutput};

    #[tokio::test]
    async fn test_should_return_platform_error_when_cli_fails() {
        let runner = MockCommandRunner::failure(
            r#"{"message": "Not Found", "code": 404}"#,
            256,
        );
        let provider = GitHubIssueProvider::with_runner("owner/repo".into(), runner);

        let result = provider.view(999).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::Platform(_)));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json() {
        let runner = MockCommandRunner::success("not valid json");
        let provider = GitHubIssueProvider::with_runner("owner/repo".into(), runner);

        let result = provider.list(ListIssueArgs::default()).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::Serialization(_)));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_spawn_fails() {
        let runner = MockCommandRunner::spawn_error();
        let provider = GitHubIssueProvider::with_runner("owner/repo".into(), runner);

        let result = provider.view(1).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::Platform(_)));
    }
}
```

#### Test Naming Convention

```
test_should_return_platform_error_when_cli_fails_for_{method}
test_should_return_serialization_error_on_invalid_json_for_{method}
test_should_return_platform_error_when_url_parse_fails_for_create
test_should_return_io_error_when_stdin_write_fails_for_login
test_should_return_platform_error_when_spawn_fails_for_{method}
```

### 4. Failure Coverage Matrix

#### IssueProvider (3 platforms × 8 methods)

| Method | Test Scenarios | Expected Error Type |
|--------|---------------|---------------------|
| `create` | CLI failure (stderr has error) | `Platform` |
| `create` | URL parse failure (stdout has no valid URL) | `Platform` |
| `list` | CLI failure | `Platform` |
| `list` | Invalid JSON stdout | `Serialization` |
| `view` | CLI failure (issue doesn't exist) | `Platform` |
| `view` | Invalid JSON | `Serialization` |
| `close` | CLI failure | `Platform` |
| `close` | Invalid JSON | `Serialization` |
| `reopen` | CLI failure | `Platform` |
| `reopen` | Invalid JSON | `Serialization` |
| `comment` | CLI failure | `Platform` |
| `add_labels` | CLI failure | `Platform` |
| `remove_label` | CLI failure | `Platform` |

#### PrProvider (3 platforms × 11 methods)

| Method | Test Scenarios | Expected Error Type |
|--------|---------------|---------------------|
| `create` | CLI failure | `Platform` |
| `create` | URL parse failure | `Platform` |
| `list` | CLI failure / Invalid JSON | `Platform` / `Serialization` |
| `view` | CLI failure / Invalid JSON | `Platform` / `Serialization` |
| `close` | CLI failure | `Platform` |
| `reopen` | CLI failure | `Platform` |
| `merge` | CLI failure (merge conflict) | `Platform` |
| `merge` | Invalid JSON | `Serialization` |
| `checkout` | CLI failure | `Platform` |
| `sync_branch` | CLI failure | `Platform` |
| `mark_ready` | CLI failure | `Platform` |
| `mark_wip` | CLI failure | `Platform` |
| `comment` | CLI failure | `Platform` |

#### ReleaseProvider (3 platforms × 7 methods)

| Method | Test Scenarios | Expected Error Type |
|--------|---------------|---------------------|
| `create` | CLI failure | `Platform` |
| `create` | Invalid JSON | `Serialization` |
| `list` | CLI failure / Invalid JSON | `Platform` / `Serialization` |
| `view` | CLI failure (tag doesn't exist) | `Platform` |
| `view` | Invalid JSON | `Serialization` |
| `edit` | CLI failure / Invalid JSON | `Platform` / `Serialization` |
| `delete` | CLI failure | `Platform` |
| `upload_asset` | CLI failure | `Platform` |
| `download_asset` | CLI failure | `Platform` |
| `download_asset` | rename failure (GitHub/GitLab only) | `Platform` |

#### AuthProvider (3 platforms × 4 methods)

| Method | Test Scenarios | Expected Error Type |
|--------|---------------|---------------------|
| `login` | CLI failure | `Platform` |
| `login` | stdin write failure | `Io` |
| `logout` | CLI failure | `Platform` |
| `status` | CLI failure | `Platform` |
| `status` | Not logged in (stderr contains "not logged in") | Returns `AuthStatus { logged_in: false }`, not error |
| `token` | CLI failure | `Platform` |
| `token` | stdout is empty | `Platform` |

#### PipelineProvider (3 platforms × 4 methods)

| Method | Test Scenarios | Expected Error Type |
|--------|---------------|---------------------|
| `status` | CLI failure | `Platform` |
| `status` | Invalid JSON | `Serialization` |
| `logs` | CLI failure (pipeline doesn't exist) | `Platform` |
| `jobs` | CLI failure / Invalid JSON | `Platform` / `Serialization` |
| `report` | CLI failure | `Platform` |
| `report` | Invalid JSON | `Serialization` |
| **GitCode special** | All 4 methods | Returns `Platform("GitCode does not support...")` |

### 5. Estimated Test Count

| Platform | Issue | PR | Release | Auth | Pipeline | Total |
|----------|-------|-----|---------|------|----------|-------|
| GitHub   | ~13   | ~14 | ~10     | ~5   | ~6       | ~48   |
| GitLab   | ~13   | ~14 | ~10     | ~5   | ~6       | ~48   |
| GitCode  | ~13   | ~14 | ~10     | ~5   | ~4 (stub)| ~46   |
| **Total**|       |     |         |      |          | **~142** |

## Implementation Strategy

### Phase 1: Infrastructure (per platform)

1. Create `runner.rs` with `CommandRunner` trait + `RealCommandRunner` + `MockCommandRunner`
2. Add `mod runner;` to `lib.rs`
3. Verify with `cargo build`

### Phase 2: Provider Refactoring (per platform)

For each provider:
1. Add generic parameter `R: CommandRunner = RealCommandRunner`
2. Add `runner: R` field
3. Implement `with_runner` constructor
4. Replace all `std::process::Command::new()` with `self.runner.run()`
5. Verify with `cargo build` and `cargo test` (existing tests should still pass)

### Phase 3: Failure Tests (per platform)

For each provider method:
1. Write `test_should_return_platform_error_when_cli_fails_for_{method}`
2. Write `test_should_return_serialization_error_on_invalid_json_for_{method}` (if method parses JSON)
3. Write special cases (URL parse failure, stdin failure, rename failure)
4. Verify with `cargo test`

### Order

Complete GitHub first as template, then replicate to GitLab and GitCode.

## Acceptance Criteria

- [ ] All 5 traits have failure tests across all 3 platforms
- [ ] Every failure assertion uses `matches!()` to verify `CoreError` variant
- [ ] All existing tests continue to pass (backward compatibility)
- [ ] No `unwrap()` or `expect()` in production code
- [ ] `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic` passes
- [ ] `cargo test` shows ~142 new failure tests

## Out of Scope

- Integration tests that actually call CLI binaries
- Tests for `AuthChecker` trait (separate concern)
- Tests for `SafePath` validation (already covered in core)
- Tests for error message content (only error type is verified)

## Risks

1. **Refactoring complexity**: Adding generic parameters to 15 providers is invasive. Mitigation: default generic ensures backward compatibility.
2. **Test maintenance**: Mock runners need to stay in sync with actual CLI behavior. Mitigation: mocks are simple and localized.
