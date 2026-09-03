//! `GitCode` issue 命令 E2E 实测(真实凭据 + 真实仓库,严格 schema 断言)
//!
//! 无 `E2E_GITCODE_TOKEN` 或 `E2E_TEST_REPO_GITCODE` 时自动 skip(真实测试仓库/凭据
//! 基础设施留待后续 Issue 配置)。

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "Test code uses unwrap/expect/indexing for simplicity"
)]

use e2e_core::{TestConfig, TestMode, TtyMode, TtyRunner, scratch_repo_dir};

#[tokio::test]
async fn test_should_list_open_issues_with_valid_schema() {
    let config = TestConfig::from_env_lenient();
    let Some(repo) = config.gitcode_test_repo.clone() else {
        eprintln!("skipped: E2E_TEST_REPO_GITCODE not set");
        return;
    };
    if config.gitcode_mode() != TestMode::Authenticated {
        eprintln!("skipped: E2E_GITCODE_TOKEN not set");
        return;
    }

    let scratch = scratch_repo_dir(&format!("https://gitcode.com/{repo}.git"))
        .await
        .expect("scratch repo setup must succeed");

    let mut runner = TtyRunner::new(TtyMode::NonInteractive);
    runner.dir(scratch.path().to_path_buf());
    for (key, value) in config.gitcode_env() {
        runner.env(key, value);
    }

    let output = runner
        .run(&[
            "issue",
            "list",
            "--platform",
            "gitcode",
            "--state",
            "open",
            "--output",
            "json",
        ])
        .await
        .unwrap();

    assert!(output.status.success(), "stderr: {}", output.stderr);
    let parsed: serde_json::Value =
        serde_json::from_str(&output.stdout).expect("stdout must be a JSON envelope");
    assert_eq!(
        parsed["success"],
        serde_json::json!(true),
        "stdout: {}",
        output.stdout
    );

    let items = parsed["data"]
        .as_array()
        .expect("data must be an array of issues");
    for item in items {
        assert!(
            item["number"].as_u64().is_some(),
            "number must be an unsigned integer: {item}"
        );
        assert!(
            item["title"].as_str().is_some_and(|t| !t.is_empty()),
            "title must be a non-empty string: {item}"
        );
    }
}
