# Skill Analysis: `gitflow-issue-review`

**Date:** 2026-07-07
**Source:** `skills/gitflow-issue-review/SKILL.md`
**GitHub Issue:** #33
**Analyst:** implementer subagent

## Abstract

The `gitflow-issue-review` skill is a six-step linear workflow that fetches an issue, evaluates its requirement completeness across three dimensions (title clarity, description sufficiency, acceptance criteria clarity), writes a report, posts it as a comment, and cleans up temp files. While the three-dimension analysis framework is well-structured and the skill has a clear single purpose, it lacks the structural rigor, boundary declarations, testability guarantees, and trigger-accuracy required by the Superpowers writing-skills methodology. This document provides a four-dimension analysis with prioritized improvement recommendations.

---

## Dimension 1: Skill Structure & Documentation Conventions

**Rating: ⚠️ 需改进 (Needs Improvement)**

### ✅ Strengths

| Item | Status | Notes |
|------|--------|-------|
| YAML frontmatter present | ✅ | Has `name` and `description` fields |
| `name` field correct | ✅ | `gitflow-issue-review` matches directory name |
| File location | ✅ | `skills/gitflow-issue-review/SKILL.md` |
| Language consistency | ✅ | Entirely in Chinese, no dilution |
| Three-dimension framework | ✅ | Title / Description / Acceptance Criteria is a clear, memorable analysis model |
| Quality grading system | ✅ | 🟢/🟡/🔴 three-level scale per dimension is concrete and actionable |
| Report template | ✅ | Step 3 provides a complete Markdown template for the analysis output |
| Single-purpose clarity | ✅ | The skill does one thing: review an issue's requirement completeness |

### ❌ Deficiencies

| Item | Status | Details |
|------|--------|---------|
| `description` format | ❌ | Current: `"Issue 需求分析工作流 — 获取 Issue 详情，从标题清晰度、描述充分度、验收标准明确度三个维度分析需求完整性，输出改进建议并回写到 Issue 评论"` — describes the *full workflow*, not *when* to trigger. Superpowers convention requires `"Use when..."` trigger-only phrasing. |
| Structural sections | ⚠️ | Missing canonical sections: `When to Use`, `Core Pattern`, `Quick Reference`, `Implementation`, `Common Mistakes`. Current structure is a flat workflow list without the layered design writing-skills prescribes. |
| Trigger keywords | ❌ | No `Trigger` section enumerating user phrases or contextual cues (e.g., "review this issue", "is this issue ready", "需求分析"). |
| Token efficiency | ⚠️ | ~598 words total. The skill body is ~380 words, which is under the 500-word limit for a full skill. However, the two narrative examples in "使用示例" consume ~120 words and are redundant with the Step 3 template. Removing them would bring the skill well under budget. |
| Anti-patterns present | ⚠️ | "使用示例" section contains two full narrative walkthroughs (a feature issue and a bug issue) that duplicate the Step 3 template. This is the "narrative example" anti-pattern — the examples tell a story rather than illustrating the pattern contract. |
| Temp file usage | ⚠️ | Step 4 writes to `/tmp/issue-analysis.md` and Step 5 reads it back. This is an unnecessary indirection — the skill could pass the report content directly. The temp file pattern adds failure modes (permissions, disk space, concurrent runs) without benefit. |

### Recommended Description Rewrite

```yaml
description: "Use when the user wants to evaluate the requirement completeness of a GitHub issue — including checking title clarity, description sufficiency, and acceptance criteria. Triggers on phrases like 'review this issue', 'analyze issue requirements', 'is this issue ready', '需求分析', or direct invocation of issue review workflows."
```

---

## Dimension 2: Responsibility Boundary Clarity

**Rating: ❌ 不合格 (Unacceptable)**

### Missing Elements

| Required Element | Present? | Gap |
|------------------|----------|-----|
| Responsibility boundary declaration | ❌ | No `## Responsibility` or `## Boundary` section. The skill never states what it owns vs. what the underlying CLI owns. |
| Prohibition list (`🚫 Do not...`) | ❌ | No explicit prohibitions. Unclear whether the skill should: edit the issue title/body, close the issue, add labels, or modify assignees. |
| Scope matrix (`✅ In scope` / `❌ Out of scope`) | ❌ | No scope table. Without it, the skill risks scope creep into `gitflow-issue-triage` (label management), `gitflow-issue` (issue editing), or `gitflow-pr-review` (code review). |
| "Rationalization excuse" counter-table | ❌ | No `## When NOT to use this skill` table that preempts common justifications for misapplication. |
| Red Flags list | ❌ | No `## Red Flags` section warning about misapplication contexts. |

