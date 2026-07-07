# Task 25: Refactor gitflow-auth — Report

> **Task Date:** 2026-07-07
> **Target:** `skills/gitflow-auth/SKILL.md` (+ `.claude/skills/gitflow-auth/SKILL.md` local runtime copy)
> **Related Issue:** #15
> **Effort:** 3h
> **Risk:** Low
> **Status:** Done

---

## 执行摘要

Refactored `gitflow-auth` from the legacy "命令概览 + 参数说明"清单格式 to the canonical Superpowers template (mirroring `gitflow-pr-inline-review`). Added explicit **token safety boundaries** (brief requirement): a mandatory `Token Safety` section, three token-specific Red Flags, and a Rationalization Excuses counter-table focused on token handling temptations.

Word count landed at **473** (canonical script, CJK-aware), within the 500-word budget.

---

## Before → After

| Aspect | Before | After |
|--------|--------|-------|
| Frontmatter description | Functional ("gitflow-cli 的认证操作命令封装…") | Trigger-only ("Use when the user needs to authenticate…") |
| Structure | 2 sections (命令概览 / 参数说明) + examples | 15 sections per skill-template.md |
| Token safety | None | Mandatory Token Safety section + 3 Red Flags + 3 Do Nots + 3 rationalization counters |
| Boundaries | None | In Scope / Out of Scope / Do Not + delegation redirects |
| Testability | None | 4 test scenarios (Happy / Negative / Boundary / Error) |
| Cross-refs | None | 2 skill refs + 1 template ref |
| Trigger keywords | None | 4 EN / 4 ZH |
| Success criteria | None | 3 checkboxes |

---

## Self-Review Checklist (skill-conventions §15)

| Item | Status |
|------|--------|
| `description` matches `/^Use when/i` (English portion) | Pass |
| Contains 1–2 sentence overview under H1 | Pass |
| `## When to Use` with EN/ZH trigger + Context column | Pass |
| `## Core Pattern` executable skeleton | Pass |
| `## Quick Reference` cheat-sheet (4 rows) | Pass |
| `## Implementation` with Preconditions + 3 Steps + Error Handling | Pass |
| `## Responsibility` (In / Out / Do Not) | Pass |
| `## Common Mistakes` (≥ 2) | Pass (2) |
| `## Red Flags` (1 skill-specific) | Pass (3, all token-related) |
| `## Trigger Keywords` (≥ 3 EN + 3 ZH) | Pass (4+4) |
| `## See Also` (≥ 2 cross-refs) | Pass (3) |
| `## Test Scenarios` (≥ 4 incl. 1 negative) | Pass (4 incl. 1 negative) |
| `## Success Criteria` (checkboxes, verifiable) | Pass (3) |
| Word count ≤ 500 | Pass (473) |
| No fictional data in examples | Pass |
| No narrative examples | Pass |
| Pattern language (Condition → Action → Result) | Pass |
| Cross-references bidirectional | Partial — peer skills (`gitflow-issue`, `gitflow-pr`) do not yet link back to `gitflow-auth` because they still run the legacy template. Bidirectionality will be achieved when those skills are refactored. |

---

## Token Safety Boundaries Added

Centerpiece of this refactor. Prior skill had zero token handling guidance. Additions:

1. **`### Token Safety (Mandatory)` section** — four rules:
   - Never log / echo / surface token in conversation, comments, commits, diagnostics.
   - Never store token in files (`.env`, notes, shell history) — only OS credential store via `auth login`.
   - Never ask user to paste token into chat — terminal-only `auth login --token <token>`.
   - `auth token` output captured only into Shell variables (`TOKEN=$(gitflow-cli auth token)`).

2. **`### Do Not`** — 5 items, three directly about token exposure / persistence / prompting.

3. **`## Red Flags`** — three token-specific tripwires ("Print my token here", "Paste token into chat", "Store the token in .env") each mapped to "Refuse. Cite Token Safety."

4. **`## Rationalization Excuses`** — three common Claude temptations around token handling ("Print once for debugging", "Test token so it's safe", "Quick retry will fix login") with one-line rebuttals.

---

## Test Scenarios Summary

| # | Type | Given | When | Then |
|---|------|-------|------|------|
| 1 | Happy | Installed, not logged in | "Login" | Runs `auth login`; status → `logged_in: true` |
| 2 | Negative | "Close issue #42" | No auth intent | NOT loaded; redirect to `gitflow-issue` |
| 3 | Boundary | "Print my token" | Token exposure attempt | Refuses; cites Token Safety |
| 4 | Error | Platform `gitea` | `auth status --platform gitea` | "not yet supported"; stops |

---

## Commit

```
af9c92e refactor(skill): gitflow-auth — conform to Superpowers template with token safety boundaries (#15)
```

Pre-commit hooks (fmt, typos, gitleaks, etc.) all Passed. Branch `feat/issue-repo-parameter`. Only `skills/gitflow-auth/SKILL.md` committed (the `.claude/skills/` copy is gitignored runtime state and was updated in place but not committed).

---

## Files Touched

| File | Change |
|------|--------|
| `skills/gitflow-auth/SKILL.md` | Refactored (committed) |
| `.claude/skills/gitflow-auth/SKILL.md` | Refactored (gitignored runtime copy, kept in sync) |

---

## Known Gaps (P2 / Future Work)

- Bidirectional cross-refs with `gitflow-issue` / `gitflow-pr` not yet satisfied — those skills still run the legacy template. Will resolve as peer skills are refactored.
- No Mermaid flowchart added — auth subcommand control flow is linear (4 flat subcommands), below the 3+ decision-branch threshold in conventions §9.1.
- Stress test scenarios (conventions §3.3 P2) not in TASK-25 scope — expected in a later task.
- Publish token-safety audit scenarios in `docs/superpowers/tests/skills/gitflow-auth-test.md` once P2 testing ramps up.
