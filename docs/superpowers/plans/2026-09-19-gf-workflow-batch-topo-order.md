# gf-workflow-batch 依赖边拓扑排序 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让 `gf-workflow-batch` 的 Pending Derivation 在派发前解析 `Blocked by: #N` 依赖边，跳过未完成前置、检测成环并报错终止，无依赖声明的 Issue 保持原有编号升序。

**Architecture:** 纯 prompt/伪代码文档改动。在 `references.md` 的 Pending Derivation 与 Serial Dispatch Loop 之间插入一个新阶段"Dependency Resolution"，产出 `ready ⊆ pending` 供 `candidates` 取数；`SKILL.md` 同步更新 Core Pattern 描述与 Test Scenarios。没有 Rust 代码改动，因此没有 `cargo test`；用一次性脚本先验证正则/DFS 伪代码的正确性，再誊写进文档。

**Tech Stack:** Markdown（skill 指令文档），Python3（仅用于一次性验证伪代码逻辑，不落盘不提交）。

**Spec:** `docs/superpowers/specs/2026-09-19-gf-workflow-batch-topo-order-design.md`

## Global Constraints

- 不改动 `gf-workflow` 自身的四阶段/闸门/合同 schema（`skills/gf-workflow-batch/SKILL.md` 现有 Out of Scope 条款）。
- 不引入并发派发（Serial Dispatch Loop 骨架不变，只替换 `candidates` 取数源）。
- 前置"已完成"判定 = `gf issue view <N>` 的 `state == closed`（设计 §2）。
- 依赖引用的 Issue 不存在 → 报错终止，不静默忽略（设计 §2、§3.3）。
- 成环 → 报错并打印环路径，不派发任何 Issue（设计 §2、§3.4）。
- 依赖解析范围是**全部 open Issue**，不只是 `pending` 子集（设计 §3.2）。
- 无依赖声明的 Issue 保持原有 `issue.number` 升序（设计 §3.5）。
- 这是文档改动：不运行 `cargo build`/`cargo test`/`cargo clippy`；改动落地后运行 `make check-agent-sync`（CLAUDE.md 对 skill-only 改动的验证要求）。

---

## File Structure

- **Modify:** `skills/gf-workflow-batch/references.md` — 插入 Dependency Resolution 算法段落，改写 Serial Dispatch Loop 伪代码。
- **Modify:** `skills/gf-workflow-batch/SKILL.md` — Core Pattern 描述、Implementation 段落、Test Scenarios 追加 6 条场景。
- **Modify:** `skills/gf-issue-decompose/references/dependency-edges.md:30-34` — 消除"该消费者目前不存在，Issue #337 跟踪"的过期声明，改为指向已交付的 `gf-workflow-batch` 依赖解析阶段。

无新文件；三处改动都在既有文档内，边界清晰（driver 算法 / driver 说明与测试场景 / 上游过期声明），可独立提交但属于同一个 Issue，按顺序在一次 PR/合并中交付。

---

### Task 1: 验证依赖边正则提取逻辑

**Files:**
- 无需创建/修改任何仓库文件——本任务是用一次性脚本验证设计文档 §3.2 的正则表达式，验证通过后其文本原样誊写进 Task 3。

**Interfaces:**
- 消费：设计文档 §3.2 的正则 `Blocked by:\s*((?:#\d+(?:,\s*)?)+)` 与外层 `#(\d+)`。
- 产出：确认无误的正则文本，供 Task 3 直接引用。

- [ ] **Step 1: 编写验证脚本并运行**

```bash
python3 - <<'EOF'
import re

def extract_edges(body: str) -> set[int]:
    edges = set()
    for group in re.findall(r'Blocked by:\s*((?:#\d+(?:,\s*)?)+)', body):
        for blocker in re.findall(r'#(\d+)', group):
            edges.add(int(blocker))
    return edges

cases = [
    ("单个引用", "## Context\nBlocked by: #12\n", {12}),
    ("多个引用逗号分隔", "Blocked by: #12, #14\n", {12, 14}),
    ("大小写/空格容错", "Blocked by:   #12,#14\n", {12, 14}),
    ("无声明", "## Context\n没有依赖\n", set()),
    ("多行声明取并集", "Blocked by: #12\n...\nBlocked by: #14\n", {12, 14}),
    ("正文提到 Blocked by 但非声明行不应误判", "本设计讨论 Blocked by 依赖边的解析方式。\n", set()),
]

failed = False
for name, body, expected in cases:
    got = extract_edges(body)
    status = "OK" if got == expected else "FAIL"
    if got != expected:
        failed = True
    print(f"[{status}] {name}: got={got} expected={expected}")

import sys
sys.exit(1 if failed else 0)
EOF
```

