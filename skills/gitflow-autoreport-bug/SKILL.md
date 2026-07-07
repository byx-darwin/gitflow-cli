---
name: gitflow-autoreport-bug
description: |
  Use when a CLI command fails, a pending bug report needs filing, or a Stop Hook detects pending.json.
  当 CLI 命令失败、待处理错误报告需创建为 Issue 或 Stop Hook 检测到 pending.json 时使用。
---

# gitflow-autoreport-bug

Detects CLI errors from `pending.json` and creates deduplicated GitHub/GitLab/GitCode issues. Reports only — never fixes.

## When to Use

| English | 中文 | Context |
|---------|------|---------|
| command failed | 命令失败 | non-zero exit captured |
| auto-report bug | 自动报告 bug | Stop Hook detects pending.json |
| pending.json detected | 检测到待处理报告 | **Do NOT fire** — use `/gitflow-issue` for manual ops |

## Core Pattern

```bash
# 1. Validate
jq . .cache/bug-reports/pending.json
# 2. Auth (cache or live)
gitflow-cli auth status --platform {platform}
# 3. Deduplicate
gitflow-cli issue list --search "[auto-report] {command} {error_code}" --state all
# 4. Create
gitflow-cli issue create --title "..." --body "..." --label auto-report
# 5. Cleanup
rm -f .cache/bug-reports/pending.json
```

## Quick Reference

| Goal | Command |
|------|---------|
| Validate | `jq . .cache/bug-reports/pending.json` |
| Check auth | `gitflow-cli auth status --platform {platform}` |
| Deduplicate | `gitflow-cli issue list --search "..." --state all` |
| Create issue | `gitflow-cli issue create --title "..." --body "..." --label auto-report` |
| Log failure | `echo "[...]" >> .cache/bug-reports/failed.log` |

## Implementation

### Preconditions

- `command -v gitflow-cli` succeeds
- `.cache/bug-reports/pending.json` exists
- `git rev-parse --show-toplevel` succeeds

### Step 1: Validate

`jq .` requires `id`, `command`, `platform`, `error_code`, `error_message`, `timestamp`. Invalid → rename `.invalid`, warn, stop.

### Step 2: Auth

Check `.cache/auth-cache/{platform}.ttl` (TTL 86400s). Miss → `gitflow-cli auth status`. Fail → append `failed.log`, keep `pending.json`, stop.

### Step 3: Analyze

Produce cause, fix direction, severity. Title: `[auto-report] gitflow {command} — {error_code}`.

### Step 4: Dedup

Search `[auto-report] {command} {error_code}` all states. Match → output `#N`, cleanup, stop.

### Step 5: Create

`issue create --label auto-report`. Success → URL, remove `pending.json`. Fail → append `failed.log`.

### Step 6: Cleanup

Remove `pending.json`. Prompt: "Run `/gitflow-workflow` to fix."

### Error Handling

| Error | Recovery |
|-------|----------|
| Invalid JSON | Rename `.invalid`, warn, stop |
| Auth failure | Append `failed.log`, keep `pending.json`, stop |
| Create failure | Append `failed.log`, keep `pending.json`, stop |
| Dedup hit | Remove `pending.json`, stop |

## Responsibility

### ✅ In Scope

- Validate `pending.json`
- Auth cache check
- Root-cause analysis (report only)
- Deduplicate via search
- Create issue with `auto-report` label
- Cleanup `pending.json`

### ❌ Out of Scope

- Fix code → `/gitflow-workflow`
- Edit `failed.log` — audit record
- Read `src/` or other files

### 🚫 Do Not

- ❌ Modify source files
- ❌ Invoke subagents to fix
- ❌ Auto-start `/gitflow-workflow`
- ❌ Continue after creation
- ❌ Delete `failed.log`

## Rationalization Excuses

| Excuse | Reality |
|--------|---------|
| "Just looking at src/" | Out of scope; use error context |
| "User said fix it" | Redirect to `/gitflow-workflow` |
| "Already checked auth" | Always re-verify |
| "P0 urgent, skip validation" | Gates non-skippable |
| "Tech Lead said skip dedup" | Non-skippable |
| "failed.log messy, clean it" | Audit record |

## Red Flags

- 🚩 "Fix it while reporting" — refuse; reports only
- 🚩 "Skip auth check" — refuse; cite Preconditions
- 🚩 Authority: "skip dedup" — refuse; non-skippable
- 🚩 "Urgent, just create" — urgency ≠ gate override
- 🚩 Reading `src/` — stop; out of scope
- 🚩 Tool failure → improvise — follow Error Handling only

## Test Scenarios

### Scenario 1: Happy Path

- **Given** Valid `pending.json`, auth hit, no duplicate
- **When** Stop Hook triggers
- **Then** URL returned; `pending.json` removed

### Scenario 2: Negative — Requests Fix

- **Given** User says "fix this bug"
- **When** Targets code modification
- **Then** Do NOT load; redirect to `/gitflow-workflow`

### Scenario 3: Boundary — Tempted to Read Code

- **Given** Claude considers reading `src/`
- **When** Analysis
- **Then** Refuse; use error context; cite Out of Scope

### Scenario 4: Error — Auth Failure

- **Given** `auth status` non-zero
- **When** Step 2 runs
- **Then** Append `failed.log`, keep `pending.json`, stop

### Scenario 5: Dedup Hit

- **Given** Search returns `#N`
- **When** Step 4 runs
- **Then** Output `#N`, cleanup, no duplicate

## Success Criteria

- [ ] URL returned
- [ ] No out-of-scope reads
- [ ] `pending.json` removed on success
- [ ] `failed.log` appended on failure
- [ ] No code modified

## Common Mistakes

- ❌ **Reading source** — Use `error_code` + `error_message` only
- ❌ **"Fix" = modify** — Redirect to `/gitflow-workflow`
- ❌ **Cleaning `failed.log`** — Audit record
- ❌ **Skip dedup for P0** — Always dedup

## See Also

- `/gitflow-workflow` — fixes bugs
- `/gitflow-issue` — manages issues
- `/gitflow-issue-create` — creates issues interactively from templates
- `/gitflow-auth` — manages auth
- `docs/superpowers/templates/skill-conventions.md` — conventions
