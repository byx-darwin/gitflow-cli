# Skill Analysis: `gitflow-pr-create`

**Date:** 2026-07-07
**Source:** `skills/gitflow-pr-create/SKILL.md`
**GitHub Issue:** #27
**Analyst:** implementer subagent

## Abstract

The `gitflow-pr-create` skill is a linear eight-step workflow guide for creating Pull Requests through the `gitflow-cli pr create` command. It covers branch validation, change review, base-branch freshness checks, title/description collection, target-branch confirmation, PR creation, and post-creation guidance. While the workflow is logically complete and the command examples are concrete, the skill lacks the structural rigor, boundary declarations, trigger-accuracy, and testability guarantees required by the Superpowers writing-skills methodology. This document provides a four-dimension analysis with prioritized improvement recommendations.

---

## Dimension 1: Skill Structure & Documentation Conventions

**Rating: ⚠️ 需改进 (Needs Improvement)**

### ✅ Strengths

| Item | Status | Notes |
|------|--------|-------|
| YAML frontmatter present | ✅ | Has `name` and `description` fields |
| `name` field correct | ✅ | `gitflow-pr-create` matches directory name |
| File location | ✅ | `skills/gitflow-pr-create/SKILL.md` |
| Language consistency | ✅ | Entirely in Chinese, no dilution |
| Token efficiency | ✅ | ~158 lines, under 500-word limit for a full skill guide |
| Command examples | ✅ | Three concrete `gitflow-cli pr create` invocations with realistic flags |
| Checklist template | ✅ | PR description template includes a useful Checklist section |

### ❌ Deficiencies

| Item | Status | Details |
|------|--------|---------|
| `description` format | ❌ | Current: `"引导用户完成 Pull Request 创建工作流 — 检查分支、变更和 base 状态，填写标题描述后调用 gitflow-cli pr create"` — describes the *entire workflow*, not the *trigger condition*. Superpowers convention requires `"Use when..."` trigger-only phrasing. |
| Structural sections | ⚠️ | Missing canonical sections: `When to Use`, `Core Pattern`, `Quick Reference`, `Implementation`, `Common Mistakes`. Current structure is a flat sequential workflow without the layered design writing-skills prescribes. |
| Trigger keywords | ❌ | No `Trigger Keywords` section enumerating error messages, user phrases, or contextual cues (e.g., "create a PR", "open a pull request", `pr create` command). |
| Anti-patterns present | ⚠️ | Uses narrative prose for workflow steps rather than pattern-language. Steps read like a tutorial ("确认当前所在分支...") rather than a skill contract. The `## 使用示例` section mixes narrative commentary with code blocks. |

### Recommended Description Rewrite

```yaml
description: "Use when the user wants to create a Pull Request through gitflow-cli — including feature PRs, bug-fix PRs, and draft PRs. Triggers on phrases like 'create a PR', 'open a pull request', 'submit changes for review', or direct invocation of `gitflow-cli pr create`."
```

---

## Dimension 2: Responsibility Boundary Clarity

**Rating: ❌ 不合格 (Unacceptable)**

### Missing Elements

| Required Element | Present? | Gap |
|------------------|----------|-----|
| Responsibility boundary declaration | ❌ | No `## Responsibility` or `## Boundary` section. The skill never states what it owns vs. what the underlying CLI owns. |
| Prohibition list (`🚫 Do not...`) | ❌ | No explicit prohibitions. Unclear whether the skill should: edit existing PRs, merge PRs, review PRs, resolve comments, or manage labels. |
| Scope matrix (`✅ In scope` / `❌ Out of scope`) | ❌ | No scope table. Without it, the skill risks scope creep into `gitflow-pr-review`, `gitflow-pr-inline-review`, `gitflow-pr-apply-feedback`, or `gitflow-pr` (general PR operations). |
| "Rationalization excuse" counter-table | ❌ | No `## When NOT to use this skill` table that preempts common justifications for misapplication (e.g., "I can also review the PR right after creating — should I call pr-review instead?"). |
| Red Flags list | ❌ | No `## Red Flags` section warning about misapplication contexts (e.g., creating PRs from CI pipelines, batch-creation, creating PRs with uncommitted changes, or cross-repository PRs). |

### Recommended Scope Contract

