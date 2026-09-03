//! GitLab pr(mr)命令 E2E 实测(真实凭据 + 真实仓库,严格 schema 断言)
//!
//! 无 `E2E_GITLAB_TOKEN` 或 `E2E_TEST_REPO_GITLAB` 时自动 skip。
//!
//! 与 `e2e-github` 的差异:`e2e-github` 断言 `closed` 列表非空(利用本仓库自身
//! 已有已合并 PR 的确定性)。GitLab 测试仓库在基础设施到位前身份未知(可能是全新
//! 空仓库),因此本测试放宽为"若 `items` 非空则逐项校验 schema",不强制非空。

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "Test code uses unwrap/expect/indexing for simplicity"
)]

use e2e_core::{TestConfig, TestMode, TtyMode, TtyRunner, scratch_repo_dir};

#[tokio::test]
async fn test_should_list_closed_prs_with_valid_schema() {
    let config = TestConfig::from_env_lenient();
    let Some(repo) = config.gitlab_test_repo.clone() else {
        eprintln!("skipped: E2E_TEST_REPO_GITLAB not set");
        return;
    };
    if config.gitlab_mode() != TestMode::Authenticated {
        eprintln!("skipped: E2E_GITLAB_TOKEN not set");
        return;
    }

    let scratch = scratch_repo_dir(&format!("https://gitlab.com/{repo}.git"))
        .await
        .expect("scratch repo setup must succeed");

    let mut runner = TtyRunner::new(TtyMode::NonInteractive);
    runner.dir(scratch.path().to_path_buf());
    for (key, value) in config.gl_env() {
        runner.env(key, value);
    }

    let output = runner
        .run(&[
            "pr",
            "list",
            "--platform",
            "gitlab",
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
        .expect("data must be an array of pull/merge requests");
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
