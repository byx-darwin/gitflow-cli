# Smell Report — gitflow-cli

- **生成时间**：2026-09-16
- **探测语言**：Rust（根目录存在 `Cargo.toml`，按 `gf-quality/references/detector.md` 判定为 Cargo workspace，加载 `skills/gf-smell/references/rust.md`）
- **扫描范围**：整个工作区 108 个 `.rs` 文件 / 43,361 行。按 Stage 0 排除 `target/`、`.worktree/`、`.claude/worktrees/`、`vendor/`
- **检测工具与版本**：`clippy 0.1.96 (ac68faa20c 2026-05-25)` · `rustc 1.96.0` · `cargo 1.96.0` · GNU 风格 `find` / `grep` / `awk` 结构扫描
- **修订**：2026-09-16 按 `skills/gf-smell` 修订版重跑 `dead_code` 并复核抑制判定，见文末「报告修订记录」
- **位置引用约定**：所有 `path:line` 一律指向**声明行**，即 `fn` / `struct` / `impl` 关键字所在那一行，与 clippy 诊断 `-->` 给出的行号一致；其上的 `#[allow(...)]`、`#[must_use]`、`///` 文档注释**不计入**起点。`path:a-b` 形式表示代码区间（函数体或文件区段）的起止行。全文已按此约定逐条复核：`pr.rs:214`、`issue.rs:175`、`release.rs:138`、`label.rs:135` / `:264` 均为 `pub async fn` 行（其 `#[allow]` 分别在 `:211`、`:172`、`:135`、`:132` / `:261`）；`toon.rs:65` 为 `pub fn analyze` 行（`#[must_use]` 在 `:64`）

## 摘要

| 严重度 | 条数 |
|---|---|
| High | 0 |
| Medium | 2 |
| Low | 1 |

> 待测量候选 2 条，单独成档，不计入上表。

## 检测执行记录

Stage 1 按 `references/rust.md` 分两次捕获，每条命令**只运行一次**、各自捕获后复用：

1. 结构类 lint 用 `--all-targets`，输出 782 行，含 `Compiling gitflow-cli v1.9.0` 编译行，12 个 target 诊断完整回放，非缓存空跑。
2. `dead_code` 用**默认 target**（不加 `--all-targets`），输出 295 行，含编译行，命中 22 条。

`--force-warn` 确实穿透了源码里的 `#[allow(...)]`：本仓库 5 处 `too_many_lines` 命中
全部位于带 `#[allow(clippy::too_many_lines, reason = ...)]` 的函数上。这同时构成
`cognitive_complexity` **零命中是真实结果**的证据——若 `--force-warn` 未生效，
`too_many_lines` 也不会有任何输出。

| 信号 | 命中 | 阈值 | 说明 |
|---|---|---|---|
| `clippy::too_many_lines` | 5 | 100 行 | 见候选表 |
| `clippy::cognitive_complexity` | 0 | 25 | 真实零命中，非缓存、非抑制 |
| `clippy::excessive_nesting` | 0 | —— | **不构成证据**：该 lint 默认阈值为 0（即关闭），启用需在 `clippy.toml` 显式配置 `excessive-nesting-threshold`；已确认本仓库 `clippy.toml` 只含 `disallowed-types` / `disallowed-methods`，未配置该项，且本次扫描不修改策略配置。Deep Nesting 类目改用结构扫描 |
| `clippy::too_many_arguments` | 0 | 7 个 | —— |
| `clippy::type_complexity` | 0 | 250 | —— |
| `dead_code` | **22**（默认 target 捕获） | —— | 另有 54 个位置只出现在 `--all-targets` 捕获中，是二进制 crate 被重编为 test harness 的产物，按修订后的检测程序**不进候选表**；22 条真实候选 Stage 2 后全部落入已排除，见下 |
| 结构扫描：文件行数 | **15** 个文件 > 800 行 | 800 行 | Stage 2 后仅 1 个存活 |
| 结构扫描：单文件函数数 | **16** 个文件 > 40 个 | 40 个 | Stage 2 后 0 个存活 |
| 结构扫描：模块扇出 | 0 | 15 个内部模块 | 全仓库最大扇出为 1，无信号 |
| 结构扫描：花括号嵌套深度 | 最大 7（`crates/core/src/toon.rs`） | —— | 逐条人工复核，见 SM-002 |

