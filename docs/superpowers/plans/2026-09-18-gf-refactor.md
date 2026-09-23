# gf-refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 新增 `skills/gf-refactor/`，支撑 CLAUDE.md TDD 循环的 REFACTOR 阶段：语言无关的 Fowler 重构手法目录（含每个手法的语义风险标注）、行为不变安全协议、语义安全边界原则（承接自 #328），语言专属的校验命令/惯用法映射/语义陷阱判例下沉到 `references/<lang>.md`。

**Architecture:** SKILL.md 承载 Golden Rules、Decision Rubric、Semantic Safety Boundary、按 Fowler 六大类分组的技术目录（52 条手法，每条标注 等价/条件等价/可能变更）、Language Layer Contract、Safety Protocol 执行步骤与建议报告骨架；`references/rust.md` 承载本仓库唯一可实跑验证的校验命令、惯用法映射、语义陷阱判例。语言探测复用 `gf-quality/references/detector.md`。严重度与证据强度词汇与 `skills/gf-smell/SKILL.md`（#327）共享。验收标准中可机械验证的部分写成 Makefile target `check-refactor-skill`，先红后绿。

**Tech Stack:** Markdown（skill 定义）· GNU Make（验收断言）· 本仓库 Rust 工作区（Task 4 dogfooding 实跑）

**Spec:** Issue #332（`https://github.com/byx-darwin/gitflow-cli/issues/332`）—— 本次未跑 brainstorming（需求已充分，经用户确认跳过），Issue 正文即设计输入；无独立 `specs/*.md` 文件

## Global Constraints

- SKILL.md 正文**不得**出现任何单一语言的构建命令或惯用法（`cargo` / `rustc` / `go build` / `go test` / `golang` / `npm run` / `npm test` / `node_modules` / `pytest` / `python3` / `mvn ` / `gradle` / `Cargo.toml` / `go.mod` / `package.json` / `pyproject.toml` / `pom.xml` / `make check` / `make test` / `make build` / `make fmt` / `make clippy`）。
- 语言探测复用 `gf-quality/references/detector.md`，**不得**自行实现。
- 每份 `references/<lang>.md` 必须含三个二级标题：`## 校验命令`、`## 惯用法映射`、`## 语义陷阱判例`；`## 语义陷阱判例` 下至少 2 个 `### 陷阱` 具名案例，不得只列手法名。
- 技术目录中每一条手法必须在同一行标注 `等价` / `条件等价` / `可能变更` 三档之一；总条数 ≥40。
- 与 `skills/gf-smell/SKILL.md` 共享 `Measured` / `Observed` / `Inferred`（证据强度）与 `证据强度` / `严重度` 术语，不得另造一套。
- `语义变更一律降级为建议` 原则须逐字出现在 SKILL.md 正文。
- 标注为「条件等价」或「可能变更」的手法不得自动应用——只能作为建议呈现。
- 禁止修改 `clippy.toml`、`deny.toml`、`.pre-commit-config.yaml`、`rust-toolchain.toml`。
- 禁止运行 `cargo clean`。
- Skill 源码只改 `skills/gf-refactor/`，**不改** `.claude/skills/`（那是副本）。
- Task 4 dogfooding 对生产代码的改动必须逐字遵守 SKILL.md 自身的 Safety Protocol：一次一个手法、每步跑 `make check` 与 `make test`、失败即撤销、通过即刻单独 commit，不与本 skill 自身的文档改动混在一个 commit。

---

### Task 1: 验收断言（RED）

**Files:**
- Modify: `Makefile`（在 `check-smell-skill` target 之后新增 `check-refactor-skill`）

**Interfaces:**
- Consumes: 无
- Produces: `make check-refactor-skill` —— 后续每个 Task 都用它验证；退出码 0 表示全部断言通过

- [ ] **Step 1: 写入 Makefile target（此时必然失败，因为 skill 尚不存在）**

