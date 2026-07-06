//! Structure verification tests for Phase 1 (needs clarification) of the gitflow-workflow SKILL.md.

use std::fs;

/// Load SKILL.md content. `CARGO_MANIFEST_DIR` points to `apps/cli` when running
/// under `cargo test -p gitflow-cli`, so we navigate two levels up to the workspace root.
fn load_skill_md() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = format!("{manifest_dir}/../../skills/gitflow-workflow/SKILL.md");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read SKILL.md at {path}: {e}"))
}

#[test]
fn test_should_contain_phase1_heading() {
    let content = load_skill_md();
    assert!(
        content.contains("Phase 1: 需求澄清"),
        "SKILL.md must contain 'Phase 1: 需求澄清' heading"
    );
}

#[test]
fn test_should_contain_read_open_issues_step() {
    let content = load_skill_md();
    assert!(
        content.contains("读取 Open Issues"),
        "SKILL.md must contain '读取 Open Issues' step description"
    );
}

#[test]
fn test_should_contain_full_and_fast_mode_in_phase1() {
    let content = load_skill_md();
    // Phase 1 must reference both working modes
    assert!(
        content.contains("完整模式"),
        "SKILL.md must contain '完整模式' (full mode) reference"
    );
    assert!(
        content.contains("快速模式"),
        "SKILL.md must contain '快速模式' (fast mode) reference"
    );
}

#[test]
fn test_should_contain_issue_list_commands() {
    let content = load_skill_md();
    // Phase 1 must show gitflow-cli issue list commands
    assert!(
        content.contains("gitflow-cli issue list"),
        "SKILL.md must contain 'gitflow-cli issue list' command"
    );
    // Full mode lists all open issues
    assert!(
        content.contains("gitflow-cli issue list --state open"),
        "SKILL.md must contain open state filter for issue list"
    );
}

#[test]
fn test_should_contain_brainstorming_skill_reference() {
    let content = load_skill_md();
    assert!(
        content.contains("superpowers:brainstorming"),
        "SKILL.md must reference 'superpowers:brainstorming' skill in Phase 1"
    );
}

#[test]
fn test_should_contain_issue_create_skill_reference() {
    let content = load_skill_md();
    assert!(
        content.contains("gitflow-issue-create"),
        "SKILL.md must reference 'gitflow-issue-create' skill"
    );
}

#[test]
fn test_should_contain_issue_review_skill_reference() {
    let content = load_skill_md();
    assert!(
        content.contains("gitflow-issue-review"),
        "SKILL.md must reference 'gitflow-issue-review' skill"
    );
}
