---
name: gitflow-autoreport-bug
description: 自动分析 CLI 错误报告，去重检查后创建 GitHub/GitLab Issue，并清理 pending.json
---

# gitflow-autoreport-bug

自动处理 gitflow CLI 的错误报告：读取 `pending.json` → 分析错误上下文 → 去重检查 → 创建 Issue → 清理临时文件。

## 前置条件

- 当前目录位于 git 仓库中
- `pending.json` 文件存在于 `.cache/bug-reports/pending.json`
- `gitflow` CLI 已安装且可用

**前置检查：** 在执行任何步骤之前，先验证 `gitflow` CLI 是否可用：

```bash
command -v gitflow
```

- 失败 → 输出「gitflow CLI 未安装，请运行：cargo install gitflow-cli」，保留 `pending.json`，结束
- 成功 → 继续执行步骤

## 执行步骤

### Step 1: 读取 pending.json

1. 检查 `.cache/bug-reports/pending.json` 是否存在
   - 不存在 → 输出「无待处理的错误报告」，结束
2. 读取并解析 JSON，提取以下字段：
   - `error_id` — 错误唯一标识
   - `command` — 失败的 CLI 命令
   - `platform` — 目标平台（github / gitlab / gitcode）
   - `error_code` — 错误代码
   - `error_message` — 错误信息
   - `timestamp` — 错误发生时间
3. 检查 gitflow 认证状态：

```bash
gitflow auth status --platform {platform}
```

   - 失败 → 保留 `pending.json` + 追加记录到 `.cache/bug-reports/failed.log`：

```bash
echo "[{timestamp}] 命令: {command} | 平台: {platform} | 错误: {error_code} | 失败原因: auth 检查失败" >> .cache/bug-reports/failed.log
```

   输出「认证失败，已记录到 failed.log，pending.json 保留待后续重试」，结束
   - 成功 → 继续 Step 2

**异常处理：** 如果 JSON 格式无效（缺少必要字段或语法错误），将文件重命名为 `pending.json.invalid`，输出警告信息，结束。

### Step 2: Claude 分析

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

### Step 3: 去重检查

1. 构造搜索关键词：`[auto-report] {command} {error_code}`
2. 调用 `gitflow issue list` 搜索已有 Issue：

```bash
gitflow issue list --search "[auto-report] {command} {error_code}" --state all
```

3. 判断结果：
   - **匹配到已有 Issue** → 跳过创建，删除 `pending.json`，输出「已存在相似报告: {url}」，结束
   - **未匹配** → 继续 Step 4

### Step 4: 创建 Issue

调用 `gitflow issue create` 创建 Issue：

```bash
gitflow issue create \
  --title "[auto-report] gitflow {command} — {error_code}" \
  --body "<Step 2 生成的分析报告>" \
  --label "bug" \
  --label "auto-reported" \
  --label "{platform}" \
  --label "{error_code}"
```

**成功路径：** 删除 `.cache/bug-reports/pending.json`，继续 Step 5。

**失败路径：** 如果 `gitflow issue create` 返回非零退出码或超时：

1. 保留 `pending.json`（不删除）
2. 追加失败记录到 `.cache/bug-reports/failed.log`：

```bash
echo "[{timestamp}] 命令: {command} | 平台: {platform} | 错误: {error_code} | 失败原因: Issue 创建失败" >> .cache/bug-reports/failed.log
```

3. 输出「Issue 创建失败，已记录到 failed.log，pending.json 保留待后续重试」，结束

### Step 5: 输出结果

- **创建成功：** 输出 Issue 链接和简要摘要
- **重复跳过：** 输出「已存在相似报告: {url}」

## 异常处理

| 场景 | 处理方式 |
|------|----------|
| `gitflow` CLI 未安装 | 输出安装提示 + 保留 `pending.json` + 结束 |
| `gh auth` 失败 | 保留 `pending.json` + 记录到 `failed.log` |
| Issue 创建失败 | 保留 `pending.json` + 记录到 `failed.log` |
| `pending.json` 格式无效 | 重命名为 `.invalid` 后缀 |
| 不在 git 仓库中 | 静默退出 |

### failed.log 格式

```
[{timestamp}] 命令: {command} | 平台: {platform} | 错误: {error_code} | 失败原因: {failure_reason}
```

## 示例输出

### 创建成功

```
✅ Bug 报告已创建: https://github.com/owner/repo/issues/123
   标题: [auto-report] gitflow issue create — AUTH_FAILED
   严重程度: high
```

### 重复跳过

```
⚠️ 已存在相似报告: https://github.com/owner/repo/issues/100
   跳过创建，pending.json 已清理
```
