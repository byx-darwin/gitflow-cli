# Dogfooding Summary — wf-2026-07-31-001（Issue #97 / PR #112）

**Date:** 2026-08-01
**Executor:** byx-darwin（+ Claude gf-workflow 编排）
**Result:** PASS（范围化：只读 / 已演练项）
**Release Decision:** N/A（本 PR 非发布；发布风险项归入 #97-B 的发布前闸门）

## 范围说明

经用户确认，本次 dogfooding 采用**范围化**执行：仅覆盖只读 / 本工作流已实际演练的命令；带远程写副作用的风险项（创建/删除 release、GitCode 测试 PR merge）延后到 #97-B 真正的发布前闸门；GitLab 因未认证无法执行。

## 平台结果

| Platform | 计划项 | 已演练 | 通过 | 失败 | 说明 |
|----------|-------|-------|------|------|------|
| GitHub   | 4（release 风险项） | 命令面已演练（见下） | ✅ | 0 | release create/delete 风险项延后 #97-B |
| GitLab   | 5（中文标签 CRUD） | — | ⏭️ | — | 未认证（auth status 无返回），跳过 |
| GitCode  | 4（pr merge 非交互） | — | ⏭ | — | pr merge 风险项延后 #97-B |

## 本工作流实际演练的 GitHub 命令（全部成功）

工作流 Phase 1–4 全程以 `gf` 自身驱动（dogfooding）：

| 命令 | 用途 | 结果 |
|------|------|------|
| `auth status` | 认证校验（github/gitcode） | ✅ |
| `issue view 97` | 读取需求 | ✅ |
| `issue list --state open` | 列出 open Issues（triage） | ✅ |
| `issue comment 97` | 回写设计文档引用 + 需求审查报告 | ✅（写入成功） |
| `issue add-label` | triage 标签（#109/#110/#111） | ✅ |
| `pr create` | 创建 PR #112 | ✅ |
| `pipeline status/report` | 流水线分析 | ✅ |

## 发现的 Bug

- **#111** `fix(github): issue comment 返回 stale comment id（首条而非新建）`
  - 现象：`issue comment` 写入成功，但返回的 JSON 回显**上一条评论**的 `id`/`body`，而非新建评论。
  - 发现于 Phase 1（回写需求审查评论时），已独立成 Issue 并标记 `type:bug` / `priority:medium` / `triage:done`。
  - 建议合并 #112 后排期修复。

## 前置条件核对

- [x] Phase 1–3 已完成（issue → plan → implement → test → PR #112）
- [x] GitHub / GitCode 认证正常；GitLab 未认证（记录为环境约束）
- [x] 主工作区干净（Phase 4 报告为新增未提交工件）

## 后续

1. #97-B 发布前：执行完整 dogfooding（release 风险项 + GitCode pr merge + GitLab 中文标签，需先配置 GitLab 认证）。
2. 修复 #111（stale comment id）。
