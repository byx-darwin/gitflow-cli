# gitflow-quality Multi-Language Command Reference

> **Companion to:** `skills/gitflow-quality/SKILL.md`
> **Purpose:** Non-Rust project command matrix. The skill itself only embeds the Rust path; this file covers Node.js, Python, Go, Java.

---

## Language Detection

```bash
if [ -f "Cargo.toml" ]; then
    LANG="rust"
elif [ -f "package.json" ]; then
    LANG="node"
elif [ -f "pyproject.toml" ] || [ -f "setup.py" ]; then
    LANG="python"
elif [ -f "go.mod" ]; then
    LANG="go"
elif [ -f "pom.xml" ] || [ -f "build.gradle" ]; then
    LANG="java"
else
    LANG="unknown"
fi
```

## Command Matrix

| Check | Rust | Node.js | Python | Go | Java |
|-------|------|---------|--------|-----|------|
| build | `cargo build --workspace` | `npm run build` | `python -m py_compile src/` | `go build ./...` | `mvn compile -q` / `gradle compileJava` |
| test | `cargo test --workspace` | `npm test` | `pytest` | `go test ./...` | `mvn test` / `gradle test` |
| coverage | `cargo tarpaulin --workspace` | `npx jest --coverage` / `npx vitest run --coverage` | `pytest --cov` | `go test -coverprofile=coverage.out ./... && go tool cover -func=coverage.out \| grep total` | `mvn verify -Pcoverage` / `gradle jacocoTestReport` |
| format | `cargo +nightly fmt -- --check` | `npx prettier --check .` | `black --check .` | `test -z "$(gofmt -l .)"` | `mvn spotless:check` / `mvn formatter:format` |
| static | `cargo clippy --workspace --all-targets -- -D warnings` | `npx eslint .` | `ruff check .` | `golangci-lint run` | `mvn pmd:check` / `mvn spotbugs:check` |
| pre-commit | `pre-commit run --all-files` | `npx lint-staged` / skip | `pre-commit run --all-files` | `pre-commit run --all-files` | `pre-commit run --all-files` |

## Notes

- **Rust** is the primary path for this project. All other languages are fallback.
- **Coverage threshold** defaults to 80% (`COVERAGE_THRESHOLD` env var overrides).
- **Pre-commit N/A**: if `.pre-commit-config.yaml` is absent, mark `N/A` and continue.
