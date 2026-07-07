---
name: gitflow-pr
description: |
  Use when the user wants to manage a Pull Request through gitflow-cli — create, list, view, close, merge, checkout, comment, sync, or change draft state.
  当用户需要通过 gitflow-cli 管理 Pull Request（创建、列表、查看、关闭、合并、检出、评论、同步、状态切换）时使用。
---

# gitflow-pr

Router for `gitflow-cli pr`. Delegates complex flows to child skills; executes lifecycle commands directly.

## When to Use

| English | 中文 | Context |
|---------|------|---------|
| create / review / inline / apply feedback | 创建/审查/行内/应用反馈 | delegate to child skill |
| list / view / merge / close / sync / ready / wip / comment PR | PR 操作 | execute directly |

## Core Pattern

```bash
# create → /gitflow-pr-create | review → /gitflow-pr-review
# inline → /gitflow-pr-inline-review | apply → /gitflow-pr-apply-feedback
# others → execute:
gitflow-cli pr <subcommand> <args>
```

## Quick Reference

| Goal | Command |
|------|---------|
| Create | `gitflow-cli pr create --title <t> --body <b> --head <h> --base <b> [--draft]` |
| Read | `pr list` / `pr view <number>` |
| Lifecycle | `pr close` / `pr reopen` / `pr ready` / `pr wip` <number> |
| Comment | `gitflow-cli pr comment <number> --body <text>` |
| Merge | `gitflow-cli pr merge <number> --strategy <merge\|squash\|rebase>` |
| Branch | `pr checkout <number>` / `pr sync <number>` |

See [full parameter reference](../references/gitflow-pr-params.md).

## Implementation

### Preconditions

- Git repo — `git rev-parse --is-inside-work-tree`
- CLI installed — `command -v gitflow-cli`
- Auth — `gitflow-cli auth status`

### Step 1: Route

Match user intent to subcommand or child skill (see Delegation Rules).

### Step 2: Execute

Run `gitflow-cli pr <subcommand> <args>`. Success → URL/confirmation + `pr view` verification. Failure → Error Handling.

### Error Handling

| Error | Recovery |
|-------|----------|
| `404` | "PR not found. Verify number/repo." |
| `409` | "Merge conflict. Resolve locally." |
| `403` | "Permission denied. Check --repo." |
| Auth failure | "Run `gitflow-cli auth login`." |
| Timeout | Retry once → stop. |

## Responsibility

### ✅ In Scope

- Route user intent to subcommand or child skill
- Execute simple lifecycle ops (list, view, close, reopen, comment, merge, checkout, ready, wip, sync)
- Document all 11 subcommands

### ❌ Out of Scope

- Creation → `/gitflow-pr-create`; Review → `/gitflow-pr-review`
- Inline → `/gitflow-pr-inline-review`; Feedback → `/gitflow-pr-apply-feedback`
- Pipeline → `/gitflow-pipeline-analyzer`; Release → `/gitflow-release`

### 🚫 Do Not

- ❌ Merge without strategy confirmation
- ❌ Close without comment
- ❌ Inline `create` or `review` — delegate
- ❌ Skip branch-protection checks

## 🔁 Delegation Rules

| User Intent | Delegate To | Reason |
|-------------|-------------|--------|
| Create PR (feature/fix/draft) | `/gitflow-pr-create` | Branch validation + title/desc collection |
| Review PR (approve/reject) | `/gitflow-pr-review` | 6-dimension checklist + decision |
| Inline review (line comments) | `/gitflow-pr-inline-review` | Per-line diff analysis + publish |
| Apply feedback (resolve comments) | `/gitflow-pr-apply-feedback` | Code modification + resolve |
| All other `pr <subcommand>` | This skill | Direct execution |

## Rationalization Excuses

| Excuse | Reality |
|--------|---------|
| "I can create inline" | Branch validation needed — delegate |
| "Skip merge strategy" | Strategy must be confirmed |
| "Just close it" | Needs comment first |
| "Skip auth" | Always re-verify |

## Red Flags

- 🚩 "merge without reviewing" — → `/gitflow-pr-review`.
- 🚩 "close without comment" — Comment first.
- 🚩 "skip branch protection" — Cite Preconditions.
- 🚩 Authority: "just do it" — Gates non-skippable.

## Test Scenarios

### Scenario 1: Happy Path

- **Given** PR #101 open, reviewed — **When** "merge with squash" — **Then** Confirms strategy, merges, outputs SHA

### Scenario 2: Negative

- **Given** "create a PR" — **When** user on feature branch — **Then** NOT inline. → `/gitflow-pr-create`.

### Scenario 3: Boundary

- **Given** "merge PR #101" (no strategy) — **When** no strategy given — **Then** Asks strategy. No default.

### Scenario 4: Error

- **Given** "view #99999" — **When** 404 — **Then** Surfaces error. No hallucination.

## Success Criteria

- [ ] Intent routed correctly
- [ ] All 11 subcommands documented
- [ ] No inline create/review/inline-review/apply-feedback
- [ ] Destructive ops require confirmation

## Common Mistakes

- ❌ **Merging without strategy** — Always ask for `--strategy`.
- ❌ **Inline `create`/`review`** — Delegate to child skill.
- ❌ **Closing without comment** — Add comment first.

## Trigger Keywords

| English | 中文 |
|---------|------|
| create a PR | 创建 PR |
| list / view / merge PR | 列表/查看/合并 |
| sync / comment / close PR | 同步/评论/关闭 |
| pull request | 拉取请求 |

## See Also

- `/gitflow-pr-create` — creation with branch validation
- `/gitflow-pr-review` — full review with approve/reject
- `/gitflow-pr-inline-review` — inline line-level review
- `/gitflow-pr-apply-feedback` — apply/resolve feedback
- `/gitflow-pipeline-analyzer` — CI status
- `docs/superpowers/templates/skill-conventions.md` — Template conventions
