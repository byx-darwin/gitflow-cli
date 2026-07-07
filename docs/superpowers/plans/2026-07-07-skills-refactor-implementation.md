# Skills Refactor Implementation Plan

> **Date:** 2026-07-07
> **Plan Type:** Superpowers writing-skills RED-GREEN-REFACTOR
> **Source Analysis:** `docs/research/skills-refactor-analysis.md`
> **Scope:** 26 skills, GitHub Issues #15–#41
> **Estimated Total Effort:** ~293 hours

---

## 1. Refactoring Scope

### 1.1 Skills Requiring P0 Refactoring (all 26)

Every skill in the project requires P0 (blocking) refactoring. No skill currently meets the minimum Superpowers writing-skills standard.

| Skill | Issue | Current State | P0 Scope |
|-------|-------|---------------|----------|
| gitflow-auth | #15 | ❌ All 4 dimensions | description, boundaries, keywords, cross-refs, testability |
| gitflow-autoreport-bug | #38 | ⚠️ D2 only | description, red flags, rationalization table, testability |
| gitflow-commit | #16 | ❌ All 4 dimensions | description, boundaries, keywords, cross-refs, testability |
| gitflow-issue-create | #26 | ❌ All 4 dimensions | description, boundaries, keywords, cross-refs, testability |
| gitflow-issue-review | #33 | ❌ All 4 dimensions | description, boundaries, keywords, cross-refs, idempotency, testability |
| gitflow-issue-triage | #29 | ❌ All 4 dimensions | description, boundaries, keywords, cross-refs, idempotency, testability |
| gitflow-issue | #36 | ❌ All 4 dimensions | description, boundaries, keywords, cross-refs, flowchart, testability |
| gitflow-label-milestone | #17 | ❌ All 4 dimensions | description, boundaries, keywords, cross-refs, split recommendation, testability |
| gitflow-label-stats | #30 | ❌ All 4 dimensions | description, boundaries, keywords, cross-refs, token compression, testability |
| gitflow-pipeline-analyzer | #28 | ❌ All 4 dimensions | description, boundaries, keywords, cross-refs, token compression, testability |
| gitflow-pr-apply-feedback | #33 | ❌ All 4 dimensions | description, boundaries, keywords, cross-refs, token compression, testability |
| gitflow-pr-create | #27 | ❌ All 4 dimensions | description, boundaries, keywords, cross-refs, testability |
| gitflow-pr-inline-review | #41 | ❌ All 4 dimensions | description, boundaries, keywords, cross-refs, publish-confirm mechanism, testability |
| gitflow-pr-review | #34 | ❌ All 4 dimensions | description, boundaries, keywords, cross-refs, testability |
| gitflow-pr | #32 | ❌ All 4 dimensions | description, boundaries, keywords, cross-refs, delegation model, testability |
| gitflow-precommit | #24 | ❌ All 4 dimensions | description, boundaries, keywords, cross-refs, token compression, testability |
| gitflow-quality | #35 | ❌ All 4 dimensions | description, boundaries, keywords, cross-refs, token compression, testability |
| gitflow-regression | #25 | ❌ All 4 dimensions | description, boundaries, keywords, cross-refs, token compression, testability |
| gitflow-release-helper | #31 | ❌ All 4 dimensions | description, boundaries, keywords, cross-refs, token compression, testability |
| gitflow-release | #18 | ❌ All 4 dimensions | description, boundaries, keywords, cross-refs, testability |
| gitflow-repo-onboarding | #20 | ❌ All 4 dimensions | description, boundaries, keywords, cross-refs, token compression, testability |
| gitflow-repo | #19 | ❌ All 4 dimensions | description, boundaries, keywords, cross-refs, token compression, testability |
| gitflow-review | #39 | ❌ All 4 dimensions | description, boundaries, keywords, cross-refs, flowchart, testability |
| gitflow-security-check | #22 | ❌ All 4 dimensions | description, boundaries, keywords, cross-refs, testability |
| gitflow-weekly-report | #23 | ❌ All 4 dimensions | description, boundaries, keywords, cross-refs, token compression, testability |
| gitflow-workflow | #37 | ❌ D1, D3; ⚠️ D2, D4 | description, token compression, rationalization table, red flags, testability |

