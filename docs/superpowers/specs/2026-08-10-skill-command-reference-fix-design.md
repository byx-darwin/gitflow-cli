# Skill 文档命令引用修复 — 设计文档

**Issue:** #179
**Date:** 2026-08-10
**Status:** Draft
**Approach:** A — Minimal Fix + Validation Script

## Problem Summary

Skill 文档中引用的 `gf` CLI 命令与实际 CLI 实现不匹配，导致命令执行失败、工作流中断。经过全面扫描，发现 **8 类**不匹配问题，涉及 **12 个** skill 文件、**30+ 处**命令引用。

## Scope

### In Scope

1. 修复所有 skill 文档中不存在的 `gf` 命令引用
2. 为 `gf-repo` skill 添加 `[PLANNED]` 标记（整个命令组未实现）
3. 在 `gf-auth` skill 中添加认证状态同步的 caveat
4. 创建验证脚本防止未来漂移

### Out of Scope

- 实现缺失的 CLI 命令（`gf repo *`, `gf pr diff`, `gf pr cleanup` 等）
- 修复 `gf auth status` 与 `gh` 认证状态不同步的实现 bug（拆分为独立 Issue）

## Detailed Findings

### 1. `gf issue label` → `gf issue add-label` / `gf issue remove-label`

**实际命令语法：**
```bash
gf issue add-label <NUMBER> --label <LABEL>...    # 添加标签（支持多个）
gf issue remove-label <NUMBER> --label <LABEL>     # 移除标签（单个）
```

**影响文件：**

| File | Line(s) | Current | Fix |
|------|---------|---------|-----|
| `skills/gf-issue-triage/SKILL.md` | 50, 58, 100 | `gf issue label <n> --label "..."` | `gf issue add-label <n> --label "..."` |
| `skills/gf-issue/SKILL.md` | 65 | `gf issue label <number> --add <l> --remove <l>` | `gf issue add-label <n> --label <l>` + `gf issue remove-label <n> --label <l>` |

### 2. `gf pr diff` → 不存在

**实际命令：** CLI 无 `gf pr diff`。获取 PR diff 需使用 `git diff` 或 `gh pr diff`。

**影响文件：**

| File | Line(s) | Current | Fix |
|------|---------|---------|-----|
| `skills/gf-pr-inline-review/SKILL.md` | 51, 61, 76 | `gf pr diff <n>` | `gh pr diff <n>` 或 `git diff` |
| `skills/gf-pr-review/SKILL.md` | 52, 75 | `gf pr diff <n>` | `gh pr diff <n>` 或 `git diff` |

### 3. `gf pr resolve-comment` → 不存在

**影响文件：**

| File | Line(s) | Current | Fix |
|------|---------|---------|-----|
| `skills/gf-pr-apply-feedback/SKILL.md` | 56 | `gf pr resolve-comment <pr> --comment-id <id>` | 移除该行，改为注释说明使用平台 Web UI |

### 4. `gf pipeline retry` → 不存在

**影响文件：**

| File | Line(s) | Current | Fix |
|------|---------|---------|-----|
| `skills/gf-pipeline-analyzer/SKILL.md` | 44 | `Manual gf pipeline retry or platform web UI` | `Use platform web UI` |

### 5. `gf repo *` → 整个命令组不存在

**影响文件：**

| File | Line(s) | References | Fix |
|------|---------|------------|-----|
| `skills/gf-repo/SKILL.md` | 全文 | 6 个命令定义 + 命令表 | 添加 `[PLANNED]` banner |
| `skills/gf-issue/SKILL.md` | 142 | `gf repo clone` | `git clone` |
| `skills/gf-repo-onboarding/SKILL.md` | 117 | `gf repo clone` | `git clone` |

### 6. `gf auth status` 与 `gh` 认证状态不同步

**影响：** `gf auth status` 显示已登录，但底层 `gh` token 可能已过期。

**Fix：** 在 `gf-auth` skill 中添加 caveat 说明。

