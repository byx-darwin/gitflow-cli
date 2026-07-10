# Crates.io 发布方案

## 当前状态

- ✅ Workspace 配置完整（version, authors, license, repository 等）
- ✅ CLI crate 有 description, categories, keywords
- ❌ `release.toml` 设置 `publish = false`
- ❌ 缺少 crates.io 认证配置
- ❌ 部分 crate 缺少 `publish` 配置

## 需要发布的 Crates

### 核心库（发布）
1. `gitflow-cli-core` — 核心功能库
2. `gitflow-cli-github` — GitHub 平台支持
3. `gitflow-cli-gitlab` — GitLab 平台支持
4. `gitflow-cli-gitcode` — GitCode 平台支持
5. `gitflow-cli` — CLI 应用程序

### 测试/内部 crate（不发布）
- `e2e-core` — 端到端测试核心
- `e2e-github` — GitHub 端到端测试

## 实施步骤

### Step 1: 配置 Crate Metadata

确保所有要发布的 crate 有以下字段：

```toml
[package]
name = "gitflow-cli"
version.workspace = true
authors.workspace = true
edition.workspace = true
license.workspace = true
description = "Cross-platform Git engineering workflow orchestration tool"
repository.workspace = true
homepage.workspace = true
documentation.workspace = true
readme = "README.md"
categories = ["development-tools", "command-line-utilities"]
keywords = ["gitflow", "git", "cli", "workflow"]

# 明确指定发布到 crates.io
publish = ["crates-io"]
```

### Step 2: 更新 release.toml

```toml
# cargo-release workspace configuration

# Single version tag for the entire workspace
shared-version = true
tag-name = "v{{version}}"
tag-message = "Release v{{version}}"

# Publish to crates.io
publish = true
registry = "crates-io"

# Allow release only from main branch
allow-branch = ["main"]

# Pre-release commit message
pre-release-commit-message = "chore: release v{{version}}"

# Pre-release hooks
pre-release-hook = ["cargo", "fmt", "--all"]
pre-release-hook = ["cargo", "clippy", "--all-targets", "--all-features", "--", "-D", "warnings"]

# Verify before publishing
verify = true
```

### Step 3: Crates.io 认证

用户需要：

1. 登录 crates.io：
```bash
cargo login
```

2. 或使用 API token：
```bash
export CARGO_REGISTRY_TOKEN=<your-token>
```

### Step 4: 更新 Release 脚本

修改 `scripts/release.sh` 以支持 crates.io 发布：

```bash
# 在 execute_release() 中添加
if confirm "Publish to crates.io?"; then
    log_info "Publishing to crates.io..."
    cargo release publish --execute --no-confirm
fi
```

### Step 5: 添加 Pre-release 检查

发布到 crates.io 前需要验证：

```bash
# 检查 crate metadata
cargo package --list
cargo package --no-verify

# 检查是否有未提交的更改
git status --porcelain

# 检查版本号是否已存在
cargo search gitflow-cli --limit 1
```

## 发布流程

### 方式 A: 全自动发布

```bash
make release
# 选择版本 → 确认 changelog → 自动发布到 crates.io
```

### 方式 B: 分步发布

```bash
# Step 1: 准备发布（不执行）
cargo release --dry-run

# Step 2: 只发布到 crates.io
cargo release publish --execute

# Step 3: 创建 tag 和 GitHub Release
cargo release tag --execute
git push origin main --tags
```

## 注意事项

### 1. Crate 发布顺序

依赖关系决定发布顺序：
1. `gitflow-cli-core` (无依赖)
2. `gitflow-cli-github` (依赖 core)
3. `gitflow-cli-gitlab` (依赖 core)
4. `gitflow-cli-gitcode` (依赖 core)
5. `gitflow-cli` (依赖所有)

cargo-release 会自动处理依赖顺序。

### 2. 版本锁定

一旦发布到 crates.io，版本号**不可更改**。如需修改，必须发布新版本。

### 3. README 和文档

- 确保 README.md 不包含本地路径
- 确保所有文档链接有效
- 确保 examples 可运行

### 4. 许可证

- 确保 LICENSE 文件存在
- 确保所有依赖的许可证兼容

## 测试发布

在正式发布前，可以测试：

```bash
# Dry run
cargo release --dry-run

# 只打包不发布
cargo package

# 发布到测试 registry（如果有）
cargo publish --registry=test-registry
```

## 回滚计划

如果发布失败：

1. **发布过程中失败**：
   - 部分 crate 可能已发布
   - 无法删除已发布的版本
   - 需要发布修复版本（yank 或新版本）

2. **Yank 版本**：
```bash
cargo yank --version 0.8.0
```

3. **发布修复版本**：
```bash
cargo release patch --execute
```

## 监控和验证

发布后验证：

```bash
# 检查 crates.io
cargo search gitflow-cli

# 检查文档
open https://docs.rs/gitflow-cli/0.8.0

# 测试安装
cargo install gitflow-cli --version 0.8.0
```

## CI/CD 集成

可选：在 GitHub Actions 中自动发布

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

## 下一步行动

- [ ] 更新所有 crate 的 Cargo.toml metadata
- [ ] 更新 release.toml 配置
- [ ] 添加 crates.io 认证说明到文档
- [ ] 更新 release.sh 脚本支持 crates.io 发布
- [ ] 测试 dry-run 发布
- [ ] 执行首次发布
- [ ] 验证 docs.rs 文档生成