### 1.2 Skills Requiring P1 Refactoring (all 26)

All skills need P1 items: structured template compliance, error handling, preconditions, rationalization counter-tables, flowcharts (10 skills), Quick Reference cards.

### 1.3 Skills Requiring P2 Enhancements (all 26)

All skills benefit from P2: baseline tests, stress tests, English descriptions, TDD records. Additionally, 2 skills (label-milestone, repo) may need file splits.

---

## 2. Methodology: Superpowers writing-skills RED-GREEN-REFACTOR

### 2.1 TDD for Skills Cycle

For each skill, follow the complete RED → GREEN → REFACTOR cycle:

```
RED       → Write a failing test that describes expected skill behavior
            (trigger accuracy, boundary enforcement, output format)
GREEN     → Write the minimal skill content to make the test pass
REFACTOR  → Improve structure, remove duplication, add cross-references
            while keeping tests green
```

### 2.2 RED Phase (per skill)

1. **Trigger-accuracy test:** Given user utterance X, does Claude load this skill?
2. **Negative trigger test:** Given user utterance Y (out of scope), does Claude NOT load this skill?
3. **Boundary test:** Given a scenario that tempts Claude to overstep, does the skill prevent it?
4. **Output format test:** Does the skill produce the expected output structure?
5. **Success criteria test:** Can an independent verifier confirm the skill executed correctly?

Concrete RED deliverable: Add a `## Test Scenarios` section with at least:
- 1 happy-path scenario
- 1 negative scenario (should not trigger or should refuse)
- 1 boundary scenario (temptation to overstep)
- 1 error scenario (CLI failure, auth failure, network timeout)

### 2.3 GREEN Phase (per skill)

1. Rewrite `description` frontmatter to "Use when..." trigger-only format
2. Add `## Overview` (1–2 sentences)
3. Add `## When to Use` with trigger keywords (English + Chinese)
4. Add `## Core Pattern` (executable skeleton)
5. Add `## Quick Reference` (command cheat-sheet)
6. Add `## Implementation` (step-by-step instructions)
7. Add `## Responsibility` with `✅ In scope` / `❌ Out of scope` / `🚫 Do Not`
8. Add `## Red Flags`
9. Add `## See Also` cross-references
10. Add `## Test Scenarios` (from RED phase)
11. Add `## Success Criteria`
12. Add `## Common Mistakes`

### 2.4 REFACTOR Phase (per skill)

1. Compress to ≤ 500 words (externalize templates to `docs/templates/` or `docs/references/`)
2. Replace narrative examples with pattern-language examples
3. Replace fictional data with placeholders
4. Add rationalization excuse counter-table
5. Add Mermaid flowchart (where decision logic exists)
6. Add error handling section
7. Add preconditions check
8. Verify `make check-agent-sync` passes
9. Verify `superpowers:writing-skills` review returns zero P0 findings

### 2.5 Writing-Skills Checklist (per skill)

Before marking any task complete, verify:

- [ ] `description` matches `/^Use when/i` and contains no functional/workflow description
- [ ] Contains `## Overview`
- [ ] Contains `## When to Use` with trigger keywords
- [ ] Contains `## Core Pattern`
- [ ] Contains `## Quick Reference`
- [ ] Contains `## Implementation`
- [ ] Contains `## Common Mistakes`
- [ ] Contains `## Responsibility` / `## Boundary`
- [ ] Contains `## Red Flags`
- [ ] Contains `## Trigger Keywords`
- [ ] Contains `## See Also`
- [ ] Contains `## Test Scenarios` (≥ 4 scenarios including 1 negative)
- [ ] Contains `## Success Criteria`
- [ ] Token count ≤ 500 words
- [ ] No fictional data in examples
- [ ] No narrative examples
- [ ] Passes `superpowers:writing-skills` review

---

## 3. Task Breakdown

### 3.0 Foundation Task (prerequisite)

**Task 0: Create unified skill template and conventions**

