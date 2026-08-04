//! E2E harness 自测:二进制发现与错误传播。
//!
//! 需要 `gf` 在 PATH 中:`cargo build --release` 后
//! `export PATH="$PWD/target/release:$PATH"`;CI 由 e2e-tests.yml
//! 的构建步骤保证。二进制缺失时测试 skip(不 fail)。

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "Test code uses unwrap for simplicity"
)]

use e2e_core::{TtyError, TtyMode, TtyRunner};

fn is_missing_binary(err: &TtyError) -> bool {
    matches!(err, TtyError::Io(e) if e.kind() == std::io::ErrorKind::NotFound)
}

#[tokio::test]
async fn test_should_run_help_successfully_in_both_tty_modes() {
    for mode in [TtyMode::Interactive, TtyMode::NonInteractive] {
        let runner = TtyRunner::new(mode);
        let output = match runner.run(&["--help"]).await {
            Ok(output) => output,
            Err(e) if is_missing_binary(&e) => {
                eprintln!("skipped: gf not in PATH (cargo build --release first)");
                return;
            }
            Err(e) => panic!("unexpected runner error: {e}"),
        };
        assert!(
            output.status.success(),
            "mode {mode:?}: exit {:?}, stderr: {}",
            output.status,
            output.stderr
        );
        assert!(
            output.stdout.contains("gf"),
            "mode {mode:?}: stdout missing product name: {}",
            output.stdout
        );
    }
}

#[tokio::test]
async fn test_should_propagate_nonzero_exit_for_unknown_subcommand() {
    let runner = TtyRunner::new(TtyMode::NonInteractive);
    let output = match runner.run(&["definitely-not-a-real-subcommand"]).await {
        Ok(output) => output,
        Err(e) if is_missing_binary(&e) => {
            eprintln!("skipped: gf not in PATH (cargo build --release first)");
            return;
        }
        Err(e) => panic!("unexpected runner error: {e}"),
    };
    assert!(!output.status.success(), "unknown subcommand must fail");
    assert!(
        !output.stderr.is_empty(),
        "stderr should contain a clap usage error"
    );
}
