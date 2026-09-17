# Node.js / TypeScript Quality Toolchain

**Detection:** `package.json` in project root.

## Runtime Detection

Check for package manager lock files **in order**:

| Lock File | Runtime | Install Command |
|-----------|---------|----------------|
| `bun.lockb` / `bun.lock` | Bun | `bun install` |
| `pnpm-lock.yaml` | pnpm | `pnpm install` |
| `yarn.lock` | Yarn | `yarn install` |
| `package-lock.json` | npm | `npm install` |

First match wins. If no lock file, default to `npm`.

## Gate Commands

### Bun

| # | Gate | Command | Pass Criteria |
|---|------|---------|---------------|
| 1 | build | `bun run build` or `bunx tsc --noEmit` (TS) | exit 0 |
| 2 | test | `bun test` | all pass |
| 3 | coverage | `bun test --coverage` | exit 0 (threshold enforced via `bunfig.toml`'s `[test] coverageThreshold`, a fraction where `0.80` = the skill's default 80%); SKIPPED if `bunfig.toml` has no `[test] coverageThreshold` — `bun test --coverage` then exits 0 at any coverage and the skill must not create the file; N/A if no `.js`/`.jsx`/`.ts`/`.tsx` in change set |
| 4 | format | `bunx prettier --check .` | exit 0 |
| 5 | static | `bunx eslint .` or `bun run lint` | exit 0, no errors |
| 6 | pre-commit | `pre-commit run --all-files` or `bunx lint-staged` | all hooks pass (or N/A) |

### npm

| # | Gate | Command | Pass Criteria |
|---|------|---------|---------------|
| 1 | build | `npm run build` or `npx tsc --noEmit` (TS) | exit 0 |
| 2 | test | `npm test` | all pass |
| 3 | coverage | `npx jest --coverage --coverageThreshold="{\"global\":{\"lines\":${COV_THRESHOLD:-80}}}"` (preferred — it is the only form that provably enforces the threshold) | exit 0 (total line coverage ≥ threshold); N/A if no `.js`/`.jsx`/`.ts`/`.tsx` in change set |
| 4 | format | `npx prettier --check .` or `npm run format:check` | exit 0 |
| 5 | static | `npx eslint .` or `npm run lint` | exit 0, no errors |
| 6 | pre-commit | `pre-commit run --all-files` or `npx lint-staged` | all hooks pass (or N/A) |

A project's own `npm run test:coverage` may be used **only** after inspecting the
script in `package.json` and confirming it passes a threshold (`--coverageThreshold`,
`--cov-fail-under`, a `coverageThreshold` key in jest config, or
`coverage.thresholds` in vitest config). An arbitrary `test:coverage` script
guarantees nothing: if no threshold is enforced, report Gate 3 as **SKIPPED**
rather than PASS.

Note the quoting of the jest command: the JSON argument is in **double** quotes so
`${COV_THRESHOLD:-80}` expands. The original single-quoted form would have passed
the literal text `${COV_THRESHOLD:-80}` to jest.

### pnpm / Yarn

Replace `npm` → `pnpm` or `yarn`, `npx` → `pnpm exec` or `yarn exec` accordingly.

## Runtime Detection Command

```bash
for f in bun.lockb bun.lock pnpm-lock.yaml yarn.lock package-lock.json; do
  [ -f "$f" ] && echo "DETECTED: $f" && break
done
```

## Notes

- Gate 1: check scripts in lock file's package manager; for TypeScript, also run type check
- Gate 3: a `test:coverage` script counts only if it enforces a threshold; a bare `jest --coverage` or `vitest --coverage` enforces none — in that case mark SKIPPED
- Gate 3: jest takes the threshold from `--coverageThreshold`; vitest from `coverage.thresholds.lines` in `vitest.config.*`; Bun from `[test] coverageThreshold` in `bunfig.toml`. Absent threshold config → SKIPPED, never PASS
- Gate 3 is N/A when the change set contains no `.js`/`.jsx`/`.ts`/`.tsx` file — report N/A, not SKIPPED
- Gate 4: respect `.prettierrc` or config in `package.json`
- Gate 5: respect `.eslintrc*` or `eslintConfig` in `package.json`
- Gate 6: if no pre-commit config, check for `husky` + `lint-staged` setup

## Forbidden Actions

- ❌ Never run install without user confirmation
- ❌ Never modify `package.json` or lock files during quality check
- ❌ Never auto-fix lint issues with `eslint --fix` — report only
- ❌ Never mix runtimes (e.g., run `npm install` in a bun project)

## Configuration

### Tool Setup

