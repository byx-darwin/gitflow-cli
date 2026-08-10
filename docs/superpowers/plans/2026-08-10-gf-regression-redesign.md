# gf-regression Skill Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rewrite gf-regression skill documentation to improve clarity, add explicit boundaries, and optimize trigger word matching.

**Architecture:** Update the SKILL.md file with four major sections: (1) When to Use with 5 specific scenarios, (2) When NOT to Use with 6 exclusions, (3) Three-tier trigger system replacing current keyword list, (4) Four usage examples with expected output. Sync changes between source (`skills/`) and Claude Code copy (`.claude/skills/`).

**Tech Stack:** Markdown, SKILL.md format conventions

## Global Constraints

- Source file: `skills/gf-regression/SKILL.md`
- Claude Code copy: `.claude/skills/gf-regression/SKILL.md`
- Both files MUST be updated in sync
- Follow skill documentation conventions from CLAUDE.md (bilingual description, English narrative)
- Preserve existing sections that are not being replaced (Core Pattern, Quick Reference, Flowchart, Implementation, etc.)

---

### Task 1: Rewrite When to Use Section

**Files:**
- Modify: `skills/gf-regression/SKILL.md:12-19` (current When to Use section)
- Modify: `.claude/skills/gf-regression/SKILL.md:12-19` (sync copy)

**Interfaces:**
- Consumes: Current When to Use table (4 rows)
- Produces: Expanded When to Use section with 5 specific scenarios + language-specific triggers table

- [ ] **Step 1: Replace When to Use section in source file**

Open `skills/gf-regression/SKILL.md` and replace lines 12-19 (current When to Use section) with:

```markdown
## When to Use

Use this skill when you need to verify that the `gf` CLI is working correctly
after changes, before release, or when debugging CLI-related issues.

### Specific Scenarios

| # | Scenario | Trigger Phrase | Expected Action |
|---|----------|----------------|-----------------|
| 1 | Post-change verification | "verify my changes didn't break gf" | Run read-only smoke test, report PASS/FAIL |
| 2 | Pre-release gate | "run pre-release checks" | Run smoke test as part of release preparation |
| 3 | Debugging CLI issues | "gf commands are failing" | Run smoke test to identify which operations fail |
| 4 | Quick health check | "is gf working?" | Run minimal smoke test, report status |
| 5 | Regression detection | "check for regressions" | Run full smoke test, compare with baseline |

### Language-Specific Triggers

| English | 中文 | Context |
|---------|------|---------|
| smoke test | 冒烟测试 | Quick CLI health check |
| regression test | 回归测试 | Post-change verification |
| pre-release check | 发版前检查 | Before publishing release |
| verify CLI | 验证 CLI | Confirm CLI functionality |
| gf is broken | gf 坏了 | Debug CLI failures |
```

- [ ] **Step 2: Sync changes to Claude Code copy**

Copy the updated When to Use section to `.claude/skills/gf-regression/SKILL.md` (lines 12-19).

- [ ] **Step 3: Verify changes**

Run: `diff skills/gf-regression/SKILL.md .claude/skills/gf-regression/SKILL.md`
Expected: No differences in the When to Use section (lines 12-40 approximately)

- [ ] **Step 4: Commit**

```bash
git add skills/gf-regression/SKILL.md .claude/skills/gf-regression/SKILL.md
git commit -m "docs(skill): rewrite gf-regression When to Use with 5 scenarios

Expand When to Use from 4 generic rows to 5 specific scenarios:
1. Post-change verification
2. Pre-release gate
3. Debugging CLI issues
4. Quick health check
5. Regression detection

Add language-specific triggers table with EN/ZH pairs."
```

---

### Task 2: Add When NOT to Use Section

**Files:**
- Modify: `skills/gf-regression/SKILL.md` (insert after When to Use)
- Modify: `.claude/skills/gf-regression/SKILL.md` (sync copy)

