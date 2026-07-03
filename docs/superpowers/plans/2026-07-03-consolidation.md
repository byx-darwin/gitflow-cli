# Consolidation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Consolidate gitcode-dev-workflow's orchestration strengths and ncgo-code-skills' weekly-report + improved auto-report-bug into gitflow-cli as the single source of truth.

**Architecture:** Five documentation/script-level changes: (1) gitflow-workflow gains compliance checklists, --body-file rule, enforcement header; (2) gitflow-quality gains pre-commit as step 6; (3) new gitflow-weekly-report skill ported from ncgo-code; (4) gitflow-autoreport-bug merges both implementations with auth cache + JSON validation; (5) hooks/sync-readme-check.sh added and registered.

**Tech Stack:** Markdown (SKILL.md), Bash (hooks), JSON (settings.json, pending.json schema)

## Global Constraints

- All changes are SKILL.md text edits or new bash files — no Rust compilation changes
- weekly-report 保持纯 bash，不引入 gitflow-cli API
- weekly-report 推荐用户级安装（`gitflow-cli skills install -g`）
- auto-report-bug 保留 gitflow-cli 现有 pending.json schema（error_id / command / platform / error_code / error_message / timestamp），仅追加可选字段 `auth_cache_ttl`
- 不删除或归档源仓库
- hooks 注册沿用 gitflow-cli 现有的 `.claude/settings.json` matcher 模式

---

### Task 1: gitflow-quality — Add Pre-commit Step

**Files:**
- Modify: `skills/gitflow-quality/SKILL.md`

**Interfaces:**
- Consumes: existing 5-step quality gate (build → test → coverage → format → static)
- Produces: updated quality gate with optional 6th step

- [ ] **Step 1: Add pre-commit check to checklist table**

In the "检查清单" section, append row 6 to the existing table:

```markdown
| 6 | pre-commit | `pre-commit run --all-files` 或读取 `.pre-commit-config.yaml` 检查配置 | 全部 hook 通过 |
```

Also add a note below the table:

```markdown
> **Pre-commit N/A 处理：** 如果项目没有 `.pre-commit-config.yaml` 配置文件，pre-commit 检查标记为 `N/A` 跳过。
```

- [ ] **Step 2: Add pre-commit step explanation**

After the existing "步骤 5: Static" section and before "## Quality Report 格式", add a new section:

```markdown
### 步骤 6：Pre-commit — pre-commit 检查

如果项目配置了 `.pre-commit-config.yaml`：

```bash
pre-commit run --all-files 2>&1
```

- **通过**：全部 hook 通过 → 记录 `✅ pre-commit`
- **失败**：有 hook 失败 → 记录 `❌ pre-commit` + 失败摘要 → **fast-fail**

**N/A 处理**：如果项目没有 `.pre-commit-config.yaml`，标记为 `N/A` 跳过，不影响最终判定。
```

- [ ] **Step 3: Update Quality Report format**

In the "Quality Report 格式" example, append the pre-commit row:

```markdown
| pre-commit | ✅     | All hooks passed |
```

Add N/A example in the notes:

```markdown
或

| pre-commit | N/A    | No configuration |
```

- [ ] **Step 4: Update non-Rust project adaptation table**

In "非 Rust 项目的适配命令" table, append the pre-commit row：

| 检查项 | Node.js | Python | Go |
|--------|---------|--------|-----|
| pre-commit | `npx lint-staged` 或留空 | `pre-commit run --all-files` | `pre-commit run --all-files` |

Note: the table is informational for non-Rust projects; the primary `pre-commit run --all-files` command is language-agnostic and works identically across all four supported languages when the tool is installed. Rust projects reuse the same command.

- [ ] **Step 5: Update "注意事项" section**

Append:

```markdown
- **pre-commit 可选**：pre-commit 检查仅在项目配置了 `.pre-commit-config.yaml` 时执行，否则标记 N/A
```

- [ ] **Step 6: Verify and commit**

```bash
cat skills/gitflow-quality/SKILL.md | head -5
# Verify frontmatter intact

git add skills/gitflow-quality/SKILL.md
git commit -m "quality: add pre-commit as 6th quality gate step"
```

---

### Task 2: gitflow-workflow — Add Compliance Checklists, --body-file Rule, Enforcement Header

