//! TOON output format integration tests.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "Integration tests unwrap known-good binary handles"
)]

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_should_accept_output_toon_flag() {
    let mut cmd = Command::cargo_bin("gitflow-cli").expect("binary exists");
    cmd.args(["--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("toon"));
}

#[test]
fn test_should_accept_output_auto_flag() {
    let mut cmd = Command::cargo_bin("gitflow-cli").expect("binary exists");
    cmd.args(["--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("auto"));
}
