//! Integration tests verifying the two workflow modes (complete and fast)
//! described in the gitflow-workflow SKILL.md.

mod common;

/// Helper: load SKILL.md content once per test via the shared utility.
fn skill_md() -> String {
    common::load_skill_md()
}

// ---------------------------------------------------------------------------
// Complete Mode Tests
// ---------------------------------------------------------------------------

#[test]
fn test_should_contain_complete_mode_section() {
    let md = skill_md();
    assert!(
        md.contains("Full Mode"),
        "SKILL.md must contain a 'Full Mode' (complete mode) section"
    );
}

#[test]
fn test_should_contain_complete_mode_brainstorming() {
    let md = skill_md();
    // Complete mode Phase 1 requires brainstorming
    assert!(
        md.contains("superpowers:brainstorming"),
        "Complete mode must reference superpowers:brainstorming"
    );
}

#[test]
fn test_should_contain_complete_mode_writing_plans() {
    let md = skill_md();
    // Complete mode Phase 2 requires writing-plans
    assert!(
        md.contains("superpowers:writing-plans"),
        "Complete mode must reference superpowers:writing-plans"
    );
}

#[test]
fn test_should_contain_complete_mode_subagent_driven_development() {
    let md = skill_md();
    // Complete mode Phase 3 requires subagent-driven-development
    assert!(
        md.contains("superpowers:subagent-driven-development"),
        "Complete mode must reference superpowers:subagent-driven-development"
    );
}

#[test]
fn test_should_reference_all_four_phases_in_complete_mode() {
    let md = skill_md();
    // The phase overview section lists all four phases
    for phase in 1..=4 {
        assert!(
            md.contains(&format!("Phase {phase}")),
            "SKILL.md must reference Phase {phase}"
        );
    }
}

#[test]
fn test_should_contain_issue_create_in_complete_mode() {
    let md = skill_md();
    assert!(
        md.contains("gitflow-issue-create"),
        "Complete mode must include gitflow-issue-create"
    );
}

#[test]
fn test_should_contain_issue_review_in_complete_mode() {
    let md = skill_md();
    assert!(
        md.contains("gitflow-issue-review"),
        "Complete mode must include gitflow-issue-review"
    );
}

// ---------------------------------------------------------------------------
// Fast Mode Tests
// ---------------------------------------------------------------------------

#[test]
fn test_should_contain_fast_mode_section() {
    let md = skill_md();
    assert!(
        md.contains("Fast Mode"),
        "SKILL.md must contain a 'Fast Mode' (fast mode) section"
    );
}

#[test]
#[allow(clippy::panic, reason = "Test-only panic for missing SKILL.md section")]
fn test_should_mark_brainstorming_optional_in_fast_mode() {
    let md = skill_md();
    // In fast mode, brainstorming is marked as optional
    // Look for the fast mode brainstorming line with the optional marker
    let Some(fast_phase1) = md.split("Fast Mode — Required Skills Checklist").nth(1) else {
        panic!(
            "Fast mode skills section 'Fast Mode — Required Skills Checklist' must exist in \
             SKILL.md"
        );
    };
    let fast_until_phase2 = fast_phase1.split("Phase 2").next().unwrap_or(fast_phase1);
    assert!(
        fast_until_phase2.contains("optional"),
        "Fast mode Phase 1 must mark brainstorming as optional (optional)"
    );
}

#[test]
#[allow(clippy::panic, reason = "Test-only panic for missing SKILL.md section")]
fn test_should_mark_writing_plans_optional_in_fast_mode() {
    let md = skill_md();
    // In fast mode, writing-plans is optional
    let Some(fast_section) = md.split("Fast Mode — Required Skills Checklist").nth(1) else {
        panic!(
            "Fast mode skills section 'Fast Mode — Required Skills Checklist' must exist in \
             SKILL.md"
        );
    };
    let phase2_area = fast_section.split("Phase 2").nth(1).unwrap_or("");
    assert!(
        phase2_area.contains("optional"),
        "Fast mode Phase 2 must mark writing-plans as optional (optional)"
    );
}

