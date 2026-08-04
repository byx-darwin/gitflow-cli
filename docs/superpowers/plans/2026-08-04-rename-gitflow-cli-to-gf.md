# Rename gitflow-cli → gf Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rename all `gitflow-cli*` packages, binaries, and references to `gf*` across the entire workspace in a single atomic change.

**Architecture:** Mechanical find-and-replace across Cargo manifests, Rust source, documentation, CI/CD workflows, Homebrew formula, and skills. No functional changes — only name substitutions. Execute in dependency order: Cargo manifests → Rust source → tests → docs → CI/CD → validation.

**Tech Stack:** Rust 2024, Cargo workspace, GitHub Actions, Homebrew, trycmd

## Global Constraints

- All `gitflow-cli` → `gf`, all `gitflow_cli` → `gf_`
- Binary name: `gitflow-cli` → `gf`
- Cargo package names: `gitflow-cli*` → `gf*`
- crates.io: publish new `gf*` packages, old `gitflow-cli*` remain (no yank)
- GitHub repo URL unchanged: `https://github.com/byx-darwin/gitflow-cli`
- Directory structure unchanged: `crates/core`, `crates/github`, `apps/cli` keep names
- Skill directory names unchanged: `.claude/skills/gitflow-workflow/` etc.
- CHANGELOG.md: historical entries preserve original names, new entry documents rename
- contract.schema.json: `$id` URL changes to `https://gf.ai/schemas/...`

---

## File Structure

### Cargo Manifests (7 files)

| File | Change |
|------|--------|
| `Cargo.toml` (root) | Update `[workspace.dependencies]`: `gitflow-cli*` → `gf*` |
| `apps/cli/Cargo.toml` | `name = "gf"`, `[[bin]] name = "gf"`, deps → `gf-core` etc. |
| `crates/core/Cargo.toml` | `name = "gf-core"`, description/keywords/docs updates |
| `crates/github/Cargo.toml` | `name = "gf-github"`, dep → `gf-core` |
| `crates/gitlab/Cargo.toml` | `name = "gf-gitlab"`, dep → `gf-core` |
| `crates/gitcode/Cargo.toml` | `name = "gf-gitcode"`, dep → `gf-core` |

### Rust Source (~33 files)

**Pattern:** Global replace `gitflow_cli_core` → `gf_core`, `gitflow_cli_github` → `gf_github`, etc.

**Key files:**
- `apps/cli/src/main.rs`: `#[command(name = "gf")]`, `about`, doc comments
- `apps/cli/src/commands/completions.rs`: filenames `"gf.bash"`, `"_gf"`, `"gf.fish"`, test assertions
- `apps/cli/build.rs`: doc comment
- All `lib.rs`: doc comments with module paths
- All files with `use gitflow_cli_*::` imports

### Test Fixtures (2 files)

| File | Change |
|------|--------|
| `apps/cli/tests/cmd/version.trycmd` | `$ gf --version`, `gf [..]` |
| `apps/cli/tests/cmd/help.trycmd` | `$ gf --help`, `gf CLI tool`, `Usage: gf` |

### Documentation (~141 markdown files)

- `README.md`, `CONTRIBUTING.md`, `CLAUDE.md`
- `CHANGELOG.md`: add `[Unreleased]` section with breaking change note
- `docs/*.md`, `specs/*.md`
- `crates/*/README.md`

### Skills (24 files, 181 references)

- All `SKILL.md` files: command examples `gitflow-cli` → `gf`
- `description` fields (EN + ZH)
- `contract.schema.json`: `$id` → `https://gf.ai/schemas/...`
- `references.md`

### CI/CD (5 workflow files)

| File | Change |
|------|--------|
| `.github/workflows/build.yml` | `BIN_NAME="gf${{ matrix.ext }}"` |
| `.github/workflows/release.yml` | `BINARY_NAME: gf`, artifact URLs, cargo publish commands, Homebrew path |
| `.github/workflows/smoke-test.yml` | `gf --version`, `gf --help` |
| `.github/workflows/ci.yml` | `./target/release/gf` |
| `.github/workflows/upstream-patrol.yml` | Step name |

### Homebrew Formula

- Rename file: `HomebrewFormula/gitflow-cli.rb` → `HomebrewFormula/gf.rb`
- Update class name, bin.install, URLs, test block

### Makefile

- Update output messages and `local-install` target

---

## Tasks

### Task 1: Cargo Manifests — Workspace Root

**Files:**
- Modify: `Cargo.toml` (root)

**Interfaces:**
- Produces: Updated `[workspace.dependencies]` section with `gf*` package names

- [ ] **Step 1: Update workspace dependencies**

Open `Cargo.toml` and replace in `[workspace.dependencies]`:

```toml
# BEFORE:
gitflow-cli-core = { version = "0.9.0", path = "crates/core" }
gitflow-cli-github = { version = "0.9.0", path = "crates/github" }
gitflow-cli-gitlab = { version = "0.9.0", path = "crates/gitlab" }
gitflow-cli-gitcode = { version = "0.9.0", path = "crates/gitcode" }
gitflow-cli = { version = "0.9.0", path = "apps/cli" }

# AFTER:
gf-core = { version = "0.9.0", path = "crates/core" }
gf-github = { version = "0.9.0", path = "crates/github" }
gf-gitlab = { version = "0.9.0", path = "crates/gitlab" }
gf-gitcode = { version = "0.9.0", path = "crates/gitcode" }
gf = { version = "0.9.0", path = "apps/cli" }
```

- [ ] **Step 2: Verify changes**

Run: `grep -A5 "workspace.dependencies" Cargo.toml | grep "gf"`
Expected: All 5 `gf*` entries visible

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "chore: rename workspace dependencies gitflow-cli → gf"
```

---

### Task 2: Cargo Manifests — Core Crate

**Files:**
- Modify: `crates/core/Cargo.toml`

**Interfaces:**
- Consumes: Updated workspace deps from Task 1
- Produces: `gf-core` package with updated metadata

- [ ] **Step 1: Update package name and metadata**

Open `crates/core/Cargo.toml` and make these changes:

```toml
# BEFORE:
[package]
name = "gitflow-cli-core"
description = "gitflow-cli 核心库：Platform trait 抽象、跨平台适配器与 toon 输出。Core library: platform abstraction, adapters & toon output."
categories = ["development-tools"]
keywords = ["gitflow", "git", "cli", "core", "workflow"]
# ...
documentation = "https://docs.rs/gitflow-cli-core"

# AFTER:
[package]
name = "gf-core"
description = "gf 核心库：Platform trait 抽象、跨平台适配器与 toon 输出。Core library: platform abstraction, adapters & toon output."
categories = ["development-tools"]
keywords = ["gitflow", "git", "cli", "core", "workflow", "gf"]
# ...
documentation = "https://docs.rs/gf-core"
```

- [ ] **Step 2: Verify**

Run: `grep "name = " crates/core/Cargo.toml | head -1`
Expected: `name = "gf-core"`

- [ ] **Step 3: Commit**

```bash
git add crates/core/Cargo.toml
git commit -m "chore: rename gitflow-cli-core → gf-core"
```

---

### Task 3: Cargo Manifests — Platform Crates

**Files:**
- Modify: `crates/github/Cargo.toml`
- Modify: `crates/gitlab/Cargo.toml`
- Modify: `crates/gitcode/Cargo.toml`

**Interfaces:**
- Consumes: `gf-core` from Task 2
- Produces: `gf-github`, `gf-gitlab`, `gf-gitcode` packages

- [ ] **Step 1: Update crates/github/Cargo.toml**

```toml
# BEFORE:
[package]
name = "gitflow-cli-github"
description = "gitflow-cli 的 GitHub 适配器..."
documentation = "https://docs.rs/gitflow-cli-github"

[dependencies]
gitflow-cli-core.workspace = true

# AFTER:
[package]
name = "gf-github"
description = "gf 的 GitHub 适配器..."
documentation = "https://docs.rs/gf-github"

[dependencies]
gf-core.workspace = true
```

- [ ] **Step 2: Update crates/gitlab/Cargo.toml**

Same pattern as github:
- `name = "gitflow-cli-gitlab"` → `"gf-gitlab"`
- `description`: `gitflow-cli` → `gf`
- `documentation`: `gitflow-cli-gitlab` → `gf-gitlab`
- `gitflow-cli-core.workspace = true` → `gf-core.workspace = true`

- [ ] **Step 3: Update crates/gitcode/Cargo.toml**

Same pattern:
- `name = "gitflow-cli-gitcode"` → `"gf-gitcode"`
- `description`: `gitflow-cli` → `gf`
- `documentation`: `gitflow-cli-gitcode` → `gf-gitcode`
- `gitflow-cli-core.workspace = true` → `gf-core.workspace = true`

- [ ] **Step 4: Verify all three crates**

Run: `grep "^name = " crates/github/Cargo.toml crates/gitlab/Cargo.toml crates/gitcode/Cargo.toml`
Expected:
```
crates/github/Cargo.toml:name = "gf-github"
crates/gitlab/Cargo.toml:name = "gf-gitlab"
crates/gitcode/Cargo.toml:name = "gf-gitcode"
```

- [ ] **Step 5: Commit**

```bash
git add crates/github/Cargo.toml crates/gitlab/Cargo.toml crates/gitcode/Cargo.toml
git commit -m "chore: rename platform crates gitflow-cli-* → gf-*"
```

---

### Task 4: Cargo Manifests — CLI App

**Files:**
- Modify: `apps/cli/Cargo.toml`

**Interfaces:**
- Consumes: All `gf*` crates from Tasks 1-3
- Produces: `gf` binary package

- [ ] **Step 1: Update package name and binary name**

```toml
# BEFORE:
[package]
name = "gitflow-cli"
description = "跨平台 Git 工程化工作流编排框架 CLI..."
documentation = "https://docs.rs/gitflow-cli"