### Stage 1 候选表（尚无一条是 finding）

| Category | Location | Measured value | Threshold | Source |
|---|---|---|---|---|
| Long Function | `apps/cli/src/commands/issue.rs:175` | 172 行 | 100 | `clippy::too_many_lines` |
| Long Function | `apps/cli/src/commands/label.rs:135` | 101 行 | 100 | `clippy::too_many_lines` |
| Long Function | `apps/cli/src/commands/label.rs:264` | 113 行 | 100 | `clippy::too_many_lines` |
| Long Function | `apps/cli/src/commands/pr.rs:214` | 239 行 | 100 | `clippy::too_many_lines` |
| Long Function | `apps/cli/src/commands/release.rs:138` | 129 行 | 100 | `clippy::too_many_lines` |
| Dead Code | 22 处（`core` 2 · `gitcode` 7 · `gitlab` 8 · `github` 3 · `e2e-core` 1 · `apps/cli` 1） | —— | —— | `dead_code`（默认 target） |
| God Structure | 15 个文件 | 1786 ~ 821 行 | 800 行 | 结构扫描（文件行数） |
| God Structure | 16 个文件 | 82 ~ 41 个函数 | 40 个 | 结构扫描（函数数） |
| Deep Nesting | `crates/core/src/toon.rs` 等 10 个文件 | 花括号深度 7 ~ 6 | —— | 结构扫描（嵌套深度） |
| Duplicated Logic | `crates/github` / `gitlab` / `gitcode` 三套适配器 | —— | —— | 结构扫描 + 阅读比对 |

## Findings

### SM-001 · Long Function · Low

- **位置**（规则 2 模式合并，全列）：
  - `apps/cli/src/commands/pr.rs:214` — `handle`，239/100
  - `apps/cli/src/commands/issue.rs:175` — `handle`，172/100
  - `apps/cli/src/commands/release.rs:138` — `handle`，129/100
  - `apps/cli/src/commands/label.rs:264` — `handle_milestone`，113/100
  - `apps/cli/src/commands/label.rs:135` — `handle_label`，101/100
- 证据强度：Measured
- 置信度：Low
- **实测值 / 阈值**：239 / 172 / 129 / 113 / 101 行，阈值 100 行（`clippy::too_many_lines` 工具默认）。同批函数的 `clippy::cognitive_complexity` 在阈值 25 下**零命中**
- **去重规则**：**规则 2 — 模式合并**。**同一模式的 5 个独立实例**，每处需各自改动：修好 `issue.rs::handle` 不会让 `pr.rs::handle` 变短。整改成本为 5 处，而非 1 处
- **根因**：每个命令模块的入口函数把该命令**全部子命令的函数体内联在一个 `match` 里**。这是一个设计决策（分发与子命令实现不分层），在 5 个模块中被独立重复了 5 次
- **影响**：新增一个子命令时函数持续增长；单个 `match` 臂的改动与其余十余个臂共享同一函数的 diff 范围。已实测阅读了 `pr.rs:214` 的完整函数体：15 个 `match` 臂中 11 个是「调用 provider → 包装 `CliOutput` → `print_output`」的薄壳，另外 4 个（`Create` / `List` / `Merge` / `Cleanup`）在臂内内联了参数解析与分支逻辑
- **为什么是 Low 而不是更高**：`cognitive_complexity` 零命中是一个真实的反向信号——这些函数长但**不难读**，扁平分发表的阅读成本与行数不成正比。证据强度 Measured 不等于置信度高，本条正是该解耦的典型样本
- **关于抑制的判定（SKILL.md 三分支规则第 3 支）**：5 处**全部**带 `#[allow(clippy::too_many_lines, reason = "Command dispatch: each match arm maps to one operation")]`。按 SKILL.md「有陈述的理由只是一个待核验的主张」，已逐条打开代码核验该主张：理由准确描述了分发骨架，但 `pr.rs::handle` 的 15 个臂中有 4 个（`Create`/`List`/`Merge`/`Cleanup`）内联了参数解析与三路分支，不在「each match arm maps to one operation」的覆盖范围内。这命中第 3 支——**理由被代码部分证伪**，本身即构成 finding，故不移入已排除。本条报告的正是「主张与代码的偏差」：抑制理由需要更新或代码需要回到理由所描述的形态，二选一。另有 3 处同款抑制（`auth.rs:49`、`review.rs:127`、`commit.rs:76`）本次未越界，不产生候选
- **建议（仅建议，不执行）**：若未来决定整改，可把每个 `match` 臂抽成 `async fn` 子处理函数，使 `handle` 退化为纯分发。这是一个一次性的、跨 5 个模块的架构决定，不是 5 个独立缺陷，不建议逐个零散处理