### 7. `gf comment` → 不存在（应为 `gf commit comment`）

**影响文件：**

| File | Line(s) | Current | Fix |
|------|---------|---------|-----|
| `skills/gf-commit/SKILL.md` | 53 | `gf comment <sha> --body <t> --path <p> --line <n>` | `gf commit comment <sha> --body <t> --path <p> --line <n>` |
| `skills/gf-pr-inline-review/SKILL.md` | 54, 62, 92 | `gf comment <sha> --body "..." --path <f> --line <l>` | `gf commit comment <sha> --body "..." --path <f> --line <l>` |

### 8. `gf review <verdict>` → 不存在（应为具体子命令）

**实际命令：** `gf review` 有子命令 `approve`, `request-changes`, `comment`, `submit`。不存在 `gf review <verdict>` 的通用形式。

**影响文件：**

| File | Line(s) | Current | Fix |
|------|---------|---------|-----|
| `skills/gf-pr-review/SKILL.md` | 54 | `gf review <verdict> <n> --body "<c>"` | `gf review approve <n> --body "<c>"` 或具体子命令 |
| `skills/gf-review/SKILL.md` | 54, 100 | `gf review <verdict> <n> --body "<c>"` | 替换为具体子命令 `approve`/`request-changes`/`comment` |

## Validation Script

**路径：** `scripts/validate-skill-commands.sh`

**功能：**
1. 递归调用 `gf --help` 和各子命令 `--help` 提取所有有效命令
2. 扫描 `skills/*/SKILL.md` 中的 `gf <command>` 模式
3. 报告不存在于实际 CLI 中的命令引用
4. 排除已标注 `[PLANNED]` 的 skill 文件
5. 退出码：0 = 全部通过，1 = 存在不匹配

**集成：** 可在 CI 中运行，防止未来漂移。

## Auth Caveat

在 `skills/gf-auth/SKILL.md` 的 Preconditions 部分后添加：

```markdown
## Known Caveats

> **Note:** `gf auth status` may show cached login state. If commands fail with
> auth errors, run `gh auth status` to verify the underlying token is valid.
> Re-run `gf auth login` or `gh auth login` if the token has expired.
```

## Testing Plan

1. **Manual verification:** 逐文件检查每处修改后的命令是否与实际 CLI 一致
2. **Validation script:** 运行 `scripts/validate-skill-commands.sh`，确保零告警
3. **Spot check:** 在工作流中实际调用修改后的命令路径，验证不报错

## Files Changed Summary

| # | File | Type of Change |
|---|------|---------------|
| 1 | `skills/gf-issue-triage/SKILL.md` | Replace `gf issue label` → `gf issue add-label` |
| 2 | `skills/gf-issue/SKILL.md` | Replace `gf issue label` → `add-label`/`remove-label`; `gf repo clone` → `git clone` |
| 3 | `skills/gf-pr-inline-review/SKILL.md` | Replace `gf pr diff` → `gh pr diff`; `gf comment` → `gf commit comment` |
| 4 | `skills/gf-pr-review/SKILL.md` | Replace `gf pr diff` → `gh pr diff`; `gf review <verdict>` → `gf review approve/request-changes` |
| 5 | `skills/gf-pr-apply-feedback/SKILL.md` | Remove `gf pr resolve-comment` |
| 6 | `skills/gf-pipeline-analyzer/SKILL.md` | Remove `gf pipeline retry` reference |
| 7 | `skills/gf-repo/SKILL.md` | Add `[PLANNED]` banner |
| 8 | `skills/gf-repo-onboarding/SKILL.md` | Replace `gf repo clone` → `git clone` |
| 9 | `skills/gf-auth/SKILL.md` | Add auth desync caveat |
| 10 | `skills/gf-commit/SKILL.md` | Replace `gf comment` → `gf commit comment` |
| 11 | `skills/gf-review/SKILL.md` | Replace `gf review <verdict>` → specific subcommands |
| 12 | `scripts/validate-skill-commands.sh` | New file — validation script |
