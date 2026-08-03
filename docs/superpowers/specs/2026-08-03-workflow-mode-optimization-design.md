# gitflow-workflow Mode Optimization — Design Spec

**Date:** 2026-08-03
**Issue:** [#121](https://github.com/byx-darwin/gitflow-cli/issues/121)
**Status:** Draft (pending user review)
**Approach:** Incremental Enhancement (Approach 1)

## TL;DR

Add a `standard` mode between existing `fast`/`full`, auto-detect mode from issue title/labels with user confirmation, align quality gate with CI (`-W clippy::pedantic`), and introduce smart subagent batching for simple tasks.

---

## 1. Mode System & Auto-Detection

### 1.1 Mode Definitions

| Mode | 适用场景 | Phase 1 | Phase 2 | Phase 3 | Phase 4 |
|------|---------|---------|---------|---------|---------|
| **fast** | `fix: typo`, `docs: update`, `hotfix` | issue-create ✅<br>brainstorming ❌<br>issue-review ❌ | ❌ Skipped | ✅ TDD + Review 必选 | Pipeline + Branch cleanup |
| **standard** | `fix: bug`, `refactor: small`, 单模块改动 | issue-create ✅<br>brainstorming ✅<br>issue-review ✅ | ✅ writing-plans + quality gate | ✅ TDD + Review 必选 | Pipeline + Code review + Branch cleanup |
| **full** | `feat: new feature`, `refactor: large`, 跨模块 | issue-create ✅<br>brainstorming ✅<br>issue-review ✅ | ✅ writing-plans + quality gate | ✅ TDD + Review 必选 | Pipeline + Triage + Review + Dogfooding + Branch cleanup |

### 1.2 Auto-Detection Rules

Detection priority (highest to lowest):

1. **User explicit override** → `/gitflow-workflow --mode fast`
2. **Issue labels** → `good-first-issue` / `kind/typo` → fast; `kind/feature` → full
3. **Issue title prefix** → conventional commit format:
   - `fix: typo`, `docs:`, `chore:` → fast
   - `fix:`, `refactor:`, `perf:` (single file/module) → standard
   - `feat:`, `refactor:` (cross-module), `!` (breaking) → full
4. **Default** → standard (balanced safety vs efficiency)

### 1.3 Confirmation Flow

```
检测到 `refactor(skills)` 前缀 → 建议 standard 模式
自动检测结果：standard
是否确认？[Y/n/override]
```

User input:
- `Y` / Enter → accept suggested mode
- `n` → enter mode selection menu
- `fast` / `standard` / `full` → direct override

---

## 2. Phase 4 Scope by Mode

### 2.1 Phase 4 Step Matrix

| # | Step | Full | Standard | Fast | Rationale |
|---|------|------|----------|------|-----------|
| 1 | Pipeline analysis | ✅ | ✅ | ✅ | CI status is a basic guarantee |
| 2 | Issue triage | ✅ | ❌ | ❌ | Label classification only needed for complex tasks |
| 3 | Code review report | ✅ | ✅ | ❌ | Review is a quality baseline (fast mode: small changes can skip) |
| 4 | Dogfooding checklist | ✅ | ❌ | ❌ | Full self-check only needed for complex tasks |
| 5 | Branch Finish | ✅ | ✅ | ✅ | Worktree cleanup is hygiene |

### 2.2 Phase 4 Execution Flow by Mode

**Full mode:**
```
Pipeline → Triage → Review → Dogfooding → Branch Finish → Archive
```

**Standard mode:**
```
Pipeline → Review → Branch Finish → Archive
```

**Fast mode:**
```
Pipeline → Branch Finish → Archive
```

### 2.3 Contract Schema Update

New evidence field (mode-agnostic):
- `phase4_steps_executed`: list of steps actually executed (for audit)

```json
{
  "phases": {
    "4": {
      "evidence": {
        "pipeline_ok": true,
        "review_report_path": "docs/code-review-report-xxx.md",
        "dogfooding_passed": true,
        "branch_cleaned": true,
        "phase4_steps_executed": ["pipeline", "review", "branch_finish"]
      }
    }
  }
}
```

### 2.4 Mode-Specific Gate Rules

Gate 3→4 conditions unchanged (`pr_url` + `tests_passed`), but Phase 4 internally skips steps based on `mode`:

```python
def get_phase4_steps(mode):
    steps = ["pipeline", "branch_finish"]  # all modes
    if mode in ("full", "standard"):
        steps.insert(1, "review")
    if mode == "full":
        steps.insert(1, "triage")
        steps.insert(3, "dogfooding")
    return steps
```

---

## 3. Quality Gate Alignment

### 3.1 Root Cause

Current `gitflow-quality` Rust reference (`references/rust.md`) Gate 5:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

vs `Makefile` `clippy` target:

```bash
cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic
```

**Differences:**
- skill missing `-W clippy::pedantic`
- skill missing `--all-features`
- skill uses `--workspace`, Makefile doesn't (but equivalent for workspace projects)

This causes local pass but CI failure.

### 3.2 Fix

**Update `gitflow-quality/references/rust.md` Gate 5:**

```markdown
| 5 | static | `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic` | exit 0, no warnings |
```

**Remove "Strict Mode (Optional)" section** — pedantic becomes default, not optional.

### 3.3 Makefile-First Principle

New rule: if project root has `Makefile` with `clippy`/`fmt`/`test` targets, **prefer `make clippy` over direct `cargo clippy`**.

Rationale:
- Makefile is project-level config, represents project conventions
- Changing Makefile propagates globally, no need to change skill

```markdown
## Makefile-First Rule

If project has `Makefile` with `clippy` target:

| Gate | Preferred Command | Fallback |
|------|-------------------|----------|
| build | `make build` | `cargo build --workspace --quiet` |
| test | `make test` | `cargo test --workspace --quiet` |
| format | `make fmt` | `cargo +nightly fmt -- --check` |
| static | `make clippy` | `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic` |
```

### 3.4 Verification

After fix, running `gitflow-quality` should produce CI-consistent results.

---

## 4. Smart Subagent Batching

### 4.1 Current Problem

`subagent-driven-development` dispatches one subagent per task:

```
task_1: subagent → implement → review → fix   (~5k tokens)
task_2: subagent → implement → review → fix   (~5k tokens)
task_3: subagent → implement → review → fix   (~5k tokens)
total: ~15k tokens
```

For simple tasks, subagent context loading and review overhead is disproportionately large.

### 4.2 Smart Batching Strategy

Dynamically decide execution method based on task complexity:

| Task Type | Criteria | Execution Method |
|-----------|----------|------------------|
| **Simple** | Single file, typo fix, config change, docs update | Batch execute + consolidated review |
| **Medium** | 2-5 files, single function/module change | Independent subagent + TDD + review |
| **Complex** | >5 files, cross-module, API change, new feature | Independent subagent + TDD + review + extra scrutiny |

### 4.3 Batch Execution Flow

**Simple task batch mode:**

```
Phase A: Batch Implement
  ├── task_1: implement (in main agent, no subagent)
  ├── task_2: implement (in main agent)
  └── task_3: implement (in main agent)

Phase B: Batch Review
  └── single subagent reviews all 3 tasks together

Phase C: Fix (if needed)
  └── main agent fixes review findings
```

**Token estimate:**

| Mode | Implement | Review | Total |
|------|-----------|--------|-------|
| Current (per-task subagent) | 3 × 3k = 9k | 3 × 2k = 6k | ~15k |
| Batch mode | 3 × 1k = 3k (main agent) | 1 × 3k = 3k | ~6k |
| **Savings** | — | — | **~60%** |

### 4.4 TDD Discipline Guarantee

**Batch mode TDD:**

```
task_1: write failing test → implement → test passes
task_2: write failing test → implement → test passes
task_3: write failing test → implement → test passes
→ batch review all changes
```

- Each task still requires **RED → GREEN** cycle
- REFACTOR phase deferred until after batch review
- Code review covers all changes

### 4.5 Complexity Scoring Algorithm

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

### 4.6 Mode-Batch Relationship

| Mode | Default Behavior |
|------|------------------|
| fast | Lean toward batch (changes usually simple) |
| standard | Score-based decision |
| full | Lean toward independent subagent (changes usually complex) |

User can override batching strategy during plan phase.

---

## 5. File Changes & Migration

### 5.1 Change Manifest

| # | File | Change Type | Description |
|---|------|-------------|-------------|
| 1 | `.claude/skills/gitflow-workflow/contract.schema.json` | Modify | Add `standard` to mode enum |
| 2 | `.claude/skills/gitflow-workflow/SKILL.md` | Modify | 3-mode definitions, auto-detect rules, Phase 4 matrix, batching strategy |
| 3 | `.claude/skills/gitflow-workflow/gates.md` | Modify | Add standard mode gate exemptions |
| 4 | `.claude/skills/gitflow-workflow/references.md` | Modify | Add mode-specific context to cross-session recovery |
| 5 | `.claude/skills/gitflow-quality/references/rust.md` | Modify | Gate 5 add `-W clippy::pedantic --all-features`, remove Strict Mode section |
| 6 | `docs/gitflow-workflow-guide.md` | Modify | 3-mode comparison table, Phase 4 matrix, batch execution docs |
| 7 | `CLAUDE.md` | Modify | Update Mode Comparison table to 3 modes |

### 5.2 Backward Compatibility

**Existing contract compatibility:**

| Scenario | Handling |
|----------|----------|
| Active contract `mode = "full"` / `"fast"` | Continue valid, no migration needed |
| Archived contracts | No action |
| New contracts | Use 3-mode system |

**Schema migration:**

```json
{
  "mode": {
    "type": "string",
    "enum": ["full", "standard", "fast"]
  }
}
```

Adding `standard` value doesn't break existing `full`/`fast` contract validation.

### 5.3 Documentation Update Strategy

| Document | Update Content |
|----------|----------------|
| `SKILL.md` | Full rewrite of mode system section, add 3-mode table, auto-detect algorithm, batching strategy |
| `gates.md` | Add standard mode gate exemptions column |
| `gitflow-workflow-guide.md` | Add 3-mode comparison quick reference, update Phase 4 section |
| `CLAUDE.md` | Update "Mode Comparison" table |

### 5.4 Implementation Order

```
Step 1: Schema update (contract.schema.json)
Step 2: Quality gate fix (rust.md) — independently verifiable
Step 3: Core skill update (SKILL.md + gates.md + references.md)
Step 4: Documentation sync (gitflow-workflow-guide.md + CLAUDE.md)
Step 5: Verification — create test contract to validate schema
```

### 5.5 Verification Plan

| Verification | Method | Expected Result |
|--------------|--------|-----------------|
| Schema compatibility | Create contracts with `full`/`standard`/`fast` | All pass JSON Schema validation |
| Quality gate alignment | Run `gitflow-quality` on sample Rust project | clippy output matches CI |
| Auto-detection | Simulate detection on different titles | Correct mode inferred |
| Phase 4 steps | Execute per mode | Step matrix correctly skips |

---

## Out of Scope

- Refactoring to capability-based system (Approach 3) — rejected
- Centralized mode matrix (Approach 2) — premature for 3 modes
- Changing `subagent-driven-development` skill itself — only workflow orchestration changes
- CI pipeline changes — quality gate fix is local-side only

---

## Success Criteria

- [ ] gitflow-workflow supports 3 modes: fast/standard/full
- [ ] Auto-detect from issue title/labels with user confirmation
- [ ] Local quality gate fully aligned with CI (includes pedantic)
- [ ] Phase 4 dynamically adjusts steps based on mode
- [ ] Subagent usage efficiency improved 50%+ (token consumption)
- [ ] Documentation updated: SKILL.md, gates.md, gitflow-workflow-guide.md, CLAUDE.md
