# gf-workflow Mode ① Removal Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the "① Background agent" execution mode from `gf-workflow`'s Phase 3 GO gate menu across all three skill doc files, leaving a two-option menu (① manual new window / ② same-session).

**Architecture:** Pure documentation edit. No Rust code, no `contract.schema.json` change (the `executor` field is already a free-form string). Three files edited independently, each verified by `grep` for residue, no cross-file code dependency.

**Tech Stack:** Markdown tables, `grep`/`rg` for verification.

**Spec:** `specs/gf-workflow-mode1-removal-design.md` — this plan implements every change listed in that doc's "改动范围" section. Read it first; it documents the two structural defects (git-command isolation + fixed fork base from `origin/<default-branch>`) that motivate removing Mode ① outright rather than annotating it as a known limitation.

## Global Constraints

- Do not touch `contract.schema.json` — the `executor` field needs no change.
- Do not touch any Rust source, `Cargo.toml`, or CI config — this is a docs/skill-only change per project CLAUDE.md ("For documentation/spec/skill-only changes... do not run Rust build/test/clippy").
- Skill source of truth is `skills/gf-workflow/*.md` (project CLAUDE.md: "Skill 源代码在 `skills/` 目录下... 不要修改 `.claude/skills/` 中的副本").
- After all edits, no file may contain the strings "Mode ①" together with "background agent"/"后台" in the same table row, and no file may reference a three-option menu for Phase 3 execution.

---

### Task 1: Update `skills/gf-workflow/references.md`

**Files:**
- Modify: `skills/gf-workflow/references.md:367` (Execution engine table row)
- Modify: `skills/gf-workflow/references.md:410-420` (Phase 3 Execution Modes table)

**Interfaces:**
- Consumes: nothing (independent file)
- Produces: the canonical two-row Execution Modes table that Task 2 and Task 3 reference by name (row labels "① Manual new window" / "② Same-session")

- [ ] **Step 1: Read current state**

Run: `sed -n '360,435p' skills/gf-workflow/references.md`

Confirm line 367 reads:
```
| Execution engine | `subagent-driven-development` (same-session) / `executing-plans` (new window) / background agent — per GO gate | ✋ `/implement` per ticket (internal `/tdd` mandatory) | per mode / user-invoked |
```
and lines 416-420 contain the 3-row Mode table with "① Background agent ⭐default" as the first row.

- [ ] **Step 2: Edit line 367 — drop the background-agent clause**

Replace:
```
| Execution engine | `subagent-driven-development` (same-session) / `executing-plans` (new window) / background agent — per GO gate | ✋ `/implement` per ticket (internal `/tdd` mandatory) | per mode / user-invoked |
```
with:
```
| Execution engine | `subagent-driven-development` (same-session) / `executing-plans` (new window) — per GO gate | ✋ `/implement` per ticket (internal `/tdd` mandatory) | per mode / user-invoked |
```

- [ ] **Step 3: Replace the Phase 3 Execution Modes table**

Replace the existing 3-row table (the row starting `| ① Background agent ⭐default |` through the row starting `| ③ Same-session |`) with:

```
| Mode | Description | Availability |
|---|---|---|
| ① Manual new window ⭐default | Print opening guidance: worktree path (or creation command) + contract recovery command (`gf workflow status <id>` + plan doc path) + **Worktree Preflight steps verbatim**; new window creates branch/worktree itself and runs `executing-plans` (superpowers) or per-ticket `/implement` (mattpocock); user reports back, orchestrator verifies evidence | both sources |
| ② Same-session | Current behavior: orchestrator creates worktree and drives the engine inline | explicit request only |
```

Immediately below the table (before the "Quality compensation" paragraph), add:

```
**Why no background-agent mode:** an earlier "① Background agent" mode (`isolation: worktree` + `run_in_background`) was removed after Issue #325 confirmed two structural defects that cannot be fixed by documentation alone: (1) a worktree-isolated agent's git operations are hard-restricted to its own harness-managed worktree — it cannot target `.worktree/<branch-name>`; (2) that harness-managed worktree forks from `origin/<default-branch>`, not from the orchestrator's `base_branch`, so any resulting branch would silently miss `base_branch`-only commits. See `specs/gf-workflow-mode1-removal-design.md` for the investigation.
```

