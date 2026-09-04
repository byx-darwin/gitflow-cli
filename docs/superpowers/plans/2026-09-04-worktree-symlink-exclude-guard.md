# Worktree 共享符号链接防误提交 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让 gf-workflow Worktree Preflight 创建的 `.cache/workflows`、`.claude` 符号链接在任何情况下都不可能被意外提交进主分支，并在交付前拦截任何已经溜进提交历史的符号链接。

**Architecture:** 纯文档/流程规则变更，四处文本编辑：(1)(2) `SKILL.md` 里两处内联指令补上安全动作；(3)(4)(5) `references.md` 里补写细节说明、示例、检测命令。不涉及任何 Rust 代码或二进制逻辑。

**Tech Stack:** Markdown、Bash（git 命令片段，作为文档内容，不是可执行脚本，不需要 shellcheck/cargo）。

**Spec:** `docs/superpowers/specs/2026-09-04-worktree-symlink-exclude-guard-design.md`

## Global Constraints

- 改动只落在 `skills/gf-workflow/SKILL.md` 与 `skills/gf-workflow/references.md`（源文件，不是 `.claude/skills/` 副本）— 见 CLAUDE.md「Skill 源代码在 `skills/` 目录下」。
- `.git/info/exclude` 写入必须走 `$(git rev-parse --git-common-dir)/info/exclude`（已验证：worktree 无独立 exclude，此文件由主仓库与该 clone 下所有 worktree 共享）。
- 幂等：exclude 写入用 `grep -qxF '<line>' "$FILE" || echo '<line>' >> "$FILE"`，避免重复 workflow 运行时重复追加。
- 符号链接提交检测命令：`git diff --summary "$BASE_BRANCH"...HEAD | grep 'create mode 120000'`，命中即 ✋ PAUSE，不自动放行。
- 文档/流程变更：不运行 `cargo build/test/clippy`；验证方式为人工核对 Markdown 渲染 + shell 语法 + （若适用）`make check-agent-sync`。

## 范围修正说明（相对于设计文档的补充）

设计文档（`2026-09-04-worktree-symlink-exclude-guard-design.md`）写的改动范围是"仅
`references.md`"。规划阶段复核发现：`SKILL.md` Phase 3 Step 1（第 344 行）里也**内联复述了一份**
创建符号链接的完整命令（`mkdir -p ... && ln -s ... && ln -s ...`），这份内联命令在
"Every execution mode must run this preflight... handoff MUST carry the preflight steps
verbatim" 的规则下会被**原样复制**给后台 agent / 新窗口执行者——如果只改
`references.md` 的示例块，`SKILL.md` 自己的内联命令仍然没有 exclude 写入，防护会被绕过。

因此本计划把 `SKILL.md` 的两处内联文本（Step 1 的符号链接命令 + Step 3 的交付前检测缺口）
也纳入改动范围，作为 Task 1 的一部分。这是对设计文档范围的必要修正，不改变设计的技术方案本身。

---

### Task 1: `SKILL.md` Phase 3 Step 1 — 内联符号链接命令补上 exclude 写入

**Files:**
- Modify: `skills/gf-workflow/SKILL.md:344`

**Interfaces:**
- Consumes: 无（纯文本编辑）
- Produces: Step 1 的内联命令片段中新增一段 exclude 写入，供后续 Task 2/3/4 引用同样的写法

- [ ] **Step 1: 定位并替换第 344 行的符号链接命令片段**

当前文本（第 344 行，节选）：

```
**After worktree creation**: symlink shared directories so workflow contracts and Claude config are accessible from the worktree: `mkdir -p <worktree-path>/.cache && ln -s ../../.cache/workflows <worktree-path>/.cache/workflows && ln -s ../../.claude <worktree-path>/.claude`. **Then assert** each contract-referenced document exists under `<worktree-path>/` — abort if missing.
```

替换为：

```
**After worktree creation**: symlink shared directories so workflow contracts and Claude config are accessible from the worktree: `mkdir -p <worktree-path>/.cache && ln -s ../../.cache/workflows <worktree-path>/.cache/workflows && ln -s ../../.claude <worktree-path>/.claude`. **Immediately exclude them** so no later commit can pick them up — write to the COMMON git dir's `info/exclude` (verified: worktrees share one `info/exclude` with the main tree and all sibling worktrees, there is no per-worktree copy): `EF="$(git rev-parse --git-common-dir)/info/exclude"; grep -qxF '.cache/workflows' "$EF" || echo '.cache/workflows' >> "$EF"; grep -qxF '.claude' "$EF" || echo '.claude' >> "$EF"`. **Then assert** each contract-referenced document exists under `<worktree-path>/` — abort if missing. Full rationale: `references.md` → Worktree Preflight.
```