**Files:**
- Modify: `skills/gitflow-workflow/SKILL.md`

**Interfaces:**
- Consumes: existing 4-phase structure with gates
- Produces: enhanced SKILL.md with compliance checklists after each phase, --body-file rule

- [ ] **Step 1: Add 🚨 enforcement header**

After the opening paragraph and before "## 启动条件", insert:

```markdown
## 🚨 强制执行规则

以下规则不可跳过，任何一步不满足就不能进入下一步。
```

- [ ] **Step 2: Add compliance checklist after Phase 1**

After step 1.4 and the Phase 1 gate section, add:

```markdown

### Phase 1 合规检查

```
Phase 1 合规检查:
  [ ] 步骤 1: brainstorming — 明确的功能描述、验收标准、边界条件
  [ ] 步骤 2: issue-create — Issue URL 已获取
  [ ] 步骤 3: issue-review — 需求分析报告已完成
  [ ] Issue 评论已发布: <comment_link>

  缺失项: <list or "无">
  是否允许进入下一阶段: [ ]
```
```

- [ ] **Step 3: Add compliance checklist after Phase 2**

After step 2.5 and the Phase 2 gate section, add:

```markdown

### Phase 2 合规检查

```
Phase 2 合规检查:
  [ ] 步骤 1: writing-plans — 原子任务清单已生成
  [ ] 步骤 2: TDD 循环 — 所有任务完成 RED→GREEN→REFACTOR
  [ ] 步骤 3: subagent-dev — 每个任务有独立子代理执行记录
  [ ] 步骤 4: requesting-review — 每个任务已完成代码审查
  [ ] Issue 评论已发布: <comment_link>

  缺失项: <list or "无">
  是否允许进入下一阶段: [ ]
```
```

- [ ] **Step 4: Add compliance checklist after Phase 3**

After step 3.3 and the Phase 3 gate section, add:

```markdown

### Phase 3 合规检查

```
Phase 3 合规检查:
  [ ] 步骤 1: build ✅ — 编译通过
  [ ] 步骤 2: test ✅ — 全部测试通过
  [ ] 步骤 3: coverage ✅ — 覆盖率达标
  [ ] 步骤 4: format ✅ — 格式规范
  [ ] 步骤 5: static ✅ — 无警告
  [ ] 步骤 6: pre-commit ✅ — 全部 hook 通过 (或 N/A)
  [ ] Issue 评论已发布: <comment_link>

  缺失项: <list or "无">
  是否允许进入下一阶段: [ ]
```
```

- [ ] **Step 5: Add compliance checklist after Phase 4**

After step 4.6 (the final audit comment), add:

```markdown

### Phase 4 合规检查

```
Phase 4 合规检查:
  [ ] 步骤 1: pr-create — PR URL 已获取
  [ ] 步骤 2: pr-review — 6 维度审查完成
  [ ] 步骤 3: finishing — 分支已合并
  [ ] 步骤 4: release-helper (可选) — Release 已创建
  [ ] 步骤 5: issue-close — Issue 已关闭
  [ ] Issue 最终审计日志已发布: <comment_link>

  缺失项: <list or "无">
  工作流是否全部完成: [ ]
```
```

- [ ] **Step 6: Replace long `--body` calls with `--body-file`**

In steps 1.4, 2.5, 3.3, and 4.6, replace patterns like:

```bash
gitflow-cli issue comment <number> --body "## Phase X: ...<long content>..."
```

With:

```bash
# 写入临时文件
cat > /tmp/phase-report.md << 'REPORT'
## Phase X: ...
...长内容...
REPORT

gitflow-cli issue comment <number> --body-file /tmp/phase-report.md
rm -f /tmp/phase-report.md
```

Keep step 4.5 uses `--body` as it's short enough (<100 bytes).

- [ ] **Step 7: Update 注意事项 section**

Append to the existing list:

```markdown
- **--body-file 强制**：长内容（>100 字节）必须通过 `--body-file` 传入，不使用 `--body` 直接传
- **合规检查不可跳过**：每个阶段结束必须输出合规清单，逐项打勾后才能进入下一阶段
```

- [ ] **Step 8: Verify and commit**

Verify the markdown renders correctly and frontmatter is intact, then commit:

```bash
git add skills/gitflow-workflow/SKILL.md
git commit -m "workflow: add compliance checklists, --body-file rule, enforcement header"
```

