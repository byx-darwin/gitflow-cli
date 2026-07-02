---
name: gitflow-issue
description: gitflow-cli 的 Issue 操作命令封装，支持创建、列表、查看、关闭、重新打开、评论和标签管理
---

# gitflow-cli issue

封装 `gitflow-cli issue` 命令族，用于在 GitHub/GitLab/GitCode 等平台上管理 Issue。

## 命令概览

| 子命令 | 说明 |
|--------|------|
| `create` | 创建新 Issue |
| `list` | 列出仓库的 Issue 列表 |
| `view` | 查看指定 Issue 的详情 |
| `close` | 关闭指定 Issue |
| `reopen` | 重新打开已关闭的 Issue |
| `comment` | 在 Issue 上添加评论 |
| `label` | 管理 Issue 的标签（添加/移除） |

## 参数说明

### `gitflow-cli issue create`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `--title` | string | 是 | Issue 标题 |
| `--body` | string | 否 | Issue 正文（Markdown） |
| `--label` | string | 否 | 附加标签名，可多次使用 |
| `--assignee` | string | 否 | 指派人登录名，可多次使用 |

### `gitflow-cli issue list`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `--state` | string | 否 | 按状态过滤：`open`、`closed`、`all` |
| `--label` | string | 否 | 按标签名过滤，可多次使用 |
| `--assignee` | string | 否 | 按指派人过滤 |
| `--search` | string | 否 | 关键字搜索 |
| `--limit` | int | 否 | 返回数量上限，默认 30 |

### `gitflow-cli issue view`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<number>` | int | 是 | Issue 编号 |

### `gitflow-cli issue close`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<number>` | int | 是 | Issue 编号 |

### `gitflow-cli issue reopen`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<number>` | int | 是 | Issue 编号 |

### `gitflow-cli issue comment`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<number>` | int | 是 | Issue 编号 |
| `--body` | string | 是 | 评论内容（Markdown） |

### `gitflow-cli issue label`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<number>` | int | 是 | Issue 编号 |
| `--add` | string | 否 | 要添加的标签名，可多次使用 |
| `--remove` | string | 否 | 要移除的标签名，可多次使用 |

## 使用示例

### 创建带标签和指派人的 Issue

```bash
gitflow-cli issue create --title "Fix login redirect loop" --body "Auth middleware 缺少重定向检查" --label bug --label high-priority --assignee alice
```

### 列出所有已关闭的 bug 类 Issue

```bash
gitflow-cli issue list --state closed --label bug --limit 50
```

### 查看 Issue 详情并添加评论

```bash
gitflow-cli issue view 42
gitflow-cli issue comment 42 --body "已复现，正在排查 middleware 逻辑"
```

### 关闭 Issue 并移除标签

```bash
gitflow-cli issue close 42
gitflow-cli issue label 42 --remove high-priority
```
