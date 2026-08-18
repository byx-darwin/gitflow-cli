//! Command execution abstraction for GitLab CLI (`glab`).
//!
//! This module provides the [`CommandRunner`] trait and its implementations,
//! allowing the GitLab crate to spawn CLI processes in production and inject
//! controlled outputs in tests.

use std::process::ExitStatus;
#[cfg(test)]
use std::sync::Arc;

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

    /// Execute a command with the given program and arguments, writing
    /// `stdin_data` to the child process's standard input.
    ///
    /// This avoids exposing sensitive values (such as tokens) in process
    /// arguments, where they would be visible to other users via `ps`.
    ///
    /// # Errors
    ///
    /// Returns [`std::io::Error`] if the command cannot be spawned or if
    /// writing to its standard input fails.
    async fn run_with_stdin(
        &self,
        program: &str,
        args: &[&str],
        stdin_data: &[u8],
    ) -> std::io::Result<CommandOutput>;
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

    async fn run_with_stdin(
        &self,
        program: &str,
        args: &[&str],
        stdin_data: &[u8],
    ) -> std::io::Result<CommandOutput> {
        use tokio::io::AsyncWriteExt;

        let mut child = tokio::process::Command::new(program)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(stdin_data).await?;
            drop(stdin);
        }

        let output = child.wait_with_output().await?;
        Ok(CommandOutput {
            status: output.status,
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }
}

/// A single recorded command invocation: `(program, args)`.
#[cfg(test)]
type RecordedCall = (String, Vec<String>);

/// All recorded command invocations in execution order.
#[cfg(test)]
type RecordedCalls = Vec<RecordedCall>;

/// Mock implementation for testing failure scenarios.
///
/// Stores either a success output or an error kind with a message,
/// enabling `Clone` without requiring [`std::io::Error`] itself to be cloneable.
#[cfg(test)]
#[derive(Debug, Clone)]
pub struct MockCommandRunner {
    result: MockResult,
    /// Recorded `(program, args)` sequences for every `run`/`run_with_stdin` call.
    recorded: Arc<std::sync::Mutex<RecordedCalls>>,
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
    pub(crate) fn make_exit_status(code: i32) -> ExitStatus {
        use std::os::unix::process::ExitStatusExt;
        ExitStatus::from_raw(code)
    }

