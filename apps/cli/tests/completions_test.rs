//! Integration tests for the `gitflow-cli completions` subcommand.
//!
//! These tests exercise the compiled binary via `assert_cmd` to verify
//! end-to-end behaviour of the completion generator, install, and
//! uninstall flows.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "Integration tests unwrap known-good binary handles"
)]
#![allow(
    clippy::disallowed_methods,
    reason = "Tests use std::fs for fixture setup/verification and cannot use tokio::fs"
)]

use assert_cmd::Command;
use predicates::prelude::*;

/// `gitflow completions bash` emits a bash completion script to stdout.
#[test]
fn test_should_generate_bash_completion_to_stdout() {
    let mut cmd = Command::cargo_bin("gitflow-cli").expect("binary should build");
    cmd.args(["completions", "bash"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("complete -F"));
}

/// `gitflow completions zsh` emits a zsh completion script to stdout.
#[test]
fn test_should_generate_zsh_completion_to_stdout() {
    let mut cmd = Command::cargo_bin("gitflow-cli").expect("binary should build");
    cmd.args(["completions", "zsh"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("#compdef"));
}

/// `gitflow completions fish` emits a fish completion script to stdout.
#[test]
fn test_should_generate_fish_completion_to_stdout() {
    let mut cmd = Command::cargo_bin("gitflow-cli").expect("binary should build");
    cmd.args(["completions", "fish"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("complete -c"));
}

/// `gitflow completions` (no args) should fail — shell is required without flags.
#[test]
fn test_should_require_shell_without_flags() {
    let mut cmd = Command::cargo_bin("gitflow-cli").expect("binary should build");
    cmd.arg("completions");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

/// `gitflow completions --install --uninstall` should fail — flags are mutually exclusive.
#[test]
fn test_should_reject_install_and_uninstall_together() {
    let mut cmd = Command::cargo_bin("gitflow-cli").expect("binary should build");
    cmd.args(["completions", "--install", "--uninstall"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

/// `gitflow completions --install <shell>` writes the completion file to the
/// expected location and emits the success message.
#[cfg(unix)]
#[test]
fn test_should_install_completion_for_explicit_shell() {
    let tmp_home = tempfile::tempdir().expect("tempdir should succeed");
    let tmp_home_path = tmp_home.path();

    let mut cmd = Command::cargo_bin("gitflow-cli").expect("binary should build");
    cmd.args(["completions", "--install", "bash"])
        .env("HOME", tmp_home_path);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Completion installed for bash"));

    let installed = tmp_home_path.join(".local/share/bash-completion/completions/gitflow-cli.bash");
    assert!(
        installed.exists(),
        "completion file should exist at {installed:?}"
    );

    let contents = std::fs::read_to_string(&installed).expect("file should be readable");
    assert!(
        contents.contains("complete -F"),
        "installed bash completion should contain `complete -F`"
    );
}

/// `gitflow completions --uninstall <shell>` removes a previously installed
/// completion file.
#[cfg(unix)]
#[test]
fn test_should_uninstall_existing_completion() {
    let tmp_home = tempfile::tempdir().expect("tempdir should succeed");
    let tmp_home_path = tmp_home.path();

    // Install first.
    let completion_dir = tmp_home_path.join(".local/share/bash-completion/completions");
    std::fs::create_dir_all(&completion_dir).expect("create_dir_all should succeed");
    let file_path = completion_dir.join("gitflow-cli.bash");
    std::fs::write(&file_path, "# fake completion").expect("write should succeed");
    assert!(
        file_path.exists(),
        "fixture file should exist before uninstall"
    );

    let mut cmd = Command::cargo_bin("gitflow-cli").expect("binary should build");
    cmd.args(["completions", "--uninstall", "bash"])
        .env("HOME", tmp_home_path);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Completion uninstalled for bash"));

    assert!(
        !file_path.exists(),
        "completion file should be removed after uninstall"
    );
}

/// `gitflow completions --uninstall` fails with a clear error when no file is
/// installed.
#[test]
fn test_should_fail_uninstall_when_file_missing() {
    let tmp_home = tempfile::tempdir().expect("tempdir should succeed");
    let tmp_home_path = tmp_home.path();

    let mut cmd = Command::cargo_bin("gitflow-cli").expect("binary should build");
    cmd.args(["completions", "--uninstall", "bash"])
        .env("HOME", tmp_home_path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("nothing to uninstall"));
}

/// `gitflow completions --install` auto-detects the shell from `$SHELL`.
#[cfg(unix)]
#[test]
fn test_should_auto_detect_shell_from_env() {
    let tmp_home = tempfile::tempdir().expect("tempdir should succeed");
    let tmp_home_path = tmp_home.path();

    let mut cmd = Command::cargo_bin("gitflow-cli").expect("binary should build");
    cmd.args(["completions", "--install"])
        .env("HOME", tmp_home_path)
        .env("SHELL", "/usr/bin/fish");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Completion installed for fish"));

    let installed = tmp_home_path.join(".config/fish/completions/gitflow-cli.fish");
    assert!(
        installed.exists(),
        "fish completion file should be created at {installed:?}"
    );
}

/// Auto-detection fails with a clear error when `$SHELL` names an unsupported
/// shell.
#[test]
fn test_should_reject_unsupported_shell_from_env() {
    let tmp_home = tempfile::tempdir().expect("tempdir should succeed");

    let mut cmd = Command::cargo_bin("gitflow-cli").expect("binary should build");
    cmd.args(["completions", "--install"])
        .env("HOME", tmp_home.path())
        .env("SHELL", "/bin/tcsh");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Unsupported shell"));
}

/// Auto-detection fails with a clear error when `$SHELL` is unset.
#[test]
fn test_should_fail_when_shell_env_is_missing() {
    let tmp_home = tempfile::tempdir().expect("tempdir should succeed");

    let mut cmd = Command::cargo_bin("gitflow-cli").expect("binary should build");
    cmd.args(["completions", "--install"])
        .env("HOME", tmp_home.path())
        .env_remove("SHELL");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("$SHELL"));
}