在 `Makefile` 中 `check-smell-skill` target 的紧后方插入：

```make
check-refactor-skill: ## Verify gf-refactor skill meets Issue #332 acceptance criteria
	@S=skills/gf-refactor/SKILL.md; R=skills/gf-refactor/references; FAIL=0; \
	if [ ! -f "$$S" ]; then echo "✗ missing $$S"; exit 1; fi; \
	if grep -nE 'cargo|rustc|go build|go test|golang|npm run|npm test|node_modules|pytest|python3|mvn |gradle|Cargo\.toml|go\.mod|package\.json|pyproject\.toml|pom\.xml|make check|make test|make build|make fmt|make clippy' "$$S"; then \
		echo "✗ AC#1 SKILL.md 正文含单一语言的构建命令或惯用法"; FAIL=1; \
	else echo "✓ AC#1 正文无语言专属标识"; fi; \
	if [ -f "$$R/rust.md" ]; then echo "✓ AC#2 references/rust.md 存在"; else echo "✗ AC#2 缺少 $$R/rust.md"; FAIL=1; fi; \
	grep -qF 'gf-quality/references/detector.md' "$$S" \
		&& echo "✓ AC#3 复用既有语言探测" \
		|| { echo "✗ AC#3 未引用 detector.md"; FAIL=1; }; \
	grep -qF '一次只做一个手法' "$$S" \
		&& echo "✓ AC#4 声明单次一手法规则" \
		|| { echo "✗ AC#4 缺少单次一手法规则"; FAIL=1; }; \
	grep -qF '必须编译干净' "$$S" && grep -qF '全部测试通过' "$$S" \
		&& echo "✓ AC#5 声明每手法后校验且失败即停止" \
		|| { echo "✗ AC#5 缺少编译/测试校验规则"; FAIL=1; }; \
	grep -qF '行为变更与重构不能混在同一次改动里' "$$S" \
		&& echo "✓ AC#6 声明行为变更与重构分离" \
		|| { echo "✗ AC#6 缺少行为变更与重构分离规则"; FAIL=1; }; \
	grep -qF '## When NOT to Refactor' "$$S" && grep -qF '无测试覆盖' "$$S" \
		&& echo "✓ AC#7 含 When NOT to Refactor 且排除无测试覆盖" \
		|| { echo "✗ AC#7 缺少 When NOT to Refactor 或未排除无测试覆盖"; FAIL=1; }; \
	VOCAB=0; \
	for K in Measured Observed Inferred 证据强度 严重度; do \
		grep -qF "$$K" "$$S" || { echo "✗ AC#8 缺少词汇 $$K"; VOCAB=1; FAIL=1; }; \
		grep -qF "$$K" skills/gf-smell/SKILL.md || { echo "✗ AC#8 gf-smell 中未找到对照词汇 $$K"; VOCAB=1; FAIL=1; }; \
	done; \
	[ $$VOCAB -eq 0 ] && echo "✓ AC#8 与 gf-smell 共享严重度与证据强度词汇"; \
	grep -qF '语义变更一律降级为建议' "$$S" \
		&& echo "✓ AC#9 含语义变更降级原则" \
		|| { echo "✗ AC#9 缺少「语义变更一律降级为建议」原则"; FAIL=1; }; \
	TOTAL=$$(grep -cE '^\| [A-Z][A-Za-z ]+ [^ -~]+[^|]*\|[^|]*\|[^|]*\|$$' "$$S"); \
	TAGGED=$$(grep -cE '^\| [A-Z][A-Za-z ]+ [^ -~]+[^|]*\| (等价|条件等价|可能变更) \|[^|]*\|$$' "$$S"); \
	if [ "$${TOTAL:-0}" -ge 40 ] 2>/dev/null && [ "$$TOTAL" = "$$TAGGED" ]; then \
		echo "✓ AC#10 技术目录 $$TOTAL 条手法均已标注语义风险"; \
	else echo "✗ AC#10 手法总数($$TOTAL)与已标注数($$TAGGED)不一致，或总数<40"; FAIL=1; fi; \
	grep -qF '条件等价' "$$S" && grep -qF '可能变更' "$$S" && grep -qF '不得自动应用' "$$S" \
		&& echo "✓ AC#11 声明条件等价/可能变更不得自动应用" \
		|| { echo "✗ AC#11 缺少不得自动应用声明"; FAIL=1; }; \
	if [ -f "$$R/rust.md" ]; then \
		for H in '## 校验命令' '## 惯用法映射' '## 语义陷阱判例'; do \
			grep -qF "$$H" "$$R/rust.md" || { echo "✗ 契约 rust.md 缺少 $$H"; FAIL=1; }; \
		done; \
		PIT=$$(grep -cE '^### 陷阱' "$$R/rust.md"); \
		if [ "$${PIT:-0}" -ge 2 ] 2>/dev/null; then echo "✓ AC#12 rust.md 含 $$PIT 条语义陷阱判例"; \
		else echo "✗ AC#12 rust.md 语义陷阱判例不足 2 条（当前 $$PIT）"; FAIL=1; fi; \
	fi; \
	if [ $$FAIL -ne 0 ]; then echo "FAILED"; exit 1; fi; \
	echo "ALL CHECKS PASSED"
```

