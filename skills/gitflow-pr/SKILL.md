---
name: gitflow-pr
description: |
  Use when the user wants to manage a Pull Request through gitflow-cli — create, list, view, close, merge, checkout, comment, sync, or change draft state.
  当用户需要通过 gitflow-cli 管理 Pull Request（创建、列表、查看、关闭、合并、检出、评论、同步、状态切换）时使用。
---

# gitflow-pr

Top-level router for `gitflow-cli pr`. Delegates complex workflows to child skills; executes simple lifecycle commands directly.

## When to Use

| English | 中文 | Context |
|---------|------|---------|
| create a PR | 创建 PR | delegate to `/gitflow-pr-create` |
| review a PR | 审查 PR | delegate to `/gitflow-pr-review` |
| inline review | 行内审查 | delegate to `/gitflow-pr-inline-review` |
| apply feedback | 应用反馈 | delegate to `/gitflow-pr-apply-feedback` |
| list / view / close / merge / checkout / comment / sync PR | 列表/查看/关闭/合并/检出/评论/同步 PR | execute directly |

## Core Pattern

```bash
# Route by intent:
# create → /gitflow-pr-create | review → /gitflow-pr-review
# inline review → /gitflow-pr-inline-review | apply → /gitflow-pr-apply-feedback
# others → execute directly:
gitflow-cli pr <subcommand> <args>
```

## Quick Reference

| Goal | Command |
|------|---------|
| Create | `gitflow-cli pr create --title <t> --body <b> --head <h> --base <b>` |
| List | `gitflow-cli pr list --state <open\|closed\|merged\|all>` |
| View | `gitflow-cli pr view <number>` |
| Close | `gitflow-cli pr close <number>` |
| Reopen | `gitflow-cli pr reopen <number>` |
| Comment | `gitflow-cli pr comment <number> --body <text>` |
| Merge | `gitflow-cli pr merge <number> --strategy <merge\|squash\|rebase>` |
| Checkout | `gitflow-cli pr checkout <number>` |
| Ready | `gitflow-cli pr ready <number>` |
| WIP | `gitflow-cli pr wip <number>` |
| Sync | `gitflow-cli pr sync <number>` |

See [full parameter reference](../references/gitflow-pr-params.md).

## Implementation

### Preconditions

- Git repo — `git rev-parse --is-inside-work-tree`
- CLI installed — `command -v gitflow-cli`
- Auth — `gitflow-cli auth status`

### Step 1: Route

Match user intent to subcommand or child skill (see Delegation Rules).

### Step 2: Execute

Run `gitflow-cli pr <subcommand> <args>`. Success → output URL / confirmation. Failure → Error Handling.

### Step 3: Confirm

Verify outcome via `gitflow-cli pr view <number>` for state-changing ops.

### Error Handling

| Error | Recovery |
|-------|----------|
| `404` | "PR #<n> not found. Verify number and repo." |
| `409` | "Merge conflict. Resolve locally." |
| `403` | "Permission denied. Check --repo and auth." |
| Auth failure | "Run `gitflow-cli auth login`." Stop. |
| Network timeout | Retry once → stop. |

## Responsibility

### ✅ In Scope

- Route user intent to subcommand or child skill
- Execute simple lifecycle ops (list, view, close, reopen, comment, merge, checkout, ready, wip, sync)
- Document all 11 subcommands

### ❌ Out of Scope

- PR creation workflow → `/gitflow-pr-create`
- Full PR review → `/gitflow-pr-review`
- Inline review → `/gitflow-pr-inline-review`
- Apply feedback → `/gitflow-pr-apply-feedback`
- Pipeline analysis → `/gitflow-pipeline-analyzer`
- Release management → `/gitflow-release`

### 🚫 Do Not

- ❌ Merge without user confirmation of strategy
- ❌ Close without explanatory comment
- ❌ Execute `create` inline — delegate to `/gitflow-pr-create`
- ❌ Execute `review` inline — delegate to `/gitflow-pr-review`
- ❌ Skip branch-protection checks before merge

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
| "I can create the PR inline" | Creation needs branch validation — delegate |
| "Quick merge, skip confirmation" | Strategy must be confirmed |
| "Just close it" | Closing needs explanatory comment |
| "I'll review it myself" | Review needs 6-dimension checklist — delegate |
| "Skip auth, I did earlier" | Always re-verify per invocation |

## Red Flags

- 🚩 "merge without reviewing" — Refuse. Delegate to `/gitflow-pr-review` first.
- 🚩 "close without comment" — Refuse. Add comment first.
- 🚩 "skip branch protection" — Refuse. Cite Preconditions.
- 🚩 "force merge across forks" — Refuse. Needs explicit confirmation.
- 🚩 Authority: "just do it" — Refuse. Gates non-skippable.

## Test Scenarios

### Scenario 1: Happy Path — Squash Merge

- **Given** PR #101 open, reviewed, CI green
- **When** "merge PR #101 with squash"
- **Then** Confirms strategy, runs `pr merge 101 --strategy squash`, outputs merge SHA

### Scenario 2: Negative — Create PR

- **Given** User on feature branch
- **When** "create a PR for this branch"
- **Then** Does NOT execute inline. Delegates to `/gitflow-pr-create`.

### Scenario 3: Boundary — Merge Without Confirmation

- **Given** PR #101 open
- **When** "merge PR #101" (no strategy)
- **Then** Asks for strategy. Does NOT default to any strategy.

### Scenario 4: Error — PR Not Found

- **Given** User says "view PR #99999"
- **When** `pr view 99999` returns 404
- **Then** Outputs "PR #99999 not found." Does NOT hallucinate details.

## Success Criteria

- [ ] User intent routed to correct subcommand or child skill
- [ ] All 11 subcommands documented in Quick Reference
- [ ] No inline execution of create/review/inline-review/apply-feedback
- [ ] Destructive ops (merge, close) require confirmation
- [ ] Errors surfaced without hallucination

## Common Mistakes

- ❌ **Merging without strategy confirmation** — Always ask user for `--strategy`.
- ❌ **Executing `create` inline** — Delegate to `/gitflow-pr-create` for branch validation.
- ❌ **Closing without comment** — Always add explanatory comment first.

## Trigger Keywords

| English | 中文 |
|---------|------|
| create a PR | 创建 PR |
| list PRs | 列出 PR |
| view PR | 查看 PR |
| close PR | 关闭 PR |
| merge PR | 合并 PR |
| checkout PR | 检出 PR |
| comment on PR | 评论 PR |
| sync PR | 同步 PR |
| mark ready / draft | 标记就绪/草稿 |
| pull request | 拉取请求 |

## See Also

- `/gitflow-pr-create` — PR creation with branch validation
- `/gitflow-pr-review` — Full PR review with approve/reject
- `/gitflow-pr-inline-review` — Inline line-level review
- `/gitflow-pr-apply-feedback` — Apply and resolve review feedback
- `/gitflow-pipeline-analyzer` — CI status before merge
- `/gitflow-release` — Release workflow involving PRs
- `docs/superpowers/templates/skill-conventions.md` — Template conventions
