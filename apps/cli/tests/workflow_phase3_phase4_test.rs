//! Structure verification tests for Phase 3 (execution) and Phase 4 (post-delivery)
//! of the gitflow-workflow SKILL.md.

mod common;

// ── Phase 3: Execution ──────────────────────────────────────────────────────

#[test]
fn test_should_contain_phase3_heading() {
    let content = common::load_skill_md();
    assert!(
        content.contains("Phase 3: 执行"),
        "SKILL.md must contain 'Phase 3: 执行' heading"
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
        content.contains("Phase 4: 交付后检查"),
        "SKILL.md must contain 'Phase 4: 交付后检查' heading"
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
        content.contains("流水线分析报告"),
        "SKILL.md must list '流水线分析报告' as Phase 4 output"
    );
    assert!(
        content.contains("Issue 分类报告"),
        "SKILL.md must list 'Issue 分类报告' as Phase 4 output"
    );
    assert!(
        content.contains("代码审查报告"),
        "SKILL.md must list '代码审查报告' as Phase 4 output"
    );
}
