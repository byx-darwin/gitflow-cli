# gitflow-workflow Mode Optimization — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add 3-tier mode system (fast/standard/full) to gitflow-workflow with auto-detection, align quality gate with CI, and introduce smart subagent batching.

**Architecture:** Incremental enhancement of existing skill files. Schema adds `standard` mode + `phase4_steps_executed` evidence field. SKILL.md/gates.md/references.md updated with 3-mode logic. Quality gate Rust reference aligned with Makefile. Documentation synced across guide and CLAUDE.md.

**Tech Stack:** JSON Schema, Markdown, Python pseudocode (for algorithm illustration only — no code execution)

## Global Constraints

- No Rust code changes — this is a documentation/skill-file-only refactor
- Existing `full`/`fast` contracts must remain valid (backward compatible)
- Schema version stays at `1.1` (no breaking change)
- All skill file changes must maintain bilingual descriptions (EN + ZH) per skill conventions
- Phase 4 step matrix: Pipeline + Branch cleanup mandatory for ALL modes
- Quality gate Rust reference: `-W clippy::pedantic --all-features` becomes default
- Auto-detection priority: user override > labels > title prefix > default (standard)

---

### Task 1: Schema Update — contract.schema.json

**Files:**
- Modify: `.claude/skills/gitflow-workflow/contract.schema.json`

**Interfaces:**
- Consumes: Current schema with `mode: ["full", "fast"]`
- Produces: Schema with `mode: ["full", "standard", "fast"]` + `phase4_steps_executed` evidence field

- [ ] **Step 1: Update mode enum to include "standard"**

Change line 27 from:
```json
"enum": ["full", "fast"],
```
to:
```json
"enum": ["full", "standard", "fast"],
```

- [ ] **Step 2: Update mode description**

Change line 28 from:
```json
"description": "full = 完整四阶段；fast = 跳过非必需 Phase"
```
to:
```json
"description": "full = 完整四阶段；standard = 中等复杂度；fast = 简单任务跳过非必需 Phase"
```

- [ ] **Step 3: Add `phase4_steps_executed` to phase evidence definition**

In `$defs.phase.properties.evidence.properties`, add after `branch_cleaned`:

```json
"phase4_steps_executed": {
  "type": "array",
  "items": { "type": "string" },
  "description": "List of Phase 4 steps actually executed (for audit trail)"
}
```

- [ ] **Step 4: Verify schema is valid JSON**

Run: `cat .claude/skills/gitflow-workflow/contract.schema.json | jq . > /dev/null && echo "VALID JSON"`
Expected: `VALID JSON`

- [ ] **Step 5: Commit**

```bash
git add .claude/skills/gitflow-workflow/contract.schema.json
git commit -m "feat(skills): add standard mode to workflow contract schema

- Add 'standard' to mode enum (between full and fast)
- Add phase4_steps_executed evidence field for audit trail
- Backward compatible: existing full/fast contracts remain valid

Refs: #121"
```

---

### Task 2: Quality Gate Alignment — rust.md

**Files:**
- Modify: `.claude/skills/gitflow-quality/references/rust.md`

**Interfaces:**
- Consumes: Current Gate 5 command without pedantic
- Produces: Gate 5 with pedantic + Makefile-first rule

- [ ] **Step 1: Update Gate 5 static command**

Change line 13 from:
```markdown
| 5 | static | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, no warnings |
```
to:
```markdown
| 5 | static | `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic` | exit 0, no warnings |
```

- [ ] **Step 2: Remove "Strict Mode (Optional)" section**

Delete lines 37-43:
```markdown
## Strict Mode (Optional)

Append `-W clippy::pedantic` to Gate 5 for stricter linting:

```bash
cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic
```
```

- [ ] **Step 3: Add Makefile-First Rule section**

After the "Forbidden Actions" section, add:

```markdown
## Makefile-First Rule

If project root contains a `Makefile` with matching targets, prefer `make` commands over direct tool invocations:

| Gate | Preferred Command | Fallback |
|------|-------------------|----------|
| build | `make build` | `cargo build --workspace --quiet` |
| test | `make test` | `cargo test --workspace --quiet` |
| format | `make fmt` | `cargo +nightly fmt -- --check` |
| static | `make clippy` | `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic` |

Detection: `make -n <target> >/dev/null 2>&1` returns 0 → target exists.
```

- [ ] **Step 4: Verify file renders correctly**

Run: `cat .claude/skills/gitflow-quality/references/rust.md | head -20`
Expected: Gate 5 line includes `--all-features` and `-W clippy::pedantic`

- [ ] **Step 5: Commit**

```bash
git add .claude/skills/gitflow-quality/references/rust.md
git commit -m "fix(skills): align quality gate with CI clippy configuration

- Gate 5 now includes -W clippy::pedantic and --all-features by default
- Remove 'Strict Mode (Optional)' section (pedantic is now default)
- Add Makefile-first rule: prefer make targets over direct cargo commands

This fixes the inconsistency where local clippy passed but CI failed.

Refs: #121"
```

---

### Task 3: Core Skill Update — SKILL.md

**Files:**
- Modify: `.claude/skills/gitflow-workflow/SKILL.md`

**Interfaces:**
- Consumes: Current 2-mode skill definition
- Produces: 3-mode skill with auto-detection, Phase 4 matrix, batching strategy

This is the largest change. The SKILL.md has multiple sections to update:

1. Mode comparison table
2. Phase 1 description (mode-specific behavior)
3. Phase 2 description (standard mode skips in fast)
4. Phase 4 description (step matrix by mode)
5. Auto-detection rules
6. Smart batching strategy

- [ ] **Step 1: Update Mode Comparison table**

Find the existing "Mode Comparison" table (around line 26-33) and replace with:

```markdown
## Mode Comparison

| Phase | Full Mode | Standard Mode | Fast Mode |
|-------|-----------|---------------|-----------|
| 1 | brainstorming + issue-create + issue-review | brainstorming + issue-create + issue-review | issue-create only |
| 2 | writing-plans + quality gate | writing-plans + quality gate | **skippable** |
| 3 | subagent-driven-development (TDD + Code Review) | subagent-driven-development (TDD + Code Review) | **required** (TDD + Code Review) |
| 4 | pipeline + triage + review + dogfooding + branch-finish | pipeline + review + branch-finish | pipeline + branch-finish |

**Mode auto-detection:** "fix: typo"/"docs:"/"chore:"/"hotfix" → `fast` · "fix: bug"/"refactor: small" → `standard` · "feat: new feature"/"refactor: large"/breaking → `full` · unclear → `standard` (default). User can always override.
```

- [ ] **Step 2: Update "When to Use" section**

Find the "When to Use" table (around line 50-55) and update to include standard mode:

```markdown
## When to Use

| EN | ZH |
|----|----|
| full workflow | 全流程（默认） |
| clarify → plan → execute → deliver | 需求→计划→执行→交付 |

**When NOT to Use:** quick fix → `gitflow-commit` · PR review → `gitflow-pr-review` · architecture discussion → `superpowers:brainstorming` directly · user says "don't create an Issue" → do NOT invoke.

**Mode auto-detection:** "fix"/"typo"/"hotfix"/"docs"/"chore" → `fast` · "refactor: small"/"fix: bug" → `standard` · "feat"/"refactor: large"/breaking → `full` · `good-first-issue` label → `fast` · unclear → `standard` (default). User can override with `--mode <mode>`.
```

- [ ] **Step 3: Update Fast Mode section to add Standard Mode**

After the "Fast Mode — Required Skills Checklist" section, add Standard Mode section:

```markdown
## Standard Mode — Required Skills Checklist

In standard mode, the following skills are invoked per phase:

**Phase 1:** `superpowers:brainstorming` (required), `gitflow-issue-create` (required), `gitflow-issue-review` (required)

**Phase 2:** `superpowers:writing-plans` (required) + `gitflow-quality` gate (required)

**Phase 3:** `superpowers:subagent-driven-development` with TDD + Code Review (required)

**Phase 4:** `gitflow-pipeline-analyzer` → `gitflow-review` → Branch Finish (all required)
```

- [ ] **Step 4: Add Auto-Detection Rules section**

Add new section after "Mode Comparison":

```markdown
## Mode Auto-Detection

Detection priority (highest to lowest):

1. **User explicit override** → `--mode fast` / `--mode standard` / `--mode full`
2. **Issue labels** → `good-first-issue` / `kind/typo` → fast; `kind/feature` → full
3. **Issue title prefix** → conventional commit format:
   - `fix: typo`, `docs:`, `chore:`, `hotfix` → **fast**
   - `fix:`, `refactor:`, `perf:` (single file/module) → **standard**
   - `feat:`, `refactor:` (cross-module), `!` (breaking change) → **full**
4. **Default** → **standard** (balanced safety vs efficiency)

### Confirmation Flow

```
检测到 `refactor(skills)` 前缀 → 建议 standard 模式
自动检测结果：standard
是否确认？[Y/n/override]
```

User input:
- `Y` / Enter → accept suggested mode
- `n` → enter mode selection menu (fast/standard/full)
- `fast` / `standard` / `full` → direct override
```

- [ ] **Step 5: Update Phase 4 section with step matrix**

Find the "Phase 4: Post-Delivery Checks" section and update to include mode-specific steps:

```markdown
## Phase 4: Post-Delivery Checks

**Entry:** Gate 3→4 passed · **Exit:** `phases.4.status = complete` · **Auto-advance:** archive on complete

### Phase 4 Step Matrix by Mode

| # | Step | Full | Standard | Fast |
|---|------|------|----------|------|
| 1 | Pipeline analysis | ✅ | ✅ | ✅ |
| 2 | Issue triage | ✅ | ❌ | ❌ |
| 3 | Code review report | ✅ | ✅ | ❌ |
| 4 | Dogfooding checklist | ✅ | ❌ | ❌ |
| 5 | Branch Finish | ✅ | ✅ | ✅ |

### Execution Flow by Mode

- **Full:** Pipeline → Triage → Review → Dogfooding → Branch Finish → Archive
- **Standard:** Pipeline → Review → Branch Finish → Archive
- **Fast:** Pipeline → Branch Finish → Archive
```

- [ ] **Step 6: Update Phase 4 steps table to reflect mode-specific behavior**

Replace the existing Phase 4 steps table with:

```markdown
| Step | Action | Output |
|------|--------|--------|
| 1 | **[AUTO]** `gitflow-pipeline-analyzer` — generates pipeline analysis report (all modes) | `pipeline_ok` |
| 2 | **[AUTO]** `gitflow-issue-triage` — produces Issue triage report (full mode only) | — |
| 3 | **[AUTO]** `gitflow-review` — creates code review report (full + standard modes) | `review_report_path` |
| 4 | **[AUTO]** Dogfooding checklist (`docs/specs/phase4-dogfooding-checklist.md`) (full mode only) | `dogfooding_passed` |
| 5 | **[CONFIRM]** Branch Finish — detect PR merge status, user-confirmed cleanup (all modes) | `branch_cleaned` |
| 6 | **[AUTO]** Update contract: `evidence = { pipeline_ok, review_report_path, dogfooding_passed, branch_cleaned, phase4_steps_executed }` | — |
| 7 | **[AUTO]** Archive contract → `.cache/workflows/archive/YYYY-MM/` | — |
```

- [ ] **Step 7: Add Smart Batching section**

Add new section after "Sub-Skill Invocation Rules":

