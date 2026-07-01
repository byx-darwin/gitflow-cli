# Pre-commit Usage

This template uses `pre-commit` to run fast repository checks before commits and heavier checks manually when needed.

## What it checks

The current `.pre-commit-config.yaml` configures:

- generic file hygiene: BOM, case conflicts, merge conflicts, broken symlinks, YAML syntax, EOF, line endings, trailing whitespace;
- Python formatting through `black`;
- Rust formatting through `cargo +nightly fmt -- --check`;
- dependency and policy checks through `cargo deny check -d`;
- typo checks through `typos`;
- Rust compilation checks through `cargo check --all`;
- Rust linting through `cargo clippy --all-targets --all-features --tests --benches -- -D warnings`;
- tests through `cargo nextest run --all-features -- --include-ignored`.

## Prerequisites

Install the tools used by the hooks before enabling them:

macOS with Homebrew:

```bash
brew install pre-commit
cargo install cargo-nextest cargo-deny typos-cli
```

Alternative Python-based install:

```bash
pipx install pre-commit
# or: python3 -m pip install --user pre-commit
```

## Setup

Install the configured git hooks:

```bash
pre-commit install --install-hooks
```

## Daily commands

Run hooks for changed files:

```bash
pre-commit run
```

Run all hooks against the whole repository:

```bash
pre-commit run --all-files
```

Run one hook only:

```bash
pre-commit run cargo-clippy --all-files
```

Run hooks configured for manual stages:

```bash
pre-commit run --hook-stage manual --all-files
```

Update hook revisions intentionally:

```bash
pre-commit autoupdate
```

## Notes for this template

- Prefer Makefile targets for normal development checks: `make fmt`, `make clippy`, `make lint`, and `make test`.
- Use `pre-commit` as the local safety net before commits.
- Do not bypass hooks unless the user explicitly asks and the reason is documented.
- Preserve template placeholders such as `gitflow-cli`.

---

## 中文版本

本模板使用 `pre-commit` 在提交前运行仓库检查；较重的检查可以按需手动执行。

## 检查内容

当前 `.pre-commit-config.yaml` 配置了：

- 通用文件卫生检查：BOM、大小写冲突、merge conflict、损坏的 symlink、YAML 语法、EOF、换行符、行尾空格；
- 使用 `black` 格式化 Python 文件；
- 使用 `cargo +nightly fmt -- --check` 检查 Rust 格式；
- 使用 `cargo deny check -d` 检查依赖和策略；
- 使用 `typos` 检查拼写；
- 使用 `cargo check --all` 检查 Rust 编译错误；
- 使用 `cargo clippy --all-targets --all-features --tests --benches -- -D warnings` 做 Rust lint；
- 使用 `cargo nextest run --all-features -- --include-ignored` 运行测试。

## 前置工具

启用 hooks 前需要安装对应工具：

macOS + Homebrew：

```bash
brew install pre-commit
cargo install cargo-nextest cargo-deny typos-cli
```

Python 安装方式备选：

```bash
pipx install pre-commit
# 或者：python3 -m pip install --user pre-commit
```

## 初始化

安装 git hooks：

```bash
pre-commit install --install-hooks
```

## 日常命令

只检查变更文件：

```bash
pre-commit run
```

检查整个仓库：

```bash
pre-commit run --all-files
```

只运行某一个 hook：

```bash
pre-commit run cargo-clippy --all-files
```

运行 manual stage 的 hooks：

```bash
pre-commit run --hook-stage manual --all-files
```

有意升级 hook 版本：

```bash
pre-commit autoupdate
```

## 模板仓库注意事项

- 日常开发优先使用 Makefile targets：`make fmt`、`make clippy`、`make lint`、`make test`。
- `pre-commit` 作为提交前本地安全网。
- 除非用户明确要求且记录原因，不要绕过 hooks。
- 保留 `gitflow-cli` 等模板占位符。
