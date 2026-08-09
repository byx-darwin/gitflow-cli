# fix(release): release.toml 模板占位符修复 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix `release.toml` template placeholders from `{version}` to `{{version}}` (cargo-release 1.1.3 correct syntax), and enhance `scripts/release.sh` to detect both single-brace and double-brace template residue.

**Architecture:** Two-file fix — update `release.toml` to use correct Mustache-style `{{version}}` syntax, then expand `scripts/release.sh` template residue detection regex to catch both `{var}` and `{{var}}` patterns. Add self-test cases for single-brace residue. Verify end-to-end with `make release-rehearse`.

**Tech Stack:** cargo-release 1.1.3, bash, regex (ERE)

## Global Constraints

- cargo-release 1.1.3 uses `{{version}}` (double curly braces, Mustache-style) for template variables
- Template residue detection must catch BOTH `{var}` (single brace) AND `{{var}}` (double braces)
- All existing self-test cases must continue to pass
- `make release-rehearse` must succeed after the fix
- No changes to Rust source code — this is a config/script-only fix

---

### Task 1: Fix release.toml Template Syntax

**Files:**
- Modify: `release.toml`

**Interfaces:**
- Consumes: None (standalone config fix)
- Produces: Correct `{{version}}` syntax in `tag-name`, `tag-message`, `pre-release-commit-message`

- [ ] **Step 1: Update release.toml template syntax**

Replace all `{version}` with `{{version}}` in `release.toml`:

```toml
# cargo-release workspace configuration
# See: https://github.com/crate-ci/cargo-release/blob/master/docs/reference.md

# Single version tag for the entire workspace
shared-version = true
tag-name = "v{{version}}"
tag-message = "Release v{{version}}"

# Publish to crates.io
publish = true

# Allow release only from main branch
allow-branch = ["main"]

# Pre-release commit message (when versions are bumped)
pre-release-commit-message = "chore: release v{{version}}"

# Pre-release verification
verify = true

# Sign tags (optional, uncomment if needed)
# sign-tag = true

# Sign commits (optional, uncomment if needed)
# sign-commit = true
```

- [ ] **Step 2: Verify template syntax with dry-run**

Run: `cargo release --dry-run patch 2>&1 | head -20`
Expected: No errors about template syntax. Output should show version bump preview.

- [ ] **Step 3: Commit the fix**

```bash
git add release.toml
git commit -m "fix(release): use {{version}} template syntax for cargo-release 1.1.3 (#132)"
```

---

### Task 2: Enhance Template Residue Detection in scripts/release.sh

**Files:**
- Modify: `scripts/release.sh:80` (TEMPLATE_RESIDUE_PATTERN)
- Modify: `scripts/release.sh:122-178` (run_self_test function — add new test cases)

**Interfaces:**
- Consumes: `release.toml` with correct `{{version}}` syntax (from Task 1)
- Produces: Expanded `TEMPLATE_RESIDUE_PATTERN` regex, additional self-test cases

- [ ] **Step 1: Write failing self-test for single-brace residue**

Add new test cases to `run_self_test()` function in `scripts/release.sh` (after line 150):

```bash
    # NEW: single-brace residue detection
    expect_fail "commit subject: single-brace residue" validate_commit_subject "chore: release v{version}"
    expect_fail "tag: single-brace residue" validate_tag_name "v{version}"
```

Insert these lines after the existing `expect_fail "commit subject: template residue"` line (line 150) and after `expect_fail "tag: template residue"` line (line 155).

- [ ] **Step 2: Run self-test to verify it fails**

Run: `bash scripts/release.sh --self-test`
Expected: FAIL — the two new test cases fail because `TEMPLATE_RESIDUE_PATTERN` only matches `{{var}}`, not `{var}`.

Output should show:
```
✗ commit subject: single-brace residue (expected fail, got pass)
✗ tag: single-brace residue (expected fail, got pass)
```

- [ ] **Step 3: Update TEMPLATE_RESIDUE_PATTERN to match both single and double braces**

Change line 80 in `scripts/release.sh` from:

```bash
TEMPLATE_RESIDUE_PATTERN='\{\{[a-zA-Z_]+\}\}'
```

to:

```bash
TEMPLATE_RESIDUE_PATTERN='\{\{?[a-zA-Z_]+\}\}?'
```

This regex now matches:
- `{version}` (single brace) ✅
- `{{version}}` (double braces) ✅

- [ ] **Step 4: Run self-test to verify it passes**

Run: `bash scripts/release.sh --self-test`
Expected: PASS — all test cases pass, including the new single-brace residue tests.

Output should show:
```
✓ commit subject: single-brace residue
✓ tag: single-brace residue
✓ Self-test passed
```

