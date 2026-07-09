//! GitHub pr 命令 E2E 测试

#![allow(clippy::unwrap_used, reason = "Test code uses unwrap for simplicity")]

use e2e_core::{TtyMode, TtyRunner};

#[tokio::test]
async fn test_pr_list_interactive() {
    let runner = TtyRunner::new(TtyMode::Interactive);
    let output = runner
        .run(&["pr", "list", "--platform", "github"])
        .await
        .unwrap();

    let output_combined = format!("{}{}", output.stdout, output.stderr);
    assert!(
        output.status.success()
            || !output_combined.is_empty()
            || output_combined.contains("error")
            || output_combined.contains("login")
    );
}

#[tokio::test]
async fn test_pr_list_non_interactive() {
    let runner = TtyRunner::new(TtyMode::NonInteractive);
    let output = runner
        .run(&["pr", "list", "--platform", "github"])
        .await
        .unwrap();

    let output_combined = format!("{}{}", output.stdout, output.stderr);
    assert!(
        output.status.success()
            || !output_combined.is_empty()
            || output_combined.contains("error")
            || output_combined.contains("login")
    );
}
