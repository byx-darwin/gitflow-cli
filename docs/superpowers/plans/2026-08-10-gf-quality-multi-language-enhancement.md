# gf-quality Multi-Language Enhancement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enhance gf-quality skill with comprehensive multi-language support including configuration guides, troubleshooting documentation, workspace detection, and example projects.

**Architecture:** Incremental enhancement of existing reference files. Each language reference gains Configuration and Troubleshooting sections. Detector enhanced with 3-level scan and workspace detection. Minimal example projects created for validation. Main SKILL.md updated with workspace-aware execution flow.

**Tech Stack:** Markdown, Shell scripting, Go, Node.js, Python

## Global Constraints

- All content in English except bilingual description field
- Configuration guides inline in `references/<lang>.md` (no separate files)
- Troubleshooting inline in `references/<lang>.md` (no separate files)
- Example projects minimal: one function, one test, all config files
- Workspace detection scans 3 levels deep (not 2, not 4)
- SKILL.md remains < 500 words (detailed content in references/)
- No auto-fix capabilities (report-only policy maintained)

---

### Task 1: Enhance Rust Reference (Configuration + Troubleshooting)

**Files:**
- Modify: `skills/gf-quality/references/rust.md`

**Interfaces:**
- Consumes: Existing Rust reference structure
- Produces: Enhanced reference with Configuration + Troubleshooting sections

- [ ] **Step 1: Add Configuration section to rust.md**

Append the following to `skills/gf-quality/references/rust.md`:

```markdown
## Configuration

### Tool Setup

| Tool | Install | Config File | Required |
|------|---------|-------------|----------|
| cargo-tarpaulin | `cargo install cargo-tarpaulin` | — | Gate 3 (coverage) |
| nightly toolchain | `rustup toolchain install nightly` | — | Gate 4 (format) |
| rustfmt | Included with rustup | `rustfmt.toml` | Gate 4 |
| clippy | Included with rustup | `clippy.toml` | Gate 5 |

### Config File Examples

#### rustfmt.toml

```toml
edition = "2021"
max_width = 100
imports_layout = "Mixed"
```

#### clippy.toml

```toml
cognitive-complexity-threshold = 30
too-many-arguments-threshold = 7
```

#### Cargo.toml (workspace)

```toml
[workspace]
members = ["crates/*", "apps/*"]
resolver = "2"

[workspace.lints.clippy]
pedantic = { level = "warn", priority = -1 }
```

### Environment Variables

| Variable | Effect | Default |
|----------|--------|---------|
| `COV_THRESHOLD` / `COVERAGE_THRESHOLD` | Override coverage threshold | 80% |
| `RUSTFLAGS` | Pass flags to rustc | — |
| `CARGO_HOME` | Cargo cache location | `~/.cargo` |

### Language-Specific Notes

- For Rust workspaces, run gates at workspace root (covers all members)
- Gate 3 requires `cargo-tarpaulin` — if missing, mark SKIPPED
- Gate 4 requires nightly toolchain — if missing, mark SKIPPED
- Gate 5 uses `-D warnings` — any warning fails the gate
```

- [ ] **Step 2: Add Troubleshooting section to rust.md**

Append the following to `skills/gf-quality/references/rust.md`:

```markdown
## Troubleshooting

### Common Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `cargo-tarpaulin: command not found` | Tool not installed | `cargo install cargo-tarpaulin` |
| `error: toolchain 'nightly' is not installed` | Nightly missing | `rustup toolchain install nightly` |
| `error: could not compile` | Compilation error | Read error message, fix code |
| `test failed, doctests failed` | Test failure | Run `cargo test --workspace -- --nocapture` |

### Exit Code Reference

| Code | Meaning | Action |
|------|---------|--------|
| 0 | Success | Continue to next gate |
| 101 | Compilation error | Fix compilation errors |
| 102 | Test failure | Fix failing tests |
| 1 | Clippy warnings (with -D) | Fix lint warnings |

### FAQ

**Q: Why does coverage show 0%?**
A: Ensure `cargo-tarpaulin` is installed and project builds successfully. Check for `#[cfg(test)]` modules.

**Q: How to skip doc tests?**
A: Run `cargo test --lib --bins` instead of `cargo test --workspace`.

**Q: Workspace build slow?**
A: Use `cargo build --workspace --quiet` to reduce output. Enable incremental compilation in `Cargo.toml`.

### Performance Tips

- Use `cargo build --workspace --quiet` to reduce output noise
- Enable parallel test execution: `cargo test --workspace -- --test-threads=4`
- Use incremental compilation: add `profile.dev.incremental = true` to `Cargo.toml`
- Skip doc tests if not needed: `cargo test --lib --bins`
```

- [ ] **Step 3: Verify syntax**

Proofread the added sections. Ensure all code blocks are properly formatted and all tables render correctly.

- [ ] **Step 4: Commit**

```bash
git add skills/gf-quality/references/rust.md
git commit -m "docs(gf-quality): add Configuration and Troubleshooting to Rust reference

