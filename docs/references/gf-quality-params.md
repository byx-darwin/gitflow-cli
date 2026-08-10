# Quality Gate Reference — Parameters and Multi-Language Support

## Quality Report Template

```markdown
## Quality Report — YYYY-MM-DD

| Check       | Status | Details                                   |
|-------------|--------|-------------------------------------------|
| build       | ✅     | 0 errors, 0 warnings                      |
| test        | ✅     | 47 passed, 0 failed                       |
| coverage    | ✅     | 85.3% (threshold: 80%)                    |
| format      | ✅     | No diff                                   |
| static      | ✅     | No warnings                               |
| pre-commit  | ✅ / N/A | All hooks passed / No .pre-commit-config.yaml |
```

**Overall conclusion:**

- `**Result: ✅ ALL CHECKS PASSED — Ready for delivery**`
- `**Result: ❌ QUALITY GATE FAILED — Return to Phase 2 for fixes**`

Skipped gates in failed reports are marked `⏭️ SKIPPED`.

## Multi-Language Toolchains

When the project manifest is **not** `Cargo.toml`, swap commands per language.

| Gate      | Node.js                       | Python            |Go                          | Java                          |
|-----------|-------------------------------|-------------------|----------------------------|-------------------------------|
| build     | `npm run build`               | `python -m py_compile src/` | `go build ./...`    | `mvn compile -q`              |
| test      | `npm test`                    | `pytest`          | `go test ./...`            | `mvn test`                    |
| coverage  | `npx jest --coverage`         | `pytest --cov`    | `go test -coverprofile=...`| `mvn verify -Pcoverage`       |
| format    | `npx prettier --check .`      | `black --check .` | `test -z "$(gofmt -l .)"`  | `mvn spotless:check`          |
| static    | `npx eslint .`                | `ruff check .`    | `golangci-lint run`        | `mvn pmd:check`               |
| pre-commit| `npx lint-staged`             | `pre-commit run --all-files` | `pre-commit run --all-files` | `pre-commit run --all-files` |

Use `rustfmt.toml`, `clippy.toml`, `eslintrc`, etc. to infer convention where installed.

## Issue Publishing Behavior

When `.claude/gh-issue/current-issue.txt` exists and `gf` is on `PATH`:

1. Render report to temp file `quality-report.md`.
2. Ask user to confirm publish.
3. On yes: `gf issue comment "${ISSUE_NUMBER}" --body-file quality-report.md`, then `rm -f quality-report.md`.

Otherwise: output report to terminal only.

## Fix Commands by Gate (shown to user, not run automatically)

| Gate      | User Fix Command                              |
|-----------|-----------------------------------------------|
| build     | `cargo build --workspace` — read errors        |
| test      | `cargo test --workspace -- --nocapture`        |
| coverage  | Add tests for untested paths                   |
| format    | `cargo +nightly fmt`                           |
| static    | `cargo clippy --fix --workspace --all-targets` |
| pre-commit| `pre-commit run --all-files` — inspect failures |

## Workspace Detection

The skill scans the project root **and 3 levels deep** for language marker files, excluding build artifact and dependency directories (`node_modules/`, `target/`, `.git/`, `vendor/`, `dist/`, `build/`).

### Workspace Markers

After detecting marker files, the skill checks for workspace configurations:

| Marker | Workspace Type | Execution Strategy |
|--------|----------------|-------------------|
| `go.work` | Go workspace | Run `go build/test` at root (covers all modules) |
| `Cargo.toml` with `[workspace]` | Rust workspace | Run `cargo build/test` at root (covers all crates) |
| `settings.gradle` / `settings.gradle.kts` | Gradle multi-project | Run `./gradlew build` at root (covers all subprojects) |
| `package.json` with `"workspaces"` | npm/yarn/pnpm workspace | Run gates in each workspace package independently |

### Workspace-Aware Execution

- **Rust workspace:** Single `cargo build/test/clippy` at root covers all members
- **Go workspace:** Single `go build/test` at root covers all modules
- **Gradle multi-project:** Single `./gradlew build` at root covers all subprojects
- **npm/yarn workspace:** Run gates in each workspace package independently (one failure does NOT block others)

### Project Tree Output

Before running gates, the skill outputs a visual tree showing detected languages and their paths. See `skills/gf-quality/references/detector.md` for the full detection rules and tree generation commands.

## Configuration Guide Structure

Each language reference (`skills/gf-quality/references/<lang>.md`) includes a `## Configuration` section with the following subsections:

