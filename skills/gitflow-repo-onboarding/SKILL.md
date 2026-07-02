---
name: gitflow-repo-onboarding
description: 仓库入门指引工作流 — 分析仓库结构和约定，生成面向新成员的入门指南，涵盖构建、测试、代码规范和项目约定
---

# gitflow-repo-onboarding

帮助新成员快速理解和上手一个仓库。通过自动分析仓库的项目结构、语言、框架、测试框架和 CI 配置，生成结构化的入门指南，涵盖如何构建项目、如何运行测试、项目约定和常用命令。

## 工作流

### 步骤 1：检测项目基础信息

分析仓库的基础特征，确定项目类型和技术栈：

```bash
# 获取远程仓库信息
git remote -v

# 获取默认分支
git remote show origin | grep 'HEAD branch'

# 查看仓库基本信息
gitflow-cli repo stats 2>/dev/null || echo "gitflow-cli repo 不可用，使用 git 命令"
```

**检测项目语言和框架：**

| 特征文件 | 语言/框架 | 构建工具 |
|----------|-----------|----------|
| `Cargo.toml` | Rust | `cargo` |
| `package.json` | Node.js / TypeScript | `npm` / `pnpm` / `yarn` |
| `pyproject.toml` / `setup.py` | Python | `pip` / `poetry` |
| `go.mod` | Go | `go` |
| `pom.xml` / `build.gradle` | Java / Kotlin | `mvn` / `gradle` |
| `Makefile` | 通用（辅助构建） | `make` |

### 步骤 2：分析项目结构

扫描仓库的目录结构，识别关键目录和模块：

```bash
# 查看顶层目录结构
ls -la

# 查看源码目录结构（按语言适配）
# Rust: src/
# Node.js: src/ 或 lib/
# Python: src/ 或 <package_name>/
# Go: cmd/ 和 pkg/

# 查看测试目录
# Rust: tests/ 或 src/ 中的 #[cfg(test)]
# Node.js: __tests__/ 或 *.test.ts
# Python: tests/
# Go: *_test.go

# 查看文档目录
ls docs/ 2>/dev/null
ls -la README* CONTRIBUTING* CHANGELOG* 2>/dev/null
```

**识别项目结构模式：**

| 模式 | 特征 | 说明 |
|------|------|------|
| 单 crate/package | 单个 `src/` 目录 | 简单项目 |
| Monorepo | `packages/`、`crates/`、`apps/` | 多模块工作区 |
| Workspace | `Cargo.toml` 中有 `[workspace]` | Rust 工作区 |

### 步骤 3：分析构建和测试方式

确定如何构建项目和运行测试：

**3.1 检查 Makefile 目标**

```bash
# 列出 Makefile 中的可用目标
make help 2>/dev/null || grep -E '^[a-zA-Z_-]+:' Makefile 2>/dev/null
```

常见的 Makefile 目标：

| 目标 | 用途 |
|------|------|
| `build` | 构建项目 |
| `test` | 运行测试 |
| `fmt` | 格式化代码 |
| `lint` / `clippy` | 静态分析 |
| `clean` | 清理构建产物 |
| `run` | 运行项目 |

**3.2 检查 CI 配置**

```bash
# GitHub Actions
ls .github/workflows/ 2>/dev/null

# GitLab CI
cat .gitlab-ci.yml 2>/dev/null

# 通用 CI 文件
ls .circleci/ .travis.yml 2>/dev/null
```

从 CI 配置中提取：

- CI 运行了哪些检查步骤
- 使用的工具链版本
- 特殊的环境变量或依赖

**3.3 确定构建和测试命令**

根据项目类型生成推荐命令：

**Rust 项目：**

| 操作 | 命令 |
|------|------|
| 构建 | `cargo build` |
| 运行测试 | `cargo test` |
| 格式化检查 | `cargo +nightly fmt -- --check` |
| 静态分析 | `cargo clippy -- -D warnings` |
| 安全审计 | `cargo audit` |

### 步骤 4：分析项目约定

从项目文件中提取开发约定和规范：

**4.1 代码规范**

```bash
# 检查 Rust lint 配置
grep -A 20 '\[workspace.lints\]' Cargo.toml 2>/dev/null

# 检查 rustfmt 配置
cat rustfmt.toml .rustfmt.toml 2>/dev/null

# 检查 clippy 配置
cat clippy.toml .clippy.toml 2>/dev/null

# 检查 editorconfig
cat .editorconfig 2>/dev/null
```

**4.2 Commit 规范**

```bash
# 检查 commitlint 配置
cat .commitlintrc* commitlint.config.* 2>/dev/null

# 查看最近的 commit 消息格式
git log --oneline -20
```

常见的 commit 规范：

- Conventional Commits（`feat:`, `fix:`, `docs:` 等）
- 关联 Issue 编号（`#123`）
- 签名要求（`-S` 标志）

**4.3 分支策略**

```bash
# 查看分支命名模式
git branch -r | head -20
```

常见的分支策略：

| 策略 | 分支命名 | 说明 |
|------|----------|------|
| GitHub Flow | `feature/*`, `fix/*` | 简单直接 |
| Git Flow | `feature/*`, `release/*`, `hotfix/*` | 复杂但完整 |
| Trunk Based | 直接提交 main | 高频集成 |