### Specific Problems

1. **Comment posting is a side effect with no guardrails** — The skill posts a comment to the issue (Step 5). There is no declaration of:
   - Whether it should post without user confirmation
   - Whether it should check for duplicate review comments before posting
   - Whether it should post on closed issues
   - Whether it should post on issues authored by the bot itself

2. **No idempotency declaration** — If the skill runs twice on the same issue, it will post a duplicate comment. The skill does not declare whether it should check for an existing review comment before posting.

3. **Ambiguous relationship with `gitflow-issue-triage`** — Both skills operate on issues. Without a scope boundary, the agent might:
   - Add labels during review (triage territory)
   - Reassign the issue (triage territory)
   - Close the issue if it's "not ready" (issue management territory)

4. **No prohibition on modifying the issue** — The skill's stated purpose is analysis, but there is no explicit prohibition against editing the issue title/body to "fix" the problems it identifies.

### Recommended Scope Contract

```
## ✅ Responsible For
- Fetching issue details via `gitflow-cli issue view`
- Evaluating requirement completeness across 3 dimensions (title, description, acceptance criteria)
- Generating a structured analysis report
- Posting the report as a comment on the issue
- Cleaning up temporary files

## ❌ Not Responsible For
- Editing or closing issues (→ gitflow-issue)
- Managing labels or milestones (→ gitflow-label-milestone, gitflow-issue-triage)
- Triaging or prioritizing issues (→ gitflow-issue-triage)
- Reviewing code or PRs (→ gitflow-pr-review, gitflow-pr-inline-review)
- Implementing the suggested improvements (→ gitflow-workflow)

## 🚫 Do Not
- Post a review comment without user confirmation
- Post duplicate reviews — check for existing review comments first
- Edit the issue title, body, or metadata
- Add labels, assignees, or milestones
- Close or reopen the issue
- Analyze closed issues without explicit user request
- Proceed if `gitflow-cli issue view` returns an error
```

---

## Dimension 3: Testability

**Rating: ❌ 不合格 (Unacceptable)**

### Missing Elements

| Required Element | Present? | Gap |
|------------------|----------|-----|
| Test scenarios | ❌ | No `## Test Scenarios` section with concrete input/expected-output pairs. |
| Baseline test (no-skill behavior) | ❌ | No description of what happens without the skill. |
| Stress / edge-case scenarios | ❌ | No coverage of: non-existent issue, private repo, API failure, empty description, very long issue body, Unicode-only issues. |
| Success criteria | ❌ | No measurable definition of "skill worked correctly". |
| Writing-skills test methodology | ❌ | No alignment with the `superpowers:writing-skills` approach. |

### Specific Problems

1. **Zero testability surface** — There is no way to verify that the skill produces correct behavior:
   - How do we know the three-dimension evaluation is applied consistently?
   - How do we know the comment is posted to the correct issue?
   - How do we know the skill doesn't post on closed issues?

2. **No baseline contrast** — Without the skill, the agent might:
   - Run `gitflow-cli issue view` and give a free-form opinion
   - Skip the structured three-dimension analysis
   - Forget to post the comment
   - The skill's value proposition (structured analysis + persistent comment) is not articulated as a testable delta.

3. **Missing edge cases** — The skill does not address:
   - Issue number does not exist → `gitflow-cli issue view` fails
   - Issue has empty description → all three dimensions should be 🔴
   - Issue already has a review comment → should we skip or append?
   - API rate limit → `gitflow-cli issue comment` fails
   - Issue is closed → should we still review?
   - Issue is a PR (some platforms treat PRs as issues) → should we reject?

4. **No success criteria** — Should define:
   - A structured report is generated with all three dimensions evaluated
   - The report is posted as a comment on the target issue
   - No duplicate comment is posted on re-run
   - The skill exits gracefully on CLI errors

### Recommended Test Scenario Skeleton