| Field | Value |
|-------|-------|
| ID | TASK-0 |
| Effort | 8h |
| Depends on | None |
| Deliverables | `docs/superpowers/templates/skill-template.md`, `docs/superpowers/templates/skill-conventions.md` |

Sub-tasks:
1. Author canonical skill template with all required sections
2. Define token budget policy (SKILL.md ≤ 500 words, externalize references)
3. Define cross-reference convention (See Also format)
4. Define test scenario format (baseline + happy-path + negative + stress)
5. Define trigger keyword convention (English + Chinese bilingual)
6. Define rationalization excuse counter-table format
7. Define Red Flags format
8. Review with `superpowers:writing-skills` for template compliance

---

### 3.1 Phase 1 Tasks (P0 Refactoring)

#### Batch A: High-Risk Skills (side-effect risk)

**Task 1: Refactor gitflow-pr-inline-review**

| Field | Value |
|-------|-------|
| ID | TASK-1 |
| Issue | #41 |
| Effort | 4h |
| Depends on | TASK-0 |
| Risk | 🔴 High (publishes review comments) |

P0 items: description, boundaries (with publish-confirm mechanism), prohibition list, red flags, keywords, cross-refs, testability hooks
P1 items: structured template, error handling, preconditions, rationalization table, flowchart, Quick Reference

**Task 2: Refactor gitflow-security-check**

| Field | Value |
|-------|-------|
| ID | TASK-2 |
| Issue | #22 |
| Effort | 4h |
| Depends on | TASK-0 |
| Risk | 🔴 High (scans sensitive data) |

P0 items: description, boundaries (no auto-fix, no data exfiltration), prohibition list, red flags, keywords, cross-refs, testability hooks
P1 items: structured template, error handling, preconditions, rationalization table, Quick Reference

**Task 3: Refactor gitflow-quality**

| Field | Value |
|-------|-------|
| ID | TASK-3 |
| Issue | #35 |
| Effort | 4h |
| Depends on | TASK-0 |
| Risk | 🔴 High (Issue publishing) |

P0 items: description, boundaries (Issue publish requires confirmation), prohibition list, red flags, keywords, cross-refs, token compression, testability hooks
P1 items: structured template, error handling, preconditions, rationalization table, flowchart, Quick Reference

**Task 4: Refactor gitflow-review**

| Field | Value |
|-------|-------|
| ID | TASK-4 |
| Issue | #39 |
| Effort | 4h |
| Depends on | TASK-0 |
| Risk | 🔴 High (merge gate) |

P0 items: description, boundaries (approve requires prior analysis), prohibition list, red flags, keywords, cross-refs, testability hooks
P1 items: structured template, error handling, preconditions, rationalization table, flowchart (approve vs submit), Quick Reference

**Task 5: Refactor gitflow-pr-apply-feedback**

| Field | Value |
|-------|-------|
| ID | TASK-5 |
| Issue | #33 |
| Effort | 4h |
| Depends on | TASK-0 |
| Risk | 🔴 High (code modify + commit + push) |

P0 items: description, boundaries (each modification requires confirmation, push requires confirmation), prohibition list, red flags, keywords, cross-refs, token compression, testability hooks
P1 items: structured template, error handling, preconditions, rationalization table, flowchart, Quick Reference

---

#### Batch B: Medium-Risk Skills

**Task 6: Refactor gitflow-workflow**

| Field | Value |
|-------|-------|
| ID | TASK-6 |
| Issue | #37 |
| Effort | 6h |
| Depends on | TASK-0 |
| Risk | 🟡 Medium (orchestration, gate enforcement) |

P0 items: description, token compression (1725 → < 500), structured boundaries, rationalization table, red flags, testability hooks
P1 items: structured template, Mermaid flowchart, Quick Reference, See Also (11+ skills), error handling

**Task 7: Refactor gitflow-release-helper**

| Field | Value |
|-------|-------|
| ID | TASK-7 |
| Issue | #31 |
| Effort | 4h |
| Depends on | TASK-0 |
| Risk | 🟡 Medium (version decision, Release creation) |

