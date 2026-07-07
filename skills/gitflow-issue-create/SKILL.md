---
name: gitflow-issue-create
description: |
  Use when the user wants to create a new issue through gitflow-cli — collecting title, description, labels, and assignees interactively.
  当用户希望通过 gitflow-cli 交互式地创建新 issue（收集标题、描述、标签、指派人）时使用。
---

# gitflow-issue-create

Guides the user through structured issue creation — collects title (conventional-commit prefix), description (Markdown template), labels, and assignees, then invokes `gitflow-cli issue create` and returns the new issue URL. Does not edit, close, or triage existing issues.

## When to Use

| English | 中文 | Context |
|---------|------|---------|
| create an issue | 创建 issue | user wants a new issue |
| file a bug / feature | 提交 bug / 功能需求 | new work item |
| open an issue | 新建 issue | manual creation |
| close / edit issue | 关闭 / 编辑 issue | **do NOT fire** → `/gitflow-issue` |
| triage backlog | 分流待办 | **do NOT fire** → `/gitflow-issue-triage` |

## Core Pattern

```bash
# 1. Preconditions
command -v gitflow-cli && gitflow-cli auth status
# 2. Collect title (conventional-commit prefix) + body (Markdown template)
# 3. Collect labels / assignees (optional)
# 4. Confirm command
gitflow-cli issue create --title "<title>" --body "<body>" --label <l> --assignee <u>
# 5. Output issue URL
```

## Quick Reference

| Goal | Command |
|------|---------|
| Create | `gitflow-cli issue create --title "<t>" --body "<b>" --label <l> --assignee <u>` |
| Add label | append `--label <name>` (repeatable) |
| Add assignee | append `--assignee <login>` (repeatable) |
| Minimal | `gitflow-cli issue create --title "<t>"` |

## Implementation

### Preconditions

- `command -v gitflow-cli` succeeds
- `gitflow-cli auth status` succeeds — else run `/gitflow-auth`

### Step 1: Collect Title

Prompt for a conventional-commit title: `feat:`, `fix:`, `docs:`, `refactor:`, `chore:`, `test:`, `perf:`. Reject vague titles; require scope for non-chore types.

### Step 2: Collect Description

Prompt for Markdown body with sections: `## 背景`, `## 目标`, `## 验收标准` (checkbox `- [ ]`), `## 备注`. Confirm before proceeding.

### Step 3: Collect Labels (Optional)

Ask whether to attach labels. Common: `bug`, `enhancement`, `documentation`, `high-priority`, `good-first-issue`. Skip if user declines.

### Step 4: Collect Assignees (Optional)

Ask for assignee login names. Skip if user declines.

### Step 5: Confirm + Invoke

Show the assembled command. On confirmation, run `gitflow-cli issue create`. Success → output issue URL. Failure → Error Handling.

### Error Handling

| Error | Recovery |
|-------|----------|
| Auth failure | Stop. Direct to `/gitflow-auth`. |
| Title rejected | Re-prompt; require conventional prefix. |
| API 422 (validation) | Surface error; suggest field fix; stop. |
| Network timeout | Surface error; no retry alone. |

## Responsibility

### ✅ In Scope

- Collect title / body / labels / assignees interactively
- Enforce conventional-commit title prefix
- Confirm command before invoking
- Invoke `gitflow-cli issue create`; return URL

### ❌ Out of Scope

- Edit / close / reopen → `/gitflow-issue`
- Triage / prioritize → `/gitflow-issue-triage`
- Auto-report from crash → `/gitflow-autoreport-bug`
- Review existing issue → `/gitflow-issue-review`

### 🚫 Do Not

- ❌ Invoke without user confirmation
- ❌ Skip conventional-commit prefix enforcement
- ❌ Edit existing issues
- ❌ Auto-add labels not requested by user
- ❌ Retry on 5xx without surfacing to user

## Rationalization Excuses

| Excuse | Reality |
|--------|---------|
| "Skip the prefix — user knows what they want" | Prefix enables auto-categorization; non-negotiable |
| "Just create it, no need to confirm" | Mutation requires explicit confirmation |
| "Add `bug` label — looks like a bug" | Labels are user-driven; never infer |
| "Auth was fine earlier" | Preconditions run every invocation |

## Red Flags

- 🚩 "Skip the confirmation" — Refuse. Confirmation required.
- 🚩 "Add a label while you're at it" — Refuse. Labels are user-driven.
- 🚩 "Create and assign to me" without explicit ask — Refuse. Assign only if requested.
- 🚩 CLI fails → improvise — Follow Error Handling only.

## Test Scenarios

### Scenario 1: Happy Path

- **Given** Auth valid, user provides title `fix(auth): token refresh loop`, body with acceptance criteria, label `bug`
- **When** "create an issue"
- **Then** Command confirmed; `issue create` invoked; issue URL returned.

### Scenario 2: Negative — "close issue #42"

- **Given** User wants to close an existing issue
- **When** "close issue #42"
- **Then** Do NOT load. Redirect to `/gitflow-issue`.

### Scenario 3: Boundary — "create issue and assign it to alice"

- **Given** User did not explicitly request an assignee
- **When** "create issue for the login bug"
- **Then** Skill asks before assigning; refuses to infer assignee.

### Scenario 4: Error — `issue create` returns 422

- **Given** Title missing conventional prefix rejected by API
- **When** `issue create` invoked
- **Then** Error surfaced; suggest prefix fix; stop.

## Success Criteria

- [ ] Issue URL returned to user
- [ ] Title has conventional-commit prefix
- [ ] Command confirmed before invocation
- [ ] No labels/assignees added without explicit user request
- [ ] No out-of-scope mutation performed

## Common Mistakes

- ❌ **Skipping prefix enforcement** — Always require `type(scope):` prefix.
- ❌ **Inferring labels** — Labels are user-driven; never auto-add.
- ❌ **Invoking without confirmation** — Mutation requires explicit approval.

## Trigger Keywords

| English | 中文 |
|---------|------|
| create an issue | 创建 issue |
| file a bug | 提交 bug |
| open an issue | 新建 issue |
| new feature request | 新功能需求 |
| report a problem | 报告问题 |

## See Also

- `/gitflow-issue` — close, reopen, edit, comment on issues
- `/gitflow-issue-review` — analyze issue completeness
- `/gitflow-issue-triage` — batch classify and prioritize
- `/gitflow-autoreport-bug` — auto-report from crash artifacts
- `docs/superpowers/templates/skill-conventions.md` — skill conventions