- [ ] **Step 4: Verify**

Run: `grep -n "Mode ①\|background agent\|Background agent" skills/gf-workflow/references.md`
Expected: no matches (the explanatory paragraph above uses "background-agent" as a compound adjective without "Mode ①" — if the grep flags it, reword to avoid the literal string "background agent" / "Background agent", e.g. "background-executor mode").

Run: `grep -n "① Manual new window\|② Same-session" skills/gf-workflow/references.md`
Expected: both present, exactly once each.

- [ ] **Step 5: Commit**

```bash
git add skills/gf-workflow/references.md
git commit -m "docs(gf-workflow): remove Mode ① background agent from Execution Modes"
```

---

### Task 2: Update `skills/gf-workflow/SKILL.md`

**Files:**
- Modify: `skills/gf-workflow/SKILL.md:334` (Gate 2→3 GO gate description)
- Modify: `skills/gf-workflow/SKILL.md:344` (Worktree Preflight executor clause)
- Modify: `skills/gf-workflow/SKILL.md:345` (Execution engine clause)

**Interfaces:**
- Consumes: Task 1's new two-row mode table (referenced by name/number, not by content — no code coupling)
- Produces: nothing consumed by other tasks

- [ ] **Step 1: Read current state**

Run: `sed -n '330,350p' skills/gf-workflow/SKILL.md`

Confirm:
- Line 334 contains: `**execution-mode choice (GO gate)**: ① background agent (default, superpowers only) ② manual new window ③ same-session (explicit request only)`
- Line 344 contains: `Created here for same-session mode; created by the executor (background agent / new window) otherwise`
- Line 345 contains: `superpowers:executing-plans` (new window / background agent)`

- [ ] **Step 2: Edit line 334**

Replace:
```
**execution-mode choice (GO gate)**: ① background agent (default, superpowers only) ② manual new window ③ same-session (explicit request only); mattpocock menu is trimmed — see `references.md` → Phase 3 Execution Modes
```
with:
```
**execution-mode choice (GO gate)**: ① manual new window (default) ② same-session (explicit request only); identical menu for both `skill_source` values — see `references.md` → Phase 3 Execution Modes
```

- [ ] **Step 3: Edit line 344**

Replace the clause:
```
Created here for same-session mode; created by the executor (background agent / new window) otherwise
```
with:
```
Created here for same-session mode; created by the executor (new window) otherwise
```

(Leave the rest of line 344 — the preflight bucket classification, symlink depth computation, `info/exclude` write — untouched; only this one clause changes.)

- [ ] **Step 4: Edit line 345**

Replace:
```
| 2 | **[AUTO] Execution engine** (per `skill_source` + chosen execution mode): superpowers → `superpowers:subagent-driven-development` (same-session) or `superpowers:executing-plans` (new window / background agent); mattpocock → ✋ PAUSE per ticket → user runs `/implement` in dependency order (internal `/tdd` mandatory). All paths: TDD RED → GREEN → REFACTOR | implementation |
```
with:
```
| 2 | **[AUTO] Execution engine** (per `skill_source` + chosen execution mode): superpowers → `superpowers:subagent-driven-development` (same-session) or `superpowers:executing-plans` (new window); mattpocock → ✋ PAUSE per ticket → user runs `/implement` in dependency order (internal `/tdd` mandatory). All paths: TDD RED → GREEN → REFACTOR | implementation |
```

- [ ] **Step 5: Verify**

Run: `grep -n "Mode ①\|background agent\|Background agent\|③ same-session" skills/gf-workflow/SKILL.md`
Expected: no matches.

Run: `grep -n "① manual new window\|① background agent" skills/gf-workflow/SKILL.md`
Expected: only the new "① manual new window (default)" line matches; no "① background agent" left.

- [ ] **Step 6: Commit**

```bash
git add skills/gf-workflow/SKILL.md
git commit -m "docs(gf-workflow): trim GO gate to two execution modes"
```

---

### Task 3: Update `skills/gf-workflow/gates.md`

**Files:**
- Modify: `skills/gf-workflow/gates.md:48-56` (GO 闸门 section)

**Interfaces:**
- Consumes: Task 1's two-row mode table (referenced by name, no code coupling)
- Produces: nothing consumed by other tasks

- [ ] **Step 1: Read current state**

Run: `sed -n '43,57p' skills/gf-workflow/gates.md`

Confirm the block reads:
```
**GO 闸门——执行模式选择（Issue #141）:** 用户批准后、进入 Phase 3 前，编排器必须提供执行模式选择：
① 后台代理（默认推荐，仅 superpowers 来源可用）② 手动新窗口 ③ 同会话执行（仅显式要求）。
mattpocock 来源下菜单自动裁剪为 ②③（`/implement` 为 user-invoked，后台代理无法调用）。
模式语义详见 `references.md` → Phase 3 Execution Modes。