P0 items: description, boundaries, prohibition list, red flags, keywords, cross-refs, token compression (916 → < 500), testability hooks
P1 items: structured template, error handling, preconditions, rationalization table, Quick Reference

**Task 8: Refactor gitflow-regression**

| Field | Value |
|-------|-------|
| ID | TASK-8 |
| Issue | #25 |
| Effort | 4h |
| Depends on | TASK-0 |
| Risk | 🟡 Medium (indirect Issue creation via autoreport-bug) |

P0 items: description, boundaries (chain boundary with autoreport-bug), prohibition list, red flags, keywords, cross-refs, token compression, testability hooks
P1 items: structured template, error handling, preconditions, rationalization table, flowchart, Quick Reference

**Task 9: Refactor gitflow-precommit**

| Field | Value |
|-------|-------|
| ID | TASK-9 |
| Issue | #24 |
| Effort | 4h |
| Depends on | TASK-0 |
| Risk | 🟡 Medium (file writes, --fix operations) |

P0 items: description, boundaries (no auto-write hooks, no --fix without confirmation), prohibition list, red flags, keywords, cross-refs, token compression, testability hooks
P1 items: structured template, error handling, preconditions, rationalization table, flowchart, Quick Reference

**Task 10: Refactor gitflow-release**

| Field | Value |
|-------|-------|
| ID | TASK-10 |
| Issue | #18 |
| Effort | 3h |
| Depends on | TASK-0 |
| Risk | 🟡 Medium (Release CRUD, delete is irreversible) |

P0 items: description, boundaries, prohibition list, red flags, keywords, cross-refs, testability hooks
P1 items: structured template, error handling, preconditions, rationalization table, Quick Reference

**Task 11: Refactor gitflow-repo**

| Field | Value |
|-------|-------|
| ID | TASK-11 |
| Issue | #19 |
| Effort | 4h |
| Depends on | TASK-0 |
| Risk | 🟡 Medium (remote write: create, sync, push) |

P0 items: description, boundaries, prohibition list, red flags, keywords, cross-refs, token compression (900+ → < 500), testability hooks
P1 items: structured template, error handling, preconditions, rationalization table, Quick Reference

**Task 12: Refactor gitflow-autoreport-bug**

| Field | Value |
|-------|-------|
| ID | TASK-12 |
| Issue | #38 |
| Effort | 3h |
| Depends on | TASK-0 |
| Risk | 🟡 Medium (Issue creation) |

P0 items: description (currently workflow description), red flags, rationalization table, testability hooks
P1 items: structured template, token compression, Quick Reference, See Also

**Task 13: Refactor gitflow-issue**

| Field | Value |
|-------|-------|
| ID | TASK-13 |
| Issue | #36 |
| Effort | 4h |
| Depends on | TASK-0 |
| Risk | 🟡 Medium (7 subcommands with side effects) |

P0 items: description, boundaries, prohibition list, red flags, keywords, cross-refs, testability hooks
P1 items: structured template, error handling, preconditions, rationalization table, flowchart (subcommand selection), Quick Reference

**Task 14: Refactor gitflow-issue-triage**

| Field | Value |
|-------|-------|
| ID | TASK-14 |
| Issue | #29 |
| Effort | 3h |
| Depends on | TASK-0 |
| Risk | 🟡 Medium (label changes) |

P0 items: description, boundaries, prohibition list, red flags, keywords, cross-refs, idempotency declaration, testability hooks
P1 items: structured template, error handling, preconditions, rationalization table, flowchart, Quick Reference

**Task 15: Refactor gitflow-label-milestone**

| Field | Value |
|-------|-------|
| ID | TASK-15 |
| Issue | #17 |
| Effort | 3h |
| Depends on | TASK-0 |
| Risk | 🟡 Medium (label/milestone CRUD) |

P0 items: description, boundaries, prohibition list, red flags, keywords, cross-refs, testability hooks
P1 items: structured template, error handling, preconditions, rationalization table, Quick Reference, split recommendation

**Task 16: Refactor gitflow-issue-review**

