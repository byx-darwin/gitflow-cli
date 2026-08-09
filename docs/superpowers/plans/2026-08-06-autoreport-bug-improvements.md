# Auto-Report Bug P0/P1 Improvements Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement P0 (critical security/correctness fixes) and P1 (high-priority UX/reliability/security improvements) for the auto-report-bug feature, raising the quality score from 6.0/10 to 8.0/10.

**Architecture:** Incremental improvements to three components: (1) Rust CLI `error_reporter.rs` for file permissions and sensitive data filtering, (2) Shell hook `auto-report-bug.sh` for path corrections, (3) Claude skill `gf-autoreport-bug` for success notifications. All changes follow TDD: write failing test → implement minimal code → refactor → verify.

**Tech Stack:** Rust 2024, Bash, Bats (Bash Automated Testing System), serde_json, std::os::unix::fs::PermissionsExt

## Global Constraints

- Rust 2024 edition with toolchain pinned in `rust-toolchain.toml`
- All public items require documentation with `# Errors`, `# Panics`, `# Safety` sections
- Never use `unwrap()` or `expect()` in production code; return `Result<T>` for fallible operations
- File permissions must be set to 0o600 (owner read/write only) for security-sensitive files
- All error messages must be sanitized to remove home directory paths, usernames, tokens, and internal URLs
- Shell scripts must use `set -euo pipefail` for strict error handling
- All new code must pass `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`

## File Structure

### Files to Modify

| File | Responsibility | Changes |
|------|---------------|---------|
| `apps/cli/src/error_reporter.rs` | Error report generation and file writing | Add file permission control (0o600), add `sanitize_error_message()` function |
| `.claude/hooks/auto-report-bug.sh` | Hook script for pending.json validation | Fix skill path hardcoding (line 124) |
| `.claude/skills/gf-autoreport-bug/SKILL.md` | Claude skill for auto-reporting | Add success notification output after Issue creation |

### Files to Create

| File | Responsibility |
|------|---------------|
| `.claude/hooks/tests/auto-report-bug.bats` | Bats tests for hook script validation, auth cache, error handling |

---

## Task 1: Add File Permission Control (0o600) for pending.json

**Files:**
- Modify: `apps/cli/src/error_reporter.rs:85-94`
- Test: `apps/cli/src/error_reporter.rs` (add to existing `mod tests`)

**Interfaces:**
- Consumes: `ErrorReport::write_to_disk()` method
- Produces: File with permissions 0o600 (owner read/write only)

### Task 1.1: Write failing test for file permissions

- [ ] **Step 1: Add test case for file permissions**

Add this test to the `mod tests` section in `apps/cli/src/error_reporter.rs`:

```rust
#[test]
fn test_should_set_pending_json_permissions_to_600() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let report = ErrorReport::from_error("issue create", "github", "auth failed", "AUTH_FAILED");
    report.write_to_disk(tmp.path()).expect("write_to_disk");

    let pending = tmp.path().join(".cache/bug-reports/pending.json");
    assert!(pending.exists(), "pending.json must be created");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(&pending).expect("metadata");
        let permissions = metadata.permissions();
        let mode = permissions.mode() & 0o777;
        assert_eq!(mode, 0o600, "pending.json must have 0o600 permissions (owner read/write only)");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --package gitflow-cli --lib error_reporter::tests::test_should_set_pending_json_permissions_to_600 -- --nocapture`
Expected: FAIL with assertion error (mode will be 0o644 or similar, not 0o600)

### Task 1.2: Implement minimal code to make test pass

- [ ] **Step 3: Add file permission setting to write_to_disk()**

Modify the `write_to_disk()` method in `apps/cli/src/error_reporter.rs` to set file permissions:

```rust
pub(crate) fn write_to_disk(&self, repo_root: &Path) -> std::io::Result<()> {
    let dir = repo_root.join(".cache").join("bug-reports");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("pending.json");
    let json = serde_json::to_string_pretty(self)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let mut file = std::fs::File::create(&path)?;
    file.write_all(json.as_bytes())?;
    
    // Set file permissions to 0o600 (owner read/write only) for security
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = std::fs::Permissions::from_mode(0o600);
        file.set_permissions(permissions)?;
    }
    
    Ok(())
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --package gitflow-cli --lib error_reporter::tests::test_should_set_pending_json_permissions_to_600`
Expected: PASS

