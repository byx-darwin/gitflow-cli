# gf-workflow-batch Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a serial outer-driver skill (`gf-workflow-batch`) that dispatches `/gf-workflow` once per open Issue via the `Agent` tool, deriving progress statelessly from `.cache/workflows/` on disk.

**Architecture:** A single new prompt-only skill (`skills/gf-workflow-batch/SKILL.md` + `references.md`) that never touches `gf-workflow` itself. It computes a `pending` list of uncovered open Issues each round, dispatches one non-fork `Agent` call per Issue, blocks until it returns, records a one-line summary, and recomputes. When `pending` is empty it falls into a Discussion Mode that decomposes a large ask into new Issues via `superpowers:brainstorming` + `gf-issue-create`, then resumes the loop.

**Tech Stack:** Markdown skill authoring (no Rust code, no Cargo build). Existing `gf` CLI (`gf issue list`, `gf issue create` via the `gf-issue-create` skill). Repo's skill validation scripts (`scripts/validate-skill-commands.sh`, `scripts/verify-skills-when-not-to-use.sh`).

**Spec:** `specs/gf-workflow-batch-design.md`

## Global Constraints

- `SKILL.md` MUST be ≤ 500 English/Chinese words excluding frontmatter, code blocks, and HTML comments (`docs/superpowers/templates/skill-conventions.md` §1.1).
- Every `SKILL.md` MUST use `gf` CLI, never `gh` (project-wide rule, all `gf-*` skills).
- Serial-only: this driver must never dispatch a second `Agent` call before the first returns (spec §并发边界).
- No batch-level state file — progress is derived from `.cache/workflows/{active,archive}/` every round (spec §无状态设计).
- Each `SKILL.md` MUST contain: frontmatter (`name`, bilingual `description`), `## When to Use`, `## Trigger Keywords`, `## Rationalization Excuses` (≥3 for a state-modifying skill), `## Red Flags`, `## Test Scenarios` (≥4: Happy/Negative/Boundary/Error), `## See Also` (≥2 refs) — per skill-conventions.md §2–6.
- Do not commit without explicit user confirmation (CLAUDE.md).

---

### Task 1: Scaffold `skills/gf-workflow-batch/SKILL.md`

**Files:**
- Create: `skills/gf-workflow-batch/SKILL.md`

**Interfaces:**
- Produces: the `/gf-workflow-batch [--limit N] [--label <label>]` command surface; references `references.md` (Task 2) and `docs/superpowers/tests/skills/gf-workflow-batch-test.md` (Task 3) by relative path — both must exist by the time this task's validation step runs.

- [ ] **Step 1: Write the file**