| Tool | Install | Config File | Required |
|------|---------|-------------|----------|
| prettier | `npm install -D prettier` | `.prettierrc` | Gate 4 |
| eslint | `npm install -D eslint` | `.eslintrc.json` | Gate 5 |
| typescript | `npm install -D typescript` | `tsconfig.json` | Gate 1 (TS projects) |

### Config File Examples

#### .prettierrc

```json
{
  "semi": true,
  "trailingComma": "all",
  "printWidth": 100,
  "singleQuote": false
}
```

#### .eslintrc.json

```json
{
  "env": {
    "node": true,
    "es2022": true
  },
  "extends": "eslint:recommended",
  "parserOptions": {
    "ecmaVersion": 2022,
    "sourceType": "module"
  },
  "rules": {
    "no-console": "warn"
  }
}
```

#### tsconfig.json (TypeScript)

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ES2022",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true
  },
  "include": ["src/**/*"]
}
```

#### bunfig.toml (Bun coverage threshold)

```toml
[test]
coverageThreshold = 0.80
```

`coverageThreshold` is a fraction, not a percentage — `0.80` corresponds to the skill's default
`COV_THRESHOLD=80`. Bun's `bun test --coverage` has no CLI flag for the threshold; it only reads
`bunfig.toml`. `bun test` silently ignores unrecognized flags, so a threshold passed on the
command line has no effect and must not be relied upon. If `bunfig.toml` is missing or has no
`[test] coverageThreshold`, `bun test --coverage` exits 0 no matter how low coverage is — report
Gate 3 as SKIPPED and recommend adding the key. The skill never creates or edits `bunfig.toml`.

#### package.json (scripts section)

```json
{
  "scripts": {
    "test": "node --test",
    "test:coverage": "node --test --experimental-test-coverage",
    "lint": "eslint .",
    "format:check": "prettier --check .",
    "format:fix": "prettier --write ."
  }
}
```

### Environment Variables

| Variable | Effect | Default |
|----------|--------|---------|
| `COV_THRESHOLD` / `COVERAGE_THRESHOLD` | Override coverage threshold — interpolated into jest's `--coverageThreshold`; for Bun and vitest it must match the value in `bunfig.toml` / `vitest.config.*`, which the skill reads rather than writes | 80% |
| `NODE_ENV` | Node environment | `development` |
| `npm_config_*` | npm configuration | — |

### Language-Specific Notes

- Detect runtime from lock file: bun → pnpm → yarn → npm
- Gate 1: for TypeScript, also run `tsc --noEmit` for type checking
- Gate 3: a `test:coverage` script counts only if it enforces a threshold; a bare `jest --coverage` or `vitest --coverage` enforces none — in that case mark SKIPPED
- Gate 3: jest takes the threshold from `--coverageThreshold`; vitest from `coverage.thresholds.lines` in `vitest.config.*`; Bun from `[test] coverageThreshold` in `bunfig.toml`. Absent threshold config → SKIPPED, never PASS
- Gate 3 is N/A when the change set contains no `.js`/`.jsx`/`.ts`/`.tsx` file — report N/A, not SKIPPED
- Gate 4: respect `.prettierrc` or config in `package.json`
- Gate 5: respect `.eslintrc*` or `eslintConfig` in `package.json`

## Troubleshooting

### Common Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `npm ERR! permission denied` | Permission issue | Use `sudo` or fix npm directory permissions |
| `ERESOLVE could not resolve dependency` | Lock file conflict | Delete `node_modules` and `package-lock.json`, run `npm install` |
| `TS2304: Cannot find name` | TypeScript error | Check imports and type definitions |
| `Cannot find module` | Import error | Check path and ensure module is installed |

### Exit Code Reference

| Code | Meaning | Action |
|------|---------|--------|
| 0 | Success | Continue to next gate |
| 1 | Test/lint failure | Fix errors and retry |
| 2 | Lint errors | Fix lint warnings |
| 127 | Command not found | Install missing tool |

### FAQ

**Q: npm vs yarn vs pnpm?**
A: All work. pnpm is fastest and most disk-efficient. yarn is mature. npm is default.

**Q: How to clear node_modules cache?**
A: Delete `node_modules` and lock file, then run `npm install` (or `yarn install`, `pnpm install`).

**Q: TypeScript strict mode?**
A: Enable in `tsconfig.json`: `"strict": true`. Fix all type errors before proceeding.

### Performance Tips

- Use `npm ci` instead of `npm install` in CI for faster, deterministic installs
- Use parallel test runners: `jest --parallel` or `vitest --pool=forks`
- Skip dev dependencies in CI: `npm install --production`
- Use `--ignore-scripts` to skip postinstall scripts if not needed
