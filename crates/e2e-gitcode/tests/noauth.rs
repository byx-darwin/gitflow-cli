//! 未认证错误路径 E2E 测试(无需凭据,前提是运行环境已安装 `gc`/`gitcode` CLI)
//!
//! 通过 `env_remove` 清除继承的 `GITCODE_TOKEN`,构造确定性的未认证环境。
//! `GitCodeAuthProvider` 是纯 env-var 短路 + 真实 `gc` 子进程读取
//! (见 `crates/gitcode/src/auth.rs`),没有本地配置文件状态,`env_remove` 单独
//! 即可保证确定性。

#![allow(clippy::unwrap_used, reason = "Test code uses unwrap for simplicity")]

use e2e_core::{TtyMode, TtyRunner};

fn scrubbed_runner() -> TtyRunner {
    let mut runner = TtyRunner::new(TtyMode::NonInteractive);
    runner.env_remove("GITCODE_TOKEN");
    runner
}

#[tokio::test]
async fn test_should_fail_with_login_guidance_when_status_checked_unauthenticated() {
    let runner = scrubbed_runner();
    let output = runner
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
        .run(&["issue", "list", "--platform", "gitcode", "--output", "json"])
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
