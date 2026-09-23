//! 未认证错误路径 E2E 测试(无需凭据,前提是运行环境已安装 `gc`/`gitcode` CLI)。
//!
//! **不能只靠 `env_remove` 构造未认证环境**:gitcode CLI 把登录态存在
//! `~/.config/gc/auth.json`,清掉环境变量后它依然认为自己已登录,会让这两条
//! 测试在已登录的开发机上恒红(CI 无登录态故不暴露)。
//!
//! 实测(gitcode-cli 0.12.0)`GC_TOKEN` 优先于该配置文件**且会被服务端校验**,
//! 因此注入一个确定无效的值即可稳定构造「有凭据但认证失败」的确定性环境,
//! 无论本机是否登录,结果一致。
//!
//! 代价:该手段依赖一次真实的 token 校验请求,离线环境下报错形态可能不同。

#![allow(clippy::unwrap_used, reason = "Test code uses unwrap for simplicity")]

use e2e_core::{TtyMode, TtyRunner};

/// 语法合法但确定无效的 token,用于构造认证失败路径。
const INVALID_GC_TOKEN: &str = "invalid-token-for-e2e";

fn scrubbed_runner() -> TtyRunner {
    let mut runner = TtyRunner::new(TtyMode::NonInteractive);
    runner.env_remove("GITCODE_TOKEN");
    runner.env("GC_TOKEN", INVALID_GC_TOKEN);
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