    #[cfg(windows)]
    pub(crate) fn make_exit_status(code: i32) -> ExitStatus {
        use std::os::windows::process::ExitStatusExt;
        // Exit codes are conventionally non-negative; clamp negatives to 1.
        let raw = u32::try_from(code).unwrap_or(1);
        ExitStatus::from_raw(raw)
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
            recorded: Arc::new(std::sync::Mutex::new(Vec::new())),
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
            recorded: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    /// Create a mock that returns a spawn error.
    #[must_use]
    pub fn spawn_error() -> Self {
        Self {
            result: MockResult::Error(std::io::ErrorKind::NotFound, "command not found".to_owned()),
            recorded: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    /// Return the recorded `(program, args)` sequences from every executed call.
    ///
    /// # Panics
    ///
    /// Panics if the internal recording mutex is poisoned (a prior panic while
    /// holding the lock).
    #[must_use]
    pub fn recorded_calls(&self) -> Vec<(String, Vec<String>)> {
        self.recorded.lock().expect("mock mutex poisoned").clone()
    }
}

#[cfg(test)]
#[async_trait::async_trait]
impl CommandRunner for MockCommandRunner {
    async fn run(&self, program: &str, args: &[&str]) -> std::io::Result<CommandOutput> {
        self.recorded.lock().expect("mock mutex poisoned").push((
            program.to_string(),
            args.iter().map(|s| (*s).to_string()).collect(),
        ));
        match &self.result {
            MockResult::Output(output) => Ok(output.clone()),
            MockResult::Error(kind, message) => Err(std::io::Error::new(*kind, message.clone())),
        }
    }

    async fn run_with_stdin(
        &self,
        program: &str,
        args: &[&str],
        _stdin_data: &[u8],
    ) -> std::io::Result<CommandOutput> {
        self.recorded.lock().expect("mock mutex poisoned").push((
            program.to_string(),
            args.iter().map(|s| (*s).to_string()).collect(),
        ));
        self.run(program, args).await
    }
}

/// Mock implementation that returns a sequence of preconfigured responses.
///
/// Each call to [`CommandRunner::run`] pops the next response from the queue.
/// Useful for testing retry logic where different commands must succeed or fail
/// in a specific order (e.g., `add_labels` retries after auto-creating a label).
#[cfg(test)]
#[derive(Debug, Clone)]
pub struct SequencedMockCommandRunner {
    responses: std::sync::Arc<std::sync::Mutex<std::collections::VecDeque<CommandOutput>>>,
    /// Recorded `(program, args)` sequences for every `run`/`run_with_stdin` call.
    recorded: Arc<std::sync::Mutex<RecordedCalls>>,
}

#[cfg(test)]
impl SequencedMockCommandRunner {
    /// Build a runner that yields `outputs` in order, one per `run` call.
    #[must_use]
    pub fn new(outputs: Vec<CommandOutput>) -> Self {
        Self {
            responses: std::sync::Arc::new(std::sync::Mutex::new(outputs.into())),
            recorded: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    /// Convenience: build a sequence from `(success, stdout_or_stderr)` tuples.
    ///
    /// When `success` is `true`, the string is used as stdout with exit code 0.
    /// When `false`, it is used as stderr with exit code 1.
    #[must_use]
    pub fn from_results(results: &[(bool, &str)]) -> Self {
        let outputs = results
            .iter()
            .map(|&(ok, text)| {
                if ok {
                    CommandOutput {
                        status: MockCommandRunner::make_exit_status(0),
                        stdout: text.as_bytes().to_vec(),
                        stderr: Vec::new(),
                    }
                } else {
                    CommandOutput {
                        status: MockCommandRunner::make_exit_status(1),
                        stdout: Vec::new(),
                        stderr: text.as_bytes().to_vec(),
                    }
                }
            })
            .collect();
        Self::new(outputs)
    }

    /// Return the recorded `(program, args)` sequences from every executed call.
    ///
    /// # Panics
    ///
    /// Panics if the internal recording mutex is poisoned (a prior panic while
    /// holding the lock).
    #[must_use]
    pub fn recorded_calls(&self) -> Vec<(String, Vec<String>)> {
        self.recorded.lock().expect("mock mutex poisoned").clone()
    }
}

#[cfg(test)]
#[async_trait::async_trait]
impl CommandRunner for SequencedMockCommandRunner {
    async fn run(&self, program: &str, args: &[&str]) -> std::io::Result<CommandOutput> {
        self.recorded.lock().expect("mock mutex poisoned").push((
            program.to_string(),
            args.iter().map(|s| (*s).to_string()).collect(),
        ));
        let mut guard = self
            .responses
            .lock()
            .expect("SequencedMockCommandRunner mutex poisoned");
        guard
            .pop_front()
            .ok_or_else(|| std::io::Error::other("no more responses"))
    }

    async fn run_with_stdin(
        &self,
        program: &str,
        args: &[&str],
        _stdin_data: &[u8],
    ) -> std::io::Result<CommandOutput> {
        self.run(program, args).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_should_return_success_output_from_mock() {
        let runner = MockCommandRunner::success("hello");
        let output = runner
            .run("glab", &["--version"])
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
            .run("glab", &["repo", "view"])
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
            .run("glab", &["--version"])
            .await
            .expect_err("should fail");
        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    }

    #[tokio::test]
    async fn test_should_clone_command_output() {
        let runner = MockCommandRunner::success("data");
        let output = runner.run("glab", &[]).await.expect("should succeed");
        let cloned = output.clone();
        assert_eq!(output.stdout, cloned.stdout);
        assert_eq!(output.stderr, cloned.stderr);
    }

    #[tokio::test]
    async fn test_should_clone_mock_runner() {
        let runner = MockCommandRunner::success("cloneable");
        let cloned = runner.clone();
        let output = cloned.run("glab", &[]).await.expect("should succeed");
        assert_eq!(output.stdout, b"cloneable");
    }

    #[tokio::test]
    async fn test_should_yield_sequenced_responses() {
        let runner = SequencedMockCommandRunner::from_results(&[
            (true, "first"),
            (false, "error"),
            (true, "third"),
        ]);

        let out1 = runner.run("glab", &[]).await.expect("first should succeed");
        assert!(out1.status.success());
        assert_eq!(out1.stdout, b"first");

        let out2 = runner
            .run("glab", &[])
            .await
            .expect("second should 'succeed' as Output");
        assert!(!out2.status.success());
        assert_eq!(out2.stderr, b"error");

        let out3 = runner.run("glab", &[]).await.expect("third should succeed");
        assert_eq!(out3.stdout, b"third");
    }

    #[tokio::test]
    async fn test_should_error_when_sequence_exhausted() {
        let runner = SequencedMockCommandRunner::from_results(&[(true, "only")]);

        let _ = runner.run("glab", &[]).await.expect("first call");
        let err = runner.run("glab", &[]).await.expect_err("exhausted");
        assert_eq!(err.kind(), std::io::ErrorKind::Other);
    }

    #[tokio::test]
    async fn test_should_record_glab_calls() {
        let runner = MockCommandRunner::success("ok");
        runner
            .run("glab", &["issue", "close", "42", "--repo", "owner/repo"])
            .await
            .expect("should succeed");
        let calls = runner.recorded_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, "glab");
        assert_eq!(
            calls[0].1,
            vec!["issue", "close", "42", "--repo", "owner/repo"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn test_should_record_sequenced_glab_calls() {
        let runner = SequencedMockCommandRunner::from_results(&[(true, "first"), (true, "second")]);
        runner
            .run("glab", &["issue", "list", "--repo", "owner/repo"])
            .await
            .expect("first should succeed");
        runner
            .run("glab", &["issue", "view", "1", "--repo", "owner/repo"])
            .await
            .expect("second should succeed");
        let calls = runner.recorded_calls();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].0, "glab");
        assert_eq!(calls[1].0, "glab");
        assert_eq!(
            calls[1].1,
            vec!["issue", "view", "1", "--repo", "owner/repo"]
        );
    }
}