```markdown
## Smart Subagent Batching

### Complexity Scoring

```python
def classify_task_complexity(task):
    score = 0
    score += len(task.files_changed) * 1
    score += 3 if task.crosses_module_boundary else 0
    score += 2 if task.changes_public_api else 0
    score += 1 if task.requires_migration else 0

    if score <= 2:
        return "simple"    # batch
    elif score <= 6:
        return "medium"    # independent subagent
    else:
        return "complex"   # independent subagent + extra review
```

### Execution by Complexity

| Complexity | Method | Description |
|-----------|--------|-------------|
| Simple (score ≤ 2) | Batch in main agent | Implement all tasks in main agent, single review pass |
| Medium (score 3-6) | Independent subagent | One subagent per task + TDD + review |
| Complex (score > 6) | Independent subagent + extra review | One subagent per task + TDD + review + extra scrutiny |

### Batch Execution Flow (Simple Tasks)

```
Phase A: Batch Implement (main agent)
  ├── task_1: RED → GREEN
  ├── task_2: RED → GREEN
  └── task_3: RED → GREEN

Phase B: Batch Review (single subagent reviews all changes)

Phase C: Fix (if needed, main agent addresses findings)
```

### Mode-Batch Defaults

| Mode | Default Behavior |
|------|------------------|
| fast | Lean toward batch (changes usually simple) |
| standard | Score-based decision |
| full | Lean toward independent subagent (changes usually complex) |

User can override batching strategy during plan phase.
```

- [ ] **Step 8: Verify file renders correctly**

Run: `wc -l .claude/skills/gitflow-workflow/SKILL.md`
Expected: Line count increased significantly (from ~350 to ~450+)

- [ ] **Step 9: Commit**

```bash
git add .claude/skills/gitflow-workflow/SKILL.md
git commit -m "feat(skills): add 3-mode system to gitflow-workflow

- Add standard mode between full and fast
- Add auto-detection rules with user confirmation
- Add Phase 4 step matrix (mode-specific steps)
- Add smart subagent batching strategy with complexity scoring
- Update mode comparison table and all mode references

Refs: #121"
```

---

### Task 4: Gates Update — gates.md

**Files:**
- Modify: `.claude/skills/gitflow-workflow/gates.md`

**Interfaces:**
- Consumes: Current gate rules with full/fast exemptions
- Produces: Gate rules with full/standard/fast exemptions

- [ ] **Step 1: Update Gate 1→2 section**

Find the "Gate 1→2" section and update fast mode exemptions to include standard mode:

Change:
```markdown
**fast 模式豁免:**
- `comment_id` 可省略（issue-review 可选）
- `design_doc_path` 可省略（brainstorming 可选）
```
to:
```markdown
**fast 模式豁免:**
- `comment_id` 可省略（issue-review 可选）
- `design_doc_path` 可省略（brainstorming 可选）

**standard 模式:** 无豁免（与 full 模式相同）
```

- [ ] **Step 2: Update Gate 2→3 section**

Find the "Gate 2→3" section and update:

Change:
```markdown
**fast 模式豁免:** `spec_path` 和 `user_approved` 可省略（writing-plans 可选）
```
to:
```markdown
**fast 模式豁免:** `spec_path` 和 `user_approved` 可省略（writing-plans 可选）

**standard 模式:** 无豁免（与 full 模式相同，writing-plans 必选）
```

- [ ] **Step 3: Update gate check algorithm**

Replace the Python gate check algorithm with:

