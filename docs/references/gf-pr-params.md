# gf pr 子命令参数完整参考

> 本文档为 `gf-pr` skill 的子命令参数速查，由 SKILL.md 主文档外部化引用。

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
| `--auto` | bool | 否 | 排队合并：由平台在必需检查/pipeline 通过后自动合并，调用立即返回，不必等待 CI。默认 `false` |

> **`--auto` 平台支持**：GitHub（`gh pr merge --auto`，需仓库开启 *Allow auto-merge*）与 GitLab（`glab mr merge --auto-merge`）支持；GitCode 无排队合并语义，传 `--auto` 返回 `CoreError::Platform` 且不执行任何 CLI 调用。
>
> **返回语义**：`--auto` 成功时 `merged` 为 `false`（仅表示已排期，合并尚未落地），`message` 携带平台原文。

## `pr checkout`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<number>` | int | 是 | PR 编号 |

## `pr ready` / `pr wip` / `pr sync`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<number>` | int | 是 | PR 编号 |
