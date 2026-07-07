---
name: gitflow-quality
description: |
  Use when running the 6-gate quality check (build, test, coverage, format, lint, pre-commit) before delivery, verifying a branch is ready for release, or generating a Quality Report.
  当用户在交付前需运行 build/test/coverage/format/static/pre-commit 6 项质量检查、验证分支可交付或生成 Quality Report 时使用。
---

# gitflow-quality

## Overview

6-gate fast-fail quality gate. Run gates in order; first failure stops the chain. Output a `Quality Report`. Optionally publish to a linked Issue **after user confirmation**. Full report template and non-Rust toolchains: `docs/references/gitflow-quality-params.md`.

## When to Use

| Trigger | 中文 | Redirect |
|---------|------|----------|
| run checks / quality gate | 跑检查, 质量闸门 | — |
| is this ready / safe to release | 能交付吗, 可以发布吗 | — |
| pre-commit failed | pre-commit 挂了 | → `gitflow-precommit` |
| just commit | — | → `gitflow-commit` |

## Core Pattern

```bash
cargo build --workspace --quiet                 # 1. build
cargo test --workspace --quiet                  # 2. test
cargo tarpaulin --workspace 2>&1 | tail -3      # 3. coverage > 80%
cargo +nightly fmt -- --check                  # 4. format
cargo clippy --workspace --all-targets -- -D warnings  # 5. static
pre-commit run --all-files                      # 6. pre-commit (or N/A)
```

Env var `COV_THRESHOLD` or `COVERAGE_THRESHOLD` overrides 80% default.

## Quick Reference / Preconditions

| # | Gate | Pass | Precondition |
|---|------|------|-------------|
| 1 | build | exit 0 | `git rev-parse --show-toplevel` |
| 2 | test | all pass | clean workspace (`git status --porcelain` empty) |
| 3 | coverage | > 80% | `cargo tarpaulin` installed (else skip, warn) |
| 4 | format | exit 0, no diff | — |
| 5 | static | exit 0, no warnings | — |
| 6 | pre-commit | all hooks pass | `.pre-commit-config.yaml` exists |

## Rationalization Excuse

| Excuse | Reality |
|--------|---------|
| "fmt clean, auto-fix diff" | Report only; user fixes |
| "minor clippy, auto-fix" | Report, do not fix by default |
| "Just publish the report" | User confirms first |
| "Install tarpaulin for them" | Recommend install only |
| "Skip pre-commit when no config" | Correct — mark N/A |

## Red Flags

- 🚩 "Auto-fix all lint issues" — report only
- 🚩 "Skip coverage for speed" — gate 3 mandatory unless opt-out
- 🚩 "Publish report straight to Issue" — require confirmation
- 🚩 "Run cargo clean to fix build" — never

## Error Handling

| Error | Recovery |
|-------|----------|
| Gate 1 fails | Fast-fail → gates 2-6 = `SKIPPED` |
| Gate 2 fails | Fast-fail → list failed tests |
| `tarpaulin` missing | Warn; gate = `SKIPPED` |
| Coverage < threshold | Fast-fail → show value vs threshold |
| Format diff | Fast-fail → list files |
| Clippy warnings | Fast-fail → summarize by file |
| No pre-commit config | Mark `N/A` |
| Issue file missing | Output terminal only |
| `gitflow-cli` missing | Output terminal only |

## Flowchart

```mermaid
flowchart TD
  A[Start] -->{workspace clean?}
  |dirty| ASK[Commit or stash?] --> G1[cargo build]
  |clean| G1
  G1-->|fail|F1[SKIPPED 2-6] --> REPORT
  G1-->|ok| G2[cargo test]
  G2-->|fail|F2[SKIPPED 3-6]
  G2-->|ok|{tarpaulin?}
  |no| W[Warn, skip gate 3] --> G4
  |yes| G3b{coverage > threshold?}
  G3b-->|below|F3[SKIPPED 4-6]
  G3b-->|ok| G4[cargo fmt --check]
  G4-->|diff|F4[SKIPPED 5-6]
  G4-->|ok| G5[cargo clippy -- -D warnings]
  G5-->|warn|F5[SKIPPED 6]
  G5-->|ok|{pre-commit config?}
  |no| NA[Mark N/A] --> REPORT
  |yes| G6b[pre-commit run --all-files]
  G6b-->|fail|F6[Fail gate 6]
  G6b-->|ok|PASS[All pass]
  F2 & F3 & F4 & F5 & F6 --> REPORT
  REPORT[Render report] -->{Issue file?}
  |no| TERM[Terminal only]
  |yes| CONFIRM{Confirm publish?}
  CONFIRM-->|yes|PUB[gitflow-cli issue comment] --> TERM
  CONFIRM-->|no| TERM
  TERM--> END
```

## Test Scenarios

### 1: Happy Path
- **Given** clean workspace + tarpaulin · **When** "run checks" · **Then** All 6 gates pass → "ALL CHECKS PASSED"

### 2: Negative
- **Given** "run pre-commit only" · **Then** NOT loaded → `gitflow-precommit`

### 3: Boundary
- **Given** coverage = 80.0% · **Then** Gate 3 PASSED

### 4: Error
- **Given** gate 2 fails · **Then** `test=failed`, gates 3-6 = `SKIPPED`
### 5: Publish Consent
- **Given** `.claude/gh-issue/current-issue.txt` exists · **Then** Show summary; ask consent before publish

## Success Criteria

- [ ] All 6 gates attempted (or N/A) before report
- [ ] Fast-fail enforced
- [ ] Pre-commit absent → `N/A`
- [ ] Report has: date, gates, Result
- [ ] Issue publish only after explicit confirmation
- [ ] No source modified · no auto-fix · no installs

## See Also

- `gitflow-precommit` — gate 6 in isolation
- `gitflow-commit` — commit after passing gate
- `gitflow-release` — release workflow (gate is pre-req)
- `gitflow-security-check` — security layer alongside quality
- `gitflow-pipeline-analyzer` — CI inspection after quality gate

## Trigger Keywords

| English | 中文 |
|---------|------|
| quality gate, run checks | 质量闸门, 跑检查, 质量检查 |
| is this ready, safe to release | 能交付吗, 可以发布吗 |
| pre-commit failed, coverage too low | pre-commit 挂了, 覆盖率不够 |
| clippy warnings, format check | clippy 警告, 格式检查 |

## Common Mistakes

- ❌ **Running `cargo fmt` to auto-fix** — report-only; user executes fixes.
- ❌ **Publishing Quality Report without confirmation** — always ask first.
