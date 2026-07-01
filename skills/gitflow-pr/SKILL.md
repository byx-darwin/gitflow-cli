---
name: gitflow-pr
description: gitflow CLI 的 Pull Request 操作命令封装，支持创建、列表、查看、关闭、合并、检出、状态切换和分支同步
---

# gitflow pr

封装 `gitflow pr` 命令族，用于在 GitHub/GitLab/GitCode 等平台上管理 Pull Request。

## 命令概览

| 子命令 | 说明 |
|--------|------|
| `create` | 创建新 Pull Request |
| `list` | 列出仓库的 PR 列表 |
| `view` | 查看指定 PR 的详情 |
| `close` | 关闭指定 PR |
| `reopen` | 重新打开已关闭的 PR |
| `comment` | 在 PR 上添加评论 |
| `merge` | 合并指定 PR |
| `checkout` | 在本地检出 PR 分支 |
| `ready` | 将草稿 PR 标记为可审查 |
| `wip` | 将 PR 标记为草稿状态 |
| `sync` | 同步 PR 分支（将 base 合入 head） |

## 参数说明

### `gitflow pr create`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `--title` | string | 是 | PR 标题 |
| `--body` | string | 否 | PR 正文（Markdown） |
| `--head` | string | 是 | 来源分支名 |
| `--base` | string | 是 | 目标分支名 |
| `--draft` | flag | 否 | 以草稿方式创建 |
| `--repo` | string | 否 | 目标仓库（`owner/name` 格式） |

### `gitflow pr list`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `--state` | string | 否 | 按状态过滤：`open`、`closed`、`merged`、`all` |
| `--limit` | int | 否 | 返回数量上限，默认 30 |

### `gitflow pr view`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<number>` | int | 是 | PR 编号 |

### `gitflow pr close`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<number>` | int | 是 | PR 编号 |

### `gitflow pr reopen`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<number>` | int | 是 | PR 编号 |

### `gitflow pr comment`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<number>` | int | 是 | PR 编号 |
| `--body` | string | 是 | 评论内容（Markdown） |

### `gitflow pr merge`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<number>` | int | 是 | PR 编号 |
| `--strategy` | string | 否 | 合并策略：`merge`、`squash`、`rebase` |

### `gitflow pr checkout`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<number>` | int | 是 | PR 编号 |

### `gitflow pr ready`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<number>` | int | 是 | PR 编号 |

### `gitflow pr wip`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<number>` | int | 是 | PR 编号 |

### `gitflow pr sync`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<number>` | int | 是 | PR 编号 |

## 使用示例

### 创建功能分支的 PR

```bash
gitflow pr create --title "Add user authentication" --body "Implements login/logout per spec #42" --head feature/auth --base main
```

### 以草稿方式创建跨仓库 PR

```bash
gitflow pr create --title "WIP: experimental cache" --head feature/cache --base main --draft --repo org/shared-lib
```

### 查看 PR 并使用 squash 策略合并

```bash
gitflow pr view 101
gitflow pr merge 101 --strategy squash
```

### 检出 PR 分支并在审查后标记为就绪

```bash
gitflow pr checkout 55
# 本地审查完成后...
gitflow pr ready 55
```

### 同步过时的 PR 分支

```bash
gitflow pr sync 55
```