#[test]
#[allow(clippy::panic, reason = "Test-only panic for missing SKILL.md section")]
fn test_should_include_phase4_in_fast_mode() {
    let md = skill_md();
    // Phase 4 is mandatory for both modes
    let Some(fast_section) = md.split("Fast Mode — Required Skills Checklist").nth(1) else {
        panic!(
            "Fast mode skills section 'Fast Mode — Required Skills Checklist' must exist in \
             SKILL.md"
        );
    };
    assert!(
        fast_section.contains("Phase 4"),
        "Fast mode must include Phase 4 (mandatory for both modes)"
    );
}

#[test]
#[allow(clippy::panic, reason = "Test-only panic for missing SKILL.md section")]
fn test_should_include_mandatory_phase4_skills_in_fast_mode() {
    let md = skill_md();
    let Some(fast_section) = md.split("Fast Mode — Required Skills Checklist").nth(1) else {
        panic!(
            "Fast mode skills section 'Fast Mode — Required Skills Checklist' must exist in \
             SKILL.md"
        );
    };
    let phase4_area = fast_section.split("Phase 4").nth(1).unwrap_or("");
    assert!(
        phase4_area.contains("gitflow-pipeline-analyzer"),
        "Fast mode Phase 4 must include gitflow-pipeline-analyzer"
    );
    assert!(
        phase4_area.contains("gitflow-issue-triage"),
        "Fast mode Phase 4 must include gitflow-issue-triage"
    );
    assert!(
        phase4_area.contains("gitflow-review"),
        "Fast mode Phase 4 must include gitflow-review"
    );
}

// ---------------------------------------------------------------------------
// Cross-Mode Consistency Tests
// ---------------------------------------------------------------------------

#[test]
fn test_should_contain_mode_comparison_table() {
    let md = skill_md();
    assert!(
        md.contains("Mode Comparison"),
        "SKILL.md must contain a 'Mode Comparison' (mode comparison) section"
    );
    // The table must include both mode names
    assert!(
        md.contains("Full Mode") && md.contains("Fast Mode"),
        "Mode comparison table must reference both complete and fast modes"
    );
}

#[test]
fn test_should_contain_enforcement_rules_section() {
    let md = skill_md();
    assert!(
        md.contains("Enforcement Rules"),
        "SKILL.md must contain a 'Enforcement Rules' (enforcement rules) section"
    );
}

#[test]
fn test_should_reference_four_phases_not_three() {
    let md = skill_md();
    assert!(
        md.contains("four-phase"),
        "SKILL.md must reference 'four-phase' (four phases), not '三阶段'"
    );
    assert!(
        !md.contains("三阶段"),
        "SKILL.md must NOT reference '三阶段' (three phases) — it is a four-phase workflow"
    );
}

#[test]
fn test_should_contain_forbidden_behaviors_section() {
    let md = skill_md();
    assert!(
        md.contains("Forbidden Actions"),
        "SKILL.md must contain a 'Forbidden Actions' (forbidden behaviors) section"
    );
}

#[test]
fn test_should_forbid_skipping_phase4_in_any_mode() {
    let md = skill_md();
    // The forbidden behaviors section should explicitly forbid skipping Phase 4
    let forbidden_section = md.split("Forbidden Actions").nth(1).unwrap_or("");
    assert!(
        forbidden_section.contains("Phase 4"),
        "Forbidden behaviors must explicitly forbid skipping Phase 4"
    );
}

#[test]
fn test_should_forbid_skipping_tdd_and_code_review_in_fast_mode() {
    let md = skill_md();
    // The forbidden behaviors section should forbid skipping TDD and code review in fast mode
    assert!(
        md.contains("Fast mode forbids skipping TDD"),
        "Forbidden behaviors must state that fast mode cannot skip TDD"
    );
    assert!(
        md.contains("Fast mode forbids skipping") && md.contains("Code Review"),
        "Forbidden behaviors must state that fast mode cannot skip Code Review"
    );
}