- Add tool setup table with install commands
- Add config file examples (rustfmt.toml, clippy.toml, workspace)
- Add environment variables reference
- Add common errors and exit codes
- Add FAQ and performance tips"
```

---

### Task 2: Enhance Go Reference (Configuration + Troubleshooting)

**Files:**
- Modify: `skills/gf-quality/references/go.md`

**Interfaces:**
- Consumes: Existing Go reference structure
- Produces: Enhanced reference with Configuration + Troubleshooting sections

- [ ] **Step 1: Add Configuration section to go.md**

Append the following to `skills/gf-quality/references/go.md`:

```markdown
## Configuration

### Tool Setup

| Tool | Install | Config File | Required |
|------|---------|-------------|----------|
| golangci-lint | `go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest` | `.golangci.yml` | Gate 5 |
| gofmt | Included with Go | — | Gate 4 |
| go vet | Included with Go | — | Gate 5 |

### Config File Examples

#### .golangci.yml

```yaml
linters:
  enable:
    - gofmt
    - govet
    - staticcheck
    - errcheck
  disable:
    - gocyclo
  fast: true

linters-settings:
  gofmt:
    simplify: true
```

#### go.mod

```go
module github.com/example/project

go 1.21

require (
    github.com/stretchr/testify v1.8.4
)
```

### Environment Variables

| Variable | Effect | Default |
|----------|--------|---------|
| `GOPROXY` | Go module proxy | `https://proxy.golang.org` |
| `GONOSUMCHECK` | Skip checksum verification | — |
| `GOFLAGS` | Default go command flags | — |

### Language-Specific Notes

- Gate 2 includes `-race` for race condition detection
- Gate 3: compare against previous run; incremental coverage ≥ 80%
- Gate 4: auto-fix with `gofmt -w .` only after user confirmation
- Gate 5: `staticcheck ./...` as fallback if golangci-lint unavailable
```

- [ ] **Step 2: Add Troubleshooting section to go.md**

Append the following to `skills/gf-quality/references/go.md`:

```markdown
## Troubleshooting

### Common Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `golangci-lint: command not found` | Tool not installed | `go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest` |
| `go: downloading: module not found` | Module proxy issue | Check `GOPROXY` or use `go mod vendor` |
| `FAIL: TestX (0.00s)` | Test failure | Run `go test -v ./...` for details |
| `race detected` | Race condition | Fix concurrent access patterns |

### Exit Code Reference

| Code | Meaning | Action |
|------|---------|--------|
| 0 | Success | Continue to next gate |
| 1 | Build/test failure | Fix errors and retry |
| 2 | Vet errors | Fix vet warnings |

### FAQ

**Q: Why does `go test` hang?**
A: Check for deadlocks or infinite loops. Use `go test -timeout 30s` to set a timeout.

**Q: How to update dependencies?**
A: Run `go get -u ./...` then `go mod tidy`.

**Q: Module proxy issues?**
A: Set `GOPROXY=https://goproxy.cn` (China) or use `go mod vendor` for offline builds.

### Performance Tips

- Use `go test -parallel 4` for parallel test execution
- Enable build caching: Go caches builds automatically
- Use `go mod vendor` for offline builds and faster CI
- Run `go clean -testcache` to clear test cache if needed
```

- [ ] **Step 3: Verify syntax**

Proofread the added sections. Ensure all code blocks are properly formatted and all tables render correctly.

- [ ] **Step 4: Commit**

```bash
git add skills/gf-quality/references/go.md
git commit -m "docs(gf-quality): add Configuration and Troubleshooting to Go reference

- Add tool setup table with install commands
- Add config file examples (.golangci.yml, go.mod)
- Add environment variables reference
- Add common errors and exit codes
- Add FAQ and performance tips"
```

---

### Task 3: Enhance Node.js Reference (Configuration + Troubleshooting)

**Files:**
- Modify: `skills/gf-quality/references/node.md`

**Interfaces:**
- Consumes: Existing Node.js reference structure
- Produces: Enhanced reference with Configuration + Troubleshooting sections

- [ ] **Step 1: Add Configuration section to node.md**

Append the following to `skills/gf-quality/references/node.md`:

```markdown
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
| `NODE_ENV` | Node environment | `development` |
| `npm_config_*` | npm configuration | — |

### Language-Specific Notes

