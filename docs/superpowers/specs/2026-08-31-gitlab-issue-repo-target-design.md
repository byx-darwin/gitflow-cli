# GitLab Issue 命令 `--repo` 目标修复设计（Issue #267）

- **状态**：已批准
- **关联 Issue**：#267
- **分类**：Bounded（既有代码路径内的定向修复）

## 背景

`gf issue add-label` 在自建 GitLab（Work Items 模型，如
`192.168.230.23`）上报错 `GitLab CLI 执行失败`，但直接手动执行等价的
`glab issue update <iid> --label <label>`（不带 `--repo`）完全成功。

## 排查结论

1. `--repo owner/repo` 的多层 group 路径拼接逻辑本身正确（已有测试覆盖），不是 bug。
2. Work Items URL（`/-/work_items/<n>`）的解析已经兼容，issue 作者最初的猜测方向不对。
3. **确认的诊断缺口**：`glab` 失败时的原始 stderr（`PlatformCliError.raw_stderr`）被捕获，但从未写入日志——文档注释声称"仅用于 `tracing::debug!`"，实际全仓库无一处调用。导致目前完全无法得知 `glab` 的真实报错原因。
4. **根因假设**（基于 GitLab CLI 官方文档 + 已知 issue gitlab-org/cli#1370）：`--repo OWNER/REPO`（裸形式，不含 host）不保证复用 cwd 的 git remote 做 host 探测；而用户手动执行时完全不传 `--repo`，纯靠 cwd 自动探测 host+repo。这很可能是自建实例上行为不一致的根因。`glab` 的 `--repo` 同时也接受"完整 URL / Git URL"形式，可用于显式锁定 host。

**局限**：沙箱内无法连接到真实的自建 GitLab 实例复现，此根因假设未经实测验证，属于基于文档的最佳推断。建议合并后由用户在真实环境验证。

## 修复方案

### 1. `crates/gitlab/src/issue.rs`

- `GitLabIssueProvider` 新增 `repo_target: String` 字段，专用于 `glab issue ...` 子命令的 `--repo` 值。
- 原 `repo` 字段保留，继续供 `encode_project_path(&self.repo)`（REST notes 调用路径）使用——两者语义不同，不可合并。
- `new()` / `with_runner()` 保持现状：`repo_target = repo.clone()`（向后兼容，裸 `owner/repo`）。
- 新增 `with_remote_url(repo, remote_url)` 构造器：`repo_target = remote_url`（完整 git remote URL，`glab` 官方文档确认 `--repo` 接受该形式）。
- 所有 11 处 `--repo` 传参（`add_labels` / `remove_label` / `view` / `close` / `reopen` / `edit` / `create` / `list` / `label create`）统一改用 `&self.repo_target`——一次性修复同类问题，而非只修 label。

### 2. `crates/gitlab/src/error.rs`

- `parse_glab_error` 入口处新增 `tracing::debug!(raw_stderr = %text, "glab command failed")`，把当前被丢弃的真实 stderr 记录下来。此项独立于根因假设，确定有效。

### 3. `apps/cli/src/main.rs`

- `resolve_platform()` 返回值从 `(platform, repo)` 扩展为 `(platform, repo, remote_url)`。
- `async_main` 签名相应扩展，透传 `remote_url`。

### 4. `apps/cli/src/commands/issue.rs`

- `handle()` 新增 `remote_url: &str` 参数。
- 仅当 `platform == "gitlab"` 且用户未通过 `--repo` CLI 参数覆盖仓库时，使用 `GitLabIssueProvider::with_remote_url(effective_repo, remote_url)`；否则退回 `GitLabIssueProvider::new(effective_repo)`（`--repo` 覆盖场景没有对应的 remote_url，不应强凑）。

## 测试计划（TDD）

- `repo_target` 在 `add_labels` / `remove_label` 中被正确用作 `--repo` 值（新增测试，RED → GREEN）。
- `new()` / `with_runner()` 路径保持裸 `owner/repo`，不回归（既有测试保持通过）。
- `encode_project_path` 仍使用 `self.repo`（notes 相关测试不受影响）。
- `parse_glab_error` 通过一个不引入新依赖的轻量自定义 `tracing::Subscriber` 断言 `raw_stderr` 事件被记录。
- `resolve_platform` / `extract_repo_from_url` 相关测试补充 `remote_url` 返回值断言。
- `commands/issue.rs::handle` 补充：GitLab 平台且无 `--repo` 覆盖时使用 `with_remote_url`；有覆盖时使用 `new`。

## 影响范围

- 4 个文件：`crates/gitlab/src/issue.rs`、`crates/gitlab/src/error.rs`、`apps/cli/src/main.rs`、`apps/cli/src/commands/issue.rs`。
- 不涉及公共 API 破坏性变更之外的下游 crate（`GitLabIssueProvider` 仅在 `apps/cli/src/commands/issue.rs` 一处构造）。
- 不改动 `deny.toml` / `.pre-commit-config.yaml` / `rust-toolchain.toml`。
