# Node.js / TypeScript Pre-commit Checks

**Shared language profile:** `gf-quality/references/profiles/node.md`. Read it for tools, version sources, and scan exclusions.


## Check Commands

### Bun

| Check | Command | Fix Command |
|-------|---------|-------------|
| format | `bunx prettier --check .` | `bunx prettier --write .` |
| lint | `bunx eslint .` or `bun run lint` | `bunx eslint --fix .` |
| test | `bun test` | — |

### npm

| Check | Command | Fix Command |
|-------|---------|-------------|
| format | `npx prettier --check .` | `npx prettier --write .` |
| lint | `npx eslint .` or `npm run lint` | `npx eslint --fix .` |
| test | `npm test` | — |

### pnpm / Yarn

Replace `npm` → `pnpm` / `yarn`, `npx` → `pnpm exec` / `yarn exec`.

## Notes

- Check `package.json` scripts first — use project-defined scripts when available
- Fix commands require user confirmation before execution
- Never run install without user confirmation
- Never mix runtimes (detect lock file first, stick with it)
