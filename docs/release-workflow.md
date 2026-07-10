# Release Workflow Guide

## Overview

The release workflow has been improved with safety checks, version inference, and interactive previews. It supports publishing to both GitHub Releases and crates.io.

## Quick Start

```bash
# Interactive release (recommended)
make release

# Quick release without previews (for CI/automation)
make release-quick
```

## What Happens

### Pre-flight Checks
- ✓ Verifies you're on `main` branch
- ✓ Checks working directory is clean
- ✓ Runs all tests (`make test`)
- ✓ Runs clippy (`make clippy`)

### Version Inference
The script automatically analyzes commits since the last tag:
- `feat!` or `BREAKING CHANGE` → **Major** bump (1.0.0 → 2.0.0)
- `feat` → **Minor** bump (0.7.0 → 0.8.0)
- `fix`/`refactor`/`perf` → **Patch** bump (0.7.0 → 0.7.1)

### Interactive Preview
You'll see:
- Current version and last tag
- Inferred version bump
- Commit summary (features, fixes count)
- Changelog preview
- Option to override version (major/minor/patch/custom)

### Dry Run
Before executing, you'll see exactly what will happen:
- Version bump
- Changelog generation
- crates.io publish (optional)
- Tag creation
- Push to remote

### Execution
If confirmed, the script will:
1. Bump version in `Cargo.toml`
2. Commit version change
3. Generate `CHANGELOG.md`
4. Commit changelog
5. **Publish to crates.io** (optional, prompted)
6. Create Git tag
7. Push to `origin/main` with tags

## Publishing to crates.io

### Prerequisites

1. **Crates.io Account**:
   - Sign up at https://crates.io
   - Verify your email

2. **Authentication**:
```bash
# Login to crates.io
cargo login

# Or use API token
export CARGO_REGISTRY_TOKEN=<your-token>
```

3. **Verify Crate Metadata**:
```bash
# Check package is ready
cargo package --list
cargo package --no-verify
```

### Publish Options

**Option A: Publish during release**
```bash
make release
# When prompted: "Publish to crates.io?" → y
```

**Option B: Skip crates.io**
```bash
make release
# When prompted: "Publish to crates.io?" → n
```

**Option C: Manual publish later**
```bash
# After release, publish manually
cargo publish --all-features
```

### Published Crates

The following crates are published to crates.io:
- `gitflow-cli` — Main CLI application
- `gitflow-cli-core` — Core library
- `gitflow-cli-github` — GitHub platform support
- `gitflow-cli-gitlab` — GitLab platform support
- `gitflow-cli-gitcode` — GitCode platform support

Internal test crates (`e2e-core`, `e2e-github`) are **not** published.

### Verify Publication

```bash
# Search for the crate
cargo search gitflow-cli

# Check specific version
cargo search gitflow-cli --limit 1

# View on crates.io
open https://crates.io/crates/gitflow-cli

# View documentation
open https://docs.rs/gitflow-cli/0.8.0
```

### Troubleshooting

**Error: "user does not have permissions"**
```bash
cargo login
# Re-authenticate
```

**Error: "crate version already exists"**
- Version numbers are immutable on crates.io
- Must bump version and release again
- Or yank the existing version (not recommended)

**Error: "missing required fields"**
```bash
# Check Cargo.toml has all required fields
cargo package --list
```

## Manual Steps

The legacy `make release-push VERSION=patch|minor|major` is still available but marked as legacy.

## Troubleshooting

### Release Fails Mid-way

If the release fails during execution:

```bash
# Remove version bump commit
git reset --hard HEAD~1

# Remove local tag (if created)
git tag -d v0.8.0

# Remove remote tag (if pushed)
git push origin :v0.8.0

# Yank crates.io version (if published)
cargo yank --version 0.8.0
```

### Prerequisites

Required tools:
- `cargo` (Rust)
- `cargo-release` (`cargo install cargo-release`)
- `git-cliff` (`cargo install git-cliff`)

Install all with:
```bash
make install-tools
```

## Configuration

- `release.toml` — cargo-release configuration (publish to crates.io enabled)
- `cliff.toml` — git-cliff configuration

## CI/CD Integration

For automated releases in CI:

```bash
make release-quick
```

Or use the script directly:
```bash
bash scripts/release.sh --quick
```

### GitHub Actions Example

```yaml
name: Publish to crates.io

on:
  push:
    tags:
      - 'v*'

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Publish to crates.io
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
        run: cargo publish --token $CARGO_REGISTRY_TOKEN
```

## Important Notes

### Version Immutability

Once published to crates.io:
- **Cannot delete or modify** a published version
- Can only `yank` (hide from new installs)
- Must publish a new version for fixes

### Publish Order

cargo-release automatically handles dependency order:
1. `gitflow-cli-core` (no dependencies)
2. `gitflow-cli-github` (depends on core)
3. `gitflow-cli-gitlab` (depends on core)
4. `gitflow-cli-gitcode` (depends on core)
5. `gitflow-cli` (depends on all)

### License Compliance

Ensure:
- LICENSE file exists
- All dependencies have compatible licenses
- README doesn't contain local paths

### Documentation

After publishing:
- docs.rs will auto-generate documentation
- May take 10-30 minutes to appear
- Check: https://docs.rs/gitflow-cli/
