# Skill Analysis: `gitflow-pr-review`

**Date:** 2026-07-07
**Source:** `skills/gitflow-pr-review/SKILL.md`
**GitHub Issue:** #34
**Analyst:** implementer subagent

## Abstract

The `gitflow-pr-review` skill provides a 6-dimensional code review checklist workflow. It guides the reviewer through correctness, security, performance, maintainability, test-coverage, and documentation dimensions, then invokes `gitflow-cli review` to submit the conclusion. While the checklist content is thorough and domain-appropriate, the skill is structured as a flat "checklist + sequential steps" document rather than a contract-driven Superpowers skill. It lacks trigger-accurate frontmatter, boundary declarations, test scenarios, and cross-references. This document provides a four-dimension analysis with prioritized improvement recommendations.

---

## Dimension 1: Skill Structure & Documentation Conventions

**Rating: ⚠�需改进 (Needs Improvement)**

### ✅ Strengths

| Item | Status | Notes |
|------|--------|-------|
| YAML frontmatter present | ✅ | Has `name` and `description` fields |
| `name` field correct | ✅ | `gitflow-pr-review` matches directory name |
| File location | ✅ | `skills/gitflow-pr-review/SKILL.md` |
| Language consistency | ✅ | Entirely in Chinese, no dilution |
| Token efficiency | ✅ | ~168 lines, well under the 500-word threshold for a full guide |
| Command examples | ✅ | Three concrete `gitflow-cli review` invocations (approve, request-changes, comment) |
| Checklist quality | ✅ | 6 dimensions with 5-6 concrete items each; maps closely to `CLAUDE.md` code-quality section |
| Review conclusion template | ✅ | Structured markdown template with per-dimension pass/fail and inline file:line references |

### ❌ Deficiencies

| Item | Status | Details |
|------|--------|---------|
| `description` format | ❌ | Current: `"6 维度代码审查工作流 — 获取 PR 详情，按清单逐项审查，调用 gitflow-cli review 提交审查结论"` — describes the *entire workflow*, not the *trigger condition*. Superpowers convention requires `"Use when..."` trigger-only phrasing. |
| Structural sections | ⚠️ | Missing canonical sections: `When to Use`, `Core Pattern`, `Quick Reference`, `Implementation`, `Common Mistakes`. Current structure is "checklist → linear steps → examples → notes" without layered design. |
| Trigger keywords | ❌ | No `Trigger Keywords` section enumerating user phrases, error messages, or contextual cues. |
| Anti-patterns present | ⚠️ | The 6-dimension checklist is presented as reference content rather than an executable algorithm. Steps 1-5 are narrative prose ("调用...记录...汇总...撰写...") rather than pattern-language with decision branches. |
| Frontmatter description doubles as title | ❌ | The `description` field reads like a mini-overview, not a trigger signal. This causes false-positive loading when the user says "what is a good review process" or "show me the review CLI commands". |

### Recommended Description Rewrite

```yaml
description: "Use when the user requests a code review of a Pull Request through gitflow-cli — including approval, request-changes, and comment-only reviews. Triggers on phrases like 'review PR', 'check this pull request', 'approve PR', 'request changes on PR', or direct invocation of `gitflow-cli review`."
```

---

## Dimension 2: Responsibility Boundary Clarity

**Rating: ❌ 不合格 (Unacceptable)**

### Missing Elements

