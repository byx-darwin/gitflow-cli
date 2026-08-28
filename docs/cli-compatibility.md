# CLI 版本兼容矩阵

gf 依赖三个外部 CLI 工具执行平台操作。本文档记录各 CLI 的最低版本要求、
已知功能限制和版本变更影响。

> **维护规则**：升级最低版本时，同步更新 `prerequisites.rs` 中的 `min_version`
> 和本文档的矩阵表。

---

## 兼容矩阵

| 平台 | CLI 二进制 | 最低版本 | 当前推荐 | 版本检查命令 |
|------|-----------|---------|---------|-------------|
| GitHub | `gh` | 2.0.0 | 2.50.0+ | `gh --version` |
| GitLab | `glab` | 1.30.0 | 1.113.0+ | `glab --version` |
| GitCode | `gc` / `gitcode` | 0.6.0 | 0.6.1+ | `gc --version` 或 `gitcode --version` |

`gf doctor` 会自动检测已安装 CLI 的版本并与最低要求对比。

---

## GitHub — `gh`

### 功能依赖

| gh 版本 | 新增 gf 依赖的功能 |
|---------|------------------|
| 2.0.0 | `gh api`、`gh pr create/list/view/merge/close`、`gh issue create/list/view`、`gh release create/list/view` |
| 2.12.0 | `gh run list/view` — CI/CD 流水线查询 |
| 2.17.0 | `gh label create/list/edit/delete` — 标签管理 |
| 2.20.0 | `gh pr convert-to-draft`、`gh pr ready` — PR 草稿状态切换 |

### 已知限制

| gh 版本 | 限制 | gf 应对方式 |
|---------|------|-----------|
| < 2.12 | `gh run` 命令不可用 | Pipeline 功能返回不支持错误 |
| < 2.17 | `gh label` 命令不可用 | 回退到 `gh api` REST 调用 |
| ≤ 2.97 | 无 `gh label view` 子命令 | 使用 `gh api repos/{repo}/labels/{name}` 替代 |

### JSON 输出格式

gf 大量使用 `--json <fields>` 解析输出。`gh` 的 JSON 字段名在主要版本间稳定，
但 minor 版本可能新增字段。gf 仅读取已知字段名，新增字段不影响解析。

---

## GitLab — `glab`

### 功能依赖

| glab 版本 | 新增 gf 依赖的功能 |
|----------|------------------|
| 1.30.0 | `glab mr create/list/view/merge/close`、`glab issue create/list/view`、`glab release create/list/view`、`glab ci list/trace` |
| 1.35.0 | `glab mr rebase` — MR 变基操作 |
| 1.40.0 | `glab auth status --show-token` — Token 查看 |

### 已知限制

| glab 版本 | 限制 | gf 应对方式 |
|----------|------|-----------|
| < 1.113 | 无 `glab release edit` 子命令 | 通过 `glab release create` 覆盖实现 |
| < 1.113 | `glab release create` 不支持 `--draft` / `--prerelease` | 回退到 GitLab API 调用 |
| 所有版本 | `glab mr diff` 不支持 `--repo` 参数 | gf 在调用前切换仓库上下文 |

### JSON 输出格式

glab 使用 `--output json`（而非 `--json`）。gf 已适配两种格式差异。

---

## GitCode — `gc` / `gitcode`

### 功能依赖

| gc 版本 | 新增 gf 依赖的功能 |
|--------|------------------|
| 0.6.0 | `issue create/list/view/close/reopen`、`pr create/list/view/merge/close/checkout`、`release create/list/view/edit/delete`、`label create/list/edit/delete/view`、`milestone create/list/edit/close/reopen` |
| 0.6.1 | `commit diff/patch`、`mr diff/patch`、`pr review`、`auth token` |

### 已知限制

| gc 版本 | 限制 | gf 应对方式 |
|--------|------|-----------|
| ≤ 0.6.1 | 无 `run` / `pipeline` 子命令 | Pipeline 功能返回 "GitCode CLI v0.6.1 does not have 'run' command" |
| 所有版本 | 二进制名称为 `gc`（Linux/macOS）或 `gitcode`（跨平台） | gf 按 `gc` → `gitcode` 顺序搜索 |

### 安装方式

GitCode CLI 通过 pip 分发，支持 Python 3.7+：

```bash
pip install gitcode-cli
```

---

## 版本升级检查清单

当外部 CLI 发布新版本时，按以下步骤验证兼容性：

1. **阅读 changelog** — 检查 JSON 输出格式、子命令名称、flag 是否有 breaking change
2. **本地测试** — 安装新版本后运行 `gf doctor` 确认版本识别正常
3. **功能验证** — 对受影响的子命令逐一测试（优先测试 JSON 解析路径）
4. **更新矩阵** — 若发现新的最低版本要求或功能限制，更新本文档和 `prerequisites.rs`
5. **CI 矩阵** — 若 CI 安装了特定版本，确认版本范围覆盖

---

## 相关代码位置

| 文件 | 职责 |
|------|------|
| `apps/cli/src/commands/prerequisites.rs` | 版本要求定义 + 前置检查 |
| `apps/cli/src/commands/doctor.rs` | 诊断报告（版本 vs 最低要求） |
| `crates/core/src/doctor.rs` | 诊断类型定义 |
| `crates/{github,gitlab,gitcode}/src/runner.rs` | CLI 调用执行层 |