```
## Test Scenarios

### Scenario 1: Happy-path feature request
Input:  user says "review issue #42"
Expect: skill fetches #42, evaluates 3 dimensions, generates report,
        posts comment, cleans up temp file.

### Scenario 2: Issue with empty description
Input:  issue #10 has title "fix: bug" but empty body
Expect: title 🟢 or 🟡, description 🔴, acceptance 🔴,
        report includes actionable suggestions.

### Scenario 3: Non-existent issue
Input:  user says "review issue #99999"
Expect: skill detects CLI error, reports "issue not found", does not
        attempt to post a comment.

### Scenario 4: Duplicate review (idempotency)
Input:  user runs review on #42 twice
Expect: skill detects existing review comment, warns user, does not
        post duplicate (or offers to overwrite).

### Scenario 5: Closed issue
Input:  user says "review issue #5" but #5 is closed
Expect: skill warns that issue is closed, asks for confirmation before
        proceeding.

### Baseline (without skill)
User asks "review issue #42" — agent runs `gitflow-cli issue view 42`
and gives a free-form textual opinion. No structured report, no
persistent comment, no consistent evaluation framework.
```

---

## Dimension 4: Alignment with Superpowers Best Practices

**Rating: ❌ 不合格 (Unacceptable)**

### ✅ Compliant Aspects

| Practice | Status | Notes |
|----------|--------|-------|
| No flowchart abuse | ✅ | No flowchart; uses sequential steps which are appropriate for a linear workflow. |
| No embedded code in flowcharts | ✅ | N/A |
| Single-skill single-responsibility (conceptually) | ✅ | The skill does one thing: review an issue's requirements. |
| Has examples | ✅ | Two concrete command examples provided (though they are narrative anti-patterns). |
| Quality framework is memorable | ✅ | The three-dimension model (标题/描述/验收标准) with 🟢/🟡/🔴 grading is a strong pattern. |

### ❌ Gaps

| Practice | Status | Details |
|----------|--------|---------|
| TDD for skills (RED-GREEN-REFACTOR) | ❌ | No evidence of test-first design. No test section at all. |
| Description describes triggers only | ❌ | Description describes the full 6-step workflow, not just the trigger condition. |
| Keyword coverage (errors, symptoms, synonyms, tools) | ❌ | No trigger keyword table. Missing: "review issue", "analyze issue", "issue quality", "需求分析", "issue 审查", `gitflow-cli issue view`. |
| Cross-references to related skills | ❌ | No `## See Also` or `## Related Skills` section. Should reference: `gitflow-issue`, `gitflow-issue-triage`, `gitflow-issue-create`, `gitflow-label-milestone`. |
| Quick Reference / Cheat Sheet | ❌ | No single-page summary. The three-dimension criteria are embedded in step descriptions rather than extracted as a quick-reference table. |
| Pattern-language over narrative | ⚠️ | The three-dimension framework is good pattern language, but the "使用示例" section uses narrative walkthroughs. |
| Idempotency / re-entrance safety | ❌ | No declaration of what happens when the skill runs twice on the same issue. |

### Recommended Trigger Keywords

```
## Trigger Keywords

- Direct:   "review this issue", "analyze issue requirements", "issue quality check",
            "需求分析", "审查 issue", "issue 完整吗"
- CLI:      `gitflow-cli issue view`
- Symptoms: user is unsure if an issue is ready for development, user asks
            "is this issue clear enough", user wants feedback on issue quality
- Synonyms: "issue review", "requirement analysis", "issue assessment",
            "issue critique", "backlog grooming"
```

### Recommended Quick Reference

```
## Quick Reference

| Dimension | 🟢 Excellent | 🟡 Acceptable | 🔴 Insufficient |
|-----------|--------------|---------------|-----------------|
| Title     | Conventional prefix + scope + clear | Understandable but vague | Missing prefix or too vague |
| Description | Background + goal + constraints | Basic info, missing some | Empty or title-only |
| Acceptance | Specific, verifiable, covers edge cases | Present but vague | Missing or unverifiable |
```

---

## Improvement Recommendations

### P0 (Must Fix — blocking compliance)

| # | Item | Dimension | Description |
|---|------|-----------|-------------|
| P0-1 | Rewrite `description` frontmatter | D1, D4 | Convert from workflow-description to `Use when...` trigger-only format. Current description includes the full 6-step workflow. |
| P0-2 | Add Responsibility Boundary section | D2 | Add `## ✅ Responsible For` / `## ❌ Not Responsible For` / `## 🚫 Do Not` lists. Must explicitly prohibit editing/closing issues and posting without confirmation. |
| P0-3 | Add idempotency declaration | D2, D4 | Declare behavior on re-run: check for existing review comment, warn user, do not post duplicate. |
| P0-4 | Add Trigger Keywords section | D4 | List direct phrases, CLI commands, synonyms, symptoms. Cover both English and Chinese triggers. |
| P0-5 | Add Test Scenarios | D3 | At least: happy-path, empty description, non-existent issue, duplicate review, closed issue, API failure. |
| P0-6 | Add When-NOT-to-Use table | D2 | With common rationalization excuses and counter-arguments (e.g., "I can fix the title while reviewing" → No, analysis only). |