- [ ] **Step 2: 运行断言，确认它失败（RED）**

Run: `make check-refactor-skill`
Expected: FAIL，输出 `✗ missing skills/gf-refactor/SKILL.md`，非零退出。

- [ ] **Step 3: 确认 help 列表能看到新 target**

Run: `make help | grep check-refactor-skill`
Expected: 输出一行 `check-refactor-skill    Verify gf-refactor skill meets Issue #332 acceptance criteria`

- [ ] **Step 4: Commit**

```bash
git add Makefile
git commit -m "test(refactor): 新增 check-refactor-skill 验收断言（RED）

把 Issue #332 中可机械验证的验收标准写成 Makefile target：
正文无语言标识、references/rust.md 存在、复用 detector.md、
单次一手法与编译测试校验规则、行为/重构分离、
When NOT to Refactor 排除无测试覆盖、与 gf-smell 共享证据强度/严重度词汇、
语义变更降级原则、技术目录全部标注语义风险（≥40 条）、
条件等价/可能变更不得自动应用、rust.md 三段契约与≥2条语义陷阱判例。

当前 skills/gf-refactor/ 尚不存在，断言必然失败。

Refs #332"
```

---

### Task 2: SKILL.md 正文（语言无关层）

**Files:**
- Create: `skills/gf-refactor/SKILL.md`

**Interfaces:**
- Consumes: Task 1 的 `make check-refactor-skill`
- Produces: 语言层契约（三个二级标题 `## 校验命令` / `## 惯用法映射` / `## 语义陷阱判例`），Task 3 的 `references/rust.md` 必须照此结构填写

- [ ] **Step 1: 创建目录并写入 SKILL.md**

```bash
mkdir -p skills/gf-refactor/references
```

写入 `skills/gf-refactor/SKILL.md`：

````markdown
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
| Encapsulate Variable 封装变量 | 等价 | 访问路径改变，值不变 |
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
| Move Statements into Function 搬移语句进入函数 | 等价 | 纯粹搬移，无新分支 |
| Move Statements to Callers 搬移语句到调用方 | 条件等价 | 多调用点时需逐一核对执行时机 |
| Replace Inline Code with Function Call 以函数调用取代内联代码 | 等价 | 已存在等价函数的替换 |
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
| Parameterize Function 令函数携带参数 | 等价 | 合并同构函数，行为由参数值决定 |
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
| Extract Superclass 提炼超类 | 等价 | 抽出共同行为，各子类行为不变 |
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
````

- [ ] **Step 2: 运行断言，确认语言无关层全绿、references 仍红**

