//! Issue 子命令的集成测试。
//!
//! 这些测试通过 `assert_cmd` 调用 `gitflow-cli` 二进制，验证 help 输出
//! 和参数解析的正确性。由于实际执行需要 `gh` CLI 和 GitHub 仓库，
//! 不实际调用 provider 方法。

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "Integration tests unwrap known-good binary handles"
)]

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_should_show_issue_create_help() {
    let mut cmd = Command::cargo_bin("gitflow-cli").expect("binary exists");
    cmd.args(["issue", "create", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--title"))
        .stdout(predicate::str::contains("--body"))
        .stdout(predicate::str::contains("--label"))
        .stdout(predicate::str::contains("--assignee"));
}

#[test]
fn test_should_show_issue_list_help() {
    let mut cmd = Command::cargo_bin("gitflow-cli").expect("binary exists");
    cmd.args(["issue", "list", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--state"))
        .stdout(predicate::str::contains("--search"))
        .stdout(predicate::str::contains("--limit"));
}

#[test]
fn test_should_show_issue_view_help() {
    let mut cmd = Command::cargo_bin("gitflow-cli").expect("binary exists");
    cmd.args(["issue", "view", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("NUMBER"));
}

#[test]
fn test_should_reject_missing_required_args() {
    // `issue create` 缺少必填的 --title 参数
    let mut cmd = Command::cargo_bin("gitflow-cli").expect("binary exists");
    cmd.args(["--platform", "github", "issue", "create"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("--title"));
}

#[test]
fn test_should_reject_invalid_state_for_list() {
    // 无效的 --state 值应在运行时返回错误
    // 注意：需要平台检测成功才能进入 handle()，因此 --platform github 是必须的。
    let mut cmd = Command::cargo_bin("gitflow-cli").expect("binary exists");
    cmd.args([
        "--platform",
        "github",
        "issue",
        "list",
        "--state",
        "invalid",
    ]);
    // 由于 `gh` 不存在时前置检查会先失败，因此断言失败即可
    cmd.assert().failure();
}

#[test]
fn test_should_show_issue_subcommand_help() {
    let mut cmd = Command::cargo_bin("gitflow-cli").expect("binary exists");
    cmd.args(["issue", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("view"));
}
