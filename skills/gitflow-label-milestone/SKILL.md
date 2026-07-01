---
name: gitflow-label-milestone
description: gitflow CLI 的 Label 和 Milestone 操作命令封装，支持仓库标签和里程碑的增删改查
---

# gitflow label / milestone

封装 `gitflow label` 和 `gitflow milestone` 命令族，用于管理仓库的标签（Label）和里程碑（Milestone）资源。

## Label 命令概览

| 子命令 | 说明 |
|--------|------|
| `create` | 创建新标签 |
| `list` | 列出仓库所有标签 |
| `edit` | 编辑已有标签 |
| `delete` | 删除指定标签 |

## Label 参数说明

### `gitflow label create`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `--name` | string | 是 | 标签名 |
| `--color` | string | 是 | 标签颜色（十六进制，如 `d73a4a`） |
| `--description` | string | 否 | 标签描述 |

### `gitflow label list`

无需额外参数。

### `gitflow label edit`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<name>` | string | 是 | 要编辑的标签名 |
| `--name` | string | 否 | 新标签名 |
| `--color` | string | 否 | 新颜色 |
| `--description` | string | 否 | 新描述 |

### `gitflow label delete`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<name>` | string | 是 | 要删除的标签名 |

## Milestone 命令概览

| 子命令 | 说明 |
|--------|------|
| `create` | 创建新里程碑 |
| `list` | 列出仓库里程碑 |
| `edit` | 编辑已有里程碑 |
| `close` | 关闭指定里程碑 |
| `reopen` | 重新打开已关闭的里程碑 |

## Milestone 参数说明

### `gitflow milestone create`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `--title` | string | 是 | 里程碑标题 |
| `--description` | string | 否 | 里程碑描述（Markdown） |
| `--due-on` | string | 否 | 截止日期（ISO 8601 格式） |

### `gitflow milestone list`

无需额外参数。

### `gitflow milestone edit`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<number>` | int | 是 | 里程碑编号 |
| `--title` | string | 否 | 新标题 |
| `--description` | string | 否 | 新描述 |
| `--due-on` | string | 否 | 新截止日期 |

### `gitflow milestone close`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<number>` | int | 是 | 里程碑编号 |

### `gitflow milestone reopen`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<number>` | int | 是 | 里程碑编号 |

## 使用示例

### 创建项目标签

```bash
gitflow label create --name bug --color d73a4a --description "Something isn't working"
gitflow label create --name enhancement --color a2eeef --description "New feature or request"
```

### 编辑已有标签的颜色和描述

```bash
gitflow label edit bug --color ff0000 --description "Confirmed defect"
```

### 创建版本里程碑

```bash
gitflow milestone create --title "v1.0 Release" --description "First stable release" --due-on 2026-06-01T00:00:00Z
```

### 关闭里程碑并查看列表

```bash
gitflow milestone close 1
gitflow milestone list
```
