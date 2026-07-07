---
name: gitflow-repo
description: |
  Use when the user needs to clone, list, create, inspect statistics, sync forks, or view details of a git repository via gitflow-cli.
  当用户克隆、列出、创建、统计、同步 fork 或查看仓库详情时使用。
---

# gitflow-repo

## Overview

Single entry for `gitflow-cli repo` commands.

## When to Use

| English | 中文 | Context |
|---------|------|-------|
| clone / 拉代码 | 克隆仓库 | has explicit `<owner>/<repo>` target |
| list repos | 仓库列表 | user or org scope |
| create / 新仓库 | 创建仓库 | brand-new repo request |
| repo stats | 仓库统计 | metrics/health, NOT PR counts |
| sync / fork 同步 | 同步 fork | fork with upstream remote |
| view / 查看仓库 | 查看仓库 | quick facts about a repo |

## Trigger Keywords

clone, list, create, stats, sync, view, delete (unsupported) — 克隆、列表、创建、统计、同步、查看、删除（不支持）

## Quick Reference

| Goal | Command |
|------|---------|
| Clone | `gitflow-cli repo clone <owner>/<repo> [--dir <d>] [--branch <b>]` |
| List | `gitflow-cli repo list [--org <o>] [--visibility <v>] [--language <l>]` |
| Create | `gitflow-cli repo create --name <n> --visibility <v> [--init]` |
| Stats | `gitflow-cli repo stats [<owner>/<repo>]` |
| Sync | `gitflow-cli repo sync` |
| View | `gitflow-cli repo view [<owner>/<repo>]` |

## Core Pattern

```bash
# Step 1 — Auth & CLI check
gitflow-cli auth status        # 401? → `auth login --platform <p>`
gh --version                   # missing? → install, retry

# Step 2 — Classify intent (one action per invocation)

# Step 3 — Execute (create & sync require confirmation gate)
gitflow-cli repo <cmd> <args>
```

## Preconditions

- `gitflow-cli auth status` returns OK before create/sync
- `gh` CLI installed before stats
- Inside a git repo for sync (view works without)

## Error Handling

| Error | Recovery |
|-------|----------|
| `401` / auth failure | `gitflow-cli auth login --platform <p>`, retry once |
| `404` (clone/view/stats) | Confirm `<owner>/<repo>` spelling, stop |
| Name conflict (create) | Suggest alternative name, stop |
| Merge conflict (sync) | Pause, list conflicted files, ask manual resolution |
| `gh` missing (stats) | Install `gh`, retry; do not improvise REST |

## Responsibility

### ✅ In Scope

- Clone / list / view repos
- Create remote repo (user-confirmed)
- Read repo statistics
- Sync fork from upstream

### ❌ Out of Scope

- Onboarding → `gitflow-repo-onboarding`
- Commits → `gitflow-commit`
- Issues / PRs → `gitflow-issue`, `gitflow-pr`
- Releases → `gitflow-release`
- Delete / rename remote repo → manual

### 🚫 Do Not

- ❌ Auto-push sync result to origin without confirmation
- ❌ Create repo without explicit `--visibility` confirmation
- ❌ Clone into existing dir without `--force`
- ❌ Force-push during sync

## Red Flags

- 🚩 User says "create it quickly, public or private" — Refuse. Visibility must be explicit.
- 🚩 User says "push the sync result now" — Do not push. Pause, user pushes manually.

## Common Mistakes

- ❌ **Skipping visibility confirmation** — wrong visibility leaks confidential code. Always confirm first.
- ❌ **Assuming upstream default branch is `main`** — fetch `defaultBranchRef` via stats first. Repos may use `master`/`trunk`.

## Test Scenarios

### Scenario 1: Happy Path

- **Given** Authenticated, `byx/gitflow-cli` exists
- **When** "clone repo and show it to me"
- **Then** `repo clone` exits 0, `repo view` outputs stats and URL

### Scenario 2: Negative — issue intent should NOT fire this skill

- **Given** User asks "open an issue on repo X"
- **When** `repo X` phrase co-occurs with issue verbs
- **Then** Claude does NOT load gitflow-repo; redirects to `/gitflow-issue`

### Scenario 3: Boundary — user bundles create with setup tasks

- **Given** "create my-new-repo public, also add README and LICENSE"
- **When** User bundles create and onboarding
- **Then** Confirm visibility, run `repo create`, redirect to `/gitflow-repo-onboarding`.

### Scenario 4: Error — merge conflict during sync

- **Given** Local fork behind upstream, conflict on `Cargo.toml`
- **When** `repo sync` encounters conflict
- **Then** Pause, list `Cargo.toml`, ask user to resolve. Does NOT push, does NOT abort silently.

## Success Criteria

- [ ] Command chosen from Trigger Keywords, confirmed if ambiguous
- [ ] `--visibility` confirmed before `repo create`
- [ ] `sync` does NOT push without explicit confirmation
- [ ] All errors mapped to Error Handling table; no ad-hoc REST

## See Also

- `gitflow-repo-onboarding` — initializes local environment after clone or create
- `gitflow-auth` — verifies authentication before remote writes
- `gitflow-workflow` — starts workflow for a new repo
- `docs/superpowers/templates/skill-template.md` — template this skill conforms to
