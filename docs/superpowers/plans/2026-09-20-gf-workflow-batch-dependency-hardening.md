# gf-workflow-batch Dependency Resolution Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 处理 #374 的 6 项 Minor 强化：4 项改伪代码，1 项（正则容错）判定不改但补取舍说明，1 项（AC6）是逐条闭环的自检要求。

**Architecture:** 单文件 `skills/gf-workflow-batch/references.md`，5 处精确编辑，均在既有的 Dependency Resolution / Serial Dispatch Loop 两个小节内。纯 markdown + 伪代码，非编译代码。

**Tech Stack:** Markdown

**Spec:** `docs/superpowers/specs/2026-09-20-gf-workflow-batch-dependency-hardening-design.md`

## Global Constraints

- 不改变 #337 已确认的核心设计决策：前置判定=closed、全 open Issue 扫描范围、成环即报错终止
- AC2 判定不改正则；其余 4 项改伪代码
- 不引入 Python 之外的语言（这段伪代码一贯用 Python 风格，保持一致）

---

### Task 1: 编辑 Dependency Resolution 与 Serial Dispatch Loop 五处

**Files:**
- Modify: `skills/gf-workflow-batch/references.md:70-232`

**Interfaces:**
- 无跨任务接口——单文件单任务的自包含改动

- [ ] **Step 1（AC1）：`bodies` 复用 `open_issues` 已有的 `body` 字段**

把第 83-84 行：

```python
open_issues = gf_issue_list_open()                     # existing, unchanged
bodies = {i.number: gf_issue_view(i.number).body for i in open_issues}
```

替换为：

```python
open_issues = gf_issue_list_open()                     # existing, unchanged;
                                                         # already includes `body`
                                                         # (gh/glab/gc issue list shares
                                                         # the same field set as `view`,
                                                         # see crates/github/src/issue.rs
                                                         # ISSUE_FIELDS)
bodies = {i.number: i.body for i in open_issues}        # reuse — no extra `gf issue view` calls
```

- [ ] **Step 2（AC2）：补 `Blocked by` 正则容错的取舍说明**

在第 97-100 行（"Multiple `Blocked by:` lines in one body union their references..."
那一段）之后，新增一段说明。改动前该段落结尾是：

```markdown
Multiple `Blocked by:` lines in one body union their references (see
`skills/gf-issue-decompose/references/dependency-edges.md` for the
declaration format this parses — one edge per referenced number, declared
on the blocked ticket only).
```

在这段之后紧接着插入新段落：

```markdown
The regex intentionally matches only the canonical `Blocked by: #N[, #M...]`
form documented there — case-sensitive `Blocked by:`, colon required,
comma-separated. Variants (`Blocked By:`, `Blocked by #12` without a colon,
space-separated `#12 #14`) are **not** recognized and are silently treated
as no dependency at all, same as a body with no `Blocked by` line at all.
This is intentional, not an oversight: the producing skill
(`gf-issue-decompose`) always emits the canonical form, so loosening the
regex would only widen the ambiguity surface for hand-edited bodies without
a corresponding real need — YAGNI.
```

- [ ] **Step 3（AC3）：把 Dependency Resolution 挪到 `pending` 判空之后执行**

把 Serial Dispatch Loop 里第 198-211 行：

```
loop:
    if limit is set and dispatched >= limit: break
    pending = derive_pending()   # recomputed every iteration, see above
    ready = resolve_dependencies(pending)   # re-lists open Issues itself, every round;
                                             # see Dependency Resolution above; raises
                                             # WorkflowBatchError → abort the whole run,
                                             # no dispatch this run
    if pending is empty:
        if not discussion_attempted:
            run_discussion_mode()
            discussion_attempted = true
            continue   # recompute pending, which now includes new Issues
        else:
            break       # nothing left even after discussion mode
```

替换为：

```
loop:
    if limit is set and dispatched >= limit: break
    pending = derive_pending()   # recomputed every iteration, see above
    if pending is empty:
        if not discussion_attempted:
            run_discussion_mode()
            discussion_attempted = true
            continue   # recompute pending, which now includes new Issues
        else:
            break       # nothing left even after discussion mode
    ready = resolve_dependencies(pending)   # only runs when there's something to
                                             # dispatch this round — an empty `pending`
                                             # means every open Issue is already
                                             # covered by some contract, so a cycle or
                                             # bad reference among them is irrelevant
                                             # to this round and must not block
                                             # Discussion Mode above. Re-lists open
                                             # Issues itself, every round; see
                                             # Dependency Resolution above; raises
                                             # WorkflowBatchError → abort the whole run,
                                             # no dispatch this run
