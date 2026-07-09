//! Command execution abstraction for GitHub CLI (`gh`).
//!
//! This module provides the [`CommandRunner`] trait and its implementations,
//! allowing the GitHub crate to spawn CLI processes in production and inject
//! controlled outputs in tests.

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
#[async_trait::async_trait]
pub trait CommandRunner: std::fmt::Debug + Send + Sync {
    /// Execute a command with the given program and arguments.
    ///
    /// # Errors
    ///
    /// Returns [`std::io::Error`] if the command cannot be spawned.
    async fn run(&self, program: &str, args: &[&str]) -> std::io::Result<CommandOutput>;
}

/// Default implementation that spawns real processes via [`tokio::process::Command`].
#[derive(Debug, Clone, Default)]
pub struct RealCommandRunner;

#[async_trait::async_trait]
impl CommandRunner for RealCommandRunner {
    async fn run(&self, program: &str, args: &[&str]) -> std::io::Result<CommandOutput> {
        let output = tokio::process::Command::new(program)
            .args(args)
            .output()
            .await?;
        Ok(CommandOutput {
            status: output.status,
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }
}

/// Mock implementation for testing failure scenarios.
///
/// Stores either a success output or an error kind with a message,
/// enabling `Clone` without requiring [`std::io::Error`] itself to be cloneable.
#[cfg(test)]
#[derive(Debug, Clone)]
pub struct MockCommandRunner {
    result: MockResult,
}

#[cfg(test)]
#[derive(Debug, Clone)]
enum MockResult {
    /// Successful command output.
    Output(CommandOutput),
    /// Spawn error with kind and message.
    Error(std::io::ErrorKind, String),
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
    #[must_use]
    pub fn success(stdout: &str) -> Self {
        Self {
            result: MockResult::Output(CommandOutput {
                status: Self::make_exit_status(0),
                stdout: stdout.as_bytes().to_vec(),
                stderr: Vec::new(),
            }),
        }
    }

    /// Create a mock that returns failure with the given stderr and exit code.
    #[must_use]
    pub fn failure(stderr: &str, code: i32) -> Self {
        Self {
            result: MockResult::Output(CommandOutput {
                status: Self::make_exit_status(code),
                stdout: Vec::new(),
                stderr: stderr.as_bytes().to_vec(),
            }),
        }
    }

    /// Create a mock that returns a spawn error.
    #[must_use]
    pub fn spawn_error() -> Self {
        Self {
            result: MockResult::Error(std::io::ErrorKind::NotFound, "command not found".to_owned()),
        }
    }
}

#[cfg(test)]
#[async_trait::async_trait]
impl CommandRunner for MockCommandRunner {
    async fn run(&self, _program: &str, _args: &[&str]) -> std::io::Result<CommandOutput> {
        match &self.result {
            MockResult::Output(output) => Ok(output.clone()),
            MockResult::Error(kind, message) => Err(std::io::Error::new(*kind, message.clone())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_should_return_success_output_from_mock() {
        let runner = MockCommandRunner::success("hello");
        let output = runner
            .run("gh", &["--version"])
            .await
            .expect("should succeed");
        assert!(output.status.success());
        assert_eq!(output.stdout, b"hello");
        assert!(output.stderr.is_empty());
    }

    #[tokio::test]
    async fn test_should_return_failure_output_from_mock() {
        let runner = MockCommandRunner::failure("not found", 1);
        let output = runner
            .run("gh", &["repo", "view"])
            .await
            .expect("should succeed");
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert_eq!(output.stderr, b"not found");
    }

    #[tokio::test]
    async fn test_should_return_spawn_error_from_mock() {
        let runner = MockCommandRunner::spawn_error();
        let err = runner
            .run("gh", &["--version"])
            .await
            .expect_err("should fail");
        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    }

    #[tokio::test]
    async fn test_should_clone_command_output() {
        let runner = MockCommandRunner::success("data");
        let output = runner.run("gh", &[]).await.expect("should succeed");
        let cloned = output.clone();
        assert_eq!(output.stdout, cloned.stdout);
        assert_eq!(output.stderr, cloned.stderr);
    }

    #[tokio::test]
    async fn test_should_clone_mock_runner() {
        let runner = MockCommandRunner::success("cloneable");
        let cloned = runner.clone();
        let output = cloned.run("gh", &[]).await.expect("should succeed");
        assert_eq!(output.stdout, b"cloneable");
    }
}