- [ ] **Step 2: 校对替换结果**

Run: `grep -n "info/exclude" skills/gf-workflow/SKILL.md`
Expected: 第 344 行命中，且反引号成对、命令可复制执行不报语法错误（用 `bash -n` 校验，见下）。

- [ ] **Step 3: 语法自检（提取内联命令片段单独跑 `bash -n`）**

```bash
cat > /tmp/task1-check.sh <<'EOF'
EF="$(git rev-parse --git-common-dir)/info/exclude"
grep -qxF '.cache/workflows' "$EF" || echo '.cache/workflows' >> "$EF"
grep -qxF '.claude' "$EF" || echo '.claude' >> "$EF"
EOF
bash -n /tmp/task1-check.sh && echo "syntax OK"
rm -f /tmp/task1-check.sh
```

Expected: `syntax OK`

- [ ] **Step 4: Commit**

```bash
git add skills/gf-workflow/SKILL.md
git commit -m "docs(gf-workflow): worktree symlinks write to shared info/exclude (#318)"
```

---

### Task 2: `SKILL.md` Phase 3 Step 3 — 交付前新增符号链接提交检测指引

**Files:**
- Modify: `skills/gf-workflow/SKILL.md:346`

**Interfaces:**
- Consumes: Task 1 里确立的 exclude 写法（仅作为背景一致性参考，无代码依赖）
- Produces: Step 3 表格行新增一句指引，指向 `references.md` 的具体检测命令（Task 4 产出）

- [ ] **Step 1: 定位第 346 行，在 "Delivery choice" 描述最前面插入检测前置动作**

当前文本（第 346 行，节选）：

```
| 3 | **[AUTO]** Delivery choice — ask user: ① 推送 + 建 PR（默认）② 本地合并. **① PR**: `gf-pr-create`, PR body MUST include `Closes #<issue-number>`; ...
```

替换为：

```
| 3 | **[AUTO]** Pre-delivery symlink guard — before offering the delivery choice, run `git diff --summary "$base_branch"...HEAD \| grep 'create mode 120000'`; a hit means a symlink (most likely `.cache/workflows` or `.claude`) got committed onto `branch` — ✋ PAUSE, show the matched path(s), do not proceed to PR/merge until the user resolves it (drop the commit or confirm intentional). Full command + rationale: `references.md` → Worktree Preflight. Clean → continue. Delivery choice — ask user: ① 推送 + 建 PR（默认）② 本地合并. **① PR**: `gf-pr-create`, PR body MUST include `Closes #<issue-number>`; ...
```

（保留该行原有的 ① PR / ② 本地合并 后续全部文本不变，只在行首插入上述前置检测段落。）

- [ ] **Step 2: 校对替换结果**

Run: `grep -n "Pre-delivery symlink guard" skills/gf-workflow/SKILL.md`
Expected: 第 346 行命中一次。

- [ ] **Step 3: 确认表格未破坏（Markdown 表格分隔符 `|` 数量与其余行一致）**

Run: `awk -F'|' 'NR==346{print NF}' skills/gf-workflow/SKILL.md`
Expected: 输出的字段数与该表格其它数据行的字段数一致（该表格是两列：Step / Output，所以应为 4，即开头空 + Step 列 + Output 列 + 末尾空）。若不一致，检查是否有未转义的 `|` 破坏了表格列数。

- [ ] **Step 4: Commit**

```bash
git add skills/gf-workflow/SKILL.md
git commit -m "docs(gf-workflow): pre-delivery symlink-commit guard in Phase 3 Step 3 (#318)"
```

---

### Task 3: `references.md` — Worktree Path Convention 示例块补上 exclude 写入

**Files:**
- Modify: `skills/gf-workflow/references.md:113-120`

**Interfaces:**
- Consumes: 无
- Produces: 示例代码块里新增的 exclude 写入片段，供 Task 4 的说明小节引用同一套命令

- [ ] **Step 1: 定位并替换第 113-120 行**

当前文本：

```bash
# Symlink shared directories (workflow contracts + Claude config)
mkdir -p .worktree/feat-146-worktree-path/.cache
ln -s ../../.cache/workflows .worktree/feat-146-worktree-path/.cache/workflows
ln -s ../../.claude .worktree/feat-146-worktree-path/.claude

