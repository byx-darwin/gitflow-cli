---
name: gf-review
description: |
  Use when the user wants to submit a formal code review decision
  (approve, request changes, or comment) on a PR through gf.
  当用户希望通过 gf 提交正式 PR 审查结论时使用。
---

# gf-review

Submits review verdicts via `gf review`. Read-only skill — does not analyze code, edit files, or choose verdicts. Users must run `/gf-pr-review` or `/gf-pr-inline-review` first to form verdict, or supply verdict explicitly.

## When to Use

| English | 中文 | Context |
|---------|------|---------|
| approve PR, LGTM | 批准 PR、通过了 | verdict ready |
| request changes, reject | 要求修改、驳回 | blocking issues |
| submit review | 提交审查 | after inline comments |
| code review decision | 审查决策 | formal verdict |
| merge / close | 合并/关闭 | → `/gf-pr` |
| review analysis | 审查分析 | → `/gf-pr-review` |
| inline review | 行内审查 | → `/gf-pr-inline-review` |

## Core Pattern

```bash
gf pr view <n>   # verify open
gf review <verdict> <n> --body "<c>"
```

## Quick Reference

| Goal | Command |
|------|---------|
| Comment | `gf review comment <n> --body "<c>"` |
| Approve | `gf review approve <n> --body "<c>"` |
| Request changes | `gf review request-changes <n> --body "<c>"` |
| Submit (after inline) | `gf review submit <n> --event <approved|changes_requested|commented> --body "<c>"` |

**Decision rule:** single verdict → `approve/request-changes`; after inline comments → `submit`; neutral only → `comment`.

## Flowchart

```mermaid
flowchart TD
    START[Start review] --> PR{PR open?}
    PR -->|no| STOP[refuse]
    PR -->|yes| INLINE{done inline review?}
    INLINE -->|no| DO[do inline first]
    INLINE -->|yes| VERDICT{Verdict?}
    VERDICT -->|approve| CONF{confirm?}
    CONF -->|yes| APPROVE[review approve]
    CONF -->|no| STOP2[abort]
    VERDICT -->|changes| CHANGES[review request-changes]
    VERDICT -->|comment only| COMMENT[review comment]
    APPROVE --> SUBMIT[review submit]
    CHANGES --> SUBMIT
    COMMENT --> SUBMIT
```

## Implementation

### Preconditions

- PR `<n>` open — `gf pr view <n>`
- Verdict justified by prior analysis (`/gf-pr-review` or `/gf-pr-inline-review`) or explicit user statement
- Auth valid — `gf auth status`

### Steps

1. **Verify** — `gf pr view <n>`. Confirm open, not draft/merged, no blocking CI. 404 → stop.
2. **Form verdict** — skill does NOT choose; user or prior skill supplies.
3. **Confirm** — present verdict + `--body` to user; require explicit OK before invoking CLI.
4. **Invoke** — `gf review <verdict> <n> --body "<c>"`.
5. **Output** — show review URL + next-step guidance.

### Error Handling

| Error | Recovery |
|-------|----------|
| PR not found / closed | Stop. Check number |
| Already reviewed | Surface; no duplicate |
| Auth failure | `auth login`. Stop |
| Network timeout | Surface; no retry |
| Merge conflict | Advise rebase first |

## Responsibility

### ✅ In Scope

- Verify PR, confirm verdict, invoke CLI, relay result

### ❌ Out of Scope

- Code analysis → `/gf-pr-review`
- Inline comments → `/gf-pr-inline-review`
- Apply feedback → `/gf-pr-apply-feedback`
- Merge / close → `/gf-pr`
- Security scan → `/gf-security-check`

### 🚫 Do Not

- ❌ Approve without prior analysis
- ❌ Auto-submit without user confirmation
- ❌ Review own PR — refuse
- ❌ Submit without `--body`

## 🔁 Delegation

| Intent | Delegate To |
|--------|-------------|
| Submit verdict | This skill |
| Form verdict | `/gf-pr-review` |
| Inline comments | `/gf-pr-inline-review` |
| Apply feedback | `/gf-pr-apply-feedback` |
| Merge / close | `/gf-pr` |

## Rationalization

| Excuse | Reality |
|--------|---------|
| "Urgent, skip analysis" | Urgency ≠ safety |
| "Tiny change" | Small changes can hide vulnerabilities |
| "Already someone approved" | Independent assessment required |

## Red Flags

- 🚩 "Approve without review" — Refuse. Require `/gf-pr-review` first.
- 🚩 "Submit for me" — Refuse. User must confirm.
- 🚩 "My own PR" — Refuse. Self-review prohibited.

## Trigger Keywords

| English | 中文 |
|---------|------|
| approve PR, LGTM | 批准 PR、通过 |
| request changes, reject | 要求修改、驳回 |
| submit review | 提交审查 |
| review verdict | 审查结论 |

## Test Scenarios

### 1: Happy Path — `/gf-pr-review` done, "approve #101" → present body, confirm, invoke `review approve`, output URL.

### 2: Request Changes — 6-dim review found ⚠️ — "request changes on #101" → `review request-changes 101 --body "<conclusion w/ path:line>"`.

### 3: Boundary — "approve #101" without analysis → refuse, require `/gf-pr-review`.

### 4: Negative — "merge #101" → NOT loaded. → `/gf-pr`.

### 5: Self-Review — "approve my PR" → refuse.

## Success Criteria

- [ ] Verdict confirmed by user before invocation
- [ ] Prior analysis exists or user supplies verdict
- [ ] CLI returns success
- [ ] Out-of-scope intents delegated

## Common Mistakes

- ❌ **Auto-submitting** — Step 3 confirmation is mandatory.
- ❌ **Approving without analysis** — violates Preconditions.
- ❌ **Using merge/close** — use `/gf-pr`.

## See Also

- `/gf-pr-review` — 6-dim analysis
- `/gf-pr-inline-review` — line-level comments
- `/gf-pr-apply-feedback` — apply feedback
- `/gf-pr` — PR lifecycle
- `/gf-security-check` — pre-approve scan
- `docs/superpowers/templates/skill-conventions.md` — conventions
