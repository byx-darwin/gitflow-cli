# GitLab glab 1.113 Compatibility Fix — Design

> **Workflow:** `wf-2026-08-18-001` · **Issue:** [#199](https://github.com/byx-darwin/gitflow-cli/issues/199)
> **Mode:** standard · **Skill source:** superpowers
> **Date:** 2026-08-18

## 1. Problem Statement

gf 1.3.0 + glab 1.113.0 下，GitLab **写操作几乎全部失败**。根因是 gf 内部调用
glab 的参数/子命令与 glab 1.113 不兼容，而非未认证。完整测试记录见
`scripts/gf-gitlab-compat-test-report.md`（位于 iproost-docs 仓库）。

**已验证事实（本地 `glab 1.113.0` 实测 + 自建 GitLab `192.168.230.23`）：**

| gf 调用 | glab 1.113 实际情况 | 判定 |
|---|---|---|
| `issue close/reopen/note` + `--output json` | 无 `--output` flag | ❌ Unknown flag |
| `label create` + `--output json` | 无 `--output` flag | ❌ |
| `mr create/close/reopen/note` + `--output json` | 无 `--output` flag | ❌ |
| `milestone create/close` + `--output json` | 无 `--output` flag | ❌ |
| `release create` + `--output json` | 无 `--output` flag | ❌ |
| `ci view` + `--output json` | 无 `--output` flag | ❌ |
| `glab auth token` | 子命令不存在 | ❌ 返回帮助文本 |
| `glab mr ready <n>` | 子命令不存在 | ❌ |
| `glab mr draft <n>` | 子命令不存在 | ❌ |
| `glab label edit --name X` | 要求 `--label-id` + `--new-name` | ❌ |
| `glab label delete --yes` | 无 `--yes` | ❌ |
| `glab issue note --body` | 参数为 `--message`（`-m`） | ❌ |
| `glab mr note --body` | 参数为 `--message`（`-m`） | ❌ |
| `glab mr merge --merge` | 无 `--merge` flag（默认即 merge） | ❌ |
| `parse_issue_iid_from_url` 只匹配 `/issues/N` | 新版返回 `/-/work_items/N` | ❌ 解析失败 |
| `pr list --state all` | glab 用 `--all` | ❌ Invalid state |

**回归基线（多走 `glab api`，必须不回归）：** `issue list/view/comments`、
`commit view/diff/patch/comment`、`label/milestone/release list`、`pipeline status/report`、
`workflow` 全套、`doctor`、`auth status`。

## 2. 范围（Scope）

**纳入：**
1. 写操作 `--output json` 策略修正
2. 不存在的子命令/参数替换
3. `parse_issue_iid_from_url` 兼容 `/work_items/N`
4. note 参数名 `--body` → `--message`
5. 错误处理/退出码统一
6. 配套单元测试 + 本地真实 GitLab 实测

**明确推迟（不在本次）：**
- `gf repo` 子命令缺失但存在 `gf-repo` 技能（技能与 CLI 不一致）→ 单开 issue
- 任何 GitHub / GitCode 平台行为变更
- glab 版本兼容性矩阵（`docs/compatibility-matrix.md`）如需要另行更新

## 3. 设计原则

- **写操作不追加 `--output json`**；如需 JSON 数据，写完后用现有 `view`/`list`
  重新拉取（复用 `issue create → view` 既有模式）。
- **读操作保留 `--output json`**（`issue list/view`、`mr list/view`、
  `release list/view`、`ci list`、`label list`）。
- **回归基线不动**：凡已走 `glab api` 的路径保持原样。
- 单元测试用 `MockCommandRunner` 断言精确的 glab 参数序列（成功/失败双路径）。

## 4. 详细设计

### 4.1 写操作 `--output json` 策略

**移除 `--output json` 的位置：**

| 文件 | 位置 | 动作 |
|---|---|---|
| `crates/gitlab/src/issue.rs` | `close`/`reopen`/`note` | 移除 `--output json` |
| `crates/gitlab/src/mr.rs` | `create`/`close`/`reopen`/`note` | 移除 |
| `crates/gitlab/src/label.rs` | `create`/`edit`/`delete` | 移除 |
| `crates/gitlab/src/release.rs` | `create`/`edit` | 移除 |
| `crates/gitlab/src/pipeline.rs` | `ci view`/`ci trace` | 移除 |
| `crates/gitlab/src/review.rs` | `mr note`（comment / request changes） | 移除 |
| `crates/gitlab/src/milestone.rs`（若存在） | `create`/`close` | 移除 |

**保留 `--output json` 的读操作：** `issue list`、`issue view`、`mr list`、
`mr view`、`release list`、`release view`、`ci list`、`label list`。

**返回实体的写操作**：写成功（exit 0）后经现有 `view`/`list` 重新拉取实体，
不解析写命令自身的 stdout。

### 4.2 不存在的子命令/参数替换

| 文件/函数 | 现状 | 改为 |
|---|---|---|
| `auth.rs` `token()` | `glab auth token` | `glab auth status --show-token`（支持 `--hostname <host>` 定位实例），从 stdout 解析 token 行 |
| `mr.rs` `ready()` | `glab mr ready <n>` | `glab mr update <n> --draft=false` |
| `mr.rs` `wip()` | `glab mr draft <n>` | `glab mr update <n> --draft=true` |
| `label.rs` `edit()` | `glab label edit --name <name> ...` | 先 `label list` 解析 label-id → `glab label edit --label-id <id> --new-name <name> --color ... --description ...` |
| `label.rs` `delete()` | `glab label delete <name> --yes` | `glab label delete <name>`（去 `--yes`） |

**`auth token` 解析**：`glab auth status --show-token` 输出为多实例文本块，
需定位目标 host 的 `Token found ...: <token>` 行。目标 host 从 `self.repo` 或
当前实例上下文解析；无 token 行 → 返回错误（不再把帮助文本当 token）。

### 4.3 URL 解析 `/work_items/N`

`crates/gitlab/src/issue.rs` `parse_issue_iid_from_url`：
- 同时匹配 `/-/issues/N` 与 `/-/work_items/N`。
- 实现：按行扫描，逐行尝试两种模式；任一命中即返回 IID。
- 补充单测：work item URL 样例 `http://192.168.230.23/iproost/iproost-docs/-/work_items/1`。

### 4.4 note 参数名

| 文件/位置 | 现状 | 改为 |
|---|---|---|
| `issue.rs` `note` | `--body` | `--message` |
| `mr.rs` `note` | `--body` | `--message` |
| `review.rs` comment ×2 | `--body` | `--message` |

### 4.5 错误处理 / 退出码

**`crates/gitlab/src/error.rs`：**
- JSON 解析分支（`parse_glab_error`）：错误提示基于真实 `code`，仅
  `UNAUTHORIZED`/`NOT_FOUND` 等明确认证类错误提示登录。
- 纯文本回退分支：仅当 stderr 含 `not authenticated`/`401`/`Unauthorized`/`token`
  时才提示 `glab auth login`；否则提示真实 stderr 内容。
- 统一 hint 文案：认证类 → `运行 \`glab auth login\` 完成登录`；非认证类 → 展示原始错误。

**退出码语义**：写操作失败 → 非零退出码 + 具体错误信息（不吞错误、不误报成功）。
逐处检查 `issue create/reopen`、`label create`、`milestone create`、`release create`
的失败路径确保 `exit != 0`。

**`apps/cli/src/commands/issue.rs` / `pr.rs`：**
- `list --state` 接受 `all` → 映射为 glab 的 `--all`（issue 用 `--all`；pr 用 `--all`）。
- 现有 `Invalid state '{other}'. Expected 'open' or 'closed'` 校验扩展 `all`。

**`mr.rs` `merge()`：**
- 移除 `--merge`（glab 默认即 merge）；保留 `--squash`/`--rebase` 映射。
- `MergeStrategy::Merge` 与 `None` → 不再传任何 strategy flag。

### 4.6 受影响文件清单

```
crates/gitlab/src/auth.rs        # token() → auth status --show-token
crates/gitlab/src/error.rs       # 认证 vs 非认证错误提示分支
crates/gitlab/src/issue.rs       # close/reopen/note --output、note --message、URL 解析
crates/gitlab/src/label.rs       # create/edit/delete --output、edit --label-id、delete 去 --yes
crates/gitlab/src/mr.rs          # create/close/reopen/note --output、ready/wip → update --draft、merge 去 --merge
crates/gitlab/src/pipeline.rs    # ci view/trace 去 --output
crates/gitlab/src/release.rs     # create/edit 去 --output
crates/gitlab/src/review.rs      # mr note --body → --message、去 --output
apps/cli/src/commands/issue.rs   # list --state all → --all
apps/cli/src/commands/pr.rs      # list --state all → --all
```

（`milestone` 若存在独立模块，同样处理；当前源码中未发现单独 milestone 文件。）

## 5. 测试策略

### 5.1 单元测试（TDD，MockCommandRunner）

每个修复点断言**精确的 glab 参数序列**，含成功/失败双路径：

| 测试点 | 断言 |
|---|---|
| `issue close/reopen/note` | 无 `--output`；note 用 `--message` |
| `mr create/close/reopen/note` | 无 `--output`；note 用 `--message` |
| `mr ready/wip` | `mr update <n> --draft=false/true` |
| `mr merge` | 无 `--merge`；squash/rebase 保留 |
| `label edit` | `--label-id <id> --new-name ... --color ...` |
| `label delete` | 无 `--yes` |
| `release create/edit` | 无 `--output` |
| `ci view` | 无 `--output` |
| `auth token` | `auth status --show-token --hostname ...`；解析 token 行 |
| `parse_issue_iid_from_url` | `/work_items/N` 命中；`/issues/N` 不回归 |
| `error.rs` | 认证类提示登录；非认证类展示真实错误 |

### 5.2 本地真实 GitLab 实测（192.168.230.23，已登录）

**写操作冒烟：**
- `gf issue close/reopen/comment` ✓
- `gf label create/edit/delete` ✓
- `gf pr create/close/reopen/comment` ✓（或降级为 mr 相关最小验证）
- `gf release create` ✓
- `gf auth token` 返回真实 token ✓

**读操作回归：**
- `gf issue list/view/comments`、`gf commit view`、`gf workflow status`、`gf doctor` ✓

**失败路径：** 无权限写操作 → 非零退出码 + 具体错误（非「请登录」误导文案）。

### 5.3 验证命令

```bash
make build && make test && make fmt && make clippy
```

## 6. 验收标准映射

| Issue #199 验收项 | 对应修复 |
|---|---|
| `gf issue close/reopen/comment` 成功 | 4.1 + 4.4 |
| `gf label create/edit/delete`、`gf milestone create/close` 成功 | 4.1 + 4.2 |
| `gf pr create/close/reopen/comment/merge` 成功 | 4.1 + 4.2 + 4.4 |
| `gf release create` 成功 | 4.1 |
| `gf issue create` 不再误报失败 | 4.3 |
| `gf auth token` 返回真实 token | 4.2 |
| `gf pr ready/wip` 正常 | 4.2 |
| 失败命令退出码非零、不误导登录 | 4.5 |
| 回归基线不损坏 | 4.1 保留读操作 + 5.2 |

## 7. 风险与缓解

| 风险 | 缓解 |
|---|---|
| glab 1.113 行为与旧版差异未完全覆盖 | 实测优先；`--help` 全量核对受影响子命令 |
| 写操作去 `--output` 后无 JSON 回显 | 统一 `view`/`list` 重新拉取 |
| `auth status --show-token` 多实例输出解析复杂 | 用 `--hostname` 定位目标实例 |
| 自建 GitLab 写操作有副作用 | 使用独立测试 label/release 名称，测后清理 |
