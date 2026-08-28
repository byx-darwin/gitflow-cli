# Pi Code Agent & OpenCode Path Fix — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Pi Code Agent platform to `gf skills install --agent` and fix OpenCode's global install path to follow XDG convention.

**Architecture:** All changes in a single file `apps/cli/src/commands/skills.rs`. Add `Pi` variant to `AgentPlatform` enum with non-standard nested paths (`.pi/agent/*`). Add `global_skills_dir_name()` method that defaults to `skills_dir_name()` but overrides OpenCode to `.config/opencode/skills`. Update `resolve_target_dir` to use the new method for global installs.

**Tech Stack:** Rust, clap (ValueEnum derive), miette (error handling), dirs crate

---

### Task 1: Add failing tests for Pi Code Agent

**Files:**
- Modify: `apps/cli/src/commands/skills.rs` (test module, ~line 1235+)

- [ ] **Step 1: Add Pi platform tests**

Add these tests after the existing Qoder tests (after line 1347 for settings, after line 1262 for dir, etc.):

```rust
    #[test]
    fn test_agent_platform_pi_dir() {
        assert_eq!(
            AgentPlatform::Pi.skills_dir_name(),
            ".pi/agent/skills"
        );
    }

    #[test]
    fn test_agent_platform_pi_hooks_dir() {
        assert_eq!(
            AgentPlatform::Pi.hooks_dir_name(),
            ".pi/agent/hooks"
        );
    }

    #[test]
    fn test_agent_platform_pi_settings_path() {
        assert_eq!(
            AgentPlatform::Pi.settings_file_path(),
            ".pi/agent/settings.json"
        );
    }

    #[test]
    fn test_agent_platform_pi_global_skills_dir() {
        // Pi uses same path for global and project-level
        assert_eq!(
            AgentPlatform::Pi.global_skills_dir_name(),
            ".pi/agent/skills"
        );
    }
```

Also add Pi to the `supports_hooks_matrix` test (after line 1398):

```rust
        assert!(!AgentPlatform::Pi.supports_hooks());
```

And add a global resolve test:

```rust
    #[test]
    fn test_resolve_global_target_pi() {
        let dir = resolve_target_dir(true, Some(AgentPlatform::Pi), None).expect("resolve");
        assert!(dir.ends_with(".pi/agent/skills"));
    }
```

- [ ] **Step 2: Verify tests fail to compile**

Run: `cargo test --package gitflow-cli --lib -- commands::skills::tests 2>&1 | tail -5`
Expected: compile error — `AgentPlatform::Pi` does not exist

- [ ] **Step 3: Commit failing tests**

```bash
git add apps/cli/src/commands/skills.rs
git commit -m "test(skills): add failing tests for Pi Code Agent platform"
```

---

### Task 2: Add Pi variant to AgentPlatform enum

**Files:**
- Modify: `apps/cli/src/commands/skills.rs:44-117`

- [ ] **Step 1: Add Pi to enum definition**

After `Qoder` variant (line 57), add:

```rust
    /// Pi Code Agent — `~/.pi/agent/skills/`
    Pi,
```

- [ ] **Step 2: Add Pi to `skills_dir_name()`**

After `Qoder` arm (line 70), add:

```rust
            AgentPlatform::Pi => ".pi/agent/skills",
```

- [ ] **Step 3: Add Pi to `hooks_dir_name()`**

After `Qoder` arm (line 83), add:

```rust
            AgentPlatform::Pi => ".pi/agent/hooks",
```

- [ ] **Step 4: Add Pi to `settings_file_path()`**

After `Qoder` arm (line 96), add:

```rust
            AgentPlatform::Pi => ".pi/agent/settings.json",
```

- [ ] **Step 5: Add `global_skills_dir_name()` method**

Add this method after `supports_hooks()` (after line 107), before `detect()`:

```rust
    /// 返回该 Agent 的全局（用户级）skills 子目录名。
    ///
    /// 默认与 `skills_dir_name()` 相同；仅当 Agent 的全局路径与项目级路径不同
    /// 时才覆写（如 OpenCode 遵循 XDG 规范，全局配置在 `~/.config/opencode/`）。
    #[must_use]
    pub fn global_skills_dir_name(self) -> &'static str {
        match self {
            AgentPlatform::OpenCode => ".config/opencode/skills",
            other => other.skills_dir_name(),
        }
    }
```

