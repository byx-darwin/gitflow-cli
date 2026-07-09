//! Structure verification tests for Phase 2 (plan creation) of the gitflow-workflow SKILL.md.

mod common;

#[test]
fn test_should_contain_phase2_heading() {
    let content = common::load_skill_md();
    assert!(
        content.contains("Phase 2: Planning"),
        "SKILL.md must contain 'Phase 2: Planning' heading"
    );
}

#[test]
fn test_should_contain_create_full_plan_step() {
    let content = common::load_skill_md();
    assert!(
        content.contains("create a full plan"),
        "SKILL.md must contain 'create a full plan' step description"
    );
}

#[test]
fn test_should_contain_gitflow_quality_reference() {
    let content = common::load_skill_md();
    assert!(
        content.contains("gitflow-quality"),
        "SKILL.md must reference 'gitflow-quality' skill for quality gate"
    );
}

#[test]
fn test_should_contain_all_six_quality_checks() {
    let content = common::load_skill_md();
    // The 6 quality checks that must be present
    let checks = [
        ("Build", "Build check"),
        ("Test", "Test check"),
        ("Coverage", "Coverage check"),
        ("Format", "Format check"),
        ("Static", "Static check"),
        ("Pre-commit", "Pre-commit check"),
    ];

    for (name, pattern) in &checks {
        assert!(
            content.contains(pattern),
            "SKILL.md must contain quality check '{name}' (expected '{pattern}')"
        );
    }
}

#[test]
fn test_should_contain_writing_plans_skill_reference() {
    let content = common::load_skill_md();
    assert!(
        content.contains("superpowers:writing-plans"),
        "SKILL.md must reference 'superpowers:writing-plans' skill in Phase 2"
    );
}

#[test]
fn test_should_contain_quality_report_all_passed() {
    let content = common::load_skill_md();
    assert!(
        content.contains("ALL CHECKS PASSED"),
        "SKILL.md must contain quality report pass condition 'ALL CHECKS PASSED'"
    );
}