### SM-002 · Deep Nesting · Medium

- **位置**：`crates/core/src/toon.rs:74-99`（函数 `analyze`，声明于 `:65`）
- 证据强度：Observed
- 置信度：High
- **实测值 / 阈值**：最大缩进 36 列（9 层 × 4 空格），出现在 `:90`；单个表达式内嵌套 `all(...)` → `is_some_and(...)` → 块表达式 → `is_some_and(...)` → `map(...)` 共 5 层闭包。本仓库其余文件最大缩进为 27 列
- **为什么证据强度是 Observed 而非 Measured**：`clippy::excessive_nesting` 在本仓库永远不产出信号（阈值默认 0 即关闭，启用需改 `clippy.toml`，本次扫描不动策略配置）。替代信号来自花括号/缩进结构扫描，而花括号计数会被 format string 的 `{}` 与 raw string 欺骗，**因此该位置是逐行打开文件人工确认过的**，不是扫描直接输出的清单项
- **根因**：`is_uniform_array` 的判定（「数组各元素的键集合是否与首元素一致」）被写成一个单一表达式，而不是先取出首元素键集合再逐元素比较
- **影响**：读者必须在一个表达式里同时跟踪 `arr`、`item`、`obj`、`first`、`o` 五个绑定，其中 `arr.first()` 在外层 `all()` 闭包**内部**被重新求值。这是本仓库唯一一处结构上明显偏离其余代码风格的嵌套
- **建议（仅建议，不执行）**：把首元素键集合提升为 `analyze` 的局部变量，闭包内只做一次比较；嵌套即可降到 2 层

### SM-003 · God Structure · Medium

- **位置**：`apps/cli/src/commands/skills.rs:1-928`（`#[cfg(test)]` 之前的生产代码部分）
- 证据强度：Observed
- 置信度：Medium
- **实测值 / 阈值**：生产代码 928 行 / 阈值 800 行；生产函数 25 个 / 阈值 40 个（函数数未越界）
- **根因**：单个文件同时承担 6 类职责——CLI 参数 schema（`SkillsCommand` / `InstallArgs` / `ListArgs` / `UninstallArgs` / `SkillsUpdateArgs`）、Agent 平台探测（`AgentPlatform::detect`）、skill 来源探测（`detect_skill_sources` / `plugin_source_present` / `bare_sentinels_present`）、install/uninstall/list/update 编排、通用递归文件复制工具（`copy_dir_all`）、以及被其他命令模块共用的 TTY 确认提示（`pub(crate) fn confirm`）
- **影响**：`copy_dir_all` 与 `confirm` 与 skill 管理语义无关，却只能从本文件引入；`pub(crate) fn confirm` 已构成跨模块依赖，使该文件成为 `apps/cli/src/commands` 中的隐式工具模块
- **为什么置信度不是 High**：越界幅度为 16%（928/800），且阈值「文件 800 行」来自 `references/rust.md` 自定义而非工具默认；职责划分的判断含主观成分
- **本条是 15 个文件行数候选中唯一存活的一条**：其余 14 个在 Stage 2 全部因「测试代码占多数」被排除，详见已排除表（逐个列出，可核）
- **建议（仅建议，不执行）**：把 `copy_dir_all` 与 `confirm` 迁到独立的工具模块，可同时消除跨模块的隐式依赖

