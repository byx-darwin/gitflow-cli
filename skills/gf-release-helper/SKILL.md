---
name: gf-release-helper
description: >
  Use when the user wants to create a new release, auto-generate release notes
  from conventional commits since the last tag, or decide SemVer version bumps.
  当用户需要按 conventional commits 决定版本号、生成 changeline、创建
  Release 时使用。
---

# gf-release-helper — Semantic Release Helper

Automates: determine next version → generate changelog → create release → output URL.
Full reference: docs/references/gf-release-helper-params.md

## CLI Requirement

**MUST use `gf` CLI, NOT `gh` CLI.**

| CLI | Scope | Platform Support |
|-----|-------|------------------|
| `gf` | This project | GitHub + GitLab + GitCode |
| `gh` | GitHub only | GitHub only |

**Why**: `gf` is the unified CLI for this project. Using `gh` breaks GitLab/GitCode compatibility.

## Preconditions
- `gf` installed: `command -v gf`
- `gf` authenticated: `gf auth status`
## Overview

Infers the next SemVer version from conventional commits, generates a changelog, and creates the release.

## Trigger Keywords

CN 发布 release 版本号 changelog 打标签
EN create release bump version semantic version release notes tag major minor
CLI `gf release-helper <subcommand>`

## When NOT to Use

| Scenario | Why Not | Use Instead |
|----------|---------|-------------|
| Managing existing Release metadata (edit/delete/list) | This skill creates new releases with auto-generated changelogs | `/gf-release` for Release CRUD operations |
| Deleting a release | This skill only creates releases, never deletes them | `/gf-release` for release deletion (with double-confirm) |
| Deciding version without user confirmation | This skill infers SemVer but always requires user approval | Non-negotiable — version must be confirmed interactively |
| Publishing without showing release notes | This skill requires user review of changelog before creation | Non-negotiable — always show release notes first |
| Creating Git tags | This skill assumes tags exist or creates them as part of release flow | `git tag` + `git push --tags` for tag-only operations |
| Running release without CI verification | Releasing without confirming CI status risks broken releases | `/gf-pipeline-analyzer` to verify pipeline health first |

## Version Decision Flow

```mermaid
flowchart TD
  U[User wants to release] --> A{Tag exists?}
  A -->|no tag| B[Initialize v0.1.0 or v1.0.0]
  A -->|has tag| C[git describe --tags --abbrev=0]
  C --> D[git log <last-tag>..HEAD --pretty=format:'%h %s']
  D --> E{Classify commit types}
  E -->|feat! / BREAKING CHANGE| MAJOR[Major X+1.0.0]
  E -->|feat without breaking| MINOR[Minor x.Y+1.0]
  E -->|only fix/perf/refactor| PATCH[Patch x.y.Z+1]
  MAJOR --> CONFIRM[User confirms version]
  MINOR --> CONFIRM
  PATCH --> CONFIRM
  CONFIRM --> GEN[Generate grouped changelog]
  GEN --> REVIEW[User reviews]
  REVIEW --> CREATE[release create --tag <v> --notes '...']
  CREATE --> OUT[Output Release URL]
```

## Quick Reference

| Step | Command |
|------|---------|
| Latest tag | `git describe --tags --abbrev=0` |
| commits | `git log <tag>..HEAD --pretty=format:"%h %s" --no-merges` |
| Create release | `gf release create --tag <v> --notes "..."` |

## Pattern Triplets

| Scenario | Handling |
|------|------|
| breaking change | Major +1 → confirm → changelog → `release create` |
| feat only | Minor +1 |
| fix/refactor/perf only | Patch +1 |

## Responsibility / Forbidden

✅ Version inference + changelog generation + invoking `release create`
🔴 Never decide the version unilaterally / release unattended / skip draft / modify tags

## Red Flags + Defense

- "Auto-publish" → refuse; the user must confirm interactively
- Creating without showing the Release Note → force a review

## Common Mistakes

| Mistake | Fix |
|------|------|
| breaking change not bumped to Major | re-check every time |
| `--notes-file` not cleaned up | delete the temp file after a successful release |

## Rationalization

"I'll just guess a version" → SemVer affects dependents; it must be confirmed

## Error Handling

| Error | Handling |
|------|------|
| brand-new repo with no tag | suggest v0.1.0, user confirms |
| CI not passing | suggest running pipeline-analyzer first |
| `release create` fails | keep the Note; prompt to retry |

## Test Scenarios

- **Happy**: "Release the next version" → infer Minor → confirm → changelog → create → URL
- **Negative**: "Delete this release" → refuse; suggest gf-release CRUD
- **Boundary**: breaking change but Patch still chosen → warn about the mismatch; insist on Major
- **Error**: repo has no tag → suggest starting fresh at v0.1.0; create after the user confirms

## Success Criteria

- Version inference conforms to SemVer
- Release is created only after the user confirms the version and Release Note
- Release URL is output successfully
- Temp files are cleaned up

## See Also

- gf-release — Release CRUD
- gf-auth — pre-release status check
- gf-pipeline-analyzer — confirm CI status before release
- gf-label-milestone — associate version milestones
