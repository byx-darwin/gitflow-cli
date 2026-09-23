# gf-workflow-batch Dependency Resolution 六项强化设计（#374）

**Status:** Approved（Bounded 路径，无需架构级评审）
**Workflow:** wf-2026-09-20-001

## 背景

#337 交付后的 review 在 `skills/gf-workflow-batch/references.md` 的 Dependency
Resolution 段落（第 70-232 行，纯 markdown/伪代码，非编译代码）里发现 6 项
Minor/low-confidence 强化项，均不阻塞 #337 已完成的交付。本次逐条判定并处理。

## 判定与方案

### 1. `bodies` 复用 list 已返回的 body 字段（改）

`crates/github/src/issue.rs` 的 `ISSUE_FIELDS` 常量：

```rust
const ISSUE_FIELDS: &str =
    "number,title,body,state,labels,author,assignees,createdAt,updatedAt,url";
```

`list` 和 `view` 共享同一字段集，`gf_issue_list_open()` 已经带回 `body`。原伪代码
`bodies = {i.number: gf_issue_view(i.number).body for i in open_issues}` 对每个
open Issue 多打一次 `gf issue view`，纯属多余。改为直接复用 `open_issues` 里
已有的 `.body` 字段，消除每轮 N 次多余 CLI 调用。

### 2. `Blocked by` 正则容错（不改正则，补说明）

查证 `skills/gf-issue-decompose/references/dependency-edges.md`："`Blocked by: #N`
是 `gf-workflow-batch` 的依赖解析阶段解析的字面量……写成散文形式……那个解析器读不出来"
——`Blocked by: #12, #14` 是生产端（`gf-issue-decompose`）**明确规定、受控统一**要写成
的唯一格式，不是自由文本输入。放宽正则去兼容 `Blocked By:`/无冒号/空格分隔等变体，
只会增加解析歧义面，收益不成比例。**不改正则**，按 AC2 自带的退让条款，在解析小节
补一句显式说明：其他写法会被静默当作无依赖处理，写 Issue 正文时必须用规范格式。

### 3. Dependency Resolution 挪到 `pending` 判空之后（改）

Pending Derivation 算法：`pending` = 尚未被任何 contract 覆盖的 open Issue。
`pending` 为空意味着**所有** open Issue 都已被某个 contract 覆盖（进行中或已交付）。
此时对全量 open Issue 跑依赖解析（尤其是成环检测）纯属多余开销，还可能因为一堆
已经在别处处理、跟本轮候选毫无关系的 Issue 之间存在的环，把本该直接进入
Discussion Mode 的这一轮意外挡住（`WorkflowBatchError` 会中止整轮，不进
Discussion Mode）。**调整执行顺序**：`derive_pending()` 之后先判断
`pending is empty`，为空则直接走 Discussion Mode / 结束分支；非空时才执行
Dependency Resolution 计算 `ready`。

### 4. DFS 递归深度（改为显式栈迭代）

这段伪代码是 Claude 执行 `/gf-workflow-batch` 时的操作指南；成环检测这类需要
可靠性保证的图算法，实践中很可能通过 Bash 跑一段一次性 python3 脚本完成，而非
手工推理。递归版 DFS 理论上无深度保护，超长依赖链会触发未捕获的 `RecursionError`
而非预期的 `WorkflowBatchError`。**改写为显式栈迭代**，直接消除这个失败模式
（比加一个深度上限更彻底，且实现复杂度相当）。

### 5. 派发汇总区分终止状态（改）

Serial Dispatch Loop 在 `candidates` 耗尽时统一 `break`，无论是"全部正常完成"
还是"pending 里还有 Issue 卡在未解锁的前置"，`print_summary_table(summary)`
展示的都是同一张表，看不出区别。**在循环结束后、打印 summary table 之后**，
按 `pending` 是否非空追加一行状态说明：非空则列出仍被阻塞的 Issue 编号，提示
"阻塞前置关闭后重新运行即可继续"；为空则不追加（正常完成，无需额外说明）。

### 6. 逐项闭环

上述 5 项，4 项改伪代码，1 项（AC2）判定"不改"并按 AC6 要求补了取舍说明——
逐条都有对应处理，无遗漏项。

## 测试策略

纯 markdown/伪代码文件，不涉及编译代码，无单元测试。验证方式：
- 人工核对改动后的伪代码逻辑自洽（执行顺序、变量作用域、循环终止条件）
- 核对与 `#337` 已确认的核心设计决策（前置判定=closed、全 open Issue 扫描范围、
  成环即报错终止）保持不变，本次只调整执行时机与实现细节，不改变这些决策本身

## 验收标准回应（对齐 #374 原始 AC）

- [x] AC1 — 改，复用 `open_issues` 的 `body`
- [x] AC2 — 不改正则，补显式说明
- [x] AC3 — 改，调整执行顺序
- [x] AC4 — 改，DFS 改显式栈迭代
- [x] AC5 — 改，终止状态区分
- [x] AC6 — 逐项闭环，AC2 附带取舍说明
