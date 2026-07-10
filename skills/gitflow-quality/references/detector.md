# Language Detection

Detect project language(s) by scanning marker files at root and in sub-directories.

## Detection Scope

Scan **up to 2 levels deep** from project root. This catches:
- Root-level projects (`Cargo.toml`, `go.mod`, etc.)
- Monorepo sub-projects (`apps/*/package.json`, `services/*/Cargo.toml`)
- Workspace member detection

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
# Scan root + 2 levels deep for all marker files
find . -maxdepth 3 \( \
  -name "Cargo.toml" -o \
  -name "go.mod" -o \
  -name "pom.xml" -o \
  -name "build.gradle" -o \
  -name "build.gradle.kts" -o \
  -name "pyproject.toml" -o \
  -name "setup.py" -o \
  -name "package.json" -o \
  -name "Gemfile" \
\) -not -path "*/node_modules/*" -not -path "*/target/*" -not -path "*/.git/*" -not -path "*/vendor/*"
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

## No Marker File (Generic)

If no marker file is found anywhere:

1. Check for `.pre-commit-config.yaml` at root → run `pre-commit run --all-files` only
2. If no pre-commit config → report "No project detected, no quality gates to run"