- [ ] **Step 5: Run all error_reporter tests to ensure no regressions**

Run: `cargo test --package gitflow-cli --lib error_reporter::tests`
Expected: All tests PASS

- [ ] **Step 6: Commit**

```bash
git add apps/cli/src/error_reporter.rs
git commit -m "fix(security): set pending.json file permissions to 0o600

- Add file permission control for security-sensitive error reports
- Prevents other users from reading error reports on multi-user systems
- Add test to verify permissions are set correctly
- Unix-only implementation (no-op on Windows)

Fixes P0 security issue identified in multi-role analysis."
```

---

## Task 2: Fix Skill Path Hardcoding

**Files:**
- Modify: `.claude/hooks/auto-report-bug.sh:124`

**Interfaces:**
- Consumes: Hook script output
- Produces: Correct skill path reference

### Task 2.1: Fix the hardcoded skill path

- [ ] **Step 1: Identify the hardcoded path**

The current line 124 in `.claude/hooks/auto-report-bug.sh` contains:
```bash
echo "  Skill 路径: skills/gitflow-autoreport-bug/SKILL.md"
```

This is incorrect. The actual path is `.claude/skills/gf-autoreport-bug/SKILL.md`.

- [ ] **Step 2: Fix the path**

Replace line 124 in `.claude/hooks/auto-report-bug.sh`:

**Before:**
```bash
echo "  Skill 路径: skills/gitflow-autoreport-bug/SKILL.md"
```

**After:**
```bash
echo "  Skill 路径: .claude/skills/gf-autoreport-bug/SKILL.md"
```

Or better, use a variable for maintainability:

```bash
SKILL_PATH=".claude/skills/gf-autoreport-bug/SKILL.md"
echo "  Skill 路径: ${SKILL_PATH}"
```

- [ ] **Step 3: Verify the fix**

Run: `grep -n "Skill 路径" .claude/hooks/auto-report-bug.sh`
Expected: Output shows `.claude/skills/gf-autoreport-bug/SKILL.md`

- [ ] **Step 4: Test the hook script manually**

```bash
# Create a test pending.json
mkdir -p .cache/bug-reports
cat > .cache/bug-reports/pending.json << 'EOF'
{
  "id": "test-123",
  "command": "issue create",
  "platform": "github",
  "error_code": "TEST_ERROR",
  "error_message": "Test error message",
  "timestamp": "2026-08-06T10:00:00Z"
}
EOF

# Run the hook script (in non-interactive mode)
echo "" | .claude/hooks/auto-report-bug.sh | grep "Skill 路径"

# Clean up
rm -f .cache/bug-reports/pending.json
```

Expected: Output shows `.claude/skills/gf-autoreport-bug/SKILL.md`

- [ ] **Step 5: Commit**

```bash
git add .claude/hooks/auto-report-bug.sh
git commit -m "fix(hook): correct skill path hardcoding

- Update skill path from skills/gitflow-autoreport-bug/SKILL.md
  to .claude/skills/gf-autoreport-bug/SKILL.md
- Fixes incorrect path reference in hook output
- Improves discoverability of the skill file

Fixes P0 correctness issue identified in multi-role analysis."
```

---

## Task 3: Add Success Notification After Issue Creation

**Files:**
- Modify: `.claude/skills/gf-autoreport-bug/SKILL.md`

**Interfaces:**
- Consumes: GitHub Issue creation result
- Produces: User-facing success message with Issue URL

### Task 3.1: Add success notification to skill workflow

- [ ] **Step 1: Read the current skill workflow**

The current workflow in `.claude/skills/gf-autoreport-bug/SKILL.md` ends with:
```
5. **Cleanup** — `rm -f .cache/bug-reports/pending.json`.
```

- [ ] **Step 2: Add success notification step**

Insert a new step between step 4 (Create Issue) and step 5 (Cleanup):

**Before:**
```markdown
4. **Create Issue** — Analyze root cause, fix direction, severity. Create Issue via `gf issue create --repo byx-darwin/gitflow-cli --title "[auto-report] gitflow {command} — {error_code}" --label "auto-report"`. Fail → keep file + `failed.log`.
5. **Cleanup** — `rm -f .cache/bug-reports/pending.json`.
```