```python
def check_gate(contract, target_phase):
    mode = contract["mode"]

    if target_phase == 2:
        evidence = contract["phases"]["1"]["evidence"]
        if mode == "fast":
            # fast mode: only issue_url required
            return contract["phases"]["1"]["status"] == "complete" \
                   and evidence.get("issue_url")
        # standard and full: all evidence required
        return contract["phases"]["1"]["status"] == "complete" \
               and evidence.get("issue_url") \
               and evidence.get("comment_id") \
               and evidence.get("design_doc_path")

    elif target_phase == 3:
        if mode == "fast":
            return True  # fast mode skips planning
        # standard and full: spec + approval required
        evidence = contract["phases"]["2"]["evidence"]
        return contract["phases"]["2"]["status"] == "complete" \
               and evidence.get("spec_path") \
               and evidence.get("user_approved")

    elif target_phase == 4:
        evidence = contract["phases"]["3"]["evidence"]
        return contract["phases"]["3"]["status"] == "complete" \
               and evidence.get("pr_url") \
               and evidence.get("tests_passed")

    return False
```

- [ ] **Step 4: Add Phase 4 internal step selection**

Add after the gate check algorithm:

```python
def get_phase4_steps(mode):
    """Return list of Phase 4 steps to execute based on mode."""
    steps = ["pipeline", "branch_finish"]  # all modes

    if mode in ("full", "standard"):
        steps.insert(1, "review")

    if mode == "full":
        steps.insert(1, "triage")
        steps.insert(3, "dogfooding")

    return steps
```

- [ ] **Step 5: Verify file renders correctly**

Run: `cat .claude/skills/gitflow-workflow/gates.md | grep -c "standard"`
Expected: At least 3 occurrences of "standard"

- [ ] **Step 6: Commit**

```bash
git add .claude/skills/gitflow-workflow/gates.md
git commit -m "feat(skills): add standard mode gate exemptions

- Gate 1→2: standard mode requires all evidence (same as full)
- Gate 2→3: standard mode requires spec + approval (same as full)
- Add get_phase4_steps() function for mode-specific Phase 4 flow
- Update gate check algorithm to handle 3 modes

Refs: #121"
```

---

### Task 5: References Update — references.md

**Files:**
- Modify: `.claude/skills/gitflow-workflow/references.md`

**Interfaces:**
- Consumes: Current cross-session recovery procedure
- Produces: Mode-aware recovery procedure

- [ ] **Step 1: Update Create Contract template**

Find the Create Contract bash block and update the mode placeholder:

Change:
```bash
"mode": "<full|fast>",
```
to:
```bash
"mode": "<full|standard|fast>",
```

- [ ] **Step 2: Update Cross-Session Recovery section**

Find the "Cross-Session Recovery" section and update the context loading table:

Change:
```
3. Load context:
   • Phase 1: No doc needed (start fresh)
   • Phase 2: Read design_doc_path
   • Phase 3: Read spec_path (plan document)
   • Phase 4: Read pr_url + review reports
```
to:
```
3. Load context based on mode and current_phase:
   • Phase 1: No doc needed (start fresh)
   • Phase 2: Read design_doc_path; check mode for Phase 1 exemptions
   • Phase 3: Read spec_path (plan document); check mode for fast skip
   • Phase 4: Read pr_url + review reports; use get_phase4_steps(mode) to determine remaining steps
```

- [ ] **Step 3: Commit**

```bash
git add .claude/skills/gitflow-workflow/references.md
git commit -m "docs(skills): update references for 3-mode system

- Create Contract template now includes standard mode
- Cross-session recovery is mode-aware (uses get_phase4_steps)

Refs: #121"
```

---

### Task 6: Documentation Sync — gitflow-workflow-guide.md

**Files:**
- Modify: `docs/gitflow-workflow-guide.md`

**Interfaces:**
- Consumes: Current 2-mode guide
- Produces: 3-mode guide with Phase 4 matrix

- [ ] **Step 1: Update "两种模式" to "三种模式"**

Find the "两种模式" table (around line 20-26) and replace with:

```markdown
三种模式：

| 模式 | 适用场景 | 必选子技能 |
|---|---|---|
| **完整模式 (full)** | 新功能 / 大重构 / 跨模块 | 全部 7 个 |
| **标准模式 (standard)** | 中等复杂度 / 单模块改动 | 6 个（Phase 4 简化：无 triage/dogfooding） |
| **快速模式 (fast)** | Bug fix / 小改动 / typo | 4 个（Phase 1 issue-create、Phase 3 subagent、Phase 4 pipeline + branch-finish） |
```

