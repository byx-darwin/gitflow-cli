# GitLab 非 issue 命令族 `--repo`/`--project` 目标修复设计（Issue #275）

- **状态**：已批准
- **关联 Issue**：#275（PR #274 / Issue #267 的后续跟进）
- **分类**：Bounded（复用 #267 已验证的既有模式，跨多个既有文件重复应用）

## 背景

PR #274 修复了 `gf issue` 命令族在自建 GitLab 实例上因 `--repo` 裸 `owner/repo`（不含 host）导致的执行失败，引入了 `repo`/`repo_target` 字段拆分 + `with_remote_url()` 构造器模式。PR #274 的代码审查指出该修复范围只限于 `issue` 命令族，其他 GitLab 命令族可能有同样问题。

## 调研结论

逐一核对 `crates/gitlab/src/{mr,release,review,commit,pipeline,label}.rs` 后，实际受影响的 provider：

| 文件 | Provider | 受影响 | 说明 |
|---|---|---|---|
| `mr.rs` | `GitLabMrProvider` | ✅ | 8 处 `--repo` 均为裸 `self.repo`（create/list/view/close/reopen/merge/checkout/rebase） |
| `release.rs` | `GitLabReleaseProvider` | ✅ | 7 处 `--repo` 均为裸（create ×2/list/view/upload/download/delete） |
| `pipeline.rs` | `GitLabPipelineProvider` | ✅ | 2 处 `--repo`（`ci list`/`ci trace`） |
| `label.rs::GitLabLabelProvider` | | ✅ | 4 处 `--repo`（list/create/edit/delete） |
| `label.rs::GitLabMilestoneProvider` | | ✅（不同 flag） | 5 处 `--project`（create/list/edit/close/reopen），`glab milestone` 用 `--project` 而非 `--repo`，但语义等价，同样是裸值 |
| `commit.rs` | `GitLabCommitProvider` | ❌ 不需要改 | 全是 `glab api` REST 调用（`encode_project_path`），从不传 `--repo`/`--project`，裸 `repo` 本来就是正确用法 |
| `review.rs` | `GitLabReviewProvider` | ❌ 不需要改 | `glab mr approve`/`revoke` 代码里明确注释"NO --repo"，完全靠 cwd git remote 自动探测——这正是 #267 里"手动执行能成功"的那种模式，不是 bug |

## 修复方案

对 `mr.rs`、`release.rs`、`pipeline.rs`、`label.rs`（两个 provider）逐一应用 #267 已验证的模式：

1. 各 provider struct 新增 `repo_target: String` 字段（`GitLabMilestoneProvider` 命名为 `project_target`，因为对应的 flag 是 `--project` 不是 `--repo`，语义更贴切）；`repo` 字段保留，继续供 REST 路径编码（`encode_project_path`）使用。
2. 各 `new()`/`with_session()`/`with_runner()` 保持向后兼容：`repo_target`（或 `project_target`）默认等于 `repo`。
3. 各新增 `with_remote_url(repo, remote_url)` 构造器（`RealCommandRunner` 专用）+ `with_runner_and_repo_target(repo, repo_target, runner)`（测试专用，泛型 runner）。
4. 所有 `--repo`/`--project` 传参统一改用新字段。
5. `mr.rs::create` 已有的 `args.repo` 用户显式覆盖逻辑（`CreatePrArgs.repo: Option<String>`）与 #267 里 `IssueCommand::Create { repo: Some(_) }` 语义一致：用户显式覆盖仓库时，不应强行拼接 `remote_url`。

CLI 层：
- `apps/cli/src/main.rs::router()` 把已经存在的 `remote_url: &str` 参数也传给 `commands::pr::handle`、`commands::release::handle`、`commands::pipeline::handle`、`commands::label::handle_label`、`commands::label::handle_milestone`。
- `commands::commit::handle`、`commands::review::handle` 不需要改（对应 provider 不受影响，跳过）。
- 每个受影响 handler 内的 GitLab 分支比照 `commands/issue.rs:191-202` 的判断逻辑：无用户显式仓库覆盖时用 `with_remote_url`，否则退回裸 `new()`。

## 测试计划（TDD）

- 每个受影响 provider：新增测试断言 `repo_target`/`project_target` 通过 `with_remote_url` 正确出现在 `--repo`/`--project` CLI 参数位置（至少覆盖 1-2 个代表性动词，仿照 #267 对 `add_labels`/`remove_label`/`view` 的覆盖方式）。
- 既有测试（断言裸 `owner/repo` 的用例）保持不动、不回归——`repo_target` 默认等于 `repo`。
- CLI 层：每个受影响 handler 新增一个纯函数判断测试（比照 `should_use_remote_url_for_gitlab`），覆盖有/无用户仓库覆盖两个分支。
- `commit.rs`/`review.rs` 不新增测试（未改动）。

## 影响范围

- 5 个 provider 文件改动：`crates/gitlab/src/{mr,release,pipeline,label}.rs`（label.rs 含两个 provider）。
- 6 个 CLI 层文件改动：`apps/cli/src/main.rs`（router 分支）+ `apps/cli/src/commands/{pr,release,pipeline,label}.rs`。
- 不改动 `commit.rs`、`review.rs`、`crates/gitlab/src/error.rs`（#267 已修）。
- 不涉及 `deny.toml`/`.pre-commit-config.yaml`/`rust-toolchain.toml`。
