# Skill Analysis: `gitflow-pr`

**Date:** 2026-07-07
**Source:** `skills/gitflow-pr/SKILL.md`
**GitHub Issue:** (to be created)
**Analyst:** implementer subagent

## Abstract

The `gitflow-pr` skill serves as a top-level command reference for the `gitflow-cli pr` command family. It documents 11 subcommands (`create`, `list`, `view`, `close`, `reopen`, `comment`, `merge`, `checkout`, `ready`, `wip`, `sync`) with parameter tables and usage examples. However, the skill reads like flattened CLI `--help` output rather than a Superpowers writing-skills compliant guide. It lacks workflow guidance, boundary declarations, trigger-accuracy, testability harnesses, and all structural sections required by the writing-skills methodology. This document provides a four-dimension analysis with prioritized improvement recommendations.

---

## Dimension 1: Skill Structure & Documentation Conventions

**Rating: ❌ 不合格 (Unacceptable)**

### ✅ Strengths

| Item | Status | Notes |
|------|--------|-------|
| YAML frontmatter present | ✅ | Has `name` and `description` fields |
| `name` field correct | ✅ | `gitflow-pr` matches directory name |
| File location | ✅ | `skills/gitflow-pr/SKILL.md` |
| Language consistency | ✅ | Entirely in Chinese, no dilution |
| Command examples | ✅ | Five concrete `gitflow-cli pr <subcommand>` invocations with realistic flags |
| Parameter tables | ✅ | Each subcommand has a consistent four-column parameter table |

### ❌ Deficiencies

| Item | Status | Details |
|------|--------|---------|
| `description` format | ❌ | Current: `"gitflow-cli 的 Pull Request 操作命令封装，支持创建、列表、查看、关闭、合并、检出、状态切换和分支同步"` — describes *functionality*, not the *trigger condition*. Superpowers convention requires `"Use when..."` trigger-only phrasing. |
| Structural sections | ❌ | Missing all canonical sections: `When to Use`, `Core Pattern`, `Quick Reference`, `Implementation`, `Common Mistakes`. Current structure is a flat command-reference dump with no layered design. |
| Token efficiency | ⚠️ | ~136 lines, ~600 Chinese words — likely exceeds the 500-word threshold for a frequently-loaded top-level skill because every `pr`-related invocation will pull in documentation for *all* 11 subcommands even when the user only needs one. |
| Anti-patterns present | ⚠️ | Pure reference-table dump with zero workflow guidance. The `## 使用示例` section shows only 5 of 11 subcommands — no coverage of `list`, `close`, `reopen`, `ready`, `wip`, or `sync` examples. Resembles exported `--help` output rather than a skill the agent can reason about. |
| Workflow abstraction | ❌ | No guidance on *when* to choose each subcommand, no decision tree, no pattern language linking user intent → subcommand selection. |

### Recommended Description Rewrite

```yaml
description: "Use when the user wants to manage Pull Requests through gitflow-cli — including creating, listing, viewing, closing, merging, checking out, commenting on, or syncing PRs. Triggers on phrases like 'create a PR', 'list open PRs', 'merge pull request', 'checkout PR locally', 'close this PR', or direct invocation of `gitflow-cli pr <subcommand>`."
```

---

## Dimension 2: Responsibility Boundary Clarity

**Rating: ❌ 不合格 (Unacceptable)**

### Missing Elements

| Required Element | Present? | Gap |
|------------------|----------|-----|
| Responsibility boundary declaration | ❌ | No `## Responsibility` or `## Boundary` section. The skill never articulates what it owns as a top-level entry point vs. what child skills (`gitflow-pr-create`, `gitflow-pr-review`, `gitflow-pr-inline-review`, `gitflow-pr-apply-feedback`) own. |
| Prohibition list (`🚫 Do not...`) | ❌ | No explicit prohibitions. Unclear whether this skill should: resolve merge conflicts, write code, run CI, triage PRs, manage labels, or perform code review. |
| Scope matrix (`✅ In scope` / `❌ Out of scope`) | ❌ | No scope table. Without it, the skill risks scope-creep into adjacent skills or over-simplification (acting as a dumb command wrapper). |
| "Rationalization excuse" counter-table | ❌ | No `## When NOT to use this skill` table that preempts common misapplications (e.g., "I can call `pr merge` after running tests — should I also call `pipeline-analyzer`?"). |
| Red Flags list | ❌ | No `## Red Flags` section warning about misapplication contexts (e.g., merging without review, closing without comment, force-merging across forks, bypassing branch protection). |
| Child skill delegation model | ❌ | This skill is a parent command wrapper (`pr`) with multiple child skills (`pr-create`, `pr-review`, `pr-inline-review`, `pr-apply-feedback`) that handle specific subcommands in depth. No guidance on when to delegate to a child skill vs. staying in the parent skill. |