### P1 (Should Fix — recommended for polish)

| # | Item | Dimension | Description |
|---|------|-----------|-------------|
| P1-1 | Add cross-references | D4 | `## See Also` linking to `gitflow-issue`, `gitflow-issue-triage`, `gitflow-issue-create`, `gitflow-label-milestone`. |
| P1-2 | Add Quick Reference section | D1, D4 | Extract the three-dimension criteria into a single-page cheat sheet. |
| P1-3 | Add Common Mistakes section | D1 | e.g., posting without confirmation, reviewing closed issues, editing the issue during review, inconsistent grading. |
| P1-4 | Add Red Flags section | D2 | Warnings about: reviewing bot-authored issues, reviewing in bulk, posting on issues you don't own, reviewing PRs as issues. |
| P1-5 | Remove narrative examples | D1 | Replace the two "使用示例" walkthroughs with 1-2 line command patterns. The Step 3 template already serves as the example. |
| P1-6 | Eliminate temp file indirection | D1 | Pass report content directly instead of writing to `/tmp/issue-analysis.md` and reading back. Reduces failure modes. |
| P1-7 | Add preconditions section | D1 | Declare: must be in a git repo, `gitflow-cli` must be installed, user must have issue read/write access. |

### P2 (Optional — nice to have)

| # | Item | Dimension | Description |
|---|------|-----------|-------------|
| P2-1 | Baseline test | D3 | Explicit description of no-skill behavior for comparison. |
| P2-2 | Stress test | D3 | Very long issue body (10k+ chars), Unicode-only content, issue with 50+ comments, rate-limit handling. |
| P2-3 | Token budget | D1 | If skill grows beyond 200 lines after P0/P1 fixes, split into `SKILL.md` (core) + `references/` (examples). |
| P2-4 | Add error handling section | D1 | Cover: CLI not found, auth failure, network timeout, malformed API response. |
| P2-5 | Add confirmation prompt pattern | D2 | Before posting the comment, show the user the report and ask for confirmation. |
| P2-6 | Support batch review | D4 | Allow reviewing multiple issues in sequence (with explicit opt-in). |

---

## Success Criteria (Acceptance)

After the refactor is applied, the skill SHOULD pass the following checks:

- [ ] `description` matches `/^Use when/i` and contains no workflow description.
- [ ] Contains a `## ✅ Responsible For` / `## ❌ Not Responsible For` section.
- [ ] Contains a `## 🚫 Do Not` prohibition list with ≥ 4 items.
- [ ] Contains a `## Trigger Keywords` section with ≥ 6 trigger phrases (including Chinese).
- [ ] Contains a `## Test Scenarios` section with ≥ 5 scenarios (including 1 negative).
- [ ] Contains a `## See Also` cross-reference section with ≥ 3 related skills.
- [ ] Contains a `## When NOT to Use this Skill` table with rationalization-counterpairs.
- [ ] Contains an idempotency declaration for re-run behavior.
- [ ] Contains a `## Quick Reference` section with the three-dimension criteria table.
- [ ] No narrative walkthrough examples; examples are ≤ 3 lines each.
- [ ] Temp file indirection is eliminated (or justified if retained).
- [ ] `superpowers:writing-skills` review returns zero P0 findings.

---

## Summary Scorecard

| Dimension | Rating | Key Gap |
|-----------|--------|---------|
| 1. Structure & Documentation | ⚠️ 需改进 | Description violates trigger-only rule; narrative examples; temp file indirection |
| 2. Responsibility Boundaries | ❌ 不合格 | No boundary, scope, prohibition, idempotency, or red-flag declarations |
| 3. Testability | ❌ 不合格 | No test scenarios, baseline, success criteria, or edge-case coverage |
| 4. Superpowers Best Practices | ❌ 不合格 | Missing keywords, cross-references, quick-ref; TDD not applied; no idempotency |

**Overall verdict:** The skill has a strong three-dimension analysis framework that is its core intellectual value, but the surrounding skill contract is underdeveloped. It functions as a workflow tutorial rather than a testable, boundary-safe, trigger-accurate skill. The P0 recommendations should be applied before this skill is considered safe for autonomous invocation. The three-dimension model (标题清晰度 / 描述充分度 / 验收标准明确度 with 🟢/🟡/🔴 grading) is the skill's greatest asset and should be preserved and elevated as the Quick Reference centerpiece during refactor.
