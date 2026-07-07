---
name: gitflow-release-helper
description: |
  Use when the user wants to create a new release with auto-generated release notes from conventional commits since the last tag.
  当用户想基于上次 tag 以来的 conventional commits 自动生成 Release Note 并创建发布时使用。
---

# gitflow-release-helper

Orchestrates the release workflow: infers next SemVer via conventional commits, generates release notes, calls `/gitflow-release` to create the release, emits the URL. Does not perform CRUD on existing releases.

## When to Use

| English | 中文 | Context |
|---------|------|---------|
| create release / publish version | 发布版本 | user asks to cut a release |
| bump version / next version | 升级版本 | user asks what next version should be |
| release notes / changelog | 发布说明 | user wants tag-to-HEAD notes |
| delete / edit release | 删除/编辑 release | **do NOT fire** → `/gitflow-release` |

## Core Pattern

```bash
command -v gitflow-cli && git rev-parse --git-dir
last=$(git describe --tags --abbrev=0)
git log "$last"..HEAD --pretty=format:"%h %s" --no-merges
# infer → confirm → create
gitflow-cli release create --tag "$next" --notes-file /tmp/rel.md
rm -f /tmp/rel.md
```

## Quick Reference

| Goal | Command |
|------|---------|
| Latest tag | `git describe --tags --abbrev=0` |
| Commits since tag | `git log <tag>..HEAD --pretty=format:"%h %s" --no-merges` |
| Create release | `gitflow-cli release create --tag <v> --notes-file <path>` |
| Draft release | `gitflow-cli release create --tag <v> --draft --notes-file <path>` |

## Implementation

### Preconditions

- `command -v gitflow-cli` installed
- `git rev-parse --git-dir` inside a repo
- `gitflow-cli auth status` valid
- On `main` or `release/*`
- CI green (if `.github/workflows` exists) — `gitflow-cli pipeline status`

### Step 1 — Determine Next Version

`git describe --tags --abbrev=0` → `<last>` (or repo root if no tag). Pull commits; infer: `feat!`/breaking → major; `feat` → minor; `fix`/`perf`/`refactor` → patch. **Present inference — wait for explicit yes.**

### Step 2 — Release Notes

Group commits by conventional type; breaking changes pinned top. Write to `/tmp/rel.md`. Show; await approval.

### Step 3 — Create Release

```bash
gitflow-cli release create --tag <v> --notes-file /tmp/rel.md
```

Success → emit URL. Failure → Error Handling table. Do not improvise.

### Step 4 — Cleanup

`rm -f /tmp/rel.md`. Present version, tag, URL.

## Error Handling

| Error | Recovery |
|-------|----------|
| No tags | Use repo root as baseline; continue |
| CI not green | Refuse; offer `--draft` only on explicit user request |
| Tag exists | Refuse; ask for different version |
| API failure | Preserve `/tmp/rel.md`; emit error; stop |

## Responsibility

- ✅ Infer SemVer, generate notes, create release, emit URL
- ❌ Edit/delete releases → `/gitflow-release` · Tag management → manual `git` · Fixing CI → `/gitflow-workflow`
- ❌ Do not: decide version without confirmation · run unattended in CI/CD · skip draft gate · delete/move tags · delete any released release

## Rationalization Excuses

| Excuse | Reality |
|--------|---------|
| "User isn't here, I'll pick the version" | Explicit confirmation required — always wait |
| "CI is flaky, skip the check" | Gate is non-negotiable; offer `--draft` instead |
| "Let me also delete the old release" | Out of scope; never mutate existing releases |

## Red Flags

- 🚩 "auto-publish" / "release without asking" — refuse; confirmation mandatory
- 🚩 "skip the CI check" — refuse; cite Preconditions
- 🚩 "just pick the version" — present inference, require explicit yes
- 🚩 "release from this feature branch" — refuse; only `main` or `release/*`

## Trigger Keywords

| English | 中文 |
|---------|------|
| create release | 创建发布 |
| publish version | 发布版本 |
| release notes | 发布说明 |
| bump version | 升级版本 |
| changelog | 变更日志 |
| breaking change | 破坏性变更 |

## Test Scenarios

### S1 Happy Path
- **Given** authed, on `main`, tag `v1.2.0`, commits `feat:`/`fix:`/`docs:`
- **When** user says "create a new release"
- **Then** proposes `v1.3.0`, shows grouped notes, waits for confirmation, runs `release create`, emits URL

### S2 Negative
- **Given** "delete v1.0.0 release"
- **When** utterance matches delete/edit intent
- **Then** does NOT load this skill; redirects to `/gitflow-release`

### S3 Boundary
- **Given** user says "publish without confirmation"
- **When** user bypasses confirmation gate
- **Then** refuses, cites `🚫`, stops

### S4 Error
- **Given** `auth status` returns `401`
- **When** `release create` runs
- **Then** runs `auth login --platform <p>`, retries once; if still failing preserves `/tmp/rel.md`, stops. Does NOT improvise with `gh release create`.

## Success Criteria

- [ ] Version proposed and confirmed before any mutation
- [ ] Release created via `gitflow-cli release create` with URL returned
- [ ] No out-of-scope action (no tag deletion, no release edit)
- [ ] Temp file cleaned up after success; preserved on failure

## Common Mistakes

- ❌ **Auto-selecting the version** — always present inference and wait for explicit confirmation
- ❌ **Skipping CI gate because "it's just docs"** — gate is unconditional; offer `--draft` if user insists
- ❌ **Using `gh release create` when gitflow-cli fails** — follow Error Handling; do not improvise

## See Also

- `/gitflow-release` — CRUD on existing releases (edit, delete, upload, download)
- `/gitflow-auth` — authentication prerequisite
- `docs/superpowers/templates/skill-conventions.md` — template conventions this skill conforms to