## 待测量候选（不计入严重度统计）

### SC-001 · Performance（`analyze` 的首元素键集合重复求值）

- **位置**：`crates/core/src/toon.rs:86-94`
- **需要测量什么**：`arr.first()` 的键集合在外层 `all()` 闭包内被每个元素重复求值一次，复杂度由 O(n+k) 变为 O(n·k log k)（含每次 `sort_unstable`）。这是否构成真实开销，取决于实际流经 TOON 格式化的 JSON 数组元素数 n 与对象键数 k
- **为什么不进严重度表**：本次扫描未测量任何运行时数据。`gf` 的 TOON 输出用于 CLI 结果渲染，数组规模可能只有个位数，也可能是 `pr list --limit` 的上百条；未测量前把它称作性能坏味道就是把「我猜它慢」包装成「它慢」
- **如何测量**：对 `analyze` 加 Criterion 基准，以 n ∈ {10, 100, 1000} × k ∈ {5, 20} 的合成数组取样；同时统计生产路径上实际传入 `analyze` 的数组长度分布

### SC-002 · Shotgun Surgery（三套适配器的实际改动联动率）

- **位置**：`crates/github/` · `crates/gitlab/` · `crates/gitcode/`
- **需要测量什么**：一次语义变更实际需要同步改动几个适配器 crate。`docs/architecture-review-2026-08-28.md` 的 R3 已把该风险记录为 MEDIUM / HIGH 并给出 P3 整改项，但**其判断依据是结构相似度，不是实测的改动联动率**
- **为什么不进严重度表**：Duplicated Logic 本身已作为「有意权衡」移入已排除（见下表）；本条追问的是一个不同的、尚未被测量的量——R3 的 P3 整改是否值得投入，取决于同时触及 ≥2 个适配器 crate 的提交占比，而该比例本次未统计
- **如何测量**：`git log --name-only` 统计近 N 次提交中同时改动 ≥2 个 `crates/{github,gitlab,gitcode}` 的比例；若显著低于预期，R3 的 P3 可降级

## 已排除

下表分两类：**A 类**为 What NOT to Flag 命中；**B 类**为 Stage 2 判定为检测产物（工具口径导致的误报），不属于排除项但同样必须公开，否则读者无法核对漏报。

