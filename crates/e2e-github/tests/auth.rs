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

    assert!(output.status.success() || output.stdout.contains("login"));
}

#[tokio::test]
async fn test_auth_status_non_interactive() {
    let runner = TtyRunner::new(TtyMode::NonInteractive);
    let output = runner
        .run(&["auth", "status", "--platform", "github"])
        .await
        .unwrap();

    assert!(output.status.success() || output.stderr.contains("login"));
}

#[tokio::test]
async fn test_auth_token_interactive() {
    let runner = TtyRunner::new(TtyMode::Interactive);
    let output = runner
        .run(&["auth", "token", "--platform", "github"])
        .await
        .unwrap();

    assert!(output.status.success() || output.stdout.contains("login"));
}

#[tokio::test]
async fn test_auth_token_non_interactive() {
    let runner = TtyRunner::new(TtyMode::NonInteractive);
    let output = runner
        .run(&["auth", "token", "--platform", "github"])
        .await
        .unwrap();

    assert!(output.status.success() || output.stderr.contains("login"));
}
