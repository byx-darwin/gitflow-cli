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
- `gf` — Main CLI application
- `gitflow-core` — Core library
- `gitflow-github` — GitHub platform support
- `gitflow-gitlab` — GitLab platform support
- `gitflow-gitcode` — GitCode platform support

Internal test crates (`e2e-core`, `e2e-github`) are **not** published.

### Verify Publication

```bash
# Search for the crate
cargo search gf

# Check specific version
cargo search gf --limit 1

# View on crates.io
open https://crates.io/crates/gf

# View documentation
open https://docs.rs/gf/0.8.0
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
1. `gitflow-core` (no dependencies)
2. `gitflow-github` (depends on core)
3. `gitflow-gitlab` (depends on core)
4. `gitflow-gitcode` (depends on core)
5. `gf` (depends on all)

### License Compliance

Ensure:
- LICENSE file exists
- All dependencies have compatible licenses
- README doesn't contain local paths

### Documentation

After publishing:
- docs.rs will auto-generate documentation
- May take 10-30 minutes to appear
- Check: https://docs.rs/gf/

## 事故复盘:`v{{version}}` 模板未替换

**现象**:历史提交 `9331bfa`/`0b0e9d7` 的提交主题字面为 `chore: release v{{version}}`。

**根因**:当时使用的 cargo-release 旧版本**未替换** `release.toml` 里已配置的
`{{version}}` 模板,导致模板字面量被原样写入提交主题与 tag 消息。

**修复**:`release.toml` 模板回滚为双括号 `{{version}}`,与当前已安装的
cargo-release **1.1.3** 的模板替换约定一致(Issue #96)。

**防复发闸门**(`scripts/release.sh`):

| 闸门 | 位置 | 失败动作 |
|------|------|----------|
| dry-run 输出校验 | `cargo release --dry-run` 后 | 扫描 `{version}` / `{{version}}` 残留 + 强校验退出码;命中即中止 |
| 提交主题校验 | `cargo release commit` 后 | `git reset --hard HEAD~1` + 中止 |
| CHANGELOG 残留校验 | `git cliff` 后 | 中止 |
| tag 名校验 | `cargo release tag` 后、push 前 | `git tag -d` + 中止 |

校验器为纯函数,`bash scripts/release.sh --self-test` 可随时自测。
dry-run 同时检测 `{version}` 与 `{{version}}` 两种残留,未来无论 cargo-release
向哪一方向漂移模板语法,演练都会失败并阻断发布。

**受验证的 cargo-release 版本**:1.1.3(双括号 `{{version}}` 语法)。

**强制 dry-run**:`--quick` 不再跳过 dry-run,仅跳过交互确认。

## 发布演练(1.0 发布前必做)

```bash
make release-rehearse
```

完整 dry 链路:前置检查 → main/干净工作区 → 测试 → clippy → 版本预览 →
`cargo release --dry-run`(输出级 `{version}`/`{{version}}` 残留扫描) → 校验器自检。
输出 ✅ 清单报告;任一失败退出码非 0;绝不产生变更。演练输出摘录应附在对应发布 Issue 中作为证据。

**受验证的 cargo-release 版本**:1.1.3(双括号 `{{version}}` 语法)。
