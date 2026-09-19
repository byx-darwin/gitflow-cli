# gf-workflow 错误分类恢复表与重试上限 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `skills/gf-workflow/references.md` 新增「Phase 3 Execution Error Classification」一节，定义 10 类执行期错误各自的恢复策略与重试上限；`skills/gf-workflow/SKILL.md` Phase 3 Step 2 引用该表分类处理失败，不再笼统重试。

**Architecture:** 纯文档改动，两个文件各一个 Task。`references.md` 是权威表格来源，`SKILL.md` 只新增一句引用，不复制表格内容（避免双份维护漂移）。

**Tech Stack:** Markdown (SKILL.md / references.md)。

**Spec:** `docs/superpowers/specs/2026-09-19-gf-workflow-error-classification-design.md`

## Global Constraints

- Skill 源代码在 `skills/` 目录下 — 只改 `skills/gf-workflow/SKILL.md` 与 `skills/gf-workflow/references.md`，不改 `~/.claude/skills/` 中的部署副本（来自项目 CLAUDE.md）。
- `merge_conflict` 类的恢复策略必须与现有 Phase 3 Step 3 的 `git merge --abort` + 交回用户语义兼容，不得引入自动重推 commit（设计文档「Context」一节明确排除）。
- `ci` 类的恢复策略必须与排队合并语义兼容：排队绑定的 SHA 上追加 commit 不会被带上，恢复策略不得是「重推 commit」（设计文档「Context」一节明确排除）。
- `auth`、`rate_limit` 两类重试上限为 0，出现即立即升级交还用户，不做任何自动重试（Issue 原文验收标准第 3 条）。
- 十类覆盖以设计文档「Acceptance Criteria」的范围澄清为准：包含 `issue_unclear`（Context 提及但 Issue 原文验收标准第 1 条列表遗漏），共 10 类：build / test / lint / merge_conflict / ci / auth / rate_limit / network / issue_unclear / unknown。
- 不新增 CLI 子命令，不改动合约 schema（`contract.schema.json`）。
- 本次是纯文档改动：不涉及 `cargo build/test/clippy`；验证靠 markdown 自洽性 + `make check-agent-sync`（`validate-skill-commands.sh` + `validate-skill-links.sh` + `verify-skills-when-not-to-use.sh`）。

---

### Task 1: references.md 新增 Phase 3 Execution Error Classification 章节

**Files:**
- Modify: `skills/gf-workflow/references.md`（在 `### Phase 3 Execution Modes (GO gate)` 章节之后、`### Worktree Location Convention (Issue #146)` 章节之前插入新章节；插入点以当前文件 `grep -n "### Phase 3 Execution Modes\|### Worktree Location Convention"` 的实际输出为准）

**Interfaces:**
- Consumes: 无上游任务。
- Produces: 新章节标题固定为 `### Phase 3 Execution Error Classification (Issue #336)`，Task 2 将以这个精确标题文本作为 `SKILL.md` 引用链接的锚点，不得改动大小写或措辞。

- [ ] **Step 1: 记录基线，定位插入点**

```bash
grep -n "### Phase 3 Execution Modes\|### Worktree Location Convention" skills/gf-workflow/references.md
```

Expected: 命令跑通，输出两行行号（`### Phase 3 Execution Modes (GO gate)` 在前，`### Worktree Location Convention (Issue #146)` 在后）。若行号已漂移，以实际输出为准，把新章节插在两者之间。

- [ ] **Step 2: 在两个章节之间插入新章节**

在 `### Phase 3 Execution Modes (GO gate)` 章节内容结束、`### Worktree Location Convention (Issue #146)` 开始之前，插入：