| Field | Value |
|-------|-------|
| ID | TASK-16 |
| Issue | #33 |
| Effort | 4h |
| Depends on | TASK-0 |
| Risk | 🟡 Medium (posts comment on issue) |

P0 items: description, boundaries (comment posting is side effect), prohibition list, red flags, keywords, cross-refs, idempotency, testability hooks
P1 items: structured template, error handling, preconditions, rationalization table, Quick Reference

---

#### Batch C: Low-Risk Skills (read-only)

**Task 17: Refactor gitflow-pipeline-analyzer**

| Field | Value |
|-------|-------|
| ID | TASK-17 |
| Issue | #28 |
| Effort | 3h |
| Depends on | TASK-0 |
| Risk | 🟢 Low (read-only analysis) |

P0 items: description, boundaries (read-only declaration), prohibition list, red flags, keywords, cross-refs, token compression, testability hooks
P1 items: structured template, error handling, preconditions, Quick Reference

**Task 18: Refactor gitflow-pr-create**

| Field | Value |
|-------|-------|
| ID | TASK-18 |
| Issue | #27 |
| Effort | 3h |
| Depends on | TASK-0 |
| Risk | 🟢 Low (workflow guide, CLI executes) |

P0 items: description, boundaries, prohibition list, red flags, keywords, cross-refs, testability hooks
P1 items: structured template, error handling, preconditions, rationalization table, Quick Reference

**Task 19: Refactor gitflow-pr-review**

| Field | Value |
|-------|-------|
| ID | TASK-19 |
| Issue | #34 |
| Effort | 3h |
| Depends on | TASK-0 |
| Risk | 🟢 Low (checklist + conclusion) |

P0 items: description, boundaries (distinguish from pr-inline-review), prohibition list, red flags, keywords, cross-refs, testability hooks
P1 items: structured template, error handling, preconditions, rationalization table, Quick Reference

**Task 20: Refactor gitflow-pr**

| Field | Value |
|-------|-------|
| ID | TASK-20 |
| Issue | #32 |
| Effort | 4h |
| Depends on | TASK-0 |
| Risk | 🟢 Low (command reference, but 11 subcommands) |

P0 items: description, boundaries (delegation model to child skills), prohibition list, red flags, keywords, cross-refs, testability hooks
P1 items: structured template, error handling, preconditions, rationalization table, Quick Reference, delegation rules

**Task 21: Refactor gitflow-issue-create**

| Field | Value |
|-------|-------|
| ID | TASK-21 |
| Issue | #26 |
| Effort | 3h |
| Depends on | TASK-0 |
| Risk | 🟢 Low (workflow guide) |

P0 items: description, boundaries, prohibition list, red flags, keywords, cross-refs, testability hooks
P1 items: structured template, error handling, preconditions, rationalization table, Quick Reference

**Task 22: Refactor gitflow-label-stats**

| Field | Value |
|-------|-------|
| ID | TASK-22 |
| Issue | #30 |
| Effort | 3h |
| Depends on | TASK-0 |
| Risk | 🟢 Low (read-only statistics) |

P0 items: description, boundaries (read-only, no inference), prohibition list, red flags, keywords, cross-refs, token compression, testability hooks
P1 items: structured template, error handling, preconditions, Quick Reference

**Task 23: Refactor gitflow-repo-onboarding**

| Field | Value |
|-------|-------|
| ID | TASK-23 |
| Issue | #20 |
| Effort | 4h |
| Depends on | TASK-0 |
| Risk | 🟢 Low (analysis + doc generation) |

P0 items: description, boundaries (no auto-write files), prohibition list, red flags, keywords, cross-refs, token compression (968 → < 500), testability hooks
P1 items: structured template, error handling, preconditions, rationalization table, Quick Reference

**Task 24: Refactor gitflow-weekly-report**

| Field | Value |
|-------|-------|
| ID | TASK-24 |
| Issue | #23 |
| Effort | 3h |
| Depends on | TASK-0 |
| Risk | 🟢 Low (read-only, cross-repo) |