- [ ] **Step 6: Run tests to verify Pi tests pass**

Run: `cargo test --package gitflow-cli --lib -- commands::skills::tests::test_agent_platform_pi 2>&1 | tail -10`
Expected: all Pi tests PASS

- [ ] **Step 7: Commit**

```bash
git add apps/cli/src/commands/skills.rs
git commit -m "feat(skills): add Pi Code Agent to AgentPlatform enum"
```

---

### Task 3: Fix OpenCode global path in `resolve_target_dir`

**Files:**
- Modify: `apps/cli/src/commands/skills.rs:233-235`

- [ ] **Step 1: Add failing test for OpenCode global path**

Add test after existing `test_resolve_global_target_qoder`:

```rust
    #[test]
    fn test_resolve_global_target_opencode_xdg() {
        let dir = resolve_target_dir(true, Some(AgentPlatform::OpenCode), None).expect("resolve");
        assert!(
            dir.ends_with(".config/opencode/skills"),
            "OpenCode global must follow XDG: got {}",
            dir.display()
        );
    }
```

- [ ] **Step 2: Verify test fails**

Run: `cargo test --package gitflow-cli --lib -- commands::skills::tests::test_resolve_global_target_opencode_xdg 2>&1 | tail -10`
Expected: FAIL — path ends with `.opencode/skills` not `.config/opencode/skills`

- [ ] **Step 3: Update `resolve_target_dir` to use `global_skills_dir_name()`**

Change line 235 from:

```rust
        Ok(home.join(platform.skills_dir_name()))
```

to:

```rust
        Ok(home.join(platform.global_skills_dir_name()))
```

- [ ] **Step 4: Verify test passes**

Run: `cargo test --package gitflow-cli --lib -- commands::skills::tests::test_resolve_global_target_opencode_xdg 2>&1 | tail -5`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add apps/cli/src/commands/skills.rs
git commit -m "fix(skills): use XDG path for OpenCode global install"
```

---

### Task 4: Fill test coverage gaps

**Files:**
- Modify: `apps/cli/src/commands/skills.rs` (test module)

- [ ] **Step 1: Add missing `skills_dir_name` tests for Gemini and Copilot**

```rust
    #[test]
    fn test_agent_platform_gemini_dir() {
        assert_eq!(
            AgentPlatform::Gemini.skills_dir_name(),
            ".gemini/skills"
        );
    }

    #[test]
    fn test_agent_platform_copilot_dir() {
        assert_eq!(
            AgentPlatform::Copilot.skills_dir_name(),
            ".copilot/skills"
        );
    }
```

- [ ] **Step 2: Add missing `resolve_global_target` tests for OpenCode and Copilot**

OpenCode is already covered in Task 3. Add Copilot:

```rust
    #[test]
    fn test_resolve_global_target_copilot() {
        let dir = resolve_target_dir(true, Some(AgentPlatform::Copilot), None).expect("resolve");
        assert!(dir.ends_with(".copilot/skills"));
    }
```

- [ ] **Step 3: Run full test suite**

Run: `cargo test --package gitflow-cli --lib -- commands::skills::tests 2>&1 | tail -15`
Expected: all tests PASS, no regressions

- [ ] **Step 4: Commit**

```bash
git add apps/cli/src/commands/skills.rs
git commit -m "test(skills): fill coverage gaps for Gemini, Copilot, OpenCode"
```

---

### Task 5: Final verification

- [ ] **Step 1: Run full crate test suite**

Run: `cargo test --workspace 2>&1 | tail -20`
Expected: all tests pass

- [ ] **Step 2: Run clippy**

Run: `cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -10`
Expected: no warnings

- [ ] **Step 3: Verify CLI help shows Pi**

Run: `cargo run -- skills install --help 2>&1 | grep -A 20 "agent"`
Expected: `pi` appears in the agent value list