- Detect runtime from lock file: bun → pnpm → yarn → npm
- Gate 1: for TypeScript, also run `tsc --noEmit` for type checking
- Gate 3: look for `test:coverage` script, fallback to `jest --coverage` or `vitest --coverage`
- Gate 4: respect `.prettierrc` or config in `package.json`
- Gate 5: respect `.eslintrc*` or `eslintConfig` in `package.json`
```

- [ ] **Step 2: Add Troubleshooting section to node.md**

Append the following to `skills/gf-quality/references/node.md`:

```markdown
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
```

- [ ] **Step 3: Verify syntax**

Proofread the added sections. Ensure all code blocks are properly formatted and all tables render correctly.

- [ ] **Step 4: Commit**

```bash
git add skills/gf-quality/references/node.md
git commit -m "docs(gf-quality): add Configuration and Troubleshooting to Node.js reference

- Add tool setup table with install commands
- Add config file examples (.prettierrc, .eslintrc.json, tsconfig.json)
- Add environment variables reference
- Add common errors and exit codes
- Add FAQ and performance tips"
```

---

### Task 4: Enhance Python Reference (Configuration + Troubleshooting)

**Files:**
- Modify: `skills/gf-quality/references/python.md`

**Interfaces:**
- Consumes: Existing Python reference structure
- Produces: Enhanced reference with Configuration + Troubleshooting sections

- [ ] **Step 1: Add Configuration section to python.md**

Append the following to `skills/gf-quality/references/python.md`:

```markdown
## Configuration

### Tool Setup

| Tool | Install | Config File | Required |
|------|---------|-------------|----------|
| ruff | `pip install ruff` | `.ruff.toml` or `pyproject.toml` | Gate 4, 5 |
| black | `pip install black` | `pyproject.toml` | Gate 4 (fallback) |
| pylint | `pip install pylint` | `.pylintrc` | Gate 5 (fallback) |
| pytest-cov | `pip install pytest-cov` | `pyproject.toml` | Gate 3 |

### Config File Examples

#### pyproject.toml

```toml
[project]
name = "example"
version = "0.1.0"
requires-python = ">=3.10"

[tool.ruff]
line-length = 100
select = ["E", "F", "I"]

[tool.black]
line-length = 100
target-version = ["py310"]

[tool.pytest.ini_options]
testpaths = ["tests"]
python_files = ["test_*.py"]
```

#### .ruff.toml

```toml
line-length = 100
target-version = "py310"

[lint]
select = ["E", "F", "I", "N", "W"]
ignore = ["E501"]
```

### Environment Variables

| Variable | Effect | Default |
|----------|--------|---------|
| `PYTHONPATH` | Python module search path | — |
| `PYTHONDONTWRITEBYTECODE` | Skip .pyc files | — |
| `VIRTUAL_ENV` | Active virtual environment path | — |

### Language-Specific Notes

- Prefer `ruff` (fast, covers format + lint). Fall back to `black` + `pylint` if ruff not configured
- Gate 1: for compiled Python checks; skip for pure script projects (mark N/A)
- Gate 4: auto-fix with `ruff format .` or `black .` only after user confirmation
- Gate 5: check for TODO/FIXME/HACK residuals with `grep -rn "TODO\|FIXME\|HACK" --include="*.py" .`
- Always use virtual environments — never install into system Python
```

- [ ] **Step 2: Add Troubleshooting section to python.md**

Append the following to `skills/gf-quality/references/python.md`:

```markdown
## Troubleshooting

### Common Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `pip: command not found` | pip not installed | `python -m ensurepip --upgrade` |
| `Permission denied` | System Python | Use virtual environment: `python -m venv .venv` |
| `ModuleNotFoundError` | Import error | Activate venv: `source .venv/bin/activate`, then `pip install -e .` |
| `ImportError: cannot import name` | Circular import | Restructure imports |

### Exit Code Reference

| Code | Meaning | Action |
|------|---------|--------|
| 0 | Success | Continue to next gate |
| 1 | Test failure | Fix failing tests |
| 2 | Syntax error | Fix syntax errors |
| 127 | Command not found | Install missing tool |

### FAQ

**Q: ruff vs black vs pylint?**
A: ruff is fastest (covers format + lint). black is format-only. pylint is comprehensive but slow.

**Q: How to manage multiple Python versions?**
A: Use `pyenv` to manage versions. Set per-project with `pyenv local 3.10`.

**Q: pytest fixtures?**
A: Define in `conftest.py`. Use `@pytest.fixture` decorator. Share across tests.

### Performance Tips

- Use `pytest-xdist` for parallel test execution: `pytest -n auto`
- Use `--cov-report=term-missing` for faster coverage reports
- Use `pytest --cache-clear` to clear cache if tests behave unexpectedly
- Use `pip install -e .` for editable installs during development
```

- [ ] **Step 3: Verify syntax**

Proofread the added sections. Ensure all code blocks are properly formatted and all tables render correctly.

- [ ] **Step 4: Commit**

```bash
git add skills/gf-quality/references/python.md
git commit -m "docs(gf-quality): add Configuration and Troubleshooting to Python reference

