---
name: gitflow-release
description: |
  Use when the user needs to manage releases: create, list, view, edit, upload/download assets, or delete a release on GitHub/GitLab/GitCode.
  当用户需要管理 release（创建、列表、查看、编辑、上传/下载资源、删除发布）时使用。
---

# gitflow-release

## Overview

Release CRUD (create, list, view, edit, upload/download, delete). Does NOT decide versions, generate changelogs, or drive publish workflows.

## When to Use

| English | 中文 | Context |
|---------|------|---------|
| create release | 创建 release | publish a version |
| upload asset | 上传资源 | attach binary to release |
| download release | 下载 release | fetch release binary |
| list / edit / delete release | 列表/编辑/删除 | lifecycle management |
| prerelease / draft | 预发布 / 草稿 | RC or draft release |
| changelog / version decision | 变更日志 / 版本决策 | **NOT** → `gitflow-release-helper` |

## Core Pattern

```bash
gitflow-cli auth status                     # 1. verify auth
gitflow-cli release view <tag>              # 2. verify target
gitflow-cli release <sub> <args>            # 3. execute
gitflow-cli release view <tag>              # 4. verify outcome
```

## Quick Reference

| Goal | Command |
|------|---------|
| Create | `gitflow-cli release create --tag <tag> [--name <n>] [--body <b>] [--draft] [--prerelease] [--target <ref>]` |
| List | `gitflow-cli release list` |
| View | `gitflow-cli release view <tag>` |
| Edit | `gitflow-cli release edit <tag> [--name <n>] [--body <b>] [--draft] [--prerelease]` |
| Upload | `gitflow-cli release upload <tag> --file <path> [--asset-name <n>]` |
| Download | `gitflow-cli release download <tag> --asset <name> [--dest <dir>]` |
| Delete | `gitflow-cli release delete <tag>` |

## Implementation

### Preconditions

`command -v gitflow-cli`, `git rev-parse --is-inside-work-tree`, verified auth via `gitflow-cli auth status`.

### Steps

1. **Auth guard.** `gitflow-cli auth status` — failure → "Run `gitflow-cli auth login`", stop.
2. **Target check** (edit/delete/upload only). `gitflow-cli release view <tag>` — 404 → "Not found", stop.
3. **Execute** user intent against Quick Reference. Delete & non-draft publish **require explicit user confirmation** (see Red Flags).
4. **Verify.** `release view <tag>` (or `list`) — confirm state, return URL.

### Error Handling

| Error | Recovery |
|-------|----------|
| `401` | "Run `auth login`", stop |
| `404` | "Release not found", stop |
| `409` (tag exists) | "Tag conflict — use different tag or delete existing first", stop |
| Timeout | Retry once, then "Network error", stop |
| Upload file missing | "File not found", stop |

## Responsibility

### ✅ In Scope

CRUD + auth/target checks + confirm destructive ops.

### ❌ Out of Scope

Version decisions / changelog → `gitflow-release-helper`. Publish workflow → `gitflow-release-helper`. Auth → `gitflow-auth`.

### 🚫 Do Not

- ❌ Delete without explicit confirmation
- ❌ Overwrite asset
- ❌ Publish non-draft without confirmation
- ❌ Modify tags
- ❌ Decide versions / changelog

## Rationalization

| Excuse | Reality |
|--------|---------|
| "Skip confirmation — user wants publish" | Publish/delete always require confirmation |
| "Asset matches — safe" | Overwriting requires confirmation |
| "Tag exists — must be stale" | Ask before delete |
| "Release handles this" | Helper drives workflow; this executes CRUD |
| "Quick delete — busy" | Irreversible ops need confirmation |

## Red Flags

- 🚩 "Delete release" — Require confirmation. Irreversible.
- 🚩 "Overwrite asset" — Require confirmation. Destructive.
- 🚩 "Skip confirmation" — Refuse. Cite §Do Not. Stop.
- 🚩 "Publish without asking" — Refuse. Confirm first.
- 🚩 "Skip precondition" — Non-skippable. Stop.

## Test Scenarios

### 1: Happy Path

- **Given** Auth OK, tag absent, user confirms — **When** "Create release v1.0.0" — **Then** create → view → URL returned

### 2: Negative

- **Given** "Generate changelog and decide version" — **When** No CRUD keyword — **Then** Does NOT load → `gitflow-release-helper`

### 3: Boundary

- **Given** Release exists — **When** "Delete it" without user confirming — **Then** Claude asks, refuses `release delete` until confirmed

### 4: Error

- **Given** Not authenticated — **When** `auth status` → `401` — **Then** "Run `gitflow-cli auth login`", stop

## Success Criteria

- [ ] URL returned
- [ ] Delete/publish: explicit confirmation
- [ ] Preconditions verified
- [ ] Error recovery verbatim
- [ ] No version/changelog decisions

## Common Mistakes

- ❌ **Deleting without confirmation** — Irreversible. Always ask.
- ❌ **Creating release on existing tag** — Tag conflict. Check first.
- ❌ **Generating changelog** — Out of scope → `gitflow-release-helper`.

## Trigger Keywords

| English | 中文 |
|---------|------|
| create release | 创建 release |
| upload / download asset | 上传/下载资源 |
| list releases | 列出 releases |
| edit release | 编辑发布 |
| delete release | 删除 release |
| prerelease / draft | 预发布 / 草稿 |

## See Also

- `gitflow-release-helper` — version decision + changelog + publish workflow
- `gitflow-auth` — authentication prerequisite
- `docs/superpowers/templates/skill-conventions.md` — conventions