提示文案须告知：Phase 3 Step 1 会先跑 **Worktree Preflight**——设计文档（Bucket A）提交前也会
暂停询问是否提交，主工作区若有与本工作流无关的改动（Bucket B）会再次中断询问。
此处**不新增闸门条件**——Gate 2→3 只校验合同证据；且模式 ①② 下建 worktree 的是执行者，
在闸门里查树状态保护不到它，preflight 必须随 handoff 下发。
```

- [ ] **Step 2: Replace the block**

Replace the entire block above with:

```
**GO 闸门——执行模式选择（Issue #141，Issue #325 移除后台代理选项）:** 用户批准后、进入 Phase 3 前，编排器必须提供执行模式选择：
① 手动新窗口（默认）② 同会话执行（仅显式要求）。两个来源（superpowers / mattpocock）菜单一致，
无需按来源裁剪。
模式语义详见 `references.md` → Phase 3 Execution Modes。

提示文案须告知：Phase 3 Step 1 会先跑 **Worktree Preflight**——设计文档（Bucket A）提交前也会
暂停询问是否提交，主工作区若有与本工作流无关的改动（Bucket B）会再次中断询问。
此处**不新增闸门条件**——Gate 2→3 只校验合同证据；且模式 ① 下建 worktree 的是执行者，
在闸门里查树状态保护不到它，preflight 必须随 handoff 下发。
```

- [ ] **Step 3: Verify**

Run: `grep -n "① 后台代理\|后台代理\|③ 同会话\|模式 ①②" skills/gf-workflow/gates.md`
Expected: no matches.

Run: `grep -n "① 手动新窗口\|② 同会话执行" skills/gf-workflow/gates.md`
Expected: both present.

- [ ] **Step 4: Commit**

```bash
git add skills/gf-workflow/gates.md
git commit -m "docs(gf-workflow): sync GO gate description in gates.md to two modes"
```

---

### Task 4: Cross-file consistency check

**Files:**
- Read-only verification across all three files above

**Interfaces:**
- Consumes: the committed state of Tasks 1-3
- Produces: pass/fail confirmation for Gate 2→3 quality check (`gf-quality`) and the Phase 3 pre-delivery review

- [ ] **Step 1: Repo-wide residue scan**

Run: `grep -rn "Mode ①\|① background agent\|后台代理\|① 后台" skills/gf-workflow/`
Expected: zero matches across `SKILL.md`, `references.md`, `gates.md`.

- [ ] **Step 2: Repo-wide new-menu consistency scan**

Run: `grep -rn "手动新窗口\|manual new window" skills/gf-workflow/`
Expected: at least one match in each of `references.md`, `SKILL.md`, `gates.md`.

- [ ] **Step 3: If `make check-agent-sync` exists, run it**

Run: `grep -q "check-agent-sync" Makefile && make check-agent-sync || echo "target not present, skipping"`

If the target exists, it must pass (no diff between `skills/gf-workflow/` source and any generated `.claude/skills/gf-workflow/` copy left stale).

- [ ] **Step 4: Commit (only if Step 3 or manual fixes touched files)**

```bash
git status --porcelain skills/gf-workflow/
```

If clean (Tasks 1-3 already committed everything), no further commit needed.