- Add tool setup table with install commands
- Add config file examples (pyproject.toml, .ruff.toml)
- Add environment variables reference
- Add common errors and exit codes
- Add FAQ and performance tips"
```

---

### Task 5: Enhance Java Reference (Configuration + Troubleshooting)

**Files:**
- Modify: `skills/gf-quality/references/java.md`

**Interfaces:**
- Consumes: Existing Java reference structure
- Produces: Enhanced reference with Configuration + Troubleshooting sections

- [ ] **Step 1: Add Configuration section to java.md**

Append the following to `skills/gf-quality/references/java.md`:

```markdown
## Configuration

### Tool Setup

| Tool | Install | Config File | Required |
|------|---------|-------------|----------|
| JaCoCo | Maven/Gradle plugin | `pom.xml` or `build.gradle` | Gate 3 |
| Spotless | Maven/Gradle plugin | `pom.xml` or `build.gradle` | Gate 4 |
| PMD | Maven/Gradle plugin | `pmd-ruleset.xml` | Gate 5 |
| SpotBugs | Maven/Gradle plugin | `spotbugs-exclude.xml` | Gate 5 (fallback) |

### Config File Examples

#### pom.xml (Maven)

```xml
<project>
  <build>
    <plugins>
      <plugin>
        <groupId>org.jacoco</groupId>
        <artifactId>jacoco-maven-plugin</artifactId>
        <version>0.8.11</version>
        <executions>
          <execution>
            <goals>
              <goal>prepare-agent</goal>
            </goals>
          </execution>
          <execution>
            <id>report</id>
            <phase>test</phase>
            <goals>
              <goal>report</goal>
            </goals>
          </execution>
        </executions>
      </plugin>
    </plugins>
  </build>
</project>
```

#### build.gradle (Gradle)

```groovy
plugins {
    id 'jacoco'
    id 'com.diffplug.spotless' version '6.25.0'
}

jacoco {
    toolVersion = "0.8.11"
}

spotless {
    java {
        googleJavaFormat()
    }
}
```

#### spotbugs-exclude.xml

```xml
<FindBugsFilter>
  <Match>
    <Class name="~.*\.*Test"/>
  </Match>
</FindBugsFilter>
```

### Environment Variables

| Variable | Effect | Default |
|----------|--------|---------|
| `MAVEN_OPTS` | Maven JVM options | — |
| `GRADLE_OPTS` | Gradle JVM options | — |
| `JAVA_HOME` | JDK location | — |

### Language-Specific Notes

- Most tools are Maven/Gradle plugins — no separate install needed
- Gate 3: requires JaCoCo plugin configured; if not present, mark N/A
- Gate 4: Spotless is preferred; fall back to formatter-maven-plugin
- Gate 5: try PMD first, then SpotBugs, then Checkstyle — use whatever is configured
- Check for existing config files (`spotbugs-exclude.xml`, `pmd-ruleset.xml`, etc.)
- Respect `maven.test.skip` property — if set, warn user that tests are being skipped
```

- [ ] **Step 2: Add Troubleshooting section to java.md**

Append the following to `skills/gf-quality/references/java.md`:

```markdown
## Troubleshooting

### Common Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `Plugin not found` | Plugin not configured | Add plugin to `pom.xml` or `build.gradle` |
| `./gradlew: Permission denied` | Wrapper not executable | `chmod +x gradlew` |
| `java.lang.OutOfMemoryError` | JVM memory issue | Increase heap: `MAVEN_OPTS="-Xmx2g"` |
| `BUILD FAILURE` | Build error | Read error message, fix code or config |

### Exit Code Reference

| Code | Meaning | Action |
|------|---------|--------|
| 0 | Success | Continue to next gate |
| 1 | Build failure | Fix errors and retry |
| 2 | Test failure | Fix failing tests |
| 137 | OOM killed | Increase JVM heap size |

### FAQ

**Q: Maven vs Gradle?**
A: Maven is XML-based, convention over configuration. Gradle is Groovy/Kotlin-based, more flexible.

**Q: How to skip tests temporarily?**
A: Maven: `mvn install -DskipTests`. Gradle: `./gradlew build -x test`. Warning: report will show tests SKIPPED.

**Q: JaCoCo coverage report location?**
A: Maven: `target/site/jacoco/index.html`. Gradle: `build/reports/jacoco/test/html/index.html`.

### Performance Tips

- Maven parallel builds: `mvn -T 1C` (1 thread per CPU core)
- Gradle daemon: enabled by default, speeds up builds
- Enable incremental compilation in Gradle: `org.gradle.caching=true`
- Use `mvn clean install` only when necessary; prefer `mvn install` for incremental builds
```

- [ ] **Step 3: Verify syntax**

Proofread the added sections. Ensure all code blocks are properly formatted and all tables render correctly.

- [ ] **Step 4: Commit**

```bash
git add skills/gf-quality/references/java.md
git commit -m "docs(gf-quality): add Configuration and Troubleshooting to Java reference