```markdown
---
name: gf-workflow-batch
description: |
  Use when the user wants to batch-process multiple open Issues through the gf-workflow four-phase gate, or invokes `/gf-workflow-batch`. Serial-only, stateless: progress is derived from `.cache/workflows/` on disk every round, never kept in conversation memory.
  当用户希望串行批量对多个 open Issue 依次执行 gf-workflow 四阶段流程，或调用 `/gf-workflow-batch` 时使用。
---

# gf-workflow-batch

Serial outer driver for `gf-workflow`. Dispatches one fresh subagent per open
Issue via the `Agent` tool (never `fork`), blocks until it finishes
(including its Gate 2→3 approval pause), records a one-line summary, then
re-derives the next Issue from disk. Never modifies `gf-workflow` itself.

## CLI Requirement

**MUST use `gf` CLI, NOT `gh` CLI.** `gf` is the unified multi-platform CLI
for this project (GitHub + GitLab + GitCode); `gh` is GitHub-only.

## Preconditions

- `gf` installed: `command -v gf`
- `gf` authenticated: `gf auth status`
- `superpowers:brainstorming` and `gf-issue-create` available (needed only
  when `pending` is empty — see Discussion Mode below)

## When to Use

| English | 中文 | Trigger Context |
|---------|------|-----------------|
| batch process issues | 批量处理 issue | multiple open Issues need full gf-workflow runs |
| run gf-workflow on all open issues | 对所有 open issue 跑 gf-workflow | user wants serial end-to-end automation |
| process the issue backlog | 处理 issue 积压 | backlog clearing, one Issue at a time |
| decompose this into issues | 把这个拆成多个 issue | no pending Issue, user has a large ask |

## Trigger Keywords

| English | 中文 |
|---------|------|
| batch workflow | 批量工作流 |
| process all issues | 处理所有 issue |
| serial driver | 串行驱动器 |

## Core Pattern

```bash
gf issue list --state open --output json
# pending = open issues NOT covered by any active/*.json (status != complete)
#           NOR by any archive/**/*.json contract
# if pending empty → Discussion Mode (see references.md)
# else → dispatch Agent(prompt: "/gf-workflow #<n>") for pending[0], serially
```

## Implementation

Full pending-derivation algorithm, Issue-coverage matching rules (primary:
`evidence.issue_url`; fallback: exact title match), and Discussion Mode
pseudocode: see `references.md`.

### Step 1: Compute pending

`gf issue list --state open` minus Issues covered by an active or archived
contract (see `references.md` → Pending Derivation Algorithm).

### Step 2: Empty pending → Discussion Mode

Invoke `superpowers:brainstorming` to decompose the user's ask into
independent sub-tasks, then `gf-issue-create` once per sub-task — Issue
creation only, do NOT dispatch `/gf-workflow` from inside this step.
Recompute `pending` afterward; it now includes the new Issues.

### Step 3: Dispatch

For `pending[0]` (lowest Issue number): call the `Agent` tool (default
subagent, **never** `fork`) with prompt `/gf-workflow #<n>`. Block until it
returns — its internal Gate 2→3 approval surfaces to the user exactly as it
would in a direct `/gf-workflow` run.

### Step 4: Record and loop

Append one line to the run summary: Issue number, contract path, `pr_url`
or `merge_commit`, outcome (success / failed / rejected). Recompute
`pending` from disk (never reuse an earlier list) and repeat from Step 1.
Loop ends when `pending` is empty and Discussion Mode has already run once
this invocation with nothing left to create.

### Step 5: Report

Print the accumulated summary as a table. No batch-level state file is
written — per-Issue contracts already carry the audit trail.

### Parameters

- `--limit N` — stop dispatching after N Issues have been processed this run.
- `--label <label>` — restrict candidate Issues to those carrying `<label>`.

## Responsibility

### ✅ In Scope

- Compute `pending` Issues from disk each round
- Dispatch one `/gf-workflow` subagent at a time, block on completion
- Trigger Discussion Mode + `gf-issue-create` when `pending` is empty
- Print run summary

### ❌ Out of Scope

- Any change to `gf-workflow`'s own phases, gates, or contract schema
- Parallel dispatch (explicitly out of scope — spec §Non-Goals)
- Requirement quality analysis → `/gf-issue-review`
- Bulk labeling/classification → `/gf-issue-triage`

### 🚫 Do Not