### 步骤 5：检查前置工具

确认新成员需要安装的工具和配置：

```bash
# Rust 项目
rustup show            # 查看 Rust 工具链版本
cat rust-toolchain.toml  # 查看项目要求的版本

# Node.js 项目
cat .nvmrc 2>/dev/null   # 查看 Node.js 版本
cat .node-version 2>/dev/null

# 通用
git config --get core.hooksPath  # 检查 git hooks 路径
```

**生成前置工具清单：**

```markdown
### 前置工具

- [ ] Rust 工具链（通过 `rust-toolchain.toml` 自动安装）
- [ ] `cargo-make` 或 `just`（如果使用）
- [ ] `gitflow` CLI（`cargo install gitflow-cli`）
- [ ] Git hooks（`make install-hooks` 或 `pre-commit install`）
- [ ] 编辑器插件（rust-analyzer、EditorConfig）
```

### 步骤 6：生成入门指南

汇总所有分析结果，生成结构化的入门指南：

```markdown
# 🚀 项目入门指南

## 项目概述

<项目名称> — <一句话描述>

**技术栈：** <语言> + <框架> + <构建工具>
**项目结构：** <单模块 / Monorepo / Workspace>

## 前置准备

1. 安装 <语言> 工具链（版本要求：<version>）
2. 安装 `gitflow` CLI：`cargo install gitflow-cli`
3. 克隆仓库：`git clone <repo-url>`
4. 安装 Git hooks：`make install-hooks`

## 快速开始

### 构建项目

```bash
<build-command>
```

### 运行测试

```bash
<test-command>
```

### 运行项目

```bash
<run-command>
```

## 项目结构

```
<project-root>/
├── src/           # 源代码
├── tests/         # 测试文件
├── docs/          # 文档
├── scripts/       # 脚本工具
├── Cargo.toml     # 依赖配置
└── Makefile       # 构建自动化
```

## 开发约定

### Commit 规范

<commit-convention>

### 分支策略

<branch-strategy>

### 代码规范

<code-style>

## 常用命令

| 操作 | 命令 |
|------|------|
| 构建 | `<build-command>` |
| 测试 | `<test-command>` |
| 格式化 | `<fmt-command>` |
| 静态分析 | `<lint-command>` |
| 运行 | `<run-command>` |

## CI 检查

提交 PR 后，CI 会自动运行以下检查：

- [ ] 构建通过
- [ ] 全部测试通过
- [ ] 代码格式检查通过
- [ ] 静态分析无警告

## 相关资源

- [项目文档](docs/)
- [贡献指南](CONTRIBUTING.md)
- [变更日志](CHANGELOG.md)
```

## 使用示例

### 为一个 Rust Workspace 项目生成入门指南

```bash
# 分析项目结构
ls -la
cat Cargo.toml
make help

# 检查 Rust 工具链
rustup show
cat rust-toolchain.toml

# 查看 CI 配置
ls .github/workflows/

# 查看最近的 commit 格式
git log --oneline -10

# 生成入门指南后输出到文件
cat > docs/ONBOARDING.md << 'EOF'
# 🚀 项目入门指南

## 项目概述

gitflow-cli — 基于 Rust 的 Git 工作流 CLI 工具

**技术栈：** Rust 2024 + Cargo Workspace
**项目结构：** Workspace（apps/ + crates/）

## 前置准备

1. 安装 Rust 工具链（版本见 `rust-toolchain.toml`）
2. 安装 gitflow CLI：`cargo install gitflow-cli`
3. 克隆仓库：`git clone https://github.com/org/gitflow-cli.git`

## 快速开始

### 构建项目

\`\`\`bash
make build
\`\`\`

### 运行测试

\`\`\`bash
make test
\`\`\`

## 常用命令

| 操作 | 命令 |
|------|------|
| 构建 | `make build` |
| 测试 | `make test` |
| 格式化 | `make fmt` |
| 静态分析 | `make clippy` |
EOF
```

### 为一个 Node.js 项目生成入门指南

```bash
# 检测项目类型
cat package.json | jq '{name, scripts, engines}'

# 查看目录结构
ls -la src/

# 查看 CI 配置
cat .github/workflows/ci.yml

# 确定命令
# 构建: npm run build
# 测试: npm test
# 格式化: npx prettier --check .
```

## 注意事项

- 入门指南应面向完全不了解项目的新成员，避免使用项目内部缩写或术语
- 构建和测试命令应优先使用 Makefile 目标，其次使用项目原生命令
- 如果发现项目缺少 README 或 CONTRIBUTING 文件，应在指南中建议补充
- 分析结果应基于实际文件内容，而非假设——如果某个文件不存在就跳过相关分析
- 入门指南生成后应保存为文件（如 `docs/ONBOARDING.md`），便于团队维护更新
- 对于多语言项目，应分别说明每种语言的构建和测试方式
- CI 检查部分应从实际 CI 配置文件中提取，而非凭空编造
- 定期更新入门指南，确保工具和命令与实际项目状态一致