---

### Task 3: Create gitflow-weekly-report Skill

**Files:**
- Create: `skills/gitflow-weekly-report/SKILL.md`

**Interfaces:**
- Consumes:用户指定项目路径和时间范围
- Produces: 结构化研发周报（纯文本）

- [ ] **Step 1: Create the SKILL.md**

Write `skills/gitflow-weekly-report/SKILL.md` with adapted content from ncgo-code's weekly-report. Rename skill, update description for gitflow-cli context, add install note:

```markdown
---
name: gitflow-weekly-report
description: |
  生成研发周报，扫描多个 Git 仓库的提交记录，按照统一模板输出本周工作复盘（纯文本、无表格）。
  支持指定截止时间和项目列表，自动合并跨天提交。跨项目工具，推荐用户级安装。

  TRIGGER when: 用户要生成周报、查看本周提交汇总、统计多项目工作量、
  或说 "weekly report"、"周报"、"本周工作总结"、"上周做了什么"、
  "生成研发周报"、"多项目周报"。
---

# 研发周报生成

> **安装建议：** 推荐使用用户级安装，跨项目可用：
> ```bash
> gitflow-cli skills install -g gitflow-weekly-report
> ```

扫描一个或多个 Git 仓库，汇总指定时间段内的提交记录，按项目分组生成结构化的研发周报。输出采用纯文本格式，不使用表格。

## 输入说明

用户应提供：
- **截止时间**：周报统计截止时间，格式如 "6月5日 18:00" 或 "2026-06-05T18:00"。未指定时默认本周五 18:00。
- **项目路径**：一个或多个项目的本地路径。未指定时默认当前项目。
- **是否包含周日**：是否将截止时间之后（含周日）的提交也纳入统计。默认否。

## 工作流程

### 第一步：确定时间范围

1. 根据用户提供的截止时间，解析出 `--since` 和 `--until` 参数。
2. 如果用户要求包含周日，将 `--until` 延长到下周一 00:00。
3. `--since` 默认为截止时间所在周的周一（ISO week），或用户指定的开始日期。

### 第二步：扫描各项目 Git 日志

对每个项目执行：

```bash
# 获取带日期的提交日志
git log --format="%h %ai %s" --since="<start>" --until="<end>"

# 统计提交数
git log --format="%h" --since="<start>" --until="<end>" | wc -l

# 可选：变更统计
git diff --stat --since="<start>" --until="<end>" | tail -1
```

注意事项：
- 年份务必匹配实际提交年份（不要用 2025 去查 2026 的数据）。
- 如果项目有多个分支，默认使用当前分支（通常 master/main）。
- 仅统计已提交的内容，不含未暂存的工作区修改。

### 第三步：分类汇总

按以下维度对提交进行分类：
- **功能开发**：`feat:` 前缀或新增功能相关的提交
- **Bug 修复**：`fix:` 前缀或修复相关提交
- **重构**：`refactor:` 前缀
- **文档**：`docs:` 前缀
- **CI/质量**：`chore:`、`ci:`、`test:` 前缀及 clippy/fmt/test 相关
- **其他**：无法归类的提交

分类时尽量合并同类项，用一句话概括一组相关提交的完成内容。不需要逐条列出每个提交，而是在某个功能方向下用一句话总结。

### 第四步：生成报告

按以下模板输出（纯文本，**不使用表格**）：

```
# 研发本周工作复盘（<日期范围>）

---

## 本周完成事项

### 一、<项目名>（N 个提交）

**<分类标签>：**
- <一句话描述完成内容>。（状态）`<commit-hash>`

### 二、<项目名>（N 个提交）

...

---

## 本周关键数据

- **总提交数**：X 个（项目A N 个，项目B M 个，...）
- **分布**：<主要工作量分布描述>
- **分支情况**：<各项目所在分支>

---

## 未完成事项及原因

- **<事项名>**：<描述>。原因：<原因>。下步：<处理方案>

---

## 需要协调的问题

- <问题描述>。（如无则写"无"并注明原因）

---

## 下周工作建议

- **<建议事项>**：<建议原因>，预期结果：<预期结果>。

---

