# GitLab glab 1.113 Compatibility Matrix Update — Design

> **Workflow:** `wf-2026-08-18-003` · **Issue:** [#198](https://github.com/byx-darwin/gitflow-cli/issues/198)
> **Mode:** standard · **Skill source:** superpowers
> **Date:** 2026-08-18

## 1. Problem Statement

巡检机器人发现 glab 发布 **1.113.0**，而 GitLab 兼容性矩阵
`crates/core/resources/compatibility-matrix.json` 的 `tested_versions` 上限仍为
**1.112.0**。需验证 glab 1.113.0 兼容性并将版本纳入矩阵。

## 2. 已验证事实

- 本地 glab：`glab 1.113.0 (d62881304)`（Homebrew）
- `make smoke-test-gitlab`：**54 passed, 0 failed, 5 skipped** —— 与 glab 1.112.0
  验证结果（#144）完全一致
- **前置修复已合入**：glab 1.113 对 gf 的破坏性变更（写操作 `--output json`、
  `mr ready/draft` 等）已由 Issue #199 / PR #201（`980d5c5`）修复并合入 `dev`
- 冒烟测试 5 项 API 读取 SKIP 为最佳努力模式无凭据跳过，非回归

## 3. 范围（Scope）

**纳入：**
1. `crates/core/resources/compatibility-matrix.json`：GitLab `tested_versions`
   `["1.111.0", "1.112.0"]` → `["1.111.0", "1.112.0", "1.113.0"]`，`updated_at` → `2026-08-18`
2. `docs/compatibility-matrix.md`：运行 `make compatibility-matrix` 自动重新生成
   （顺带修复 #144 遗留的过期文档，GitLab 行仍显示 1.111.0）

**明确不纳入：**
- `min_version` 提升：1.113.0 的破坏性变更已在 gf 侧修复（#199），最低版本仍为 1.30.0
- 契约 fixture 补充：GitLab fixtures 为 `v1` 通用版本，无逐版本 fixture；破坏性变更
  的代码修复与测试属 #199 范围
- 任何 GitHub / GitCode 平台变更

## 4. 验证

- `cargo test -p gitflow-core`（compatibility.rs 解析嵌入 JSON + 矩阵测试）
- `cargo fmt` + `cargo clippy -- -D warnings`
- 回归确认：`make smoke-test-gitlab` 不回归

## 5. 提交与 PR

- 提交信息：`chore(deps): add glab 1.113.0 to compatibility matrix`
- PR 关闭：`Closes #198`
