//! GitHub auth 命令 E2E 实测(真实凭据,严格断言)
//!
//! 无 `E2E_GITHUB_TOKEN` 时自动 skip(fork PR 路径)。

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "Test code uses unwrap/expect/indexing for simplicity"
)]

use e2e_core::{TestConfig, TestMode, TtyMode, TtyRunner};

#[tokio::test]
async fn test_should_report_logged_in_with_real_credentials() {
    let config = TestConfig::from_env_lenient();
    if config.mode() != TestMode::Authenticated {
        eprintln!("skipped: E2E_GITHUB_TOKEN not set");
        return;
    }

    for tty_mode in [TtyMode::Interactive, TtyMode::NonInteractive] {
        let mut runner = TtyRunner::new(tty_mode);
        for (key, value) in config.gh_env() {
            runner.env(key, value);
        }

        let output = runner
            .run(&["auth", "status", "--platform", "github", "--output", "json"])
            .await
            .unwrap();

        assert!(
            output.status.success(),
            "mode {tty_mode:?}: stderr: {}",
            output.stderr
        );
        let parsed: serde_json::Value =
            serde_json::from_str(&output.stdout).expect("stdout must be a JSON envelope");
        assert_eq!(
            parsed["success"],
            serde_json::json!(true),
            "mode {tty_mode:?}: stdout: {}",
            output.stdout
        );
        assert_eq!(
            parsed["data"]["loggedIn"],
            serde_json::json!(true),
            "mode {tty_mode:?}: expected logged-in, stdout: {}",
            output.stdout
        );
    }
}
