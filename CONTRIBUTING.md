# Contributing to gitflow-cli

感谢您对 gitflow-cli 的关注！本项目是一个多平台 Git 锻造 CLI 工具，统一 GitHub、GitLab 和 GitCode 的操作接口。

## 开发环境搭建

### 必需工具

- **Rust 工具链**：参见 `rust-toolchain.toml`（当前要求 1.96.0）
- **原生 CLI**（按需安装）：
  - GitHub 平台：`gh` CLI（v2.0.0+）
  - GitLab 平台：`glab` CLI（v1.30.0+）
  - GitCode 平台：`gitcode` CLI（v0.6.0+）

### 安装开发工具

```bash
make install-tools
```

这会安装：
- `cargo-deny` — 许可证和依赖审计
- `cargo-audit` — 安全漏洞扫描
- `cargo-nextest` — 更快的测试运行器
- `cargo-vet` — 供应链审查
- `pre-commit` — Git pre-commit 钩子

### IDE 配置

推荐使用 VS Code + `rust-analyzer` 插件。项目根目录的 `rust-toolchain.toml` 会自动选择正确的工具链版本。

## 代码规范

项目遵循严格的 Rust 2024 规范，详见 `CLAUDE.md`。核心规则：

- **禁止 `unwrap()` / `expect()`**：所有错误必须通过 `Result<T>` 传播
- **公共 API 必须文档化**：所有 `pub` 项需包含文档注释
- **TDD（测试驱动开发）**：先写测试 → 确认失败 → 写实现 → 重构
- **Conventional Commits**：`feat:` / `fix:` / `docs:` / `chore:` / `test:` / `refactor:`
- **`#![forbid(unsafe_code)]`**：所有 crate 禁止 unsafe 代码

## TDD 开发循环

```
RED   → 编写一个描述预期行为的失败测试
GREEN → 编写最简代码使测试通过
REFACTOR → 清理代码，保持测试绿色
```

```bash
# 开发时使用 watch 模式
make test-watch

# 提交前运行完整检查
make lint
make test
```

## 项目结构

```
gitflow-cli/
├── apps/cli/          # CLI 二进制（clap + miette）
├── crates/
│   ├── core/          # 领域类型 + trait 抽象
│   ├── github/        # GitHub provider（调用 gh CLI）
│   ├── gitlab/        # GitLab provider（调用 glab CLI）
│   └── gitcode/       # GitCode provider（调用 gitcode CLI）
├── skills/            # Superpowers Skills（26 个）
├── specs/             # 设计规格文档
├── docs/              # 项目文档
└── scripts/           # 辅助脚本
```

## PR 提交流程

1. **创建 Issue**：在 GitHub Issues 中描述需求或 Bug
2. **创建分支**：`git checkout -b feat/my-feature`
3. **开发**：遵循 TDD 循环，保持提交原子化
4. **自检**：运行 `make lint && make test`
5. **推送**：`git push origin feat/my-feature`
6. **创建 PR**：使用 `gitflow pr create` 或 GitHub Web UI
7. **代码审查**：等待审查，根据反馈修改
8. **合并**：审查通过后合并到 `main`

## Issue 标签说明

| 标签 | 用途 |
|------|------|
| `bug` | 缺陷报告 |
| `enhancement` | 功能增强 |
| `docs` | 文档相关 |
| `phase-1` ~ `phase-5` | 开发阶段 |
| `triage:done` | 已分类 |

## Release 流程

发布由维护者通过 `make release` 自动完成，包括：
1. `cargo release tag` — 打版本标签
2. `git cliff` — 生成 CHANGELOG
3. GitHub Release workflow — 自动构建并上传二进制

## 获取帮助

- 查阅 `CLAUDE.md` 了解完整的代码规范
- 查阅 `specs/` 了解设计决策
- 使用 `gitflow issue create` 提交问题