Expected: 每个用例打印 `[OK]`，脚本以状态码 0 退出。若"正文提到 Blocked by 但非声明行不应误判"用例出现 `FAIL`（正则会匹配任意含 `Blocked by:` 后跟 `#数字` 的文本，这条用例故意没有冒号后紧跟 `#`，所以应为 `OK`；这一步是确认设计里的正则不会对纯 prose 提及误报），据此判断正则是否需要收紧（例如要求整行以 `Blocked by:` 开头）。

- [ ] **Step 2: 根据结果确认或修正设计文档的正则**

若 Step 1 全部 `[OK]`，正则维持设计文档 §3.2 原样，进入 Task 3。若有 `FAIL`，先修正正则、重跑 Step 1 直到全 `OK`，并在 Task 3 落地时使用修正后的版本（不需要回写设计文档本身，设计文档已在 Phase 1 提交，实现阶段的微调记录在 PR/commit message 里即可）。

---

### Task 2: 验证三色 DFS 成环检测逻辑

**Files:**
- 同 Task 1，无需创建/修改仓库文件。

**Interfaces:**
- 消费：设计文档 §3.4 的三色 DFS 伪代码。
- 产出：确认无误的 DFS 逻辑，供 Task 3 直接引用。

- [ ] **Step 1: 编写验证脚本并运行**

```bash
python3 - <<'EOF'
import sys

WHITE, GRAY, BLACK = 0, 1, 2

def find_cycle(edges: dict[int, set[int]]) -> list[int] | None:
    color = {}
    path = []
    result = {"cycle": None}

    def dfs(n):
        color[n] = GRAY
        path.append(n)
        for b in edges.get(n, []):
            if color.get(b, WHITE) == GRAY:
                result["cycle"] = path[path.index(b):] + [b]
                return True
            if color.get(b, WHITE) == WHITE:
                if dfs(b):
                    return True
        path.pop()
        color[n] = BLACK
        return False

    for n in list(edges):
        if color.get(n, WHITE) == WHITE:
            if dfs(n):
                return result["cycle"]
    return None

cases = [
    ("无环-线性链", {1: {2}, 2: {3}}, None),
    ("无环-空图", {}, None),
    ("直接互环", {1: {2}, 2: {1}}, [1, 2, 1]),
    ("三节点环", {1: {2}, 2: {3}, 3: {1}}, [1, 2, 3, 1]),
    ("环+无关分支不误报", {1: {2}, 2: {1}, 3: {4}}, [1, 2, 1]),
]

failed = False
for name, edges, expected in cases:
    got = find_cycle(edges)
    ok = (got == expected) or (expected is not None and got is not None and set(got) == set(expected))
    status = "OK" if ok else "FAIL"
    if not ok:
        failed = True
    print(f"[{status}] {name}: got={got} expected={expected}")

sys.exit(1 if failed else 0)
EOF
```

Expected: 每个用例打印 `[OK]`，脚本以状态码 0 退出，确认三色 DFS 能在互环、三节点环上正确报出环路径，在无环图上返回 `None`，且不误报与环无关的分支。

- [ ] **Step 2: 确认设计文档 §3.4 伪代码与验证一致**

若全 `OK`，Task 3 直接誊写设计文档 §3.4 的伪代码；若发现设计文档的伪代码与本任务验证版本有出入（例如颜色回溯时机），以本任务验证通过的版本为准落地到 Task 3。

---

### Task 3: 改写 `references.md` — Dependency Resolution 算法段落

**Files:**
- Modify: `skills/gf-workflow-batch/references.md`

**Interfaces:**
- 消费：Task 1/Task 2 验证通过的正则与 DFS 伪代码。
- 产出：`references.md` 新增一节 `## Dependency Resolution`（置于 `## Discussion Mode` 与 `## Serial Dispatch Loop` 之间），并改写 `## Serial Dispatch Loop (full pseudocode)` 一节的 `candidates` 取数逻辑，供 `SKILL.md`（Task 4）引用。

- [ ] **Step 1: 在 `## Discussion Mode` 小节之后、`## Serial Dispatch Loop` 小节之前插入新小节**

在 `references.md` 第 54 行（`## Discussion Mode` 一节结束、`## Serial Dispatch Loop (full pseudocode)` 开始之前）插入：