```

（`candidates = [i for i in ready if ...]` 及之后的代码不变，只是现在 `ready`
的计算点挪到了这两个分支之后。）

- [ ] **Step 4（AC4）：把递归 DFS 改写为显式栈迭代**

把第 133-166 行整个 `find_cycle` 函数（含 `WHITE, GRAY, BLACK` 声明与调用行）：

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
```

替换为：

```python
WHITE, GRAY, BLACK = 0, 1, 2

def find_cycle(edges: dict[int, set[int]]) -> list[int] | None:
    # Explicit-stack DFS, not recursive: an unbounded dependency chain must
    # raise WorkflowBatchError (a real cycle) or return None, never an
    # uncaught RecursionError from a deep-but-acyclic chain.
    color = {}

    for start in list(edges):
        if color.get(start, WHITE) != WHITE:
            continue
        stack = [(start, iter(edges.get(start, ())))]
        path = [start]
        color[start] = GRAY
        while stack:
            node, neighbors = stack[-1]
            b = next(neighbors, None)
            if b is None:
                color[node] = BLACK
                path.pop()
                stack.pop()
                continue
            if color.get(b, WHITE) == GRAY:
                return path[path.index(b):] + [b]
            if color.get(b, WHITE) == WHITE:
                color[b] = GRAY
                path.append(b)
                stack.append((b, iter(edges.get(b, ()))))
    return None

cycle = find_cycle(edges)
```

- [ ] **Step 5（AC5）：派发汇总区分"正常完成"与"仍有 Issue 卡住"**

把第 224 行（`print_summary_table(summary)`，Serial Dispatch Loop 伪代码的最后一行）：

```
    summary.append({issue: issue.number, contract: result.contract_path,
                     delivery: result.pr_url or result.merge_commit,
                     outcome: result.outcome})   # success | failed | rejected
print_summary_table(summary)
```

替换为：

```
    summary.append({issue: issue.number, contract: result.contract_path,
                     delivery: result.pr_url or result.merge_commit,
                     outcome: result.outcome})   # success | failed | rejected
print_summary_table(summary)
if pending:   # loop exited via `candidates is empty` with unmet-dependency
              # Issues still sitting in `pending` — distinguish this from a
              # clean run where nothing was left to do
    print(f"⏸ {len(pending)} Issue(s) still blocked on unmet dependencies: "
          + ", ".join(f"#{i.number}" for i in pending))
    print("Re-run /gf-workflow-batch after their blockers close.")
```

- [ ] **Step 6：核对伪代码逻辑自洽**

Run: `sed -n '70,235p' skills/gf-workflow-batch/references.md`

人工核对：
1. Step 3 改动后，`ready`/`candidates` 在 `pending` 非空分支之外不再被引用（没有遗留的悬空变量读取）
2. Step 4 改动后的 `find_cycle` 对每个还是 `WHITE` 的起点都跑一遍外层 `for start in list(edges)` 循环，语义与原递归版一致（多个不连通的环分量都能被发现）
3. Step 5 的 `pending` 引用的是循环体内最后一次 `derive_pending()` 算出的值（Python 变量作用域里循环内赋值在循环外可见），不是一个新变量

- [ ] **Step 7：YAML/Markdown 语法检查**

Run: `pre-commit run --all-files`
Expected: 全部 `Passed` 或因未改 Rust/YAML 文件而 `Skipped`（`trim trailing whitespace`、`fix end of files` 等通用 hook 应对本次改动的 markdown 文件生效并 `Passed`）

- [ ] **Step 8: 提交**

```bash
git add skills/gf-workflow-batch/references.md
git commit -m "hardening(gf-workflow-batch): address 6 minor dependency resolution findings (#374)"
```

---

## Final Verification

- [ ] **Step 1: 全量 pre-commit**

Run: `pre-commit run --all-files`
Expected: 全部 `Passed`（本次未改 Rust/Cargo 相关文件，`cargo fmt`/`cargo check` 应为 `Skipped`）
