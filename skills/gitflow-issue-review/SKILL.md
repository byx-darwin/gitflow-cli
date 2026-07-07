---
name: gitflow-issue-review
description: |
  Use when the user asks to review an issue's requirements completeness, evaluate issue quality, or analyze whether an issue is ready for development.
  当用户审查 issue 需求完整性、评估 issue 质量、或分析 issue 是否可开发时使用。
---

# gitflow-issue-review

Analyzes issue completeness (title / description / acceptance criteria) and posts a structured report as an issue comment. Read + comment only — never mutates metadata, never writes code.

## When to Use

| English | 中文 | Context |
|---------|------|---------|
| review issue | 审查 issue | user provides issue number |
| check acceptance criteria | 检查验收标准 | improve readiness |
| fix while reviewing | 顺便修复 | **do NOT fire** → manual edit |

## Quick Reference

| Goal | Command |
|------|---------|
| Fetch | `gitflow-cli issue view <number>` |
| Post | `gitflow-cli issue comment <number> --body-file <path>` |

## Implementation

### Preconditions

- `command -v gitflow-cli` succeeds
- `git rev-parse --git-dir` succeeds

### Step 1: Fetch

`gitflow-cli issue view <number>`. On failure follow Error Handling, stop. Capture title, body, labels, state, comments.

### Step 2: Three-Dimension Analysis

Title 🟢/🟡/🔴 — conventional-commit prefix + scope. Description — background + goal + constraints. Acceptance — checkbox + verifiable + happy/error paths.

### Step 3: Report + Idempotency

Produce scorecard, analysis, suggestions. If existing comment contains `## Issue 需求分析报告` ask before posting; default: skip duplicate.

### Step 4: Post Comment

```bash
cat > /tmp/issue-analysis.md << 'EOF'
<report>
EOF
gitflow-cli issue comment <number> --body-file /tmp/issue-analysis.md
rm -f /tmp/issue-analysis.md
```

Success → URL. Failure → Error Handling.

### Error Handling

| Error | Recovery |
|-------|----------|
| `issue view` 404/401/network | Output error; stop; no comment |
| `issue comment` 403/timeout | Keep report in conversation; advise retry; stop |
| Issue closed | Warn; await confirmation |
| Duplicate detected | Default skip |

## Responsibility

### ✅ In Scope

- Fetch issue, three-dimension evaluation, report generation, comment posting, idempotency check

### ❌ Out of Scope

- Edit metadata → manual platform edit
- Close/reopen → `/gitflow-issue`
- Batch review → `/gitflow-issue-triage`
- Implement suggestions → analyze only

### 🚫 Do Not

- ❌ Edit issue field
- ❌ Skip three-dimension analysis ever
- ❌ Post duplicate — ask first
- ❌ Apply strict criteria to question/discussion

## Rationalization Excuses

| Excuse | Reality |
|--------|---------|
| "Quick look, skip framework" | Three dimensions = minimum; non-negotiable |
| "Fix the title while here" | Out of scope; manual edit |
| "Urgent, skip comment" | Comment = verifiable artifact; non-negotiable |

## Red Flags

- 🚩 "Fix the title for me" — Out of Scope; refuse; stop
- 🚩 "Simple issue, just skim" — Three dimensions required
- 🚩 Tool failure → follow table

## Test Scenarios

### Scenario 1: Happy Path

- **Given** Issue #42: good title, partial description, no criteria
- **When** "review issue #42"
- **Then** 🟢/🟡/🔴 scored; posted; URL returned

### Scenario 2: Negative

- **Given** User wants batch review
- **When** "review all open issues"
- **Then** Do NOT load; redirect to `/gitflow-issue-triage`

### Scenario 3: Boundary

- **Given** Issue #15 has vague title
- **When** "review #15 and fix the title"
- **Then** Analysis completes; title fix refused; Out of Scope; stop

### Scenario 4: Error

- **Given** `issue view 99999` returns 404
- **When** "review issue #99999"
- **Then** Verbatim error; no comment; no fabricated report

## Success Criteria

- [ ] `issue view` returns 0; three dimensions graded
- [ ] Report posted; URL returned
- [ ] No metadata mutated
- [ ] Idempotency check ran
- [ ] Temp file cleaned up

## Common Mistakes

- ❌ **Skipping 3 dimensions** — shorten prose not dimensions
- ❌ **Editing metadata** — read + comment only
- ❌ **Batch review** — single issue; batch via triage
- ❌ **Fabricating post-fetch** — follow table; stop

## Trigger Keywords

| English | 中文 |
|---------|------|
| review issue | 审查 issue |
| analyze issue quality | 评估 issue 质量 |
| check acceptance criteria | 检查验收标准 |
| improve issue description | 改进 issue 描述 |
| issue requirements analysis | issue 需求分析 |
| is issue ready for dev | issue 是否可开发 |

## See Also

- `/gitflow-issue` — close / reopen / label / comment commands
- `/gitflow-issue-create` — creates issues from templates
- `/gitflow-issue-triage` — batch classify + prioritize
- `docs/superpowers/templates/skill-conventions.md` — skill conventions
