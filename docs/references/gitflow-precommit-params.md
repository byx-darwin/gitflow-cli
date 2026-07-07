# gitflow-cli pre-commit 完整参考

> 本文档为 `gitflow-precommit` skill 的参数与模板外部化引用。

## 参数速查

| 参数 | 类型 | 默认 | 说明 |
|------|------|------|------|
| `--check-only` | flag | false | 仅检查不修复 |
| `--allow-dirty` | flag | false | 允许在未提交变更时运行 |
| `--skip-fmt` | flag | false | 跳过格式化检查 |
| `--skip-clippy` | flag | false | 跳过 clippy 检查 |
| `--skip-test` | flag | false | 跳过测试 |

## 项目配置文件清单

| 文件 | 用途 |
|------|------|
| `Cargo.toml` | workspace lint 配置、`rust-toolchain` |
| `.pre-commit-config.yaml` | pre-commit 框架 hook 配置 |
| `rustfmt.toml` / `.rustfmt.toml` | 格式化选项 |
| `clippy.toml` / `.clippy.toml` | clippy lint 配置 |

## Git hook 模板（直接执行版）

```bash
cat > .git/hooks/pre-commit << 'HOOK'
#!/usr/bin/env bash
set -euo pipefail
echo "🔍 Running pre-commit checks..."
cargo fmt -- --check || { echo "❌ 格式失败 → 运行 cargo fmt"; exit 1; }
cargo clippy --all-targets --all-features -- -D warnings || { echo "❌ Clippy 失败"; exit 1; }
cargo test --workspace || { echo "❌ 测试失败"; exit 1; }
echo "✅ 全部通过"
HOOK
chmod +x .git/hooks/pre-commit
```

## pre-commit 框架配置模板

```yaml
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
```
