---
name: gitflow-issue-review
description: |
  Use when the user wants to analyze an Issue's requirement completeness (title clarity, description sufficiency, acceptance criteria) and post findings as an Issue comment.
  当用户希望分析 Issue 需求完整性（标题清晰度、描述充分度、验收标准）并回写评论时使用。
---

# gitflow-issue-review

Three-dimensional Issue requirement review — title clarity / description sufficiency / acceptance criteria — emits a structured analysis report, then posts it as an Issue comment. Does not edit the Issue itself.

## When to Use

| English | 中文 | Context |
|---------|------|---------|
| review the requirement | 审查需求质量 | check description completeness |
| is this issue clear enough | 这个 Issue 描述够吗 | before triage/triage |
| improve issue description | 改进 Issue 描述 | before development |
| triage this issue | 对 Issue 进行分类 | **NOT** → `/gitflow-issue-triage` |

## Core Pattern

```bash
gitflow-cli issue view <n>
# analyze 3 dimensions → write /tmp/issue-analysis.md
gitflow-cli issue comment <n> --body-file /tmp/issue-analysis.md
rm -f /tmp/issue-analysis.md
```

## Quick Reference

| Goal | Command |
|------|---------|
| Fetch Issue | `gitflow-cli issue view <n>` |
| Post comment | `gitflow-cli issue comment <n> --body-file <path>` |

**Three dimensions:** Title clarity · Description sufficiency · Acceptance criteria

## Implementation

### Preconditions

- Issue `<n>` exists — `issue view <n>`
- `gitflow-cli` authenticated
- Write access to Issue comments

### Step 1: Fetch — `issue view <n>`. Record title, body, labels, links, comments.

### Step 2: Score each dimension 🟢/🟡/🔴

| Dimension | Checks |
|-----------|--------|
| Title | conventional prefix · scope · unambiguous · length |
| Description | context · goal · constraints · references |
| Acceptance | `- [ ]` format · verifiable · happy + error paths |

### Step 3: Draft report — scorecard table + detailed findings + improvement suggestions + proposed title (if needed) + proposed content. Write to `/tmp/issue-analysis.md`.

**Report template:**

```markdown
## Issue Requirement Report

**Issue:** #<n> — <title>
**Analysis time:** <timestamp>

| Dimension | Rating | Notes |
|------|------|------|
| Title Clarity | 🟢/🟡/🔴 | <brief> |
| Description Sufficiency | 🟢/🟡/🔴 | <brief> |
| Acceptance Criteria Clarity | 🟢/🟡/🔴 | <brief> |

### Improvement Suggestions
1. <actionable>
2. ...

### Suggested Title
`<proposed title>`
```

### Step 4: Confirm with user before posting (side effect).

### Step 5: Post — `issue comment <n> --body-file /tmp/issue-analysis.md`.

### Step 6: Cleanup — `rm -f /tmp/issue-analysis.md`.

### Error Handling

| Error | Recovery |
|-------|----------|
| 404 | Stop. Issue not found. |
| Auth | Stop. `auth login`. |
| Comment API failure | Surface; do not retry. |
| Insufficient dimension info | Mark 🟡; don't speculate. |

## Responsibility

### ✅ In Scope

- Three-dimension Analysis
- Draft report
- Post as comment (after user confirm)

### ❌ Out of Scope

- Classifying / triaging → `/gitflow-issue-triage`
- Editing Issue body → `/gitflow-issue` (edit subcommand)
- Code review → `/gitflow-pr-review`
- Creating Issue → `/gitflow-issue-create`

### 🚫 Do Not

- ❌ Post without user confirmation
- ❌ Speculate beyond available info
- ❌ Mention implementation details (out of scope)
- ❌ Score without evidence

## Rationalization Excuses

| Excuse | Reality |
|--------|---------|
| "Just post the report" | Comment posting is a **side effect** — requires user confirmation. |
| "Guess what they meant" | Report only verifiable findings; mark gaps as 🟡. |
| "Rewrite the title for them" | Suggest — never edit without consent. |

## Red Flags

- 🚩 "Post the analysis directly" — Always confirm first.
- 🚩 "Score without reading" — Read the full description first.
- 🚩 "Suggest code changes" — Out of scope. Stick to requirement quality.

## Test Scenarios

### 1: Happy Path
- **Given** "review issue #42" — **When** title/description/scores drafted — **Then** user confirms → `issue comment 42 --body-file ...`, report posted.

### 2: Negative
- **Given** "create a new issue" — **Then** NOT loaded. → `/gitflow-issue-create`.

### 3: Boundary
- **Given** "analyze and then fix the code" — **Then** analyze only; code fixes not in scope → stop.

### 4: Error
- **Given** "issue view 9999" returns 404 — **Then** "Issue not found" — stop, no comment.

### 5: Boundary
- **Given** user says "skip confirmation" — **Then** refuse — posting is side effect.

## Success Criteria

- [ ] Three-dimension scorecard produced
- [ ] Comment only posted after user confirmation
- [ ] No fabricated findings
- [ ] Cleanup of temp file

## Common Mistakes

- ❌ **Posting without confirmation** — side effect requires consent.
- ❌ **Speculative scoring** — base on evidence; use 🟡 when unsure.

## See Also

- `/gitflow-issue-create` — create new Issues
- `/gitflow-issue-triage` — classify and tag Issues
- `/gitflow-issue` — CRUD reference
- `docs/superpowers/templates/skill-conventions.md` — skill conventions