### Recommended Scope Contract

```
## ✅ Responsible For
- Acting as the top-level entry point for all `gitflow-cli pr` operations
- Routing user intent to the correct subcommand or child skill
- Documenting parameter flags and types for all 11 subcommands
- Providing quick-reference command examples for common PR workflows
- Explaining the semantic difference between subcommands (e.g., `merge` strategies)

## ❌ Not Responsible For
- Linear PR creation workflow with branch validation (→ gitflow-pr-create)
- Inline code review with comment annotation (→ gitflow-pr-inline-review)
- Applying reviewer feedback and resolving comments (→ gitflow-pr-apply-feedback)
- Full PR review with approval/rejection (→ gitflow-pr-review)
- Pipeline analysis and CI checking (→ gitflow-pipeline-analyzer)
- Release management involving PRs (→ gitflow-release)
- Label and milestone management (→ gitflow-label-milestone)

## 🚫 Do Not
- Merge a PR without confirming the user has reviewed the diff
- Close a PR without leaving an explanatory comment
- Force-merge (rebase strategy) across forks without explicit user confirmation
- Invoke merge strategies the user did not request (`squash` vs `merge` vs `rebase`)
- Skip branch-protection checks before merging
- Create PRs from protected branches (delegate to gitflow-pr-create for validation)
- Make API calls without confirming the target repository (`--repo` resolution)

## 🔁 Delegation Rules
- If user wants to **create** a PR → delegate to `gitflow-pr-create`
- If user wants to **review** a PR with inline comments → delegate to `gitflow-pr-inline-review`
- If user wants to **apply feedback** from a review → delegate to `gitflow-pr-apply-feedback`
- If user wants a **full review** (approve/reject) → delegate to `gitflow-pr-review`
- For all other operations (list, view, close, reopen, comment, merge, checkout, ready, wip, sync) → remain in this skill and execute directly
```

---

## Dimension 3: Testability

**Rating: ❌ 不合格 (Unacceptable)**

### Missing Elements

| Required Element | Present? | Gap |
|------------------|----------|-----|
| Test scenarios | ❌ | No `## Test Scenarios` section with concrete input/expected-output pairs. |
| Baseline test (no-skill behavior) | ❌ | No description of what happens without the skill — user must manually recall subcommand names, flag names, and merge-strategy semantics. |
| Stress / edge-case scenarios | ❌ | No coverage of: non-existent PR number, cross-repository PR, conflicting merge, authentication failure, network timeout, detached HEAD during checkout, sync on a PR already merged, or commenting on a closed PR. |
| Success criteria | ❌ | No measurable definition of "skill worked correctly" (e.g., "PR URL is returned", "merge commit is created", "user was warned before destructive action"). |
| Writing-skills test methodology | ❌ | No alignment with the `superpowers:writing-skills` approach: no trigger-accuracy test, no spurious-trigger negative test, no hallucination probe, no subcommand-delegation probe. |

### Recommended Test Scenario Skeleton

```
## Test Scenarios

### Scenario 1: Happy-path — squash merge
Input:  user says "merge PR #101 with squash"
Expect: skill confirms squash strategy with user, invokes
        `gitflow-cli pr merge 101 --strategy squash`, returns merge commit SHA.

### Scenario 2: View non-existent PR
Input:  user says "show me PR #9999"
Expect: skill invokes `gitflow-cli pr view 9999`, receives 404,
        outputs "PR #9999 not found — confirm the number and repository",
        does not hallucinate PR details.

### Scenario 3: Close without comment (destructive)
Input:  user says "close PR #55"
Expect: skill warns that closing is irreversible and asks whether to
        add a closing comment first (via `pr comment  --body "..."`),
        does not execute `pr close 55` before confirmation.

### Scenario 4: Checkout PR from fork
Input:  user says "checkout PR #78 and review it locally"
Expect: skill invokes `gitflow-cli pr checkout 78`, confirms local branch
        was created, suggests next step: run `git flow pr review 78`.

### Scenario 5: Sync merged PR (edge case)
Input:  user says "sync PR #30" but PR #30 is already merged
Expect: skill detects merged state via `pr view 30 --state merged`,
        warns that syncing a merged PR is a no-op, advises user to
        delete the branch instead.

### Scenario 6: Wrong delegation (negative probe)
Input:  user says "create a draft PR for the auth feature"
Expect: skill delegates to `gitflow-pr-create` instead of trying to
        run `gitflow-cli pr create --draft` inline.
Baseline: user manually invokes `pr create` without branch validation,
          creates a PR from the wrong branch.
```