Run: `make check-refactor-skill`
Expected: `✓ AC#1` / `✓ AC#3` / `✓ AC#4` / `✓ AC#5` / `✓ AC#6` / `✓ AC#7` / `✓ AC#8` / `✓ AC#9` / `✓ AC#10` / `✓ AC#11` 通过；`✗ AC#2 缺少 skills/gf-refactor/references/rust.md` 仍失败，退出码 1

- [ ] **Step 3: 单独复验 AC#10（技术目录标注计数）**

Run:
```bash
grep -cE '^\| [A-Z][A-Za-z ]+ [^ -~]+[^|]*\|[^|]*\|[^|]*\|$' skills/gf-refactor/SKILL.md
grep -cE '^\| [A-Z][A-Za-z ]+ [^ -~]+[^|]*\| (等价|条件等价|可能变更) \|[^|]*\|$' skills/gf-refactor/SKILL.md
```
Expected: 两条命令输出相同的数字（52），且 ≥40。

- [ ] **Step 4: Commit**

```bash
git add skills/gf-refactor/SKILL.md
git commit -m "feat(refactor): SKILL.md 语言无关重构协议

Golden Rules（一次一手法/必须编译干净/全部测试通过/立即 commit/
行为变更与重构分离）+ Decision Rubric + Semantic Safety Boundary
（语义变更一律降级为建议，等价/条件等价/可能变更三档）+
Fowler 六大类 52 条手法目录（逐条标注语义风险）+
Language Layer Contract + Safety Protocol 执行步骤 + 建议报告骨架
（复用 gf-smell 的证据强度/严重度词汇）。

When NOT to Refactor 章节明确排除无测试覆盖的代码路径。

Refs #332"
```

---

### Task 3: references/rust.md（本仓库唯一可实跑验证的语言层）

**Files:**
- Create: `skills/gf-refactor/references/rust.md`

**Interfaces:**
- Consumes: Task 2 的 Language Layer Contract
- Produces: Task 4 dogfooding 实跑所用的校验命令与惯用法映射

- [ ] **Step 1: 写入 `skills/gf-refactor/references/rust.md`**

````markdown
# Rust Refactoring Layer

**Detection:** `Cargo.toml` at project root (per `gf-quality/references/detector.md`).

## 校验命令

**每个手法应用后各自单独运行，不得合并成一次跑到底再回头看。**

```bash
make check   # 编译检查（不生成代码，比 build 快）
make test    # nextest 全量测试
```

若 `make check` 失败 → 撤销本次编辑，不进入测试。若 `make check` 通过但
`make test` 失败 → 撤销本次编辑，不应用下一个手法。

引入新依赖或改动 `Cargo.toml` 的手法（例如 Replace Constructor with Factory
Function 用到 `typed-builder`）额外跑：

```bash
make lint    # fmt + clippy，捕获新代码是否符合本仓库 pedantic 门槛
```

**不得使用 `cargo clean`**——CLAUDE.md 明确禁止，需要时须先问用户。

## 惯用法映射

| 手法 | Rust 落点 |
|---|---|
| Extract Function 提炼函数 | 私有 `fn`，签名从调用点的类型推导 |
| Introduce Parameter Object 引入参数对象 | 字段 >5 时用 `typed-builder`（本仓库约定），否则普通 struct |
| Replace Conditional with Polymorphism 以多态取代条件表达式 | `enum` + `match`（穷尽性检查代替「遗漏分支」的语义风险）；跨 crate 扩展点才用 trait object |
| Replace Type Code with Subclasses 以子类取代类型码 | `enum` variants，而非类层次（Rust 无类继承） |
| Extract Superclass / Collapse Hierarchy 提炼超类/折叠继承体系 | `trait` 默认方法，或直接合并（组合优先于继承） |
| Encapsulate Variable 封装变量 | 私有字段 + `pub fn` getter |
| Change Value to Reference 将值对象改为引用对象 | `Rc<RefCell<T>>` 或 `Arc<Mutex<T>>`——见陷阱 1 |
| Replace Derived Variable with Query 以查询取代派生变量 | 移除缓存字段，改成方法；若原字段是缓存，见陷阱 2 |

