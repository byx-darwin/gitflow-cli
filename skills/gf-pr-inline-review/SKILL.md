---
name: gf-pr-inline-review
description: |
  Use when user requests line-level inline review comments on a specific Pull Request.
  当用户要求对 PR 进行逐行行内评论审查时使用。
---

# gf-pr-inline-review

Publishes inline comments on PR changed lines. No review decisions, no code fixes, no unchanged-line comments.

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
| inline review | 行内审查 | line-level PR comments |
| line-level comments | 逐行评论 | per-line feedback |
| review PR line by line | 逐行审查 PR | per-file analysis |
| overall PR review | PR 总体审查 | **NOT** → `gf-pr-review` |
| approve / request changes | 审批 PR | **NOT** → `gf-review` |

## When NOT to Use

| Scenario | Why Not | Use Instead |
|----------|---------|-------------|
| Submitting an overall review verdict | This skill publishes line-level comments, not overall verdicts | `/gf-pr-review` for 6-dimension review with approve/request-changes |
| Approving or requesting changes | This skill only posts inline comments, not formal verdicts | `/gf-review` for submitting approve/request-changes verdicts |
| Applying feedback as code fixes | This skill comments on code, not modifies it | `/gf-pr-apply-feedback` for applying fixes from review comments |
| Merging or closing the PR | This skill has no lifecycle operations | `/gf-pr` for merge/close/reopen |
| Commenting on unchanged lines | This skill only comments on `+` (added) lines in the diff | Use platform web UI for general PR comments |
| Publishing without user confirmation | This skill requires explicit confirmation before every publish | Non-negotiable — draft review is mandatory |

## Core Pattern

```bash
gf pr view <n>    # 1. verify PR open
gf pr diff <n>          # 2. fetch diff
# 3. analyze → draft comments
# 4. show draft → await user confirm
gf commit comment <sha> --body "<c>" --path <f> --line <l>  # 5. publish
```

## Quick Reference

| Goal | Command |
|------|---------|
| Fetch | `gf pr view <n>` + `gf pr diff <n>` |
| Publish | `gf commit comment <sha> --body "<body>" --path <file> --line <n>` |

**Dimensions:** `[logic]` `[security]` `[naming]` `[boundary]`

## Implementation

### Preconditions

- PR `<n>` exists and `open` — `gf pr view <n>`
- Platform is GitHub / GitLab / GitCode
- `gf` authenticated — `gf auth status`

### Step 1: Fetch Diff

`gf pr diff <n>`. Parse files, hunks, `+` lines. Empty → stop.

### Step 2: Analyze

For each `+` line assess `[logic]` (conditionals, loops, await), `[security]` (injection, secrets), `[naming]` (convention), `[boundary]` (empty, overflow, races). Draft:

```
**[dim]** <summary> <detail> **suggest:** ``<lang> <fix>``
```

### Step 3: Show Draft — Await Confirmation ⚠️

Present draft. **STOP. Do NOT publish until user confirms.** Non-skippable.

### Step 4: Publish

For each approved comment: `gf commit comment <head-sha> --body "<body>" --path <file> --line <line>`. Use PR HEAD sha, repo-relative path, `+` line number.

### Step 5: Summary

Output PR number, files reviewed, per-dimension counts, per-comment table.

### Error Handling

| Error | Recovery |
|-------|----------|
| 404 / empty diff / not open | Stop |
| API failure | Log, continue |
| >15 findings | Discuss first |

## Responsibility

### ✅ In Scope

- Fetch diff, analyze, draft, publish after confirmation, output summary

### ❌ Out of Scope

- Review decisions → `gf-review`; code fixes → `gf-pr-apply-feedback`; overall summary → `gf-pr-review`; PR lifecycle → `gf-pr`

### 🚫 Do Not

- ❌ Publish without approval
- ❌ Guess line numbers — use `+` only
- ❌ Comment unchanged lines; ❌ Publish merged PRs; ❌ Cross-post

## 🔁 Delegation Rules

| User Intent | Delegate To | Reason |
|-------------|-------------|--------|
| Inline review | This skill | Per-line diff + publish |
| Overall verdict | `/gf-pr-review` | 6-dim checklist |
| Apply feedback | `/gf-pr-apply-feedback` | Code changes |
| PR lifecycle | `/gf-pr` | merge/close/etc |

## Rationalization Excuses

| Excuse | Reality |
|--------|---------|
| "Author needs this" | Requires user consent |
| "Constructive feedback" | Needs approval first |
| "Line numbers probably right" | Verify against diff |
| "I'm helping review" | Decisions belong to user |
| "Skip confirmation once" | Every skip risks comment |

## Red Flags

- 🚩 "Just publish it" — Refuse. Show draft first.
- 🚩 "Skip the confirmation" — Refuse. Cite Step 3. Stop.

## Test Scenarios

### 1: Happy Path
- **Given** PR #101 open 3 files — **When** "Review inline" — **Then** fetches, drafts, shows draft, awaits, publishes

### 2: Negative
- **Given** PR #101 — **When** "Approve PR" — **Then** NOT loaded. → `gf-review`.

### 3: Boundary
- **Given** 5 drafts — **When** "Publish now" without draft — **Then** Refuses — shows draft first.

### 4: Error
- **Given** PR merged — **When** "Review" — **Then** "PR merged — refusing." Stops.

## Success Criteria

- [ ] Comments only after user confirmation
- [ ] All target verified `+` lines
- [ ] No out-of-diff comments

## Common Mistakes

- ❌ **Publishing without draft** — violates Step 3. Always present draft first.
- ❌ **Guessed line numbers** — must come from diff `+` output, not memory.

## Trigger Keywords

| English | 中文 |
|---------|------|
| inline review | 行内审查 |
| line-level review | 逐行审查 |
| inline comments on PR | PR 行内评论 |

## See Also

- `gf-pr-create` — create a PR
- `gf-pr-review` — overall review with decision
- `gf-pr` — PR lifecycle operations
- `gf-pr-apply-feedback` — applies feedback locally
- `gf-review` — approve/request-changes/comment decisions