- [ ] **Step 2: Update Phase 4 section with mode matrix**

Find the Phase 4 section (around line 278-326) and add mode-specific step table after the existing content:

```markdown
### Phase 4 步骤矩阵（按模式）

| # | 步骤 | Full | Standard | Fast |
|---|------|------|----------|------|
| 1 | 流水线分析 | ✅ | ✅ | ✅ |
| 2 | Issue 分类 | ✅ | ❌ | ❌ |
| 3 | 代码审查报告 | ✅ | ✅ | ❌ |
| 4 | Dogfooding 检查 | ✅ | ❌ | ❌ |
| 5 | Branch Finish | ✅ | ✅ | ✅ |

**执行流程：**
- Full: Pipeline → Triage → Review → Dogfooding → Branch Finish → Archive
- Standard: Pipeline → Review → Branch Finish → Archive
- Fast: Pipeline → Branch Finish → Archive
```

- [ ] **Step 3: Update "模式对比速查" table**

Find the "模式对比速查" table (around line 327-335) and replace with:

```markdown
## 模式对比速查

| 维度 | 完整模式 (full) | 标准模式 (standard) | 快速模式 (fast) |
|---|---|---|---|
| Phase 1 | brainstorming ✅ + issue-create ✅ + issue-review ✅ | brainstorming ✅ + issue-create ✅ + issue-review ✅ | issue-create ✅ |
| Phase 2 | writing-plans ✅ + 完整 quality gate | writing-plans ✅ + 完整 quality gate | 可内联计划；quality gate 不变 |
| Phase 3 | subagent-driven ✅ + TDD ✅ + review ✅ | subagent-driven ✅ + TDD ✅ + review ✅ | 同左 |
| Phase 4 | pipeline ✅ + triage ✅ + review ✅ + dogfooding ✅ + branch-finish ✅ | pipeline ✅ + review ✅ + branch-finish ✅ | pipeline ✅ + branch-finish ✅ |
| 适用 | 新功能 / 大重构 / 跨模块 | 中等复杂度 / 单模块 | bug fix / 单文件改动 / 配置调整 |
```

- [ ] **Step 4: Verify file renders correctly**

Run: `grep -c "标准模式" docs/gitflow-workflow-guide.md`
Expected: At least 2 occurrences

- [ ] **Step 5: Commit**

```bash
git add docs/gitflow-workflow-guide.md
git commit -m "docs: update workflow guide for 3-mode system

- Change '两种模式' to '三种模式' with standard mode
- Add Phase 4 step matrix by mode
- Update mode comparison quick reference table

Refs: #121"
```

---

### Task 7: Documentation Sync — CLAUDE.md

**Files:**
- Modify: `CLAUDE.md`

**Interfaces:**
- Consumes: Current Mode Comparison table
- Produces: 3-mode comparison table

- [ ] **Step 1: Update Mode Comparison table in CLAUDE.md**

Find the "Mode Comparison" table in CLAUDE.md and replace with:

```markdown
## Mode Comparison

| Phase | Full Mode | Standard Mode | Fast Mode |
|-------|-----------|---------------|-----------|
| 1 | brainstorming + issue-create + issue-review | brainstorming + issue-create + issue-review | issue-create (required), brainstorming (optional) |
| 2 | writing-plans + quality gate | writing-plans + quality gate | **skippable** |
| 3 | subagent-driven-development (TDD + Code Review) | subagent-driven-development (TDD + Code Review) | **required** |
| 4 | pipeline + triage + review + dogfooding | pipeline + review | pipeline + branch-finish |
```

- [ ] **Step 2: Verify CLAUDE.md renders correctly**

Run: `grep -A 10 "## Mode Comparison" CLAUDE.md`
Expected: 3-mode table visible

