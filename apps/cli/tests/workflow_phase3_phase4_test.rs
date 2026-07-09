//! Structure verification tests for Phase 3 (execution) and Phase 4 (post-delivery)
//! of the gitflow-workflow SKILL.md.

mod common;

// ── Phase 3: Execution ──────────────────────────────────────────────────────

#[test]
fn test_should_contain_phase3_heading() {
    let content = common::load_skill_md();
    assert!(
        content.contains("Phase 3: Execution"),
        "SKILL.md must contain 'Phase 3: Execution' heading"
    );
}

#[test]
fn test_should_contain_subagent_driven_development() {
    let content = common::load_skill_md();
    assert!(
        content.contains("subagent-driven-development"),
        "SKILL.md must reference 'subagent-driven-development' skill in Phase 3"
    );
}

#[test]
fn test_should_contain_tdd_in_execution() {
    let content = common::load_skill_md();
    // TDD must be embedded in the execution phase
    assert!(
        content.contains("TDD"),
        "SKILL.md must reference TDD in the execution workflow"
    );
}

// ── Phase 4: Post-delivery checks ───────────────────────────────────────────

#[test]
fn test_should_contain_phase4_heading() {
    let content = common::load_skill_md();
    assert!(
        content.contains("Phase 4: Post-Delivery Checks"),
        "SKILL.md must contain 'Phase 4: Post-Delivery Checks' heading"
    );
}

#[test]
fn test_should_contain_pipeline_analyzer_reference() {
    let content = common::load_skill_md();
    assert!(
        content.contains("gitflow-pipeline-analyzer"),
        "SKILL.md must reference 'gitflow-pipeline-analyzer' skill in Phase 4"
    );
}

#[test]
fn test_should_contain_issue_triage_reference() {
    let content = common::load_skill_md();
    assert!(
        content.contains("gitflow-issue-triage"),
        "SKILL.md must reference 'gitflow-issue-triage' skill in Phase 4"
    );
}

#[test]
fn test_should_contain_gitflow_review_reference() {
    let content = common::load_skill_md();
    assert!(
        content.contains("gitflow-review"),
        "SKILL.md must reference 'gitflow-review' skill in Phase 4"
    );
}

#[test]
fn test_should_contain_all_phase4_outputs() {
    let content = common::load_skill_md();
    // Phase 4 must produce three reports
    assert!(
        content.contains("pipeline analysis report"),
        "SKILL.md must list 'pipeline analysis report' as Phase 4 output"
    );
    assert!(
        content.contains("Issue triage report"),
        "SKILL.md must list 'Issue triage report' as Phase 4 output"
    );
    assert!(
        content.contains("code review report"),
        "SKILL.md must list 'code review report' as Phase 4 output"
    );
}
