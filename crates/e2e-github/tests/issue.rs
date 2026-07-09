//! GitHub issue 命令 E2E 测试

#![allow(clippy::unwrap_used, reason = "Test code uses unwrap for simplicity")]

use e2e_core::{TtyMode, TtyRunner};

#[tokio::test]
async fn test_issue_list_interactive() {
    let runner = TtyRunner::new(TtyMode::Interactive);
    let output = runner
        .run(&["issue", "list", "--platform", "github"])
        .await
        .unwrap();

    assert!(output.status.success());
}

#[tokio::test]
async fn test_issue_list_non_interactive() {
    let runner = TtyRunner::new(TtyMode::NonInteractive);
    let output = runner
        .run(&["issue", "list", "--platform", "github"])
        .await
        .unwrap();

    assert!(output.status.success());
}