**After:**
```markdown
4. **Create Issue** — Analyze root cause, fix direction, severity. Create Issue via `gf issue create --repo byx-darwin/gitflow-cli --title "[auto-report] gitflow {command} — {error_code}" --label "auto-report"`. Fail → keep file + `failed.log`.
5. **Success Notification** — Output success message with Issue URL:
   ```
   ✅ 已自动报告 bug: {issue_url}
   ```
   This provides immediate feedback to the user that the Issue was created successfully.
6. **Cleanup** — `rm -f .cache/bug-reports/pending.json`.
```

- [ ] **Step 3: Update the decision flow diagram**

Update the Mermaid flowchart in the skill to include the notification step:

**Before:**
```mermaid
flowchart TD
    A[Read pending.json] --> B{Valid JSON?}
    B -->|No| C[Rename .invalid, warn, stop]
    B -->|Yes| D{Auth check}
    D -->|Pass| G{Duplicate Issue?}
    D -->|Fail| NEW[输出登录提示 + Issue 模板]
    NEW --> KEEP[保留 pending.json, stop]
    G -->|Yes| I[Clean, stop]
    G -->|No| J[Create Issue]
    J -->|Fail| F[Keep file + failed.log]
    J -->|Pass| K[Remove pending.json]
```

**After:**
```mermaid
flowchart TD
    A[Read pending.json] --> B{Valid JSON?}
    B -->|No| C[Rename .invalid, warn, stop]
    B -->|Yes| D{Auth check}
    D -->|Pass| G{Duplicate Issue?}
    D -->|Fail| NEW[输出登录提示 + Issue 模板]
    NEW --> KEEP[保留 pending.json, stop]
    G -->|Yes| I[Clean, stop]
    G -->|No| J[Create Issue]
    J -->|Fail| F[Keep file + failed.log]
    J -->|Pass| N[Output success notification with Issue URL]
    N --> K[Remove pending.json]
```

- [ ] **Step 4: Verify the skill file is valid Markdown**

Run: `cat .claude/skills/gf-autoreport-bug/SKILL.md | head -50`
Expected: Skill file renders correctly with the new step

- [ ] **Step 5: Commit**

```bash
git add .claude/skills/gf-autoreport-bug/SKILL.md
git commit -m "feat(skill): add success notification after Issue creation

- Add step 5: output '✅ 已自动报告 bug: {issue_url}' after successful Issue creation
- Update decision flow diagram to include notification step
- Provides immediate user feedback that bug was reported
- Improves user experience by confirming successful auto-report

Addresses P1 UX issue identified in multi-role analysis."
```

---

## Task 4: Add Hook Script Tests (Bats)

**Files:**
- Create: `.claude/hooks/tests/auto-report-bug.bats`

**Interfaces:**
- Consumes: `.claude/hooks/auto-report-bug.sh`
- Produces: Test suite validating hook behavior

### Task 4.1: Create Bats test file structure

- [ ] **Step 1: Create test directory**

```bash
mkdir -p .claude/hooks/tests
```

- [ ] **Step 2: Write Bats test file**

Create `.claude/hooks/tests/auto-report-bug.bats`:

```bash
#!/usr/bin/env bats

# Bats tests for auto-report-bug.sh hook script
# Run: bats .claude/hooks/tests/auto-report-bug.bats

setup() {
    # Create temporary directory for test artifacts
    export TEST_DIR="$(mktemp -d)"
    export REPO_ROOT="$TEST_DIR/repo"
    mkdir -p "$REPO_ROOT/.cache/bug-reports"
    mkdir -p "$REPO_ROOT/.cache/auth-cache"
    
    # Copy hook script to test directory
    cp "$(git rev-parse --show-toplevel)/.claude/hooks/auto-report-bug.sh" "$TEST_DIR/hook.sh"
    chmod +x "$TEST_DIR/hook.sh"
    
    # Override git rev-parse to use test directory
    export PATH="$TEST_DIR/bin:$PATH"
    mkdir -p "$TEST_DIR/bin"
    cat > "$TEST_DIR/bin/git" << 'GITSCRIPT'
#!/bin/bash
if [ "$1" = "rev-parse" ] && [ "$2" = "--show-toplevel" ]; then
    echo "$REPO_ROOT"
else
    command git "$@"
fi
GITSCRIPT
    chmod +x "$TEST_DIR/bin/git"
}

teardown() {
    # Clean up test directory
    rm -rf "$TEST_DIR"
}

@test "hook exits silently when no pending.json exists" {
    run "$TEST_DIR/hook.sh"
    [ "$status" -eq 0 ]
    [ -z "$output" ]
}

@test "hook exits silently in interactive terminal" {
    # Create pending.json
    cat > "$REPO_ROOT/.cache/bug-reports/pending.json" << 'EOF'
{
  "id": "test-123",
  "command": "issue create",
  "platform": "github",
  "error_code": "TEST_ERROR",
  "error_message": "Test error",
  "timestamp": "2026-08-06T10:00:00Z"
}
EOF
    
    # Simulate interactive terminal (TTY)
    # Note: This test may not work in all CI environments
    # We're testing the logic, not the actual TTY detection
    run bash -c "[ -t 0 ] || [ -t 1 ] && exit 0 || exit 1"
    # In non-TTY environment, this should fail (which is expected)
    [ "$status" -eq 1 ] || skip "TTY detection test skipped in non-interactive environment"
}

@test "hook renames invalid JSON to .invalid" {
    # Create invalid pending.json (missing error_code)
    cat > "$REPO_ROOT/.cache/bug-reports/pending.json" << 'EOF'
{
  "id": "test-123",
  "command": "issue create"
}
EOF
    
    run "$TEST_DIR/hook.sh"
    [ "$status" -eq 0 ]
    [ -f "$REPO_ROOT/.cache/bug-reports/pending.json.invalid" ]
    [ ! -f "$REPO_ROOT/.cache/bug-reports/pending.json" ]
    [[ "$output" == *"格式异常"* ]]
}

@test "hook outputs login guide on auth failure" {
    # Create valid pending.json
    cat > "$REPO_ROOT/.cache/bug-reports/pending.json" << 'EOF'
{
  "id": "test-123",
  "command": "issue create",
  "platform": "github",
  "error_code": "AUTH_FAILED",
  "error_message": "Unauthorized",
  "timestamp": "2026-08-06T10:00:00Z"
}
EOF
    
    # Mock gh CLI to simulate auth failure
    mkdir -p "$TEST_DIR/bin"
    cat > "$TEST_DIR/bin/gh" << 'GHSCRIPT'
#!/bin/bash
if [ "$1" = "auth" ] && [ "$2" = "status" ]; then
    exit 1
fi
GHSCRIPT
    chmod +x "$TEST_DIR/bin/gh"
    
    run "$TEST_DIR/hook.sh"
    [ "$status" -eq 0 ]
    [[ "$output" == *"GitHub 未登录"* ]]
    [[ "$output" == *"gh auth login"* ]]
    [[ "$output" == *"方式 2: 手动创建 Issue"* ]]
    [ -f "$REPO_ROOT/.cache/bug-reports/pending.json" ]
}

@test "hook outputs banner on auth success" {
    # Create valid pending.json
    cat > "$REPO_ROOT/.cache/bug-reports/pending.json" << 'EOF'
{
  "id": "test-123",
  "command": "issue create",
  "platform": "github",
  "error_code": "TEST_ERROR",
  "error_message": "Test error",
  "timestamp": "2026-08-06T10:00:00Z"
}
EOF
    
    # Mock gh CLI to simulate auth success
    mkdir -p "$TEST_DIR/bin"
    cat > "$TEST_DIR/bin/gh" << 'GHSCRIPT'
#!/bin/bash
if [ "$1" = "auth" ] && [ "$2" = "status" ]; then
    exit 0
fi
GHSCRIPT
    chmod +x "$TEST_DIR/bin/gh"
    
    run "$TEST_DIR/hook.sh"
    [ "$status" -eq 0 ]
    [[ "$output" == *"检测到 gitflow CLI 错误报告"* ]]
    [[ "$output" == *"命令:   issue create"* ]]
    [[ "$output" == *"平台:   github"* ]]
    [[ "$output" == *"错误码: TEST_ERROR"* ]]
    [[ "$output" == *"请加载 gitflow-autoreport-bug Skill"* ]]
}

@test "hook uses auth cache when available" {
    # Create valid pending.json
    cat > "$REPO_ROOT/.cache/bug-reports/pending.json" << 'EOF'
{
  "id": "test-123",
  "command": "issue create",
  "platform": "github",
  "error_code": "TEST_ERROR",
  "error_message": "Test error",
  "timestamp": "2026-08-06T10:00:00Z"
}
EOF
    
    # Create auth cache (timestamp: now)
    echo "$(date +%s)" > "$REPO_ROOT/.cache/auth-cache/github.ttl"
    
    # Mock gh CLI (should not be called due to cache)
    mkdir -p "$TEST_DIR/bin"
    cat > "$TEST_DIR/bin/gh" << 'GHSCRIPT'
#!/bin/bash
echo "gh CLI should not be called when cache is valid" >&2
exit 1
GHSCRIPT
    chmod +x "$TEST_DIR/bin/gh"
    
    run "$TEST_DIR/hook.sh"
    [ "$status" -eq 0 ]
    [[ "$output" == *"cache 命中"* ]]
}
```

