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

## Evidence Grading

| Tier | Meaning | Hard rule |
|---|---|---|
| `Measured` | Command run this session | No output → downgrade to `Inferred`. |
| `Inferred` | Read from code/config/diff | Must cite `path:line`. |
| `Unverified` | Not verified this run | Must state why; never omitted. |

Reused verbatim from `gf-walkthrough`/`gf-smell`. Every gate result in Step 3's
report carries one of these three tiers. Never label a gate `Measured`
without that gate's command output present in the report this session.

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

| # | Gate | What It Checks | Evidence Tier |
|---|------|---------------|----------------|
| 1 | **build** | Code compiles (exit 0) | `Measured` when the command ran this session; `SKIPPED`/`N/A` → `Unverified` with reason |
| 2 | **test** | All tests pass | `Measured` when the command ran this session; `SKIPPED`/`N/A` → `Unverified` with reason |
| 3 | **coverage** | Total coverage ≥ 80% (`COV_THRESHOLD` overrides); line coverage for Rust, Python, Java, Ruby and Node.js, **statement** coverage for Go — the units differ, so always report which; N/A when the change touches no source file of that language | Coverage value `Measured` from tool output; the threshold itself `Inferred` — cite `COV_THRESHOLD` or this row's 80% default |
| 4 | **format** | No formatting diff | `Measured` when the command ran this session; `SKIPPED` → `Unverified` with reason |
| 5 | **static** | No lint/analysis warnings | `Measured` when the command ran this session; `SKIPPED` → `Unverified` with reason |
| 6 | **pre-commit** | All hooks pass (N/A if no `.pre-commit-config.yaml`) | `Measured` when the command ran this session; `N/A` → `Unverified` (no config present) |

**Preconditions:**
- `git rev-parse --show-toplevel` succeeds (in a git repo)
- Workspace clean for Gate 2 (`git status --porcelain` empty)
- If a tool is missing → mark gate `SKIPPED`, warn user, do NOT auto-install
- Gate 3 is `N/A` when the change set contains no source file of the detected language. Determine the base
  commit and the change set with:

  ```bash
  base_ref="${BASE_REF:-}"
  if [ -z "$base_ref" ]; then
    for ref in origin/main origin/master origin/HEAD main master; do
      if git rev-parse --verify --quiet "$ref" >/dev/null; then base_ref="$ref"; break; fi
    done
  fi
  [ -n "$base_ref" ] || { echo "no base ref resolved"; }

  base_commit=""
  if [ -n "$base_ref" ]; then
    base_commit="$(git merge-base HEAD "$base_ref" 2>/dev/null || true)"
  fi

  if [ -z "$base_commit" ] || [ "$base_commit" = "$(git rev-parse HEAD)" ]; then
    change_set="__WHOLE_TREE__"   # HEAD is the base (or no base resolved): run Gate 3 over everything
  else
    change_set="$(git diff --name-only "$base_commit")"
  fi
  ```

  Never write a bare `git merge-base HEAD "$(…)"` with an unchecked `${BASE_REF:-origin/main}`: GitLab and
  GitCode projects routinely use a default branch other than `main`, `origin/main` then fails to resolve, the
  command substitution collapses to an empty string, and the diff errors out instead of reporting a status.
- **`__WHOLE_TREE__` means Gate 3 runs normally, not `N/A`.** When `HEAD` *is* the base branch — the release
  check named in this skill's `description` — the merge-base is `HEAD` itself and the diff is necessarily empty
  (Gate 2's precondition already requires a clean worktree). That empty diff means "there is nothing to compare
  against", not "this change set touches no source file of that language". Treating it as `N/A` would silently
  delete the coverage gate exactly when a release is being verified. Run Gate 3 against the whole tree instead.
- Report `N/A` distinctly from a tool-missing `SKIPPED` — they have different causes and different follow-ups.
  `N/A` = the change set contains no source file of that language. `SKIPPED` = the coverage tool or its
  threshold configuration is absent, so nothing was measured. A missing tool or a missing threshold config is
  never `N/A`.

## Step 3: Quality Report

### Single-Language Report

```markdown
## Quality Gate Report

- Date: <date>
- Language: <detected language>
- Project: <repo name>

| Gate | Status | Evidence | Details |
|------|--------|----------|---------|
| 1. build | ✅/❌/N/A | Measured/Unverified | <errors if any> |
| 2. test | ✅/❌/N/A | Measured/Unverified | <failed tests if any> |
| 3. coverage | ✅/❌/N/A | Measured/Inferred/Unverified | <value vs threshold> |
| 4. format | ✅/❌/N/A | Measured/Unverified | <files if diff> |
| 5. static | ✅/❌/N/A | Measured/Unverified | <warnings if any> |
| 6. pre-commit | ✅/❌/N/A | Measured/Unverified | <hook failures if any> |

### Failing Tests (Gate 2 only)

| 失败用例 | 最后修改 commit | 是否 base 祖先 |
|---|---|---|

Never write "unrelated" / "与本次改动无关" without the commit hash and
ancestry check below (reused from `gf-walkthrough`):

```bash
H=$(git log -1 --format=%H -- "<test file>")
git merge-base --is-ancestor "$H" "$BASE" && echo "先于本次交付存在" || echo "本次引入"
```

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

| Gate | Status | Evidence | Details |
|------|--------|----------|---------|
| build | ✅ | Measured | 3 crates compiled |
| test | ✅ | Measured | 47 tests passed |

#### 2. Node.js (apps/desktop/, bun)

| Gate | Status | Evidence | Details |
|------|--------|----------|---------|
| test | ❌ | Measured | 2 tests failed |

**Failed tests:**

| 失败用例 | 最后修改 commit | 是否 base 祖先 |
|---|---|---|
| `test_add` | `<hash>` | `先于本次交付存在` / `本次引入` |

Never write "unrelated" without the commit hash and ancestry check (see
Single-Language Report → Failing Tests above for the command).

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
| "看似与改动无关，写 unrelated" | 禁止；必须先跑祖先检查并附 commit hash |
| "SKIPPED 也算 Measured，反正跑过检测" | `SKIPPED` = 工具缺失、未产出真实输出，只能标 `Unverified` |

## Red Flags — STOP

- 🚩 "Auto-fix all lint/format issues" — report only
- 🚩 "Skip coverage for speed" — mandatory unless tool missing
- 🚩 "Publish report straight to Issue" — require user confirmation
- 🚩 "Run clean command to fix build" — never (cargo clean / mvn clean / go clean)
- 🚩 "Skip language detection, just run cargo" — always detect first
- 🚩 "报告写 unrelated 却没给 commit" — 禁止，先跑祖先检查
- 🚩 "把 N/A/SKIPPED 标成 Measured" — 无输出即非 Measured

## Common Mistakes

- ❌ **Running formatter to auto-fix** — report only; user executes fixes
- ❌ **Publishing Quality Report without confirmation** — always ask first
- ❌ **Skipping language detection** — may run wrong toolchain
- ❌ **Auto-installing missing tools** — recommend, do not install
- ❌ **Running gates out of order** — fast-fail requires sequential execution
- ❌ **失败测试写「与改动无关」但无 commit 溯源** — 必须先跑祖先检查
- ❌ **把跳过的 Gate 标成 Measured** — 无输出即为 `Unverified`

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
