//! Command execution abstraction for GitCode CLI (`gitcode`/`gc`).
//!
//! Re-exports the shared [`CommandRunner`] trait, [`CommandOutput`], and
//! [`RealCommandRunner`] from `gitflow-cli-adapter-utils`.
//! Platform-specific mock implementations live in this module for testing.

#[cfg(test)]
use std::process::ExitStatus;

pub use gitflow_cli_adapter_utils::{CommandOutput, CommandRunner, RealCommandRunner};

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

    async fn run_with_stdin(
        &self,
        program: &str,
        args: &[&str],
        _stdin_data: &[u8],
    ) -> std::io::Result<CommandOutput> {
        self.run(program, args).await
    }
}

/// Mock implementation that records every call's arguments while returning
/// a preconfigured result.
///
/// Used by regression tests that must assert the exact CLI invocation shape
/// (e.g. which flags the adapter passes to `gitcode`).
#[cfg(test)]
#[derive(Debug, Clone)]
pub struct RecordingMockRunner {
    inner: MockCommandRunner,
    calls: std::sync::Arc<std::sync::Mutex<Vec<Vec<String>>>>,
}

#[cfg(test)]
impl RecordingMockRunner {
    /// Create a recording runner that returns success with the given stdout.
    #[must_use]
    pub fn success(stdout: &str) -> Self {
        Self {
            inner: MockCommandRunner::success(stdout),
            calls: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    /// Create a recording runner that returns failure with the given stderr.
    #[must_use]
    pub fn failure(stderr: &str, code: i32) -> Self {
        Self {
            inner: MockCommandRunner::failure(stderr, code),
            calls: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    /// Snapshot of all recorded calls; each entry is the argv (without program).
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned (a precedent panic occurred while
    /// holding the lock).
    #[must_use]
    pub fn calls(&self) -> Vec<Vec<String>> {
        self.calls
            .lock()
            .expect("RecordingMockRunner mutex poisoned")
            .clone()
    }
}

#[cfg(test)]
#[async_trait::async_trait]
impl CommandRunner for RecordingMockRunner {
    async fn run(&self, program: &str, args: &[&str]) -> std::io::Result<CommandOutput> {
        self.calls
            .lock()
            .expect("RecordingMockRunner mutex poisoned")
            .push(args.iter().map(|s| (*s).to_owned()).collect());
        self.inner.run(program, args).await
    }

    async fn run_with_stdin(
        &self,
        program: &str,
        args: &[&str],
        stdin_data: &[u8],
    ) -> std::io::Result<CommandOutput> {
        self.calls
            .lock()
            .expect("RecordingMockRunner mutex poisoned")
            .push(args.iter().map(|s| (*s).to_owned()).collect());
        self.inner.run_with_stdin(program, args, stdin_data).await
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
}

#[cfg(test)]
impl SequencedMockCommandRunner {
    /// Build a runner that yields `outputs` in order, one per `run` call.
    #[must_use]
    pub fn new(outputs: Vec<CommandOutput>) -> Self {
        Self {
            responses: std::sync::Arc::new(std::sync::Mutex::new(outputs.into())),
        }
    }

    /// Convenience: build a sequence from `(success, stdout_or_stderr)` tuples.
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
}

#[cfg(test)]
#[async_trait::async_trait]
impl CommandRunner for SequencedMockCommandRunner {
    async fn run(&self, _program: &str, _args: &[&str]) -> std::io::Result<CommandOutput> {
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
            .run("gc", &["--version"])
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
            .run("gc", &["repo", "view"])
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
            .run("gc", &["--version"])
            .await
            .expect_err("should fail");
        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    }

    #[tokio::test]
    async fn test_should_clone_command_output() {
        let runner = MockCommandRunner::success("data");
        let output = runner.run("gc", &[]).await.expect("should succeed");
        let cloned = output.clone();
        assert_eq!(output.stdout, cloned.stdout);
        assert_eq!(output.stderr, cloned.stderr);
    }

    #[tokio::test]
    async fn test_should_clone_mock_runner() {
        let runner = MockCommandRunner::success("cloneable");
        let cloned = runner.clone();
        let output = cloned.run("gc", &[]).await.expect("should succeed");
        assert_eq!(output.stdout, b"cloneable");
    }

    #[tokio::test]
    async fn test_should_record_arguments_in_recording_runner() {
        let runner = RecordingMockRunner::success("{}");
        let output = runner
            .run("gc", &["pr", "view", "20", "--json"])
            .await
            .expect("should succeed");
        assert!(output.status.success());

        let calls = runner.calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0], vec!["pr", "view", "20", "--json"]);
    }

    #[tokio::test]
    async fn test_should_record_multiple_calls_in_order() {
        let runner = RecordingMockRunner::success("ok");
        runner
            .run("gc", &["issue", "label", "1", "--add", "bug"])
            .await
            .expect("first");
        runner
            .run("gc", &["issue", "label", "1", "--remove", "bug"])
            .await
            .expect("second");

        let calls = runner.calls();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[1], vec!["issue", "label", "1", "--remove", "bug"]);
    }

    #[tokio::test]
    async fn test_should_yield_sequenced_responses() {
        let runner = SequencedMockCommandRunner::from_results(&[
            (true, "first"),
            (false, "error"),
            (true, "third"),
        ]);

        let out1 = runner.run("gc", &[]).await.expect("first should succeed");
        assert!(out1.status.success());
        assert_eq!(out1.stdout, b"first");

        let out2 = runner
            .run("gc", &[])
            .await
            .expect("second should 'succeed' as Output");
        assert!(!out2.status.success());
        assert_eq!(out2.stderr, b"error");

        let out3 = runner.run("gc", &[]).await.expect("third should succeed");
        assert_eq!(out3.stdout, b"third");
    }

    #[tokio::test]
    async fn test_should_error_when_sequence_exhausted() {
        let runner = SequencedMockCommandRunner::from_results(&[(true, "only")]);

        let _ = runner.run("gc", &[]).await.expect("first call");
        let err = runner.run("gc", &[]).await.expect_err("exhausted");
        assert_eq!(err.kind(), std::io::ErrorKind::Other);
    }
}
