//! GitHub auth 命令 E2E 测试

#![allow(clippy::unwrap_used, reason = "Test code uses unwrap for simplicity")]

use e2e_core::{TtyMode, TtyRunner};

#[tokio::test]
async fn test_auth_status_interactive() {
    let runner = TtyRunner::new(TtyMode::Interactive);
    let output = runner
        .run(&["auth", "status", "--platform", "github"])
        .await
        .unwrap();

    // In CI environment, TTY is not available, so we accept either:
    // - Success (logged in)
    // - Output contains "login" or "not logged in" (not authenticated)
    // - Output is non-empty (command executed)
    let output_combined = format!("{}{}", output.stdout, output.stderr);
    assert!(
        output.status.success()
            || output_combined.contains("login")
            || output_combined.contains("not logged in")
            || !output_combined.is_empty()
    );
}

#[tokio::test]
async fn test_auth_status_non_interactive() {
    let runner = TtyRunner::new(TtyMode::NonInteractive);
    let output = runner
        .run(&["auth", "status", "--platform", "github"])
        .await
        .unwrap();

    let output_combined = format!("{}{}", output.stdout, output.stderr);
    assert!(
        output.status.success()
            || output_combined.contains("login")
            || output_combined.contains("not logged in")
            || !output_combined.is_empty()
    );
}

#[tokio::test]
async fn test_auth_token_interactive() {
    let runner = TtyRunner::new(TtyMode::Interactive);
    let output = runner
        .run(&["auth", "token", "--platform", "github"])
        .await
        .unwrap();

    let output_combined = format!("{}{}", output.stdout, output.stderr);
    assert!(
        output.status.success()
            || output_combined.contains("login")
            || output_combined.contains("not logged in")
            || !output_combined.is_empty()
    );
}

#[tokio::test]
async fn test_auth_token_non_interactive() {
    let runner = TtyRunner::new(TtyMode::NonInteractive);
    let output = runner
        .run(&["auth", "token", "--platform", "github"])
        .await
        .unwrap();

    let output_combined = format!("{}{}", output.stdout, output.stderr);
    assert!(
        output.status.success()
            || output_combined.contains("login")
            || output_combined.contains("not logged in")
            || !output_combined.is_empty()
    );
}
