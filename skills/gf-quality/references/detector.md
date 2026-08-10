# Language Detection

Detect project language(s) by scanning marker files at root and in sub-directories.

## Detection Scope

Scan **up to 3 levels deep** from project root. This catches:
- Root-level projects (`Cargo.toml`, `go.mod`, etc.)
- Monorepo sub-projects (`apps/*/package.json`, `services/*/Cargo.toml`)
- Workspace member detection
- Nested workspace configurations (`go.work`, `settings.gradle`)

## Detection Rules

Check these marker files at each scanned level:

| Marker File | Language | Reference |
|-------------|----------|-----------|
| `Cargo.toml` | Rust | `references/rust.md` |
| `go.mod` | Go | `references/go.md` |
| `pom.xml` | Java (Maven) | `references/java.md` |
| `build.gradle` / `build.gradle.kts` | Java (Gradle) | `references/java.md` |
| `pyproject.toml` | Python | `references/python.md` |
| `setup.py` / `setup.cfg` | Python | `references/python.md` |
| `package.json` | Node.js / TypeScript | `references/node.md` |
| `Gemfile` | Ruby | `references/ruby.md` |

## Detection Command

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

## Single-Language Project

If only one language is detected (possibly in multiple directories):

1. Load the matching `references/<lang>.md`
2. Run gates for that language across all detected directories
3. For Rust workspaces: a single `cargo build/test/...` at root covers all members

## Multi-Language Project

If multiple languages are detected:

1. **Present a summary** to the user:

```
Detected languages:
  1. Rust       → ./ (workspace root + crates/* + apps/server)
  2. Node.js    → ./apps/desktop/ (bun runtime)

Which to check? [1/2/all]
```

2. User selects:
   - **Single language** → load that reference, run gates
   - **all** → run each language's gates independently
3. Each language runs independently — one failure does NOT block others
4. Generate **aggregate report** at the end

## Aggregate Report (Multi-Language)

```markdown
## Quality Gate Report (Multi-Language)

| Language | Path | Build | Test | Coverage | Format | Static | Pre-commit | Result |
|----------|------|-------|------|----------|--------|--------|------------|--------|
| Rust     | ./   | ✅    | ✅   | ✅ 85%   | ✅     | ✅     | ✅         | PASS   |
| Node.js  | apps/desktop/ | ✅ | ❌ 2 failed | — | ✅ | ❌ 3 warnings | N/A | FAIL |

### Summary
- Rust: ALL CHECKS PASSED
- Node.js (apps/desktop): 2 test failures, 3 lint warnings

### Actions Required
- [ ] Fix 2 failing tests in apps/desktop
- [ ] Address 3 lint warnings in apps/desktop
```

## Runtime Detection (Node.js)

When Node.js is detected, also check for package manager lock files in the same directory:

| Lock File | Runtime |
|-----------|---------|
| `bun.lockb` / `bun.lock` | Bun |
| `pnpm-lock.yaml` | pnpm |
| `yarn.lock` | Yarn |
| `package-lock.json` | npm |

**Note:** A directory may have multiple lock files (e.g., during migration). Use the first match in order: bun → pnpm → yarn → npm.

## Exclusion Rules

Skip these directories during scanning:
- `node_modules/`
- `target/` (Rust build output)
- `.git/`
- `vendor/` (Go/PHP dependencies)
- `dist/` / `build/` (build output)
- `.cache/` / `.turbo/` (tool caches)

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

## No Marker File (Generic)

If no marker file is found anywhere:

1. Check for `.pre-commit-config.yaml` at root → run `pre-commit run --all-files` only
2. If no pre-commit config → report "No project detected, no quality gates to run"