[[bin]]
name = "gitflow-cli"
path = "src/main.rs"

[dependencies]
gitflow-cli-core = { workspace = true }
gitflow-cli-github = { workspace = true }
gitflow-cli-gitlab = { workspace = true }
gitflow-cli-gitcode = { workspace = true }

# AFTER:
[package]
name = "gf"
description = "跨平台 Git 工程化工作流编排框架 CLI..."
documentation = "https://docs.rs/gf"

[[bin]]
name = "gf"
path = "src/main.rs"

[dependencies]
gf-core = { workspace = true }
gf-github = { workspace = true }
gf-gitlab = { workspace = true }
gf-gitcode = { workspace = true }
```

Note: `description` keeps Chinese text but change `gitflow-cli` → `gf` within it.

- [ ] **Step 2: Verify**

Run: `grep -E "^name = |^\[\[bin\]\]|^name = " apps/cli/Cargo.toml | head -4`
Expected: `name = "gf"` for package, `name = "gf"` for bin

- [ ] **Step 3: Commit**

```bash
git add apps/cli/Cargo.toml
git commit -m "chore: rename CLI package and binary gitflow-cli → gf"
```

---

### Task 5: Rust Source — Global Import Replacement

**Files:**
- Modify: All `.rs` files with `use gitflow_cli_*::` imports (~33 files)

**Interfaces:**
- Consumes: Updated Cargo manifests from Tasks 1-4
- Produces: Updated Rust imports using `gf_*` crate names

- [ ] **Step 1: Find all files needing import updates**

Run: `grep -r "use gitflow_cli_" --include="*.rs" -l`
Expected: ~33 files listed

- [ ] **Step 2: Perform global replacement**

Run these sed commands to update all imports:

```bash
# Replace all gitflow_cli_* with gf_* in Rust source files
find . -name "*.rs" -type f -exec sed -i '' 's/gitflow_cli_core/gf_core/g' {} +
find . -name "*.rs" -type f -exec sed -i '' 's/gitflow_cli_github/gf_github/g' {} +
find . -name "*.rs" -type f -exec sed -i '' 's/gitflow_cli_gitlab/gf_gitlab/g' {} +
find . -name "*.rs" -type f -exec sed -i '' 's/gitflow_cli_gitcode/gf_gitcode/g' {} +
```

- [ ] **Step 3: Verify no stale imports remain**

Run: `grep -r "gitflow_cli_" --include="*.rs"`
Expected: No output (or only in comments/doc strings that reference historical names)

- [ ] **Step 4: Verify build compiles**

Run: `cargo check --workspace 2>&1 | head -20`
Expected: No compilation errors related to unresolved imports

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "refactor: update Rust imports gitflow_cli_* → gf_*"
```

---

### Task 6: Rust Source — Command Name and Metadata

**Files:**
- Modify: `apps/cli/src/main.rs`
- Modify: `apps/cli/build.rs`

**Interfaces:**
- Produces: Updated CLI command name and doc comments

- [ ] **Step 1: Update apps/cli/src/main.rs**

Open `apps/cli/src/main.rs` and make these changes:

```rust
// BEFORE:
//! gitflow-cli CLI entrypoint.

/// gitflow-cli command-line interface.
#[command(name = "gitflow-cli", about = "gitflow-cli CLI tool", author)]

// AFTER:
//! gf CLI entrypoint.

/// gf command-line interface.
#[command(name = "gf", about = "gf CLI tool", author)]
```

- [ ] **Step 2: Update apps/cli/build.rs doc comment**

```rust
// BEFORE:
//! Build script for `gitflow-cli`.

// AFTER:
//! Build script for `gf`.
```

- [ ] **Step 3: Verify command name**

Run: `grep -n "command(name" apps/cli/src/main.rs`
Expected: `#[command(name = "gf", about = "gf CLI tool", author)]`

- [ ] **Step 4: Commit**

```bash
git add apps/cli/src/main.rs apps/cli/build.rs
git commit -m "refactor: update CLI command name and metadata to gf"
```

---

### Task 7: Shell Completion Code and Tests

**Files:**
- Modify: `apps/cli/src/commands/completions.rs`

**Interfaces:**
- Consumes: Updated command name from Task 6
- Produces: Updated completion filenames and test assertions

- [ ] **Step 1: Update completion filename method**

In `apps/cli/src/commands/completions.rs`, find the `completion_filename` method and update:

```rust
// BEFORE:
pub fn completion_filename(self) -> &'static str {
    match self {
        Shell::Bash => "gitflow-cli.bash",
        Shell::Zsh => "_gitflow-cli",
        Shell::Fish => "gitflow-cli.fish",
    }
}

// AFTER:
pub fn completion_filename(self) -> &'static str {
    match self {
        Shell::Bash => "gf.bash",
        Shell::Zsh => "_gf",
        Shell::Fish => "gf.fish",
    }
}
```

- [ ] **Step 2: Update doc comment**

```rust
// BEFORE:
/// Return the conventional completion-file name for `gitflow-cli` in this shell.

// AFTER:
/// Return the conventional completion-file name for `gf` in this shell.
```

- [ ] **Step 3: Update test assertions**

Find and update these tests:

```rust
// test_should_return_correct_bash_filename
assert_eq!(Shell::Bash.completion_filename(), "gf.bash");

// test_should_return_correct_zsh_filename
assert_eq!(Shell::Zsh.completion_filename(), "_gf");

// test_should_return_correct_fish_filename
assert_eq!(Shell::Fish.completion_filename(), "gf.fish");

// test_should_return_correct_bash_install_dir (path in comment)
// /tmp/gitflow-cli-test-home → /tmp/gf-test-home (optional, cosmetic)

// test_should_generate_bash_completion_contains_function
write_completion(&mut cmd, Shell::Bash, "gf", &mut output);

// test_should_generate_zsh_completion_contains_compdef
write_completion(&mut cmd, Shell::Zsh, "gf", &mut output);

// test_should_generate_fish_completion_contains_complete
write_completion(&mut cmd, Shell::Fish, "gf", &mut output);

// test_should_generate_non_empty_output_for_all_shells
write_completion(&mut cmd, shell, "gf", &mut output);

// test_should_produce_different_output_per_shell
write_completion(&mut cmd_bash, Shell::Bash, "gf", &mut bash_out);
write_completion(&mut cmd_zsh, Shell::Zsh, "gf", &mut zsh_out);
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p gf --lib commands::completions`
Expected: All completion tests pass

- [ ] **Step 5: Commit**

```bash
git add apps/cli/src/commands/completions.rs
git commit -m "refactor: update shell completion filenames and tests for gf"
```

---

### Task 8: Test Fixtures (trycmd)

**Files:**
- Modify: `apps/cli/tests/cmd/version.trycmd`
- Modify: `apps/cli/tests/cmd/help.trycmd`

**Interfaces:**
- Consumes: Updated binary name from Task 6
- Produces: Updated test expectations

- [ ] **Step 1: Update version.trycmd**

```trycmd
# BEFORE:
$ gitflow-cli --version
gitflow-cli [..]

# AFTER:
$ gf --version
gf [..]
```

- [ ] **Step 2: Update help.trycmd**

```trycmd
# BEFORE:
$ gitflow-cli --help
gitflow-cli CLI tool

Usage: gitflow-cli [OPTIONS] <COMMAND>

# AFTER:
$ gf --help
gf CLI tool

Usage: gf [OPTIONS] <COMMAND>
```

- [ ] **Step 3: Run trycmd tests**

Run: `cargo test -p gf --test cmd`
Expected: All trycmd tests pass

- [ ] **Step 4: Commit**

```bash
git add apps/cli/tests/cmd/
git commit -m "test: update trycmd fixtures for gf binary name"
```

---

### Task 9: Documentation — README and CHANGELOG

**Files:**
- Modify: `README.md`
- Modify: `CHANGELOG.md`
- Modify: `CONTRIBUTING.md`
- Modify: `crates/core/README.md`
- Modify: `crates/github/README.md`
- Modify: `crates/gitlab/README.md`
- Modify: `crates/gitcode/README.md`

**Interfaces:**
- Produces: Updated documentation with `gf` command references

- [ ] **Step 1: Update README.md**

Run: `sed -i '' 's/gitflow-cli/gf/g' README.md`

Note: This replaces ALL occurrences including GitHub URLs. For GitHub URLs, manually restore `byx-darwin/gitflow-cli` since repo name is unchanged.

Alternative (more precise):
```bash
# Replace command references but preserve repo URLs
sed -i '' 's/`gitflow-cli/`gf/g' README.md
sed -i '' 's/cargo install gitflow-cli/cargo install gf/g' README.md
# Do NOT change https://github.com/byx-darwin/gitflow-cli
```

- [ ] **Step 2: Update CHANGELOG.md**

Add at the top (after any header):

```markdown
## [Unreleased]

### ⚠ BREAKING CHANGES

- rename: `gitflow-cli` → `gf` — all crate names renamed accordingly
  - `gitflow-cli` → `gf`
  - `gitflow-cli-core` → `gf-core`
  - `gitflow-cli-github` → `gf-github`
  - `gitflow-cli-gitlab` → `gf-gitlab`
  - `gitflow-cli-gitcode` → `gf-gitcode`
  - Binary name: `gitflow-cli` → `gf`
```

