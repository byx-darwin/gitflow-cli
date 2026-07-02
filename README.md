# Rust Library Template

[![CI](https://github.com/byx-darwin/rust-lib-template/actions/workflows/build.yml/badge.svg)](https://github.com/byx-darwin/rust-lib-template/actions/workflows/build.yml)

一个现代化的 Rust CLI 项目模板，包含完整的开发工具链和最佳实践。

## 安装

### Homebrew (macOS)

```bash
brew tap byx-darwin/gitflow-cli
brew install gitflow-cli
```

### Cargo

```bash
cargo install gitflow-cli
```

### 源码编译

```bash
git clone https://github.com/byx-darwin/gitflow-cli
cd gitflow-cli
make build
```

## 快速开始

### 1. 生成新项目

```bash
cargo generate --git https://github.com/byx-darwin/rust-lib-template
```

按提示输入项目名称（如 `gitflow-cli`）、作者信息和项目描述。

### 2. 进入项目目录

```bash
cd gitflow-cli
```

### 3. 安装开发工具

```bash
make install-tools
```

这会安装 `cargo-edit`, `cargo-watch`, `cargo-audit` 等常用工具。

### 4. 构建和测试

```bash
# 构建项目
make build

# 运行测试
make test

# 代码检查和格式化
make lint
```

### 5. 运行项目

```bash
# 查看帮助
cargo run -- --help

# 运行主命令
cargo run -- run

# 生成 shell 自动补全脚本
cargo run -- completions bash > ~/.bash_completion.d/gitflow-cli
```

## 项目结构

```
.
├── apps/
│   └── cli/          # CLI 应用入口
├── crates/
│   └── core/         # 核心业务逻辑库
├── docs/             # 项目文档
├── specs/            # 功能规格说明
├── Makefile          # 常用命令集合
└── Cargo.toml        # 工作区配置
```

## 开发工作流

```bash
# 监听文件变化自动测试
make test-watch

# 运行完整检查（构建 + 测试 + lint）
make check

# 生成 API 文档
make doc
```

## 配置

生成的项目支持从多个位置读取配置：

1. 内置默认值
2. 配置文件：`$XDG_CONFIG_HOME/<project-name>/config.toml`
3. 环境变量：`<PROJECT_NAME>_` 前缀
4. 命令行参数（最高优先级）

## 特性

- ✅ Rust 2024 Edition
- ✅ 严格的 lint 配置（clippy pedantic）
- ✅ 完整的测试框架（rstest, proptest）
- ✅ 自动 CI/CD（GitHub Actions）
- ✅ 依赖安全检查（cargo-audit, cargo-deny）
- ✅ Shell 自动补全生成
- ✅ 配置文件支持
- ✅ 结构化日志（tracing）
- ✅ 错误处理最佳实践（thiserror, miette）

## 许可证

MIT — 详见 [LICENSE.md](LICENSE.md)。
