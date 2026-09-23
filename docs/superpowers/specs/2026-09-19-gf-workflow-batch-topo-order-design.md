# gf-workflow-batch 按依赖边拓扑排序取票

- **Issue**：[#337](https://github.com/byx-darwin/gitflow-cli/issues/337)
- **Workflow**：`wf-2026-09-19-005`（full 模式）
- **日期**：2026-09-19
- **状态**：设计已确认，待实现

## 1. 问题

`skills/gf-workflow-batch/references.md` 的 Pending Derivation 算法从 open 列表
无序取 `pending[0]`，不解析 Issue 之间的依赖关系。若某个 pending Issue 存在未完成的
前置（`Blocked by: #N`），批量驱动器仍可能先派发它，走完 gf-workflow 四阶段才发现
前置未完成，白费一整轮。

`skills/gf-issue-decompose/references/dependency-edges.md:30-34` 已经把
`Blocked by: #N` 定为"拓扑排序器会解析的字面量"，并明确指出"该消费者目前不存在
（Issue #337 跟踪）"。本设计就是补上这个消费者。

本改动是纯 prompt/伪代码文档修改（`skills/gf-workflow-batch/{SKILL.md,references.md}`），
不涉及 Rust 代码；`gf-workflow-batch` 与 `gf-workflow` 本身都是由 Claude 读取 skill
指令后驱动 `gf` CLI 调用完成的编排，没有独立可执行的调度器程序。

## 2. 关键设计决策（用户已确认）

| 决策点 | 选择 | 理由 |
|---|---|---|
| 前置"已完成"的判定依据 | `gf issue view <N>` 的 `state == closed` | 与手动关闭、非 workflow 交付的 Issue 兼容；不依赖 `.cache/workflows/` 归档合同这个次级数据源，判定路径更短 |
| 前置引用的 Issue 不存在时如何处理 | 报错，交还用户，不静默忽略 | 防止损坏的依赖声明（拼写错误/误删）被静默当作"已满足"而绕过阻塞语义 |
| 环检测发现成环时如何处理 | 报错并打印环路径，不派发任何 Issue，交还用户重新切片 | 遵循 `dependency-edges.md:23-24`："一个方向，成环即切片错误"，不自动破环 |

## 3. 设计

### 3.1 算法插入点

在 `references.md` 现有 Pending Derivation Algorithm（`derive_pending()`，产出
`pending` 列表，按 `issue.number` 升序）与 Serial Dispatch Loop 的
`candidates = [i for i in pending if i.number not in attempted]` 之间，插入一个新阶段
"Dependency Resolution"，产出 `ready`（`ready ⊆ pending`）。Serial Dispatch Loop 的
`candidates` 改为从 `ready` 取，其余逻辑（`attempted` 集合、`--limit` 计数、
Discussion Mode 触发条件）不变。

### 3.2 依赖图构建范围

依赖边解析对象是**全部 open Issue**（而不仅是 `pending` 子集），原因：一个尚在执行中
（已被某个 active 合同覆盖、因此不在 `pending` 里）的 Issue 仍可能是某个 pending
Issue 的前置，也仍可能是某个环的成员——只扫描 `pending` 会漏掉这两种情况。

```
open = gf issue list --state open --output json      # 已有，unchanged
bodies = { i.number: gf issue view i.number for i in open }   # 本轮内存缓存，不落盘

edges = {}   # blocked_number -> set(blocker_number)
for i in open:
    m = regex.findall(r'Blocked by:\s*((?:#\d+(?:,\s*)?)+)', bodies[i.number])
    for group in m:
        for blocker in re.findall(r'#(\d+)', group):
            edges.setdefault(i.number, set()).add(int(blocker))
```

正则与 `dependency-edges.md:11` 的字面量格式对齐：`Blocked by: #12, #14`，允许多个
用逗号分隔的引用、允许同一 Issue body 里出现多行 `Blocked by:` 声明（取并集）。

### 3.3 前置解析与已满足边剔除

```
for blocked, blockers in edges.items():
    for b in list(blockers):
        if b not in bodies and b not in {i.number for i in open}:
            # 不在 open 列表里 —— 可能已 closed，也可能号不存在，需要单独确认
            view = try gf issue view b
            if view is None (404 / not found):
                ERROR: f"Issue #{blocked} 的 Blocked by 引用了不存在的 #{b}"
                ABORT before dispatching anything
            elif view.state == "closed":
                blockers.discard(b)   # 已完成，边剔除
        elif b in {i.number for i in open}:
            pass   # 仍是 open，边保留，进入环检测 / 就绪判定
```

`open` 列表本身只含 open Issue，因此"引用号不在 open 列表里"这一步天然覆盖了
"已 closed"和"号不存在"两种情况，靠一次额外的 `gf issue view b` 区分。

### 3.4 环检测

对剔除已满足边后的剩余图（节点：仍有未满足 `Blocked by` 边参与的 open Issue 编号；
边：`blocked -> blocker`）做标准三色 DFS 环检测：

```
WHITE, GRAY, BLACK = 0, 1, 2
color = defaultdict(lambda: WHITE)
path = []

def dfs(n):
    color[n] = GRAY
    path.append(n)
    for b in edges.get(n, []):
        if color[b] == GRAY:
            cycle = path[path.index(b):] + [b]
            ERROR: f"依赖成环: {' → '.join('#' + str(x) for x in cycle)}"
            ABORT before dispatching anything
        if color[b] == WHITE:
            dfs(b)
    path.pop()
    color[n] = BLACK

for n in edges:
    if color[n] == WHITE:
        dfs(n)
```

发现环 → 立即报错并终止本轮（不进入 Discussion Mode，不派发任何 Issue），把环路径
原样交还用户；用户按 `dependency-edges.md:23-24` 的指导重新切片后再次运行
`/gf-workflow-batch`。

### 3.5 就绪集合与顺序

```
ready = [i for i in pending if not edges.get(i.number)]   # 边已在 3.3 全部剔除或从未声明
ready.sort(key=lambda i: i.number)   # 与现有 pending 排序一致，无依赖声明的 Issue 顺序不变
```

`pending` 中不在 `ready` 里的 Issue（仍有未满足前置）留在 `pending`，但本轮不进入
`candidates`。下一轮 `/gf-workflow-batch`（或同一进程的下一次循环迭代）会重新
`derive_pending()` + 重新解析依赖图——一旦前置 Issue 被关闭（无论是被
`gf-workflow` 交付关闭还是手动关闭），它在下一轮的 `bodies`/`open` 快照里就不再是
"未满足"边，自然解锁，无需任何额外的跨轮状态持久化。

### 3.6 Serial Dispatch Loop 的改动

```diff
 loop:
     if limit is set and dispatched >= limit: break
     pending = derive_pending()
+    ready = resolve_dependencies(pending, open)   # §3.2–§3.5；ERROR 路径在此直接终止整个命令
     if pending is empty:
         if not discussion_attempted:
             run_discussion_mode()
             discussion_attempted = true
             continue
         else:
             break
-    candidates = [i for i in pending if i.number not in attempted]
+    candidates = [i for i in ready if i.number not in attempted]
     if candidates is empty:
-        break           # everything remaining was already attempted this run — stuck
+        break           # 就绪集合已耗尽，或全部候选本轮已尝试过 — 停止
     issue = candidates[0]
     ...
```

`pending is empty` 的判定仍然用 `pending`（而不是 `ready`）——`pending` 为空才是
"没有未覆盖的 open Issue了"，触发 Discussion Mode 的语义不变；`ready` 为空但
`pending` 非空，是"全部候选都在等前置"，属于 `candidates is empty` 分支，正常停止
（不是错误，是正常的"本轮无可派发项"）。

## 4. 测试策略

本改动无 Rust 代码，`make test` / `cargo test` 不适用。验证方式是在
`skills/gf-workflow-batch/SKILL.md`「Test Scenarios」小节追加场景，供后续人工或
agent 走查该 skill 的 prompt 逻辑是否覆盖：

1. **无依赖声明**：3 个 pending Issue 均无 `Blocked by` → 顺序不变，与现状一致。
2. **单依赖已满足**：Issue A 声明 `Blocked by: #B`，`#B` 已 closed → A 进入
   `ready`，正常派发。
3. **单依赖未满足**：Issue A 声明 `Blocked by: #B`，`#B` 仍 open → A 不进入
   `ready`；若 A 是唯一 pending Issue，`candidates` 为空，循环停止（不报错，不是
   死锁，是"本轮无可派发项"）。
4. **前置不存在**：Issue A 声明 `Blocked by: #999`，`#999` 查不到 → 报错终止整个
   命令，不派发任何 Issue。
5. **成环**：Issue A `Blocked by: #B`，Issue B `Blocked by: #A`，两者均 open →
   报错并打印环路径 `#A → #B → #A`，不派发任何 Issue。
6. **跨轮解锁**：Issue A 声明 `Blocked by: #B`；第一轮 `#B` 仍 open，A 留在
   `pending` 不派发；`#B` 被关闭后的下一轮，A 重新计算依赖图后进入 `ready`，被
   正常派发——不需要额外的跨轮状态。

## 5. 验收对照（Issue #337）

| AC | 本设计的处理 |
|---|---|
| 解析 `Blocked by` 依赖边，忽略依赖则为假 | §3.2 |
| 存在未完成前置的 Issue 不被选中执行 | §3.3、§3.5、§3.6 |
| 依赖成环时报错并交还用户，静默死锁或无限等待则为假 | §3.4 |
| 无依赖声明的 Issue 保持原有顺序，不引入额外重排 | §3.5 |
| 串行执行前提不变，本改动不引入并发 | §3.6（Serial Dispatch Loop 骨架不变，只替换 `candidates` 的取数源） |
| 每轮取票的依据从 `.cache/workflows/` 磁盘状态推导，不依赖对话记忆 | §3.2 每轮重新 `gf issue list`/`gf issue view`，不缓存到磁盘或对话记忆；`derive_pending()` 本身已满足此点，未改动 |

## 6. 已知遗留

1. 依赖解析需要对每个 open Issue 额外调用一次 `gf issue view` 取 body（`gf issue
   list` 不返回 body），批量场景下调用次数与 open Issue 数量线性相关。当前仓库
   open Issue 规模较小，未做进一步优化（如缓存/并发拉取）；若未来规模显著增长，
   可作为后续优化项。
2. 环检测范围是全部 open Issue 的依赖图（§3.2），而非仅 `pending` 子集——这在
   "环的一端已被其它 workflow 覆盖、不在 pending 里"这种边界场景下仍能正确报错，
   但也意味着环检测的调用开销不会因为大部分 Issue 已被覆盖而减少。