**Interfaces:**
- Consumes: End of When to Use section
- Produces: New When NOT to Use section with exclusion table and misconceptions

- [ ] **Step 1: Add When NOT to Use section in source file**

Insert the following section immediately after the When to Use section (after the Language-Specific Triggers table):

```markdown
## When NOT to Use

Do NOT use this skill in the following scenarios:

| Scenario | Why Not | Use Instead |
|----------|---------|-------------|
| Fixing a bug | This skill only detects and reports bugs, doesn't fix them | `/gf-workflow` for bug fixes |
| Code review | This skill runs tests, doesn't review code | `/gf-pr-review` for code review |
| Full quality gate | This skill only runs smoke tests, not full CI | `/gf-quality` for complete quality checks |
| Testing other projects | This skill is designed for `gf` CLI only | Use project-specific test commands |
| CI pipeline | Smoke tests with autoreport shouldn't run in CI | Use `scripts/smoke-test.sh` directly with exit code |
| Performance testing | This skill checks functionality, not performance | Use benchmarking tools |

### Common Misconceptions

| Misconception | Reality |
|---------------|---------|
| "This will fix the bug" | No — it reports bugs via `/gf-autoreport-bug` |
| "This replaces CI" | No — it's a quick local check, not a CI replacement |
| "This works for any project" | No — it's hardcoded to `scripts/smoke-test.sh` |
```

- [ ] **Step 2: Sync changes to Claude Code copy**

Copy the new When NOT to Use section to `.claude/skills/gf-regression/SKILL.md` at the same position.

- [ ] **Step 3: Verify changes**

Run: `diff skills/gf-regression/SKILL.md .claude/skills/gf-regression/SKILL.md`
Expected: No differences in the When NOT to Use section

- [ ] **Step 4: Commit**

```bash
git add skills/gf-regression/SKILL.md .claude/skills/gf-regression/SKILL.md
git commit -m "docs(skill): add When NOT to Use section to gf-regression

Add explicit exclusion table with 6 scenarios where this skill should NOT be used:
- Fixing bugs (use gf-workflow)
- Code review (use gf-pr-review)
- Full quality gate (use gf-quality)
- Testing other projects
- CI pipeline
- Performance testing

Add Common Misconceptions table clarifying what this skill does NOT do."
```

---

### Task 3: Replace Trigger Keywords with Three-Tier System

**Files:**
- Modify: `skills/gf-regression/SKILL.md` (replace Trigger Keywords section)
- Modify: `.claude/skills/gf-regression/SKILL.md` (sync copy)

**Interfaces:**
- Consumes: Current Trigger Keywords section (lines ~124-129)
- Produces: New Trigger System section with three tiers + decision tree

- [ ] **Step 1: Replace Trigger Keywords section in source file**

Find the current "Trigger Keywords" section and replace it with:

```markdown
## Trigger System

This skill is triggered when the user's intent matches these scenarios:

### Primary Triggers (High Confidence)

| EN Trigger | ZH Trigger | Context Required | Action |
|------------|------------|------------------|--------|
| `run smoke test` | `跑冒烟测试` | Testing gf CLI | Execute smoke-test.sh |
| `check for regressions` | `检查回归` | Post-change verification | Execute smoke-test.sh |
| `verify gf works` | `验证 gf 是否正常` | CLI health check | Execute smoke-test.sh |
| `pre-release check` | `发版前检查` | Before release | Execute smoke-test.sh |

### Secondary Triggers (Medium Confidence)

| EN Trigger | ZH Trigger | Context Required | Action |
|------------|------------|------------------|--------|
| `gf is broken` | `gf 坏了` | CLI debugging | Diagnose + smoke test |
| `test failed after changes` | `改动后测试失败` | Post-change issue | Run regression check |

### Negative Triggers (Do NOT Load)

| Phrase | Likely Intent | Correct Skill |
|--------|---------------|---------------|
| `fix bug` | Bug fixing | `/gf-workflow` |
| `review code` | Code review | `/gf-pr-review` |
| `run tests` (for user's project) | Project testing | Project-specific commands |
| `quality check` | Full quality gate | `/gf-quality` |

### Trigger Decision Tree

```
User says something about "test" or "check"
    │
    ├─ Is it about gf CLI specifically?
    │   ├─ YES → Load gf-regression
    │   └─ NO → Is it about code review?
    │       ├─ YES → Load gf-pr-review
    │       └─ NO → Is it about full quality?
    │           ├─ YES → Load gf-quality
    │           └─ NO → Ask for clarification
    │
    └─ Is it about fixing a bug?
        └─ YES → Load gf-workflow
