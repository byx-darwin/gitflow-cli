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
   - `id` — 错误唯一标识（必填）
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
  - 环境信息（timestamp、id）

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
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "command": "gitflow issue create",
  "platform": "github",
  "error_code": "401",
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
