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
| 3 | coverage | `bun test --coverage` | incremental ≥ 80% |
| 4 | format | `bunx prettier --check .` | exit 0 |
| 5 | static | `bunx eslint .` or `bun run lint` | exit 0, no errors |
| 6 | pre-commit | `pre-commit run --all-files` or `bunx lint-staged` | all hooks pass (or N/A) |

### npm

| # | Gate | Command | Pass Criteria |
|---|------|---------|---------------|
| 1 | build | `npm run build` or `npx tsc --noEmit` (TS) | exit 0 |
| 2 | test | `npm test` | all pass |
| 3 | coverage | `npm run test:coverage` or `npx jest --coverage` | incremental ≥ 80% |
| 4 | format | `npx prettier --check .` or `npm run format:check` | exit 0 |
| 5 | static | `npx eslint .` or `npm run lint` | exit 0, no errors |
| 6 | pre-commit | `pre-commit run --all-files` or `npx lint-staged` | all hooks pass (or N/A) |

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
- Gate 3: look for `test:coverage` or `coverage` script; fallback to `jest --coverage` / `vitest --coverage`
- Gate 4: respect `.prettierrc` or config in `package.json`
- Gate 5: respect `.eslintrc*` or `eslintConfig` in `package.json`
- Gate 6: if no pre-commit config, check for `husky` + `lint-staged` setup

## Forbidden Actions

- ❌ Never run install without user confirmation
- ❌ Never modify `package.json` or lock files during quality check
- ❌ Never auto-fix lint issues without showing diff first
- ❌ Never mix runtimes (e.g., run `npm install` in a bun project)
