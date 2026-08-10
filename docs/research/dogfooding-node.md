# Dogfooding Report: Node.js / TypeScript (cookie)

**Date:** 2026-08-10
**Project:** [jshttp/cookie](https://github.com/jshttp/cookie) (version 2.0.1)
**Language:** TypeScript (Node.js)
**Runtime:** npm (via `package-lock.json`)
**Reference:** `skills/gf-quality/references/node.md`

## Detection Accuracy

| Check | Expected | Actual | Correct? |
|-------|----------|--------|----------|
| Language | Node.js | Node.js (via `package.json`) | Yes |
| Runtime | npm | npm (via `package-lock.json`) | Yes |
| TypeScript | Yes | Yes (via `tsconfig.json`) | Yes |
| Marker file | `package.json` at root | `package.json` at root | Yes |

The runtime detection order (bun -> pnpm -> yarn -> npm) correctly selected npm based on `package-lock.json`. The TypeScript detection worked via `tsconfig.json` presence.

## Gate Execution Results

| Gate | Command | Result | Details |
|------|---------|--------|---------|
| 1 (build) | `npm run build` | PASS | `ts-scripts build` compiled TypeScript to `dist/` |
| 2 (test) | `npm test` | PASS | 4 test files, 63,740 tests (includes property-based/fuzz tests), all passed |
| 3 (coverage) | `npx vitest --coverage` | PASS | Statements 98.2%, Branches 96.82%, Functions 100%, Lines 100% |
| 4 (format) | `npm run format` | PASS | Prettier check -- all 10 source files already formatted |
| 5 (static/lint) | `npm run lint` | N/A | No `lint` script in `package.json`; no ESLint config found |
| 6 (pre-commit) | `pre-commit run --all-files` | N/A | No `.pre-commit-config.yaml` in repository |

### Coverage Breakdown

```
File               | % Stmts | % Branch | % Funcs | % Lines
-------------------|---------|----------|---------|---------
All files          |    98.2 |    96.82 |     100 |     100
 src/index.ts      |    98.2 |    96.82 |     100 |     100
```

The uncovered lines (295, 441, 446, 457) are edge cases in cookie parsing that would require malformed input to trigger.

## Issues Encountered

### 1. npm allow-scripts Warning

During `npm install`, the following warning appeared:

```
npm warn allow-scripts 2 packages have install scripts not yet covered by allowScripts:
npm warn allow-scripts   esbuild@0.27.7 (postinstall: node install.js)
npm warn allow-scripts   fsevents@2.3.3 (install: (install scripts present))
```

This is a Node.js 22+ security feature. The skill should note this as an FYI but not block the gate. The warning does not cause `npm install` to fail -- it is informational.

### 2. No Lint Script Configured

The project uses `@borderless/ts-scripts` which bundles TypeScript compilation, Prettier formatting, and Vitest testing, but does **not** include ESLint or any other static analysis linter. Gate 5 (static) correctly returned N/A.

This is an interesting real-world pattern: the project prioritizes type-checking (`tsc --noEmit`) and formatting over traditional linting, relying on TypeScript's strict mode to catch many issues that ESLint would flag.

### 3. No Pre-commit Config

The project has no `.pre-commit-config.yaml`. Gate 6 returned N/A. This is common for smaller npm packages where CI/CD handles quality checks instead of local pre-commit hooks.

### 4. Requires Node >= 22

The `engines` field specifies `"node": ">=22"`. The build and test would fail on older Node.js versions. The skill does not currently validate Node.js version requirements, but this could be a useful enhancement.

## Fixes Applied

No fixes were needed. All applicable gates passed on the first run.

## Final Result

**Overall: PASS**

All applicable quality gates passed. Two gates (static/lint and pre-commit) returned N/A because the project does not configure those tools.

| Gate | Status |
|------|--------|
| Build | PASS |
| Test | PASS (63,740 tests) |
| Coverage | PASS (98.2% statements) |
| Format | PASS |
| Static/Lint | N/A (no ESLint config) |
| Pre-commit | N/A (no config) |

## Lessons Learned

1. **`ts-scripts` is an opaque tool wrapper.** The project uses `ts-scripts build/test/format` rather than direct `tsc`, `vitest`, `prettier` calls. The skill's gate commands (which reference `npm run build`, `npm test`, `npm run format`) work correctly because they rely on the scripts in `package.json` rather than assuming specific tooling.

2. **Coverage via vitest is well-integrated.** The project has `@vitest/coverage-v8` as a devDependency, and vitest's `--coverage` flag generates coverage without a separate script. The skill's fallback logic (`npx vitest --coverage` or `npx jest --coverage`) handled this correctly.

3. **High test count from property-based testing.** 63,740 tests in 4 files indicates the use of property-based/fuzz testing (likely via vitest's built-in fuzz or a fast-check-like library). The quality gate correctly distinguishes test count from test coverage.

4. **No lint script is not necessarily bad.** The project's reliance on TypeScript strict mode + Prettier + thorough testing provides significant quality assurance even without ESLint. The skill's N/A result for Gate 5 is appropriate.

5. **Lock file detection for runtime selection works.** The detection of `package-lock.json` correctly identified npm as the runtime, and all `npm` commands worked as expected.