- Add tool setup table with install commands
- Add config file examples (pom.xml, build.gradle, spotbugs-exclude.xml)
- Add environment variables reference
- Add common errors and exit codes
- Add FAQ and performance tips"
```

---

### Task 6: Enhance Detector (Workspace Detection + Project Tree)

**Files:**
- Modify: `skills/gf-quality/references/detector.md`

**Interfaces:**
- Consumes: Existing detector structure
- Produces: Enhanced detector with 3-level scan, workspace detection, project tree output

- [ ] **Step 1: Update scan depth in detector.md**

Replace the detection command in `skills/gf-quality/references/detector.md` with:

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

- [ ] **Step 2: Add workspace detection section to detector.md**

Append the following to `skills/gf-quality/references/detector.md`:

```markdown
## Workspace Detection

After detecting marker files, check for workspace configurations:

### Workspace Marker Files

| Marker | Workspace Type | Execution Strategy |
|--------|----------------|-------------------|
| `go.work` | Go workspace | Run `go build/test` at root (covers all modules) |
| `Cargo.toml` with `[workspace]` | Rust workspace | Run `cargo build/test` at root (covers all crates) |
| `settings.gradle` / `settings.gradle.kts` | Gradle multi-project | Run `./gradlew build` at root (covers all subprojects) |
| `package.json` with `"workspaces"` | npm/yarn/pnpm workspace | Run gates in each workspace package independently |

### Workspace Detection Commands

```bash
# Detect Go workspace
[ -f "go.work" ] && echo "WORKSPACE: Go workspace detected (go.work)"

# Detect Rust workspace
grep -q "^\[workspace\]" Cargo.toml 2>/dev/null && echo "WORKSPACE: Rust workspace detected"

# Detect Gradle multi-project
[ -f "settings.gradle" ] || [ -f "settings.gradle.kts" ] && echo "WORKSPACE: Gradle multi-project detected"

# Detect npm/yarn workspace
grep -q "\"workspaces\"" package.json 2>/dev/null && echo "WORKSPACE: npm/yarn workspace detected"
```

### Workspace-Aware Execution

- **Rust workspace:** Single `cargo build/test/clippy` at root covers all members
- **Go workspace:** Single `go build/test` at root covers all modules
- **Gradle multi-project:** Single `./gradlew build` at root covers all subprojects
- **npm/yarn workspace:** Run gates in each workspace package independently (one failure does NOT block others)
```

- [ ] **Step 3: Add project tree output section to detector.md**

Append the following to `skills/gf-quality/references/detector.md`:

```markdown
## Project Tree Output

Before running gates, output a visual tree showing detected languages:

```
Project Structure:
.
├── Cargo.toml (workspace root)
├── crates/
│   ├── core/Cargo.toml
│   ├── cli/Cargo.toml
│   └── github/Cargo.toml
├── apps/
│   └── desktop/
│       └── package.json (Node.js, bun)
└── services/
    └── api/
        └── go.mod (Go)

Detected:
  1. Rust (workspace) → ./ (3 crates)
  2. Node.js (bun) → ./apps/desktop/
  3. Go → ./services/api/

Which to check? [1/2/3/all]
```

### Tree Generation

Use `tree` command or manual construction:

```bash
# If tree is installed
tree -L 3 -I 'node_modules|target|vendor|.git' --prune

# Otherwise, use find + manual formatting
find . -maxdepth 3 -type f \( -name "Cargo.toml" -o -name "package.json" -o -name "go.mod" \) \
  -not -path "*/node_modules/*" -not -path "*/target/*" | sort
```
```

- [ ] **Step 4: Verify syntax**

Proofread the added sections. Ensure all code blocks are properly formatted and all tables render correctly.

- [ ] **Step 5: Commit**

```bash
git add skills/gf-quality/references/detector.md
git commit -m "docs(gf-quality): enhance detector with workspace detection and project tree

- Increase scan depth from 2 to 3 levels
- Add workspace marker detection (go.work, Cargo.toml [workspace], settings.gradle, package.json workspaces)
- Add workspace-aware execution strategies
- Add project tree output for transparency
- Add tree generation commands"
```

---

### Task 7: Create Go Example Project

**Files:**
- Create: `examples/quality-gate/go/go.mod`
- Create: `examples/quality-gate/go/main.go`
- Create: `examples/quality-gate/go/main_test.go`
- Create: `examples/quality-gate/go/README.md`

**Interfaces:**
- Consumes: Go reference structure
- Produces: Minimal Go project for validation

- [ ] **Step 1: Create go.mod**

Create `examples/quality-gate/go/go.mod`:

```go
module github.com/byx-darwin/gitflow-cli/examples/quality-gate/go