P0 items: description, boundaries (no fabrication, no performance evaluation), prohibition list, red flags, keywords, cross-refs, token compression, testability hooks
P1 items: structured template, error handling, preconditions, Quick Reference

**Task 25: Refactor gitflow-auth**

| Field | Value |
|-------|-------|
| ID | TASK-25 |
| Issue | #15 |
| Effort | 3h |
| Depends on | TASK-0 |
| Risk | 🟢 Low (command reference, but sensitive credentials) |

P0 items: description, boundaries (token safety), prohibition list, red flags, keywords, cross-refs, testability hooks
P1 items: structured template, error handling, preconditions, rationalization table, Quick Reference

**Task 26: Refactor gitflow-commit**

| Field | Value |
|-------|-------|
| ID | TASK-26 |
| Issue | #16 |
| Effort | 3h |
| Depends on | TASK-0 |
| Risk | 🟢 Low (command reference) |

P0 items: description, boundaries, prohibition list, red flags, keywords, cross-refs, testability hooks
P1 items: structured template, error handling, preconditions, rationalization table, Quick Reference

---

### 3.2 Phase 2 Tasks (P1 Completion)

Phase 2 runs after Phase 1 P0 items are stable. Each task is a continuation of the corresponding Phase 1 task.

| Phase 2 Task | Continues | Effort | Focus |
|--------------|-----------|--------|-------|
| TASK-27 | TASK-1 | 1h | pr-inline-review: flowchart, rationalization table |
| TASK-28 | TASK-2 | 1h | security-check: rationalization table |
| TASK-29 | TASK-3 | 1h | quality: flowchart, rationalization table |
| TASK-30 | TASK-4 | 1h | review: rationalization table |
| TASK-31 | TASK-5 | 1h | pr-apply-feedback: flowchart, rationalization table |
| TASK-32 | TASK-6 | 2h | workflow: Mermaid flowchart, full See Also |
| TASK-33 | TASK-7 | 1h | release-helper: rationalization table |
| TASK-34 | TASK-8 | 1h | regression: flowchart, rationalization table |
| TASK-35 | TASK-9 | 1h | precommit: flowchart, rationalization table |
| TASK-36 | TASK-10 | 1h | release: rationalization table |
| TASK-37 | TASK-11 | 1h | repo: rationalization table |
| TASK-38 | TASK-12 | 1h | autoreport-bug: structured template completion |
| TASK-39 | TASK-13 | 1h | issue: rationalization table |
| TASK-40 | TASK-14 | 1h | issue-triage: rationalization table |
| TASK-41 | TASK-15 | 1h | label-milestone: rationalization table |
| TASK-42 | TASK-16 | 1h | issue-review: rationalization table |
| TASK-43 | TASK-17 | 1h | pipeline-analyzer: rationalization table |
| TASK-44 | TASK-18 | 1h | pr-create: rationalization table |
| TASK-45 | TASK-19 | 1h | pr-review: rationalization table |
| TASK-46 | TASK-20 | 1h | pr: rationalization table |
| TASK-47 | TASK-21 | 1h | issue-create: rationalization table |
| TASK-48 | TASK-22 | 1h | label-stats: rationalization table |
| TASK-49 | TASK-23 | 1h | repo-onboarding: rationalization table |
| TASK-50 | TASK-24 | 1h | weekly-report: rationalization table |
| TASK-51 | TASK-25 | 1h | auth: rationalization table |
| TASK-52 | TASK-26 | 1h | commit: rationalization table |

**Phase 2 total estimate:** ~26h

---

### 3.3 Phase 3 Tasks (P2 Enhancements)

| Phase 3 Task | Covers | Effort | Focus |
|--------------|--------|--------|-------|
| TASK-53 | All skills (batch) | 10h | Add baseline test scenarios to all 26 skills |
| TASK-54 | All skills (batch) | 10h | Add stress test scenarios to all 26 skills |
| TASK-55 | All skills (batch) | 8h | Add English descriptions (bilingual) |
| TASK-56 | All skills (batch) | 8h | Add TDD for skills verification records |
| TASK-57 | label-milestone, repo | 4h | Evaluate and execute file splits |
| TASK-58 | 10 skills with flowcharts | 4h | Add Mermaid flowcharts |