- [ ] **Step 3: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: update CLAUDE.md mode comparison to 3 modes

Refs: #121"
```

---

### Task 8: Quality Gate — Validate All Changes

**Files:** None (validation task)

**Interfaces:**
- Consumes: All 7 files modified in Tasks 1-7
- Produces: Validation report

This task verifies all changes are consistent and correct.

- [ ] **Step 1: Verify JSON schema is valid**

Run: `cat .claude/skills/gitflow-workflow/contract.schema.json | jq . > /dev/null && echo "SCHEMA: VALID"`
Expected: `SCHEMA: VALID`

- [ ] **Step 2: Verify schema includes standard mode**

Run: `grep -c '"standard"' .claude/skills/gitflow-workflow/contract.schema.json`
Expected: At least 1 occurrence

- [ ] **Step 3: Verify quality gate alignment**

Run: `grep "pedantic" .claude/skills/gitflow-quality/references/rust.md`
Expected: Line showing Gate 5 includes `-W clippy::pedantic`

- [ ] **Step 4: Verify no "Strict Mode" section remains**

Run: `grep -c "Strict Mode" .claude/skills/gitflow-quality/references/rust.md`
Expected: `0`

- [ ] **Step 5: Verify SKILL.md has 3 modes**

Run: `grep -c "standard" .claude/skills/gitflow-workflow/SKILL.md`
Expected: At least 5 occurrences

- [ ] **Step 6: Verify gates.md has standard mode**

Run: `grep -c "standard" .claude/skills/gitflow-workflow/gates.md`
Expected: At least 3 occurrences

- [ ] **Step 7: Verify docs/gitflow-workflow-guide.md has 3 modes**

Run: `grep -c "标准模式" docs/gitflow-workflow-guide.md`
Expected: At least 2 occurrences

- [ ] **Step 8: Verify CLAUDE.md has 3 modes**

Run: `grep -A 10 "## Mode Comparison" CLAUDE.md | grep -c "Standard"`
Expected: At least 1 occurrence

- [ ] **Step 9: Run git diff to review all changes**

Run: `git diff --stat HEAD~7`
Expected: 7 files changed, reasonable line counts

- [ ] **Step 10: Verify no broken links**

Run: `grep -r "skills/gitflow-workflow" docs/gitflow-workflow-guide.md | head -5`
Expected: Valid skill references

- [ ] **Step 11: Summary validation report**

Create a validation summary:

```
VALIDATION REPORT — Issue #121 Mode Optimization
=================================================

Schema:          ✅ Valid JSON, includes standard mode
Quality Gate:    ✅ Pedantic clippy is default, Strict Mode removed
SKILL.md:        ✅ 3-mode system documented
gates.md:        ✅ Standard mode exemptions added
references.md:   ✅ Mode-aware recovery
Guide:           ✅ 3-mode comparison table
CLAUDE.md:       ✅ Mode comparison updated

All checks passed.
```

- [ ] **Step 12: No commit needed (validation only)**

This task is validation only — no new commit required.

---

## Plan Summary

| Task | File(s) | Type | Est. Time |
|------|---------|------|-----------|
| 1 | contract.schema.json | Schema | 5 min |
| 2 | rust.md | Quality gate fix | 5 min |
| 3 | SKILL.md | Core skill update | 20 min |
| 4 | gates.md | Gate rules | 10 min |
| 5 | references.md | References | 5 min |
| 6 | gitflow-workflow-guide.md | Docs | 10 min |
| 7 | CLAUDE.md | Docs | 5 min |
| 8 | (validation) | QA | 10 min |
| **Total** | **7 files** | — | **~70 min** |

**Execution order:** Tasks 1-7 are sequential (each builds on prior). Task 8 is final validation.

**No Rust code changes** — this is a documentation/skill-file-only refactor. TDD cycle is not applicable. Validation is via grep/jq checks.
