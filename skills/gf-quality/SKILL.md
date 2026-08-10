---
name: gf-quality
description: |
  Use when running pre-delivery quality checks, verifying a branch is
  ready for release, or generating a Quality Report.
  当用户在交付前需运行质量检查、验证分支可交付或生成 Quality Report 时使用。
---

# gf-quality — Language-Agnostic 6-Gate Quality Gate

6-gate fast-fail quality gate. Detects project language, loads the matching toolchain, runs gates in order; first failure stops the chain. Outputs a Quality Report.

## CLI Requirement

**MUST use `gf` CLI, NOT `gh` CLI.**

| CLI | Scope | Platform Support |
|-----|-------|------------------|
| `gf` | This project | GitHub + GitLab + GitCode |
| `gh` | GitHub only | GitHub only |

**Why**: `gf` is the unified CLI for this project. Using `gh` breaks GitLab/GitCode compatibility.

## Preconditions
- `gf` installed: `command -v gf`
- `gf` authenticated: `gf auth status`
## When NOT to Use

| Scenario | Why Not | Use Instead |
|----------|---------|-------------|
| Running quick pre-commit checks only | This skill runs the full 6-gate pipeline, not just pre-commit hooks | `/gf-precommit` for lightweight fmt/lint/test before commit |
| Auto-fixing lint or format issues | This skill reports quality status only, never auto-fixes | User applies fixes manually after reviewing the report |
| Analyzing remote CI/CD pipeline health | This skill runs local quality gates, not remote pipeline analysis | `/gf-pipeline-analyzer` for CI/CD success-rate and failure patterns |
| Performing security audits | This skill checks code quality, not security vulnerabilities | `/gf-security-check` for cargo audit, secret detection, license compliance |
| Publishing report without user confirmation | Report publication to Issues requires explicit consent | Always ask user before publishing Quality Report |
| Running without language detection | This skill requires language detection before gate execution | Non-negotiable — always detect language first |

## Quality Pipeline

```
Language Detection → Gate 1 (build) → Gate 2 (test) → Gate 3 (coverage) → Gate 4 (format) → Gate 5 (static) → Gate 6 (pre-commit) → Report
                      ↓ fail            ↓ fail            ↓ fail            ↓ fail            ↓ fail            ↓ fail
                   STOP + SKIP       STOP + SKIP       STOP + SKIP       STOP + SKIP       STOP + SKIP       STOP + SKIP
```

**Any gate failure = immediate stop. No proceeding to next gate. No auto-fix.**

## Step 1: Language Detection

Run detection BEFORE any gate. See `references/detector.md` for full rules.

Scan root **and 3 levels deep** for marker files (skip `node_modules/`, `target/`, `vendor/`, etc.):

```bash
find . -maxdepth 3 \( -name "Cargo.toml" -o -name "go.mod" -o -name "go.work" \
  -o -name "pom.xml" -o -name "build.gradle" -o -name "settings.gradle" \
  -o -name "pyproject.toml" -o -name "package.json" \) \
  -not -path "*/node_modules/*" -not -path "*/target/*" -not -path "*/vendor/*"
```

| Detected | Load Reference |
|----------|---------------|
| `Cargo.toml` | `references/rust.md` |
| `go.mod` / `go.work` | `references/go.md` |
| `pom.xml` / `build.gradle` | `references/java.md` |
| `pyproject.toml` / `setup.py` | `references/python.md` |
| `package.json` | `references/node.md` |
| None | Run Gate 6 only (pre-commit or N/A) |

After detection, check for workspace configurations (see `references/detector.md` → Workspace Detection).

### Single-Language Project

One language detected (possibly in multiple directories) → load that reference, run gates.
For Rust/Go workspaces: a single command at root covers all members.

### Multi-Language Project

Multiple languages detected → present summary to user:

```
Detected languages:
  1. Rust       → ./ (workspace root + crates/* + apps/server)
  2. Node.js    → ./apps/desktop/ (bun runtime)

Which to check? [1/2/all]
```

- User selects one → run that language's gates
- User selects "all" → run each independently (one failure does NOT block others)
- Generate **aggregate report** at end (see Step 3)

## Step 2: Run Gates

After detection, load the matching `references/<lang>.md` and execute its gate commands.

| # | Gate | What It Checks |
|---|------|---------------|
| 1 | **build** | Code compiles (exit 0) |
| 2 | **test** | All tests pass |
| 3 | **coverage** | Incremental coverage ≥ 80% (`COV_THRESHOLD` overrides) |
| 4 | **format** | No formatting diff |
| 5 | **static** | No lint/analysis warnings |
| 6 | **pre-commit** | All hooks pass (N/A if no `.pre-commit-config.yaml`) |