go 1.21
```

- [ ] **Step 2: Create main.go**

Create `examples/quality-gate/go/main.go`:

```go
package main

import "fmt"

// Add returns the sum of two integers.
func Add(a, b int) int {
	return a + b
}

func main() {
	fmt.Println("2 + 3 =", Add(2, 3))
}
```

- [ ] **Step 3: Create main_test.go**

Create `examples/quality-gate/go/main_test.go`:

```go
package main

import "testing"

func TestAdd(t *testing.T) {
	if got := Add(2, 3); got != 5 {
		t.Errorf("Add(2, 3) = %d; want 5", got)
	}
}
```

- [ ] **Step 4: Create README.md**

Create `examples/quality-gate/go/README.md`:

```markdown
# Go Quality Gate Example

Minimal Go project for validating `gf-quality` gates.

## Validate

```bash
cd examples/quality-gate/go
gf quality
```

Expected: ALL CHECKS PASSED
```

- [ ] **Step 5: Verify project**

Run the following commands to verify the project is valid:

```bash
cd examples/quality-gate/go
go build ./...
go test ./... -race -count=1
gofmt -l .
```

All commands should succeed with no output (for gofmt).

- [ ] **Step 6: Commit**

```bash
git add examples/quality-gate/go/
git commit -m "examples(gf-quality): add minimal Go project for validation

- go.mod with module definition
- main.go with Add function
- main_test.go with TestAdd
- README with validation instructions"
```

---

### Task 8: Create Node.js Example Project

**Files:**
- Create: `examples/quality-gate/node/package.json`
- Create: `examples/quality-gate/node/index.js`
- Create: `examples/quality-gate/node/index.test.js`
- Create: `examples/quality-gate/node/.prettierrc`
- Create: `examples/quality-gate/node/.eslintrc.json`
- Create: `examples/quality-gate/node/README.md`

**Interfaces:**
- Consumes: Node.js reference structure
- Produces: Minimal Node project for validation

- [ ] **Step 1: Create package.json**

Create `examples/quality-gate/node/package.json`:

```json
{
  "name": "gf-quality-node-example",
  "version": "1.0.0",
  "type": "module",
  "scripts": {
    "test": "node --test",
    "test:coverage": "node --test --experimental-test-coverage",
    "lint": "eslint .",
    "format:check": "prettier --check .",
    "format:fix": "prettier --write ."
  },
  "devDependencies": {
    "eslint": "^8.0.0",
    "prettier": "^3.0.0"
  }
}
```

- [ ] **Step 2: Create index.js**

Create `examples/quality-gate/node/index.js`:

```javascript
/**
 * Adds two numbers.
 * @param {number} a
 * @param {number} b
 * @returns {number}
 */
export function add(a, b) {
  return a + b;
}

console.log("2 + 3 =", add(2, 3));
```

- [ ] **Step 3: Create index.test.js**

Create `examples/quality-gate/node/index.test.js`:

```javascript
import { test } from "node:test";
import assert from "node:assert";
import { add } from "./index.js";

test("add(2, 3) returns 5", () => {
  assert.strictEqual(add(2, 3), 5);
});
```

- [ ] **Step 4: Create .prettierrc**

Create `examples/quality-gate/node/.prettierrc`:

```json
{
  "semi": true,
  "trailingComma": "all",
  "printWidth": 100
}
```

- [ ] **Step 5: Create .eslintrc.json**

Create `examples/quality-gate/node/.eslintrc.json`:

```json
{
  "env": { "node": true, "es2022": true },
  "extends": "eslint:recommended",
  "parserOptions": { "ecmaVersion": 2022, "sourceType": "module" }
}
```

- [ ] **Step 6: Create README.md**

Create `examples/quality-gate/node/README.md`:

```markdown
# Node.js Quality Gate Example

Minimal Node.js project for validating `gf-quality` gates.

## Setup

```bash
cd examples/quality-gate/node
npm install
```

## Validate

```bash
gf quality
```

Expected: ALL CHECKS PASSED
```

- [ ] **Step 7: Verify project**

Run the following commands to verify the project is valid:

```bash
cd examples/quality-gate/node
npm install
npm test
npm run format:check
npm run lint
```

All commands should succeed.

- [ ] **Step 8: Commit**

```bash
git add examples/quality-gate/node/
git commit -m "examples(gf-quality): add minimal Node.js project for validation

