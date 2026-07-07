---
name: gitflow-issue-create
description: |
  Use when the user wants to create a new Issue through gitflow-cli — interactive title, description, label collection.
  当用户希望通过 gitflow-cli 创建 Issue（交互式标题、描述、标签收集）时使用。
---

# gitflow-issue-create

Interactive workflow that collects title, description, optional labels/assignee, then invokes `gitflow-cli issue create` and returns the new Issue URL.

## When to Use

| English | 中文 | Context |
|---------|------|---------|
| create an issue | 创建 Issue | bug / feature report |
| open an issue | 打开 Issue | new work item |
| file a bug | 上报缺陷 | corresponds to `--label bug` |
| new feature issue | 功能 Issue | corresponds to `feat:` prefix |

## Core Pattern

```bash
gitflow-cli auth status
gitflow-cli issue create --title "<prefix>(scope): summary" --body "<md>" [--label <l>...]
```

## Quick Reference

| Goal | Command |
|------|---------|
| Create | `gitflow-cli issue create --title "<title>" --body "<md>" [--label <l>] [--assignee <u>]` |
| Add label | append `--label <label>` (repeatable) |

**Title prefixes:** `feat:` `fix:` `docs:` `refactor:` `chore:` `test:` `perf:`

## Implementation

### Preconditions

- `gitflow-cli` authenticated — `auth status`
- Title non-empty; starts with conventional-commit prefix

### Step 1: Title — conventional prefix + scope. Example: `fix(auth): login redirect loops on expired token`.

### Step 2: Body — Markdown template.

```markdown
## 背景 / Context

## 目标 / Goal

## 验收标准 / Acceptance Criteria
- [ ] …
- [ ] …
```

### Step 3: Labels — optional. Common: `bug`, `enhancement`, `documentation`, `high-priority`, `good-first-issue`. Omit flag if none.

### Step 4: Assignee — optional. Provide login. Skip if absent.

### Step 5: Invoke. Confirm. `gitflow-cli issue create ...`. Parse output, extract + return Issue URL.

### Error Handling

| Error | Recovery |
|-------|----------|
| Auth failure | Stop. `auth login`. |
| Title empty / no prefix | Stop. Prompt again. |
| API error (4xx) | Surface message; do not retry. |
| Network timeout | Surface; do not retry. |

## Responsibility

### ✅ In Scope

- Collect title / body / labels / assignee
- Confirm then invoke CLI
- Return Issue URL

### ❌ Out of Scope

- Analysis → `/gitflow-issue-review`
- Classification → `/gitflow-issue-triage`
- Comments → `/gitflow-issue` (comment subcommand)
- Bulk label ops → `/gitflow-label-stats` + `/gitflow-issue-triage`

### 🚫 Do Not

- ❌ Create without title prefix verification
- ❌ Skip confirmation before invoking CLI
- ❌ Invent labels — only add if user specifies
- ❌ Auto-assign without user input

## Rationalization Excuses

| Excuse | Reality |
|--------|---------|
| "Prefix doesn't matter" | Prefix enables automated triage and routing. |
| "Skip confirmation, trust me" | CLI call is a side effect — always confirm. |
| "Invent a label for them" | Labels are user-defined; never infer. |

## Red Flags

- 🚩 "Just create it" — Collect all fields first, confirm, then invoke.
- 🚩 "Any title is fine" — Enforce conventional prefix.
- 🚩 "Auto-assign someone" — Only if user specifies.

## Test Scenarios

### 1: Happy Path
- **Given** "create a bug issue: `fix(auth): redirect loop`, body filled" — **When** user confirms — **Then** `issue create ... --label bug`, returns Issue URL.

### 2: Negative
- **Given** "review issue #42" — **Then** NOT loaded. → `/gitflow-issue-review`.

### 3: Boundary
- **Given** "create issue and also triage all open issues" — **Then** create only; redirect triage → `/gitflow-issue-triage`.

### 4: Error
- **Given** "create issue" but `auth status` fails — **Then** stop, prompt `auth login`, do not call create.

### 5: Boundary
- **Given** title without conventional prefix — **Then** stop, prompt user to add prefix.

## Success Criteria

- [ ] Issue URL returned
- [ ] Title has conventional prefix
- [ ] CLI invoked only after confirmation
- [ ] Out-of-scope requests redirected

## Common Mistakes

- ❌ **Creating without prefix** — Prompt user first.
- ❌ **Adding inferred labels** — Only user-specified labels.

## See Also

- `/gitflow-issue` — Issue CRUD operations
- `/gitflow-issue-review` — Issue requirement analysis
- `/gitflow-issue-triage` — Issue classification
- `docs/superpowers/templates/skill-conventions.md` — skill conventions