```
```

- [ ] **Step 2: Sync changes to Claude Code copy**

Copy the new Trigger System section to `.claude/skills/gf-regression/SKILL.md` replacing the old Trigger Keywords section.

- [ ] **Step 3: Verify changes**

Run: `diff skills/gf-regression/SKILL.md .claude/skills/gf-regression/SKILL.md`
Expected: No differences in the Trigger System section

- [ ] **Step 4: Commit**

```bash
git add skills/gf-regression/SKILL.md .claude/skills/gf-regression/SKILL.md
git commit -m "docs(skill): replace trigger keywords with three-tier system

Replace simple keyword list with scenario-based trigger system:
- Primary triggers (4 high-confidence scenarios)
- Secondary triggers (2 medium-confidence scenarios)
- Negative triggers (4 scenarios that should NOT load this skill)
- Decision tree for AI Agent to determine correct skill

This reduces false matches and helps AI Agent correctly identify
when to use gf-regression vs other skills."
```

---

### Task 4: Add Usage Examples Section

**Files:**
- Modify: `skills/gf-regression/SKILL.md` (insert before Success Criteria)
- Modify: `.claude/skills/gf-regression/SKILL.md` (sync copy)

**Interfaces:**
- Consumes: End of Quick Reference section
- Produces: New Usage Examples section with 4 detailed examples + Quick Start

- [ ] **Step 1: Add Usage Examples section in source file**

Insert the following section before the "Success Criteria" section:

```markdown
## Usage Examples

### Example 1: Quick Health Check

**User**: "Is gf working?"

**Action**:
```bash
bash scripts/smoke-test.sh --platform github
```

**Expected Output**:
```
=== Smoke Test Results ===
Platform: github
Mode: read-only

✅ PASS: auth status
✅ PASS: issue list
✅ PASS: repo view

Summary: 3 passed, 0 failed, 0 skipped
```

---

### Example 2: Post-Change Verification

**User**: "I made some changes, verify nothing is broken"

**Action**:
```bash
bash scripts/smoke-test.sh --platform github --verbose
```

**Expected Output** (if failure):
```
=== Smoke Test Results ===
Platform: github
Mode: read-only

✅ PASS: auth status
❌ FAIL: issue list
   Error: 404 Not Found
✅ PASS: repo view

Summary: 2 passed, 1 failed, 0 skipped

Classifying failures...
🟠 issue list: 4xx error (possible API change)

Creating bug report...
Issue created: https://github.com/.../issues/123
```

---

### Example 3: Pre-Release Check

**User**: "Run pre-release checks before publishing"

**Action**:
```bash
# Run smoke test first
bash scripts/smoke-test.sh --platform github
# Then run full quality gate (delegate to gf-quality)
```

**Expected Workflow**:
1. Smoke test passes → proceed to release
2. Smoke test fails → stop, investigate failures

---

### Example 4: Debugging CLI Issues

**User**: "gf commands are failing, what's wrong?"

**Action**:
1. Check auth: `gf auth status`
2. If auth valid → run smoke test to identify which operations fail
3. Classify failures and report

---

## Quick Start

```bash
# 1. Verify prerequisites
test -f scripts/smoke-test.sh && echo "Script ready" || echo "Script missing"
command -v gf && echo "gf installed" || echo "gf not found"
gf auth status && echo "Auth valid" || gf auth login

