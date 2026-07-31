//! 未认证错误路径 E2E 测试(无需凭据,任何环境均可运行)
//!
//! 通过 `env_remove` 清除继承的令牌、`GH_CONFIG_DIR` 指向空目录
//! 屏蔽 `gh` 的 hosts.yml,构造确定性的未认证环境。

#![allow(
    clippy::unwrap_used,
    clippy::disallowed_methods,
    reason = "Test code uses unwrap for simplicity; std::fs sync helper in setup"
)]

use e2e_core::{TtyMode, TtyRunner};

fn scrubbed_runner() -> TtyRunner {
    let mut runner = TtyRunner::new(TtyMode::NonInteractive);
    for var in ["GH_TOKEN", "GITHUB_TOKEN", "GH_ENTERPRISE_TOKEN"] {
        runner.env_remove(var);
    }
    let empty_config = std::env::temp_dir().join(format!("e2e-noauth-{}", std::process::id()));
    std::fs::create_dir_all(&empty_config).unwrap();
    runner.env("GH_CONFIG_DIR", empty_config.to_string_lossy().to_string());
    runner
}

#[tokio::test]
async fn test_should_fail_with_login_guidance_when_status_checked_unauthenticated() {
    let runner = scrubbed_runner();
    let output = runner
        .run(&["auth", "status", "--platform", "github", "--output", "json"])
        .await
        .unwrap();

    assert!(
        !output.status.success(),
        "unauthenticated auth status must exit non-zero"
    );
    let combined = format!("{}{}", output.stdout, output.stderr);
    assert!(
        combined.contains("gh auth login"),
        "expected login guidance in output, got: {combined}"
    );
}

#[tokio::test]
async fn test_should_fail_with_login_guidance_when_listing_issues_unauthenticated() {
    let runner = scrubbed_runner();
    let output = runner
        .run(&["issue", "list", "--platform", "github", "--output", "json"])
        .await
        .unwrap();

    assert!(
        !output.status.success(),
        "unauthenticated issue list must exit non-zero"
    );
    let combined = format!("{}{}", output.stdout, output.stderr);
    assert!(
        combined.contains("gh auth login"),
        "expected login guidance in output, got: {combined}"
    );
}