---

## Dimension 4: Alignment with Superpowers Best Practices

**Rating: ❌ 不合格 (Unacceptable)**

### ✅ Compliant Aspects

| Practice | Status | Notes |
|----------|--------|-------|
| No flowchart abuse | ✅ | No flowchart; uses tables which are appropriate for command reference. |
| No embedded code in flowcharts | ✅ | N/A |
| Single-skill single-responsibility (conceptually) | ⚠️ | The skill documents all 11 subcommands — acting as both top-level router and command encyclopedia. This is inherently multi-responsibility without delegation guidance. |
| Has examples | ⚠️ | 5 examples covering only `create`, `view`, `merge`, `checkout`, `ready`, `sync` — missing `list`, `close`, `reopen`, `comment`, `wip`. |
| Parameter documentation | ✅ | Complete parameter tables for all 11 subcommands. |

### ❌ Gaps

| Practice | Status | Details |
|----------|--------|---------|
| TDD for skills (RED-GREEN-REFACTOR) | ❌ | No evidence of test-first design. No test section at all. |
| Description describes triggers only | ❌ | Description lists supported operations instead of defining when the skill should be loaded. |
| Keyword coverage (errors, symptoms, synonyms, tools) | ❌ | No trigger keyword table. Missing: "merge pull request", "close this PR", "list open PRs", "checkout PR", "approve PR", "sync branch", "draft PR", `gitflow-cli pr <subcommand>`, "pull request", "merge request" (GitLab context), "code review". |
| Cross-references to related skills | ❌ | No `## See Also` or `## Related Skills` section. Should reference: `gitflow-pr-create`, `gitflow-pr-review`, `gitflow-pr-inline-review`, `gitflow-pr-apply-feedback`, `gitflow-pipeline-analyzer`, `gitflow-release`, `gitflow-issue`. |
| Quick Reference / Cheat Sheet | ⚠️ | The command overview table functions as a pseudo quick-reference but lacks: per-subcommand one-liners, exit-code semantics, and common flag combinations. |
| Pattern-language over narrative | ❌ | No pattern language at all — pure tabular reference with no decision guidance. |
| Error handling section | ❌ | No `## Error Handling` section. CLI failure modes (auth, network, 404, 409, merge conflict) are not addressed. |
| Delegation contract | ❌ | No guidance on when to invoke child skills (`gitflow-pr-create`, `gitflow-pr-review`, etc.) vs. executing inline. |

### Recommended Trigger Keywords

```
## Trigger Keywords

- Direct:     "create a PR", "list PRs", "view PR", "close PR", "merge PR",
              "checkout PR", "comment on PR", "mark PR ready", "mark PR draft",
              "sync PR branch", "reopen PR", "pull request", "merge request"
- CLI:        `gitflow-cli pr <subcommand>`
- Symptoms:   user wants to move a branch through the PR lifecycle,
              user asks about PR status or state, user wants local PR review.
- Synonyms:   "open a pull request", "submit PR", "approve PR", "decline PR",
              "MR" (GitLab context), "code review"
```

### Recommended Cross-References

```
## See Also

- gitflow-pr-create         — linear workflow for creating a PR with validation
- gitflow-pr-review         — full review workflow with approve/reject
- gitflow-pr-inline-review  — inline code review with line-level comments
- gitflow-pr-apply-feedback — resolve reviewer comments and apply changes
- gitflow-pipeline-analyzer — check CI status before merging
- gitflow-release           — release workflow involving PR merges
- gitflow-issue             — create issues linked to PRs
```

---

## Improvement Recommendations

### P0 (Must Fix — blocking compliance)

1. **Rewrite `description` frontmatter** — convert from functionality-description to `Use when...` trigger-only format.
2. **Add Responsibility Boundary section** — `## ✅ Responsible For` / `## ❌ Not Responsible For` / `## 🚫 Do Not` lists with a delegation rules table for child skills.
3. **Add Trigger Keywords section** — list direct phrases, CLI commands, synonyms, symptoms (≥ 10 entries).
4. **Add Test Scenarios** — at least: happy-path squash merge, non-existent PR 404, close-without-comment destructive guard, checkout from fork, sync-on-merged edge case, delegation negative probe.
5. **Add When-NOT-to-Use table** — with common rationalization excuses and counter-arguments (e.g., "I can merge right after creating — should I delegate to `pr-create` instead?").
6. **Add Error Handling section** — define recovery paths for auth failure, 404, 409 (merge conflict), network timeout, cross-repo permission denied.

