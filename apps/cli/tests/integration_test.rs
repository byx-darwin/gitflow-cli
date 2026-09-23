//! End-to-end integration tests for the `gf` binary.
//!
//! These tests invoke the compiled binary via `assert_cmd` to verify
//! that top-level flags (`--help`, `--version`) exit successfully.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "Integration tests unwrap known-good binary handles"
)]

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help_succeeds() {
    let mut cmd = Command::cargo_bin("gf").unwrap();
    cmd.arg("--help");
    cmd.assert().success();
}

#[test]
fn test_version_succeeds() {
    let mut cmd = Command::cargo_bin("gf").unwrap();
    cmd.arg("--version");
    cmd.assert().success();
}

/// The deprecated `run` subcommand must be fully removed from the CLI
/// surface: `gf --help` should no longer list it (issue #294).
#[test]
fn test_should_not_list_run_subcommand_in_help() {
    let mut cmd = Command::cargo_bin("gf").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("run        Run the main application workflow").not());
}

/// After removal, `gf run` must be rejected by clap as an unrecognized
/// subcommand rather than dispatched to the (deleted) deprecation stub.
#[test]
fn test_should_reject_run_as_unrecognized_subcommand() {
    let mut cmd = Command::cargo_bin("gf").unwrap();
    cmd.arg("run");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand 'run'"));
}
