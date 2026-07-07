---
name: gitflow-release
description: |
  Use when the user wants to manage Git releases through gitflow-cli — create, list, view, edit, upload/download assets, or delete.
  当用户希望通过 gitflow-cli 管理版本发布（创建、列表、查看、编辑、上传/下载资源或删除）时使用。
---

# gitflow-release

CRUD wrapper for `gitflow-cli release`. Manages GitHub/GitLab/GitCode releases — metadata only; tag must exist first. Delete is irreversible.

## When to Use

| English | 中文 | Context |
|---------|------|---------|
| / create a release | 创建 Release | tag exists, needs publish |
| list releases | 列出 Release | review published versions |
| view / edit release | 查看/编辑 Release | metadata update |
| upload / download asset | 上传/下载资源 | binaries, archives |
| delete release | 删除 Release | rollback / mistake |

## Core Pattern

```bash
gitflow-cli auth status
git tag -l <tag>
gitflow-cli release <subcommand> ...
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

- Tag exists locally and on remote — `git tag -l <tag>`
- `gitflow-cli` authenticated — `auth status`
- For upload: local file exists

### Flow by subcommand

- **create** — confirm tag, draft/prerelease flags, then `release create`. Wait for success. Report Release ID + URL.
- **list** — `release list`. Tabular output: tag, name, draft, prerelease, created.
- **view** — `release view <tag>`. 404 → "Release `<tag>` not found.".
- **edit** — `release edit <tag> ...`. Only non-empty flags required; confirms before publish.
- **upload** — confirm file, optional rename, `release upload`. Reports asset URL.
- **download** — confirm asset + dest, `release download <tag> --asset <n> [--dest <dir>]`.
- **delete** — **Irreversible.** Confirm `<tag>` twice, then `delete`.

### Error Handling

| Error | Recovery |
|-------|----------|
| Tag missing | Stop. `git tag` first. |
| Unauthenticated | Stop. `auth login`. |
| Not found (404) | Stop. Inform user. |
| Upload failure | Surface error; do not retry. |
| Delete already done | Stop. Confirm before invoking. |

## Responsibility

### ✅ In Scope

- Execute CRUD against Release resource
- Report new/modified/delivered URLs

### ❌ Out of Scope

- Creating the Git tag → `git tag`, `git push --tags`
- Changelog generation → `/gitflow-release-helper`
- Release orchestration → `/gitflow-release-helper`

### 🚫 Do Not

- ❌ Delete without double-confirm
- ❌ Create release without first confirming tag exists
- ❌ Upload non-existent file
- ❌ Generate changelog text — leave to release-helper

## Rationalization Excuses

| Excuse | Reality |
|--------|---------|
| "Tag probably exists" | Missing tag → CLI fails. Verify first. |
| "Just delete it, easy restore" | Release deletion is **irreversible** on all platforms. |
| "Skip the name" | Name defaults to tag; confirm if that's intended. |

## Red Flags

- 🚩 "Delete the release" — Double confirm tag before invoking.
- 🚩 "Upload without checking file" — Verify path exists first.
- 🚩 "Create release, tag doesn't matter" — Stop, tag is required.

## Test Scenarios

### 1: Happy Path
- **Given** tag `v1.0.0` exists — **When** "create release v1.0.0" — **Then** invokes `release create --tag v1.0.0 ...`, returns Release URL.

### 2: Negative
- **Given** "delete tag v1.0.0" — **Then** NOT loaded. → git CLI. This skill is for releases, not tags.

### 3: Boundary
- **Given** "upload binary and also generate the changelog" — **Then** `upload` only; redirect changelog → `/gitflow-release-helper`.

### 4: Error
- **Given** "create release v3.0.0" but no such tag — **Then** stop, "Tag v3.0.0 missing. Run `git tag` first."

### 5: Boundary
- **Given** "delete release v1.0.0" — **Then** prompt for double-confirm. Do not invoke on first ask.

## Success Criteria

- [ ] Release URL returned on create
- [ ] Tag existence verified before create
- [ ] Delete required double-confirm
- [ ] Out-of-scope intents redirected

## Common Mistakes

- ❌ **Creating release on missing tag** — verify tag first.
- ❌ **Deleting without confirmation** — always double-confirm.

## See Also

- `gitflow-release-helper` — version decision, changelog, release orchestration
- `gitflow-quality` — pre-release quality gate
- `gitflow-pr` — PR lifecycle
- `gitflow-label-milestone` — milestone association