| 类 | 位置 | 命中排除项 | 理由 |
|---|---|---|---|
| A | 文件行数越界 15 个中的 **14 个**（全列，见下方「已排除文件清单 A-1」） | Test fixtures | 逐个定位 `#[cfg(test)]` 起始行后，生产代码部分**全部低于 800 行阈值**；文件行数越界完全由测试模块贡献。唯一例外 `apps/cli/src/commands/skills.rs`（生产 928 行）未排除，见 SM-003 |
| A | 单文件函数数越界的 **全部 16 个**（全列，见下方「已排除文件清单 A-2」） | Test fixtures | 拆分后生产函数数为 6~25 个，**全部低于 40 个阈值**；例如 `github/src/issue.rs` 的 82 个中有 61 个是 `fn test_should_*` |
| A | `crates/github` / `crates/gitlab` / `crates/gitcode` 三套适配器（`github/src/commit.rs` 484 行 vs `gitcode/src/commit.rs` 482 行，`diff` 仅 138 行，约 86% 相同） | 有意权衡 | `docs/architecture.md:73` 明确记载「Structurally identical to the GitHub adapter but handles GitCode-specific JSON field formats」；`docs/architecture-review-2026-08-28.md` 的 R3「Adapter Code Duplication」已把它列为 MEDIUM / HIGH 风险并给出整改建议 P3。重复本身是「Provider Trait + CLI Adapter」模式的已知代价，已有文档记录 |
| A | 全部 22 处默认 target 的 `dead_code`（`github`/`gitlab`/`gitcode` 的 `commit.rs`、`pipeline.rs`、`label.rs`，`core/src/compatibility.rs`、`session.rs`、`types.rs`，`e2e-core/src/tty.rs`，`apps/cli/src/commands/prerequisites.rs`、`skills.rs:887`、`main.rs:35`） | 有意权衡 | 逐条核对全部 38 处涉及本次 6 个 lint 的 `#[allow(...)]`：**每一处都带 `reason = "..."`**，本仓库零裸抑制。理由集中为两类——serde 反序列化结构体的字段（「Fields deserialized by serde but not all read directly」）与显式声明的预留项（「Kept for future GitCode pipeline support」等）。按 SKILL.md 三分支规则逐条**打开代码核验主张**：结构体确由 `#[derive(Deserialize)]` 消费、预留项确未被调用，主张与代码一致，命中第 2 支（有理由且已核验）→ 有意权衡 |
| A | `apps/cli/tests/workflow_modes_test.rs:98/117/135/152` · `crates/e2e-core/src/scratch.rs:73` · `crates/github/src/pipeline.rs:764` | Test fixtures | 测试专用抑制（`clippy::panic` / `clippy::expect_used` / 测试内联 fixture 结构体），均带 reason |
| A | `apps/cli/src/main.rs:38-40` 的 3 处无 `reason=` 的 `#[allow(clippy::...)]` | 有意权衡 | 虽无 `reason=` 属性，但紧邻的文档注释（`main.rs:28-33`）逐条说明了原因（`built` crate 生成代码的 raw string / 无理由 allow / doc_markdown）。SKILL.md 的判据是「有陈述的理由」，注释与属性等效 |
| B | 54 个只出现在 `--all-targets` 捕获中的 `dead_code` 位置（`apps/cli/src/commands/*` 的 10 处 `handle`、`main`、`async_main`、`router` 等，以及 `crates/release-signer/src/main.rs` 4 处） | 检测产物 | `--all-targets` 把二进制 crate 再编译为 test harness，此时 `main` 及其整条调用链不可达。**实测**：`--all-targets` 捕获 79 条 / 76 个位置，默认 target 捕获 22 条 / 22 个位置，差集 54 个位置全部来自被重编的二进制 crate。已另行阅读源码交叉验证，例如 `apps/cli/src/main.rs:178` 调用 `commands::pr::handle(...)`。按修订后的 `references/rust.md`，这 54 个位置已**不再进入候选表** |
| B | `clippy::excessive_nesting` 零命中 | 检测产物 | 该 lint 阈值默认为 0（关闭），本仓库 `clippy.toml` 未配置 `excessive-nesting-threshold`，`--force-warn` 对其无效。零命中**不是** Deep Nesting 干净的证据，故不得据此下任何结论；本报告改以结构扫描替代，见 SM-002 |

### 已排除文件清单 A-1：文件行数越界但生产代码未越界（14 个，全列）

| 文件 | 总行数 | 生产行数 | 是否越界（>800） |
|---|---|---|---|
| `crates/github/src/issue.rs` | 1786 | 737 | 否 |
| `crates/gitlab/src/issue.rs` | 1628 | 771 | 否 |
| `crates/gitlab/src/mr.rs` | 1449 | 683 | 否 |
| `crates/gitcode/src/pr.rs` | 1397 | 701 | 否 |
| `apps/cli/src/commands/workflow.rs` | 1319 | 653 | 否 |
| `crates/gitcode/src/issue.rs` | 1282 | 720 | 否 |
| `crates/github/src/pr.rs` | 1281 | 593 | 否 |
| `crates/github/src/pipeline.rs` | 1224 | 465 | 否 |
| `apps/cli/src/commands/pr.rs` | 1077 | 564 | 否 |
| `crates/gitlab/src/release.rs` | 1056 | 454 | 否 |
| `crates/gitlab/src/label.rs` | 1045 | 630 | 否 |
| `crates/github/src/release.rs` | 917 | 396 | 否 |
| `apps/cli/src/commands/issue.rs` | 865 | 425 | 否 |
| `crates/gitlab/src/pipeline.rs` | 821 | 421 | 否 |

第 15 个越界文件 `apps/cli/src/commands/skills.rs`（1476 / 生产 928）**未被排除**，
是 SM-003。

### 已排除文件清单 A-2：函数数越界但生产函数数未越界（16 个，全列）