> 备注：<如有超过截止时间的提交，在此说明>
```

### 报告规范

1. **禁止使用表格**。所有信息用列表和段落呈现。
2. 提交 hash 用反引号包裹，如 `0f5ec81`。
3. 提交分类不求逐条详尽，同方向合并为一个要点，一句话概括。
4. 关键数据中提交数要真实准确，不可估算。
5. "未完成事项"要结合上下文推断——如果某个项目提交明显偏少或明显有未收尾的工作，应在此处体现。
6. 用中文撰写。

## 常见场景

### 多项目周报

用户提供多个项目路径，如：
```
../token-fleet-switch ../tokenless ../agent-proxy-rust 这三个项目 上周总结
```

解析逻辑：
- "上周" = 上周一到周五（或周六）
- 三个路径均需扫描
- 报告按项目分组

### 指定截止时间

```
6月5日 18:00 截止
```

解析逻辑：
- 截止 = 当前年份的 6月5日 18:00
- 起始 = 6月2日（周一）

### 包含周日

```
周日也合并
```

解析逻辑：
- 将截止时间延长到周日结束
- 同步更新报告日期标题

## 边界情况处理

- **项目路径不存在**：跳过并提示用户。
- **时间范围内无提交**：在报告中注明 "本周暂无提交"。
- **单项目单提交**：仍需完整模板，不可省略章节。
- **跨年**：提交日期可能跨年，`--since`/`--until` 需使用完整日期格式。

## 注意事项

1. 安全：所有 `git log` 和 Bash 命令均为只读操作，不修改仓库。
2. 隐私：不暴露文件路径中的用户名或敏感信息。
3. 准确性：提交数、日期等数据必须从 git 实际获取，不可编造。
4. **跨项目设计**：本 skill 推荐安装到用户级目录（`gitflow-cli skills install -g gitflow-weekly-report`），这样无论从哪个项目目录调用都能正常扫描。

---

**Version**: 2.0.0
**Last Updated**: 2026-07-03
**Source**: Migrated from ncgo-code-skills/weekly-report (adapted for gitflow-cli)
```

- [ ] **Step 2: Verify and commit**

Verify the new skill renders correctly:

```bash
head -10 skills/gitflow-weekly-report/SKILL.md
```

Commit:

```bash
git add skills/gitflow-weekly-report/SKILL.md
git commit -m "feat: add gitflow-weekly-report skill (ported from ncgo-code-skills)"
```

---

### Task 4: Merge gitflow-autoreport-bug — Dual-Source Consolidation

**Files:**
- Modify: `skills/gitflow-autoreport-bug/SKILL.md`
- Modify: `hooks/auto-report-bug.sh`

**Interfaces:**
- Consumes: pending.json (existing schema), `.cache/auth-cache/{platform}.ttl` (new)
- Produces: unified bug report flow with auth cache + dedup + Issue creation

- [ ] **Step 1: Rewrite gitflow-autoreport-bug/SKILL.md**

Replace the entire content of `skills/gitflow-autoreport-bug/SKILL.md` with the merged version. The new content combines gitflow-cli's CLI-based dedup/search with ncgo-code's auth cache, JSON validation, and failed.log retry:

```markdown
---
name: gitflow-autoreport-bug
description: |
  自动分析 CLI 错误报告，auth cache 加速认证检查，去重检查后创建
  GitHub/GitLab/GitCode Issue，失败记录保留到 failed.log 待重试。
  由 Stop Hook (hooks/auto-report-bug.sh) 自动触发。
---

# gitflow-autoreport-bug

自动处理 gitflow CLI 的错误报告：检测 pending.json → 验证 JSON →
auth cache 检查 → 去重搜索 → Claude 分析 → 创建 Issue → 清理临时文件。

## 前置条件

- 当前目录位于 git 仓库中
- `pending.json` 文件存在于 `.cache/bug-reports/pending.json`
- `gitflow` CLI 已安装且可用

**前置检查：** 在执行任何步骤之前，先验证 `gitflow` CLI 是否可用：

```bash
command -v gitflow-cli
```

- 失败 → 输出「gitflow CLI 未安装，请运行：cargo install gitflow-cli」，保留 `pending.json`，结束
- 成功 → 继续执行步骤

## 执行步骤

### Step 1: 读取并验证 pending.json

1. 检查 `.cache/bug-reports/pending.json` 是否存在
   - 不存在 → 输出「无待处理的错误报告」，结束
2. 读取并解析 JSON
   - JSON 格式无效或缺少必填字段 → 重命名为 `pending.json.invalid`，输出警告，结束
3. 提取以下字段：
   - `error_id` — 错误唯一标识（必填）
   - `command` — 失败的 CLI 命令（必填）
   - `platform` — 目标平台（必填，github / gitlab / gitcode）
   - `error_code` — 错误代码（必填）
   - `error_message` — 错误信息（必填）
   - `timestamp` — 错误发生时间（必填）
   - `auth_cache_ttl` — auth cache 有效期（可选，默认 86400 秒 = 24h）

### Step 2: Auth Cache 检查

检查 auth cache 是否在 TTL 内：

```bash
CACHE_FILE=".cache/auth-cache/{platform}.ttl"
if [ -f "$CACHE_FILE" ]; then
    CACHED_TIME=$(cat "$CACHE_FILE")
    NOW=$(date +%s)
    AGE=$(( NOW - CACHED_TIME ))
    TTL=${auth_cache_ttl:-86400}
    if [ "$AGE" -lt "$TTL" ]; then
        echo "Auth cache 命中（age: ${AGE}s, TTL: ${TTL}s），跳过认证检查"
        AUTH_OK=true
    fi
