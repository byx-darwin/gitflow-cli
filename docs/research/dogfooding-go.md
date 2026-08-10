# Dogfooding Report: Go (zerolog)

**Date:** 2026-08-10
**Project:** [rs/zerolog](https://github.com/rs/zerolog) (commit: latest main)
**Language:** Go 1.23
**Reference:** `skills/gf-quality/references/go.md`

## Detection Accuracy

| Check | Expected | Actual | Correct? |
|-------|----------|--------|----------|
| Language | Go | Go (via `go.mod`) | Yes |
| Marker file | `go.mod` at root | `go.mod` at root | Yes |
| Package structure | Single module | Single module with sub-packages (`diode/`, `hlog/`, `internal/`, etc.) | Yes |

The detection command `find . -maxdepth 3 -name "go.mod"` correctly identified the project as Go with a single module at root.

## Gate Execution Results

| Gate | Command | Result | Details |
|------|---------|--------|---------|
| 1 (build) | `go build ./...` | PASS | Built successfully after downloading dependencies |
| 2 (test) | `go test ./... -race -count=1` | PASS | 7 test packages passed; 5 packages had no test files |
| 3 (coverage) | `go test ./... -coverprofile=coverage.out` | PASS | **87.8%** overall statement coverage (threshold: >= 80%) |
| 4 (format) | `gofmt -l .` | FAIL -> PASS | 13 files were unformatted initially; fixed with `gofmt -w .` |
| 5a (vet) | `go vet ./...` | PASS | No issues found |
| 5b (lint) | `golangci-lint run ./...` | FAIL | 50 issues: 20 errcheck, 26 staticcheck, 2 unused, 1 govet, 1 ineffassign |
| 6 (pre-commit) | `pre-commit run --all-files` | N/A | No `.pre-commit-config.yaml` in repository |

### Coverage Breakdown

| Package | Coverage |
|---------|----------|
| `github.com/rs/zerolog` (root) | 96.0% |
| `internal/json` | 100.0% |
| `internal/cbor` | 96.3% |
| `hlog` | 94.2% |
| `diode` | 93.1% |
| `log` | 88.9% |
| `pkgerrors` | 84.6% |

**Overall: 87.8%** -- passes the >= 80% threshold.

## Issues Encountered

### 1. Flaky Test in diode Package

The `TestFatal` test in `diode/diode_test.go:86` failed on the first coverage run but passed on re-run. This appears to be a timing-dependent test related to the diode's async write behavior.

```
--- FAIL: TestFatal (0.01s)
    diode_test.go:86: Diode Fatal Test failed. got:, want:{"level":"fatal","message":"test"}
```

This highlights a limitation of quality gates: intermittent failures can produce false negatives, especially with race-condition and async tests.

### 2. golangci-lint Reports 50 Issues

The project has no `.golangci.yml` config, so golangci-lint runs with defaults. Most issues are:

- **errcheck (20):** Unchecked return values (e.g., `os.Setenv`, `w.Write`, `io.Closer.Close`)
- **staticcheck (26):** Various code quality suggestions (deprecated APIs, style issues, redundant checks)
- **unused (2):** Unused variable `errExample` and function `zerologToSlogLevel`
- **govet (1):** Inlined constant suggestion
- **ineffassign (1):** Ineffectual assignment

These are pre-existing and not caused by the quality gate itself. The skill correctly reports them for the user to address.

### 3. Formatting Issues

13 files had formatting inconsistencies (indentation, spacing) that `gofmt -w .` resolved. The skill correctly detected this and offered auto-fix.

### 4. Missing Pre-commit Config

No `.pre-commit-config.yaml` exists, so Gate 6 returned N/A. Many Go projects do not use pre-commit hooks.

## Fixes Applied

1. **Format auto-fix:** `gofmt -w .` resolved all 13 unformatted files
2. **No other fixes applied** -- the lint issues are pre-existing code patterns, not things to fix during dogfooding

## Final Result

**Overall: CONDITIONAL PASS** (after format auto-fix)

The project passes the core quality gates (build, test, coverage, vet, format) but fails the optional strict linting gate (golangci-lint). If the skill is configured with golangci-lint available, Gate 5b would show FAIL. If falling back to `go vet` only, all gates pass after format auto-fix.

| Gate | Status |
|------|--------|
| Build | PASS |
| Test | PASS |
| Coverage | PASS (87.8%) |
| Format | PASS (after auto-fix) |
| Static (vet) | PASS |
| Static (golangci-lint) | FAIL (50 issues) |
| Pre-commit | N/A |

## Lessons Learned

1. **Flaky tests are a real concern.** The diode `TestFatal` failure only appeared during coverage runs, not normal test runs. The skill should recommend re-running failed tests before concluding they are genuine failures.

2. **golangci-lint is strict by default.** A project without a `.golangci.yml` config will likely have many issues. The skill correctly identifies this as a FAIL and lists the issues.

3. **`gofmt` auto-fix is safe.** Formatting issues are purely cosmetic and can be auto-fixed without risk. The skill's design of showing the diff before auto-fixing is appropriate.

4. **No pre-commit config is common in Go.** This gate should default to N/A rather than FAIL when no config exists, which is what the skill does.

5. **Coverage threshold of 80% is reasonable.** Zerolog's 87.8% is well above the threshold, and the lower-coverage packages (`pkgerrors` at 84.6%, `log` at 88.9%) are still above the line.
