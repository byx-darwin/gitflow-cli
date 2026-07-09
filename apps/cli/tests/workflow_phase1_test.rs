//! Structure verification tests for Phase 1 (needs clarification) of the gitflow-workflow SKILL.md.

mod common;

#[test]
fn test_should_contain_phase1_heading() {
    let content = common::load_skill_md();
    assert!(
        content.contains("Phase 1: Clarification"),
        "SKILL.md must contain 'Phase 1: Clarification' heading"
    );
}

#[test]
fn test_should_contain_read_open_issues_step() {
    let content = common::load_skill_md();
    assert!(
        content.contains("Read Open Issues"),
        "SKILL.md must contain 'Read Open Issues' step description"
    );
}

#[test]
fn test_should_contain_full_and_fast_mode_in_phase1() {
    let content = common::load_skill_md();
    // Phase 1 must reference both working modes
    assert!(
        content.contains("Full Mode"),
        "SKILL.md must contain 'Full Mode' (full mode) reference"
    );
    assert!(
        content.contains("Fast Mode"),
        "SKILL.md must contain 'Fast Mode' (fast mode) reference"
    );
}

#[test]
fn test_should_contain_issue_list_commands() {
    let content = common::load_skill_md();
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
    let content = common::load_skill_md();
    assert!(
        content.contains("superpowers:brainstorming"),
        "SKILL.md must reference 'superpowers:brainstorming' skill in Phase 1"
    );
}

#[test]
fn test_should_contain_issue_create_skill_reference() {
    let content = common::load_skill_md();
    assert!(
        content.contains("gitflow-issue-create"),
        "SKILL.md must reference 'gitflow-issue-create' skill"
    );
}

#[test]
fn test_should_contain_issue_review_skill_reference() {
    let content = common::load_skill_md();
    assert!(
        content.contains("gitflow-issue-review"),
        "SKILL.md must reference 'gitflow-issue-review' skill"
    );
}