- [ ] **Step 3: Update CONTRIBUTING.md**

Run: `sed -i '' 's/gitflow-cli/gf/g' CONTRIBUTING.md`
(Review for any repo URLs that should remain unchanged)

- [ ] **Step 4: Update crate READMEs**

Run:
```bash
sed -i '' 's/gitflow-cli-core/gf-core/g' crates/core/README.md
sed -i '' 's/gitflow-cli-github/gf-github/g' crates/github/README.md
sed -i '' 's/gitflow-cli-gitlab/gf-gitlab/g' crates/gitlab/README.md
sed -i '' 's/gitflow-cli-gitcode/gf-gitcode/g' crates/gitcode/README.md
# Update general gitflow-cli references in crate READMEs
sed -i '' 's/gitflow-cli/gf/g' crates/*/README.md
```

- [ ] **Step 5: Verify no stale command references**

Run: `grep -r "gitflow-cli" README.md CONTRIBUTING.md crates/*/README.md | grep -v "github.com/byx-darwin/gitflow-cli"`
Expected: Only repo URL references remain

- [ ] **Step 6: Commit**

```bash
git add README.md CHANGELOG.md CONTRIBUTING.md crates/*/README.md
git commit -m "docs: update READMEs and CHANGELOG for gf rename"
```

---

### Task 10: Documentation — CLAUDE.md, docs/, specs/

**Files:**
- Modify: `CLAUDE.md`
- Modify: All `.md` files in `docs/`
- Modify: All `.md` files in `specs/`

**Interfaces:**
- Produces: Updated documentation throughout project

- [ ] **Step 1: Update CLAUDE.md**

Run:
```bash
# Replace command references
sed -i '' 's/`gitflow-cli/`gf/g' CLAUDE.md
sed -i '' 's/gitflow-cli --/gf --/g' CLAUDE.md
# Preserve repo URLs
# Review manually for any missed references
```

- [ ] **Step 2: Update docs/ directory**

Run:
```bash
find docs -name "*.md" -type f -exec sed -i '' 's/`gitflow-cli/`gf/g' {} +
find docs -name "*.md" -type f -exec sed -i '' 's/gitflow-cli --/gf --/g' {} +
# Preserve repo URLs: github.com/byx-darwin/gitflow-cli stays unchanged
```

- [ ] **Step 3: Update specs/ directory**

Run:
```bash
find specs -name "*.md" -type f -exec sed -i '' 's/`gitflow-cli/`gf/g' {} +
find specs -name "*.md" -type f -exec sed -i '' 's/gitflow-cli --/gf --/g' {} +
```

- [ ] **Step 4: Verify**

Run: `grep -r "gitflow-cli" docs/ specs/ CLAUDE.md | grep -v "github.com/byx-darwin/gitflow-cli" | grep -v "CHANGELOG"`
Expected: Minimal output (only historical references in reports)

- [ ] **Step 5: Commit**

```bash
git add CLAUDE.md docs/ specs/
git commit -m "docs: update CLAUDE.md, docs/, specs/ for gf rename"
```

---

### Task 11: Skills — SKILL.md Files

**Files:**
- Modify: All `SKILL.md` files in `.claude/skills/*/SKILL.md` (20+ files)

**Interfaces:**
- Produces: Updated skill documentation with `gf` command references

- [ ] **Step 1: Find all SKILL.md files with references**

Run: `grep -r "gitflow-cli" .claude/skills/ --include="*.md" -l`
Expected: ~20+ SKILL.md files

- [ ] **Step 2: Update all SKILL.md files**

Run:
```bash
find .claude/skills -name "SKILL.md" -type f -exec sed -i '' 's/`gitflow-cli/`gf/g' {} +
find .claude/skills -name "SKILL.md" -type f -exec sed -i '' 's/gitflow-cli --/gf --/g' {} +
find .claude/skills -name "SKILL.md" -type f -exec sed -i '' 's/command -v gitflow-cli/command -v gf/g' {} +
```

- [ ] **Step 3: Update description fields (both EN and ZH)**

The `description:` field in each SKILL.md contains bilingual text. Update both:
- English: "gitflow-cli" → "gf"
- Chinese: "gitflow-cli" → "gf"

Run:
```bash
find .claude/skills -name "SKILL.md" -type f -exec sed -i '' 's/通过 gitflow-cli/通过 gf/g' {} +
find .claude/skills -name "SKILL.md" -type f -exec sed -i '' 's/use gitflow-cli/use gf/g' {} +
# Add more patterns as needed based on grep results
```

- [ ] **Step 4: Verify**

Run: `grep -r "gitflow-cli" .claude/skills/ --include="*.md" | wc -l`
Expected: Close to 0 (only repo URLs should remain)

