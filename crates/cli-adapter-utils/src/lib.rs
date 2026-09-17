//! Shared utilities for gf platform adapter crates.
//!
//! Provides the [`CommandRunner`] trait and [`RealCommandRunner`] implementation
//! for spawning CLI processes, along with the [`CommandOutput`] type for
//! representing command results.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

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
///
/// Platform adapter crates use this trait to spawn their respective CLI tools
/// (`gh`, `glab`, `gc`). Tests can inject mock implementations to control
/// command output without requiring the actual CLI tools to be installed.
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
///
/// Used in production by all platform adapter crates.
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

/// Read-only access to process environment variables.
///
/// Abstracts environment lookups so tests can inject deterministic values
/// instead of mutating process-global state. Mutating the real environment
/// races with other tests running in parallel threads of the same process.
pub trait EnvSource: std::fmt::Debug + Send + Sync {
    /// Return the value of `key`.
    ///
    /// Returns `None` when the variable is unset or its value is not valid
    /// UTF-8 — both cases are indistinguishable to callers, matching the
    /// `std::env::var(..).ok()` behaviour this trait replaces.
    fn var(&self, key: &str) -> Option<String>;
}

/// Default [`EnvSource`] reading the real process environment.
///
/// Used in production by all platform adapter crates.
#[derive(Debug, Clone, Copy, Default)]
pub struct RealEnv;

impl EnvSource for RealEnv {
    fn var(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }
}

/// In-memory [`EnvSource`] for tests.
///
/// Holds a fixed set of variables and never touches the process environment,
/// so tests using it are immune both to parallel-test interference and to
/// whatever the developer happens to have exported in their shell.
///
/// # Examples
///
/// ```
/// use gitflow_cli_adapter_utils::{EnvSource, MockEnv};
///
/// let env = MockEnv::with("GL_TOKEN", "glpat-example");
/// assert_eq!(env.var("GL_TOKEN"), Some("glpat-example".to_string()));
/// assert_eq!(env.var("GITLAB_HOST"), None);
/// ```
#[cfg(feature = "test-util")]
#[derive(Debug, Clone, Default)]
pub struct MockEnv {
    /// Fixed variable table consulted by [`EnvSource::var`].
    vars: std::collections::HashMap<String, String>,
}

#[cfg(feature = "test-util")]
impl MockEnv {
    /// Create a source where every lookup returns `None`.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Create a source holding exactly one variable.
    #[must_use]
    pub fn with(key: &str, value: &str) -> Self {
        let mut vars = std::collections::HashMap::new();
        vars.insert(key.to_string(), value.to_string());
        Self { vars }
    }
}

#[cfg(feature = "test-util")]
impl EnvSource for MockEnv {
    fn var(&self, key: &str) -> Option<String> {
        self.vars.get(key).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "test-util")]
    #[test]
    fn test_should_return_none_from_mock_env_when_key_absent() {
        let env = MockEnv::empty();
        assert_eq!(env.var("GL_TOKEN"), None);
    }

    #[cfg(feature = "test-util")]
    #[test]
    fn test_should_return_value_from_mock_env_when_key_present() {
        let env = MockEnv::with("GL_TOKEN", "glpat-mock");
        assert_eq!(env.var("GL_TOKEN"), Some("glpat-mock".to_string()));
    }

    #[cfg(feature = "test-util")]
    #[test]
    fn test_should_isolate_unrelated_keys_in_mock_env() {
        let env = MockEnv::with("GL_TOKEN", "glpat-mock");
        assert_eq!(env.var("GITLAB_HOST"), None);
    }

    #[test]
    fn test_should_read_real_process_env_through_real_env() {
        // PATH 在所有支持的平台上均已设置，且本测试只读不写，
        // 因此不会与并行测试产生竞态。
        let env = RealEnv;
        assert!(env.var("PATH").is_some());
    }

    #[test]
    fn test_should_return_none_from_real_env_for_absent_key() {
        let env = RealEnv;
        assert_eq!(env.var("GF_DEFINITELY_NOT_SET_12345"), None);
    }
}