# 2. Run smoke test
bash scripts/smoke-test.sh --platform github

# 3. Check results
# Exit code 0 = all passed
# Exit code non-zero = failures detected (see report)
```
```

- [ ] **Step 2: Sync changes to Claude Code copy**

Copy the new Usage Examples section to `.claude/skills/gf-regression/SKILL.md` at the same position.

- [ ] **Step 3: Verify changes**

Run: `diff skills/gf-regression/SKILL.md .claude/skills/gf-regression/SKILL.md`
Expected: No differences in the Usage Examples section

- [ ] **Step 4: Commit**

```bash
git add skills/gf-regression/SKILL.md .claude/skills/gf-regression/SKILL.md
git commit -m "docs(skill): add usage examples and quick start guide

Add 4 detailed usage examples:
1. Quick health check - 'Is gf working?'
2. Post-change verification - 'Verify nothing is broken'
3. Pre-release check - 'Run pre-release checks'
4. Debugging CLI issues - 'gf commands are failing'

Each example includes user phrase, action command, and expected output.
Add Quick Start section with prerequisite verification steps."
```

---

### Task 5: Final Review and Sync Verification

**Files:**
- Verify: `skills/gf-regression/SKILL.md`
- Verify: `.claude/skills/gf-regression/SKILL.md`

**Interfaces:**
- Consumes: All previous task outputs
- Produces: Final verified and synced documentation

- [ ] **Step 1: Verify both files are in sync**

Run: `diff skills/gf-regression/SKILL.md .claude/skills/gf-regression/SKILL.md`
Expected: No output (files are identical)

If files differ, sync them:
```bash
cp skills/gf-regression/SKILL.md .claude/skills/gf-regression/SKILL.md
```

- [ ] **Step 2: Review complete document structure**

Verify the updated SKILL.md contains all required sections in order:
1. ✅ Frontmatter (name, description)
2. ✅ When to Use (5 scenarios + language triggers)
3. ✅ When NOT to Use (6 exclusions + misconceptions)
4. ✅ Core Pattern
5. ✅ Quick Reference
6. ✅ Flowchart
7. ✅ Implementation
8. ✅ Error Handling
9. ✅ Responsibility
10. ✅ Trigger System (3 tiers + decision tree)
11. ✅ Usage Examples (4 examples + quick start)
12. ✅ Success Criteria
13. ✅ Common Mistakes
14. ✅ See Also

- [ ] **Step 3: Test trigger word matching**

Verify the new trigger system correctly identifies when to use this skill:
- "run smoke test" → Primary trigger, should load
- "check for regressions" → Primary trigger, should load
- "gf is broken" → Secondary trigger, should load
- "fix bug" → Negative trigger, should NOT load
- "review code" → Negative trigger, should NOT load

- [ ] **Step 4: Final commit (if any sync changes needed)**

```bash
git add skills/gf-regression/SKILL.md .claude/skills/gf-regression/SKILL.md
git commit -m "docs(skill): final sync verification for gf-regression

Ensure skills/ and .claude/skills/ copies are identical.
Verify all sections present and trigger system works correctly."
```

---

## Success Criteria Verification

After completing all tasks, verify:

- [ ] When to Use section has 5+ specific scenarios ✅
- [ ] When NOT to Use section exists with 6+ exclusions ✅
- [ ] Trigger system uses three-tier confidence model ✅
- [ ] 4+ usage examples with expected output documented ✅
- [ ] Both `skills/` and `.claude/skills/` files are in sync ✅
- [ ] AI Agent can correctly match skill in test scenarios ✅

---

## Implementation Notes

- **No code changes**: This is a documentation-only task, no Rust code modifications
- **No tests needed**: Markdown documentation doesn't require unit tests
- **Manual verification**: Trigger word matching must be verified by reviewing the document
- **Sync requirement**: Both file locations must be updated together to maintain consistency