- [ ] **Step 5: Commit**

```bash
git add .claude/skills/
git commit -m "docs: update skill SKILL.md files for gf rename"
```

---

### Task 12: Skills — Schema and References

**Files:**
- Modify: `.claude/skills/gitflow-workflow/contract.schema.json`
- Modify: `.claude/skills/gitflow-workflow/references.md`

**Interfaces:**
- Produces: Updated schema ID and reference documentation

- [ ] **Step 1: Update contract.schema.json $id**

```json
// BEFORE:
"$id": "https://gitflow-cli.ai/schemas/workflow-contract-v1.1.json",

// AFTER:
"$id": "https://gf.ai/schemas/workflow-contract-v1.1.json",
```

- [ ] **Step 2: Update references.md**

Run: `sed -i '' 's/gitflow-cli/gf/g' .claude/skills/gitflow-workflow/references.md`

- [ ] **Step 3: Verify**

Run: `grep "gitflow-cli" .claude/skills/gitflow-workflow/contract.schema.json .claude/skills/gitflow-workflow/references.md`
Expected: No output (or only repo URLs)

- [ ] **Step 4: Commit**

```bash
git add .claude/skills/gitflow-workflow/
git commit -m "chore: update workflow schema $id and references for gf"
```

---

### Task 13: CI/CD Workflows

**Files:**
- Modify: `.github/workflows/build.yml`
- Modify: `.github/workflows/release.yml`
- Modify: `.github/workflows/smoke-test.yml`
- Modify: `.github/workflows/ci.yml`
- Modify: `.github/workflows/upstream-patrol.yml`

**Interfaces:**
- Produces: Updated CI/CD workflows referencing `gf` binary

- [ ] **Step 1: Update build.yml**

```yaml
# BEFORE:
BIN_NAME="gitflow-cli${{ matrix.ext }}"

# AFTER:
BIN_NAME="gf${{ matrix.ext }}"
```

- [ ] **Step 2: Update release.yml**

```yaml
# BEFORE:
env:
  BINARY_NAME: gitflow-cli
  # ... artifact URLs with gitflow-cli-

# AFTER:
env:
  BINARY_NAME: gf
  # ... artifact URLs with gf-
```

Also update:
- `cargo publish -p gitflow-cli-core` → `cargo publish -p gf-core`
- `cargo publish -p gitflow-cli-github` → `cargo publish -p gf-github`
- `cargo publish -p gitflow-cli-gitlab` → `cargo publish -p gf-gitlab`
- `cargo publish -p gitflow-cli-gitcode` → `cargo publish -p gf-gitcode`
- `cargo publish -p gitflow-cli` → `cargo publish -p gf`
- `HomebrewFormula/gitflow-cli.rb` → `HomebrewFormula/gf.rb`

Run:
```bash
sed -i '' 's/BINARY_NAME: gitflow-cli/BINARY_NAME: gf/g' .github/workflows/release.yml
sed -i '' 's/gitflow-cli-aarch64/gf-aarch64/g' .github/workflows/release.yml
sed -i '' 's/gitflow-cli-x86_64/gf-x86_64/g' .github/workflows/release.yml
sed -i '' 's/gitflow-cli-core/gf-core/g' .github/workflows/release.yml
sed -i '' 's/gitflow-cli-github/gf-github/g' .github/workflows/release.yml
sed -i '' 's/gitflow-cli-gitlab/gf-gitlab/g' .github/workflows/release.yml
sed -i '' 's/gitflow-cli-gitcode/gf-gitcode/g' .github/workflows/release.yml
sed -i '' 's/-p gitflow-cli /-p gf /g' .github/workflows/release.yml
sed -i '' 's/HomebrewFormula\/gitflow-cli\.rb/HomebrewFormula\/gf.rb/g' .github/workflows/release.yml
```

- [ ] **Step 3: Update smoke-test.yml**

```yaml
# BEFORE:
gitflow-cli --version
gitflow-cli --help

# AFTER:
gf --version
gf --help
```

Run: `sed -i '' 's/gitflow-cli --/gf --/g' .github/workflows/smoke-test.yml`

- [ ] **Step 4: Update ci.yml**

```yaml
# BEFORE:
./target/release/gitflow-cli --help
./target/release/gitflow-cli issue list --help
./target/release/gitflow-cli pr list --help

# AFTER:
./target/release/gf --help
./target/release/gf issue list --help
./target/release/gf pr list --help
```

Run: `sed -i '' 's/gitflow-cli/gf/g' .github/workflows/ci.yml`

- [ ] **Step 5: Update upstream-patrol.yml**

```yaml
# BEFORE:
- name: Build gitflow-cli

# AFTER:
- name: Build gf
```

Run: `sed -i '' 's/Build gitflow-cli/Build gf/g' .github/workflows/upstream-patrol.yml`

