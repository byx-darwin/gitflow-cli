---
name: gf-issue-review
description: |
  Use when the user wants to analyze an Issue's requirement completeness (title clarity, description sufficiency, acceptance criteria, slice direction) and post findings as an Issue comment.
  当用户希望分析 Issue 需求完整性（标题清晰度、描述充分度、验收标准、切片方向）并回写评论时使用。
---

# gf-issue-review

Four-dimensional Issue requirement review — title clarity / description sufficiency / acceptance criteria / slice direction — emits a structured analysis report, then posts it as an Issue comment. Does not edit the Issue itself.

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
| review the requirement | 审查需求质量 | check description completeness |
| is this issue clear enough | 这个 Issue 描述够吗 | before triage/triage |
| improve issue description | 改进 Issue 描述 | before development |
| triage this issue | 对 Issue 进行分类 | **NOT** → `/gf-issue-triage` |

## When NOT to Use

| Scenario | Why Not | Use Instead |
|----------|---------|-------------|
| Batch classifying Issues by type/priority | This skill analyzes one Issue's requirement depth, not bulk classification | `/gf-issue-triage` for batch type+priority labeling |
| Creating a new Issue | This skill analyzes existing Issues only | `/gf-issue-create` for interactive Issue creation |
| Editing Issue content | This skill posts analysis as comments but never edits the Issue itself | `/gf-issue` for edit operations |
| Reviewing PR code | This skill reviews Issue requirements, not code diffs | `/gf-pr-review` for PR code review |
| Labeling Issues with statistics | This skill scores requirement quality, not label distribution | `/gf-label-stats` for label analytics |

## Core Pattern

```bash
gf issue view <n>
# analyze 4 dimensions → write /tmp/issue-analysis.md
gf issue comment <n> --body-file /tmp/issue-analysis.md
rm -f /tmp/issue-analysis.md
```

## Quick Reference

| Goal | Command |
|------|---------|
| Fetch Issue | `gf issue view <n>` |
| Post comment | `gf issue comment <n> --body-file <path>` |

**Four dimensions:** Title clarity · Description sufficiency · Acceptance criteria · Slice direction

## Implementation

### Preconditions

- Issue `<n>` exists — `issue view <n>`
- `gf` authenticated
- Write access to Issue comments

### Step 1: Fetch — `issue view <n>`. Record title, body, labels, links, comments.

### Optional semantic precheck (read-only)

If `gf issue precheck` is available, prepare a small JSON file containing only
`title`, reviewed/redacted `body`, relevant `labels`, optional milestone title,
and at most three selected, redacted `comments`. Do not pass the raw Issue
response, URLs, credentials, author metadata, or unrelated comments. Then run:

```bash
gf issue precheck --input /tmp/issue-precheck.json --live --output json
```

`--live` calls Jev only when `gf` has the `gitflow-jev` feature and
`GF_DECISION_PROVIDER=jev` plus a TypeSafe key from `TYPESAFE_API_KEY` or the
macOS `gitflow-cli-typesafe` Keychain item are configured. Without
them, the result is `unavailable`; continue directly to Step 2. For offline
replay, use `--response <saved-typed-response.json>` instead of `--live`.

The precheck provides five advisory scores (title, context, goal, acceptance,
slice), four probability signals (missing acceptance, untestable acceptance,
mixed goals, hidden dependency), a main gap, field-level evidence sources,
and up to four ranked clarifying questions. Treat all model results as
hypotheses. Verify the cited
Issue fields and each question against the full Issue before including it in
the report. `needs_review` means model confidence is low; use the existing
four-dimension analysis without relying on that score. The precheck never
comments, edits, labels, or closes an Issue.

### Step 2: Score each dimension 🟢/🟡/🔴

| Dimension | Checks |
|-----------|--------|
| Title | conventional prefix · scope · unambiguous · length |
| Description | context · goal · constraints · references |
| Acceptance | `- [ ]` format · verifiable · happy + error paths · **each line states the observation that would prove it false; a criterion already true on the base commit is flagged 🔴 (constrains nothing)** |
| Slice Direction | ticket covers one end-to-end narrow path, not a single layer (e.g. "data layer only" is a 🔴 layer-only slice); title/body too vague to tell → 🟡, do not guess the layers touched |

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
| Slice Direction | 🟢/🟡/🔴 | <brief> |

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

- Four-dimension Analysis
- Draft report
- Post as comment (after user confirm)

### ❌ Out of Scope

- Classifying / triaging → `/gf-issue-triage`
- Editing Issue body → `/gf-issue` (edit subcommand)
- Code review → `/gf-pr-review`
- Creating Issue → `/gf-issue-create`

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
- **Given** "create a new issue" — **Then** NOT loaded. → `/gf-issue-create`.

### 3: Boundary
- **Given** "analyze and then fix the code" — **Then** analyze only; code fixes not in scope → stop.

### 4: Error
- **Given** "issue view 9999" returns 404 — **Then** "Issue not found" — stop, no comment.

### 5: Boundary
- **Given** user says "skip confirmation" — **Then** refuse — posting is side effect.

## Success Criteria

- [ ] Four-dimension scorecard produced, including Slice Direction
- [ ] Acceptance criteria already true on the base commit are flagged, not silently accepted
- [ ] Comment only posted after user confirmation
- [ ] No fabricated findings
- [ ] Cleanup of temp file

## Common Mistakes

- ❌ **Posting without confirmation** — side effect requires consent.
- ❌ **Speculative scoring** — base on evidence; use 🟡 when unsure.

## See Also

- `/gf-issue-create` — create new Issues
- `/gf-issue-triage` — classify and tag Issues
- `/gf-issue-decompose` — batch decomposition into dependency-ordered Issues
- `/gf-issue` — CRUD reference
- `docs/superpowers/templates/skill-conventions.md` — skill conventions
