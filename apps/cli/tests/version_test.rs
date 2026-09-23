//! Integration test for `gf --version` output format.

#![allow(
    clippy::disallowed_types,
    reason = "Test synchronously spawns the built binary; std::process::Command is appropriate \
              outside an async runtime"
)]
#![allow(
    clippy::expect_used,
    reason = "Test failures should panic with a clear message"
)]

use std::process::Command;

#[test]
fn test_should_print_version_with_parenthesized_suffix() {
    let output = Command::new(env!("CARGO_BIN_EXE_gf"))
        .arg("--version")
        .output()
        .expect("failed to run gf --version");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Format is "gf <version> (<sha>)" — the sha is either a short git hash
    // (hex, typically 7-12 chars) or the literal "unknown" fallback.
    let trimmed = stdout.trim();
    assert!(
        trimmed.starts_with("gf "),
        "expected version output to start with 'gf ', got: {trimmed:?}"
    );
    assert!(
        trimmed.ends_with(')') && trimmed.contains('('),
        "expected version output to end with a parenthesized suffix, got: {trimmed:?}"
    );
}
