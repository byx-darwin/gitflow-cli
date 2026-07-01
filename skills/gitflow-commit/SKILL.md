---
name: gitflow-commit
description: gitflow CLI 的 Commit 操作命令封装，支持查看、差异比较、补丁导出和行内评论
---

# gitflow commit

封装 `gitflow commit` 命令族，用于在 GitHub/GitLab/GitCode 等平台上查看 Commit 详情、获取 diff/patch 以及添加行内评论。

## 命令概览

| 子命令 | 说明 |
|--------|------|
| `view` | 查看 Commit 详情（含变更文件列表） |
| `diff` | 获取 Commit 的统一 diff 输出 |
| `patch` | 获取 Commit 的原始 patch 内容 |
| `comment` | 在 Commit 的特定文件行上添加评论 |

## 参数说明

### `gitflow commit view`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<sha>` | string | 是 | Commit 的 SHA 哈希值 |

### `gitflow commit diff`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<sha>` | string | 是 | Commit 的 SHA 哈希值 |

### `gitflow commit patch`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<sha>` | string | 是 | Commit 的 SHA 哈希值 |

### `gitflow commit comment`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<sha>` | string | 是 | Commit 的 SHA 哈希值 |
| `--body` | string | 是 | 评论内容（Markdown） |
| `--path` | string | 是 | 文件路径 |
| `--line` | int | 是 | 行号（1-based） |

## 使用示例

### 查看 Commit 详情

```bash
gitflow commit view abc1234567890def
```

返回包含作者、提交信息、变更文件列表和行增减统计的完整数据。

### 获取 Commit 的 diff 并保存到文件

```bash
gitflow commit diff abc1234567890def > commit-abc.diff
```

### 获取 Patch 并应用到本地仓库

```bash
gitflow commit patch def0987654321fed | git apply -
```

### 在特定文件的特定行添加评论

```bash
gitflow commit comment abc1234567890def --body "这里建议使用 Result 替代 Option，能携带更多错误信息" --path "src/auth.rs" --line 42
```
