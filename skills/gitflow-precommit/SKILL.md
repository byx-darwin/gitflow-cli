---
name: gitflow-precommit
description: Pre-commit 检查工作流 — 在提交前运行格式化、静态分析和测试，确保代码质量达标，并可选配置 Git pre-commit hook
---

# gitflow-precommit

在每次 `git commit` 前自动运行代码质量检查，确保提交的代码满足格式化、静态分析和测试要求。支持解析项目配置（`Cargo.toml`、`.pre-commit-config.yaml`），运行 `cargo fmt -- --check`、`cargo clippy`、`cargo test` 三项核心检查，并可配置 `.git/hooks/pre-commit` 实现自动化门禁。

## 工作流

### 步骤 1：检测项目配置

解析项目配置文件，确定适用的检查规则：

**1.1 解析 Cargo.toml（Rust 项目）**

```bash
# 检查是否为 Rust 项目
if [ -f "Cargo.toml" ]; then
    echo "检测到 Rust 项目"
fi

# 查看 workspace lint 配置
grep -A 20 '\[workspace.lints\]' Cargo.toml 2>/dev/null

# 查看 rustfmt 配置
cat rustfmt.toml .rustfmt.toml 2>/dev/null

# 查看 clippy 配置
cat clippy.toml .clippy.toml 2>/dev/null
```

**1.2 解析 .pre-commit-config.yaml**

```bash
# 检查是否使用 pre-commit 框架
if [ -f ".pre-commit-config.yaml" ]; then
    echo "检测到 pre-commit 配置"
    cat .pre-commit-config.yaml
fi
```

从 `.pre-commit-config.yaml` 提取：

- 已配置的 hook 列表
- 各 hook 对应的仓库和版本
- 排除的文件模式（`exclude`）

**1.3 确定检查命令集**

根据项目类型确定检查命令：

**Rust 项目默认检查集：**

| 检查项 | 命令 | 说明 |
|--------|------|------|
| 格式化 | `cargo fmt -- --check` | 代码格式检查 |
| 静态分析 | `cargo clippy --all-targets --all-features -- -D warnings` | Clippy lint 检查 |
| 单元测试 | `cargo test --workspace` | 运行所有测试 |

### 步骤 2：运行格式化检查

```bash
cargo fmt -- --check 2>&1
```

- **通过**：退出码 0 → 记录 `✅ fmt`
- **失败**：退出码非 0 → 记录 `❌ fmt` + 列出需要格式化的文件

**失败处理：**

```bash
# 自动修复格式问题
cargo fmt

# 重新检查确认修复
cargo fmt -- --check
```

### 步骤 3：运行静态分析

```bash
cargo clippy --all-targets --all-features -- -D warnings 2>&1
```

对于更严格的检查（pedantic 级别）：

```bash
cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic 2>&1
```

- **通过**：退出码 0，无 warning → 记录 `✅ clippy`
- **失败**：有 warning 或 error → 记录 `❌ clippy` + 警告摘要

**失败处理：**

```bash
# 自动修复部分 lint 问题
cargo clippy --fix --all-targets --all-features --allow-dirty

# 手动检查剩余问题
cargo clippy --all-targets --all-features -- -D warnings
```

### 步骤 4：运行测试

```bash
cargo test --workspace 2>&1
```

- **通过**：全部测试通过 → 记录 `✅ test`
- **失败**：有测试失败 → 记录 `❌ test` + 失败的测试列表

**失败处理：**

```bash
# 查看失败测试的详细信息
cargo test --workspace -- --nocapture

# 只运行特定测试
cargo test -p <crate-name> <test-name> -- --nocapture
```

### 步骤 5：汇总检查结果

生成 Pre-commit 检查报告：

```markdown
## Pre-commit 检查报告

| # | 检查项 | 状态 | 详情 |
|---|--------|------|------|
| 1 | fmt | ✅/❌ | <详情> |
| 2 | clippy | ✅/❌ | <详情> |
| 3 | test | ✅/❌ | <详情> |

**结论：** ✅ 全部通过，可以提交 / ❌ 存在问题，请先修复
```

### 步骤 6：配置 Git pre-commit hook（可选）

将检查配置为 Git hook，实现每次提交自动运行：

