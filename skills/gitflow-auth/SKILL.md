---
name: gitflow-auth
description: |
  Use when the user needs to authenticate, check auth status, obtain an access token, or logout.
  当用户需要登录、检查认证状态、获取访问 Token 或登出时使用。
---

# gitflow-auth

Manages authentication lifecycle: login, logout, status, token retrieval. Does not revoke, rotate, or manage platform-side tokens.

## When to Use

| English | 中文 | Context |
|---------|------|---------|
| login | 登录 | obtain credentials |
| logout | 登出 | clear local credentials |
| auth status | 认证状态 | verify active session |
| access token | 获取 Token | downstream API calls |
| manage platform tokens | 管理平台 Token | **NOT** → web console |

## Core Pattern

```bash
gitflow-cli auth status                # 1. verify state
gitflow-cli auth login [--token <t>]   # 2. authenticate
gitflow-cli auth token                 # 3. retrieve (sensitive)
gitflow-cli auth logout                # 4. clear credentials
```

## Quick Reference

| Goal | Command |
|------|---------|
| Check session | `gitflow-cli auth status` |
| Login | `gitflow-cli auth login [--token <token>]` |
| Get token | `gitflow-cli auth token` |
| Clear credentials | `gitflow-cli auth logout` |

## Implementation

### Preconditions

- `command -v gitflow-cli` available
- Platform: `github` / `gitlab` / `gitcode`
- Mutation skills must verify via `auth status` first

### Step 1: Check State

`gitflow-cli auth status` → `logged_in`, `user`, `scopes`. Not logged in → Step 2.

### Step 2: Execute

- **login**: interactive unless `--token` given. Rejected → suggest interactive.
- **token**: raw credential. MUST follow Token Safety rules below.
- **logout**: clears credentials non-destructively.

### Step 3: Verify

`auth status` confirms intent achieved.

### Token Safety (Mandatory)

🚨 Never log, echo, or surface the token in conversation, comments, commits, or diagnostics.
🚨 Never store the token in files — only OS credential store via `auth login`.
🚨 Never ask user to paste token into chat — use `auth login --token <token>` from terminal.
Output of `auth token` must be captured only into shell variables (`TOKEN=$(gitflow-cli auth token)`).

### Error Handling

| Error | Recovery |
|-------|----------|
| `Platform '{x}' not yet supported` | Stop. Only github/gitlab/gitcode. |
| Login failed / invalid token | Suggest interactive `auth login`. Do not auto-retry. |
| Token fetch while logged out | Run `auth login` first. |
| Network / API timeout | Stop. Do not improvise. |

## Responsibility

### ✅ In Scope

- Interactive and non-interactive login
- Token retrieval for downstream CLI ops
- Status query (logged_in, user, scopes)
- Logout / credential clearing

### ❌ Out of Scope

- Token creation/revocation/rotation on platform → web console
- Mutating resources with token → delegate to `gitflow-issue`, `gitflow-pr`

### 🚫 Do Not

- ❌ Echo, log, or include token in visible output except via raw `auth token` command
- ❌ Persist token to files, env files, commits, or chat
- ❌ Prompt user to paste token into conversation
- ❌ Auto-retry login — stop and report each failure
- ❌ Use token for anything outside credential lifecycle

## Rationalization Excuses

| Excuse | Reality |
|--------|---------|
| "Print the token once for debugging" | Token must never appear in any output. |
| "Test token, so it's safe" | Treat every token as production secret. |
| "Quick retry will fix login" | Surface every failure to user; do not auto-retry. |

## Red Flags

- 🚩 "Print my token here" — Refuse. Direct to terminal `auth token`.
- 🚩 "Paste token into chat" — Refuse. Token never crosses into conversation.
- 🚩 "Store the token in .env" — Refuse. Violates token safety boundary.

## Test Scenarios

### 1: Happy Path

- **Given** installed, not logged in — **When** "Login" — **Then** `auth login`; `auth status` → `logged_in: true`

### 2: Negative

- **Given** "Close issue #42" — **When** no auth intent — **Then** NOT loaded. → `gitflow-issue`.

### 3: Boundary

- **Given** "Print my token" — **When** user pushes for token — **Then** Refuses. Cites Token Safety.

### 4: Error

- **Given** platform `gitea` — **When** `auth status --platform gitea` — **Then** "not yet supported". Stops.

## Success Criteria

- [ ] Auth state matches intent
- [ ] Token never appears in logs, chat, comments, or files
- [ ] No out-of-scope mutation performed

## Common Mistakes

- ❌ **Echoing token in chat** — only returned via `auth token` shell command.
- ❌ **Storing token in `.env`** — exposes to disk/VCS. Use OS credential store only.

## Trigger Keywords

| English | 中文 |
|---------|------|
| login | 登录 |
| logout | 登出 |
| auth status | 认证状态 |
| access token | 获取 Token |

## See Also

- `gitflow-issue` — requires auth check from this skill
- `gitflow-pr` — requires auth check from this skill
- `docs/superpowers/templates/skill-conventions.md` — skill conventions
