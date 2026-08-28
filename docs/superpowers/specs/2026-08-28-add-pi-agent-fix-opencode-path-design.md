# Design: Add Pi Code Agent & Fix OpenCode Global Path

**Date:** 2026-08-28
**Issue:** TBD
**Status:** Draft

## Summary

Two changes to `AgentPlatform` in `apps/cli/src/commands/skills.rs`:

1. **Bug fix:** OpenCode global skills path uses `~/.opencode/skills/` but should be `~/.config/opencode/skills/` (XDG convention).
2. **New feature:** Add Pi Code Agent as a supported platform.

## Problem

### OpenCode Path Bug

OpenCode follows the XDG Base Directory specification. Its global config lives at `~/.config/opencode/`, not `~/.opencode/`. The current `skills_dir_name()` returns `.opencode/skills`, which is correct for project-level (`<repo>/.opencode/skills/`) but wrong for global (`~/.opencode/skills/` instead of `~/.config/opencode/skills/`).

Source: [OpenCode Skills docs](https://m.runoob.com/opencode/opencode-skills.html) — global path is `~/.config/opencode/skills/<name>/SKILL.md`.

### Pi Code Agent Missing

Pi Code Agent uses a non-standard nested directory structure (`~/.pi/agent/`), requiring explicit support in `AgentPlatform`.

## Design

### 1. OpenCode Global Path Fix

Add a `global_skills_dir_name()` method to `AgentPlatform`:

```rust
/// Returns the global (user-level) skills subdirectory name.
/// Defaults to `skills_dir_name()`; overridden for agents that use
/// XDG or other non-standard global paths.
#[must_use]
pub fn global_skills_dir_name(&self) -> &'static str {
    match self {
        AgentPlatform::OpenCode => ".config/opencode/skills",
        other => other.skills_dir_name(),
    }
}
```

Update `resolve_target_dir` to use `global_skills_dir_name()` instead of `skills_dir_name()` when `global == true`.

**Impact:** Only the global install path for OpenCode changes. Project-level path (`<repo>/.opencode/skills/`) remains unchanged.

### 2. Add Pi Code Agent

New enum variant `Pi` with non-standard nested paths:

| Method | Value |
|--------|-------|
| `skills_dir_name()` | `.pi/agent/skills` |
| `hooks_dir_name()` | `.pi/agent/hooks` |
| `settings_file_path()` | `.pi/agent/settings.json` |
| `global_skills_dir_name()` | `.pi/agent/skills` (default, same as project-level) |
| `supports_hooks()` | `false` |

CLI value: `pi` (auto-derived by clap `ValueEnum`).

### 3. Test Coverage Gaps

Add missing test assertions:

- `skills_dir_name`: Gemini, Copilot
- `resolve_global_target`: OpenCode (verify `~/.config/opencode/skills`), Copilot

## Files Changed

- `apps/cli/src/commands/skills.rs` — enum variant, method, match arms, tests

## Scope

- No changes to hook installation logic (Pi does not support hooks).
- No changes to skill source detection (`detect_skill_sources` — Claude-only).
- No changes to co-contribution system.
- No migration needed — existing OpenCode users who installed globally will need to re-run `gf skills install -g --agent opencode`.