```markdown
## Dependency Resolution

Runs every round, between Pending Derivation and the Serial Dispatch Loop's
`candidates` selection. Scope is **all open Issues**, not just `pending` —
an Issue already covered by an active contract can still be another
pending Issue's blocker, or a cycle member, and would be missed if only
`pending` were scanned.

### Parsing `Blocked by` edges

```python
import re

open_issues = gf_issue_list_open()                     # existing, unchanged
bodies = {i.number: gf_issue_view(i.number).body for i in open_issues}

def extract_edges(body: str) -> set[int]:
    edges = set()
    for group in re.findall(r'Blocked by:\s*((?:#\d+(?:,\s*)?)+)', body):
        for blocker in re.findall(r'#(\d+)', group):
            edges.add(int(blocker))
    return edges

edges = {i.number: extract_edges(bodies[i.number]) for i in open_issues}
# edges: blocked_issue_number -> set(blocker_issue_number)
```

Multiple `Blocked by:` lines in one body union their references (see
`skills/gf-issue-decompose/references/dependency-edges.md` for the
declaration format this parses — one edge per referenced number, declared
on the blocked ticket only).

### Resolving blocker completion

An Issue number that does not appear in `open_issues` is either closed or
does not exist — the two are distinguished with one extra lookup:

```python
open_numbers = {i.number for i in open_issues}

for blocked, blockers in list(edges.items()):
    for b in list(blockers):
        if b in open_numbers:
            continue   # still open — edge stays, feeds cycle detection below
        view = gf_issue_view(b)   # not in open list: closed, or doesn't exist
        if view is None:
            raise WorkflowBatchError(
                f"Issue #{blocked} 的 Blocked by 引用了不存在的 #{b}"
            )
        # view.state == "closed" — satisfied, drop the edge
        blockers.discard(b)
```

`WorkflowBatchError` here means: **stop before dispatching anything this
round**, surface the message to the user, do not enter Discussion Mode, do
not fall back to a partial dispatch.

### Cycle detection

Only edges where both ends are still open can participate in a cycle
(a closed blocker already had its edge dropped above). Three-color DFS:

```python
WHITE, GRAY, BLACK = 0, 1, 2

def find_cycle(edges: dict[int, set[int]]) -> list[int] | None:
    color = {}
    path = []
    result = {"cycle": None}

    def dfs(n):
        color[n] = GRAY
        path.append(n)
        for b in edges.get(n, []):
            if color.get(b, WHITE) == GRAY:
                result["cycle"] = path[path.index(b):] + [b]
                return True
            if color.get(b, WHITE) == WHITE:
                if dfs(b):
                    return True
        path.pop()
        color[n] = BLACK
        return False

    for n in list(edges):
        if color.get(n, WHITE) == WHITE:
            if dfs(n):
                return result["cycle"]
    return None

cycle = find_cycle(edges)
if cycle is not None:
    raise WorkflowBatchError(
        "依赖成环: " + " → ".join(f"#{n}" for n in cycle)
    )
```

Same stop semantics as the missing-reference case above: found a cycle →
stop before dispatching anything, surface the cycle path, do not
auto-break it (re-slicing is the user's call, per
`skills/gf-issue-decompose/references/dependency-edges.md`'s "one
direction only; a cycle means the slices are wrong").

### Computing the ready set

```python
ready = [i for i in pending if not edges.get(i.number)]
ready.sort(key=lambda i: i.number)   # same tie-break as existing pending sort;
                                      # Issues without any Blocked by declaration
                                      # keep their original relative order
```

`pending` Issues not in `ready` (unsatisfied blockers remain) stay in
`pending` but are excluded from this round's `candidates`. The next round
re-derives `open_issues`/`bodies` from scratch — once a blocker closes
(via `gf-workflow` delivery or a manual close), the edge no longer
survives the "not in `open_numbers`" check above and the blocked Issue
becomes ready with no extra persisted state.
```

- [ ] **Step 2: 改写 `## Serial Dispatch Loop (full pseudocode)` 小节**

将现有伪代码块（第 72-99 行左右，`discussion_attempted = false` 起，`print_summary_table(summary)` 止）中的这一行：

```
    pending = derive_pending()   # recomputed every iteration, see above
```

改为：

```
    pending = derive_pending()   # recomputed every iteration, see above
    ready = resolve_dependencies(pending, open_issues)   # see Dependency Resolution above;
                                                          # raises WorkflowBatchError → abort the
                                                          # whole command, no dispatch this run
```

并将：

