---
name: gitflow-pr-inline-review
description: |
  Use when user requests line-level inline review comments on a specific Pull Request.
  当用户要求对 PR 进行逐行行内评论审查时使用。
---

# gitflow-pr-inline-review

Publishes inline comments on PR changed lines. No review decisions, no code fixes, no unchanged-line comments.

## When to Use

| English | 中文 | Context |
|---------|------|---------|
| inline review | 行内审查 | line-level PR comments |
| line-level comments | 逐行评论 | per-line feedback |
| review PR line by line | 逐行审查 PR | per-file analysis |
| overall PR review | PR 总体审查 | **NOT** → `gitflow-pr-review` |
| approve / request changes | 审批 PR | **NOT** → `gitflow-review` |

## Core Pattern

```bash
gitflow-cli pr view <n>          # 1. verify PR open
gitflow-cli pr diff <n>          # 2. fetch diff
# 3. analyze → draft comments
# 4. show draft → await user confirm
gitflow-cli comment <sha> --body "<c>" --path <f> --line <l>  # 5. publish
```

## Quick Reference

| Goal | Command |
|------|---------|
| Fetch PR | `gitflow-cli pr view <n>` + `gitflow-cli pr diff <n>` |
| Publish inline comment | `gitflow-cli comment <sha> --body "<body>" --path <file> --line <n>` |

**Dimensions:** `[logic]` `[security]` `[naming]` `[boundary]`

## Implementation

### Preconditions

- PR `<n>` exists and `open` — `gitflow-cli pr view <n>`
- Platform is GitHub / GitLab / GitCode
- `gitflow-cli` authenticated — `gitflow-cli auth status`

### Step 1: Fetch Diff

`gitflow-cli pr diff <n>`. Parse files, hunks, `+` lines. Empty → stop.

### Step 2: Analyze

For each `+` line assess `[logic]` (conditionals, loops, await), `[security]` (injection, secrets), `[naming]` (convention), `[boundary]` (empty, overflow, races). Draft:

```
**[dim]** <summary> <detail> **suggest:** ``<lang> <fix>``
```

### Step 3: Show Draft — Await Confirmation ⚠️

Present draft. **STOP. Do NOT publish until user explicitly confirms.** Non-skippable — see Rationalization Excuses.

### Step 4: Publish

For each approved comment: `gitflow-cli comment <head-sha> --body "<body>" --path <file> --line <line>`. Use PR HEAD sha, repo-relative path, `+` line number.

### Step 5: Summary

Output PR number, files reviewed, per-dimension counts, per-comment table.

### Error Handling

| Error | Recovery |
|-------|----------|
| `pr view` 404 / empty diff / not open | Stop |
| `comment` API failure | Log, continue |
| Auth failure | "Run `gitflow-cli auth login`", stop |
| >15 findings | Stop. Discuss first |

## Responsibility

### ✅ In Scope

- Fetch diff, analyze, draft, publish after confirmation, output summary

### ❌ Out of Scope

- Review decisions → `gitflow-review`; code fixes → `gitflow-pr-apply-feedback`; overall summary → `gitflow-pr-review`; PR lifecycle → `gitflow-pr`

### 🚫 Do Not

- ❌ Publish without user approval
- ❌ Guess line numbers — use only `+` from diff
- ❌ Comment unchanged/deleted lines; ❌ Publish closed/merged PRs; ❌ Copy across PRs; ❌ Auto-publish

## 🔁 Delegation Rules

| User Intent | Delegate To | Reason |
|-------------|-------------|--------|
| Inline line-level review | This skill | Per-line diff analysis + publish |
| Overall review verdict | `/gitflow-pr-review` | 6-dimension checklist + decision |
| Apply reviewer feedback | `/gitflow-pr-apply-feedback` | Code modification + resolve |
| Create a PR | `/gitflow-pr-create` | Branch validation + title/desc collection |
| PR lifecycle (merge/close) | `/gitflow-pr` | Lifecycle operation |

## Rationalization Excuses

| Excuse | Reality |
|--------|---------|
| "Author needs this" | Requires user consent |
| "Constructive feedback" | Needs approval first |
| "Line numbers probably right" | Verify against diff |
| "I'm helping review" | Decisions belong to user |
| "Skip confirmation once" | Every skip risks comment |

## Red Flags

- 🚩 "Just publish it" / "直接发" — Refuse. Show draft first.
- 🚩 "Skip the confirmation" — Refuse. Cite Step 3. Stop.

## Test Scenarios

### 1: Happy Path
- **Given** PR #101 open 3 files — **When** "Review inline" — **Then** fetches, drafts, shows draft, awaits, publishes

### 2: Negative
- **Given** PR #101 — **When** "Approve PR" — **Then** NOT loaded. → `gitflow-review`.

### 3: Boundary
- **Given** 5 drafts — **When** "Publish now" without draft — **Then** Refuses — shows draft first.

### 4: Error
- **Given** PR merged — **When** "Review" — **Then** "PR merged — refusing." Stops.

## Success Criteria

- [ ] Comments published only after explicit user confirmation
- [ ] All comments target verified `+` numbers
- [ ] No unchanged/deleted/out-of-diff comments

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

- `gitflow-pr-review` — overall review with decision
- `gitflow-pr` — PR lifecycle operations
- `gitflow-pr-apply-feedback` — applies feedback locally
- `gitflow-review` — approve/request-changes/comment decisions
- `docs/superpowers/templates/skill-conventions.md` — skill conventions