```markdown
## Configuration

### Tool Setup

| Tool | Install | Config File | Required |
|------|---------|-------------|----------|

### Config File Examples

#### <config-file>

```<lang>
// Example configuration
```

### Environment Variables

| Variable | Effect | Default |
|----------|--------|---------|

### Language-Specific Notes

- <nuance-1>
- <nuance-2>
```

**Covered per language:**

| Language | Config Files Documented |
|----------|------------------------|
| Rust | `rustfmt.toml`, `clippy.toml`, `Cargo.toml` workspace, `COV_THRESHOLD` |
| Go | `.golangci.yml`, `go.mod`, `GOPROXY` |
| Node.js | `.prettierrc`, `.eslintrc.json`, `tsconfig.json`, `package.json` scripts |
| Python | `pyproject.toml`, `.ruff.toml`, virtual environment |
| Java | `pom.xml`, `build.gradle`, `spotbugs-exclude.xml`, `JAVA_HOME` |

## Troubleshooting Section Structure

Each language reference (`skills/gf-quality/references/<lang>.md`) includes a `## Troubleshooting` section with the following subsections:

```markdown
## Troubleshooting

### Common Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `<error-message>` | <root-cause> | `<fix-command>` |

### Exit Code Reference

| Code | Meaning | Action |
|------|---------|--------|

### FAQ

**Q: <common-question>?**
A: <answer>

### Performance Tips

- <tip-1>
- <tip-2>
```

**Covered per language:**

| Language | Common Errors | FAQ Topics |
|----------|--------------|------------|
| Rust | `cargo-tarpaulin` missing, nightly toolchain missing, compilation errors | Coverage shows 0%, skip doc tests, slow workspace builds |
| Go | `golangci-lint` missing, module download failures, race conditions | Test hangs, dependency updates, module proxy issues |
| Node.js | Permission denied, lock file conflicts, TypeScript errors | npm vs yarn vs pnpm, cache clearing, strict mode |
| Python | pip missing, permission denied, import errors | ruff vs black vs pylint, Python version management, pytest fixtures |
| Java | Plugin not found, permission denied, OOM errors | Maven vs Gradle, skip tests, JaCoCo report location |

## Enhanced Aggregate Report Format

When multiple languages are detected in a project, the skill generates an enhanced aggregate report with the following structure:

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

**Key features of the enhanced report:**

- **Detection Summary:** Lists all detected languages with their paths, project type (workspace vs package), and runtime/build system
- **Gate Results Matrix:** Consolidated table showing all languages and their gate statuses side by side
- **Per-Language Details:** Each language gets its own detailed section with gate-by-gate breakdown, inline failure details (test names, lint warnings), and specific error information
- **Consolidated Actions:** All required fixes across languages are gathered into a single actionable checklist
- **Clear Overall Result:** Single PASS/FAIL verdict with a one-line summary

This format is defined in `skills/gf-quality/SKILL.md` Step 3 and `skills/gf-quality/references/detector.md`.

## Cross-Reference Index

| Document | Path | Purpose |
|----------|------|---------|
| SKILL.md | `skills/gf-quality/SKILL.md` | Core skill definition and execution flow |
| Detector | `skills/gf-quality/references/detector.md` | Language detection rules, workspace detection, project tree |
| Rust Reference | `skills/gf-quality/references/rust.md` | Rust gate commands, configuration, troubleshooting |
| Go Reference | `skills/gf-quality/references/go.md` | Go gate commands, configuration, troubleshooting |
| Node.js Reference | `skills/gf-quality/references/node.md` | Node.js gate commands, configuration, troubleshooting |
| Python Reference | `skills/gf-quality/references/python.md` | Python gate commands, configuration, troubleshooting |
| Java Reference | `skills/gf-quality/references/java.md` | Java gate commands, configuration, troubleshooting |
| Design Spec | `docs/superpowers/specs/2026-08-10-gf-quality-multi-language-enhancement-design.md` | Feature design and decisions |
| Implementation Plan | `docs/superpowers/plans/2026-08-10-gf-quality-multi-language-enhancement.md` | Task-by-task implementation guide |
| Skill Analysis | `docs/research/skill-analysis-gf-quality.md` | Original skill audit (2026-07-07) and Issue #171 resolution (2026-08-10) |
| Dogfooding (Go) | `docs/research/dogfooding-go.md` | Real-world validation against zerolog |
| Dogfooding (Node) | `docs/research/dogfooding-node.md` | Real-world validation against cookie |
| Dogfooding (Python) | `docs/research/dogfooding-python.md` | Real-world validation against click |
| Example (Go) | `examples/quality-gate/go/` | Minimal Go project for validation |
| Example (Node) | `examples/quality-gate/node/` | Minimal Node.js project for validation |
| Example (Python) | `examples/quality-gate/python/` | Minimal Python project for validation |