fi
```

Cache 未命中时，调用 gitflow-cli 检查认证：

```bash
gitflow-cli auth status --platform {platform}
```

- 失败 → 保留 `pending.json` + 追加记录到 `.cache/bug-reports/failed.log`：

```bash
echo "[{timestamp}] 命令: {command} | 平台: {platform} | 错误: {error_code} | 失败原因: auth 检查失败" >> .cache/bug-reports/failed.log
```

输出「认证失败，已记录到 failed.log，pending.json 保留待后续重试」，结束

- 成功 → 更新 auth cache timestamp：

```bash
mkdir -p .cache/auth-cache
date +%s > .cache/auth-cache/{platform}.ttl
```

### Step 3: Claude 分析

基于 Step 1 提取的错误上下文，生成以下分析内容：

1. **可能原因** — 根据 `error_code` 和 `error_message` 推断可能的根因
2. **建议修复方向** — 给出具体的修复建议或排查方向
3. **严重程度评估** — 基于 `error_code` 和影响范围评估严重程度（critical / high / medium / low）

基于分析结果，生成 Issue 标题和正文：

- **标题格式：** `[auto-report] gitflow {command} — {error_code}`
- **正文内容：**
  - 错误摘要（command、platform、error_code、error_message）
  - 可能原因分析
  - 建议修复方向
  - 严重程度评估
  - 环境信息（timestamp、error_id）

### Step 4: 去重检查

构造搜索关键词：`[auto-report] {command} {error_code}`，调用 gitflow-cli 搜索已有 Issue：

```bash
gitflow-cli issue list --search "[auto-report] {command} {error_code}" --state all
```

- 找到匹配 Issue → 去重命中，输出「已存在相同报告: #N」，清理 `pending.json`，结束
- 未找到 → 继续 Step 5

### Step 5: 创建 Issue

调用 gitflow-cli 创建 Issue：

```bash
gitflow-cli issue create --title "[auto-report] gitflow {command} — {error_code}" --body "..." --label "auto-report"
```

- 成功 → 输出 Issue URL，清理 `pending.json`
- 失败 → 保留 `pending.json`，追加到 `failed.log`

### Step 6: 清理

成功创建 Issue 后：

```bash
rm -f .cache/bug-reports/pending.json
```

输出完成信息。

## auth cache 机制

- **缓存位置：** `.cache/auth-cache/{platform}.ttl`（每平台独立）
- **缓存内容：** Unix 时间戳（认证成功的时刻）
- **TTL：** 默认 86400 秒（24 小时），可通过 `pending.json` 的 `auth_cache_ttl` 字段覆盖
- **缓存失效：** TTL 过期后，下次运行时重新调用 `gitflow-cli auth status`

## failed.log 格式

```
[2026-07-03T10:00:00Z] 命令: gitflow issue create | 平台: github | 错误: 401 | 失败原因: auth 检查失败
[2026-07-03T11:30:00Z] 命令: gitflow pr create | 平台: gitlab | 错误: 500 | 失败原因: issue create 失败
```

## pending.json Schema

```json
{
  "error_id": "550e8400-e29b-41d4-a716-446655440000",
  "command": "gitflow issue create",
  "platform": "github",
  "error_code": 401,
  "error_message": "Unauthorized",
  "timestamp": "2026-07-03T10:00:00Z",
  "auth_cache_ttl": 86400
}
```

- `auth_cache_ttl` 可选，缺省 86400
- 其余字段必填，缺失则视为无效 JSON

## 异常处理

- **JSON 格式无效：** 缺少必填字段或语法错误，将文件重命名为 `pending.json.invalid`，输出警告，结束
- **auth 检查失败：** 保留 pending.json，追加到 failed.log，等待下次重试
- **去重命中：** 清理 pending.json，输出已有 Issue 信息
- **Issue 创建失败：** 保留 pending.json + 追加 failed.log

## 触发机制

本 skill 由 Stop Hook (`hooks/auto-report-bug.sh`) 自动触发。当 Claude 完成响应后，hook 检测 `.cache/bug-reports/pending.json` 是否存在，存在则输出内容引导 Claude 加载本 skill。
```

