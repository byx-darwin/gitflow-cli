# Skills Refactor Analysis Report

> **Date:** 2026-07-07
> **Scope:** 26 gitflow-cli skills × 4 evaluation dimensions
> **Sources:** `docs/research/skill-analysis-gitflow-*.md` (26 analysis docs), GitHub Issues #15–#41
> **Methodology:** Superpowers writing-skills 4-dimension framework (Structure & Documentation, Responsibility Boundaries, Testability, Superpowers Best Practices)

---

## 1. Evaluation Matrix

### 1.1 Dimension Key

| Score | Meaning |
|-------|---------|
| ✅ Good | Meets Superpowers standard |
| ⚠️ Needs Improvement | Partially compliant, fixable gaps |
| ❌ Unacceptable | Critical gaps, blocking compliance |

### 1.2 Full Matrix (26 Skills × 4 Dimensions)

| # | Skill | Issue | D1 Structure | D2 Boundaries | D3 Testability | D4 Best Practice | Overall |
|---|-------|-------|--------------|---------------|----------------|------------------|---------|
| 1 | gitflow-auth | #15 | ⚠️ | ❌ | ❌ | ❌ | ❌ |
| 2 | gitflow-autoreport-bug | #38 | ⚠️ | ⚠️ | ❌ | ⚠️ | ⚠️ |
| 3 | gitflow-commit | #16 | ⚠️ | ❌ | ❌ | ❌ | ❌ |
| 4 | gitflow-issue-create | #26 | ⚠️ | ❌ | ❌ | ⚠️ | ❌ |
| 5 | gitflow-issue-review | #33 | ⚠️ | ❌ | ❌ | ❌ | ❌ |
| 6 | gitflow-issue-triage | #29 | ⚠️ | ❌ | ❌ | ❌ | ❌ |
| 7 | gitflow-issue | #36 | ⚠️ | ❌ | ❌ | ❌ | ❌ |
| 8 | gitflow-label-milestone | #17 | ⚠️ | ❌ | ❌ | ❌ | ❌ |
| 9 | gitflow-label-stats | #30 | ⚠️ | ❌ | ❌ | ⚠️ | ❌ |
| 10 | gitflow-pipeline-analyzer | #28 | ⚠️ | ❌ | ❌ | ⚠️ | ❌ |
| 11 | gitflow-pr-apply-feedback | #33 | ⚠️ | ❌ | ❌ | ⚠️ | ❌ |
| 12 | gitflow-pr-create | #27 | ⚠️ | ❌ | ❌ | ⚠️ | ❌ |
| 13 | gitflow-pr-inline-review | #41 | ⚠️ | ❌ | ❌ | ⚠️ | ❌ |
| 14 | gitflow-pr-review | #34 | ⚠️ | ❌ | ❌ | ⚠️ | ❌ |
| 15 | gitflow-pr | #32 | ❌ | ❌ | ❌ | ❌ | ❌ |
| 16 | gitflow-precommit | #24 | ⚠️ | ❌ | ❌ | ⚠️ | ❌ |
| 17 | gitflow-quality | #35 | ⚠️ | ❌ | ❌ | ⚠️ | ❌ |
| 18 | gitflow-regression | #25 | ⚠️ | ❌ | ❌ | ⚠️ | ❌ |
| 19 | gitflow-release-helper | #31 | ❌ | ❌ | ❌ | ❌ | ❌ |
| 20 | gitflow-release | #18 | ⚠️ | ❌ | ❌ | ❌ | ❌ |
| 21 | gitflow-repo-onboarding | #20 | ❌ | ❌ | ❌ | ❌ | ❌ |
| 22 | gitflow-repo | #19 | ⚠️ | ❌ | ❌ | ❌ | ❌ |
| 23 | gitflow-review | #39 | ⚠️ | ❌ | ❌ | ❌ | ❌ |
| 24 | gitflow-security-check | #22 | ⚠️ | ❌ | ❌ | ⚠️ | ❌ |
| 25 | gitflow-weekly-report | #23 | ⚠️ | ❌ | ❌ | ⚠️ | ❌ |
| 26 | gitflow-workflow | #37 | ❌ | ⚠️ | ❌ | ⚠️ | ❌ |