**Phase 3 total estimate:** ~44h

---

### 3.4 Phase 4: Validation

| Phase 4 Task | Effort | Focus |
|--------------|--------|-------|
| TASK-59 | 26h | Run `superpowers:writing-skills` review on each skill, fix findings, close issues #15–#41 |

---

## 4. Estimated Effort Summary

| Phase | Tasks | Hours |
|-------|-------|-------|
| Phase 0: Foundation | TASK-0 | 8 |
| Phase 1: P0 (Batch A — high-risk) | TASK-1 to TASK-5 | 20 |
| Phase 1: P0 (Batch B — medium-risk) | TASK-6 to TASK-16 | 41 |
| Phase 1: P0 (Batch C — low-risk) | TASK-17 to TASK-26 | 33 |
| Phase 2: P1 completion | TASK-27 to TASK-52 | 26 |
| Phase 3: P2 enhancements | TASK-53 to TASK-58 | 44 |
| Phase 4: Validation | TASK-59 | 26 |
| **Total** | **59 tasks** | **~198h** |

### Effort by Risk Tier

| Risk Tier | Skills | P0 Effort | P1 Effort | P2 Effort | Total |
|-----------|--------|-----------|-----------|-----------|-------|
| 🔴 High | 5 | 20h | 5h | 10h | 35h |
| 🟡 Medium | 11 | 41h | 11h | 22h | 74h |
| 🟢 Low | 10 | 33h | 10h | 20h | 63h |
| Foundation | — | 8h | — | — | 8h |
| Validation | — | — | — | 26h | 26h |
| **Total** | **26** | **102h** | **26h** | **78h** | **~198h** |

---

## 5. Dependency Graph

```
TASK-0 (Foundation)
  ├── TASK-1  (pr-inline-review) ──→ TASK-27
  ├── TASK-2  (security-check) ──→ TASK-28
  ├── TASK-3  (quality) ──→ TASK-29
  ├── TASK-4  (review) ──→ TASK-30
  ├── TASK-5  (pr-apply-feedback) ──→ TASK-31
  ├── TASK-6  (workflow) ──→ TASK-32
  ├── TASK-7  (release-helper) ──→ TASK-33
  ├── TASK-8  (regression) ──→ TASK-34
  ├── TASK-9  (precommit) ──→ TASK-35
  ├── TASK-10 (release) ──→ TASK-36
  ├── TASK-11 (repo) ──→ TASK-37
  ├── TASK-12 (autoreport-bug) ──→ TASK-38
  ├── TASK-13 (issue) ──→ TASK-39
  ├── TASK-14 (issue-triage) ──→ TASK-40
  ├── TASK-15 (label-milestone) ──→ TASK-41
  ├── TASK-16 (issue-review) ──→ TASK-42
  ├── TASK-17 (pipeline-analyzer) ──→ TASK-43
  ├── TASK-18 (pr-create) ──→ TASK-44
  ├── TASK-19 (pr-review) ──→ TASK-45
  ├── TASK-20 (pr) ──→ TASK-46
  ├── TASK-21 (issue-create) ──→ TASK-47
  ├── TASK-22 (label-stats) ──→ TASK-48
  ├── TASK-23 (repo-onboarding) ──→ TASK-49
  ├── TASK-24 (weekly-report) ──→ TASK-50
  ├── TASK-25 (auth) ──→ TASK-51
  └── TASK-26 (commit) ──→ TASK-52

Phase 1 complete → Phase 2 (TASK-27 to TASK-52)
Phase 2 complete → Phase 3 (TASK-53 to TASK-58)
Phase 3 complete → Phase 4 (TASK-59)
```

### Cross-Skill Dependencies (must be coordinated)

