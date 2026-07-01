//! Pull Request 子命令的集成测试。
//!
//! 这些测试通过 `assert_cmd` 调用 `gitflow-cli` 二进制，验证 help 输出
//! 和参数解析的正确性。由于实际执行需要 `gh` CLI 和 GitHub 仓库，
//! 不实际调用 provider 方法。

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_should_show_pr_create_help() {
    let mut cmd = Command::cargo_bin("gitflow-cli").expect("binary exists");
    cmd.args(["pr", "create", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--title"))
        .stdout(predicate::str::contains("--head"))
        .stdout(predicate::str::contains("--base"))
        .stdout(predicate::str::contains("--body"))
        .stdout(predicate::str::contains("--draft"))
        .stdout(predicate::str::contains("--repo"));
}

#[test]
fn test_should_show_pr_list_help() {
    let mut cmd = Command::cargo_bin("gitflow-cli").expect("binary exists");
    cmd.args(["pr", "list", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--state"))
        .stdout(predicate::str::contains("--limit"));
}

#[test]
fn test_should_show_pr_view_help() {
    let mut cmd = Command::cargo_bin("gitflow-cli").expect("binary exists");
    cmd.args(["pr", "view", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("NUMBER"));
}

#[test]
fn test_should_reject_missing_create_title() {
    // `pr create` 缺少必填的 --title 参数
    let mut cmd = Command::cargo_bin("gitflow-cli").expect("binary exists");
    cmd.args(["--platform", "github", "pr", "create"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("--title"));
}