### 1.3 Score Distribution

| Dimension | ✅ Good | ⚠️ Needs Improvement | ❌ Unacceptable |
|-----------|---------|----------------------|-----------------|
| D1: Structure & Documentation | 0 | 22 | 4 |
| D2: Responsibility Boundaries | 0 | 2 | 24 |
| D3: Testability | 0 | 0 | 26 |
| D4: Superpowers Best Practices | 0 | 10 | 16 |

**Key finding:** No skill passes any dimension at "Good" level. Testability (D3) is universally zero. Only `gitflow-autoreport-bug` reaches ⚠️ on D2 (the project's boundary-declaration benchmark).

---

## 2. Problem Statistics

### 2.1 Per-Dimension Issue Counts

#### D1: Structure & Documentation

| Issue | Count | Skills Affected |
|-------|-------|-----------------|
| `description` is functional/workflow description, not trigger condition | 26/26 | All |
| Missing canonical sections (Overview / When to Use / Core Pattern / Quick Reference / Common Mistakes) | 26/26 | All |
| Token count exceeds 500-word limit | 10/26 | pipeline-analyzer, pr-apply-feedback, precommit, quality, regression, release-helper, repo, repo-onboarding, weekly-report, workflow |
| Narrative example anti-pattern | 12/26 | autoreport-bug, issue-triage, label-stats, pipeline-analyzer, pr-apply-feedback, pr-inline-review, quality, regression, release-helper, repo, repo-onboarding, workflow |
| Fictional data in examples | 8/26 | label-stats, pipeline-analyzer, pr-apply-feedback, quality, release-helper, repo, repo-onboarding, security-check |

#### D2: Responsibility Boundaries

| Issue | Count | Skills Affected |
|-------|-------|-----------------|
| No responsibility boundary declaration | 24/26 | All except autoreport-bug, workflow |
| Missing prohibition list (Do Not) | 24/26 | All except autoreport-bug, workflow |
| Missing Red Flags list | 26/26 | All |
| Missing "rationalization excuse" counter-table | 26/26 | All |
| Missing structured scope matrix (In scope / Out of scope) | 24/26 | All except autoreport-bug, workflow |

#### D3: Testability

| Issue | Count | Skills Affected |
|-------|-------|-----------------|
| No test scenarios defined | 26/26 | All |
| No baseline test (no-skill behavior) | 26/26 | All |
| No stress / edge-case scenarios | 26/26 | All |
| No success criteria | 26/26 | All |
| No writing-skills test methodology hooks | 26/26 | All |

#### D4: Superpowers Best Practices

| Issue | Count | Skills Affected |
|-------|-------|-----------------|
| Description describes triggers only (violation) | 26/26 | All |
| No keyword/trigger-term coverage section | 26/26 | All |
| No cross-references (See Also) section | 24/26 | All except autoreport-bug (implicit), label-stats (implicit), workflow (implicit) |
| No TDD for skills (RED-GREEN-REFACTOR) evidence | 26/26 | All |
| Missing flowchart where decision logic exists | 10/26 | issue, issue-triage, pr-apply-feedback, pr-inline-review, precommit, quality, regression, review, security-check, workflow |

### 2.2 Most Common Issues (Top 10)

| Rank | Issue | Frequency |
|------|-------|-----------|
| 1 | No test scenarios / no testability surface | 26/26 (100%) |
| 2 | `description` is not trigger-condition format | 26/26 (100%) |
| 3 | Missing canonical structured sections | 26/26 (100%) |
| 4 | No Red Flags list | 26/26 (100%) |
| 5 | No rationalization excuse counter-table | 26/26 (100%) |
| 6 | No keyword/trigger-term coverage | 26/26 (100%) |
| 7 | No TDD for skills evidence | 26/26 (100%) |
| 8 | No prohibition list | 24/26 (92%) |
| 9 | No structured scope matrix | 24/26 (92%) |
| 10 | No cross-references | 24/26 (92%) |

### 2.3 Side-Effect Risk Ratings

| Risk Level | Skills |
|------------|--------|
| 🔴 High (publicly visible writes, merge gates, sensitive data) | pr-inline-review (publish review comments), security-check (scans sensitive data), quality (Issue publishing), review (merge gate), pr-apply-feedback (code modify + commit + push) |
| 🟡 Medium (Issue creation, label changes, file writes, indirect side effects) | autoreport-bug, issue, issue-triage, label-milestone, precommit, regression, release, release-helper, repo, workflow |
| 🟢 Low (read-only analysis) | auth, commit, issue-create, issue-review, label-stats, pipeline-analyzer, pr, pr-create, pr-review, repo-onboarding, weekly-report |

---

## 3. Priority Ranking

### 3.1 P0 Items (Blocking Compliance)

Aggregated across all 26 skills, the P0 items represent the minimum changes required to make each skill loadable and safe for autonomous invocation.

| P0 Item | Dimension | Skills Requiring | Effort per Skill |
|---------|-----------|------------------|------------------|
| Rewrite `description` as trigger condition ("Use when...") | D1, D4 | 26 | ~0.5h |
| Add responsibility boundary declaration section | D2 | 24 | ~1h |
| Add prohibition list (Do Not) | D2 | 24 | ~0.5h |
| Add keyword/trigger-term coverage | D4 | 26 | ~0.5h |
| Add cross-references (See Also) | D4 | 24 | ~0.5h |
| Add Red Flags list | D2 | 26 | ~0.5h |
| Compress tokens to < 500 words (where over) | D1 | 10 | ~1–2h |
| Add testability hooks (success criteria + baseline) | D3 | 26 | ~1h |

**Total P0 estimated effort:** ~80–100 hours (26 skills × average 3–4h per skill for P0 items)

### 3.2 P1 Items (Recommended for Quality)

| P1 Item | Dimension | Skills Requiring |
|---------|-----------|------------------|
| Refactor to structured template (Overview / When to Use / Core Pattern / Quick Reference / Implementation / Common Mistakes) | D1 | 26 |
| Add error handling section | D1 | 26 |
| Add preconditions check | D1 | 26 |
| Add rationalization excuse counter-table | D2 | 26 |
| Add decision flowchart (where applicable) | D4 | 10 |
| Add Quick Reference card | D1 | 26 |
| Extract templates/references to external files | D1 | 10 |
| Define success criteria | D3 | 26 |

**Total P1 estimated effort:** ~78–104 hours (26 skills × average 3–4h per skill)

### 3.3 P2 Items (Optional Enhancements)

| P2 Item | Dimension | Skills Requiring |
|---------|-----------|------------------|
| Add baseline test scenarios | D3 | 26 |
| Add stress test scenarios | D3 | 26 |
| Provide English description | D1 | 26 |
| Add TDD for skills verification record | D3, D4 | 26 |
| Token budget compliance (split files) | D1 | 10 |
| Add Mermaid flowcharts | D4 | 10 |
| Split multi-command skills | D1 | 2 (label-milestone, repo) |

**Total P2 estimated effort:** ~52–78 hours (26 skills × average 2–3h per skill)

### 3.4 Priority-Ordered Skill List (by Risk × Effort)

Skills ranked by: side-effect risk (🔴=3, 🟡=2, 🟢=1) × P0 gap count. Higher score = higher priority.

| Priority | Skill | Risk | P0 Gaps | Score | Issue |
|----------|-------|------|---------|-------|-------|
| 1 | gitflow-pr-inline-review | 🔴 | 6 | 18 | #41 |
| 2 | gitflow-security-check | 🔴 | 5 | 15 | #22 |
| 3 | gitflow-quality | 🔴 | 5 | 15 | #35 |
| 4 | gitflow-review | 🔴 | 5 | 15 | #39 |
| 5 | gitflow-pr-apply-feedback | 🔴 | 6 | 18 | #33 |
| 6 | gitflow-workflow | 🟡 | 5 | 10 | #37 |
| 7 | gitflow-release-helper | 🟡 | 6 | 12 | #31 |
| 8 | gitflow-regression | 🟡 | 5 | 10 | #25 |
| 9 | gitflow-precommit | 🟡 | 5 | 10 | #24 |
| 10 | gitflow-release | 🟡 | 4 | 8 | #18 |
| 11 | gitflow-repo | 🟡 | 4 | 8 | #19 |
| 12 | gitflow-autoreport-bug | 🟡 | 4 | 8 | #38 |
| 13 | gitflow-issue | 🟡 | 5 | 10 | #36 |
| 14 | gitflow-issue-triage | 🟡 | 4 | 8 | #29 |
| 15 | gitflow-label-milestone | 🟡 | 4 | 8 | #17 |
| 16 | gitflow-pipeline-analyzer | 🟢 | 5 | 5 | #28 |
| 17 | gitflow-pr-create | 🟢 | 5 | 5 | #27 |
| 18 | gitflow-pr-review | 🟢 | 5 | 5 | #34 |
| 19 | gitflow-pr | 🟢 | 6 | 6 | #32 |
| 20 | gitflow-issue-create | 🟢 | 5 | 5 | #26 |
| 21 | gitflow-issue-review | 🟢 | 6 | 6 | #33 |
| 22 | gitflow-label-stats | 🟢 | 6 | 6 | #30 |
| 23 | gitflow-repo-onboarding | 🟢 | 4 | 4 | #20 |
| 24 | gitflow-weekly-report | 🟢 | 6 | 6 | #23 |
| 25 | gitflow-auth | 🟢 | 4 | 4 | #15 |
| 26 | gitflow-commit | 🟢 | 4 | 4 | #16 |

---

## 4. Improvement Roadmap

### Phase 0: Foundation (prerequisite, not skill-specific)

1. **Define unified skill template** — create `docs/superpowers/templates/skill-template.md` with all required sections
2. **Define token budget policy** — SKILL.md ≤ 500 words; externalize templates/references
3. **Define cross-reference convention** — standardized See Also format
4. **Define test scenario format** — baseline + happy-path + negative + stress

### Phase 1: P0 Refactoring (all 26 skills)

**Goal:** Every skill has trigger-condition description, boundary declaration, prohibition list, keyword coverage, cross-references, and testability hooks.

**Batch A — High-risk skills (🔴 side-effect risk, 5 skills):**
- gitflow-pr-inline-review, gitflow-security-check, gitflow-quality, gitflow-review, gitflow-pr-apply-feedback
- Estimated: ~20h

**Batch B — Medium-risk skills (🟡 side-effect risk, 11 skills):**
- gitflow-workflow, gitflow-release-helper, gitflow-regression, gitflow-precommit, gitflow-release, gitflow-repo, gitflow-autoreport-bug, gitflow-issue, gitflow-issue-triage, gitflow-label-milestone, gitflow-issue-review
- Estimated: ~44h

**Batch C — Low-risk skills (🟢 read-only, 10 skills):**
- gitflow-pipeline-analyzer, gitflow-pr-create, gitflow-pr-review, gitflow-pr, gitflow-issue-create, gitflow-label-stats, gitflow-repo-onboarding, gitflow-weekly-report, gitflow-auth, gitflow-commit
- Estimated: ~40h

**Phase 1 total estimate:** ~104h

### Phase 2: P1 Refactoring (all 26 skills)

**Goal:** Full structured template compliance, error handling, preconditions, rationalization counter-tables, flowcards, Quick Reference cards.

**Estimated:** ~78–104h

### Phase 3: P2 Enhancements (all 26 skills)

**Goal:** Baseline tests, stress tests, English descriptions, TDD records, file splits.

**Estimated:** ~52–78h

### Phase 4: Validation

1. Run `superpowers:writing-skills` review on each skill
2. Run `make check-agent-sync` for consistency
3. Verify token counts
4. Close issues #15–#41

**Estimated:** ~26h (1h per skill review + fix cycle)

### Total Estimated Effort

| Phase | Hours |
|-------|-------|
| Phase 0: Foundation | ~8h |
| Phase 1: P0 (all 26 skills) | ~104h |
| Phase 2: P1 (all 26 skills) | ~90h |
| Phase 3: P2 (all 26 skills) | ~65h |
| Phase 4: Validation | ~26h |
| **Total** | **~293h** |

---

## 5. Cross-Cutting Patterns & Systemic Issues

### 5.1 Systemic Patterns

1. **Universal description violation** — All 26 skills use functional/workflow description instead of trigger-condition format. This is the single highest-impact fix.
2. **Universal testability gap** — Zero skills have any testability surface. This is the deepest systemic gap.
3. **Near-universal boundary gap** — 24/26 skills lack boundary declarations. Only `gitflow-autoreport-bug` has a complete one; `gitflow-workflow` has partial.
4. **Token bloat pattern** — 10 skills exceed the 500-word limit due to embedded templates, narrative examples, and full scripts. These should be externalized.
5. **Cross-reference vacuum** — 24/26 skills have no structured cross-references, fragmenting the skill ecosystem.

### 5.2 Skill Clusters Requiring Coordinated Refactoring

| Cluster | Skills | Coordination Need |
|---------|--------|-------------------|
| PR review | pr-review, pr-inline-review, pr-apply-feedback, pr, review | Must define mutual boundaries and delegation rules |
| Issue lifecycle | issue, issue-create, issue-review, issue-triage | Must define handoff boundaries |
| Release | release, release-helper | Must define division of labor (CRUD vs workflow) |
| Quality | precommit, quality, security-check | Must define scope boundaries (pre-commit vs gate vs audit) |
| Repo | repo, repo-onboarding | Must define read vs write boundaries |

### 5.3 Reference: gitflow-autoreport-bug as Benchmark

`gitflow-autoreport-bug` is the project's only skill with a complete responsibility boundary declaration. It should serve as the structural benchmark for all other skills. However, it still has gaps:
- Description is workflow description, not trigger condition
- Missing Red Flags list
- Missing rationalization excuse counter-table
- No testability surface
- Token count (~680 words) exceeds 500-word limit

Even the benchmark needs P0/P1 fixes.

---

## 6. Acceptance Criteria Summary

After refactor, every skill MUST:

- [ ] `description` matches `/^Use when/i` and contains no functional/workflow description
- [ ] Contains `## Overview` section
- [ ] Contains `## When to Use` section with trigger keywords
- [ ] Contains `## Core Pattern` section
- [ ] Contains `## Quick Reference` section
- [ ] Contains `## Implementation` section
- [ ] Contains `## Common Mistakes` section
- [ ] Contains `## Responsibility` or `## Boundary` section with `✅ In scope` / `❌ Out of scope` / `🚫 Do Not`
- [ ] Contains `## Red Flags` section
- [ ] Contains `## Trigger Keywords` section (English + Chinese)
- [ ] Contains `## See Also` cross-reference section
- [ ] Contains `## Test Scenarios` section (baseline + happy-path + negative + stress)
- [ ] Contains `## Success Criteria` section
- [ ] Token count ≤ 500 words (externalized content excluded)
- [ ] No fictional data in examples (use placeholders)
- [ ] No narrative examples (use pattern-language)
- [ ] Passes `superpowers:writing-skills` review with zero P0 findings
