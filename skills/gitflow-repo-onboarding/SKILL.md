---
name: gitflow-repo-onboarding
description: |
  Use when generating a project onboarding guide from repo structure, conventions, and toolchain. Chat-only output — never writes files.
  当用户要求生成项目入门指南或总结项目结构和约定时使用 — 输出纯对话，不写入文件。
---

# gitflow-repo-onboarding

## Overview

Read-only analysis → onboarding walkthrough in chat. Never writes files.

## When to Use

| Trigger | 中文 | Redirect |
|---------|------|----------|
| onboarding, walkthrough | 入门指南, 上手 | — |
| how to build / setup / contribute | 如何构建 | — |
| repo conventions | 项目约定 | — |
| PR review | 审查 PR | → `gitflow-pr-review` |

## Core Pattern

```bash
git remote -v && git remote show origin | grep 'HEAD branch'
ls -F && head -5 README.md 2>/dev/null
ls Makefile Cargo.toml package.json go.mod 2>/dev/null
make help 2>/dev/null; ls .github/workflows/ 2>/dev/null
git log --oneline -10 && git branch -r | head -5
```

## Preconditions

```bash
test -d .git || { echo "Not in a git repo"; exit 1; }
command -v git
```

Detect language via manifest: `Cargo.toml` · `package.json` · `pyproject.toml` · `go.mod` · `pom.xml` · `Makefile`.

## Steps

1. **Detect** — `git remote -v` + `ls -F`. Map language + default branch.
2. **Toolchain** — `ls Makefile Cargo.toml package.json 2>/dev/null; make help 2>/dev/null`. Makefile first, native CLI fallback.
3. **Conventions** — `rustfmt.toml`, `commitlint`, `git log -15`, `git branch -r | head -10`.
4. **CI** — `ls .github/workflows/`. Cite actual; never invent.
5. **Synthesize** — Sections: overview · prereqs · quickstart · tree · conventions · CI · resources. Stay in chat.

## Quick Reference

| Goal | Action |
|------|--------|
| Toolchain | Manifest → Cargo.toml / package.json / go.mod / ... |
| Build/test/lint | `make help` → native CLI fallback |
| Lint/fmt/commit | `rustfmt.toml`, `commitlint`, `git log -15` |
| CI | `.github/workflows/*` — cite actual only |

## Responsibility

**In:** read-only analysis · synthesize walkthrough · chat output.

**Out:** writing files · editing configs/CI · installs · repo pages (→ `gitflow-repo`).

### 🚫 Do Not

- ❌ Auto-write the guide — ask explicit consent first
- ❌ Fabricate CI steps — cite existing workflow files only
- ❌ Execute installs — describe only
- ❌ Edit manifests or `.pre-commit-config.yaml`

## Rationalization Excuses

| Excuse | Reality |
|--------|---------|
| "Save the output" | User decides — never auto-write |
| "Missing CI is fine" | Omit CI section when absent |
| "Install hooks for them" | Describe only; do not execute |
| "Infer build from README" | Verify with actual manifest + `make help` |

## Red Flags

- 🚩 "Save as `docs/ONBOARDING.md`" — confirm before `Write`
- 🚩 "Skip conventions" — conventions are non-negotiable
- 🚩 "Assume CI checks" — must cite real `.github/workflows/` config
- 🚩 "Install the hooks" — describe, do not execute

## Error Handling

| Error | Recovery |
|-------|----------|
| Not a git repo | Run inside repo (`cd` or `gitflow-cli repo clone`) |
| No Makefile/CI | CLI fallback / Omit CI section |
| Ambiguous manifests | Parse top two; multi-language sections |
| README missing | Skip prereqs summary |

## Test Scenarios

### 1: Happy Path
- **Given** Rust workspace · **When** "generate onboarding" · **Then** Walkthrough in chat

### 2: Negative
- **Given** "merge my PR?" · **Then** → `gitflow-pr` (NOT loaded)

### 3: Boundary
- **Given** "save as `docs/ONBOARDING.md`" · **Then** Ask explicit consent before Write

### 4: Error
- **Given** No Makefile/CI · **Then** Native CLI fallback; omit CI section

## Success Criteria

- [ ] Commands derived from actual files (never guessed)
- [ ] Commit/branch conventions from `git log`
- [ ] CI matches real workflow files — no fabrication
- [ ] Chat-only output (no auto-write)
- [ ] Plain-language, newbie-friendly
- [ ] No fabricated CI, no executed installs

## Common Mistakes

- ❌ **Saving the guide without consent** — chat-only unless user explicitly asks.
- ❌ **Fabricating CI steps** — cite only what `.github/workflows/` actually contains.

## See Also

- `gitflow-repo` — repo-level writes
- `gitflow-auth` — verify login
- `gitflow-commit` — commit conventions
- `gitflow-label-milestone` — label/milestone CRUD

## Trigger Keywords

| English | 中文 |
|---------|------|
| onboarding, walkthrough, newcomer | 入门指南, 上手, 新人 |
| how to build / setup / contribute | 如何构建, 如何开始 |
| repo conventions, code style | 项目约定, 代码规范 |
| getting started, quick start | 快速开始, 快速上手 |
