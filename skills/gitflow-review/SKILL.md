---
name: gitflow-review
description: gitflow CLI 的代码审查操作命令封装，支持评论、批准、要求修改和提交审查
---

# gitflow-cli review

封装 `gitflow-cli review` 命令族，用于在 GitHub/GitLab/GitCode 等平台上对 Pull Request 进行代码审查。

## 命令概览

| 子命令 | 说明 |
|--------|------|
| `comment` | 在 PR 上发表评论（不表态） |
| `approve` | 批准 PR，可以合并 |
| `request-changes` | 要求修改后才能合并 |
| `submit` | 提交一次完整的审查结论 |

## 参数说明

### `gitflow-cli review comment`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<pr-number>` | int | 是 | PR 编号 |
| `--body` | string | 是 | 评论内容（Markdown） |

### `gitflow-cli review approve`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<pr-number>` | int | 是 | PR 编号 |
| `--body` | string | 否 | 批准说明（可选） |

### `gitflow-cli review request-changes`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<pr-number>` | int | 是 | PR 编号 |
| `--body` | string | 是 | 修改要求的说明 |

### `gitflow-cli review submit`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<pr-number>` | int | 是 | PR 编号 |
| `--event` | string | 是 | 审查结论：`approved`、`changes_requested`、`commented` |
| `--body` | string | 否 | 审查总结说明（可选） |

## 使用示例

### 批准 PR

```bash
gitflow-cli review approve 101 --body "代码结构清晰，测试覆盖完整，LGTM!"
```

### 要求修改

```bash
gitflow-cli review request-changes 101 --body "auth.rs 第 42 行建议使用 Result 类型替代 Option"
```

### 发表中立评论

```bash
gitflow-cli review comment 101 --body "建议参考 issue #42 的 spec 文档"
```

### 提交完整审查（批量操作后一次性提交）

```bash
# 先添加多条行内评论后，一次性提交审查结论
gitflow-cli review submit 101 --event approved --body "全面审查通过，所有修改点已处理"
```