- [ ] **Step 2: Replace hooks/auto-report-bug.sh**

Replace `hooks/auto-report-bug.sh` with the merged version incorporating ncgo-code's banner output with gitflow-cli's existing interactive-TTY guard and auth cache concept:

```bash
#!/usr/bin/env bash
# Stop Hook: detect CLI errors and surface them for automated reporting.
#
# Triggered by the Claude Code Stop Hook configured in .claude/settings.json.
# The Rust CLI writes error reports to .cache/bug-reports/pending.json
# whenever it fails in non-interactive mode (CI / subprocess). This script:
#   1. Checks for pending.json
#   2. Shallow-validates JSON
#   3. Uses auth cache (24h TTL) to avoid redundant auth checks
#   4. Outputs a banner that triggers the gitflow-autoreport-bug skill
#
# Exit codes: 0 always (silent no-op when nothing to do)

set -euo pipefail

REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || exit 0)
[ -z "$REPO_ROOT" ] && exit 0

PENDING_FILE="$REPO_ROOT/.cache/bug-reports/pending.json"

# No pending error report — silent exit.
if [ ! -f "$PENDING_FILE" ]; then
  exit 0
fi

# Interactive terminal guard — skip if in TTY.
if [ -t 1 ] || [ -t 0 ]; then
  exit 0
fi

# Read pending report content.
PENDING_CONTENT=$(cat "$PENDING_FILE")

# Shallow JSON validation — require at least "error_code" field.
if ! echo "$PENDING_CONTENT" | grep -q '"error_code"'; then
  mv "$PENDING_FILE" "${PENDING_FILE}.invalid"
  echo "⚠️  pending.json 格式异常，已重命名为 pending.json.invalid" >&2
  exit 0
fi

# Extract key fields for the prompt banner.
COMMAND=$(echo "$PENDING_CONTENT" | grep -o '"command"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | sed 's/.*: *"//;s/"$//')
ERROR_CODE=$(echo "$PENDING_CONTENT" | grep -o '"error_code"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | sed 's/.*: *"//;s/"$//')
PLATFORM=$(echo "$PENDING_CONTENT" | grep -o '"platform"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | sed 's/.*: *"//;s/"$//')
TIMESTAMP=$(echo "$PENDING_CONTENT" | grep -o '"timestamp"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | sed 's/.*: *"//;s/"$//')

# Auth cache check (24h TTL).
CACHE_FILE="$REPO_ROOT/.cache/auth-cache/${PLATFORM}.ttl"
AUTH_CACHE_TTL=86400
AUTH_STATUS="未知"

if [ -f "$CACHE_FILE" ]; then
  CACHED_TIME=$(cat "$CACHE_FILE")
  NOW=$(date +%s 2>/dev/null || python3 -c "import time; print(int(time.time()))")
  AGE=$(( NOW - CACHED_TIME ))
  if [ "$AGE" -lt "$AUTH_CACHE_TTL" ]; then
    AUTH_STATUS="✅ cache 命中（age: ${AGE}s）"
  fi
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  🐛 检测到 gitflow CLI 错误报告"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "  命令:   ${COMMAND:-unknown}"
echo "  平台:   ${PLATFORM:-unknown}"
echo "  错误码: ${ERROR_CODE:-unknown}"
echo "  时间:   ${TIMESTAMP:-unknown}"
echo "  认证:   ${AUTH_STATUS}"
echo ""
echo "  原始报告:"
echo "$PENDING_CONTENT"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  请加载 gitflow-autoreport-bug Skill 执行自动 Bug 报告流程。"
echo "  Skill 路径: skills/gitflow-autoreport-bug/SKILL.md"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

exit 0
```