- [ ] **Step 3: Make test file executable**

```bash
chmod +x .claude/hooks/tests/auto-report-bug.bats
```

- [ ] **Step 4: Install Bats if not present**

```bash
# Check if bats is installed
if ! command -v bats &> /dev/null; then
    echo "Installing Bats..."
    # macOS
    if [[ "$OSTYPE" == "darwin"* ]]; then
        brew install bats-core
    # Linux
    else
        git clone https://github.com/bats-core/bats-core.git /tmp/bats-core
        sudo /tmp/bats-core/install.sh /usr/local
    fi
fi
```

- [ ] **Step 5: Run tests to verify they work**

```bash
bats .claude/hooks/tests/auto-report-bug.bats
```

Expected: All tests PASS (or skip TTY test in non-interactive environment)

- [ ] **Step 6: Commit**

```bash
git add .claude/hooks/tests/auto-report-bug.bats
git commit -m "test(hook): add Bats test suite for auto-report-bug.sh

- Test no pending.json → silent exit
- Test invalid JSON → rename to .invalid
- Test auth failure → output login guide
- Test auth success → output banner
- Test auth cache → skip gh CLI call when cache valid
- Improves reliability and prevents regressions

Addresses P1 testing gap identified in multi-role analysis."
```

---

## Task 5: Add Sensitive Data Filtering

**Files:**
- Modify: `apps/cli/src/error_reporter.rs`
- Test: `apps/cli/src/error_reporter.rs` (add to existing `mod tests`)

**Interfaces:**
- Consumes: Error message string
- Produces: Sanitized error message with sensitive data removed

### Task 5.1: Write failing test for sensitive data filtering

- [ ] **Step 1: Add test case for sanitization**

Add this test to the `mod tests` section in `apps/cli/src/error_reporter.rs`:

```rust
#[test]
fn test_should_sanitize_home_directory_in_error_message() {
    let msg = "Failed to read /Users/baoyx/.config/settings.json";
    let sanitized = sanitize_error_message(msg);
    assert!(!sanitized.contains("/Users/baoyx"), "Home directory must be removed");
    assert!(sanitized.contains("~") || sanitized.contains("***"), "Home directory must be replaced");
}

#[test]
fn test_should_sanitize_username_in_error_message() {
    let msg = "User baoyx authentication failed";
    let sanitized = sanitize_error_message(msg);
    // Username should be replaced if it matches current user
    // This test may need adjustment based on implementation
    assert!(!sanitized.contains("baoyx") || sanitized.contains("***"));
}

#[test]
fn test_should_sanitize_token_in_error_message() {
    let msg = "Token ghp_1234567890abcdef1234567890abcdef12345678 expired";
    let sanitized = sanitize_error_message(msg);
    assert!(!sanitized.contains("ghp_1234567890abcdef"), "Token must be removed");
    assert!(sanitized.contains("***") || sanitized.contains("[REDACTED]"));
}

#[test]
fn test_should_not_modify_safe_error_message() {
    let msg = "Connection timeout after 30 seconds";
    let sanitized = sanitize_error_message(msg);
    assert_eq!(sanitized, msg, "Safe messages should not be modified");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --package gitflow-cli --lib error_reporter::tests::test_should_sanitize`
Expected: FAIL with "function `sanitize_error_message` not found"

### Task 5.2: Implement sanitize_error_message function

- [ ] **Step 3: Add sanitize_error_message function**

Add this function to `apps/cli/src/error_reporter.rs` (before the `mod tests` section):