```
    candidates = [i for i in pending if i.number not in attempted]
    if candidates is empty:
        break           # everything remaining was already attempted this run — stuck
```

改为：

```
    candidates = [i for i in ready if i.number not in attempted]
    if candidates is empty:
        break           # ready 集合已耗尽，或全部候选本轮已尝试过——正常停止，
                         # 不是错误：pending 里可能还有 Issue 在等前置关闭
```

`if pending is empty:` 分支（触发 Discussion Mode）保持不变，仍然判断 `pending` 而非 `ready`——`pending` 为空才代表"没有未覆盖的 open Issue 了"，`ready` 为空但 `pending` 非空属于"全部候选都在等前置"，走上面改写后的 `candidates is empty` 分支。

- [ ] **Step 3: 检查改写后的 `references.md` 无残留旧引用**

```bash
grep -n "candidates = \[i for i in pending" skills/gf-workflow-batch/references.md
```

Expected: 无匹配（该行已被 Step 2 替换为 `for i in ready`）。

- [ ] **Step 4: Commit**

```bash
git add skills/gf-workflow-batch/references.md
git commit -m "feat(gf-workflow-batch): add dependency-edge resolution to pending derivation (#337)"
```

---

### Task 4: 改写 `SKILL.md` — Core Pattern / Implementation / Test Scenarios

**Files:**
- Modify: `skills/gf-workflow-batch/SKILL.md`

**Interfaces:**
- 消费：Task 3 产出的 `references.md` 新小节名（`Dependency Resolution`）与改写后的 Serial Dispatch Loop。
- 产出：`SKILL.md` 的 Core Pattern/Implementation 段落引用 Dependency Resolution，Test Scenarios 新增 6 条，供本 Issue 验收对照与后续人工走查。

- [ ] **Step 1: 改写 Core Pattern 代码块**

将 `SKILL.md` 第 53-59 行的代码块：

```
gf issue list --state open --output json
# pending = open issues NOT covered by any active/*.json (status != complete)
#           NOR by any archive/**/*.json contract
# if pending empty → Discussion Mode (see references.md)
# else → dispatch Agent(prompt: "/gf-workflow #<n>") for pending[0], serially
```

改为：

```
gf issue list --state open --output json
# pending = open issues NOT covered by any active/*.json (status != complete)
#           NOR by any archive/**/*.json contract
# ready = pending Issues whose `Blocked by: #N` refs are all closed
#         (parsed from all open Issues' bodies; missing ref or a cycle
#         aborts the whole run — see references.md § Dependency Resolution)
# if pending empty → Discussion Mode (see references.md)
# else → dispatch Agent(prompt: "/gf-workflow #<n>") for ready[0], serially
```

- [ ] **Step 2: 改写 Implementation 段落首句**

将第 63-68 行：

```
Each round: compute `pending`; empty triggers Discussion Mode, then
recompute. Otherwise dispatch `pending[0]` via `Agent` (never `fork`),
block until it returns (including Gate 2→3), append a summary line, and
loop. Stop and print the summary table once `pending` is empty and
Discussion Mode already ran with nothing left to create. Full algorithm:
see `references.md`.
```

改为：

```
Each round: compute `pending`, then resolve `Blocked by` edges across all
open Issues to derive `ready ⊆ pending` (missing reference or a cycle
aborts the whole run before any dispatch). Empty `pending` triggers
Discussion Mode, then recompute. Otherwise dispatch `ready[0]` via `Agent`
(never `fork`), block until it returns (including Gate 2→3), append a
summary line, and loop. Stop and print the summary table once `pending`
is empty and Discussion Mode already ran with nothing left to create, or
once every remaining `ready` candidate has been attempted this run. Full
algorithm: see `references.md`.
```

- [ ] **Step 3: 在 `## Test Scenarios` 小节末尾（现有场景 6 之后、`## See Also` 之前）追加 6 条新场景**

```markdown
### 7: Happy Path
- **Given** 3 pending Issues, none declares `Blocked by` — **When** `/gf-workflow-batch` runs — **Then** dispatch order unchanged from plain `issue.number` ascending (identical to pre-#337 behavior).

### 8: Boundary
- **Given** Issue A declares `Blocked by: #B`, `#B` is closed — **When** `/gf-workflow-batch` runs — **Then** A's edge to `#B` is dropped, A is dispatched normally.

### 9: Boundary
- **Given** Issue A declares `Blocked by: #B`, `#B` is still open, A is the only pending Issue — **When** `/gf-workflow-batch` runs — **Then** `ready` is empty, `candidates` is empty, the loop stops without error (not a deadlock — A remains pending for a future round).

