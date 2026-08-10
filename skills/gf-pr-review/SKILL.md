---
name: gf-pr-review
description: |
  Use when the user requests an overall code review of a Pull Request
  and needs to submit a verdict via gf.
  当要求对 PR 进行整体代码审查并提交审查结论时使用。
---

# gf-pr-review

6-dimension PR diff assessment + overall verdict via `gf review`. Line-level comments → `gf-pr-inline-review`.

## CLI Requirement

**MUST use `gf` CLI, NOT `gh` CLI.**

| CLI | Scope | Platform Support |
|-----|-------|------------------|
| `gf` | This project | GitHub + GitLab + GitCode |
| `gh` | GitHub only | GitHub only |

**Why**: `gf` is the unified CLI for this project. Using `gh` breaks GitLab/GitCode compatibility.

## Preconditions
- `gf` installed: `command -v gf`
- `gf` authenticated: `gf auth status`
## When to Use

| English | 中文 | Context |
|---------|------|---------|
| review PR | 审查 PR | overall verdict |
| approve / LGTM | 审批 / 通过 | post-analysis |
| request changes | 要求修改 | PR blocked |
| inline / line review | 逐行评论 | → `gf-pr-inline-review` |
| merge / close | 合并/关闭 | → `gf-pr` |

## When NOT to Use

| Scenario | Why Not | Use Instead |
|----------|---------|-------------|
| Leaving line-level inline comments | This skill submits an overall verdict, not per-line comments | `/gf-pr-inline-review` for line-level inline review |
| Applying review fixes to code | This skill assesses code and submits verdict, not modifies source | `/gf-pr-apply-feedback` for applying fixes |
| Merging or closing the PR | This skill submits review verdicts, not lifecycle operations | `/gf-pr` for merge/close/reopen |
| Creating a Pull Request | This skill reviews existing PRs only | `/gf-pr-create` for PR creation workflow |
| Deep security vulnerability scanning | This skill includes security as one of 6 dimensions, not a full audit | `/gf-security-check` for comprehensive security audit |
| Submitting verdict without reading diff | This skill requires full diff analysis before verdict | Non-negotiable — always read diff first |

## Core Pattern

```bash
gf pr view <n>              # 1. verify
gh pr diff <n>                          # 2. diff
# 3. assess 6 dims; draft conclusion
gf review approve <n> --body "<c>"      # 4. submit (or request-changes / comment)
```

## Quick Reference

| Goal | Command |
|------|---------|
| Approve | `gf review approve <n> --body "<c>"` |
| Request changes | `gf review request-changes <n> --body "<c>"` |
| Comment | `gf review comment <n> --body "<c>"` |

Dimensions: correctness, security, performance, maintainability, test-coverage, documentation. Full items: [checklist](../references/pr-review-checklist.md).

## Implementation

### Preconditions

- Open PR — `gf pr view <n>`

### Step 1: Fetch

`gf pr view <n>` then `gh pr diff <n>`. Confirm open, not draft/merged. Empty diff → stop.

### Step 2: Assess 6 Dimensions

For each dimension (correctness, security, performance, maintainability, test-coverage, docs): ✅ or ⚠️ with `path:line`. See [checklist](../references/pr-review-checklist.md).

### Step 3: Draft Conclusion

Per-dimension verdicts with `path:line` for ⚠️ items. See [template](../references/pr-review-checklist.md).

### Step 4: Submit

- All ✅ → `gf review approve <n> --body "<conclusion>"`
- Any ⚠️ → `gf review request-changes <n> --body "<conclusion>"`
- Comment only → `gf review comment <n> --body "<conclusion>"`

Output PR URL.

### Error Handling

- `pr view` 404 → stop. Check PR number.
- Empty diff → stop. PR may be merged.
- Auth failure → run `gf auth login`.
- `review` fails → surface error, stop.

## Responsibility

### ✅ In Scope

- Fetch PR metadata + diff
- 6-dimension assessment
- Conclusion with `path:line` citations
- Submit verdict via `gf review`

### ❌ Out of Scope

- Line-level inline comments → `gf-pr-inline-review`
- Applying fixes → `gf-pr-apply-feedback`
- PR lifecycle → `gf-pr`
- Deep security scanning → `gf-security-check`

### 🚫 Do Not

- ❌ Verdict before reading diff
- ❌ Publish `[logic]`/`[security]` inline comments — that is `gf-pr-inline-review`
- ❌ Edit source or run `cargo fix` from findings
- ❌ Merge / close after approve
- ❌ Skip security — even for small changes

## 🔁 Delegation

| User Intent | Delegate To |
|-------------|-------------|
| Inline review | `/gf-pr-inline-review` |
| Apply feedback | `/gf-pr-apply-feedback` |
| Merge / close | `/gf-pr` |

## Rationalization Excuses

| Excuse | Reality |
|--------|---------|
| "Small change, skip" | One-liners can hide vulnerabilities. |
| "Inline faster" | Inline is `gf-pr-inline-review`'s job. |

## Red Flags

- 🚩 "approve without reviewing" — Refuse. Read diff.
- 🚩 "leave line comments" — → `gf-pr-inline-review`.
- 🚩 "fix the issues" — → `gf-pr-apply-feedback`.

## Test Scenarios

### 1: Happy Path

- **Given** PR #101 open
- **When** "review #101"
- **Then** Fetches diff, approves #101, outputs URL

### 2: Negative — Inline Comments

- **Given** Wants line-level
- **When** "Leave inline comments on #101"
- **Then** NOT loaded. → `gf-pr-inline-review`.

### 3: Boundary — Apply Fixes

- **Given** User asks to fix findings
- **When** "review #101 and fix"
- **Then** Submits request-changes. No edits. → `gf-pr-apply-feedback`.

### 4: Error — PR Not Found

- **Given** PR #99999 doesn't exist
- **When** "review #99999"
- **Then** `pr view` 404. No fabricated verdict.

## Success Criteria

- [ ] Verdict submitted with PR URL
- [ ] All 6 dimensions assessed; ⚠️ cite `path:line`
- [ ] Security evaluated
- [ ] No inline comments / fix / merge

## Common Mistakes

- ❌ **Approving without reading diff** — violates Preconditions. Read diff first.
- ❌ **Publishing inline comments** — line-level belongs to `gf-pr-inline-review`.

## Trigger Keywords

| English | 中文 |
|---------|------|
| review PR, check pull request | 审查 PR |
| approve, LGTM | 审批、通过 |
| request changes, reject | 要求修改、驳回 |
| code review verdict | 代码审查结论 |
| overall PR review | 整体审查 PR |

## See Also

- `gf-pr-create` — create a PR
- `gf-pr-inline-review` — line-level inline comments
- `gf-pr-apply-feedback` — applies feedback as code changes
- `gf-pr` — PR lifecycle
