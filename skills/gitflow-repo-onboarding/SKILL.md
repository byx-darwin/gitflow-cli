---
name: gitflow-repo-onboarding
description: |
  Use when generating a project onboarding guide from repo structure, conventions, and toolchain. Chat-only output — never writes files.
---

# gitflow-repo-onboarding

Read-only analysis → onboarding walkthrough in chat. **Never writes files.**

## When to Use

| Trigger | 中文 | Redirect |
|---------|------|----------|
| onboarding, walkthrough | 入门指南、上手 | — |
| how to build/setup/contribute | 如何构建 | — |
| repo conventions | 项目约定 | — |
| PR review | PR/审查 | → `gitflow-pr-review` |

## Core Pattern

```bash
git remote -v && git remote show origin | grep 'HEAD branch'
ls -F && head -5 README.md 2>/dev/null
ls Makefile Cargo.toml package.json go.mod 2>/dev/null
make help 2>/dev/null; ls .github/workflows/ 2>/dev/null
git log --oneline -10 && git branch -r | head -5
```

## Quick Reference

| Goal | Action |
|------|--------|
| Toolchain | Manifests |
| Build/test/lint | `make help` → CLI |
| Lint/fmt/commit | `rustfmt.toml`, `commitlint` |
| CI | `.github/workflows/*` — cite only actual |

## Steps

Confirm `git rev-parse --is-inside-work-tree`.

1. **Detect** — `git remote` + `ls -F`. Map language + branch.
2. **Toolchain** — `ls Makefile Cargo.toml package.json 2>/dev/null; make help 2>/dev/null`. Makefile first, CLI fallback.
3. **Conventions** — `rustfmt.toml`, `commitlint`, `git log -15`, `git branch -r | head -10`.
4. **CI** — `ls .github/workflows/`. Cite actual; never invent.
5. **Synthesize** — Sections: overview · prereqs · quickstart · tree · conventions · CI · resources. **Stay in chat.**

### Error Handling

| Error | Recovery |
|-------|----------|
| Not a git repo | Run inside repo |
| No Makefile / No CI | CLI fallback / Omit |
| Ambiguous manifests | Parse top two |

## Responsibility

**In:** Read-only analysis · synthesize walkthrough · chat.

**Out:** Writing files · editing configs/CI · installs · repo pages (→ `gitflow-repo`).

**Prohibited:** ❌ Writing files · ❌ Fabricating CI · ❌ Executing installs · ❌ Editing manifests.

## Rationalization Excuses

| Excuse | Truth |
|--------|-------|
| "Save" | User decides. |
| "Missing CI" | Omit. |
| "Install hooks" | Describe. |

## Red Flags

- 🚩 "save the guide" — no auto-write
- 🚩 "skip conventions" — non-negotiable
- 🚩 "assume CI checks" — cite real config
- 🚩 "install hooks for them" — describe only

## Test Scenarios

### 1: Happy Path
- **Given** Rust workspace · **When** "generate" · **Then** Walkthrough in chat.

### 2: Negative
- **Given** "merge my PR?" · **Then** → `gitflow-pr`.

### 3: Boundary
- **Given** "save as docs/ONBOARDING.md" · **Then** Ask consent before `Write`.

### 4: Error
- **Given** No Makefile/CI · **Then** Native CLI; omit CI.

## Success Criteria

- [ ] Commands from actual files
- [ ] Commit/branch from git history
- [ ] CI matches real workflows
- [ ] Chat-only output
- [ ] Plain language
- [ ] No auto-write, no fabricated CI, no executed installs

## Trigger Keywords

onboarding, newcomer, walkthrough, setup, contribute, conventions, code tour
