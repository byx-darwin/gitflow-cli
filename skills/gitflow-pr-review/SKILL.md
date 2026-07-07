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
| code review verdict | 代码审查结论 | assessment |
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

Dimensions: `correctness`, `security`, `performance`, `maintainability`, `test-coverage`, `documentation`. Full items: [checklist](../references/pr-review-checklist.md).

## Implementation

### Preconditions

- `open` PR — `gitflow-cli pr view <n>` non-404
- Authenticated — `gitflow-cli auth status`

### Step 1: Fetch

`gitflow-cli pr view <n>` then `gitflow-cli pr diff <n>`. Confirm open, not draft/merged. Empty diff → stop.

### Step 2: Assess 6 Dimensions

For each dimension (`correctness`, `security`, `performance`, `maintainability`, `test-coverage`, `documentation`): ✅ or ⚠️ with `path:line`. See [checklist](../references/pr-review-checklist.md).

### Step 3: Draft Conclusion

Per-dimension verdicts with `path:line` for ⚠️ items. See [template](../references/pr-review-checklist.md).

### Step 4: Submit

- All ✅ → `gitflow-cli review approve <n> --body "<conclusion>"`
- Any ⚠️ → `gitflow-cli review request-changes <n> --body "<conclusion>"`
- Comment only → `gitflow-cli review comment <n> --body "<conclusion>"`

Output PR URL.

### Error Handling

| Error | Recovery |
|-------|----------|
| `pr view` 404 / not open | Stop. Check PR number. |
| Empty diff | Stop. PR may be merged. |
| Auth failure | "Run `gitflow-cli auth login`", stop. |
| `review` API failure | Surface error, stop. |
| Unclear verdict | Ask before submit. |

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
- ❌ Skip security dimension — even for small changes

## Rationalization Excuses

| Excuse | Reality |
|--------|---------|
| "Small change, skip analysis" | One-liners can hide vulnerabilities. |
| "Inline comments faster" | Inline feedback is `gitflow-pr-inline-review`'s job. |
| "Author needs verdict quickly" | Verdict requires full diff review. |

## Red Flags

- 🚩 "approve without reviewing" — Refuse. Read diff first.
- 🚩 "leave line comments too" — Refuse. → `gitflow-pr-inline-review`.
- 🚩 "fix the issues you find" — Refuse. → `gitflow-pr-apply-feedback`.

## Test Scenarios

### Scenario 1: Happy Path

- **Given** PR #101 open, authenticated
- **When** "review PR #101"
- **Then** Fetches diff, calls `review approve 101`, outputs PR URL

### Scenario 2: Negative — Inline Comments

- **Given** User wants line-level feedback
- **When** "Leave inline comments on PR #101"
- **Then** NOT loaded. → `gitflow-pr-inline-review`.

### Scenario 3: Boundary or Error

- **Given** User asks to fix findings OR PR #99999 doesn't exist
- **When** "review PR #101 and fix" or "review PR #99999"
- **Then** For boundary: submits request-changes, no edits, → `gitflow-pr-apply-feedback`. For error: `pr view` 404, surfaces error, no fabricated verdict.

## Success Criteria

- [ ] Verdict submitted and PR URL returned
- [ ] All 6 dimensions assessed; ⚠️ cite `path:line`
- [ ] Security evaluated (never skipped)
- [ ] No inline comments, no fix/merge commands

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

- `gitflow-pr-inline-review` — line-level inline comments on diff
- `gitflow-pr-apply-feedback` — applies feedback as code changes
- `gitflow-pr` — PR lifecycle: merge/close/ready
- `gitflow-review` — approve / comment / request-changes
- `docs/references/pr-review-checklist.md` — full 6-dim checklist
- `docs/superpowers/templates/skill-conventions.md` — conventions