## 语义陷阱判例

### 陷阱 1: Change Value to Reference 与 `Drop` 时机

把一个值类型字段改成 `Rc<RefCell<T>>` 后，原本随值类型的所有者一起在作用域
结束时确定性析构的 `T`，析构时机变成"最后一个 `Rc` 引用计数归零时"——若 `T`
的 `Drop` 有可观察副作用（关闭文件、释放锁、写日志），析构时机从"编译期确定"
变成"运行期依赖引用计数"。这正是「可能变更」标注的具体体现，不能当等价手法
处理。

### 陷阱 2: Split Loop / Replace Loop with Pipeline 与迭代器惰性求值

Rust 的 `Iterator` 是惰性的：`.map(f)` 不会立即调用 `f`，只有被 `.collect()`
/ `.for_each()` / `for` 消费时才真正求值。把一个带副作用的 `for` 循环体拆成
`.iter().map(|x| { side_effect(x); x })` 而不消费返回值，`side_effect`
根本不会执行——这不是"重构后行为不变"，是重构后代码变成死代码。Split Loop /
Replace Loop with Pipeline 应用前必须确认：拆分出的每个迭代器链都有终结消费
者，且消费顺序与原循环的副作用顺序一致。

### 陷阱 3: Extract Function 与闭包捕获

把一段引用外层可变局部变量的代码提炼成闭包（而非独立 `fn`）时，闭包会捕获该
变量的可变借用，其生命周期从"和原作用域一致"变成"和闭包一致"——若闭包被存进
一个比原作用域更长寿的结构（如注册进事件回调表），编译器会报借用检查错误；若
通过 `move` 强行让闭包拥有变量所有权则消除了错误，但也悄悄改变了"谁最终拥有
这份状态"，调用方对该变量的后续读取会读到一份已被移动的影子。Extract Function
标注为「等价」隐含的前提是"提炼成独立函数，不提炼成捕获式闭包"，一旦落到闭包
就需要重新评估为 条件等价。

### 陷阱 4: Remove Setting Method 与测试 fixture 复用路径

若某字段的设值方法（setter）被移除，字段变为仅构造时可设；但该类型若派生了
`#[derive(Default)]` 且其它代码通过 `T::default()` 再手动赋值来"复用" setter
路径（常见于测试 fixture），移除 setter 会使这条路径编译失败——这不是行为
改变，是接口改变，但由于错误只在依赖它的下游代码触发，本地 `make check` 未必
覆盖，须额外搜索 workspace 内全部调用点。

## 工具缺失降级

| 缺失 | 降级 |
|---|---|
| `cargo-nextest` 未安装（`make test` 依赖它） | 提示安装；本层的校验命令强制依赖它，不得回退到裸 `cargo test`（覆盖率/汇报格式不一致） |
| Rust 工具链版本与 `rust-toolchain.toml` 不符 | 停止，不自行切换工具链版本（配置文件变更需用户确认） |
````

- [ ] **Step 2: 运行断言，确认全绿**

Run: `make check-refactor-skill`
Expected: 全部 `✓`，最后一行 `ALL CHECKS PASSED`，退出码 0

- [ ] **Step 3: 确认三段契约齐备且陷阱数≥2**

Run:
```bash
grep -c '^## \(校验命令\|惯用法映射\|语义陷阱判例\)$' skills/gf-refactor/references/rust.md
grep -c '^### 陷阱' skills/gf-refactor/references/rust.md
```
Expected: 第一条输出 `3`，第二条输出 `4`

- [ ] **Step 4: Commit**

