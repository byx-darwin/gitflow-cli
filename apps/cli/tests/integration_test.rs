//! End-to-end integration tests for the `gitflow-cli` binary.
//!
//! These tests invoke the compiled binary via `assert_cmd` to verify
//! that top-level flags (`--help`, `--version`) exit successfully.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "Integration tests unwrap known-good binary handles"
)]

use assert_cmd::Command;

#[test]
fn test_help_succeeds() {
    let mut cmd = Command::cargo_bin("gitflow-cli").unwrap();
    cmd.arg("--help");
    cmd.assert().success();
}

#[test]
fn test_version_succeeds() {
    let mut cmd = Command::cargo_bin("gitflow-cli").unwrap();
    cmd.arg("--version");
    cmd.assert().success();
}
