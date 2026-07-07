---
name: gitflow-label-milestone
description: |
  Use when the user needs to create, list, edit, or delete repository labels or milestones via CLI.
  当用户需要通过 CLI 创建、列出、编辑或删除仓库标签或里程碑时使用。
---

# gitflow-label-milestone

## Overview

CRUD for labels and milestones. Read list freely; all mutations require explicit confirmation. Wraps `gitflow-cli label` and `gitflow-cli milestone`.

> **Split recommendation:** bundles two unrelated command families. Prefer `/gitflow-label` + `/gitflow-milestone` as independent skills — each gets its own description, keywords, and token budget.

## When to Use

| English | 中文 | Context |
|---------|------|---------|
| create / edit / delete label | 创建/编辑/删除标签 | label lifecycle |
| create / close milestone | 创建/关闭里程碑 | milestone lifecycle |
| label color / milestone due | 标签颜色/截止日期 | property change |
| assign label to issue | 给 issue 打标 | **do NOT fire** → `/gitflow-issue` |

## Core Pattern

```bash
command -v gitflow-cli && git rev-parse --git-dir
gitflow-cli label list          # verify before mutate
gitflow-cli label create --name <n> --color <hex> [--description <d>]
gitflow-cli label edit <n> [--name <n>] [--color <hex>] [--description <d>]
gitflow-cli label delete <n>
gitflow-cli milestone list
gitflow-cli milestone create --title <t> [--description <md>] [--due-on <iso>]
gitflow-cli milestone edit <n> [--title <t>] [--due-on <iso>]
gitflow-cli milestone close <n>
gitflow-cli milestone reopen <n>
```

## Quick Reference

| Goal | Command |
|------|---------|
| List labels | `gitflow-cli label list` |
| Create label | `gitflow-cli label create --name <n> --color <hex>` |
| Edit label | `gitflow-cli label edit <n> [--name <n>] [--color <hex>]` |
| Delete label | `gitflow-cli label delete <n>` |
| List milestones | `gitflow-cli milestone list` |
| Create milestone | `gitflow-cli milestone create --title <t>` |
| Close milestone | `gitflow-cli milestone close <n>` |

## Implementation

### Preconditions

- `command -v gitflow-cli`
- `git rev-parse --git-dir`
- `gitflow-cli auth status`

### Step 1 — Read

`gitflow-cli {label,milestone} list`. Confirm target exists before edit/delete.

### Step 2 — Confirm

Present intended change; await explicit yes. For delete: warn if label in use.

### Step 3 — Execute & Verify

Run mutation; re-run list; emit URL/identifier. Failure → Error Handling.

### Error Handling

| Error | Recovery |
|-------|----------|
| Already exists | Refuse; suggest edit or rename |
| Not found | Stop; show valid names/numbers |
| Invalid color | Refuse; require 6-char hex, no `#` |
| API 403/401 | `auth login --platform <p>`; retry once |
| Delete in-use label | Refuse; redirect `/gitflow-issue` |

## Responsibility

### ✅ In Scope

- Label CRUD (create, list, edit, delete)
- Milestone CRUD (create, list, edit, close, reopen)
- Confirm-before-mutation guard

### ❌ Out of Scope

- Assign labels to issues → `/gitflow-issue`
- Auto-triage / bulk labeling → `/gitflow-issue-triage`
- Bulk delete → manual per-resource
- Release association → `/gitflow-release`

### 🚫 Do Not

- ❌ Delete without confirming unused
- ❌ Bulk-create/delete without per-item confirm
- ❌ Assign labels to issues
- ❌ Set past `due-on` without warning
- ❌ Skip confirmation on mutation

## Rationalization Excuses

| Excuse | Reality |
|--------|---------|
| "Delete all labels" | Bulk forbidden; confirm each |
| "Label probably unused" | Must verify; refuse if unsure |
| "Add label while triaging" | Out of scope; `/gitflow-issue-triage` |
| "Color close enough" | Require valid 6-char hex |

## Red Flags

- 🚩 "delete all labels" — refuse; bulk prohibited
- 🚩 "skip confirmation" — refuse; explicit yes required
- 🚩 "add label to this issue" — redirect `/gitflow-issue`
- 🚩 "just pick a color" — require explicit hex
- 🚩 "close milestone and archive issues" — out of scope

## Trigger Keywords

| English | 中文 |
|---------|------|
| create label | 创建标签 |
| delete label | 删除标签 |
| edit label | 编辑标签 |
| create milestone | 创建里程碑 |
| close milestone | 关闭里程碑 |
| reopen milestone | 重开里程碑 |
| label color | 标签颜色 |
| milestone due date | 里程碑截止日期 |

## Test Scenarios

### S1 Happy — Create Label

- **Given** authed, in repo, `bug` absent
- **When** "create bug label color d73a4a"
- **Then** `label create` runs, `list` confirms, URL emitted

### S2 Negative

- **Given** "add bug label to issue #42"
- **When** assign-label intent
- **Then** NOT loaded; redirect `/gitflow-issue`

### S3 Boundary

- **Given** "delete all old labels"
- **When** bulk mutation request
- **Then** refuses; cites `🚫`; per-item only

### S4 Error

- **Given** `bug` exists
- **When** `label create --name bug`
- **Then** conflict; stop; suggest edit/rename; no improvisation

## Success Criteria

- [ ] Resource mutated with URL/identifier returned
- [ ] No mutation without confirmation
- [ ] No out-of-scope action
- [ ] Error Handling recovery verbatim

## Common Mistakes

- ❌ **Bulk delete** — each requires explicit yes
- ❌ **Assigning labels** — out of scope; `/gitflow-issue`
- ❌ **Invalid hex** — require 6-char; reject `#`
- ❌ **Skip list-before-delete** — always verify first

## See Also

- `/gitflow-issue` — assign labels and milestones
- `/gitflow-issue-triage` — auto-label during triage
- `/gitflow-release` — associate milestones
- `docs/superpowers/templates/skill-conventions.md` — template conventions