```bash
git add skills/gf-refactor/references/rust.md
git commit -m "feat(refactor): Rust 重构层

校验命令（make check / make test，附加 make lint 触发条件）+
8 条手法的惯用法映射（enum+match、typed-builder、trait 默认方法等）+
4 条语义陷阱判例：Rc<RefCell<T>> 的 Drop 时机、迭代器惰性求值、
Extract Function 闭包捕获、Remove Setting Method 与测试 fixture 复用路径。

Refs #332"
```

---

### Task 4: Dogfooding 实跑（一次真实的 Extract Function）

**Files:**
- Modify: `apps/cli/src/commands/label.rs`
- Create: `docs/refactor-report-gitflow-cli-2026-09-18.md`
- Modify: `docs/index.md`（Reports Archive 一节新增 `refactor-report-*.md` 家族）

**Interfaces:**
- Consumes: Task 3 的 `references/rust.md` 校验命令；`docs/smell-report-gitflow-cli-2026-09-16.md` 提供的候选（`label.rs:135` `handle_label` 101/100 行）
- Produces: 无（终端产物，证明 skill 可端到端工作）

`gf-smell` 的既有报告已标出 `apps/cli/src/commands/label.rs:135` 的
`handle_label` 越界（101/100 行）。本 Task 从其内部提炼一个自包含的子块——
函数体开头按 `platform` 选择 `Box<dyn LabelProvider>` 的 `match`
（第 205-220 行）——这段代码纯粹是"选择哪个 provider 实现"，无副作用，输入
输出边界清晰，是本目录里 `Extract Function 提炼函数`（等价档）的典型候选。

- [ ] **Step 1: 确认目标函数当前状态与既有测试覆盖**

Run:
```bash
sed -n '198,221p' apps/cli/src/commands/label.rs
grep -rn 'handle_label' apps/cli/src/commands/label.rs apps/cli/tests/ 2>/dev/null
```
Expected: 看到 `pub async fn handle_label` 的 provider 选择 `match`（`platform` →
`GitHubLabelProvider` / `GitLabLabelProvider` / `GitCodeLabelProvider` /
错误分支）；确认存在覆盖该函数路径的测试（集成测试按 `platform` 分支跑）。
若未发现任何覆盖，按 SKILL.md「When NOT to Refactor」**停止**，不得继续本 Task
（本步骤即該規則的實地验证）。

- [ ] **Step 2: 应用 Extract Function（等价档，唯一手法）**

按 SKILL.md Safety Protocol 提炼一个私有函数，签名与原 `match` 表达式的
输入输出完全一致：

```rust
/// 根据 `platform` 与可选的自定义 `remote_url` 选择对应的 Label 提供者实现。
///
/// # Errors
///
/// 返回错误当 `platform` 不在 `github` / `gitlab` / `gitcode` 之列。
fn select_label_provider(
    platform: &str,
    repo: &str,
    remote_url: &str,
) -> miette::Result<Box<dyn LabelProvider>> {
    let provider: Box<dyn LabelProvider> = match platform {
        "github" => Box::new(GitHubLabelProvider::new(repo)),
        "gitlab" => {
            if remote_url.is_empty() {
                Box::new(GitLabLabelProvider::new(repo))
            } else {
                Box::new(GitLabLabelProvider::with_remote_url(repo, remote_url))
            }
        }
        "gitcode" => Box::new(GitCodeLabelProvider::new(repo)),
        other => {
            return Err(miette::miette!(
                "Platform '{other}' not yet supported for label commands"
            ));
        }
    };
    Ok(provider)
}
```

`handle_label` 第 205-220 行改为：

```rust
    let provider = select_label_provider(platform, repo, remote_url)?;
```

- [ ] **Step 3: 校验（Safety Protocol Step 2-3）**

Run: `make check`
Expected: 编译通过，零错误。若失败，撤销本次编辑（`git checkout -- apps/cli/src/commands/label.rs`），停止本 Task，不进入下一步。