**Preconditions:**
- `git rev-parse --show-toplevel` succeeds (in a git repo)
- Workspace clean for Gate 2 (`git status --porcelain` empty)
- If a tool is missing → mark gate `SKIPPED`, warn user, do NOT auto-install

## Step 3: Quality Report

### Single-Language Report

```markdown
## Quality Gate Report

- Date: <date>
- Language: <detected language>
- Project: <repo name>

| Gate | Status | Details |
|------|--------|---------|
| 1. build | ✅/❌/N/A | <errors if any> |
| 2. test | ✅/❌/N/A | <failed tests if any> |
| 3. coverage | ✅/❌/N/A | <value vs threshold> |
| 4. format | ✅/❌/N/A | <files if diff> |
| 5. static | ✅/❌/N/A | <warnings if any> |
| 6. pre-commit | ✅/❌/N/A | <hook failures if any> |

### Result
- [ ] ALL CHECKS PASSED — ready for PR
- [ ] WARNINGS — recommend fixing before PR
- [ ] ERRORS — must fix before PR
```

### Multi-Language Aggregate Report

```markdown
## Quality Gate Report (Multi-Language)

**Workspace:** <root>
**Scan depth:** 3 levels
**Date:** <date>
**Languages detected:** <count>

### Detection Summary

| # | Language | Path | Type | Runtime/Build System |
|---|----------|------|------|----------------------|
| 1 | Rust     | ./   | workspace | Cargo (3 crates) |
| 2 | Node.js  | apps/desktop/ | package | bun 1.0.0 |

### Gate Results

| # | Language | Path | Build | Test | Coverage | Format | Static | Pre-commit | Result |
|---|----------|------|-------|------|----------|--------|--------|------------|--------|
| 1 | Rust     | ./   | ✅    | ✅   | ✅ 85%   | ✅     | ✅     | ✅         | PASS   |
| 2 | Node.js  | apps/desktop/ | ✅ | ❌ 2 failed | — | ✅ | ❌ 3 warn | N/A | FAIL |

### Per-Language Details

#### 1. Rust (./, workspace)

| Gate | Status | Details |
|------|--------|---------|
| build | ✅ | 3 crates compiled |
| test | ✅ | 47 tests passed |

#### 2. Node.js (apps/desktop/, bun)

| Gate | Status | Details |
|------|--------|---------|
| test | ❌ | 2 tests failed |

**Failed tests:**
- `test_add`: Expected 5, got 4

### Summary

- ✅ Rust (workspace): ALL CHECKS PASSED
- ❌ Node.js (apps/desktop): 2 test failures

### Actions Required

- [ ] Fix 2 failing tests in `apps/desktop/`

### Overall Result

❌ **QUALITY GATE FAILED** — 1 language has failures
```

**Report only. No auto-fix. No source modifications.**

## Rationalization Table

| Excuse | Reality |
|--------|---------|
| "fmt clean, auto-fix diff" | Report only; user fixes |
| "minor clippy/lint, auto-fix" | Report, do not fix by default |
| "Just publish the report" | User confirms before publishing to Issue |
| "Install tool for them" | Recommend install only |
| "Skip coverage for speed" | Gate 3 mandatory unless tool missing |
| "No project detected, skip all" | Still check Gate 6 (pre-commit) |

## Red Flags — STOP

- 🚩 "Auto-fix all lint/format issues" — report only
- 🚩 "Skip coverage for speed" — mandatory unless tool missing
- 🚩 "Publish report straight to Issue" — require user confirmation
- 🚩 "Run clean command to fix build" — never (cargo clean / mvn clean / go clean)
- 🚩 "Skip language detection, just run cargo" — always detect first

## Common Mistakes

- ❌ **Running formatter to auto-fix** — report only; user executes fixes
- ❌ **Publishing Quality Report without confirmation** — always ask first
- ❌ **Skipping language detection** — may run wrong toolchain
- ❌ **Auto-installing missing tools** — recommend, do not install
- ❌ **Running gates out of order** — fast-fail requires sequential execution

## Error Handling

| Error | Recovery |
|-------|----------|
| Gate N fails | Fast-fail: gates N+1 to 6 = `SKIPPED` |
| Language tool missing | Warn; gate = `SKIPPED` |
| Coverage < threshold | Fast-fail: show value vs threshold |
| No project detected | Run Gate 6 only (pre-commit or N/A) |
| No pre-commit config | Mark Gate 6 = `N/A` |
| Issue file missing | Output report to terminal only |

## See Also

- `gf-precommit` — Gate 6 in isolation
- `gf-commit` — commit after passing gate
- `gf-release` — release workflow (gate is pre-req)
- `gf-security-check` — security layer alongside quality
- `gf-pipeline-analyzer` — CI inspection after quality gate
