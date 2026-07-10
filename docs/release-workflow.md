# Release Workflow Guide

## Overview

The release workflow has been improved with safety checks, version inference, and interactive previews.

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
- Tag creation
- Push to remote

### Execution
If confirmed, the script will:
1. Bump version in `Cargo.toml`
2. Commit version change
3. Generate `CHANGELOG.md`
4. Commit changelog
5. Create Git tag
6. Push to `origin/main` with tags

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

- `release.toml` — cargo-release configuration
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