Run: `make test`
Expected: 全部测试通过，覆盖 `handle_label` 的用例（github/gitlab/gitcode/未知平台四个分支）结果与提炼前一致。若失败，撤销本次编辑，停止本 Task。

- [ ] **Step 4: 确认行数下降且未混入行为变更**

Run:
```bash
git diff apps/cli/src/commands/label.rs | grep -c '^[+-]'
awk '/pub async fn handle_label/,/^}/' apps/cli/src/commands/label.rs | wc -l
```
Expected: `handle_label` 本体行数低于原 101 行（provider 选择逻辑已搬出）；diff
中不应出现任何 `match` 分支条件、错误信息文案、返回类型的变化——只有"这段代码
现在住在哪个函数里"发生了变化。

- [ ] **Step 5: Commit（Safety Protocol Step 4，单独一个 commit）**

```bash
git add apps/cli/src/commands/label.rs
git commit -m "refactor(label): Extract Function 提炼 select_label_provider

从 handle_label 中提炼 provider 选择逻辑为独立函数，签名保持输入输出一致
（platform/repo/remote_url → Result<Box<dyn LabelProvider>>）。

语义风险：等价。编译与测试均在提炼后单独验证通过（make check && make test）。

Refs #332"
```

- [ ] **Step 6: 写建议报告（含未自动应用的 条件等价 候选）**

`gf-smell` 报告中同批越界的 `pr.rs:214`、`issue.rs:175`、`release.rs:138`、
`label.rs:264`（`handle_milestone`）适用同一种 Extract Function 手法，但它们
的 `match` 臂内联了参数解析与分支逻辑（据 smell 报告"11 个薄壳 + 4 个内联"的
发现），不是像本 Task 处理的这段一样纯粹的选择逻辑——需要先确认提炼边界不会
切断某个臂内的提前 return 或跨臂共享的局部变量，属于 条件等价（Slide
Statements 的语义陷阱同样适用）。将这些候选写入建议报告，不在本 Task 内应用。

写入 `docs/refactor-report-gitflow-cli-2026-09-18.md`：

```markdown
# Refactor Report — gitflow-cli
生成时间：2026-09-18 · 目标范围：label.rs 及 gf-smell 同批候选 · 应用手法数：1 · 建议手法数：1

## 已应用（等价，自动应用）
### RF-001 · Extract Function 提炼函数 · commit <本 Task Step 5 的 commit sha>
- 位置：apps/cli/src/commands/label.rs:205-220（提炼前）→ 新增 `select_label_provider`
- 校验：编译 ✓ 测试 ✓

## 建议（条件等价 / 可能变更，未自动应用）
### RF-101 · Extract Function 提炼函数 · 条件等价
- 位置：apps/cli/src/commands/pr.rs:214、issue.rs:175、release.rs:138、label.rs:264
- 证据强度：Observed（阅读 pr.rs:214 完整函数体已确认 4 个内联分支的具体位置）
- 置信度：Medium
- 严重度：Medium
- 风险说明：这 4 处 `match` 臂内联了参数解析与三路分支逻辑，提炼前需逐臂确认
  是否存在提前 return 或跨臂共享的局部变量（Slide Statements 的同类风险）；
  与本 Task 处理的纯选择逻辑不同，不能直接套用同一提炼边界
- 前置条件：逐处人工核实臂内是否有跨语句的隐含顺序依赖；未验证
- 建议：待用户确认后逐处应用 Extract Function，每处单独校验、单独 commit

## 已排除
（无 — 本次范围仅取自 gf-smell 既有报告的候选，未新增排除项）
```

- [ ] **Step 7: 更新 `docs/index.md`**

在 Reports Archive 一节的 `smell-report-*.md` 行之后插入：

```markdown
- `refactor-report-*.md` — behavior-preserving refactor runs from `gf-refactor`.
```

- [ ] **Step 8: 运行完整验证套件**

