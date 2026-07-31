//! GitHub pr 命令 E2E 实测(真实凭据,严格 schema 断言)
//!
//! 查询 closed 状态以保证结果非空(已合并 PR 不会消失)。

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "Test code uses unwrap/expect/indexing for simplicity"
)]

use e2e_core::{TestConfig, TestMode, TtyMode, TtyRunner};

#[tokio::test]
async fn test_should_list_closed_prs_with_valid_schema() {
    let config = TestConfig::from_env_lenient();
    if config.mode() != TestMode::Authenticated {
        eprintln!("skipped: E2E_GITHUB_TOKEN not set");
        return;
    }

    let mut runner = TtyRunner::new(TtyMode::NonInteractive);
    for (key, value) in config.gh_env() {
        runner.env(key, value);
    }

    let output = runner
        .run(&[
            "pr",
            "list",
            "--platform",
            "github",
            "--state",
            "closed",
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
        .expect("data must be an array of pull requests");
    assert!(
        !items.is_empty(),
        "this repository should have at least one closed PR"
    );
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