```
## ✅ Responsible For
- Validating current branch is not a protected branch (main, master, release/*)
- Checking that the branch has been pushed to the remote
- Reviewing change scope (diff stat, commit log) for conventional-commit compliance
- Ensuring base branch is up-to-date to minimize merge conflicts
- Collecting PR title (with conventional-commit prefix) and structured description
- Confirming --head and --base branches with the user
- Invoking `gitflow-cli pr create` with collected parameters
- Presenting the resulting PR URL and next-step guidance (draft → ready, or notify reviewers)

## ❌ Not Responsible For
- Reviewing PRs (→ gitflow-pr-review, gitflow-pr-inline-review)
- Applying review feedback (→ gitflow-pr-apply-feedback)
- Merging or closing PRs (→ gitflow-pr)
- Managing PR labels or assignees at scale (→ gitflow-label-milestone)
- Resolving merge conflicts (→ gitflow-pr or manual rebase)
- Running CI/CD pipelines (→ gitflow-pipeline-analyzer)

## 🚫 Do Not
- Create a PR from a protected branch (main, master, release/*)
- Create a PR with un-pushed local commits without warning the user
- Force-push or rebase without explicit user confirmation
- Merge the PR immediately after creation
- Add reviewers or labels not approved by the user
- Create PRs across multiple repositories from a single invocation
```

---

## Dimension 3: Testability

**Rating: ❌ 不合格 (Unacceptable)**

### Missing Elements

| Required Element | Present? | Gap |
|------------------|----------|-----|
| Test scenarios | ❌ | No `## Test Scenarios` section with concrete input/expected-output pairs. |
| Baseline test (no-skill behavior) | ❌ | No description of what happens without the skill — user must manually recall CLI flags, branch validation steps, and PR template conventions. |
| Stress / edge-case scenarios | ❌ | No coverage of: Unicode titles, very long descriptions, empty diff, detached HEAD state, non-existent base branch, network failure, or CLI error codes. |
| Success criteria | ❌ | No measurable definition of "skill worked correctly" (e.g., "a valid PR URL is displayed", "base branch was confirmed up-to-date", "user approved the final command"). |
| Writing-skills test methodology | ❌ | No alignment with the `superpowers:writing-skills` approach: no trigger-accuracy test, no spurious-trigger negative test, no hallucination probe. |

### Recommended Test Scenario Skeleton

```
## Test Scenarios

### Scenario 1: Happy-path feature PR
Input:  user says "create a PR for the two-factor auth feature"
Expect: skill checks current branch (feature/two-factor-auth), confirms not on main,
        reviews diff stat and commit log, verifies base (main) is up-to-date,
        collects title "feat(cli): add two-factor authentication support",
        collects structured description with Closes #N, invokes CLI, returns PR URL.

### Scenario 2: Draft PR creation
Input:  user says "open a draft PR for the LRU cache work in progress"
Expect: skill follows same validation flow, adds --draft flag to final command,
        reminds user to call `gitflow-cli pr ready <number>` after completion.

### Scenario 3: Branch not pushed to remote
Input:  user on feature/new-branch with local-only commits
Expect: skill detects missing upstream via `git rev-parse --abbrev-ref @{u}`,
        warns user that PR cannot be created, advises `git push -u origin <branch>`,
        ends without invoking CLI.

### Scenario 4: Base branch outdated
Input:  user's branch is behind origin/main (merge-base check fails)
Expect: skill detects stale base, advises `git rebase origin/main` or `git merge origin/main`,
        does not proceed with PR creation until user confirms base is updated.

### Scenario 5: On protected branch
Input:  user on `main` branch
Expect: skill detects protected branch, warns that PRs cannot be created from main,
        advises switching to a feature branch, ends without invoking CLI.

### Baseline (without skill)
User manually types `gitflow-cli pr create` — must recall flag names (--title, --body,
--head, --base, --draft), conventional-commit prefix convention, Markdown body format,
and branch validation steps from memory.
```

---

## Dimension 4: Alignment with Superpowers Best Practices

**Rating: ⚠️ 需改进 (Needs Improvement)**

### ✅ Compliant Aspects

| Practice | Status | Notes |
|----------|--------|-------|
| No flowchart abuse | ✅ | No flowchart; uses sequential steps which are appropriate for a linear workflow. |
| No embedded code in flowcharts | ✅ | N/A |
| Single-skill single-responsibility (conceptually) | ✅ | The skill does one thing: create a PR. |
| Has examples | ✅ | Three concrete command examples (feature PR, draft PR, fix PR). |
| Conventional-commit alignment | ✅ | Title prefix table and examples follow conventional-commit format. |

### ❌ Gaps

| Practice | Status | Details |
|----------|--------|---------|
| TDD for skills (RED-GREEN-REFACTOR) | ❌ | No evidence of test-first design. No test section at all. |
| Description describes triggers only | ❌ | Description describes the full workflow, not just the trigger condition. |
| Keyword coverage (errors, symptoms, synonyms, tools) | ❌ | No trigger keyword table. Missing: "create a PR", "open a pull request", "submit for review", "pr create", `gitflow-cli pr create`, "draft PR", "pull request". |
| Cross-references to related skills | ❌ | No `## See Also` or `## Related Skills` section. Should reference: `gitflow-pr`, `gitflow-pr-review`, `gitflow-pr-inline-review`, `gitflow-pr-apply-feedback`, `gitflow-pipeline-analyzer`. |
| Quick Reference / Cheat Sheet | ❌ | No single-page summary for experienced users. |
| Pattern-language over narrative | ❌ | Uses tutorial prose rather than pattern + example + anti-pattern structure. |
| Error handling section | ⚠️ | The `## 注意事项` section lists cautions but does not define error recovery paths (e.g., CLI exit code ≠ 0, network timeout, auth failure). |