| 文件 | 生产函数 | 测试函数 | 是否越界（>40） |
|---|---|---|---|
| `crates/github/src/issue.rs` | 21 | 61 | 否 |
| `crates/gitlab/src/issue.rs` | 21 | 52 | 否 |
| `apps/cli/src/commands/skills.rs` | 25 | 48 | 否 |
| `crates/gitlab/src/mr.rs` | 24 | 47 | 否 |
| `crates/github/src/pr.rs` | 18 | 45 | 否 |
| `crates/gitcode/src/pr.rs` | 19 | 44 | 否 |
| `crates/github/src/release.rs` | 11 | 44 | 否 |
| `apps/cli/src/commands/pr.rs` | 6 | 40 | 否 |
| `crates/gitcode/src/issue.rs` | 20 | 37 | 否 |
| `crates/github/src/pipeline.rs` | 13 | 37 | 否 |
| `apps/cli/src/commands/workflow.rs` | 17 | 36 | 否 |
| `crates/gitlab/src/release.rs` | 14 | 36 | 否 |
| `crates/github/src/auth.rs` | 12 | 32 | 否 |
| `crates/gitlab/src/auth.rs` | 11 | 31 | 否 |
| `crates/core/src/pr.rs` | 15 | 30 | 否 |
| `crates/gitlab/src/label.rs` | 22 | 24 | 否 |

`skills.rs` 在本表中生产函数数 25 未越界；它成为 SM-003 是因为**行数**而非函数数。

### 一条超出本 skill lint 集的裸抑制

`crates/github/src/release.rs:267` 的 `#[allow(clippy::disallowed_methods)]` 无 `reason=`、
无相邻说明注释，是全仓库唯一的裸抑制。该 lint 属项目策略配置（`clippy.toml` 的
`disallowed-methods`），不在本 skill 的 6 个坏味道 lint 之内，因此不产生 finding；
此处记录一行，供 `/gf-quality` 或代码审查跟进。

## 本次运行对 skill 自身的验证

- 阈值越界 ≠ finding：11 类候选进入 Stage 1，最终只有 3 条成为 finding。15 个文件行数候选存活 1 条，16 个函数数候选存活 0 条，22 条 `dead_code` 存活 0 条
- 证据强度与置信度确实需要解耦：SM-001 是 Measured + Low，SM-002 是 Observed + High，两条方向相反，合并成单一分数会同时误导
- `excessive_nesting` 的零命中如果被当成结论，会直接漏掉 SM-002，这是 `references/rust.md` 单独为该 lint 写一段的原因

## 报告修订记录

本报告初稿写于 `gf-smell` skill 修订之前。本次运行暴露了 skill 自身的两处缺陷，
两处均已修复，报告随之重跑并订正：

1. **`references/rust.md` 的 `dead_code` 检测口径**。原命令对 `dead_code` 也加了
   `--all-targets`，把二进制 crate 重编为 test harness，使只从 `main` 可达的代码被报成
   未使用。实测：`--all-targets` 79 条 / 76 个位置，默认 target **22 条 / 22 个位置**，
   差集 **54 个位置**全部是 harness 假阳性，放大约 3.6 倍。修订后 `dead_code` 改由
   独立的默认 target 命令捕获，`类目映射` 的 `Measured` 上限也加了「仅当来自默认 target」
   的限定。本报告据此重跑，`dead_code` 候选数由 79 改为 22。
   **结论未变**：初稿已把全部 79 条判入已排除，其中 50 条正是以「检测产物」为由排除的，
   因此**没有任何 finding 或待测量候选是 harness 假阳性**；改动只影响候选计数与口径叙述。

2. **`SKILL.md` 的抑制判定规则**。原规则只有两个分支（有理由→排除 / 裸抑制→候选），
   按字面执行会把本仓库 5 处 `handle` 的抑制整条排除。现改为三分支，并把「核验理由是否
   仍描述当前代码」写成强制步骤；新增的第 3 支「理由被代码证伪」本身即构成 finding。
   SM-001 的判定文字已改写为显式引用该分支。**结论未变**：初稿的处理与修订后的规则一致，
   区别在于初稿是逐案判断，现在是规则明文支持。
