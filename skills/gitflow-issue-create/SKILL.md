---
name: gitflow-issue-create
description: |
  Use when the user wants to create a new issue through gitflow-cli — collecting title, description, labels, and assignees interactively.
  当用户希望通过 gitflow-cli 交互式地创建新 issue（收集标题、描述、标签、指派人）时使用。
---

# gitflow-issue-create

Interactive workflow: enforces conventional-commit title prefix, collects Markdown body with acceptance criteria, optionally attaches labels/assignees, confirms before invoking `gitflow-cli issue create`, returns URL. Does not edit, close, or triage.

## When to Use

| English | 中文 | Context |
|---------|------|---------|
| create an issue | 创建 issue | user wants a new issue |
| file a bug / feature | 提交 bug / 功能需求 | new work item |
| open an issue | 新建 issue | manual creation |
| close / edit issue | 关闭 / 编辑 | **do NOT fire** → `/gitflow-issue` |
| triage backlog | 分流待办 | **do NOT fire** → `/gitflow-issue-triage` |

## Core Pattern

```bash
command -v gitflow-cli && gitflow-cli auth status
# collect title + body + labels/assignees
gitflow-cli issue create --title "<t>" --body "<b>" --label <l> --assignee <u>
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

`command -v gitflow-cli` succeeds; `gitflow-cli auth status` succeeds — else `/gitflow-auth`.

### Step 1: Collect Title

Require conventional-commit prefix: `feat:`, `fix:`, `docs:`, `refactor:`, `chore:`, `test:`, `perf:`. Reject vague titles.

### Step 2: Collect Description

Markdown: `## 背景`, `## 目标`, `## 验收标准` (`- [ ]`), `## 备注`. Confirm before proceeding.

### Step 3: Labels / Assignees (Optional)

Attach labels (`bug`, `enhancement`, `docs`, `high-priority`, `good-first-issue`) or assignees only on request.

### Step 4: Confirm + Invoke

Show assembled command. On confirmation run `gitflow-cli issue create`. Success → URL. Failure → Error Handling.

### Error Handling

| Error | Recovery |
|-------|----------|
| Auth failure | Stop. → `/gitflow-auth`. |
| Title rejected | Re-prompt. |
| API 422 | Surface; suggest fix; stop. |
| Timeout | Surface; no retry. |

## Responsibility

### ✅ In Scope

- Collect title / body / labels / assignees
- Enforce conventional-commit prefix
- Confirm before invoking
- Invoke `gitflow-cli issue create`; return URL

### ❌ Out of Scope

- Edit / close / reopen → `/gitflow-issue`
- Triage → `/gitflow-issue-triage`
- Auto-report → `/gitflow-autoreport-bug`
- Review → `/gitflow-issue-review`

### 🚫 Do Not

- ❌ Invoke without confirmation
- ❌ Skip prefix enforcement
- ❌ Edit existing issues
- ❌ Auto-add labels not requested
- ❌ Retry 5xx without surfacing

## Rationalization Excuses

| Excuse | Reality |
|--------|---------|
| "Skip the prefix — user knows" | Prefix enables auto-categorization |
| "Just create it, no confirm" | Mutation requires confirmation |
| "Add `bug` — looks like a bug" | Labels user-driven; never infer |
| "Auth was fine earlier" | Preconditions run every invocation |

## Red Flags

- 🚩 "Skip confirmation" — Refuse. Required.
- 🚩 "Add a label while at it" — Refuse. User-driven.
- 🚩 "Assign to me" without explicit ask — Refuse.
- 🚩 CLI fails → improvise — Follow Error Handling only.

## Test Scenarios

### Scenario 1: Happy Path

- **Given** Auth valid, title `fix(auth): token refresh loop`, body with criteria, label `bug`
- **When** "create an issue"
- **Then** Confirmed; `issue create` invoked; URL returned.

### Scenario 2: Negative — "close issue #42"

- **When** "close issue #42"
- **Then** Do NOT load. Redirect to `/gitflow-issue`.

### Scenario 3: Boundary — assignee not requested

- **When** "create issue for the login bug"
- **Then** Asks before assigning; refuses to infer.

### Scenario 4: Error — `issue create` returns 422

- **When** `issue create` invoked without prefix
- **Then** Error surfaced; suggest fix; stop.

## Success Criteria

- [ ] Issue URL returned
- [ ] Title has conventional-commit prefix
- [ ] Command confirmed before invocation
- [ ] No labels/assignees added without request

## Common Mistakes

- ❌ **Skipping prefix enforcement** — Always require `type(scope):`.
- ❌ **Inferring labels** — User-driven; never auto-add.
- ❌ **Invoking without confirmation** — Mutation requires approval.

## Trigger Keywords

| English | 中文 |
|---------|------|
| create an issue | 创建 issue |
| file a bug | 提交 bug |
| open an issue | 新建 issue |
| new feature request | 新功能需求 |
| report a problem | 报告问题 |

## See Also

- `/gitflow-issue` — close, reopen, edit, comment
- `/gitflow-issue-review` — analyze completeness
- `/gitflow-issue-triage` — batch classify
- `/gitflow-autoreport-bug` — auto-report from crashes
- `docs/superpowers/templates/skill-conventions.md` — conventions
