//! 未认证错误路径 E2E 测试(无需凭据,前提是运行环境已安装 `glab` CLI)
//!
//! 通过 `env_remove` 清除继承的 `GL_TOKEN`,构造确定性的未认证环境。
//! `GitLabAuthProvider` 是纯 env-var 短路 + 真实 `glab` 子进程读取
//! (见 `crates/gitlab/src/auth.rs`),没有 `gh` 那种本地 `hosts.yml` 状态,
//! 因此 `env_remove` 单独即可保证确定性,无需额外的空目录隔离。

#![allow(clippy::unwrap_used, reason = "Test code uses unwrap for simplicity")]

use e2e_core::{TtyMode, TtyRunner};

fn scrubbed_runner() -> TtyRunner {
    let mut runner = TtyRunner::new(TtyMode::NonInteractive);
    runner.env_remove("GL_TOKEN");
    runner
}

#[tokio::test]
async fn test_should_fail_with_login_guidance_when_status_checked_unauthenticated() {
    let runner = scrubbed_runner();
    let output = runner
        .run(&["auth", "status", "--platform", "gitlab", "--output", "json"])
        .await
        .unwrap();

    assert!(
        !output.status.success(),
        "unauthenticated auth status must exit non-zero, stdout: {}, stderr: {}",
        output.stdout,
        output.stderr
    );
    let combined = format!("{}{}", output.stdout, output.stderr).to_lowercase();
    assert!(
        combined.contains("auth login") || combined.contains("login"),
        "expected login guidance in output, got: {combined}"
    );
}

#[tokio::test]
async fn test_should_fail_with_login_guidance_when_listing_issues_unauthenticated() {
    let runner = scrubbed_runner();
    let output = runner
        .run(&["issue", "list", "--platform", "gitlab", "--output", "json"])
        .await
        .unwrap();

    assert!(
        !output.status.success(),
        "unauthenticated issue list must exit non-zero, stdout: {}, stderr: {}",
        output.stdout,
        output.stderr
    );
    let combined = format!("{}{}", output.stdout, output.stderr).to_lowercase();
    assert!(
        combined.contains("auth login") || combined.contains("login"),
        "expected login guidance in output, got: {combined}"
    );
}
