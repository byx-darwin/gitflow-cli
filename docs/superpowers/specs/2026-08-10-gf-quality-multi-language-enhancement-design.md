# gf-quality Multi-Language Quality Probe Enhancement Design

**Date:** 2026-08-10
**Issue:** #171
**Approach:** A (Incremental Enhancement)

## Overview

Enhance `gf-quality` skill to provide comprehensive multi-language quality gate support with configuration guides, troubleshooting documentation, workspace detection, and example projects for validation.

## Problem Statement

Current state (from Issue #171):
- ✅ Already supports Rust/Go/Java/Python/Node detection
- ✅ Has reference files per language with gate commands
- ✅ Has runtime detection for Node.js (bun/pnpm/yarn/npm)
- ✅ Has Maven + Gradle support for Java

Missing:
- ❌ Configuration guides for each language
- ❌ Troubleshooting/FAQ documentation
- ❌ Example projects for validation
- ❌ Enhanced monorepo/multi-package analysis
- ❌ Independent per-language reports in aggregate output

## Design Decisions

### Decision 1: Configuration Guide Format

**Chosen:** Inline in `references/<lang>.md`

**Rationale:**
- Keeps all language-specific knowledge colocated
- Single file per language is easier to navigate
- Avoids file proliferation
- Maintains existing pattern

**Structure per language:**
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

**Content per language:**

**Rust:**
- `rustfmt.toml` example (max_width, edition, imports_layout)
- `clippy.toml` example (cognitive-complexity-threshold, too-many-arguments-threshold)
- `Cargo.toml` workspace config example
- Environment: `COV_THRESHOLD`, `RUSTFLAGS`, `CARGO_HOME`

**Go:**
- `.golangci.yml` example (linters enable/disable, severity)
- `go.mod` module config (go version, replace directives)
- Environment: `GOPROXY`, `GONOSUMCHECK`, `GOFLAGS`

**Node.js:**
- `.prettierrc` example (semi, trailingComma, printWidth)
- `.eslintrc.json` example (extends, rules, env)
- `tsconfig.json` example (for TypeScript projects)
- `package.json` scripts section best practices
- Environment: `NODE_ENV`, `npm_config_*`

**Python:**
- `pyproject.toml` example (tool.ruff, tool.black, tool.pytest sections)
- `.ruff.toml` example (line-length, select, ignore)
- `setup.py` vs `pyproject.toml` guidance
- Environment: `PYTHONPATH`, `PYTHONDONTWRITEBYTECODE`, `VIRTUAL_ENV`

**Java:**
- `pom.xml` plugin config (JaCoCo, Spotless, PMD examples)
- `build.gradle` plugin config (jacoco, spotless, checkstyle examples)
- `spotbugs-exclude.xml` example
- Environment: `MAVEN_OPTS`, `GRADLE_OPTS`, `JAVA_HOME`

### Decision 2: Troubleshooting Documentation

**Chosen:** Inline in `references/<lang>.md`

**Rationale:**
- Colocated with gate commands and configuration
- Easy to find when debugging language-specific issues
- Maintains single-file-per-language pattern

**Structure per language:**
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

**Content per language:**

**Rust:**
- Common errors: `cargo tarpaulin` not found, nightly toolchain missing, workspace member build failures
- Exit codes: 101 (compilation error), 102 (test failure), 1 (clippy warnings with -D)
- FAQ: "Why does coverage show 0%?", "How to skip doc tests?", "Workspace build slow?"
- Performance: `cargo build --workspace --quiet`, parallel test execution, incremental compilation

**Go:**
- Common errors: `golangci-lint` not found, module download failures, race condition test failures
- Exit codes: 1 (build/test failure), 2 (vet errors)
- FAQ: "Why does `go test` hang?", "How to update dependencies?", "Module proxy issues?"
- Performance: `go test -parallel`, build caching, `go mod vendor` for offline builds

**Node.js:**
- Common errors: `npm install` permission denied, lock file conflicts, TypeScript compilation errors
- Exit codes: 1 (test failure), 2 (lint errors), 127 (command not found)
- FAQ: "npm vs yarn vs pnpm?", "How to clear node_modules cache?", "TypeScript strict mode?"
- Performance: `npm ci` instead of `npm install`, parallel test runners, skip dev dependencies in CI

**Python:**
- Common errors: `pip install` permission denied, virtual environment not activated, import errors
- Exit codes: 1 (test failure), 2 (syntax error), 127 (command not found)
- FAQ: "ruff vs black vs pylint?", "How to manage multiple Python versions?", "pytest fixtures?"
- Performance: `pytest-xdist` for parallel tests, `--cov-report=term-missing` for faster coverage

**Java:**
- Common errors: Maven plugin not found, Gradle wrapper missing, JVM memory issues
- Exit codes: 1 (build failure), 2 (test failure), 137 (OOM killed)
- FAQ: "Maven vs Gradle?", "How to skip tests temporarily?", "JaCoCo coverage report location?"
- Performance: Maven parallel builds (`-T 1C`), Gradle daemon, incremental compilation

### Decision 3: Example Projects

**Chosen:** Create minimal example projects under `examples/quality-gate/{go,node,python}/`

**Rationale:**
- Provides reproducible validation targets
- Serves as documentation samples
- Enables future automated testing
- Minimal overhead to create and maintain

**Structure:**
```
examples/quality-gate/
├── go/
│   ├── go.mod
│   ├── main.go
│   ├── main_test.go
│   └── README.md
├── node/
│   ├── package.json
│   ├── index.js
│   ├── index.test.js
│   ├── .prettierrc
│   ├── .eslintrc.json
│   └── README.md
└── python/
    ├── pyproject.toml
    ├── src/
    │   └── example/
    │       ├── __init__.py
    │       └── main.py
    ├── tests/
    │   └── test_main.py
    └── README.md
```

Each example:
- Minimal valid project with one function and one test
- Includes all necessary config files for quality gates
- README with setup and validation instructions
- Expected result: ALL CHECKS PASSED

### Decision 4: Workspace Detection Enhancement

**Chosen:** Deep scan (3 levels) + workspace config detection

**Rationale:**
- Real-world monorepos often have nested projects beyond 2 levels
- Workspace configs (Cargo.toml [workspace], go.work, npm workspaces) need special handling
- Project tree output improves transparency and debugging

**Enhancements:**

1. **Deeper scan:** Increase from 2 to 3 levels
2. **Workspace marker detection:**
   - `go.work` → Go workspace
   - `Cargo.toml` with `[workspace]` → Rust workspace
   - `settings.gradle` → Gradle multi-project
   - `package.json` with `workspaces` → npm/yarn workspace
3. **Project tree output:** Visual tree showing detected languages and paths
4. **Workspace-aware execution:** Single command for workspace projects, independent for packages

**Enhanced detection command:**
```bash
# Scan root + 3 levels deep for all marker files
find . -maxdepth 3 \( \
  -name "Cargo.toml" -o \
  -name "go.mod" -o \
  -name "go.work" -o \
  -name "pom.xml" -o \
  -name "build.gradle" -o \
  -name "build.gradle.kts" -o \
  -name "settings.gradle" -o \
  -name "pyproject.toml" -o \
  -name "setup.py" -o \
  -name "package.json" -o \
  -name "Gemfile" \
\) -not -path "*/node_modules/*" -not -path "*/target/*" \
   -not -path "*/.git/*" -not -path "*/vendor/*" \
   -not -path "*/dist/*" -not -path "*/build/*"
```

### Decision 5: Aggregate Report Enhancement

**Chosen:** Enhanced multi-language report with per-language details

**Structure:**
```markdown
## Quality Gate Report (Multi-Language)

**Workspace:** <root>
**Scan depth:** <levels>
**Date:** <date>
**Languages detected:** <count>

### Detection Summary

| # | Language | Path | Type | Runtime/Build System |
|---|----------|------|------|----------------------|

### Gate Results

| # | Language | Path | Build | Test | Coverage | Format | Static | Pre-commit | Result |
|---|----------|------|-------|------|----------|--------|--------|------------|--------|

### Per-Language Details

#### 1. <Language> (<path>, <type>)

| Gate | Status | Details |
|------|--------|---------|

**Failed tests:** (if any)
**Lint warnings:** (if any)

### Summary

- ✅ <Language> (<path>): ALL CHECKS PASSED
- ❌ <Language> (<path>): <failures>

### Actions Required

- [ ] <action-1>
- [ ] <action-2>

### Overall Result

✅/❌ **QUALITY GATE PASSED/FAILED** — <summary>
```

**Key enhancements:**
- Detection summary table with workspace context
- Per-language detailed sections
- Inline failure details (test names, lint warnings)
- Consolidated actions required
- Clear overall result

## Architecture

```
skills/gf-quality/
├── SKILL.md (enhanced: workspace-aware execution flow)
├── references/
│   ├── detector.md (enhanced: 3-4 level scan + workspace detection + project tree)
│   ├── rust.md (existing + Configuration + Troubleshooting)
│   ├── go.md (existing + Configuration + Troubleshooting)
│   ├── node.md (existing + Configuration + Troubleshooting)
│   ├── python.md (existing + Configuration + Troubleshooting)
│   └── java.md (existing + Configuration + Troubleshooting)
└── examples/ (new)
    └── quality-gate/
        ├── go/
        ├── node/
        └── python/
```

## Implementation Plan

### Phase 1: Configuration Guides (5 files)

1. Add `## Configuration` section to `references/rust.md`
2. Add `## Configuration` section to `references/go.md`
3. Add `## Configuration` section to `references/node.md`
4. Add `## Configuration` section to `references/python.md`
5. Add `## Configuration` section to `references/java.md`

### Phase 2: Troubleshooting Sections (5 files)

1. Add `## Troubleshooting` section to `references/rust.md`
2. Add `## Troubleshooting` section to `references/go.md`
3. Add `## Troubleshooting` section to `references/node.md`
4. Add `## Troubleshooting` section to `references/python.md`
5. Add `## Troubleshooting` section to `references/java.md`

### Phase 3: Workspace Detection (1 file)

1. Enhance `references/detector.md` with:
   - Deeper scan (3-4 levels)
   - Workspace config detection
   - Project tree output
   - Workspace-aware execution guidance

### Phase 4: Example Projects (3 projects)

1. Create `examples/quality-gate/go/` with minimal Go project
2. Create `examples/quality-gate/node/` with minimal Node project
3. Create `examples/quality-gate/python/` with minimal Python project

### Phase 5: SKILL.md Enhancement (1 file)

1. Update `skills/gf-quality/SKILL.md` with:
   - Workspace-aware execution flow
   - Enhanced aggregate report format
   - Updated detection summary

### Phase 6: Validation (3 phases)

1. **Example validation:** Run `gf quality` against each example project
2. **Documentation review:** Proofread all reference files
3. **Dogfooding:** Validate against 3 real-world non-Rust projects (Go, Node, Python)

### Phase 7: Documentation Sync

1. Update `docs/references/gf-quality-params.md`
2. Update `docs/research/skill-analysis-gf-quality.md`
3. Verify cross-references

## Success Criteria

- ✅ 3 example projects created and validated
- ✅ All 5 language references have Configuration + Troubleshooting sections
- ✅ Workspace detection works for Rust/npm/Go workspaces
- ✅ Multi-language aggregate report shows per-language details
- ✅ 3 dogfooding reports documented (Go, Node, Python)
- ✅ All documentation updated and cross-referenced

## Testing Strategy

### Example Project Validation

- Create each example project
- Run `gf quality` in each
- Verify ALL CHECKS PASSED

### Configuration Guide Validation

- Proofread each Configuration section
- Verify config file examples are syntactically valid
- Verify environment variable descriptions are accurate

### Troubleshooting Validation

- Proofread each Troubleshooting section
- Verify error messages and fix commands are accurate
- Verify FAQ answers are helpful

### Workspace Detection Validation

- Test with Rust workspace (this repo)
- Test with npm workspace (create test monorepo)
- Test with Go workspace (create test go.work)
- Verify project tree output is clear
- Verify aggregate report format

### Dogfooding Validation

- Identify 3 non-Rust projects (1 Go, 1 Node, 1 Python) — can be public GitHub repos, user's own projects, or sample projects from documentation
- Run `gf quality` against each
- Document results in `docs/research/dogfooding-<lang>.md`
- Include: detection accuracy, gate execution, issues encountered, fixes applied, final result

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Language tools not installed locally | Mark gate as SKIPPED, document in dogfooding report |
| Example projects fail validation | Debug and fix before proceeding |
| Workspace detection fails | Fall back to current 2-level scan |
| Configuration examples outdated | Use current tool versions, test syntax |
| Troubleshooting content inaccurate | Test error scenarios locally |

## Out of Scope

- Programmatic validation of skill behavior (no automated tests for skill execution)
- Support for additional languages (Ruby, PHP, etc.)
- Integration with CI/CD pipelines
- Auto-fix capabilities (report-only remains the policy)

## Future Enhancements

- Add Ruby/PHP support if requested
- Create automated skill validation framework
- Add video tutorials for each language setup
- Integrate with `gf-pipeline-analyzer` for CI quality trends

## References

- Issue #171: https://github.com/byx-darwin/gitflow-cli/issues/171
- Current skill: `skills/gf-quality/SKILL.md`
- Current references: `skills/gf-quality/references/`
- Analysis report: `docs/research/skill-analysis-gf-quality.md`
- Quality params: `docs/references/gf-quality-params.md`