cd .worktree/feat-146-worktree-path
git add docs && git commit -m "docs(workflow): wf-2026-08-30-001 Phase 1-2 artifacts"
cd -
```

替换为：

```bash
# Symlink shared directories (workflow contracts + Claude config)
mkdir -p .worktree/feat-146-worktree-path/.cache
ln -s ../../.cache/workflows .worktree/feat-146-worktree-path/.cache/workflows
ln -s ../../.claude .worktree/feat-146-worktree-path/.claude

# Exclude them from git tracking — writes to the COMMON git dir's info/exclude
# (verified: worktrees do NOT have a per-worktree info/exclude; this file is shared
# by the main tree + all worktrees of this local clone), so it protects every
# worktree, not just this one, without touching the project's own .gitignore.
EXCLUDE_FILE="$(cd .worktree/feat-146-worktree-path && git rev-parse --git-common-dir)/info/exclude"
grep -qxF '.cache/workflows' "$EXCLUDE_FILE" || echo '.cache/workflows' >> "$EXCLUDE_FILE"
grep -qxF '.claude' "$EXCLUDE_FILE" || echo '.claude' >> "$EXCLUDE_FILE"

cd .worktree/feat-146-worktree-path
git add docs && git commit -m "docs(workflow): wf-2026-08-30-001 Phase 1-2 artifacts"
cd -
```

- [ ] **Step 2: 校对替换结果**

Run: `grep -n "EXCLUDE_FILE" skills/gf-workflow/references.md`
Expected: 3 处命中（赋值 1 + 使用 2）。

- [ ] **Step 3: 语法自检**

```bash
cat > /tmp/task3-check.sh <<'EOF'
EXCLUDE_FILE="$(cd /tmp && git rev-parse --git-common-dir 2>/dev/null || echo /tmp/.git/info/exclude)"
grep -qxF '.cache/workflows' "$EXCLUDE_FILE" 2>/dev/null || echo '.cache/workflows'
grep -qxF '.claude' "$EXCLUDE_FILE" 2>/dev/null || echo '.claude'
EOF
bash -n /tmp/task3-check.sh && echo "syntax OK"
rm -f /tmp/task3-check.sh
```

Expected: `syntax OK`

- [ ] **Step 4: Commit**

```bash
git add skills/gf-workflow/references.md
git commit -m "docs(gf-workflow): worktree example writes symlinks to shared info/exclude (#318)"
```

---

### Task 4: `references.md` — 新增"为什么这两个链接绝不能进主分支"说明小节 + 交付前检测命令

**Files:**
- Modify: `skills/gf-workflow/references.md`（紧跟 "Worktree Preflight" 小节之后，即当前第 190-192 行之间，`## Lifecycle Management` 之前）

**Interfaces:**
- Consumes: Task 3 里的 exclude 写法（文字复用，非代码依赖）
- Produces: 新增小节标题 `### Why These Symlinks Must Never Reach the Main Branch`，供 `SKILL.md` Task 2 的 "Full command + rationale: references.md → Worktree Preflight" 指向

- [ ] **Step 1: 定位插入点**

当前第 188-192 行：

```
**Every execution mode must run this preflight.** Modes ① and ② let the *executor* create
the worktree, so the orchestrator cannot rely on having checked the tree itself — the
handoff text must carry these steps verbatim. See `Phase 3 Execution Modes` below.

## Lifecycle Management
```

- [ ] **Step 2: 在 "See `Phase 3 Execution Modes` below." 之后、`## Lifecycle Management` 之前插入新小节**

插入内容：

