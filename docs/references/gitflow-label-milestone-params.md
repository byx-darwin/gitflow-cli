# gitflow-cli label / milestone 完整参考

> 本文档为 `gitflow-label-milestone` skill 的外部化引用。

## Label 子命令

| 子命令 | 说明 |
|--------|------|
| `create` | 创建新标签 |
| `list` | 列出仓库所有标签 |
| `edit` | 编辑已有标签 |
| `delete` | 删除指定标签 |

### `label create`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `--name` | string | 是 | 标签名 |
| `--color` | string | 是 | 十六进制颜色（如 `d73a4a`） |
| `--description` | string | 否 | 标签描述 |

### `label edit`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<name>` | string | 是 | 要编辑的标签名 |
| `--name` | string | 否 | 新标签名 |
| `--color` | string | 否 | 新颜色 |
| `--description` | string | 否 | 新描述 |

### `label delete`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<name>` | string | 是 | 要删除的标签名 |

## Milestone 子命令

| 子命令 | 说明 |
|--------|------|
| `create` | 创建新里程碑 |
| `list` | 列出仓库里程碑 |
| `edit` | 编辑已有里程碑 |
| `close` | 关闭指定里程碑 |
| `reopen` | 重新打开已关闭的里程碑 |

### `milestone create`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `--title` | string | 是 | 里程碑标题 |
| `--description` | string | 否 | 里程碑描述（Markdown） |
| `--due-on` | string | 否 | 截止日期（ISO 8601） |

### `milestone edit`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<number>` | int | 是 | 里程碑编号 |
| `--title` | string | 否 | 新标题 |
| `--description` | string | 否 | 新描述 |
| `--due-on` | string | 否 | 新截止日期 |

### `milestone close` / `milestone reopen`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<number>` | int | 是 | 里程碑编号 |