- package.json with test/lint/format scripts
- index.js with add function
- index.test.js with test
- .prettierrc and .eslintrc.json configs
- README with setup and validation instructions"
```

---

### Task 9: Create Python Example Project

**Files:**
- Create: `examples/quality-gate/python/pyproject.toml`
- Create: `examples/quality-gate/python/src/example/__init__.py`
- Create: `examples/quality-gate/python/src/example/main.py`
- Create: `examples/quality-gate/python/tests/test_main.py`
- Create: `examples/quality-gate/python/README.md`

**Interfaces:**
- Consumes: Python reference structure
- Produces: Minimal Python project for validation

- [ ] **Step 1: Create pyproject.toml**

Create `examples/quality-gate/python/pyproject.toml`:

```toml
[project]
name = "gf-quality-python-example"
version = "0.1.0"
requires-python = ">=3.10"

[tool.ruff]
line-length = 100
select = ["E", "F", "I"]

[tool.pytest.ini_options]
testpaths = ["tests"]
```

- [ ] **Step 2: Create __init__.py**

Create `examples/quality-gate/python/src/example/__init__.py`:

```python
"""Example Python package for gf-quality validation."""
```

- [ ] **Step 3: Create main.py**

Create `examples/quality-gate/python/src/example/main.py`:

```python
"""Main module with a simple function."""


def add(a: int, b: int) -> int:
    """Return the sum of two integers."""
    return a + b


if __name__ == "__main__":
    print(f"2 + 3 = {add(2, 3)}")
```

- [ ] **Step 4: Create test_main.py**

Create `examples/quality-gate/python/tests/test_main.py`:

```python
"""Tests for the main module."""

from example.main import add


def test_add():
    """Test that add returns the correct sum."""
    assert add(2, 3) == 5
```

- [ ] **Step 5: Create README.md**

Create `examples/quality-gate/python/README.md`:

```markdown
# Python Quality Gate Example

Minimal Python project for validating `gf-quality` gates.

## Setup

```bash
cd examples/quality-gate/python
python -m venv .venv
source .venv/bin/activate  # Linux/macOS
pip install -e ".[dev]"
```

## Validate

```bash
gf quality
```

Expected: ALL CHECKS PASSED
```

- [ ] **Step 6: Verify project**

Run the following commands to verify the project is valid:

```bash
cd examples/quality-gate/python
python -m venv .venv
source .venv/bin/activate
pip install -e .
pip install pytest ruff
python -m pytest
ruff check .
ruff format --check .
```

All commands should succeed.

- [ ] **Step 7: Commit**

```bash
git add examples/quality-gate/python/
git commit -m "examples(gf-quality): add minimal Python project for validation

- pyproject.toml with project config
- src/example/main.py with add function
- tests/test_main.py with test
- README with setup and validation instructions"
```

---

### Task 10: Enhance SKILL.md (Workspace-Aware Execution)

**Files:**
- Modify: `skills/gf-quality/SKILL.md`

**Interfaces:**
- Consumes: Existing SKILL.md structure
- Produces: Enhanced SKILL.md with workspace-aware execution flow

- [ ] **Step 1: Update Step 1 in SKILL.md**

Replace the "Step 1: Language Detection" section in `skills/gf-quality/SKILL.md` with:

```markdown
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
```

- [ ] **Step 2: Update Step 3 in SKILL.md**

Replace the "Step 3: Quality Report" section in `skills/gf-quality/SKILL.md` with:

```markdown
## Step 3: Quality Report

### Single-Language Report

```markdown
## Quality Gate Report

- Date: <date>
- Language: <detected language>
- Project: <repo name>

| Gate | Status | Details |
|------|--------|---------|
| 1. build | ✅/❌/N/A | <errors if any> |
| 2. test | ✅/❌/N/A | <failed tests if any> |
| 3. coverage | ✅/❌/N/A | <value vs threshold> |
| 4. format | ✅/❌/N/A | <files if diff> |
| 5. static | ✅/❌/N/A | <warnings if any> |
| 6. pre-commit | ✅/❌/N/A | <hook failures if any> |

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

**Report only. No auto-fix. No source modifications.**
```

- [ ] **Step 3: Verify syntax**

Proofread the updated sections. Ensure all code blocks are properly formatted and all tables render correctly.

- [ ] **Step 4: Commit**

```bash
git add skills/gf-quality/SKILL.md
git commit -m "docs(gf-quality): enhance SKILL.md with workspace-aware execution

- Update detection to scan 3 levels deep
- Add workspace marker detection (go.work, settings.gradle)
- Enhance multi-language aggregate report with per-language details
- Add detection summary table with workspace context
- Add inline failure details and consolidated actions"
```

---

### Task 11: Validate Example Projects

**Files:**
- Test: `examples/quality-gate/go/`
- Test: `examples/quality-gate/node/`
- Test: `examples/quality-gate/python/`

**Interfaces:**
- Consumes: Example projects created in Tasks 7-9
- Produces: Validation that all projects pass quality gates

- [ ] **Step 1: Validate Go example**

Run:

```bash
cd examples/quality-gate/go
gf quality
```

Expected: ALL CHECKS PASSED

If any gate fails, debug and fix the issue before proceeding.