Run: `make check-refactor-skill && make check-agent-sync`
Expected: `ALL CHECKS PASSED`，两条命令退出码均为 0

- [ ] **Step 9: Commit（文档改动，与 Step 5 的代码 commit 分开）**

```bash
git add docs/refactor-report-gitflow-cli-2026-09-18.md docs/index.md
git commit -m "docs(refactor): 本仓库 dogfooding 实跑报告

对 label.rs::handle_label 应用 1 处 Extract Function（等价档），
编译测试均通过后单独 commit。同批 4 处内联分支候选降级为条件等价建议，
写入报告未自动应用，附风险说明与未验证的前置条件。

docs/index.md 新增 refactor-report-*.md 报告家族。

Refs #332"
```

---

## Self-Review

**1. Spec coverage**

| Issue #332 验收条款 | 对应 Task |
|---|---|
| AC#1 正文无语言专属命令/惯用法 | Task 1 断言 · Task 2 正文 |
| AC#2 references/ 至少含 rust.md，结构可扩展 | Task 1 断言 · Task 3 · Task 2 Language Layer Contract |
| AC#3 复用 gf-quality/references/detector.md | Task 1 断言 · Task 2 正文 |
| AC#4 一次只应用一个手法 | Task 1 断言 · Task 2 Golden Rules · Task 4 Step 2/5（唯一一次提炼、单独 commit） |
| AC#5 每手法后执行构建测试，失败即停止 | Task 1 断言 · Task 2 Safety Protocol Execution · Task 4 Step 3 |
| AC#6 行为变更与重构不混在同一次改动 | Task 1 断言 · Task 2 Golden Rules #5 · Task 4 Step 4/5/9（代码 commit 与文档 commit 分离） |
| AC#7 含 When NOT to Refactor，排除无测试覆盖 | Task 2 正文 · Task 4 Step 1（实地验证） |
| AC#8 与 #327 共享严重度/证据强度词汇 | Task 1 断言（对照 gf-smell/SKILL.md）· Task 2 Report 骨架 · Task 4 Step 6 |
| AC#9 含「语义变更一律降级为建议」原则 | Task 1 断言 · Task 2 Semantic Safety Boundary |
| AC#10 每手法标注语义风险 | Task 1 断言（52 条计数校验）· Task 2 Technique Catalog |
| AC#11 条件等价/可能变更不得自动应用 | Task 1 断言 · Task 2 正文 · Task 4 Step 6（RF-101 降级为建议未应用） |
| AC#12 references/<lang>.md 含语义陷阱判例 | Task 1 断言（≥2 条 `### 陷阱`）· Task 3（4 条） |

无缺口。

**2. Placeholder scan**

无 TBD / TODO / "similar to Task N" / "add appropriate error handling"。Task 4
的 Rust 代码与 commit message 逐字写出，Task 2/3 的 skill 正文逐字写出。

**3. Type consistency**

- 三段契约标题在 Task 1 断言、Task 2 契约表、Task 3 实际文件、Task 3 Step 3 复验中一律为 `## 校验命令` / `## 惯用法映射` / `## 语义陷阱判例`
- 语义风险三档在全文一律为 `等价` / `条件等价` / `可能变更`
- 证据强度/严重度词汇在 Task 1 断言、Task 2 Report 骨架、Task 4 Step 6 报告实例中一律为 `Measured` / `Observed` / `Inferred` / `证据强度` / `置信度` / `严重度`
- Makefile target 名在全文一律为 `check-refactor-skill`
- Task 4 新增函数签名 `select_label_provider(platform: &str, repo: &str, remote_url: &str) -> miette::Result<Box<dyn LabelProvider>>` 在 Step 2 定义、Step 6 报告位置描述中保持一致
- 报告路径在 Task 4 Step 6、7、9 中一律为 `docs/refactor-report-gitflow-cli-2026-09-18.md`
</content>
