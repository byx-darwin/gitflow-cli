# gf update + skills 版本管理 — 设计文档

- **日期**: 2026-08-08
- **状态**: 已确认
- **工作流**: wf-20260808-230318 (full mode)

## 概述

为 `gf` CLI 添加两个能力：

1. **`gf update`** — 从 crates.io 自更新二进制
2. **`gf skills update`** — 版本管理的 skills 更新子命令

Skills 版本与 gf binary 绑定（内嵌于 binary），更新 skills 即从新 binary 重新覆盖安装。

## 需求决策

| 决策点 | 选择 |
|--------|------|
| 更新来源 | GitHub Releases（self_update github 后端，下载预编译 binary） |
| Skills 版本模型 | 与 gf 版本绑定（内嵌于 binary） |
| 更新流程 | `gf update` 更新 binary 后提示是否同步更新 skills |
| 版本检查 | 仅手动触发，无自动检查 |
| 预发布版本 | 默认仅稳定版，`--pre` 参数包含预发布 |

> **2026-08-08 修订**：`self_update` 0.42 无 crates.io 后端，且 crates.io 仅有源码包。
> 更新来源改为 GitHub Releases（与 `[package.metadata.binstall]` 配置一致，
> `{ repo }/releases/download/v{ version }/{ name }-{ target }.tgz`）。

## CLI 接口

### `gf update`

```
gf update [OPTIONS]

Options:
  --pre          包含预发布版本（alpha/beta/rc），默认仅稳定版
  --check        仅检查是否有新版本，不执行更新
  -y, --yes      跳过确认提示，直接更新
```

### `gf skills update`

```
gf skills update [OPTIONS]

Options:
  -g, --global   更新全局 skills（默认项目级）
  --agent <AGENT> 目标 Agent 平台（默认 claude）
  --path <PATH>  自定义路径
```

行为：等价于 `gf skills install --force`，从当前 binary 内嵌数据覆盖安装所有 `gf-*` skills。

## 模块结构

```
apps/cli/src/commands/
├── update.rs          # 新增：gf update 命令实现
├── skills.rs          # 修改：新增 Update 变体 + update_skills 函数
└── main.rs            # 修改：注册 Update 命令
```

### `update.rs` — `gf update` 核心逻辑

```rust
pub fn handle_update(args: &UpdateArgs) -> miette::Result<()> {
    // 1. 获取 GitHub Releases 最新版本
    let latest = get_latest_release(args.pre)?;   // 查询 GitHub Releases API

    // 2. 无新版本 → 提示已是最新，退出
    if latest <= CURRENT_VERSION { ... }

    // 3. --check 模式：仅显示版本信息，不更新
    // 4. 确认（--yes 跳过）→ 下载并替换 binary
    let status = self_update::backends::github::Update::configure()
        .repo_owner("byx-darwin")
        .repo_name("gitflow-cli")
        .bin_name("gf")
        .current_version(current_version())
        .show_download_progress(true)
        .target_version_tag(&format!("v{latest}"))
        .build()?
        .update()?;

    // 5. 提示是否同时更新 skills → 调用 skills::update_skills()
}
```

### `skills.rs` 新增 — `gf skills update`

```rust
pub enum SkillsCommand {
    Install(InstallArgs),
    List(ListArgs),
    Uninstall(UninstallArgs),
    Update(UpdateArgs),        // 新增
}

pub fn update_skills(args: &UpdateArgs) -> miette::Result<()> {
    // 复用现有 install 流程，强制覆盖模式（等价于 `gf skills install --force`）
    // 覆盖已安装的 skills，并同步 binary 内嵌的新增 skills
    // 输出: ♻ 已覆盖: gf-workflow (1.0.0 → 1.1.0)
}
```

## 依赖变更

```toml
# Cargo.toml workspace 依赖
self_update = { version = "0.42", features = ["archive", "compression"] }
# 新增 semver 用于版本比较（--pre 过滤稳定版/预发布版）
semver = "1"
```

## 错误处理

| 场景 | 处理方式 |
|------|---------|
| 网络不可达 / GitHub Releases API 超时 | 返回错误 `网络错误: {e}`，建议重试，不修改任何文件 |
| 已是最新版本 | 输出 `✅ 已是最新版本 (v1.0.0)`，退出码 0 |
| binary 替换失败（权限） | 错误提示 + 建议 `sudo gf update` 或手动 `cargo install gitflow-cli` |
| 下载中断 | `self_update` 自动清理临时文件，返回错误 |
| `--check` 模式 | 只读，不写文件，不触发 skills 更新 |
| skills 更新部分失败 | 单个 skill 失败不中断整体，汇总报告 |

## 测试策略（TDD）

| 测试 | 验证点 |
|------|--------|
| `test_update_cli_args` | `--check`、`--pre`、`--yes` 参数解析 |
| `test_version_parse` | 版本号解析（`1.0.0` vs `1.0.0-rc.1`） |
| `test_compare_versions` | 版本比较逻辑（稳定版优先，`--pre` 包含预发布） |
| `test_latest_release_detection` | 模拟 GitHub Releases API 响应 |
| `test_skills_update_overwrites` | `skills update` 覆盖已安装 skills |
| `test_skills_update_installs_new` | `skills update` 同步 binary 内嵌的新增 skills |
| `test_skills_update_preserves_other_dirs` | `skills update` 不触碰非 `gf-*` 目录 |
| `test_update_prompt_skills` | `gf update` 后询问是否更新 skills |

## 关键约束

- **不做自动更新检查** — 仅手动 `gf update` 触发
- **不删除用户自定义 skills** — 只更新 `gf-*` 前缀的目录
- **错误时不留半成品** — 更新失败不修改当前 binary
