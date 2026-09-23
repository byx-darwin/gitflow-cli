# Worktree 共享符号链接防误提交设计（Issue #318）

## Context

`gf-workflow` Phase 3 Worktree Preflight（`skills/gf-workflow/references.md` →
Worktree Path Convention）在 worktree 内创建两个共享目录符号链接：

```bash
mkdir -p .worktree/<branch>/.cache
ln -s ../../.cache/workflows .worktree/<branch>/.cache/workflows
ln -s ../../.claude .worktree/<branch>/.claude
```

这两个符号链接从未被排除在 git 追踪之外。示例里紧邻的提交显式用了
`git add docs` 才幸免；后续 Phase 3 Step 2 的 TDD 实现提交若使用
`git add -A` / `git commit -a`（RED→GREEN→REFACTOR 循环中很常见），这两个符号
链接就会被一起提交。

**实测影响**（下游项目 `iproost/proxy/api-src`）：`.cache/workflows`、
`.claude/.claude` 已被提交为符号链接（mode `120000`，提交 `e7f4254`，与
gf-workflow 无关的业务提交顺带扫入）。从仓库根解析
`.cache/workflows -> ../../.cache/workflows` 落点在仓库**外部**，导致该项目
之后所有 workflow 合约文件实际读写在仓库外的共享路径，出现过跨仓库/跨会话
的合约交叉写入。

## Goal

让这两个符号链接在任何情况下都不可能被意外提交进主分支。

## 技术验证：`.git/info/exclude` 在 worktree 中的作用范围

用 `git worktree add` 创建的子 worktree 的 `.git` 是一个指向
`<common-dir>/worktrees/<name>` 的文件，而非独立仓库。实测验证（`git init` +
`git worktree add` + 分别在 common dir 和 `worktrees/<name>` 下写
`info/exclude` 并在两侧跑 `git status`）确认：

- **`info/exclude` 不是每个 worktree 独立的**——它始终读取
  `$(git rev-parse --git-common-dir)/info/exclude`，即主仓库的共享文件。
- 写入这个文件会同时影响**主工作区 + 该本地 clone 的所有 worktree**，而不仅
  仅是当前正在创建的这一个。

这一发现修正了 Issue 里"worktree 本地生效"的表述，但结论对目标更有利：写一次
即可保护该 clone 下的所有 worktree 和主树，且仍然是纯本地文件（不进 git、不
推送、不污染项目自身 `.gitignore`）。

## 设计

改动范围：仅 `skills/gf-workflow/references.md`，两处。

### 1. Worktree Preflight 示例块：创建符号链接后立即写入共享 exclude

在两条 `ln -s` 之后、`cd` 进 worktree 提交文档之前追加：

```bash
# Exclude them from git tracking — writes to the COMMON git dir's info/exclude
# (verified: worktrees do NOT have a per-worktree info/exclude; this file is
# shared by main tree + all worktrees of this local clone), so it protects
# every worktree, not just this one, without touching the project's own
# .gitignore.
EXCLUDE_FILE="$(cd .worktree/feat-146-worktree-path && git rev-parse --git-common-dir)/info/exclude"
grep -qxF '.cache/workflows' "$EXCLUDE_FILE" || echo '.cache/workflows' >> "$EXCLUDE_FILE"
grep -qxF '.claude' "$EXCLUDE_FILE" || echo '.claude' >> "$EXCLUDE_FILE"
```

`grep -qxF` 做幂等检查，避免多个 workflow 重复追加同一行。

### 2. 新增说明小节：为什么这两个链接绝不能进主分支

紧跟 Worktree Preflight 小节之后新增一段，内容涵盖：

- 两个符号链接指向仓库外部的相对路径；一旦被提交，`git ls-files -s` 会显示
  `120000` 模式；clone 到别处后会解析到错误/不存在的外部路径。
- 引用 Issue #318 中 `iproost/proxy/api-src` 的实测案例作为后果佐证（合约文件
  跨仓库/跨会话串读串写）。
- 说明 `.git/info/exclude` 的共享特性（本设计"技术验证"一节的结论），解释为
  什么写入它能一次性覆盖该 clone 的所有 worktree。

### 3. Phase 3 Step 3（交付前）新增符号链接提交检测

在 delivery（`gf-pr-create` / local merge）之前插入强制检查：

```bash
# Run before gf-pr-create / local merge — scan for any newly-committed symlink
git diff --summary "$BASE_BRANCH"...HEAD | grep 'create mode 120000' && {
  echo "✋ PAUSE: 检测到新增符号链接被提交，可能是 .cache/workflows 或 .claude 意外入库"
  exit 1
}
```

命中 → ✋ PAUSE，展示具体路径，交由用户决定（从提交中移除该符号链接条目 /
确认这是有意为之），不自动放行合并/建 PR。

放在 Phase 3 Step 3（交付前）而非 Phase 4 Branch Finish：交付前拦截可以在坏数据
进入 `base_branch` 之前就挡住，比事后清理更彻底，与 Goal 一致。

### 影响范围

- 纯文档/流程规则变更，不涉及 Rust 代码，无需 `cargo build/test/clippy`。
- 验证方式：`make check-agent-sync`（如适用）+ 人工核对 Markdown 渲染与命令
  语法。

## Acceptance Criteria（来自 Issue #318）

- [x] Worktree Preflight 创建符号链接时，同步写入共享 git-exclude，避免任何
      后续提交意外带上它们（见"设计 1"）
- [x] Phase 3/Branch Finish 增加一道针对"新增符号链接被提交"的检测/阻断
      （见"设计 3"，放在 Phase 3 Step 3 交付前）
- [x] 补充一条 references.md 说明，解释为什么这两个链接绝不能进主分支
      （见"设计 2"）