### 10: Error
- **Given** Issue A declares `Blocked by: #999`, `#999` does not exist — **When** `/gf-workflow-batch` runs — **Then** the whole run aborts before dispatching anything, error names `#999` and `#A`.

### 11: Error
- **Given** Issue A declares `Blocked by: #B`, Issue B declares `Blocked by: #A`, both open — **When** `/gf-workflow-batch` runs — **Then** the whole run aborts before dispatching anything, error prints the cycle path (e.g. `#A → #B → #A`).

### 12: Boundary
- **Given** Issue A declares `Blocked by: #B`; round 1 has `#B` open (A stays pending, not dispatched); round 2 runs after `#B` is closed — **When** `/gf-workflow-batch` re-derives — **Then** A appears in `ready` and is dispatched, with no state persisted between the two rounds beyond Issue state itself.
```

- [ ] **Step 4: 校验 Markdown 表格/代码块未破坏**

```bash
grep -n "^### " skills/gf-workflow-batch/SKILL.md
```

Expected: 场景编号从 `### 1: Happy Path` 连续到 `### 12: Boundary`，无重复编号、无断号。

- [ ] **Step 5: Commit**

```bash
git add skills/gf-workflow-batch/SKILL.md
git commit -m "docs(gf-workflow-batch): describe dependency resolution + add 6 test scenarios (#337)"
```

---

### Task 5: 消除 `gf-issue-decompose` 侧的过期声明

**Files:**
- Modify: `skills/gf-issue-decompose/references/dependency-edges.md:30-34`

**Interfaces:**
- 消费：Task 3/4 已交付的 `gf-workflow-batch` 依赖解析（作为"消费者已存在"的引用目标）。
- 产出：无新增接口，仅更新一段文字，避免文档自相矛盾（该文件当前仍声称"没有消费者"）。

- [ ] **Step 1: 替换过期段落**

将 `dependency-edges.md` 第 30-34 行：

```
`Blocked by: #N` is the literal a topological sorter will parse. No such consumer exists
in this repository yet — `gf-workflow-batch` still takes the first pending Issue in list
order, and sorting it by dependency edges is tracked as Issue #337. Writing the edge as
prose ("depends on the schema work") produces data that sorter cannot read, so keep the
literal even though nothing consumes it today.
```

改为：

```
`Blocked by: #N` is the literal `gf-workflow-batch`'s dependency resolution stage parses
(`skills/gf-workflow-batch/references.md` § Dependency Resolution, Issue #337). Writing
the edge as prose ("depends on the schema work") produces data that parser cannot read,
so keep the literal — a prose edge is silently treated as no dependency at all.
```

- [ ] **Step 2: 确认没有其他文件仍引用"消费者不存在"的旧说法**

```bash
grep -rn "No such consumer exists\|tracked as Issue #337" skills/ docs/ 2>/dev/null
```

Expected: 无匹配（Step 1 已替换唯一出处）。

- [ ] **Step 3: Commit**

```bash
git add skills/gf-issue-decompose/references/dependency-edges.md
git commit -m "docs(gf-issue-decompose): update stale note now that #337 ships a consumer"
```

---

### Task 6: 全量文档一致性检查

**Files:**
- 不修改文件（除非 Step 1 报告需要修复的问题）。

**Interfaces:**
- 消费：Task 3/4/5 的全部改动。
- 产出：`make check-agent-sync` 通过的确认。

- [ ] **Step 1: 运行仓库既有的 skill/文档一致性检查**

```bash
make check-agent-sync
```

Expected: 命令成功退出（exit 0）。若报告问题，按提示修复涉及的文件后重跑，直到通过。

- [ ] **Step 2: 人工走查 Test Scenarios 7-12 与设计文档 §4 的六条场景一一对应**

```bash
diff <(grep -A1 "^### [789]\|^### 1[012]:" skills/gf-workflow-batch/SKILL.md | grep -c "Given") <(echo 6)
```

Expected: 输出 `6 6`（或等价确认 6 条场景全部就位）——不要求逐字匹配设计文档措辞，只确认覆盖同样的六个场景（无依赖、单依赖已满足、单依赖未满足、前置不存在、成环、跨轮解锁）。

- [ ] **Step 3: 无需 Commit（本任务不产生文件改动，除非 Step 1 触发修复）**

若 Step 1 触发了修复，修复内容单独提交：

```bash
git add -A
git commit -m "fix(docs): address check-agent-sync findings for #337 changes"
```
