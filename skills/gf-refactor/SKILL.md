---
name: gf-refactor
description: |
  Use when the user wants to refactor code using Fowler's catalog of behavior-preserving
  refactoring techniques — extract function, replace conditional with polymorphism, and
  similar structural changes — under a strict one-technique-at-a-time safety protocol.
  当用户希望按照 Fowler 重构手法目录进行行为不变的结构重构时使用——提取函数、
  以多态取代条件表达式等，在严格的单次一手法安全协议下执行。
allowed-tools: Read, Grep, Glob, Bash, Edit, Write
---

# gf-refactor — Fowler Catalog-Driven Behavior-Preserving Refactoring

Executes RED→GREEN→**REFACTOR**'s third stage: applies Fowler's refactoring
catalog to code that already has test coverage, one technique at a time,
verifying build and tests after every step. Downstream of `/gf-smell`
(finds problems) — this skill fixes them structurally, never behaviorally.

## Preconditions

- Target code has test coverage; if not, do not refactor it — see When NOT to Refactor
- Language layer's build/test tooling installed — see `references/<lang>.md`
- Clean working tree before starting (uncommitted changes make the Safety
  Protocol's revert step ambiguous)

## When to Use

| English | 中文 | Context |
|---|---|---|
| refactor this function | 重构这个函数 | apply a technique, not rewrite |
| extract function / method | 提炼函数 | reduce size/complexity |
| replace conditional with polymorphism | 以多态取代条件表达式 | eliminate type-code branching |
| clean up while keeping tests green | 边清理边保绿 | TDD REFACTOR stage |
| is this refactor safe | 这个重构安全吗 | semantic risk question |

## When NOT to Use

| Scenario | Why Not | Use Instead |
|---|---|---|
| Finding smells/candidates in the first place | This skill acts on candidates, it does not hunt for them | `/gf-smell` |
| Pre-delivery quality gate | This skill changes structure, not gate pass/fail | `/gf-quality` |
| Any change to observable behavior is wanted | Not a refactor by definition | Regular TDD RED→GREEN change, separate commit |

## When NOT to Refactor

| Scenario | Why Not |
|---|---|
| Code path has 无测试覆盖 | Nothing verifies behavior was preserved; write the test first, or refuse |
| The technique is tagged 条件等价/可能变更 and its precondition is unverified | Downgrade to suggestion per Semantic Safety Boundary — do not touch the code |
| A behavior change is also wanted alongside | Split into two changes; refactor first (green), then the behavior change as its own commit |
| Code is about to be deleted | No return on refactoring investment |

## Golden Rules

1. **一次只做一个手法** — one technique per step, never batch two techniques into one edit
2. **必须编译干净** — compile clean after every single technique
3. **全部测试通过** — all tests pass after every single technique; any failure ⇒ stop, do not proceed to the next technique
4. **立即 commit** — commit the moment tests are green, before starting the next technique
5. **行为变更与重构不能混在同一次改动里** — a change that alters observable behavior is never combined with a structural refactor in the same commit

## Decision Rubric

Before applying any technique, answer every row; a "no" stops the row's action:

| Question | If "no" |
|---|---|
| Does this code path have test coverage? | Stop — see When NOT to Refactor |
| Is the technique tagged 等价 in the catalog below? | Downgrade to suggestion — see Semantic Safety Boundary |
| Will the working tree be clean before starting? | Commit or stash first |
| Is exactly one technique being applied this step? | Split into separate steps |

## Semantic Safety Boundary

**语义变更一律降级为建议。** A technique whose application could plausibly
change observable behavior is never auto-applied — it is presented to the
user as a suggestion, with the specific risk named, and the user decides.

Three tiers, applied to every technique in the catalog below:

| Tier | Meaning | May auto-apply? |
|---|---|---|
| **等价 (Equivalent)** | Mechanically behavior-preserving under the Golden Rules | Yes |
| **条件等价 (Conditionally Equivalent)** | Behavior-preserving only if a stated precondition holds | No — verify the precondition first; if unverifiable, downgrade to suggestion |
| **可能变更 (Possible Behavior Change)** | Very likely to alter behavior in at least one dimension (timing, aliasing, dispatch, serialization) | No — suggestion only |

**标注为「条件等价」或「可能变更」的手法不得自动应用**——即便用户在同一句话里
要求"顺便重构一下"，也必须先展示风险说明，等用户确认并核实前置条件后，才可能
按 等价 路径下的机械步骤执行；本 skill 本身从不替用户做这个判断。

## Technique Catalog

Language-agnostic. Grouped by Fowler's chapters. `语义风险` is per
technique; `references/<lang>.md` supplies the idiom a 等价-tier technique
lands on and a concrete pitfall for this language's 条件等价/可能变更 rows.

### Composing Methods

| 手法 | 语义风险 | 说明 |
|---|---|---|
| Extract Function 提炼函数 | 等价 | 纯粹的代码搬移，不改变签名外部可见的输入输出 |
| Inline Function 内联函数 | 等价 | Extract Function 的逆操作 |
| Extract Variable 提炼变量 | 等价 | 命名中间结果，不改变求值 |
| Inline Variable 内联变量 | 等价 | Extract Variable 的逆操作 |
| Change Function Declaration 改变函数声明 | 条件等价 | 若为 public API，需同步全部调用点 |
| Encapsulate Variable 封装变量 | 条件等价 | 若字段原本是 pub，封装后所有外部直接访问点都需改用 getter，属签名变化；仅原本已是私有字段时才等价 |
| Rename Variable 变量改名 | 等价 | 纯标识符替换 |
| Introduce Parameter Object 引入参数对象 | 条件等价 | 新类型的构造/默认值语义需核对 |
| Combine Functions into Class 函数组合成类 | 条件等价 | 引入共享状态，生命周期语义变化 |
| Combine Functions into Transform 函数组合成转换 | 条件等价 | 中间数据结构的拷贝语义需核对 |
| Split Phase 拆分阶段 | 条件等价 | 中间结构的构造时机/惰性求值可能改变 |

### Moving Features

| 手法 | 语义风险 | 说明 |
|---|---|---|
| Move Function 搬移函数 | 条件等价 | 跨模块可见性与依赖方向变化 |
| Move Field 搬移字段 | 条件等价 | 跨对象生命周期/所有权可能变化 |
| Move Statements into Function 搬移语句进入函数 | 条件等价 | 若目标函数存在其它调用方，它们会被动执行新搬入的语句，需确认这对它们同样安全 |
| Move Statements to Callers 搬移语句到调用方 | 条件等价 | 多调用点时需逐一核对执行时机 |
| Replace Inline Code with Function Call 以函数调用取代内联代码 | 条件等价 | 需先确认目标函数与内联代码行为确实等价，而非仅签名相似 |
| Slide Statements 移动语句 | 条件等价 | 语句间隐含顺序依赖会被打破 |
| Split Loop 拆分循环 | 条件等价 | 提前 return/break 或跨迭代累积状态会改变结果 |
| Replace Loop with Pipeline 以管道取代循环 | 条件等价 | 惰性求值/提前返回/异常传播路径可能改变 |
| Remove Dead Code 移除死代码 | 可能变更 | 先证明真正不可达，否则是移除未覆盖分支 |

### Organizing Data

| 手法 | 语义风险 | 说明 |
|---|---|---|
| Split Variable 拆分变量 | 等价 | 消除一变量多职责，不改变各自的值 |
| Rename Field 字段改名 | 条件等价 | 序列化边界可能被外部消费者依赖 |
| Replace Derived Variable with Query 以查询取代派生变量 | 条件等价 | 缓存语义丢失，重复计算的性能/副作用特征改变 |
| Change Reference to Value 将引用对象改为值对象 | 可能变更 | 别名共享语义丢失 |
| Change Value to Reference 将值对象改为引用对象 | 可能变更 | 引入共享可变状态与并发可见性语义 |

### Simplifying Conditional Logic

| 手法 | 语义风险 | 说明 |
|---|---|---|
| Decompose Conditional 分解条件表达式 | 等价 | 命名分支，不改变判定结果 |
| Consolidate Conditional Expression 合并条件表达式 | 条件等价 | 短路求值顺序变化可能触发不同副作用 |
| Replace Nested Conditional with Guard Clauses 以卫语句取代嵌套条件 | 等价 | 判定结果集合不变 |
| Replace Conditional with Polymorphism 以多态取代条件表达式 | 条件等价 | 新类型须覆盖全部原分支，遗漏即行为变更 |
| Introduce Special Case 引入特例 | 条件等价 | Null Object 与原始判空的相等性/序列化语义可能不同 |
| Introduce Assertion 引入断言 | 条件等价 | 断言可能在生产构建被剥离，不能替代真实校验 |

### Refactoring APIs

| 手法 | 语义风险 | 说明 |
|---|---|---|
| Separate Query from Modifier 分离查询函数与修改函数 | 条件等价 | 若共享同一次昂贵计算，拆分后出现重复计算 |
| Parameterize Function 令函数携带参数 | 条件等价 | 合并后调用方签名变化，需同步全部调用点 |
| Remove Flag Argument 移除标记参数 | 条件等价 | 调用方签名变化，需同步全部调用点 |
| Preserve Whole Object 保持对象完整 | 条件等价 | 需确认被调函数不会读取额外字段产生副作用 |
| Replace Parameter with Query 以查询取代参数 | 条件等价 | 查询时机变化可能取到不同值 |
| Replace Query with Parameter 以参数取代查询 | 条件等价 | 需确认所有调用方能提供一致的值 |
| Remove Setting Method 移除设值函数 | 可能变更 | 依赖后续重新赋值的调用方行为改变 |
| Replace Constructor with Factory Function 以工厂函数取代构造函数 | 条件等价 | 需手动复现构造函数的隐式行为 |
| Replace Function with Command 以命令取代函数 | 条件等价 | 引入对象生命周期，需确认无状态泄漏 |
| Replace Command with Function 以函数取代命令 | 条件等价 | 丢失命令对象的可撤销/可排队特性 |

### Dealing with Inheritance

| 手法 | 语义风险 | 说明 |
|---|---|---|
| Pull Up Method 函数上移 | 条件等价 | 子类若有覆盖差异，上移改变多态派发结果 |
| Pull Up Field 字段上移 | 等价 | 状态搬移，访问路径不变 |
| Pull Up Constructor Body 构造函数本体上移 | 条件等价 | 初始化顺序变化影响依赖构造时机的副作用 |
| Push Down Method 函数下移 | 条件等价 | 其它子类若仍调用该方法即被破坏 |
| Push Down Field 字段下移 | 条件等价 | 其它子类若仍依赖该字段即被破坏 |
| Replace Type Code with Subclasses 以子类取代类型码 | 可能变更 | 类型码运行时可变的场景无法直接迁移 |
| Remove Subclass 移除子类 | 条件等价 | 若被用作 instanceof/is 判断依据需同步改写 |
| Extract Superclass 提炼超类 | 条件等价 | 实质是 Pull Up Method/Field 的复合，且新增超类改变多态派发面，子类若有覆盖差异需逐一核对 |
| Collapse Hierarchy 折叠继承体系 | 等价 | 合并等价的父子类 |
| Replace Subclass with Delegate 以委托取代子类 | 可能变更 | 多态派发改为显式委托，动态绑定行为改变 |
| Replace Superclass with Delegate 以委托取代超类 | 可能变更 | 里氏替换关系丢失，依赖它的调用点会失败 |

## Language Layer Contract

Every `references/<lang>.md` MUST provide exactly these three sections:

| Section | Content |
|---|---|
| `## 校验命令` | This language's build and test commands; run after every technique |
| `## 惯用法映射` | Which native construct a 等价-tier technique lands on |
| `## 语义陷阱判例` | Concrete case studies for this language's 条件等价/可能变更 rows — at least two `### 陷阱` subsections with real code shapes, not just technique names |

Adding a language means adding one file with these three sections. Nothing
in this document changes. Language detection reuses
`gf-quality/references/detector.md` — **do not implement a second detection
mechanism.**

## Safety Protocol Execution

For each technique applied (must be 等价-tier, or 条件等价 with a verified
precondition):

1. Apply exactly one technique
2. Run this language's build command (per `references/<lang>.md`) — 编译不干净则撤销本次编辑，不继续
3. Run this language's test command — 有测试失败则撤销本次编辑，不进入下一个手法
4. Commit immediately with a message naming the technique
5. Move to the next technique only after step 4 completes

If any of steps 2-3 fails, revert the edit (`git checkout -- <file>` or
equivalent) rather than debugging forward on a broken state — debugging a
half-applied refactor conflates "is my refactor tool broken" with "is my
refactoring correct."

## Report

For 条件等价/可能变更 candidates that are surfaced as suggestions (never
auto-applied), write to `docs/refactor-report-<scope>-<YYYY-MM-DD>.md`,
reusing `gf-smell`'s severity/evidence vocabulary so both skills read as one
system:

```markdown
# Refactor Report — <scope>
生成时间 · 目标范围 · 应用手法数 · 建议手法数

## 已应用（等价，自动应用）
### RF-001 · <手法> · commit <sha>
- 位置：path:line
- 校验：编译 ✓ 测试 ✓

## 建议（条件等价 / 可能变更，未自动应用）
### RF-101 · <手法> · <语义风险档位>
- 位置：path:line
- 证据强度：Measured | Observed | Inferred
- 置信度：High | Medium | Low
- 严重度：High | Medium | Low
- 风险说明：（本手法为何是条件等价/可能变更，具体到这处代码）
- 前置条件：（若为条件等价，说明需验证什么；已验证/未验证）
- 建议：（仅建议，不执行）
```

## Responsibility

### ✅ In Scope

- Apply 等价-tier techniques one at a time under the Safety Protocol
- Verify a 条件等价 technique's precondition before applying it; downgrade if unverifiable
- Surface 条件等价/可能变更 candidates as a suggestion report

### ❌ Out of Scope

- Finding smells/candidates in the first place — `/gf-smell`
- Any change to observable behavior — refuse; that is not a refactor
- Pre-delivery quality gating — `/gf-quality`

### 🚫 Do Not

- ❌ Apply two techniques in one edit
- ❌ Continue after a failed build or test
- ❌ Auto-apply a 条件等价 or 可能变更 technique
- ❌ Refactor a code path with no test coverage
- ❌ Mix a behavior change into a refactor commit

## Rationalization Excuses

| Excuse | Reality |
|---|---|
| "While I'm in here I'll also fix this bug" | That's a behavior change — separate commit, separate review |
| "These two techniques are basically one step" | Apply and commit them one at a time; the safety net is per-step, not per-intent |
| "It's obviously fine, no need to verify the precondition" | 条件等价 means unverified ⇒ suggestion, not application |
| "No tests, but the change is trivial" | Trivial is a claim about the diff, not about what could observably change — write the test first, or refuse |
| "The build/test check already ran once this session" | Every technique gets its own build+test cycle; a stale pass proves nothing about this edit |

## Red Flags

- 🚩 "Just apply all these techniques at once" — Refuse. One at a time, per Golden Rule 1.
- 🚩 "Skip the test run, I can see it's fine" — Refuse. Safety Protocol step 3 is mandatory.
- 🚩 "Auto-apply this 可能变更 technique, I trust it" — Refuse. Suggestion only.
- 🚩 "Also change the return type while refactoring" — Refuse. That is a behavior/API change.

## Common Mistakes

- ❌ **Batching multiple techniques in one commit** — breaks the per-step safety net and the audit trail
- ❌ **Debugging forward after a failed test instead of reverting** — conflates tool correctness with refactor correctness
- ❌ **Treating "no compiler error" as sufficient** — tests still gate every step

## Test Scenarios

### 1: Happy Path
- **Given** a well-tested function with an Extract Function candidate — **When** "refactor this"
- **Then** one technique applied → build+test green → committed → report of what changed

### 2: Negative
- **Given** "refactor this and also change what it returns" — **Then** refused; behavior change is out of scope

### 3: Boundary
- **Given** a Replace Conditional with Polymorphism candidate with an unverified branch-coverage precondition
- **Then** downgraded to suggestion, not applied, risk stated in the report

### 4: Error
- **Given** the target code path has no test coverage — **Then** stop, per When NOT to Refactor; do not write tests unprompted, do not refactor

## Success Criteria

- [ ] Every applied technique is 等价-tier or a precondition-verified 条件等价
- [ ] Build and tests run and pass after every single technique
- [ ] One commit per technique
- [ ] No behavior change mixed into any refactor commit
- [ ] 条件等价/可能变更 candidates appear only in the suggestion report, never auto-applied

## Trigger Keywords

| English | 中文 |
|---|---|
| refactor | 重构 |
| extract function | 提炼函数 |
| replace conditional with polymorphism | 以多态取代条件表达式 |
| behavior-preserving | 行为不变 |
| safety protocol | 安全协议 |

## See Also

- `/gf-smell` — finds the candidates this skill acts on
- `/gf-quality` — pre-delivery quality gate
- `/gf-workflow` — full four-phase delivery pipeline this skill's output feeds into