- [ ] **Step 6: Verify**

Run: `grep -r "gitflow-cli" .github/workflows/`
Expected: Only repo URLs remain

- [ ] **Step 7: Commit**

```bash
git add .github/workflows/
git commit -m "ci: update workflow binary references for gf rename"
```

---

### Task 14: Homebrew Formula

**Files:**
- Delete: `HomebrewFormula/gitflow-cli.rb`
- Create: `HomebrewFormula/gf.rb`

**Interfaces:**
- Produces: Renamed and updated Homebrew formula

- [ ] **Step 1: Create new formula file**

Create `HomebrewFormula/gf.rb` with updated content:

```ruby
class Gf < Formula
  desc "Multi-platform Git forge CLI — unified interface for GitHub, GitLab, and GitCode"
  homepage "https://github.com/byx-darwin/gitflow-cli"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/byx-darwin/gitflow-cli/releases/download/v0.3.0/gf-aarch64-apple-darwin.tar.gz"
      sha256 "9efe4aed61efb3f353e9838c9b05fc903066526b1c12c1a87da99c6ba1ee062d"
    else
      url "https://github.com/byx-darwin/gitflow-cli/releases/download/v0.3.0/gf-x86_64-apple-darwin.tar.gz"
      sha256 "79b9d9104a42e7c107603b3e03069c1710ed8e32d35239d9e059ee8918b0db26"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/byx-darwin/gitflow-cli/releases/download/v0.3.0/gf-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "fd9d6c8a340b3c61eb8bc417192ab518db429d5a0bb3e43c24ba435cd5bbc7f9"
    else
      url "https://github.com/byx-darwin/gitflow-cli/releases/download/v0.3.0/gf-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "fcbc9a7a2b387f244d6dbc041f395fb0bc9dc895dcb672ac7aee5dd3ae400eea"
    end
  end

  depends_on "gh"

  def install
    bin.install "gf"
    generate_completions_from_executable(bin/"gf", "completions")
  end

  test do
    system "#{bin}/gf", "--version"
    system "#{bin}/gf", "--help"
  end
end
```

Note: SHA256 values are placeholders — they'll be updated by the release workflow when actual binaries are built.

- [ ] **Step 2: Delete old formula file**

```bash
git rm HomebrewFormula/gitflow-cli.rb
```

- [ ] **Step 3: Verify**

Run: `ls HomebrewFormula/`
Expected: Only `gf.rb`

- [ ] **Step 4: Commit**

```bash
git add HomebrewFormula/
git commit -m "chore: rename Homebrew formula gitflow-cli.rb → gf.rb"
```

---

### Task 15: Makefile

**Files:**
- Modify: `Makefile`

**Interfaces:**
- Produces: Updated Makefile output messages

- [ ] **Step 1: Update build target**

```makefile
# BEFORE:
build: ## Compile the project
	@cargo build
	@echo "✓ Debug build complete: target/debug/gitflow-cli"

# AFTER:
build: ## Compile the project
	@cargo build
	@echo "✓ Debug build complete: target/debug/gf"
```

- [ ] **Step 2: Update build-release target**

```makefile
# BEFORE:
	@echo "✓ Release build complete: target/release/gitflow-cli"

# AFTER:
	@echo "✓ Release build complete: target/release/gf"
```

- [ ] **Step 3: Update local-install target**

```makefile
# BEFORE:
local-install: ## Install gitflow-cli to ~/.cargo/bin (release build)
	@echo "Installing gitflow-cli to ~/.cargo/bin..."
	@cargo install --path apps/cli --force --locked
	@echo "✓ Installed successfully"
	@gitflow-cli --version

# AFTER:
local-install: ## Install gf to ~/.cargo/bin (release build)
	@echo "Installing gf to ~/.cargo/bin..."
	@cargo install --path apps/cli --force --locked
	@echo "✓ Installed successfully"
	@gf --version
```

- [ ] **Step 4: Verify**

Run: `grep "gitflow-cli" Makefile`
Expected: No output

- [ ] **Step 5: Commit**

```bash
git add Makefile
git commit -m "chore: update Makefile for gf binary name"
```

---

### Task 16: E2E Test Crates

**Files:**
- Modify: `crates/e2e-core/src/tty.rs` (if references exist)
- Modify: `crates/e2e-core/tests/harness.rs` (if references exist)

**Interfaces:**
- Produces: Updated E2E test code

- [ ] **Step 1: Check for references**

Run: `grep -r "gitflow-cli" crates/e2e-core/ crates/e2e-github/ --include="*.rs"`

- [ ] **Step 2: Update if found**

If references exist, replace:
```bash
find crates/e2e-core crates/e2e-github -name "*.rs" -type f -exec sed -i '' 's/gitflow-cli/gf/g' {} +
```

