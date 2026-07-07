# gitflow-cli pr Subcommand Parameter Reference

> **Companion to:** `skills/gitflow-pr/SKILL.md`
> **Purpose:** Full parameter tables for all 11 `pr` subcommands. The parent skill embeds only the Quick Reference; this file holds the detailed reference to keep the parent skill within the 500-word budget.

---

## `gitflow-cli pr create`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `--title` | string | Yes | PR title |
| `--body` | string | No | PR body (Markdown) |
| `--head` | string | Yes | Source branch name |
| `--base` | string | Yes | Target branch name |
| `--draft` | flag | No | Create as draft |
| `--repo` | string | No | Target repo (`owner/name`) |

## `gitflow-cli pr list`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `--state` | string | No | Filter: `open`, `closed`, `merged`, `all` |
| `--limit` | int | No | Max results (default 30) |

## `gitflow-cli pr view`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `<number>` | int | Yes | PR number |

## `gitflow-cli pr close`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `<number>` | int | Yes | PR number |

## `gitflow-cli pr reopen`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `<number>` | int | Yes | PR number |

## `gitflow-cli pr comment`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `<number>` | int | Yes | PR number |
| `--body` | string | Yes | Comment text (Markdown) |

## `gitflow-cli pr merge`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `<number>` | int | Yes | PR number |
| `--strategy` | string | No | `merge`, `squash`, or `rebase` |

## `gitflow-cli pr checkout`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `<number>` | int | Yes | PR number |

## `gitflow-cli pr ready`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `<number>` | int | Yes | PR number |

## `gitflow-cli pr wip`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `<number>` | int | Yes | PR number |

## `gitflow-cli pr sync`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `<number>` | int | Yes | PR number |