- [ ] **Step 5: Enhance error messages for template residue**

Update the `validate_commit_subject` function (line 86-97) to provide more helpful error messages:

```bash
validate_commit_subject() {
    local subject="$1"
    if [[ "$subject" =~ $TEMPLATE_RESIDUE_PATTERN ]]; then
        log_error "Template residue in commit subject: $subject"
        log_error "Expected: chore: release v1.2.3"
        log_error "This usually means release.toml uses incorrect placeholder syntax."
        log_error "For cargo-release 1.1.3+, use {{version}} (double curly braces)."
        return 1
    fi
    if [[ ! "$subject" =~ $RELEASE_COMMIT_PATTERN ]]; then
        log_error "Malformed release commit subject: $subject"
        return 1
    fi
    return 0
}
```

Update the `validate_tag_name` function (line 99-110) similarly:

```bash
validate_tag_name() {
    local tag="$1"
    if [[ "$tag" =~ $TEMPLATE_RESIDUE_PATTERN ]]; then
        log_error "Template residue in tag name: $tag"
        log_error "Expected: v1.2.3"
        log_error "This usually means release.toml uses incorrect placeholder syntax."
        log_error "For cargo-release 1.1.3+, use {{version}} (double curly braces)."
        return 1
    fi
    if [[ ! "$tag" =~ $VERSION_TAG_PATTERN ]]; then
        log_error "Malformed tag name: $tag"
        return 1
    fi
    return 0
}
```

- [ ] **Step 6: Run self-test again to verify enhanced messages don't break anything**

Run: `bash scripts/release.sh --self-test`
Expected: PASS — all test cases still pass.

- [ ] **Step 7: Commit the enhancement**

```bash
git add scripts/release.sh
git commit -m "fix(release): expand template residue detection to catch single-brace syntax (#132)"
```

---

### Task 3: End-to-End Verification

**Files:**
- No new file changes (verification only)

**Interfaces:**
- Consumes: Fixed `release.toml` (Task 1), enhanced `scripts/release.sh` (Task 2)
- Produces: Verification that the full release workflow works correctly

- [ ] **Step 1: Run release rehearsal**

Run: `make release-rehearse`
Expected:
- Prerequisites check passes
- Preflight checks pass (on main, clean working dir, tests pass, clippy passes)
- Version preview shows correct next version
- Changelog preview generates successfully
- `cargo release` dry-run succeeds
- No template residue detected in dry-run output
- Self-test passes
- Rehearsal report shows all green checkmarks

- [ ] **Step 2: Run full test suite**

Run: `make test`
Expected: All tests pass. No regressions.

- [ ] **Step 3: Run clippy**

Run: `make clippy`
Expected: No warnings or errors.

- [ ] **Step 4: Verify no template residue in release.toml**

Run: `grep -E '\{\{?[a-zA-Z_]+\}\}?' release.toml`
Expected: No output (no template residue found).

Wait — this will match the CORRECT `{{version}}` syntax too! Let me refine:

Run: `grep -E '\{version\}' release.toml`
Expected: No output (confirms single-brace `{version}` is gone).

Run: `grep -E '\{\{version\}\}' release.toml`
Expected: Shows the three lines with `{{version}}` (confirms correct syntax is present).

- [ ] **Step 5: Commit verification (if any adjustments needed)**

If any adjustments were made during verification:

```bash
git add -A
git commit -m "fix(release): adjust release workflow after end-to-end verification (#132)"
```

If no adjustments needed, skip this step.

- [ ] **Step 6: Summary**

Create a brief summary of what was fixed:

```markdown
## Summary

Fixed release.toml template placeholder syntax for cargo-release 1.1.3:

**Changes:**
1. `release.toml`: Changed `{version}` → `{{version}}` (3 fields)
2. `scripts/release.sh`: Expanded `TEMPLATE_RESIDUE_PATTERN` to detect both `{var}` and `{{var}}`
3. `scripts/release.sh`: Added self-test cases for single-brace residue
4. `scripts/release.sh`: Enhanced error messages with helpful guidance

**Verification:**
- `bash scripts/release.sh --self-test` — all cases pass
- `make release-rehearse` — dry-run succeeds, no template residue
- `make test` — all tests pass
- `make clippy` — no warnings
```

---

## Execution Notes

- **Task 1** is a simple config fix — no tests needed (config-only change).
- **Task 2** follows TDD: write failing test → update regex → verify pass.
- **Task 3** is end-to-end verification — no code changes unless issues are found.
- Total estimated time: 15-20 minutes.
- Risk: Low. Changes are isolated to release config/scripts, no Rust code affected.