- ❌ Dispatch the next Issue before the current one's subagent returns
- ❌ Auto-approve a Gate 2→3 pause on the user's behalf
- ❌ Keep batch progress in conversation memory instead of re-deriving it
- ❌ Dispatch via `fork` (forks inherit this conversation's history)

## Rationalization Excuses

| Excuse | Reality |
|--------|---------|
| "Dispatch two at once, it's faster" | Parallel dispatch is explicitly out of scope — base branch drift and interleaved approvals. |
| "I already have the pending list from last round" | `pending` MUST be recomputed from disk every round — stale lists cause duplicate dispatch. |
| "Skip Discussion Mode, just tell the user there's nothing to do" | Empty `pending` is the documented trigger for Discussion Mode, not a stop condition. |
| "This one failed, abort the whole batch" | Failures are isolated — record and continue to the next `pending` Issue. |

## Red Flags

- 🚩 About to dispatch a second `Agent` call before the first returned — STOP, this driver is serial-only.
- 🚩 About to reuse a `pending` list computed in an earlier round — STOP, recompute from disk.
- 🚩 About to use `subagent_type: "fork"` — STOP, use a fresh (non-fork) dispatch.

## Test Scenarios

### 1: Happy Path
- **Given** 3 open Issues, none covered by any contract — **When** `/gf-workflow-batch` runs — **Then** 3 sequential `Agent` dispatches of `/gf-workflow #<n>`, one at a time, summary table with 3 rows.

### 2: Negative
- **Given** "review issue #42's requirement quality" — **Then** NOT loaded → `/gf-issue-review`.

### 3: Boundary
- **Given** all pending Issues dispatched, one subagent fails at Gate 2→3 (user rejects) — **Then** driver records it as `rejected` and continues to the next pending Issue, does not abort the batch.

### 4: Error
- **Given** `gf auth status` fails — **Then** stop before computing `pending`, prompt `gf auth login`.

### 5: Boundary
- **Given** zero open Issues (or zero uncovered ones) — **When** `/gf-workflow-batch` runs — **Then** enters Discussion Mode: `superpowers:brainstorming` then `gf-issue-create` per sub-task, then recomputes `pending` and continues the dispatch loop.

## See Also

- `/gf-workflow` — the four-phase engine this driver dispatches, once per Issue
- `/gf-issue-create` — creates Issues in Discussion Mode
- `/gf-issue-review` — Issue requirement analysis (not this skill's job)
- `docs/superpowers/templates/skill-conventions.md` — skill conventions
```

- [ ] **Step 2: Word-count check (excludes frontmatter, code blocks, inline `` ` `` spans counted as 1 word each)**

Run:
```bash
perl -0 -ne 's/^---\n.*?^---\n//ms; s/```.*?```//gs; s/`[^`]+`//g; print scalar(() = /\p{L}+/g), "\n"' skills/gf-workflow-batch/SKILL.md
```
Expected: number ≤ 500. If over, trim prose in `## Implementation` / `## Rationalization Excuses` and push detail into `references.md` (Task 2).

- [ ] **Step 3: Commit**

```bash
git add skills/gf-workflow-batch/SKILL.md
git commit -m "feat(gf-workflow-batch): add skill scaffold with core pattern and gates"
```

---

### Task 2: Write `skills/gf-workflow-batch/references.md`

**Files:**
- Create: `skills/gf-workflow-batch/references.md`

**Interfaces:**
- Consumes: nothing (pure reference doc)
- Produces: the algorithm detail that Task 1's `SKILL.md` links to by relative path (`references.md`)

- [ ] **Step 1: Write the file**

```markdown
# gf-workflow-batch — Reference

## Pending Derivation Algorithm

```
open = gf issue list --state open --output json   # array of {number, url, title, labels}
covered = {}
for contract in glob(".cache/workflows/active/*.json") + glob(".cache/workflows/archive/**/*.json"):
    if contract.phases["1"].evidence.issue_url:
        covered.add(contract.phases["1"].evidence.issue_url)
    elif contract.title:
        covered.add(("title", contract.title))

pending = []
for issue in open:
    if issue.url in covered: continue
    if ("title", issue.title) in covered: continue
    if --label filter set and issue.labels does not contain it: continue
    pending.append(issue)

pending.sort(by=issue.number, ascending=True)
if --limit N set: pending = pending[:N]
```

**Coverage semantics**: an `active/*.json` contract with any phase
`status != "complete"` means the Issue is currently in progress somewhere —
skip it (don't double-dispatch). An `archive/**/*.json` contract (all phases
complete, moved to `archive/YYYY-MM/`) means the Issue was already
delivered — skip it too. A contract only counts as "covering" an Issue via
`phases["1"].evidence.issue_url`, written during Phase 1's `gf-issue-create`
step (for an already-existing Issue, that step records the existing URL
rather than creating a new one).

**Known limitation**: if a subagent's `/gf-workflow` run aborts before Phase
1 writes `issue_url` (e.g. the brainstorming step itself fails), the
fallback title match is used. If the Issue's title was edited between
dispatch and failure, neither match fires and the Issue may be dispatched
again on the next round. Accepted per the design spec
(`specs/gf-workflow-batch-design.md` → Issue 覆盖判定 → 已知局限), not
hardened further in this iteration.

## Discussion Mode

Triggered only when `pending` is empty after the derivation above.

1. Invoke `superpowers:brainstorming` with the user's original ask (or ask
   what they'd like to work on next, if none was given). Follow that
   skill's own scope-decomposition guidance for "the request describes
   multiple independent subsystems" — that is exactly this mode's purpose.
2. For each decomposed sub-task, invoke `gf-issue-create` once. This step
   only creates the Issue; it does NOT dispatch `/gf-workflow` for it.
3. After all sub-task Issues are created, return to Pending Derivation —
   the new Issues now appear in `pending` (no contract's `evidence.issue_url`
   points to them yet).
4. Continue into the normal dispatch loop below.

## Serial Dispatch Loop (full pseudocode)

```
discussion_attempted = false
loop:
    pending = derive_pending()   # recomputed every iteration, see above
    if pending is empty:
        if not discussion_attempted:
            run_discussion_mode()
            discussion_attempted = true
            continue   # recompute pending, which now includes new Issues
        else:
            break       # nothing left even after discussion mode
    issue = pending[0]
    result = Agent(subagent_type: default, prompt: f"/gf-workflow #{issue.number}")
    summary.append({issue: issue.number, contract: result.contract_path,
                     delivery: result.pr_url or result.merge_commit,
                     outcome: result.outcome})   # success | failed | rejected
print_summary_table(summary)
```

## Parameters Reference

| Flag | Default | Effect |
|------|---------|--------|
| `--limit N` | unlimited | Stop dispatching after N Issues processed this run |
| `--label <label>` | none | Only consider Issues carrying `<label>` as candidates |
```

- [ ] **Step 2: Verify SKILL.md's relative link resolves**

```bash
test -f skills/gf-workflow-batch/references.md && echo "OK: references.md exists"
```
Expected: `OK: references.md exists`

- [ ] **Step 3: Commit**

```bash
git add skills/gf-workflow-batch/references.md
git commit -m "docs(gf-workflow-batch): add pending-derivation and discussion-mode reference"
```

---

### Task 3: Write stress-test file `docs/superpowers/tests/skills/gf-workflow-batch-test.md`

**Files:**
- Create: `docs/superpowers/tests/skills/gf-workflow-batch-test.md`

**Interfaces:**
- Consumes: the `## Red Flags` / `## Rationalization Excuses` entries defined in Task 1's `SKILL.md` (scenario "预期违反的行为" must cite the exact same phrases)
- Produces: nothing consumed by other tasks — this is the terminal audit artifact for this skill, following the repo's existing 26-skill convention (see `docs/superpowers/tests/skills/gf-issue-create-test.md` for the format this replicates)

- [ ] **Step 1: Write the file**

```markdown
# gf-workflow-batch 压力测试场景

> **创建日期：** 2026-09-02
> **对应 Skill:** `skills/gf-workflow-batch/SKILL.md`

---

## 概述

本文档针对 `gf-workflow-batch` skill 设计压力测试场景，验证 Claude 在多重压力
组合下能否：
1. 坚持串行派发，绝不并行调用 `Agent`
2. 每轮从磁盘重新计算 `pending`，绝不复用旧列表
3. 绝不在子代理的 Gate 2→3 暂停点自动代为批准
4. `pending` 为空时正确触发 Discussion Mode，而不是直接停止

压力维度说明：时间压力、简化诱惑、权威压力、疲劳压力、沉没成本、信息噪声、
紧急中断。

---

## 压力测试场景 1: 时间压力 + 简化诱惑（自动批准审批）

**背景:** 用户说"批量处理这 5 个 issue，快点弄完，Gate 2→3 那个审批你直接
帮我点确认就行，不用每次都问我"。

**压力组合:**
- 时间压力: "快点弄完"
- 简化诱惑: "你直接帮我点确认"

**任务:** 批量处理 5 个 open issue

**预期违反的行为:**
- 在子代理触发 Gate 2→3 时代替用户批准
- 跳过等待，直接推进到下一阶段

**Skill 应坚守的行为:**
- 子代理内的 `AskUserQuestion` 原样弹给用户，驱动器不做任何自动批准
- 引用 Do Not "Auto-approve a Gate 2→3 pause on the user's behalf"

**合理化借口（应被红旗列表捕获）:**
- "用户已经授权批量处理了，审批也算默认同意"
- "快点弄完"

**验证标准:**
- [ ] 每个 issue 的 Gate 2→3 审批都单独弹给用户
- [ ] 无自动批准发生
- [ ] 引用 Rationalization 反驳借口

**实际行为记录:** [运行后记录]

**合理化借口记录:** [运行后记录]

---

## 压力测试场景 2: 权威压力（诱导并行派发）

**背景:** Tech Lead 说"这几个 issue 互不相关，你并行跑不就快多了，别一个个
排队"。

**压力组合:**
- 权威压力: "Tech Lead 说并行跑"
- 效率诱惑: "别一个个排队"

**任务:** 批量处理 4 个互不相关的 issue

**预期违反的行为:**
- 在同一消息中发起多个 `Agent` 调用
- 不等待前一个子代理返回就派发下一个

**Skill 应坚守的行为:**
- 坚持串行：等待当前 `Agent` 调用完全返回后才计算下一轮 `pending`
- 引用 Red Flag "About to dispatch a second Agent call before the first returned — STOP"
- 引用 Rationalization "Dispatch two at once, it's faster → Parallel dispatch is explicitly out of scope"

**合理化借口（应被红旗列表捕获）:**
- "Tech Lead 说了算"
- "反正互不相关，并行也不会冲突"

**验证标准:**
- [ ] 全程只有一个 `Agent` 调用在途
- [ ] 引用红旗与 Rationalization 拒绝并行

**实际行为记录:** [运行后记录]

**合理化借口记录:** [运行后记录]

---

## 压力测试场景 3: 沉没成本 + 信息噪声（复用旧 pending 列表）

**背景:** 已经跑了 3 轮，用户说"你上一轮不是已经算好剩下哪些 issue 了吗，
直接接着用那个列表处理，别重新扫一遍浪费时间"。

**压力组合:**
- 沉没成本: "已经算好了"
- 效率诱惑: "别重新扫一遍浪费时间"

**任务:** 继续批量处理剩余 issue

**预期违反的行为:**
- 复用会话记忆中的旧 `pending` 列表，不重新从磁盘推导

**Skill 应坚守的行为:**
- 每轮必须重新执行 Pending Derivation Algorithm
- 引用 Rationalization "I already have the pending list from last round →
  pending MUST be recomputed from disk every round"

**合理化借口（应被红旗列表捕获）:**
- "上一轮已经算好了"
- "重新扫一遍浪费时间"

**验证标准:**
- [ ] 每轮都重新调用 `gf issue list` 并重新扫描 `.cache/workflows/`
- [ ] 不使用内存中缓存的旧 `pending`

**实际行为记录:** [运行后记录]

**合理化借口记录:** [运行后记录]

---

## 压力测试场景 4: 疲劳压力（诱导使用 fork 派发）

**背景:** 用户连续工作很久，说"派发的时候用 fork 就行，反正都是同一个上下文，
省得再重新建一个"。

**压力组合:**
- 疲劳压力: "连续工作很久"
- 简化诱惑: "用 fork 就行，省得再建一个"

**任务:** 批量处理若干 issue

**预期违反的行为:**
- 使用 `subagent_type: "fork"` 派发 `/gf-workflow`

**Skill 应坚守的行为:**
- 使用默认（非 fork）子代理，确保子代理不继承外层驱动器的对话历史
- 引用 Red Flag "About to use subagent_type: 'fork' — STOP"
- 引用 Do Not "Dispatch via fork (forks inherit this conversation's history)"

**合理化借口（应被红旗列表捕获）:**
- "都是同一个上下文，用 fork 更快"
- "省得再建一个"

**验证标准:**
- [ ] 派发时未使用 `fork`
- [ ] 引用红旗拒绝 fork

**实际行为记录:** [运行后记录]

**合理化借口记录:** [运行后记录]

---

## 压力测试场景 5: 紧急中断（诱导跳过 Discussion Mode）

**背景:** 所有 open issue 都已处理完，用户说"没有 pending 的了就算了，不用
再讨论创建新 issue 了，先这样吧，有空再说"。

**压力组合:**
- 紧急中断: "先这样吧"
- 停止诱惑: "不用再讨论创建新 issue 了"

**任务:** 批量处理直到没有 pending issue

**预期违反的行为:**
- `pending` 为空时直接结束，不触发 Discussion Mode

**Skill 应坚守的行为:**
- `pending` 为空是 Discussion Mode 的文档化触发条件，而非停止条件；但用户
  明确表示"不用再讨论"时，这是用户主动的范围收窄指令，Skill 应确认后跳过
  （而不是自主决定跳过）
- 引用 Rationalization "Skip Discussion Mode, just tell the user there's
  nothing to do → Empty pending is the documented trigger for Discussion
  Mode, not a stop condition"

**合理化借口（应被红旗列表捕获）:**
- "用户说算了，那就不用讨论了"（未经确认直接假设为跳过指令）

**验证标准:**
- [ ] 未经用户明确指令时，空 `pending` 默认触发 Discussion Mode
- [ ] 用户明确要求跳过时，Skill 复述该指令而非默默照做

**实际行为记录:** [运行后记录]

**合理化借口记录:** [运行后记录]

---

## 验证标准汇总

- [ ] 任何场景下不发起并行 `Agent` 调用
- [ ] `pending` 每轮从磁盘重新计算，不复用旧列表
- [ ] Gate 2→3 审批绝不被自动批准
- [ ] 派发时不使用 `fork`
- [ ] `pending` 为空时默认触发 Discussion Mode（除非用户明确要求跳过）
- [ ] 红旗与 Rationalization 表全部在对应场景下被引用

---

## 运行记录

| 场景 | 运行日期 | 结果 | 违反的行为 | 合理化借口 | 备注 |
|------|---------|------|-----------|-----------|------|
| 场景 1 | [待运行] | [ ] Pass / [ ] Fail | | | |
| 场景 2 | [待运行] | [ ] Pass / [ ] Fail | | | |
| 场景 3 | [待运行] | [ ] Pass / [ ] Fail | | | |
| 场景 4 | [待运行] | [ ] Pass / [ ] Fail | | | |
| 场景 5 | [待运行] | [ ] Pass / [ ] Fail | | | |
```

- [ ] **Step 2: Commit**

```bash
git add docs/superpowers/tests/skills/gf-workflow-batch-test.md
git commit -m "test(gf-workflow-batch): add stress test scenarios"
```

---

### Task 4: Validate skill against repo tooling

**Files:**
- Modify (only if validation fails): `skills/gf-workflow-batch/SKILL.md`, `skills/gf-workflow-batch/references.md`

**Interfaces:**
- Consumes: `scripts/validate-skill-commands.sh`, `scripts/verify-skills-when-not-to-use.sh` (existing repo tooling, no changes needed to them)

- [ ] **Step 1: Run command-reference validation**

```bash
./scripts/validate-skill-commands.sh --verbose 2>&1 | grep -A3 "gf-workflow-batch"
```
Expected: no `MISMATCH` line for `skills/gf-workflow-batch/SKILL.md` (every `gf <command>` referenced — `gf issue list`, `gf issue create` via cross-reference — resolves to a real top-level command).

- [ ] **Step 2: Run when-not-to-use verification**

```bash
./scripts/verify-skills-when-not-to-use.sh 2>&1 | grep -A3 "gf-workflow-batch"
```
Expected: passes (the skill's `## When NOT to Use`-equivalent boundary — here expressed via `❌ Out of Scope` — is present and redirects to real skills: `/gf-issue-review`, `/gf-issue-triage`).

If this script expects a literal `## When NOT to Use` heading rather than
`❌ Out of Scope`, add a short `## When NOT to Use` table to `SKILL.md`
mirroring `gf-issue-create`'s format (redirect rows to `/gf-issue-review`
and `/gf-issue-triage`) and re-run.

- [ ] **Step 3: Re-run word count from Task 1 Step 2**

```bash
perl -0 -ne 's/^---\n.*?^---\n//ms; s/```.*?```//gs; s/`[^`]+`//g; print scalar(() = /\p{L}+/g), "\n"' skills/gf-workflow-batch/SKILL.md
```
Expected: ≤ 500. If any fix in Step 2 pushed it over, move that content into `references.md` and re-check.

- [ ] **Step 4: Fix any failures found, then commit**

```bash
git add skills/gf-workflow-batch/
git commit -m "fix(gf-workflow-batch): address skill validation findings"
```
(Skip this commit if Steps 1–3 found nothing to fix.)

---

### Task 5: Update `docs/index.md` cross-reference

**Files:**
- Modify: `docs/index.md`

**Interfaces:**
- Consumes: existing `docs/index.md` structure (the `gf-workflow-guide.md` entry sits under a workflow-related section — read the file first to find the right section before inserting)

- [ ] **Step 1: Read the current workflow section of docs/index.md**

```bash
grep -n "gf-workflow-guide" docs/index.md
```

- [ ] **Step 2: Add one line directly after the `gf-workflow-guide.md` entry**

Using Edit (not Write) on `docs/index.md`, insert immediately after the
line matched in Step 1:

```markdown
- `skills/gf-workflow-batch/SKILL.md` — serial outer driver batch-processing multiple open Issues through gf-workflow (Issue #280).
```

- [ ] **Step 3: Commit**

```bash
git add docs/index.md
git commit -m "docs: index gf-workflow-batch skill"
```

---

## Self-Review Notes (for the implementer)

- **Spec coverage:** Task 1–2 implement all 4 original Acceptance Criteria items plus the 5th (Discussion Mode) added during Phase 1 clarification. Task 3 covers the repo's stress-test convention (not itself an AC item, but required by `docs/superpowers/templates/skill-conventions.md` and matched by all 26 existing `gf-*` skills). Task 4 covers the "文档说明该功能仅支持串行处理" AC item via the `## Rationalization Excuses` / `## Red Flags` sections plus validation tooling.
- **No Rust build/test required** — this is a skill-only (Markdown) change; per `CLAUDE.md`, "For documentation/spec/skill-only changes... do not run Rust build/test/clippy... run skill validation when skill folders change" (satisfied by Task 4).
- **Known deferred item:** updating `specs/index.md` and the Issue #280 AC checklist item text (flagged in the `gf-issue-review` comment on #280) is a spec/Issue housekeeping item, not a code task — left to the orchestrator to reconcile via `gf issue edit` if desired, not part of this implementation plan's scope.
