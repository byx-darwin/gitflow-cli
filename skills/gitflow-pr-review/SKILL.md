---
name: gitflow-pr-review
description: |
  Use when the user requests an overall code review of a Pull Request — covering 6 dimensions and submitting an approve, request-changes, or comment-only verdict via gitflow-cli.
  当要求对 PR 进行整体代码审查（6 维度评估）并提交审批/要求修改/评论结论时使用。
---

# gitflow-pr-review

Performs a 6-dimension assessment of a PR diff and submits an overall verdict via `gitflow-cli review`. Line-level inline comments belong to `gitflow-pr-inline-review`.

## When to Use

| English | 中文 | Context |
|---------|------|---------|
| review PR | 审查 PR | overall verdict |
| approve / LGTM | 审批 / 通过 | post-analysis |
| request changes | 要求修改 | PR blocked |
| inline comments / line review | 逐行评论 | → `gitflow-pr-inline-review` |
| merge / close PR | 合并 / 关闭 PR | → `gitflow-pr` |

## Core Pattern

```bash
gitflow-cli pr view <n>          # 1. verify open
gitflow-cli pr diff <n>          # 2. fetch diff
# 3. assess 6 dimensions; 4. draft conclusion
gitflow-cli review <verdict> <n> --body "<conclusion>"  # 5. submit
```

## Quick Reference

| Goal | Command |
|------|---------|
| Approve | `gitflow-cli review approve <n> --body "<conclusion>"` |
| Request changes | `gitflow-cli review request-changes <n> --body "<conclusion>"` |
| Comment only | `gitflow-cli review comment <n> --body "<conclusion>"` |

Dimensions: correctness, security, performance, maintainability, test-coverage, documentation. Full items: [checklist](../references/pr-review-checklist.md).

## Implementation

### Preconditions

- Open PR — `gitflow-cli pr view <n>`

### Step 1: Fetch

`gitflow-cli pr view <n>` then `gitflow-cli pr diff <n>`. Confirm open, not draft/merged. Empty diff → stop.

### Step 2: Assess 6 Dimensions

For each dimension (correctness, security, performance, maintainability, test-coverage, docs): ✅ or ⚠️ with `path:line`. See [checklist](../references/pr-review-checklist.md).

### Step 3: Draft Conclusion

Per-dimension verdicts with `path:line` for ⚠️ items. See [template](../references/pr-review-checklist.md).

### Step 4: Submit

- All ✅ → `gitflow-cli review approve <n> --body "<conclusion>"`
- Any ⚠️ → `gitflow-cli review request-changes <n> --body "<conclusion>"`
- Comment only → `gitflow-cli review comment <n> --body "<conclusion>"`

Output PR URL.

### Error Handling

- `pr view` 404 → stop. Check PR number.
- Empty diff → stop. PR may be merged.
- Auth failure → run `gitflow-cli auth login`.
- `review` fails → surface error, stop.

## Responsibility

### ✅ In Scope

- Fetch PR metadata + diff
- 6-dimension assessment
- Conclusion with `path:line` citations
- Submit verdict via `gitflow-cli review`

### ❌ Out of Scope

- Line-level inline comments → `gitflow-pr-inline-review`
- Applying fixes → `gitflow-pr-apply-feedback`
- PR lifecycle → `gitflow-pr`
- Deep security scanning → `gitflow-security-check`

### 🚫 Do Not

- ❌ Verdict before reading diff
- ❌ Publish `[logic]`/`[security]` inline comments — that is `gitflow-pr-inline-review`
- ❌ Edit source or run `cargo fix` from findings
- ❌ Merge / close after approve
- ❌ Skip security — even for small changes

## 🔁 Delegation

| User Intent | Delegate To |
|-------------|-------------|
| Inline line-level review | `/gitflow-pr-inline-review` |
| Apply reviewer feedback | `/gitflow-pr-apply-feedback` |
| Merge / close / reopen PR | `/gitflow-pr` |
| Submit review decision | `/gitflow-review` |

## Rationalization Excuses

| Excuse | Reality |
|--------|---------|
| "Small change, skip" | One-liners can hide vulnerabilities. |
| "Inline faster" | Inline is `gitflow-pr-inline-review`'s job. |

## Red Flags

- 🚩 "approve without reviewing" — Refuse. Read diff.
- 🚩 "leave line comments" — → `gitflow-pr-inline-review`.
- 🚩 "fix the issues" — → `gitflow-pr-apply-feedback`.

## Test Scenarios

### 1: Happy Path

- **Given** PR #101 open
- **When** "review #101"
- **Then** Fetches diff, approves #101, outputs URL

### 2: Negative — Inline Comments

- **Given** Wants line-level
- **When** "Leave inline comments on #101"
- **Then** NOT loaded. → `gitflow-pr-inline-review`.

### 3: Boundary — Apply Fixes

- **Given** User asks to fix findings
- **When** "review #101 and fix"
- **Then** Submits request-changes. No edits. → `gitflow-pr-apply-feedback`.

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
- ❌ **Publishing inline comments** — line-level belongs to `gitflow-pr-inline-review`.

## Trigger Keywords

| English | 中文 |
|---------|------|
| review PR, check pull request | 审查 PR |
| approve, LGTM | 审批、通过 |
| request changes, reject | 要求修改、驳回 |
| code review verdict | 代码审查结论 |
| overall PR review | 整体审查 PR |

## See Also

- `gitflow-pr-inline-review` — line-level inline comments
- `gitflow-pr-apply-feedback` — applies feedback as code changes
- `gitflow-pr` — PR lifecycle: merge/close/ready
- `gitflow-review` — approve/request-changes/comment wrapper
- `docs/references/pr-review-checklist.md` — full 6-dim checklist
- `docs/superpowers/templates/skill-conventions.md` — conventions
