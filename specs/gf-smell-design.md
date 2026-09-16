# gf-smell 设计规格 —— 代码坏味道与复杂度热点检测

> Issue [#327](https://github.com/byx-darwin/gitflow-cli/issues/327) · 工作流合同 `wf-2026-09-16-001` · 2026-09-16

## 1. 背景与问题

本仓库当前的代码质量检测手段只有两处：

| 手段 | 位置 | 局限 |
|---|---|---|
| `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic` | `make clippy` / CI | 不含 nursery 复杂度 lint；被源码 `#[allow]` 抑制的项不可见 |
| 6 条人工勾选项 | `docs/references/pr-review-checklist.md:33-40` | 无阈值定义、无复杂度度量、无架构级反模式识别 |

两者都缺少**判定协议**：阈值越界之后如何从候选走到结论，证据强度如何标注，如何按根因去重，什么不该报。

对照分析 `smallnest/goal-workflow` 的 `skills/smell/SKILL.md` 后确认，其真正价值不在 58 条启发式规则，而在这套判定协议——且该协议是语言无关的。

## 2. 目标

新增 `skills/gf-smell/`：

- **SKILL.md** 承载语言无关的候选 → 验证 → 确认三阶段判定协议
- **`references/<lang>.md`** 承载各语言的检测工具、阈值与类目映射
- 默认**不自动修复**

## 3. 方案选型

判定协议的组织方式考虑过三种：

| 方案 | 做法 | 否决理由 |
|---|---|---|
| A 纯三阶段流水线 | SKILL.md 只定义三阶段输入/输出契约，坏味道名字由工具输出决定 | 报告的分类轴退化为 `clippy::cognitive_complexity` 这类工具专属标识，跨语言不可比 |
| B 规则目录式（对方做法） | SKILL.md 列 58 条启发式规则，各带阈值 | 该粒度必然夹带语言细节，直接违反「正文不含单一语言工具名或文件名」 |
| **C 三阶段 + 语言无关类目表** ✅ | 三阶段提供判定纪律；另有 10 条语言无关类目表，只给类目名与定义，阈值全部下沉语言层 | 采纳 |

采纳 C 的理由：类目表是报告的稳定分类轴，跨语言可比；阈值留在语言层同时满足「正文无语言标识」与「复用既有语言探测」两条约束；类目数量控制在 10 条以内，不会滑向 B 的 750 行。

**集成范围：独立 skill**。不接入 gf-workflow Phase 4，不改写 `pr-review-checklist.md`，不改 gf-quality。

## 4. 架构

### 4.1 文件布局

```
skills/gf-smell/
├── SKILL.md              # 三阶段协议 + 类目表 + What NOT to Flag + 报告骨架 + 语言层契约
└── references/
    ├── rust.md           # clippy nursery lint + rg/awk 结构扫描 + 阈值 + 类目映射
    ├── go.md             # gocyclo / staticcheck / go vet
    ├── node.md           # eslint complexity 规则族
    ├── python.md         # ruff C901 / radon
    └── java.md           # PMD / checkstyle
```

与 `gf-quality`、`gf-precommit` 的布局一致（SKILL.md + `references/` 五语言，无 per-skill 脚本）。

「结构可扩展」由 SKILL.md 内的 **Language Layer Contract** 一节承载，规定语言层必须提供哪几张表，不另设空契约文件——五份实例已把结构演示清楚。

### 4.2 语言探测

复用同级 skill 的 `gf-quality/references/detector.md`，**不得自行实现**。gf-smell 只消费其产出（语言 · 路径 · workspace 类型），映射到自己的 `references/<lang>.md`。多语言项目沿用 detector.md 已定义的「列出清单让用户选 1/2/all」交互。

### 4.3 三阶段数据流

```
Stage 0  Scope     用户指定路径 / diff 范围 / 全仓库（默认全仓库源码目录）
                   排除：target/ · .worktree/ · .claude/worktrees/ · node_modules/ · vendor/
   ↓
Stage 1  Detect    探测语言 → 载入 references/<lang>.md → 单次运行检测命令并捕获输出
                   产出：候选信号表（类目 · 位置 · 实测值 · 阈值 · 来源工具）
                   ⚠️ 阈值越界 = 候选信号，不是结论
   ↓
Stage 2  Verify    逐候选读代码上下文 → 标注证据强度 → 套用 What NOT to Flag 排除
                   无法在不测量的前提下判定的 → 降入「待测量候选」独立档
   ↓
Stage 3  Confirm   按根因去重 → 定严重度 → 报告落盘 docs/ + 更新 docs/index.md
```

Stage 0 的排除清单不是可选项：本仓库 `.claude/worktrees/` 下存在遗留的 agent worktree 副本，不排除会使同一文件被重复统计三次。

## 5. 判定模型

### 5.1 语言无关类目表

| 类目 | 语言无关定义 |
|---|---|
| Long Function | 单个函数的语句量超出可一屏通读的范围 |
| Deep Nesting | 控制流嵌套层级过深 |
| Excessive Parameters | 函数参数个数过多 |
| God Structure | 单一类型或模块聚集过多职责与状态 |
| Duplicated Logic | 同一逻辑在多处复制 |
| Feature Envy | 函数对外部类型数据的访问多于对自身 |
| Primitive Obsession | 用基本类型表达本应有领域类型的概念 |
| Shotgun Surgery | 一个语义变更需要在多处同步修改 |
| Cyclic Dependency | 模块间存在循环依赖 |
| Dead Code | 不可达或无引用的代码 |

阈值一律不在此表。同一类目在不同语言层映射到不同工具与阈值，由 `references/<lang>.md` 负责。

### 5.2 证据强度 × 置信度：两个维度，分别标注

| 证据强度 | 含义 |
|---|---|
| **Measured** | 数值来自工具的确定性输出 |
| **Observed** | 直接读代码看到的结构事实 |
| **Inferred** | 需要推理才能得出的判断 |

**解耦的实际含义：高证据强度 ≠ 高置信度。** 一个 Measured 的复杂度越界可能置信度很低（复杂度全来自扁平分发表，可读性并不差）；一个 Inferred 的判断可能置信度很高（多处复制粘贴的错误处理，历史上已漏改过）。

两个维度在报告中**各占一行**，禁止合并成综合分——合并等于让 Inferred 搭 Measured 的便车。

置信度同样三档：High / Medium / Low。

### 5.3 严重度：三档 + 独立第四档

- **High / Medium / Low** —— 计入统计
- **Candidate requiring measurement（待测量候选）** —— 独立成档，**不计入严重度统计**

进第四档的判据：结论依赖一个本次并未实际测量的量（运行时热度、真实调用频次、实际输入规模）。

该档的意义是堵住「我猜这里慢」伪装成「这里有性能坏味道」的路径。

### 5.4 去重：两条正交规则

合并有两种情形，判据不同，不可混为一谈。

**规则 1 —— 根因合并（因果传递）**

判据：**修掉 A 之后 B 自动消失。** 此时 A 是根因，B 是表现，合并为 1 条，类目取根因的类目。

例：一个过宽的 struct 导致 5 个函数参数爆炸 → 1 条 `God Structure`（位置列 5 处），不是 5 条 `Excessive Parameters`。改窄 struct 后参数爆炸自动消失。

**规则 2 —— 模式合并（同一修复决策）**

判据：多处**互相独立**（修一处不影响其余），但它们是同一设计模式的重复实例，**是否修复是同一个决策**。合并为 1 条，位置列全，并在根因字段写明「同一模式的 N 个独立实例，每处需各自改动」。

例：`apps/cli/src/commands/{issue,label,label,pr,release}.rs` 的 5 处 `handle` 均超长，根因同为「命令分发把所有子命令体内联在一个 `match` 里」。拆短 `issue.rs::handle` 不会让 `pr.rs::handle` 变短，因此**不适用规则 1**；但「要不要改变命令分发的形状」是一个决策，因此按规则 2 合并为 1 条 `Long Function`，位置列 5 处。

**为什么必须区分**：规则 1 合并后修复量是 1 处，规则 2 合并后修复量是 N 处。报告若不写明，读者会低估工作量。所以规则 2 的 finding 必须显式标注实例数。

### 5.5 What NOT to Flag

| 排除项 | 理由 |
|---|---|
| 冷路径 | 非热路径代码的性能类坏味道 |
| 有意权衡 | 注释 / ADR / spec 已显式说明取舍 |
| 已优化代码 | 有 benchmark 或 profile 佐证现状 |
| 生成代码 | build script 产物、宏展开、vendored |
| 测试夹具 | 为覆盖边界刻意构造的冗长夹具 |
| CI 已强制拦截项 | 重复报告已被门控拦下的东西没有增量 |

## 6. 报告格式

落盘路径：`docs/smell-report-<scope>-<YYYY-MM-DD>.md`，与 `docs/code-review-report-pr326-2026-09-06.md`、`docs/dogfooding-report-2026-08-04-pr127.md` 同风格。同步更新 `docs/index.md`。

```markdown
# Smell Report — <scope>
生成时间 · 探测语言 · 扫描范围 · 检测工具与版本

## 摘要
| 严重度 | 条数 |
|---|---|
| High / Medium / Low | n |
> 待测量候选 m 条，单独成档，不计入上表

## Findings
### SM-001 · <类目> · <严重度>
- 位置：path:line（同根因多处全列）
- 证据强度：Measured | Observed | Inferred
- 置信度：High | Medium | Low
- 实测值 / 阈值：
- 根因：（规则 2 合并的须写明「同一模式的 N 个独立实例」）
- 影响：
- 建议：（仅建议，不执行）

## 待测量候选（不计入严重度统计）
### SC-001 · <类目>
- 位置 · 需要测量什么 · 如何测量

## 已排除（What NOT to Flag 命中）
| 位置 | 命中排除项 | 理由 |
```

末尾的「已排除」表不是装饰：把排除决策摊开写，复核者才能同时检查**漏报**（该排的没排）与**过排**（不该排的排了）。

## 7. 只读约束

`allowed-tools: Read, Grep, Glob, Bash`（不含 Write / Edit），报告用 Bash heredoc 落盘。

**诚实声明**：Bash 本身也能改源码，所以「不自动修复」的**真正保障来自正文的 Out of Scope / Do Not / Red Flags 政策段落**，frontmatter 只挡掉最顺手的那条路。因此两者同时保留，不可只留其一。

这也解决了验收标准 #9（工具集不含 Write/Edit）与 #10（报告落盘到 `docs/`）之间的字面冲突。

## 8. Rust 检测层（已实测验证）

### 8.1 工具选型

放弃 Issue 原文提到的 `cargo-geiger`：它度量 unsafe 代码用量，而本仓库 CLAUDE.md 强制 `#![forbid(unsafe_code)]`，该项恒为零，无信息量。Rust 生态亦无 `gocyclo` 对等物。

实际采用两层：

| 层 | 覆盖 | 证据强度上限 |
|---|---|---|
| clippy nursery/pedantic 复杂度 lint | 函数级认知复杂度、长函数、参数爆炸、深嵌套、类型复杂度 | **Measured** |
| `rg` / `awk` 结构扫描 | 文件长度、impl 块体量、模块扇出、重复块 | **Observed**（见 8.2 第 3 条） |

### 8.2 三条从实测中长出的硬性约束

1. **必须用 `--force-warn` 而非 `-W`。** 本仓库有 8+ 处 `#[allow(clippy::too_many_lines)]`（`review.rs:127`、`commit.rs:76`、`auth.rs:49`、`pr.rs:211` 等），`-W` 穿不透。「已被 allow」不等于问题消失，只等于有人决定不看它。

2. **单次运行、捕获输出复用。** cargo 缓存会让重复调用返回空输出，把「缓存」误报成「干净」。设计过程中已实际踩中：同一命令连跑两遍，第二遍计数为 0。

3. **结构扫描产出只能标 Observed，不得标 Measured。** 花括号计数会被 format string 的 `{}`、raw string 骗。设计过程中已实际踩中：一个 awk 扫描把测试函数误报为 401 行。

### 8.3 `#[allow]` 的双向判定

被 `#[allow]` 抑制的复杂度 lint：

- **带理由注释** → 命中 What NOT to Flag 的「有意权衡」，进「已排除」表
- **裸 allow** → 进候选信号

这个区分是 clippy 本身给不了的增量。

### 8.4 本仓库基线（2026-09-16 实测）

```
cargo clippy --workspace --all-targets -- \
  --force-warn clippy::too_many_lines \
  --force-warn clippy::cognitive_complexity \
  --force-warn clippy::excessive_nesting \
  --force-warn clippy::too_many_arguments \
  --force-warn clippy::type_complexity
```

| lint | 命中 |
|---|---|
| `too_many_lines` | 5（`issue.rs:175` 172/100 · `label.rs:135` 101/100 · `label.rs:264` 113/100 · `pr.rs:214` 239/100 · `release.rs:138` 129/100） |
| `cognitive_complexity` | 0 |
| `excessive_nesting` | 0 —— **该 0 无效，见下** |
| `too_many_arguments` | 0 |
| `type_complexity` | 0 |

**更正（Task 3 审查发现，已独立复验）**：`excessive_nesting` 的 0 命中**不是**代码干净的
证据。`excessive-nesting-threshold` 的工具默认值是 **0，含义为禁用**，未在 `clippy.toml`
显式配置时该 lint 在任何嵌套深度都不触发，`--force-warn` 也无法激活——它是惰性而非被抑制。
复验：8 层嵌套的样本在无配置时 0 命中，加 `excessive-nesting-threshold = 3` 后立即 2 命中。
本仓库不得修改 `clippy.toml`，故该 lint 在此恒不触发，`Deep Nesting` 类目改由结构缩进扫描
承载，证据强度上限 `Observed`。

同一怀疑已对 `cognitive_complexity` 复验并排除：用复杂度 61 的样本测试，无配置时即以
`61/25` 触发，其默认阈值 25 确实生效，故它的 0 命中**仍是有效证据**。两个 lint 行为不同，
不可一并推翻。

**这组数字本身就是判定模型的自证样本**：5 个函数长（Measured 越界）但认知复杂度为零，说明它们是扁平的 `match` 子命令分发——证据强度 Measured 高、置信度 Low。且 5 处同属一个设计模式，按 §5.4 规则 2 合并后应为 1 条、标注 5 个独立实例。

## 9. 测试策略

### 9.1 TDD 映射

Skill 是 Markdown，TDD 映射为**可执行验收断言先行**：

- **RED** —— 新增 Makefile target `check-smell-skill`，把可机械验证的验收标准写成断言，此时全红
- **GREEN** —— 写 SKILL.md + 5 份 references，使断言转绿
- **REFACTOR** —— 精简正文，保持绿

### 9.2 验收标准可验证性分层

可机械断言：

| 验收标准 | 断言 |
|---|---|
| 正文无语言专属标识 | `grep -E 'clippy\|cargo\|gocyclo\|eslint\|ruff\|PMD\|Cargo\.toml\|go\.mod\|package\.json' SKILL.md` 零命中 |
| references 五份齐备 | `test -f` 五个文件 |
| 复用 detector.md | SKILL.md 命中 `gf-quality/references/detector.md`；不自建探测由上一条 marker 文件名断言覆盖 |
| 每条发现带证据强度 | 模板含三档；实跑报告每个 finding 块有「证据强度」行 |
| 待测量候选不计入统计 | SKILL.md 含该档定义，且模板中该档位于摘要表之外 |
| What NOT to Flag 章节 | grep 命中「冷路径 / 有意权衡 / 已优化代码」 |
| 工具集不含 Write/Edit | frontmatter `allowed-tools` 行不含 `Write`/`Edit` |

需人工复核：

| 验收标准 | 复核方式 |
|---|---|
| 命中已知长函数 | 已提前证实可行（§8.4 五条命中） |
| 同根因去重 | 实跑报告须按 §5.4 规则 2 把 5 处 `handle` 合并为 1 条，并标注 5 个独立实例 |
| 误报率可复核 | 「已排除」表让复核者能同时查漏报与过排 |

### 9.3 验证命令

skill-only 变更，按 CLAUDE.md 不跑 Rust 全套：

- `make check-smell-skill`（新增）
- `make check-agent-sync`
- Markdown 索引与链接检查（`specs/index.md`、`docs/index.md`）

dogfooding 实跑会调用 `cargo clippy`（只读，不改工作区）。

## 10. 非目标

- 不自动修复任何坏味道
- 不接入 gf-workflow Phase 4 的并行派发集
- 不改写 `docs/references/pr-review-checklist.md`
- 不修改 `clippy.toml`、`deny.toml`、`.pre-commit-config.yaml`、`rust-toolchain.toml`
- 不在非 Rust 语言层做实跑验证（本仓库无对应语言代码，其命令与阈值以各语言上游工具官方文档为依据）