### Recommended Trigger Keywords

```
## Trigger Keywords

- Direct:   "create a PR", "open a pull request", "submit for review", "create draft PR"
- CLI:      `gitflow-cli pr create`
- Symptoms: user has completed work on a branch and wants reviewer feedback
- Synonyms: "pull request", "draft PR", "code review request", "merge request" (GitLab context)
```

---

## Improvement Recommendations

### P0 (Must Fix — blocking compliance)

1. **Rewrite `description` frontmatter** — convert from workflow-description to `Use when...` trigger-only format.
2. **Add Responsibility Boundary section** — `## ✅ Responsible For` / `## ❌ Not Responsible For` / `## 🚫 Do Not` lists.
3. **Add Trigger Keywords section** — list direct phrases, CLI commands, synonyms, symptoms.
4. **Add Test Scenarios** — at least: happy-path feature PR, draft PR, unpushed branch, outdated base, and protected-branch guard.
5. **Add When-NOT-to-Use table** — with common rationalization excuses and counter-arguments.

### P1 (Should Fix — recommended for polish)

6. **Add cross-references** — `## See Also` linking to `gitflow-pr`, `gitflow-pr-review`, `gitflow-pr-inline-review`, `gitflow-pr-apply-feedback`, `gitflow-pipeline-analyzer`.
7. **Add Quick Reference section** — one-page command cheat-sheet for advanced users.
8. **Add Common Mistakes section** — e.g., creating PR from main, un-pushed commits, unescaped quotes in `--body`, missing conventional-commit prefix, omitting Closes #N linkage.
9. **Add Red Flags section** — warnings about CI pipeline usage, batch creation, cross-repo PRs, and creating PRs with uncommitted changes.
10. **Convert narrative steps to pattern language** — each `步骤 N` becomes a named `## Pattern: <name>` with `### Anti-pattern` callouts.
11. **Add explicit error handling** — define recovery paths for CLI failures (auth error, network timeout, non-existent base branch, merge conflict detected).

### P2 (Optional — nice to have)

12. **Baseline test** — explicit description of no-skill behavior for comparison.
13. **Stress test** — Unicode title, very long body (>10k chars), empty diff, detached HEAD, non-fast-forward base.
14. **Token budget** — if skill grows beyond 200 lines, split into `SKILL.md` (core) + `references/` (examples).
15. **Localization audit** — ensure all examples use ASCII shells/script names; keep user-facing prose in Chinese.

---

## Success Criteria (Acceptance)

After the refactor is applied, the skill SHOULD pass the following checks:

- [ ] `description` matches `/^Use when/i` and contains no workflow description.
- [ ] Contains a `## ✅ Responsible For` / `## ❌ Not Responsible For` section.
- [ ] Contains a `## 🚫 Do Not` prohibition list with ≥ 3 items.
- [ ] Contains a `## Trigger Keywords` section with ≥ 5 trigger phrases.
- [ ] Contains a `## Test Scenarios` section with ≥ 4 scenarios (including 1 negative).
- [ ] Contains a `## See Also` cross-reference section.
- [ ] Contains a `## When NOT to Use this Skill` table with rationalization-counterpairs.
- [ ] No `cargo` commands are embedded in narrative; only in ` ```bash ` code blocks.
- [ ] `make check-agent-sync` passes (if applicable).
- [ ] `superpowers:writing-skills` review returns zero P0 findings.

---

## Summary Scorecard

| Dimension | Rating | Key Gap |
|-----------|--------|---------|
| 1. Structure & Documentation | ⚠️ 需改进 | Description violates trigger-only rule; missing canonical sections |
| 2. Responsibility Boundaries | ❌ 不合格 | No boundary, scope, prohibition, or red-flag declarations |
| 3. Testability | ❌ 不合格 | No test scenarios, baseline, or success criteria |
| 4. Superpowers Best Practices | ⚠️ 需改进 | Missing keywords, cross-references, quick-ref; TDD not applied |

**Overall verdict:** The skill functions as a linear PR-creation tutorial with solid command examples and a useful description template, but it does not meet the writing-skills standard for trigger-accuracy, boundary clarity, testability, or anti-scope-creep guardrails. The P0 recommendations should be applied before this skill is considered safe for autonomous invocation.