```markdown
### Phase 3 Execution Error Classification (Issue #336)

`skills/gf-workflow/SKILL.md`'s top-level Error Handling table covers orchestrator-level
failures (missing contract, gate check failed). It does not cover failures that occur
**during Phase 3 execution** (the implementation engine actually running build/test/lint
commands, merging, or queuing a merge). This table fills that gap.

**Two categories were adapted, not ported verbatim, from `smallnest/goal-workflow`'s
`loop-it` skill** — this repo's semantics differ:

- `merge_conflict`: Phase 3 Step 3's local-merge path already aborts and hands back to the
  user on conflict (`git merge --abort`, branch/worktree untouched). The table below keeps
  that behavior — zero automatic retries — rather than inventing a new auto-resolve loop.
- `ci`: Phase 3 Step 5 queues the merge (`gf pr merge --auto`) against a specific SHA that
  has already passed checks. Pushing a new commit to that branch after queuing does **not**
  get carried into the queued check (verified empirically) — so `ci` failures can never be
  "fixed" by pushing more commits onto the queued branch. A fix requires a fresh commit and
  a fresh queue entry, which restarts at Step 2, not a retry of Step 5.

| Category | Trigger | Recovery | Retry Cap |
|---|---|---|---|
| `build` | Compile/build failure | Fix code, rebuild | 3 |
| `test` | Test failure | Fix code or test, rerun | 3 |
| `lint` | `cargo clippy` / `cargo fmt` (or per-language equivalent) failure | Fix, rerun `make lint` | 3 |
| `merge_conflict` | `git merge` conflict (Phase 3 Step 3, local-merge path) | `git merge --abort`; leave `branch`/worktree untouched; escalate to user immediately — no automatic retry. User resolves manually, then Step 3 is re-run as a fresh attempt (not counted against this cap) | 0 |
| `ci` | Required check fails after the merge queue (Phase 3 Step 5, PR path) | Never push a new commit to the already-queued branch. First rule out flakiness with one same-SHA platform re-run; if it still fails, a real fix requires a new commit + a fresh queue entry — that restarts at Step 2, it is not a retry of Step 5 | 1 (same-SHA re-run only) |
| `auth` | `gf auth status` failure / API 401/403 | No retry — escalate to user immediately (`auth login` is a human action) | 0 |
| `rate_limit` | API 429 / platform throttling | No retry — escalate to user immediately (waiting or rotating credentials is a human decision) | 0 |
| `network` | Transient network error (timeout, connection reset) | Retry with backoff | 3 |
| `issue_unclear` | Execution engine hits a requirement ambiguity it cannot resolve | Not a retryable error — pause and ask the user for clarification; never guess. Each clarification round is a fresh attempt, not counted against a retry cap | 0 |
| `unknown` | Uncategorized error | Capture the full error, retry once conservatively; escalate if it recurs | 1 |

**Escalation contract.** When a category's retry cap is reached (or immediately, for the
three 0-cap categories): stop retrying automatically, show the user the error category,
every recovery attempt tried so far with its outcome, and the retry count, then wait for
the user's decision (continue / change strategy / abort the task). Never silently give up
and never silently switch delivery mode (e.g. falling back from local-merge to PR) as a
substitute for asking.
```

- [ ] **Step 3: 运行 skill 校验**

```bash
make check-agent-sync
```

Expected: `validate-skill-commands.sh` / `validate-skill-links.sh` / `verify-skills-when-not-to-use.sh` 均输出 ✓，退出码 0。本任务未新增/删除文件链接、未引入新 `gf` 子命令，预期无破坏性变更。

- [ ] **Step 4: Commit**

```bash
git add skills/gf-workflow/references.md
git commit -m "feat(workflow): add Phase 3 execution error classification table (#336)"
```

---

### Task 2: SKILL.md Phase 3 Step 2 引用错误分类表

**Files:**
- Modify: `skills/gf-workflow/SKILL.md`（Phase 3 Step 表格第 2 行，`| 2 | **[AUTO] Execution engine** ...`）

**Interfaces:**
- Consumes: Task 1 产出的章节标题 `### Phase 3 Execution Error Classification (Issue #336)`，作为本任务引用文字中 `references.md` 一节的精确名称，不得改写。
- Produces: 无下游任务消费本任务产物——本任务是终态交付。

- [ ] **Step 1: 记录基线**

```bash
grep -n '| 2 | \*\*\[AUTO\] Execution engine\*\*' skills/gf-workflow/SKILL.md
```

Expected: 命令跑通，输出一行（当前预期在第 353 行附近）。若行号已漂移，以实际 `grep` 输出为准调整下一步的 old_string 定位。

- [ ] **Step 2: 修改 Phase 3 Step 2 表格行**

将现有：

```markdown
| 2 | **[AUTO] Execution engine** (per `skill_source` + chosen execution mode): superpowers → `superpowers:subagent-driven-development` (same-session) or `superpowers:executing-plans` (new window); mattpocock → ✋ PAUSE per ticket → user runs `/implement` in dependency order (internal `/tdd` mandatory). All paths: TDD RED → GREEN → REFACTOR | implementation |
```

替换为：

```markdown
| 2 | **[AUTO] Execution engine** (per `skill_source` + chosen execution mode): superpowers → `superpowers:subagent-driven-development` (same-session) or `superpowers:executing-plans` (new window); mattpocock → ✋ PAUSE per ticket → user runs `/implement` in dependency order (internal `/tdd` mandatory). All paths: TDD RED → GREEN → REFACTOR. **On execution failure:** classify against `references.md` → Phase 3 Execution Error Classification before retrying — never retry generically or escalate without first checking which category and its retry cap apply | implementation |
```

- [ ] **Step 3: 运行 skill 校验**

```bash
make check-agent-sync
```

Expected: 三个子脚本均输出 ✓，退出码 0。

- [ ] **Step 4: 本地自洽性检查——确认新增引用文字可在 references.md 中定位到对应章节**

```bash
grep -n "Phase 3 Execution Error Classification" skills/gf-workflow/SKILL.md skills/gf-workflow/references.md
```

Expected: 两个文件都至少匹配一行；`SKILL.md` 的引用文字与 `references.md` 的章节标题在关键短语（`Phase 3 Execution Error Classification`）上完全一致，无拼写偏差。

- [ ] **Step 5: Commit**

```bash
git add skills/gf-workflow/SKILL.md
git commit -m "feat(workflow): Phase 3 execution engine references error classification table (#336)"
```
