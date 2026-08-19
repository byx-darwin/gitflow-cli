# Design — Install Docs: Node.js & Skill-Source Prerequisites (Issue #192)

- **Workflow**: `wf-2026-08-19-001`
- **Mode**: fast
- **Issue**: [#192](https://github.com/byx-darwin/gitflow-cli/issues/192) — `[Feature]: 安装文档可以更具体包括node和claude的安装`
- **Date**: 2026-08-19

## Problem

A new user followed the docs and hit two undocumented walls in sequence:

1. `gf skills install` hard-blocked with `⛔ 未检测到任何技能来源，gf-workflow 无法运行` — the docs never told them a **skill source** (superpowers / mattpocock-skills) must be installed first.
2. Trying to fix that with `claude plugins install mattpocock-skills`, the install failed because their **Node.js version was too low** — a requirement documented nowhere.

The install docs (`README.md`, `website/src/pages/quickstart.mdx`) jump straight from "install `gf`" to "run `gf skills install`", with **no prerequisites section** covering Node.js, Claude Code, or the skill source. The one doc that mentions skill sources (`docs/gf-workflow-guide.md`) still omits the Node.js requirement.

## Verified Facts (authoritative)

| Requirement | Value | Source |
|-------------|-------|--------|
| `skills` CLI (`npx skills@latest add mattpocock/skills`) | Node.js **≥ 22.20.0** | npm `engines` field (`npm view skills engines`) |
| `mattpocock-skills` via `claude plugins install` | Node.js **≥ 22.20.0** | Issue reporter's observed `node version too low` error; the plugin pulls the `mattpocock/skills` tooling whose `skills` CLI declares `engines.node ≥ 22.20.0` |
| `superpowers` via `claude plugins install` | **No Node.js** (pure markdown skills, no bundled tooling) | superpowers plugin ships skills only |
| Claude Code — native install (recommended) | **No Node.js needed** (ships a native binary) | code.claude.com/docs/en/setup |
| Claude Code — npm install path | Node.js **22+** (as of v2.1.198) | code.claude.com/docs/en/setup |
| Claude Code OS | macOS 13+, Win10 1809+, Ubuntu 20.04+, Debian 10+, Alpine 3.19+; 4 GB+ RAM | code.claude.com/docs/en/setup |

**Node requirement is source-specific, not universal.** The two `mattpocock/skills` install paths (`claude plugins install mattpocock-skills` and `npx skills@latest`) both exercise the `skills` CLI and require **Node.js ≥ 22.20.0** — this matches the issue reporter's `claude plugins install mattpocock-skills` failure. `superpowers` is pure skills and needs no Node. Claude Code itself needs no Node when installed natively (recommended). Docs must therefore attach the Node floor to the `mattpocock-skills` source specifically, not to "any skill source".

## Current State (code + docs)

- Hard-block error: `apps/cli/src/commands/skills.rs:370-382` (`check_skill_source_at`) lists the three install commands but gives **no Node.js hint**.
- Gate covered by test `test_install_check_blocks_when_no_skill_source` (`skills.rs:2056-2065`), asserting the error contains `技能来源缺失`.
- `README.md:12-34` — install (brew/cargo) + "30 秒上手" (`gf skills install`); no prerequisites.
- `website/src/pages/quickstart.mdx:11-26` — install `gf` + `gf skills install`; no prerequisites.
- `docs/gf-workflow-guide.md:28-68` — mentions skill sources & 前置条件, but no Node.js version.

## Scope (confirmed with user)

**Docs + error-message hint** — both.

### 1. Error message (Rust — TDD + code review required)

Add one actionable line to the hard-block error in `check_skill_source_at` so users who hit the block learn the Node.js requirement inline, before they try (and fail) to install a source:

```text
⛔ 未检测到任何技能来源，gf-workflow 无法运行。请先安装其一：
  · claude plugins install superpowers
  · claude plugins install mattpocock-skills
  · npx skills@latest add mattpocock/skills
提示：安装 mattpocock-skills / npx skills 需要 Node.js ≥ 22.20.0（先运行 node --version 确认）
```

- The final `Err(...)` value (`技能来源缺失，安装中止`) is unchanged, so the existing test's `技能来源缺失` assertion still holds.
- Add a new assertion (or extend the test) verifying the Node hint line is present, so the hint is regression-protected.

### 2. Docs (Markdown — no Cargo gate)

Add a concise **前置条件 / Prerequisites** section to the two user-facing entry points, and fill the Node.js gap in the workflow guide:

- **`README.md`**: insert a "前置条件" block before/within 安装 covering: (a) Claude Code (native install recommended, link official setup), (b) a **skill source** — one of `superpowers` / `mattpocock-skills` — required before `gf skills install`, with the exact commands, (c) **Node.js ≥ 22.20.0** needed for the skill-source install paths. Clarify the ordering in "30 秒上手": install a skill source → `gf skills install` → `/gf-workflow`.
- **`website/src/pages/quickstart.mdx`**: mirror the same prerequisites (Node.js version, skill source, Claude Code) so the website matches the README.
- **`docs/gf-workflow-guide.md`**: add the **Node.js ≥ 22.20.0** note to the existing skill-source / 前置条件 section.

## Non-Goals

- No change to skill-source detection logic or the enum (`SkillSourceKind`).
- No new hard runtime check for Node.js version inside `gf` (out of scope; the error text hint is sufficient).
- No template placeholder changes; keep `gf` branding.

## Acceptance Criteria

1. `README.md` documents, before the "30 秒上手" flow: Node.js ≥ 22.20.0, the skill-source requirement (with commands), and Claude Code install pointer.
2. `website/src/pages/quickstart.mdx` documents the same prerequisites.
3. `docs/gf-workflow-guide.md` states the Node.js ≥ 22.20.0 requirement in its skill-source/前置条件 section.
4. `gf skills install` hard-block error includes a Node.js ≥ 22.20.0 hint line; final error value unchanged.
5. A unit test asserts the Node hint appears in the error output; existing `技能来源缺失` assertion still passes.
6. `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`, and `cargo +nightly fmt` pass for the touched crate.

## Validation

- Rust: `cargo test -p gitflow-cli` (skills tests), clippy pedantic, fmt — required because `skills.rs` changes.
- Docs: proofread rendered Markdown; verify links; `make check-agent-sync` not required (no AGENTS/CLAUDE/skill edits). Website `.mdx` is prose-only (no build gate needed for content edit, but verify it parses if a website build target exists).
