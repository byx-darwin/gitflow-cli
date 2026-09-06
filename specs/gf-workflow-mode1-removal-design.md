# gf-workflow Phase 3 Mode ① 移除设计（Issue #325）

## 背景

`skills/gf-workflow/references.md` → Phase 3 Execution Modes 把 **Mode ①（后台 agent，`isolation: worktree` + `run_in_background`）** 列为默认推荐执行方式。Issue #322 / wf-2026-09-04-005 执行时实测发现该模式在当前 Claude Code harness 下不可用，最终只能改用 Mode ③（同会话）手动完成。

## 实测结论（Issue #325 调查阶段，2026-09-06）

派发一个 `isolation: worktree` 的探测 agent，确认两条结构性缺陷：

1. **git 操作被限定在自身隔离 worktree 内。** 任何 `git -C <主仓库路径>`、`--git-dir`/`--work-tree`、`GIT_DIR`/`GIT_WORK_TREE` 指向主仓库或 `.worktree/<branch>` 固定路径的操作均被 harness 拒绝（普通文件读取如 `cat ../../../.cache/...` 不受影响，只有 git 命令被拦截）。
2. **fork 基点固定，无法指定 `base_branch`。** 该 agent 的 worktree fork 自 `origin/<default-branch>`（本仓库即 `main`），而不是 orchestrator 当前所在、gf-workflow 实际需要的 `base_branch`（通常是 `dev`）。这意味着即使解决了第 1 点，产出的分支也会天然缺失 `dev` 领先 `main` 的提交，合并/PR 时基点错误。

第 2 点是无法通过调整 gf-workflow 自身文档或握手协议规避的——Agent 工具的 `isolation: worktree` 未暴露可控的 base ref 参数。因此 **Mode ① 在结构上不可修复**，直接移除，不保留"标注已知限制但保留选项"的过渡态。

## 方案

移除 Mode ①，执行模式菜单从三选一改为二选一，两个来源（superpowers / mattpocock）菜单从此天然一致，不再需要为 mattpocock 单独裁剪说明：

- **① 手动新窗口（新默认）** — 原 Mode ②，无隔离限制，用户在新窗口/新会话中创建 worktree 并驱动执行引擎。
- **② 同会话执行（仅显式请求）** — 原 Mode ③，行为不变。

## 改动范围（纯文档，不涉及代码/schema）

1. `skills/gf-workflow/references.md`
   - `Phase 3 Execution Modes` 表格：删除 Mode ① 行，剩余两行重新编号为 ①②；表格下方补充一行说明，引用本设计文档的两条实测结论作为移除依据。
   - "Execution engine" 表格行：去掉 "background agent — per GO gate" 措辞。
2. `skills/gf-workflow/SKILL.md`
   - Gate 2→3 的 GO gate 描述：改为 "① 手动新窗口（默认）② 同会话（仅显式请求）"。
   - Worktree Preflight 描述中 "created by the executor (background agent / new window)" → 去掉 "background agent /"。
   - Execution engine 描述中 "(new window / background agent)" → "(new window)"。
3. `skills/gf-workflow/gates.md`
   - "GO 闸门" 说明由三选一改为二选一，去掉 mattpocock 单独裁剪的措辞（现在两来源菜单本就一致）。

`contract.schema.json` 的 `executor` 字段是自由字符串，无需改动。

## 验证

- `make check-agent-sync`（如适用于 skill 文档同步检查）
- 人工核对三个文件中不再残留 "① background agent" / "Mode ①" 相关措辞，且表格编号一致