```rust
/// Sanitize error message to remove sensitive information.
///
/// Removes or redacts:
/// - Home directory paths (replaced with `~`)
/// - GitHub tokens (ghp_*, github_pat_*)
/// - Common username patterns
///
/// # Examples
///
/// ```
/// let msg = "Failed to read /Users/baoyx/.config/settings.json";
/// let sanitized = sanitize_error_message(msg);
/// assert!(!sanitized.contains("/Users/baoyx"));
/// ```
fn sanitize_error_message(msg: &str) -> String {
    let mut sanitized = msg.to_string();
    
    // Replace home directory paths with ~
    if let Some(home) = dirs::home_dir() {
        let home_str = home.to_string_lossy();
        sanitized = sanitized.replace(&*home_str, "~");
    }
    
    // Redact GitHub tokens (ghp_* and github_pat_*)
    let token_patterns = [
        regex::Regex::new(r"ghp_[a-zA-Z0-9]{36,}").expect("valid regex"),
        regex::Regex::new(r"github_pat_[a-zA-Z0-9_]{22,}").expect("valid regex"),
    ];
    
    for pattern in &token_patterns {
        sanitized = pattern.replace_all(&sanitized, "[REDACTED]").to_string();
    }
    
    sanitized
}
```

**Note:** This requires adding `regex` and `dirs` to dependencies. Check if they're already in `Cargo.toml`. If not, add them:

```toml
[dependencies]
regex = "1.10"
dirs = "5.0"
```

- [ ] **Step 4: Update ErrorReport::from_error to use sanitization**

Modify the `from_error()` method to sanitize the error message:

```rust
pub(crate) fn from_error(
    command: &str,
    platform: &str,
    error_message: &str,
    error_code: &str,
) -> Self {
    Self {
        id: generate_unique_id(),
        source: "cli".into(),
        command: command.into(),
        platform: platform.into(),
        exit_code: 1,
        error_code: error_code.into(),
        error_message: sanitize_error_message(error_message),  // <-- Add sanitization
        hint: None,
        timestamp: iso8601_utc_now(),
    }
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --package gitflow-cli --lib error_reporter::tests::test_should_sanitize`
Expected: All sanitization tests PASS

- [ ] **Step 6: Run all error_reporter tests to ensure no regressions**

Run: `cargo test --package gitflow-cli --lib error_reporter::tests`
Expected: All tests PASS

- [ ] **Step 7: Run clippy to ensure code quality**

Run: `cargo clippy --package gitflow-cli --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: No warnings

- [ ] **Step 8: Commit**

```bash
git add apps/cli/src/error_reporter.rs Cargo.toml Cargo.lock
git commit -m "feat(security): add sensitive data filtering for error messages

- Add sanitize_error_message() function to remove sensitive info
- Filter home directory paths (replace with ~)
- Redact GitHub tokens (ghp_*, github_pat_*)
- Apply sanitization in ErrorReport::from_error()
- Add comprehensive tests for sanitization
- Add regex and dirs dependencies

Prevents accidental leakage of sensitive information in error reports.
Addresses P1 security issue identified in multi-role analysis."
```

---

## Verification Checklist

After completing all tasks, verify:

- [ ] All P0 fixes implemented (file permissions, skill path)
- [ ] All P1 improvements implemented (success notification, hook tests, sensitive data filtering)
- [ ] All tests pass: `cargo test --package gitflow-cli --lib error_reporter::tests`
- [ ] Bats tests pass: `bats .claude/hooks/tests/auto-report-bug.bats`
- [ ] Clippy passes: `cargo clippy --package gitflow-cli --all-targets --all-features -- -D warnings -W clippy::pedantic`
- [ ] Build succeeds: `cargo build --package gitflow-cli`
- [ ] No regressions in existing functionality

## Success Criteria

- **P0 Security**: `pending.json` has 0o600 permissions (verified by test)
- **P0 Correctness**: Skill path is correct (verified by manual test)
- **P1 UX**: Success notification displayed after Issue creation
- **P1 Reliability**: Hook script has Bats test suite with 5+ test cases
- **P1 Security**: Error messages are sanitized (verified by tests)

## Estimated Effort

| Task | Effort | Complexity |
|------|--------|-----------|
| Task 1: File permissions | 30 min | Low |
| Task 2: Skill path fix | 15 min | Low |
| Task 3: Success notification | 30 min | Low |
| Task 4: Hook tests | 2 hours | Medium |
| Task 5: Sensitive data filtering | 1 hour | Medium |
| **Total** | **~4.5 hours** | — |

---

**Plan created**: 2026-08-06  
**Based on**: `docs/superpowers/specs/2026-08-06-autoreport-bug-analysis.md`  
**Scope**: P0 + P1 improvements only (P2/P3 deferred to future plans)