- [ ] **Step 3: Verify**

Run: `grep -r "gitflow-cli" crates/e2e-core/ crates/e2e-github/ --include="*.rs"`
Expected: No output

- [ ] **Step 4: Commit (if changes made)**

```bash
git add crates/e2e-core/ crates/e2e-github/
git commit -m "test: update E2E test crates for gf rename"
```

(If no changes needed, skip commit)

---

### Task 17: Local Configuration (Optional)

**Files:**
- Modify: `.claude/settings.local.json` (optional)

**Interfaces:**
- Produces: Updated local permission patterns

- [ ] **Step 1: Update allow patterns (optional)**

Run:
```bash
sed -i '' 's/Bash(gitflow-cli/Bash(gf/g' .claude/settings.local.json
sed -i '' 's/gitflow-cli \*/gf */g' .claude/settings.local.json
```

Note: This is a local config file. Changes are optional and user-specific.

- [ ] **Step 2: Verify**

Run: `grep "gitflow-cli" .claude/settings.local.json | head -5`
Expected: Mostly `gf` references (some path references may remain)

- [ ] **Step 3: Commit (optional)**

```bash
git add .claude/settings.local.json
git commit -m "chore: update local settings for gf binary"
```

---

### Task 18: Cargo.lock Regeneration

**Files:**
- Auto-generated: `Cargo.lock`

**Interfaces:**
- Consumes: All Cargo manifest changes from Tasks 1-4
- Produces: Updated lockfile with `gf*` package names

- [ ] **Step 1: Regenerate Cargo.lock**

Run: `cargo build --workspace 2>&1 | tail -20`
Expected: Build succeeds, Cargo.lock updated

- [ ] **Step 2: Verify package names in lockfile**

Run: `grep "name = \"gf" Cargo.lock`
Expected:
```
name = "gf"
name = "gf-core"
name = "gf-github"
name = "gf-gitlab"
name = "gf-gitcode"
```

- [ ] **Step 3: Verify no stale names**

Run: `grep "name = \"gitflow-cli" Cargo.lock`
Expected: No output

- [ ] **Step 4: Commit**

```bash
git add Cargo.lock
git commit -m "chore: regenerate Cargo.lock with gf package names"
```

---

### Task 19: Full Validation

**Files:**
- None (validation only)

**Interfaces:**
- Consumes: All changes from Tasks 1-18
- Produces: Validation report

- [ ] **Step 1: Run full build**

Run: `cargo build --all-targets --all-features 2>&1 | tail -30`
Expected: Build succeeds with no errors

- [ ] **Step 2: Run full test suite**

Run: `cargo test --all-targets --all-features 2>&1 | tail -50`
Expected: All tests pass

- [ ] **Step 3: Verify binary name**

Run: `./target/debug/gf --version`
Expected: `gf x.y.z` (version number)

- [ ] **Step 4: Verify help output**

Run: `./target/debug/gf --help | head -5`
Expected: Shows `gf CLI tool` and `Usage: gf`

- [ ] **Step 5: Check for stale references**

Run:
```bash
grep -r "gitflow-cli" --include="*.rs" --include="*.toml" . | \
  grep -v "github.com/byx-darwin/gitflow-cli" | \
  grep -v "CHANGELOG.md" | \
  grep -v "target/"
```
Expected: Minimal output (only historical references)

- [ ] **Step 6: Verify Cargo metadata**

Run: `cargo metadata --format-version 1 --no-deps | jq '.packages[].name'`
Expected:
```
"gf"
"gf-core"
"gf-github"
"gf-gitlab"
"gf-gitcode"
```

- [ ] **Step 7: Verify shell completions**

Run: `./target/debug/gf completions bash | head -5`
Expected: Contains `gf` references (not `gitflow-cli`)

- [ ] **Step 8: Report results**

Document validation results:
- ✅ Build: PASS/FAIL
- ✅ Tests: PASS/FAIL (count)
- ✅ Binary name: CORRECT/INCORRECT
- ✅ Help output: CORRECT/INCORRECT
- ✅ Stale references: NONE/FOUND
- ✅ Cargo metadata: CORRECT/INCORRECT
- ✅ Shell completions: CORRECT/INCORRECT

---

### Task 20: Final Commit and Summary

**Files:**
- None (summary only)

**Interfaces:**
- Produces: Final status report

- [ ] **Step 1: Verify clean working tree**

Run: `git status`
Expected: Clean working tree (all changes committed)

- [ ] **Step 2: Count total changes**

Run: `git log --oneline main..HEAD | wc -l`
Expected: Number of commits made

- [ ] **Step 3: Create summary**

Document:
- Total commits: N
- Files changed: N
- Lines added/removed: N
- All validation checks: PASS

- [ ] **Step 4: Ready for PR**

The branch is now ready for PR creation. All rename tasks complete, all validation passed.
