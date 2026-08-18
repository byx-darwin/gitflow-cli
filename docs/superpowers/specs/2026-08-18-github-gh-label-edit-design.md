# GitHub gh 2.97 Label Edit 假失败修复 — Design

> **Workflow:** `wf-2026-08-18-002` · **Issue:** [#200](https://github.com/byx-darwin/gitflow-cli/issues/200)
> **Mode:** standard · **Skill source:** superpowers
> **Date:** 2026-08-18

## 1. Problem Statement

`gf label edit` 在 GitHub 平台上**每次执行都报错**，但**编辑实际已经生效**（假失败，false negative）。用户看到失败重试会反复覆盖写。完整检查报告见 `docs/gh-compat-check-2026-08-18.md`。

**已验证事实（gh 2.97 实测 + PATH 拦截探针）：**

| 调用 | 结果 |
|---|---|
| `gh label edit <name> --repo <repo> --color X --description Y` | ✅ 退出码 0，编辑生效 |
| `gh label view <name> --repo <repo> --json name,color,description` | ❌ 退出码 1，`unknown flag: --json` |

**根因：** `crates/github/src/label.rs` 的 `edit()` 成功后调用 `fetch_label(name)` 重新拉取数据；`fetch_label()` 调用 `gh label view <name> --json ...`，而 **gh 2.97 没有 `label view` 子命令**（Available: clone/create/delete/edit/list）→ 重新拉取失败 → 整个命令误报失败。

**兼容报告中的三个问题：**
- **P1（必修）**：`fetch_label` 改用 `gh api repos/{owner}/{repo}/labels/{name}`，删除对 `gh label view --json` 的调用。
- **P2（建议）**：`parse_gh_error` 错误信息错位 —— 所有失败统一提示「运行 `gh auth login`」，即使已登录。与 #199（GitLab 侧）同源。
- **P3（可选）**：`gh label list` 默认返回前 30 条，gf 未传 `--limit`。仓库 44 个 label 只显示 30 个。

## 2. 范围（Scope）

**纳入（P1 + P2 + P3，用户已确认）：**
1. **P1**：`fetch_label` 改用 `gh api repos/{owner}/{repo}/labels/{name}`；label 名 URL 编码。
2. **P1 配套**：`GitHubLabelProvider` 重构为 `CommandRunner` 泛型 + `with_runner()`，使命令参数可被单元测试断言（仿照 #199 双子修复 `GitLabLabelProvider` 与既有 `GitHubAuthProvider`）。
3. **P2**：`parse_gh_error` 仅在真实认证失败时提示 auth login；其余错误 hint 置 `None`。
4. **P3**：`list()` 加 `--limit 100`。
5. 配套单元测试（RED → GREEN → REFACTOR）。

**明确推迟（不在本次）：**
- `crates/gitcode/src/label.rs` 存在同款 `gh label view` 隐患（gc CLI 无 `label view` 与否待核实）→ 单开 issue 跟进。
- GitHub milestone 部分不改（已走 `gh api`，无问题）。
- P2 的 GitLab 侧修复已由 #199 完成，此处仅对齐 GitHub 侧。

## 3. 设计原则

- **重新拉取走 REST API**：`gh label view` 不存在 → `fetch_label` 改用 `gh api repos/{owner}/{repo}/labels/{name}`（REST 返回 JSON，含 name/color/description，与 `LabelData` 反序列化形状一致）。
- **命令可测性**：所有 `gh` 调用通过 `CommandRunner` 抽象，单元测试用 `MockCommandRunner` / `SequencedMockCommandRunner` 断言精确参数序列（成功/失败双路径）。
- **URL 编码**：label 名可能含空格/特殊字符，路径段按 RFC 3986 编码（仅保留 `A-Za-z0-9-._~`），不新增依赖。
- **错误信息诚实**：只有真实认证失败才提示「运行 `gh auth login`」。
- **调用点兼容**：`GitHubLabelProvider::new(repo)` 保持可用（默认类型参数 `RealCommandRunner`），`apps/cli` 无需改动。

## 4. 具体改动

### 4.1 `crates/github/src/label.rs` — P1 + P3 + runner 重构

1. `GitHubLabelProvider` 泛型化：`GitHubLabelProvider<R: CommandRunner = RealCommandRunner>`，新增 `with_runner(repo, runner)`。
2. `create` / `list` / `edit` / `delete` / `fetch_label` 全部改走 `self.runner.run("gh", &[...])`。
3. `fetch_label` 改为 `gh api` + URL 编码：
   ```rust
   let api_path = format!("repos/{repo}/labels/{name}", repo = self.repo, name = encode_path_segment(name));
   self.runner.run("gh", &["api", &api_path]).await...
   ```
4. `list` 加 `--limit 100`。
5. 私有辅助函数 `encode_path_segment(name: &str) -> String`（RFC 3986）。

### 4.2 `crates/github/src/error.rs` — P2

镜像 #199 的 GitLab 侧修复：
- 加 `is_auth_failure` 闭包（`not authenticated` / `unauthorized` / `401` / `token`）。
- JSON 路径：默认 hint 仅在 `is_auth_failure` 时设为「运行 `gh auth login`」。
- 纯文本回退路径：仅 `is_auth` 时设 auth hint；其余 `hint = None`（`unknown flag` 等 CLI 用法错误不再误导用户）。

## 5. 测试

| 测试 | 覆盖 |
|---|---|
| `test_should_fetch_label_via_gh_api` | runner 收到 `["api", "repos/owner/repo/labels/bug"]` + 返回 LabelData |
| `test_should_encode_label_name_in_api_path` | 空格 label → `labels/good%20first%20issue` |
| `test_should_edit_label_and_refetch_via_gh_api` | edit 成功 → api 重拉 → 返回最新数据（P1 回归主测试） |
| `test_should_fail_when_fetch_label_api_fails` | api 失败 → 真实报错（failure path） |
| `test_should_not_hint_auth_login_on_unknown_flag_error` | P2：`unknown flag` 不再提示 auth login |
| `test_should_hint_auth_login_on_not_authenticated_error` | P2：真实认证失败仍提示 |
| `test_should_list_labels_with_limit_flag` | P3：runner 收到 `--limit 100` |

测试约定：`test_should_<expected_behavior>` 命名，成功/失败双路径，用 runner 断言精确参数。

## 6. 验证

- `make test`（全量单元测试）
- `make lint`（clippy pedantic）
- `make fmt`
- 如可行，本地 PATH 拦截探针复验：`gf label edit` 不再出现 `gh label view`，且命令成功。

## 7. 关联

- #199（GitLab/glab 侧同源修复，已合并为 PR #201）
- `docs/gh-compat-check-2026-08-18.md`（完整检查报告）
- `docs/compatibility-matrix.md`（如需要另行更新）