- [ ] **Step 3: Make hook executable and verify**

```bash
chmod +x hooks/auto-report-bug.sh
bash -n hooks/auto-report-bug.sh
echo "Syntax OK"
```

- [ ] **Step 4: Verify and commit**

```bash
git add skills/gitflow-autoreport-bug/SKILL.md hooks/auto-report-bug.sh
git commit -m "feat: merge auto-report-bug with auth cache, JSON validation, failed.log retry"
```

---

### Task 5: Add sync-readme-check.hook + Register Hooks

**Files:**
- Create: `hooks/sync-readme-check.sh`
- Modify: `.claude/settings.json`

**Interfaces:**
- Consumes: actual `skills/` directory structure, README content
- Produces: diff reminder when structure diverges

- [ ] **Step 1: Create hooks/sync-readme-check.sh**

Adapt ncgo-code's version for gitflow-cli's structure. The key difference: gitflow-cli has a `skills/` directory at the repo root containing gitflow-* skills, plus supporting directories (`docs/`, `hooks/`, `crates/`). The sync check should compare actual top-level directories against what's documented in README.md.

```bash
#!/usr/bin/env bash
# sync-readme-check.sh — Stop Hook: check if README.md's directory
# structure section matches the actual repo structure.
# Outputs a reminder when they diverge.

set -euo pipefail

REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || exit 0)
[ -z "$REPO_ROOT" ] && exit 0
cd "$REPO_ROOT" || exit 0

# Get actual top-level directories (excluding hidden, .git, target)
get_actual_dirs() {
  find . -maxdepth 1 -type d \
    ! -name '.' \
    ! -name '.*' \
    ! -name 'target' \
    | sed 's|^\./||' \
    | sort
}

# Extract directory names from README's Structure section
get_readme_dirs() {
  local file="$1"
  awk '/^## Structure/,/^## [^S]/' "$file" 2>/dev/null \
    | grep -E '^\||├── |└── ' \
    | sed 's/|//g' \
    | grep -oE '[a-zA-Z0-9_-]+/' \
    | sed 's|/||' \
    | sort -u || true
}

# Get directories from skills/ subdirectory
get_skill_names() {
  find skills -maxdepth 1 -type d \
    ! -name 'skills' \
    | sed 's|skills/||' \
    | sort
}

# Compare top-level structure
actual_dirs=$(get_actual_dirs)
readme_dirs=$(get_readme_dirs "README.md")

missing=$(comm -23 <(echo "$actual_dirs") <(echo "$readme_dirs") 2>/dev/null || true)
extra=$(comm -13 <(echo "$actual_dirs") <(echo "$readme_dirs") 2>/dev/null || true)

if [ -n "$missing" ] || [ -n "$extra" ]; then
  echo ""
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "  📝 README 目录结构可能需要更新"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  if [ -n "$missing" ]; then
    echo "  缺少的目录:"
    while IFS= read -r dir; do
      [ -n "$dir" ] && echo "    - $dir"
    done <<< "$missing"
  fi
  if [ -n "$extra" ]; then
    echo "  多余的目录:"
    while IFS= read -r dir; do
      [ -n "$dir" ] && echo "    - $dir"
    done <<< "$extra"
  fi
  echo ""
  echo "  手动检查 README.md 的 Structure 章节是否需要更新"
  echo ""
fi

exit 0
```

- [ ] **Step 2: Make hook executable and verify syntax**

```bash
chmod +x hooks/sync-readme-check.sh
bash -n hooks/sync-readme-check.sh
echo "Syntax OK"
```

- [ ] **Step 3: Update .claude/settings.json**

Add sync-readme-check.sh to the Stop hooks array. The existing hook uses a matcher pattern — add the second hook alongside:

```json
{
  "hooks": {
    "Stop": [
      {
        "matcher": "gitflow",
        "hooks": [
          {
            "type": "command",
            "command": "bash hooks/auto-report-bug.sh"
          }
        ]
      },
      {
        "hooks": [
          {
            "type": "command",
            "command": "bash hooks/sync-readme-check.sh"
          }
        ]
      }
    ]
  }
}
```

`cat .claude/settings.json` to verify the file is valid JSON after edit.

```bash
python3 -c "import json; json.load(open('.claude/settings.json'))" && echo "JSON valid"
```

- [ ] **Step 4: Verify and commit**

```bash
git add hooks/sync-readme-check.sh .claude/settings.json
git commit -m "feat: add sync-readme-check hook + register in settings.json"
```

---

### Task 6: Final Validation — Spec vs Implementation

**Files:** all changed files from Tasks 1-5

**Interfaces:**
- Consumes: all prior tasks
- Produces: spec-compliant consolidated state

- [ ] **Step 1: Spec coverage check**

Walk through each section of `docs/superpowers/specs/2026-07-03-consolidation-design.md` and verify the corresponding change exists:

| Spec section | Task | Verify |
|---|---|---|
| 1.1 合规检查清单 | Task 2 Step 2-5 | grep 'Phase.*合规检查' skills/gitflow-workflow/SKILL.md |
| 1.2 --body-file 强制规则 | Task 2 Step 6-7 | grep 'body-file' skills/gitflow-workflow/SKILL.md |
| 1.3 强制执行规则头 | Task 2 Step 1 | grep '🚨' skills/gitflow-workflow/SKILL.md |
| 2.1 Pre-commit 第 6 步 | Task 1 Step 2 | grep '步骤 6' skills/gitflow-quality/SKILL.md |
| 2.2 Quality Report | Task 1 Step 3 | grep 'pre-commit' skills/gitflow-quality/SKILL.md |
| 3.1 - 3.4 weekly-report | Task 3 | gitflow-weekly-report/SKILL.md exists, contains -g install note, no scripts/ |
| 4.1 - 4.4 autoreport-bug | Task 4 | SKILL.md contains auth cache + dedup + failed.log, hook has auth cache check |
| 5.1 - 5.3 hooks | Task 5 | sync-readme-check.sh exists + registered |
| 5.4 auto-smoke-test 不迁 | — | hooks/ 中不存在 auto-smoke-test.sh |

- [ ] **Step 2: Validate all bash files**

Run syntax check on every bash file:

```bash
bash -n hooks/auto-report-bug.sh && echo "auto-report-bug.sh OK"
bash -n hooks/sync-readme-check.sh && echo "sync-readme-check.sh OK"
bash -n skills/_common.sh && echo "_common.sh OK"
```

- [ ] **Step 3: Validate all SKILL.md frontmatter**

Check every SKILL.md has valid YAML frontmatter (starts with `---`, contains `name:`):

```bash
for f in skills/gitflow-workflow/SKILL.md skills/gitflow-quality/SKILL.md skills/gitflow-weekly-report/SKILL.md skills/gitflow-autoreport-bug/SKILL.md; do
  head -1 "$f" | grep -q "^---$" && echo "$f: frontmatter OK" || echo "$f: frontmatter MISSING"
done
```

- [ ] **Step 4: Validate .claude/settings.json**

```bash
python3 -c "import json; d=json.load(open('.claude/settings.json')); assert 'sync-readme-check' in d['hooks']['Stop'][1]['hooks'][0]['command']; print('settings.json valid, both hooks registered')"
```

- [ ] **Step 5: Verify "不做的事" compliance**

Run the negative checks from the spec:

```bash
# Should NOT exist
test ! -f hooks/auto-smoke-test.sh && echo "auto-smoke-test correctly excluded" || echo "ERROR: auto-smoke-test found"

# weekly-report should NOT have scripts/ directory
test ! -d skills/gitflow-weekly-report/scripts && echo "weekly-report correctly has no scripts/" || echo "ERROR: unexpected scripts/ in weekly-report"
```

- [ ] **Step 6: Final commit — consolidation complete**

The 5 prior tasks committed atomically. No additional commit needed unless fixes were applied during validation. If any step needed a fix, commit now:

```bash
# Only if fixes were needed
git add -A
git commit -m "fix: address validation findings in consolidation"
```

Otherwise:

```bash
git log --oneline -6
# Verify 5 task commits + any fix commits are present
```