| Required Element | Present? | Gap |
|------------------|----------|-----|
| Responsibility boundary declaration | ❌ | No `## Responsibility` or `## Boundary` section. The skill never declares what it owns vs. what the CLI or other skills own. |
| Prohibition list (`🚫 Do not...`) | ❌ | No explicit prohibitions. Unclear whether the skill should: edit code, apply fixes, respond to review comments, merge PRs, or manage labels. |
| Scope matrix (`✅ In scope` / `❌ Out of scope`) | ❌ | No scope table. Without it, the skill risks scope creep into `gitflow-pr-inline-review` (line-level comments), `gitflow-pr-apply-feedback` (applying fixes), `gitflow-pr` (merge/close), or `gitflow-security-check` (deep security audit). |
| "Rationalization excuse" counter-table | ❌ | No `## When NOT to use this skill` table preempting common misapplications (e.g., "I can also leave line-level comments directly — should I call pr-inline-review instead?"). |
| Red Flags list | ❌ | No `## Red Flags` section warning about misapplication contexts (e.g., reviewing one's own PR, reviewing without reading the diff, running review in CI pipelines, or batch-reviewing multiple PRs). |

### Critical Boundary Conflicts

The PR review ecosystem in this repo has four distinct skills:

| Skill | Purpose | Boundary |
|-------|---------|----------|
| `gitflow-pr-review` | High-level 6-dimensional assessment + overall verdict | Overall approve / request-changes / comment |
| `gitflow-pr-inline-review` | Line-level inline comments on specific diff hunks | Per-file, per-line `[logic]`/`[security]`/`[naming]`/`[boundary]` labels |
| `gitflow-pr-apply-feedback` | Apply review feedback as code changes | Modify code, run tests, mark comments resolved |
| `gitflow-pr` | PR lifecycle operations (close, merge, ready, sync) | State transitions, not review content |

Without boundary declarations, an agent loaded with `gitflow-pr-review` may:
- Start producing line-level `[logic]` comments (territory of `gitflow-pr-inline-review`).
- Begin editing code to "fix" issues it found (territory of `gitflow-pr-apply-feedback`).
- Decide to merge or close the PR after approving (territory of `gitflow-pr`).

### Recommended Scope Contract

```
## ✅ Responsible For
- Fetching PR metadata and diff via `gitflow-cli pr view`
- Performing a structured 6-dimensional assessment (correctness, security, performance, maintainability, test coverage, documentation)
- Producing a review conclusion in the prescribed markdown template
- Submitting the conclusion via `gitflow-cli review approve`, `request-changes`, or `comment`
- Surfacing per-dimension findings with file:line references

## ❌ Not Responsible For
- Line-level inline comments on diff hunks (→ gitflow-pr-inline-review)
- Editing code to fix identified issues (→ gitflow-pr-apply-feedback)
- Merging, closing, or reopening PRs (→ gitflow-pr)
- Running security-audit tooling such as cargo audit/deny (→ gitflow-security-check)
- Managing labels or assignees (→ gitflow-label-milestone)

## 🚫 Do Not
- Submit approval before reading the full diff
- Leave line-level `[logic]`/`[security]` comments — that is gitflow-pr-inline-review's job
- Edit source files or run `cargo fix` based on review findings
- Merge or close the PR immediately after approving it
- Review your own PR (conflict of interest)
- Skip the security dimension even for small changes
```

---

## Dimension 3: Testability

**Rating: ❌ 不合格 (Unacceptable)**

### Missing Elements

| Required Element | Present? | Gap |
|------------------|----------|-----|
| Test scenarios | ❌ | No `## Test Scenarios` section with concrete input/expected-output pairs. |
| Baseline test (no-skill behavior) | ❌ | No description of what happens without the skill. |
| Stress / edge-case scenarios | ❌ | No coverage of: empty diff, very large diff, closed PR, draft PR, cross-repo PR, network failure, CLI auth failure, or non-ASCII file paths. |
| Success criteria | ❌ | No measurable definition of "skill worked correctly". |
| Writing-skills test methodology | ❌ | No alignment with `superpowers:writing-skills`: no trigger-accuracy test, no spurious-trigger negative test, no hallucination probe. |

### Recommended Test Scenario Skeleton

```
## Test Scenarios

### Scenario 1: Happy-path feature PR approval
Input:  user says "review PR #101"
Expect: skill runs `gitflow-cli pr view 101`, reads the diff,
        evaluates all 6 dimensions, produces conclusion markdown,
        runs `gitflow-cli review approve 101 --body "<conclusion>"`.

### Scenario 2: PR with security findings → request-changes
Input:  user says "please review PR #55"
Expect: skill detects hardcoded secret in diff, marks security as ⚠️,
        runs `gitflow-cli review request-changes 55 --body "<concluding SECURITY: ⚠️>"`.

### Scenario 3: Comment-only review (no verdict)
Input:  user says "leave a comment on PR #78, don't approve or reject"
Expect: skill runs `gitflow-cli review comment 78 --body "<findings>"`,
        does NOT call approve or request-changes.

### Scenario 4: User asks for inline line-level feedback
Input:  user says "leave inline comments on PR #42"
Expect: skill declines and redirects to `gitflow-pr-inline-review`
        because line-level comments are out of scope.

### Scenario 5: User asks to fix issues found during review
Input:  user says "review PR #30 and fix the problems you find"
Expect: skill performs the review but does NOT edit code;
        after submitting the conclusion, it redirects code-fix
        work to `gitflow-pr-apply-feedback` or the user.

### Scenario 6: PR not found (CLI error)
Input:  user says "review PR #99999" (non-existent)
Expect: skill surfaces the CLI error message, does NOT fabricate
        a review conclusion, advises user to verify the PR number.

### Baseline (without skill)
User manually types `gitflow-cli review approve <n> --body "LGTM"` without
reading the diff, without structured 6-dimensional analysis, and without
file:line-citation discipline — low review quality, misses security issues.
```

---

## Dimension 4: Alignment with Superpowers Best Practices

**Rating: ⚠️ 需改进 (Needs Improvement)**

### ✅ Compliant Aspects

| Practice | Status | Notes |
|----------|--------|-------|
| No flowchart abuse | ✅ | No flowchart at all; linear 5-step process is appropriate. |
| No embedded code in flowcharts | ✅ | N/A |
| Single-skill single-responsibility (conceptually) | ✅ | The skill does one thing: produce an overall review conclusion. |
| Has examples | ✅ | Three concrete command examples (approve, request-changes, comment). |
| Checklist quality | ✅ | 6 well-named dimensions that align with `CLAUDE.md` code-quality section. |
| Token efficiency | ✅ | ~168 lines, well within limits. |

### ❌ Gaps

| Practice | Status | Details |
|----------|--------|---------|
| TDD for skills (RED-GREEN-REFACTOR) | ❌ | No evidence of test-first design. No test section at all. |
| Description describes triggers only | ❌ | Description describes the full 6-dimensional workflow, not the trigger condition. |
| Keyword coverage (errors, symptoms, synonyms, tools) | ❌ | No trigger keyword table. Missing: "review PR", "check pull request", "approve PR", "代码审查", "pr review", `gitflow-cli review`, "LGTM". |
| Cross-references to related skills | ❌ | No `## See Also` or `## Related Skills` section. Should reference: `gitflow-pr`, `gitflow-pr-inline-review`, `gitflow-pr-apply-feedback`, `gitflow-security-check`, `gitflow-pipeline-analyzer`. |
| Quick Reference / Cheat Sheet | ❌ | No single-page summary for experienced users. |
| Pattern-language over narrative | ❌ | Uses tutorial prose rather than pattern + example + anti-pattern. |
| Error handling section | ⚠️ | The `## 注意事项` section lists cautions but does not define recovery paths (CLI auth failure, network timeout, nonexistent PR, empty diff). |

### Recommended Trigger Keywords

```
## Trigger Keywords

- Direct:   "review PR", "check pull request", "approve PR",
           "request changes on PR", "代码审查", "审查 PR"
- CLI:      `gitflow-cli review approve`, `gitflow-cli review request-changes`,
           `gitflow-cli review comment`
- Symptoms: user has opened a PR and wants reviewer feedback before merging
- Synonyms: "code review", "pull request review", "PR review",
           "look at this PR", "审查一下这个 PR"
```

---

## Improvement Recommendations

### P0 (Must Fix — blocking compliance)

1. **Rewrite `description` frontmatter** — convert from workflow-description to `Use when...` trigger-only format.
2. **Add Responsibility Boundary section** — `## ✅ Responsible For` / `## ❌ Not Responsible For` / `## 🚫 Do Not` lists that distinguish this skill from `gitflow-pr-inline-review`, `gitflow-pr-apply-feedback`, and `gitflow-pr`.
3. **Add Trigger Keywords section** — list direct phrases, CLI commands, synonyms, symptoms.
4. **Add Test Scenarios** — at least: happy-path approval, security-finding → request-changes, comment-only, redirect to inline-review, redirect to apply-feedback, and CLI-error propagation.
5. **Add When-NOT-to-Use table** — with common rationalization excuses and counter-arguments.

### P1 (Should Fix — recommended for polish)

6. **Add cross-references** — `## See Also` linking to `gitflow-pr`, `gitflow-pr-inline-review`, `gitflow-pr-apply-feedback`, `gitflow-security-check`, `gitflow-pipeline-analyzer`.
7. **Add Quick Reference section** — one-page cheat-sheet for advanced users (6-dimension checklist + CLI flags only).
8. **Add Common Mistakes section** — e.g., approving without reading the diff, conflating this skill with inline review, omitting file:line citations, skipping the security dimension.
9. **Add Red Flags section** — warnings about reviewing own PR, batch-reviewing, reviewing in CI pipelines, approving with known security issues.
10. **Convert narrative steps to pattern language** — each `步骤 N` becomes a named `## Pattern: <name>` with `### Anti-pattern` callouts.
11. **Add explicit error handling** — define recovery paths for CLI failures (auth error, network timeout, nonexistent PR, empty diff).

### P2 (Optional — nice to have)

12. **Baseline test** — explicit description of no-skill behavior for comparison.
13. **Stress test** — very large diff (>500 files), non-ASCII file paths, cross-repo PR, detached HEAD.
14. **Token budget** — if skill grows beyond 250 lines, split into `SKILL.md` (core) + `references/` (full checklist).
15. **Localization audit** — keep user-facing prose in Chinese; ensure `description` frontmatter is English per Superpowers convention.

---

## Success Criteria (Acceptance)

After the refactor is applied, the skill SHOULD pass the following checks:

- [ ] `description` matches `/^Use when/i` and contains no workflow description.
- [ ] Contains a `## ✅ Responsible For` / `## ❌ Not Responsible For` section.
- [ ] Contains a `## 🚫 Do Not` prohibition list with ≥ 3 items.
- [ ] Contains a `## Trigger Keywords` section with ≥ 5 trigger phrases.
- [ ] Contains a `## Test Scenarios` section with ≥ 4 scenarios (including 1 negative / redirect).
- [ ] Contains a `## See Also` cross-reference section.
- [ ] Contains a `## When NOT to Use this Skill` table with rationalization-counterpairs.
- [ ] Explicitly distinguishes overall-review from line-level inline review.
- [ ] `make check-agent-sync` passes (if applicable).
- [ ] `superpowers:writing-skills` review returns zero P0 findings.

---

## Summary Scorecard

| Dimension | Rating | Key Gap |
|-----------|--------|---------|
| 1. Structure & Documentation | ⚠️ 需改进 | Description violates trigger-only rule; missing canonical sections |
| 2. Responsibility Boundaries | ❌ 不合格 | No boundary, scope, prohibition, or red-flag declarations; high risk of scope creep into inline-review / apply-feedback / security-check |
| 3. Testability | ❌ 不合格 | No test scenarios, baseline, or success criteria |
| 4. Superpowers Best Practices | ⚠️ 需改进 | Missing keywords, cross-references, quick-ref; TDD not applied |

**Overall verdict:** The skill functions as a solid 6-dimensional review checklist with good command examples and a useful conclusion template, but it does not meet the writing-skills standard for trigger-accuracy, boundary clarity, testability, or anti-scope-creep guardrails. Because this skill sits at the center of a four-skill PR-review constellation (`pr-review`, `pr-inline-review`, `pr-apply-feedback`, `pr`), the absence of a boundary declaration is the single most dangerous gap — an agent loaded with this skill can easily drift into line-level commenting or code editing without realizing it has crossed into another skill's territory. The P0 recommendations should be applied before this skill is considered safe for autonomous invocation.
