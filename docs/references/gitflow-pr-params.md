# gitflow-cli pr 子命令参数完整参考

> 本文档为 `gitflow-pr` skill 的子命令参数速查，由 SKILL.md 主文档外部化引用。

## `pr create`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `--title` | string | 是 | PR 标题 |
| `--body` | string | 否 | PR 正文（Markdown） |
| `--head` | string | 是 | 来源分支名 |
| `--base` | string | 是 | 目标分支名 |
| `--draft` | flag | 否 | 以草稿方式创建 |
| `--repo` | string | 否 | 目标仓库（`owner/name` 格式） |

## `pr list`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `--state` | string | 否 | 按状态过滤：`open`/`closed`/`merged`/`all` |
| `--limit` | int | 否 | 返回数量上限，默认 30 |

## `pr view`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<number>` | int | 是 | PR 编号 |

## `pr close`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<number>` | int | 是 | PR 编号 |

## `pr reopen`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<number>` | int | 是 | PR 编号 |

## `pr comment`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<number>` | int | 是 | PR 编号 |
| `--body` | string | 是 | 评论内容（Markdown） |

## `pr merge`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<number>` | int | 是 | PR 编号 |
| `--strategy` | string | 否 | 合并策略：`merge`/`squash`/`rebase` |

## `pr checkout`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<number>` | int | 是 | PR 编号 |

## `pr ready` / `pr wip` / `pr sync`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<number>` | int | 是 | PR 编号 |