- [ ] **Step 2: Validate Node.js example**

Run:

```bash
cd examples/quality-gate/node
npm install
gf quality
```

Expected: ALL CHECKS PASSED

If any gate fails, debug and fix the issue before proceeding.

- [ ] **Step 3: Validate Python example**

Run:

```bash
cd examples/quality-gate/python
python -m venv .venv
source .venv/bin/activate
pip install -e .
pip install pytest ruff
gf quality
```

Expected: ALL CHECKS PASSED

If any gate fails, debug and fix the issue before proceeding.

- [ ] **Step 4: Commit any fixes**

If any example projects required fixes:

```bash
git add examples/quality-gate/
git commit -m "fix(examples): fix validation issues in example projects

- <describe fixes>"
```

---

### Task 12: Dogfooding Validation

**Files:**
- Create: `docs/research/dogfooding-go.md`
- Create: `docs/research/dogfooding-node.md`
- Create: `docs/research/dogfooding-python.md`

**Interfaces:**
- Consumes: Enhanced gf-quality skill
- Produces: Validation reports for 3 real-world non-Rust projects

- [ ] **Step 1: Identify 3 non-Rust projects**

Select 3 projects (1 Go, 1 Node, 1 Python). Can be:
- Public GitHub repos
- Your own projects
- Sample projects from documentation

Record the project names and URLs.

- [ ] **Step 2: Run gf-quality on Go project**

Clone or navigate to the Go project. Run:

```bash
gf quality
```

Document results in `docs/research/dogfooding-go.md`:

```markdown
# Dogfooding Report: Go Project

**Project:** <name>
**URL:** <url>
**Date:** <date>

## Detection

- Detected language: Go
- Detection accuracy: ✅ Correct
- Scan depth: 3 levels

## Gate Execution

| Gate | Status | Details |
|------|--------|---------|
| build | ✅/❌/⏭️ | <details> |
| test | ✅/❌/⏭️ | <details> |
| coverage | ✅/❌/⏭️ | <details> |
| format | ✅/❌/⏭️ | <details> |
| static | ✅/❌/⏭️ | <details> |
| pre-commit | ✅/❌/⏭️ | <details> |

## Issues Encountered

- <issue-1>
- <issue-2>

## Fixes Applied

- <fix-1>
- <fix-2>

## Final Result

✅/❌ **QUALITY GATE PASSED/FAILED**

## Lessons Learned

- <lesson-1>
- <lesson-2>
```

- [ ] **Step 3: Run gf-quality on Node.js project**

Clone or navigate to the Node.js project. Run:

```bash
npm install  # if needed
gf quality
```

Document results in `docs/research/dogfooding-node.md` (same format as Go).

- [ ] **Step 4: Run gf-quality on Python project**

Clone or navigate to the Python project. Run:

```bash
python -m venv .venv
source .venv/bin/activate
pip install -e .
pip install pytest ruff  # if needed
gf quality
```

Document results in `docs/research/dogfooding-python.md` (same format as Go).

- [ ] **Step 5: Commit dogfooding reports**

```bash
git add docs/research/dogfooding-*.md
git commit -m "docs(gf-quality): add dogfooding validation reports

- Go project: <name>
- Node.js project: <name>
- Python project: <name>

All projects validated with gf-quality."
```

---

### Task 13: Documentation Sync

**Files:**
- Modify: `docs/references/gf-quality-params.md`
- Modify: `docs/research/skill-analysis-gf-quality.md`

**Interfaces:**
- Consumes: Enhanced gf-quality skill
- Produces: Updated documentation reflecting changes

- [ ] **Step 1: Update gf-quality-params.md**

Add sections to `docs/references/gf-quality-params.md` documenting:
- Workspace detection (3-level scan, workspace markers)
- Configuration guide structure
- Troubleshooting section structure
- Enhanced aggregate report format

- [ ] **Step 2: Update skill-analysis-gf-quality.md**

Update `docs/research/skill-analysis-gf-quality.md` to reflect:
- Configuration guides added ✅
- Troubleshooting sections added ✅
- Workspace detection enhanced ✅
- Example projects created ✅
- Overall score improvement

- [ ] **Step 3: Verify cross-references**

Check that all references between documents are correct:
- SKILL.md → references/*.md
- Design spec → implementation plan
- Dogfooding reports → example projects

- [ ] **Step 4: Commit documentation updates**

```bash
git add docs/references/gf-quality-params.md docs/research/skill-analysis-gf-quality.md
git commit -m "docs(gf-quality): sync documentation with enhanced skill

- Update gf-quality-params.md with workspace detection
- Update skill-analysis-gf-quality.md with improvements
- Verify all cross-references"
```

---

## Plan Complete

**Total tasks:** 13
**Estimated time:** 4-6 hours

**Execution options:**

1. **Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration
2. **Inline Execution** — Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?