```markdown

### Why These Symlinks Must Never Reach the Main Branch

`.cache/workflows` and `.claude` inside a worktree are relative symlinks
(`../../.cache/workflows`, `../../.claude`). If either is ever committed, `git ls-files -s`
shows a `120000` (symlink) mode entry for that path. A clone made from a commit carrying
that entry re-creates the symlink pointing at `../../<name>` **relative to that clone's own
location** — which, outside the original working tree that produced it, resolves to a
directory that does not exist or belongs to something else entirely.

**Verified real-world impact (Issue #318):** in the downstream project
`iproost/proxy/api-src`, `.cache/workflows` and `.claude/.claude` had been committed as
symlinks (commit `e7f4254`, swept in by an unrelated broad `git add`). Resolved from that
repo's root, `.cache/workflows -> ../../.cache/workflows` landed **outside the repository**,
in a directory shared by other checkouts. Every subsequent gf-workflow contract read/write in
that project actually happened against that external shared path — including a case where a
background research fork and the main session concurrently touched the same contract file and
cross-wrote each other's Phase 3/4 progress.

**Why `info/exclude` fixes this at the source, not just in this repo.** A linked worktree has
no `info/exclude` of its own: `git rev-parse --git-common-dir` from inside any worktree
resolves to the *main* repository's `.git`, and `info/exclude` always lives there — confirmed
by writing to it from a worktree and observing `git status` change in a sibling worktree and
the main tree alike. So the one write performed right after `ln -s` (see the example above)
protects the main tree and every worktree this clone will ever create, permanently, without
depending on that project's own `.gitignore` ever mentioning `.cache/` or `.claude/` — which is
exactly the gap that let `e7f4254` happen upstream.

**Belt and suspenders.** `info/exclude` only stops *new* accidental adds; it does nothing for
a symlink that is already staged in a commit about to leave `branch` (rebase, cherry-pick,
`git commit -a` racing the exclude write, etc.). That is why Phase 3 Step 3 in `SKILL.md` also
scans the diff immediately before delivery:

```bash
git diff --summary "$BASE_BRANCH"...HEAD | grep 'create mode 120000'
```

A hit means some commit on `branch` added a symlink that `base_branch` doesn't have. Treat it
as a hard stop: show the path(s), and let the user choose to drop the offending commit/entry
or explicitly confirm it is an intentional, unrelated symlink before delivery proceeds.

## Lifecycle Management
```

- [ ] **Step 3: 校对插入结果**

Run: `grep -n "Why These Symlinks Must Never Reach the Main Branch\|## Lifecycle Management" skills/gf-workflow/references.md`
Expected: 新小节标题出现一次，且紧接其后仍能找到 `## Lifecycle Management`（顺序不变，只是中间插入了新内容，行号整体下移）。

- [ ] **Step 4: 语法自检（提取检测命令单独跑 `bash -n`）**

```bash
cat > /tmp/task4-check.sh <<'EOF'
BASE_BRANCH=main
git diff --summary "$BASE_BRANCH"...HEAD | grep 'create mode 120000'
EOF
bash -n /tmp/task4-check.sh && echo "syntax OK"
rm -f /tmp/task4-check.sh
```

Expected: `syntax OK`

- [ ] **Step 5: Commit**

```bash
git add skills/gf-workflow/references.md
git commit -m "docs(gf-workflow): explain why worktree symlinks must never reach main branch (#318)"
```

---

### Task 5: 收尾核对 — 交叉引用一致性 + Markdown 渲染 + 索引更新确认

**Files:**
- Read-only check: `skills/gf-workflow/SKILL.md`, `skills/gf-workflow/references.md`
- Verify (already done in Phase 1, no edit needed): `docs/index.md`

**Interfaces:**
- Consumes: Task 1-4 的全部改动
- Produces: 无新文件；仅验证证据

- [ ] **Step 1: 交叉引用完整性检查**

Run:
```bash
grep -n "references.md.*Worktree Preflight\|Full command + rationale" skills/gf-workflow/SKILL.md
grep -n "Why These Symlinks Must Never Reach the Main Branch" skills/gf-workflow/references.md
```
Expected: `SKILL.md` 里 Task 1、Task 2 新增的两处指向语句都能在输出中看到；`references.md` 里新小节标题存在。

- [ ] **Step 2: 全文件语法体检（反引号配对、代码块闭合）**

```bash
for f in skills/gf-workflow/SKILL.md skills/gf-workflow/references.md; do
  awk 'BEGIN{n=0} /^```/{n++} END{print FILENAME": fence count="n" (must be even)"}' "$f"
done
```
Expected: 两个文件的 fence count 都是偶数（代码块正确闭合）。

- [ ] **Step 3: （若项目提供）跑 `make check-agent-sync`**

Run: `make check-agent-sync 2>&1 | tail -30`
Expected: 无报错；若该 target 不存在或不适用于 skill 文档改动，跳过并在最终报告里说明跳过原因。

- [ ] **Step 4: 确认 `docs/index.md` 索引条目已在 Phase 1 写入（无需重复操作）**

Run: `grep -n "worktree-symlink-exclude-guard" docs/index.md`
Expected: 命中一行（Phase 1 已完成，此步骤仅确认未被后续编辑意外破坏）。

- [ ] **Step 5: 最终 diff 走查（人工读一遍，不做 commit）**

Run: `git diff --stat main -- skills/gf-workflow/`
Expected: 只有 `SKILL.md` 和 `references.md` 两个文件有改动，无 Rust 源码文件、无 Cargo.toml/lock、无 CI 配置文件被触及。

（本任务不产生新 commit——它是验证性任务，若发现问题，回到对应 Task 修正并重新 commit。）
