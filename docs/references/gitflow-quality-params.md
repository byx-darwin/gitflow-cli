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

When `.claude/gh-issue/current-issue.txt` exists and `gitflow-cli` is on `PATH`:

1. Render report to temp file `quality-report.md`.
2. Ask user to confirm publish.
3. On yes: `gitflow-cli issue comment "${ISSUE_NUMBER}" --body-file quality-report.md`, then `rm -f quality-report.md`.

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
