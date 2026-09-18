//! `GitCode` 列表分页 E2E 实测（真实服务端，只读）。
//!
//! 默认打公开仓库 `openharmony/docs`（issue 数 >200，已实测），可用
//! `E2E_TEST_REPO_GITCODE` 覆盖。本测试只读，不需要 `E2E_GITCODE_TOKEN`——
//! gate 是一次真实的 `gf auth status --platform gitcode` 探测：本机通过
//! `~/.config/gc/auth.json` 登录后，子进程 `gf` 会继承环境中的该登录态。
//! 探测失败（未装 gitcode CLI 或未登录）时自动 skip。
//!
//! **本测试在 Issue #365 修复前必然失败**：彼时 `gf issue list --limit 150`
//! 返回 100 条却报告 `truncated: false`。

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "Test code uses unwrap/expect/indexing for simplicity"
)]

use e2e_core::{TestConfig, TtyMode, TtyRunner, scratch_repo_dir};

/// 默认样本仓库：公开、只读、issue 数 >200。
const DEFAULT_PAGINATION_REPO: &str = "openharmony/docs";

#[tokio::test]
async fn test_should_not_silently_truncate_gitcode_issue_list_beyond_one_page() {
    let config = TestConfig::from_env_lenient();

    // 活体探测：需要一个可用且已登录的 gitcode CLI，而不是看 E2E_GITCODE_TOKEN
    // 是否设置——本测试只读，不需要该 token,机器上的凭据来自 `gc auth login`。
    let mut probe = TtyRunner::new(TtyMode::NonInteractive);
    for (key, value) in config.gitcode_env() {
        probe.env(key, value);
    }
    let probe_output = probe
        .run(&[
            "auth",
            "status",
            "--platform",
            "gitcode",
            "--output",
            "json",
        ])
        .await
        .unwrap();
    if !probe_output.status.success() {
        eprintln!("skipped: gitcode CLI not available or not authenticated");
        return;
    }

    let repo = config
        .gitcode_test_repo
        .clone()
        .unwrap_or_else(|| DEFAULT_PAGINATION_REPO.to_string());

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
            "all",
            "--limit",
            "150",
            "--output",
            "json",
        ])
        .await
        .unwrap();

    assert!(output.status.success(), "stderr: {}", output.stderr);
    let parsed: serde_json::Value =
        serde_json::from_str(&output.stdout).expect("stdout must be a JSON envelope");
    assert_eq!(parsed["success"], serde_json::json!(true));

    let items = parsed["data"]
        .as_array()
        .expect("data must be an array of issues");
    let truncated = parsed["pagination"]["truncated"]
        .as_bool()
        .expect("envelope must carry pagination.truncated");

    // 核心断言：要么真的翻过了单页 100 的坎，要么诚实报告截断。
    // 修复前二者皆不成立 —— 返回 100 条且 truncated=false。
    assert!(
        items.len() > 100 || truncated,
        "静默截断：返回 {} 条且 truncated={truncated}（仓库 {repo} issue 数应 >100）",
        items.len()
    );
}
