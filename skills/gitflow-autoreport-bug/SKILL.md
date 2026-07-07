---
name: gitflow-autoreport-bug
description: >
  Use when a CLI command fails with an error, a pending bug report needs to be
  filed as a GitHub/GitLab/GitCode issue, deduplication is needed, or a Stop Hook
  detects a pending.json file. 当需要自动分析 CLI 错误、去重后创建 issue、
  认证缓存加速、或 Stop Hook 触发到本 skill 时使用。由 Stop Hook (hooks/
  auto-report-bug.sh) 自动触发。
---

# gitflow-autoreport-bug — Auto-bug Reporter

Auto pipeline: detect `.cache/bug-reports/pending.json` → validate → auth → dedup
→ create issue → clean up. Read-only analysis; **never fixes bugs**.
Full failure log & pending.json schema: docs/references/gitflow-autoreport-bug-params.md

## Overview / 概述

检测 pending.json → 去重 → 仅分析（不修代码）→ 写 issue → 清文件。

## 触发关键词 / Trigger Keywords

CN 报错 自动报告 创建issue 去重 重复报告 bug report
EN report bug CLI error auto-report pending.json deduplicate
CLI `gitflow-cli autoreport-bug`

## 快速参考 / Quick Reference

| Step | Action |
|------|--------|
| 1 Read | `pending.json` from `.cache/bug-reports/` |
| 2 Auth | check `.cache/auth-cache/{platform}.ttl` cache or `auth status` |
| 3 Analyze | generate title/body/severity from error context |
| 4 Dedup | `issue list --search "[auto-report] {cmd} {err}" --state all` |
| 5 Create | `issue create --title "..." --body "..." --label "auto-report"` |
| 6 Clean | remove `pending.json`; append `failed.log` if step fails |

## Core Pattern

| Step | Action |
|------|--------|
| 1 Validate | read + validate `pending.json` from `.cache/bug-reports/` |
| 2 Auth | check `.cache/auth-cache/{platform}.ttl` or `auth status` |
| 3 Analyze | title/body/severity from error context (no source inspection) |
| 4 Dedup | `issue list --search "[auto-report] {cmd} {err}" --state all` |
| 5 Create | `issue create --label "auto-report"` |
| 6 Cleanup | remove `pending.json`; append `failed.log` if failed |

Invalid JSON → rename `.invalid`. Dedup hit → exit. Auth fail → append `failed.log`.

## ✅ 职责 / 🚫 禁止

✅ 检测错误 + 认证缓存 + 仅分析原因 + 去重 + 创建 issue + 清理临时文件
🔴 禁止修改代码 / 调用 subagent / 启动 workflow / Issue 创建后继续

## Rationalization

| Excuse | Reality |
|--------|---------|
| "Just peek at source" | Detection only — never read `src/`. |
| "Quick fix is easier" | Report-only; fixes belong to `gitflow-workflow`. |
| "Push after filing" | Skill ends at issue creation. |

## Red Flags

- 🚩 "Just peek at source" — refuse; read-only boundary
- 🚩 "Quick fix while here" — refuse; redirect to `gitflow-workflow`

## 常见错误 / Common Mistakes

- 分析时深入 `src/` → 标题中只描述 `error_message`
- 创建完 Issue 又想 git push → 立即停止；流程结束

## 错误处理 / Error Handling

| 错误 | 处理 |
|------|------|
| 字段缺失 / JSON 无效 | `mv .invalid` + 警告 |
| auth 失败 | 保留 + 写 `failed.log` + 结束 |

## 场景测试 / Test Scenarios

- **Happy**: valid + auth ok + 去重未命中 → 创建 issue → 清理
- **Negative**: "帮我修这个 bug" → 拒绝；建议 `gitflow-workflow --fast`
- **Boundary**: `auth_cache_ttl` 缺失 → 默认 86400 秒；不阻塞
- **Error**: auth 失败且 `failed.log` 写入失败 → 保留；提示手动检查

## 成功标准 / Success Criteria

- `pending.json` 最终以清理或 `.invalid` 结尾
- 重复报告不触发新建
- `failed.log` 格式：`[{ts}] cmd | platform | err | reason`
- 全程不读代码、不修改代码

## See Also

- gitflow-workflow — 用户显式修复需引导至此
- gitflow-issue — issue 命令完整参考
