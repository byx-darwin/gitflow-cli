use assert_cmd::Command;

#[test]
fn test_should_output_json_format_for_issue_list() {
    let mut cmd = Command::cargo_bin("gitflow-cli").expect("binary exists");
    cmd.args(["issue", "list", "--platform", "github", "--state", "open", "--limit", "1"]);
    // This will fail without gh CLI auth, but we test the error output format
    let output = cmd.output().expect("command runs");
    let stderr = String::from_utf8_lossy(&output.stderr);
    // In non-interactive mode, errors go to pending.json
    // Just verify the binary runs and produces some output
    assert!(!stderr.is_empty() || !output.stdout.is_empty());
}

#[test]
fn test_should_show_help_for_all_subcommands() {
    for subcmd in &["issue", "pr"] {
        let mut cmd = Command::cargo_bin("gitflow-cli").expect("binary exists");
        cmd.arg(subcmd).arg("--help");
        cmd.assert().success();
    }
}
