# Design: GitLab 嵌套 group 项目路径 `%2F` 编码不全导致 404

- **Date:** 2026-08-19
- **Issue:** [#219](https://github.com/byx-darwin/gitflow-cli/issues/219)
- **Mode:** standard
- **Workflow:** `wf-2026-08-19-002`

## Problem

在嵌套 group 项目（如 `iproost/proxy/edge`，路径 3 段）上，`gf issue comment` / `gf issue comments`
报 `GitLab CLI 执行失败`（底层 `glab api` 404 Not Found）。

根因：`crates/gitlab/src/issue.rs` 的 `comment()` 与 `list_comments()` 手拼 `glab api` 路径时，
只对 owner 与 project 之间那一层 `/` 做了 `%2F` 编码，project 内部的 `/` 未编码：

```rust
let (owner, project) = self.repo.split_once('/')...;   // iproost/proxy/edge → owner=iproost, project=proxy/edge
let api_path = format!("/projects/{owner}%2F{project}/issues/{number}/notes");
// → /projects/iproost%2Fproxy/edge/issues/3/notes   ← proxy/edge 中间的 / 未编码
```

GitLab API 要求整个项目路径作为一个 URL 段全量编码：
`/projects/iproost%2Fproxy%2Fedge/...`。未编码的 `/` 使 API 解析成项目 `iproost/proxy` + 多余段 `edge` → 404。

## Scope

同一根因模式在 4 个模块共 5 处手拼 `glab api` 路径中复现，全部需要修复：

| Site | File:line | Path 形态 |
|---|---|---|
| `issue.comment()` | `crates/gitlab/src/issue.rs:449` | `/projects/{owner}%2F{project}/issues/{n}/notes` |
| `issue.list_comments()` | `crates/gitlab/src/issue.rs:489` | `/projects/{owner}%2F{project}/issues/{n}/notes` |
| `mr.comment()` | `crates/gitlab/src/mr.rs:382` | `/projects/{owner}%2F{project}/merge_requests/{n}/notes` |
| `review.post_note()` | `crates/gitlab/src/review.rs:250` | `/projects/{owner}%2F{project}/merge_requests/{n}/notes` |
| `pipeline.jobs()` | `crates/gitlab/src/pipeline.rs:261` | `/projects/{owner}%2F{project}/pipelines/{id}/jobs` |

## Solution

复用已有共享辅助函数 `encode_project_path`（`crates/gitlab/src/commit.rs:190`，`pub(crate)`，
实现正是 `repo.replace('/', "%2F")`），替换各处的 `split_once('/')` + 部分编码逻辑：

```rust
use crate::commit::encode_project_path;

let encoded = encode_project_path(&self.repo);   // "group/subgroup/project" → "group%2Fsubgroup%2Fproject"
let api_path = format!("/projects/{encoded}/issues/{number}/notes");
```

- 移除各处的 `split_once('/')` 校验分支：repo 形状由 `GitLabIssueProvider::new("owner/project")`
  构造保证；畸形 repo 由 `glab` 自身报错。
- 无公共 API 变更：`encode_project_path` 保持 `pub(crate)`。

## Testing (TDD: RED → GREEN)

- 单测：repo 为 3 段路径（`group/subgroup/project`）时断言生成的 `glab api` 路径为
  `/projects/group%2Fsubgroup%2Fproject/issues/42/notes`（及 `merge_requests` / `pipelines` 变体）。
- 集成：`MockCommandRunner` 记录传给 `glab` 的 argv，验证嵌套 group repo 下参数正确
  （镜像现有 `test_should_post_issue_note_via_glab_api_with_message_field`）。

## Files Touched

- `crates/gitlab/src/issue.rs`（+ tests）
- `crates/gitlab/src/mr.rs`（+ tests）
- `crates/gitlab/src/review.rs`（+ tests）
- `crates/gitlab/src/pipeline.rs`（+ tests）