```bash
# 创建 hooks 目录
mkdir -p .git/hooks

# 写入 pre-commit hook
cat > .git/hooks/pre-commit << 'HOOK'
#!/usr/bin/env bash
set -euo pipefail

echo "🔍 运行 pre-commit 检查..."

# 步骤 1：格式化检查
echo "📝 检查代码格式..."
if ! cargo fmt -- --check 2>&1; then
    echo "❌ 格式化检查失败"
    echo "💡 运行 'cargo fmt' 自动修复"
    exit 1
fi
echo "✅ 格式化检查通过"

# 步骤 2：静态分析
echo "🔧 运行 clippy..."
if ! cargo clippy --all-targets --all-features -- -D warnings 2>&1; then
    echo "❌ Clippy 检查失败"
    echo "💡 运行 'cargo clippy --fix --all-targets --all-features --allow-dirty' 自动修复"
    exit 1
fi
echo "✅ Clippy 检查通过"

# 步骤 3：测试
echo "🧪 运行测试..."
if ! cargo test --workspace 2>&1; then
    echo "❌ 测试失败"
    exit 1
fi
echo "✅ 测试通过"

echo ""
echo "✅ 所有 pre-commit 检查通过，继续提交..."
HOOK

# 赋予执行权限
chmod +x .git/hooks/pre-commit
```

**使用 pre-commit 框架的替代方案：**

```bash
# 安装 pre-commit
pip install pre-commit

# 在 .pre-commit-config.yaml 中配置
cat > .pre-commit-config.yaml << 'EOF'
repos:
  - repo: local
    hooks:
      - id: cargo-fmt
        name: cargo fmt
        entry: cargo fmt -- --check
        language: system
        types: [rust]
        pass_filenames: false
      - id: cargo-clippy
        name: cargo clippy
        entry: cargo clippy --all-targets --all-features -- -D warnings
        language: system
        types: [rust]
        pass_filenames: false
      - id: cargo-test
        name: cargo test
        entry: cargo test --workspace
        language: system
        types: [rust]
        pass_filenames: false
EOF

# 安装 hooks
pre-commit install
```

### 步骤 7：输出结果

根据检查结果输出最终结论：

**全部通过：**

```
✅ Pre-commit 检查全部通过
   fmt: ✅ | clippy: ✅ | test: ✅
   可以安全提交
```

**存在失败：**

```
❌ Pre-commit 检查未通过
   fmt: ❌ | clippy: ✅ | test: ✅

   请先修复以下问题：
   1. 运行 `cargo fmt` 修复格式问题
   2. 重新提交
```

## 使用示例

### 手动运行 pre-commit 检查

```bash
# 运行完整检查
cargo fmt -- --check && \
cargo clippy --all-targets --all-features -- -D warnings && \
cargo test --workspace

# 如果全部通过，提交代码
git add -A
git commit -m "feat: add new feature"
```

### 配置 Git hook 自动检查

```bash
# 使用脚本配置 hook
cat > .git/hooks/pre-commit << 'HOOK'
#!/usr/bin/env bash
set -euo pipefail
echo "🔍 运行 pre-commit 检查..."
cargo fmt -- --check || { echo "❌ 格式化失败"; exit 1; }
cargo clippy --all-targets --all-features -- -D warnings || { echo "❌ Clippy 失败"; exit 1; }
cargo test --workspace || { echo "❌ 测试失败"; exit 1; }
echo "✅ 全部通过"
HOOK
chmod +x .git/hooks/pre-commit

# 之后每次 git commit 都会自动运行检查
```

### 跳过 pre-commit 检查（紧急情况）

```bash
# 紧急提交时跳过 hook
git commit --no-verify -m "hotfix: urgent fix"
```

### 只检查变更涉及的文件

```bash
# 获取变更的 Rust 文件
CHANGED_FILES=$(git diff --name-only --cached -- '*.rs')

if [ -n "$CHANGED_FILES" ]; then
    echo "变更的 Rust 文件："
    echo "$CHANGED_FILES"

    # 格式检查（只检查变更文件）
    cargo fmt -- --check

    # 运行完整测试（无法只运行部分）
    cargo test --workspace
fi
```

## 注意事项

- pre-commit 检查应在 `git add` 之后、`git commit` 之前运行
- 格式化检查 `cargo fmt -- --check` 不会修改文件，只检查格式是否符合规范
- Clippy 的 `--fix` 功能可以自动修复部分问题，但需要 `--allow-dirty` 标志（在有未提交变更时）
- 测试失败时应先看完整的失败输出，确定是测试本身的问题还是代码的问题
- Git hook 是本地行为，不会随仓库共享——团队成员需要各自配置
- 如果项目使用 `pre-commit` 框架，优先使用框架管理 hooks，保持一致性
- 对于 CI 环境和 Git hook 使用相同的检查命令，确保本地和 CI 行为一致
- 紧急情况下可使用 `git commit --no-verify` 跳过 hook，但事后应尽快修复问题
