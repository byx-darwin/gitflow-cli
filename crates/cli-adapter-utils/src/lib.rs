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