### P1 (Should Fix — recommended for polish)

7. **Add cross-references** — `## See Also` linking to all six adjacent skills (`gitflow-pr-create`, `gitflow-pr-review`, `gitflow-pr-inline-review`, `gitflow-pr-apply-feedback`, `gitflow-pipeline-analyzer`, `gitflow-release`).
8. **Add Quick Reference section** — compact one-liner per subcommand with the three most common flag combinations, ordered by frequency of use.
9. **Add Common Mistakes section** — e.g., merging without confirming strategy, closing without comment, checking out a PR when local changes exist, forgetting `--repo` on monorepo, confusing `ready`/`wip` semantics after merge.
10. **Add Red Flags section** — warnings about bypassing branch protection, merging across forks, force-merging without confirmation, closing others' PRs without team policy check, ignoring failing CI.
11. **Add missing usage examples** — cover `list`, `close`, `reopen`, `comment`, `wip` which currently have zero invocation examples.
12. **Add subcommand decision guide** — a short "which subcommand?" decision tree or pattern table mapping user intent → subcommand.

### P2 (Optional — nice to have)

13. **Baseline test** — explicit description of no-skill behavior (agent guesses from `--help` output).
14. **Stress test** — Unicode title, very long body (>10k chars), detached HEAD during checkout, non-existent base branch, merge on conflicting PR, rate-limited API.
15. **Token budget** — consider splitting `SKILL.md` into a lean ~200-word router (`description` + delegation table + quick-ref) with subcommand detail moved to `references/pr-commands.md`. This keeps the frequently-loaded parent skill lean.
16. **Localization audit** — ensure all examples use ASCII shell/script names; keep user-facing prose in Chinese.
17. **Exit-code semantics table** — document expected exit codes (0 success, 1 usage error, 2 auth failure, 3 not-found, 4 conflict, 5 network) and agent behavior per code.
18. **Idempotency notes** — document which subcommands are idempotent (`view`, `list`, `ready`, `wip`) vs. destructive (`merge`, `close`) so the agent can apply appropriate confirmation guards.

---

## Success Criteria (Acceptance)

After the refactor is applied, the skill SHOULD pass the following checks:

- [ ] `description` matches `/^Use when/i` and contains no functionality enumeration.
- [ ] Contains a `## ✅ Responsible For` / `## ❌ Not Responsible For` section.
- [ ] Contains a `## 🚫 Do Not` prohibition list with ≥ 5 items.
- [ ] Contains a `## 🔁 Delegation Rules` table routing to child skills.
- [ ] Contains a `## Trigger Keywords` section with ≥ 10 trigger phrases.
- [ ] Contains a `## Test Scenarios` section with ≥ 4 scenarios (including 1 negative probe).
- [ ] Contains a `## See Also` cross-reference section with ≥ 4 related skills.
- [ ] Contains a `## When NOT to Use this Skill` table with rationalization-counterpairs.
- [ ] Contains a `## Error Handling` section covering ≥ 4 failure modes.
- [ ] Contains a `## Quick Reference` section with one-liners for all 11 subcommands.
- [ ] `make check-agent-sync` passes (if applicable).
- [ ] `superpowers:writing-skills` review returns zero P0 findings.

---

## Summary Scorecard

| Dimension | Rating | Key Gap |
|-----------|--------|---------|
| 1. Structure & Documentation | ❌ 不合格 | Description violates trigger-only rule; skill dumps raw CLI reference without canonical sections; likely over token budget for a top-level skill |
| 2. Responsibility Boundaries | ❌ 不合格 | No boundary, scope, prohibition, or red-flag declarations; missing child-skill delegation model entirely |
| 3. Testability | ❌ 不合格 | No test scenarios, baseline, error-case coverage, or success criteria |
| 4. Superpowers Best Practices | ❌ 不合格 | Missing keywords, cross-references, delegation guide, error handling; TDD not applied |

**Overall verdict:** The skill functions as a flat command reference dump — essentially an exported man page — with no workflow guidance, no boundary contracts, no decision patterns, no delegation model to child skills, and no test harness. It does not meet any writing-skills standard beyond syntactic frontmatter correctness. The P0 recommendations must be applied before this skill is considered safe for autonomous invocation. Critically, this skill occupies the top-level `pr` namespace while four child skills already own specific subcommands; the parent must become a lean router + delegator rather than a redundant reference copy.