| Cluster | Skills | Coordination Rule |
|---------|--------|-------------------|
| PR review | pr-review, pr-inline-review, pr-apply-feedback, pr, review | pr-review defines overall verdict; pr-inline-review handles line-level; pr-apply-feedback handles post-review fixes; pr is the router; review submits conclusion. Must define mutual boundaries simultaneously. |
| Issue lifecycle | issue, issue-create, issue-review, issue-triage | issue is CRUD router; issue-create is interactive workflow; issue-review is analysis; issue-triage is classification. Must define handoff boundaries. |
| Release | release, release-helper | release is CRUD; release-helper is workflow automation. Must define division of labor. |
| Quality | precommit, quality, security-check | precommit is pre-commit checks; quality is 6-gate delivery check; security-check is security audit. Must define scope boundaries. |
| Repo | repo, repo-onboarding | repo is CRUD; repo-onboarding is analysis/guide generation. Must define read vs write boundaries. |

---

## 6. Execution Strategy

### 6.1 Recommended Approach: Subagent-Driven Development

Use `superpowers:subagent-driven-development` with the following structure:

1. **Phase 0** — Execute TASK-0 directly (foundation, single agent)
2. **Phase 1 Batch A** — Dispatch 5 parallel subagents (TASK-1 to TASK-5), one per skill
3. **Phase 1 Cluster Coordination** — After Batch A, run a coordination pass to resolve cross-skill boundary conflicts in the PR review cluster
4. **Phase 1 Batch B** — Dispatch 11 parallel subagents (TASK-6 to TASK-16)
5. **Phase 1 Cluster Coordination** — Resolve Issue lifecycle, Release, Quality, Repo cluster conflicts
6. **Phase 1 Batch C** — Dispatch 10 parallel subagents (TASK-17 to TASK-26)
7. **Phase 2** — Dispatch subagents for P1 completion (can be batched)
8. **Phase 3** — Dispatch subagents for P2 enhancements (can be batched)
9. **Phase 4** — Sequential validation per skill, close issues

### 6.2 Per-Skill Execution Protocol

For each skill task, the subagent MUST:

1. Read the existing `SKILL.md`
2. Read the corresponding analysis doc in `docs/research/`
3. Read the unified template from `docs/superpowers/templates/skill-template.md`
4. Apply RED-GREEN-REFACTOR cycle
5. Write the refactored `SKILL.md`
6. Run `make check-agent-sync` (if applicable)
7. Self-review against the writing-skills checklist
8. Report completion with checklist status

### 6.3 Quality Gates

| Gate | Criteria | Enforcement |
|------|----------|-------------|
| G1: P0 complete | All P0 checklist items pass | Subagent self-review |
| G2: P1 complete | All P1 checklist items pass | Subagent self-review |
| G3: Token budget | `wc -w SKILL.md` ≤ 500 | Automated check |
| G4: No fictional data | All examples use placeholders | Subagent self-review |
| G5: writing-skills review | Zero P0 findings | `superpowers:writing-skills` review |
| G6: Agent sync | `make check-agent-sync` passes | Automated check |

---

## 7. Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Cross-skill boundary conflicts | Cluster coordination passes between batches; define delegation rules early |
| Token budget overflow | Externalize templates to `docs/templates/` and `docs/references/` |
| Inconsistent style across skills | TASK-0 establishes template and conventions first |
| Subagent produces incomplete refactor | Checklist-based self-review + Phase 4 validation gate |
| Regression in skill functionality | TDD for skills: baseline test → refactor → verify trigger accuracy |
| Scope creep during refactor | Strict P0/P1/P2 separation; each task has explicit scope |

---

## 8. Definition of Done

The refactor is complete when:

- [ ] All 26 skills pass the writing-skills checklist
- [ ] All 26 skills have ≤ 500 word SKILL.md (externalized content excluded)
- [ ] All 26 skills have trigger-condition descriptions
- [ ] All 26 skills have responsibility boundary declarations
- [ ] All 26 skills have test scenarios (baseline + happy-path + negative + stress)
- [ ] All 26 skills have success criteria
- [ ] All cluster cross-references are bidirectional and consistent
- [ ] `make check-agent-sync` passes for all skills
- [ ] `superpowers:writing-skills` review returns zero P0 findings for all skills
- [ ] GitHub Issues #15–#41 are closed with summary of changes
