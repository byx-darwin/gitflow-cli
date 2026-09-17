# gf-walkthrough —— 交付走查包与三级证据标注

- **Issue**: #329
- **Workflow**: `wf-2026-09-16-002`（full 模式）
- **日期**: 2026-09-16
- **状态**: 已批准，待生成实施计划

## 背景

本仓库在「证据表达」上存在空白。实测确认：

```
$ grep -ril 'evidence\|证据\|provenance' skills/gf-quality skills/gf-pr-review skills/gf-review
(无命中)
```

`gf-review` 落盘 `docs/code-review-report-pr<N>-*.md`，但只有审查结论，没有变更叙事、没有验证输出原文、没有面向非工程读者的入口。`gf-pr-review` 做六维评估（correctness / security / performance / maintainability / test-coverage / docs），产物同样是判决而非交付说明。

缺的是一类不同的产物：**交付走查包**——让没有读过 diff 的人（产品、QA、下一个接手的工程师）能离线读懂「改了什么、为什么、验证到什么程度」，且对「验证到什么程度」不含糊其辞。

## 目标

新增 `skills/gf-walkthrough/`，产出可离线阅读的交付走查包，强制证据分级。

**非目标**：本票不修改 `gf-workflow` 编排器，不接入其 Phase 4 并行调度集（理由见「范围边界」）。

## 边界

### 与既有 skill 的职责划分

| skill | 职责 | 产物 | 读者 |
|---|---|---|---|
| `gf-pr-review` | 六维评估 | 维度判定 | 评审人 |
| `gf-review` | 提交判决 | approve / request-changes | 平台 |
| **`gf-walkthrough`** | **叙事 + 证据** | **走查包** | **非工程读者** |

三者职责互斥。`gf-walkthrough` 不产出判决，不调用 `gf review`，不决定 PR 能否合并。

### 范围边界：本票不改 gf-workflow

`gf-workflow` 的 Phase 4 并行调度集由 `gates.md → get_phase4_steps(mode)` 决定。修改它属于另一个变更面，且会让本票的验收标准与编排器行为耦合。

本票只做到「skill 可被独立调用并产出合格走查包」，在 `## See Also` 中声明其在 Phase 4 的预期位置。接入编排器另开票。

这一条是对 Issue #328 关闭教训的直接应用：该票因「与 #327、#332 边界不清」被关，本票以显式收缩范围避免重蹈。

## 架构

### 输入锚点

主键是 `<base>...<head>` 的 diff 区间，任何分支或 commit 范围都能运行。

PR 信息（标题、描述、CI 状态、评论）存在时作为额外证据源接入，不存在也能产出完整走查包。

理由：`gf-workflow` Phase 3 有 `pr` 与 `local_merge` 两条交付路径，以 PR 为锚会让 `local_merge` 路径拿不到走查包。

### 语言无关性

SKILL.md 正文不含任何单一语言的工具名、文件名或 API 名。

语言探测复用 `gf-quality/references/detector.md`；补跑命令引用 `gf-quality/references/<lang>.md` 的 Gate Commands 表（五种语言齐备，rust.md 第 10 行即 `| 2 | test | cargo test --workspace --quiet | all pass |`）。

**本 skill 自身零 `references/<lang>.md`。** 这同时满足三点：#328 的教训（复用 detector.md 而非自建探测）、#349 的方向（不新增跨 skill 重复的语言画像）、skill token 预算约束。

### 证据来源：混合模式

优先汇编已有执行记录（契约、PR diff、CI 日志、`gf-quality` 报告）；某条验收标准无现成证据时，允许补跑一条只读命令。

**补跑白名单**：
- `git log` / `diff` / `merge-base` / `show`
- `gf pr view` / `gf pr checks`
- `gf-quality/references/<lang>.md` → Gate Commands 表中的只读项

**禁止**：任何写文件、推送、改分支状态、安装依赖的命令。

补跑失败时降级为 `Unverified` 并记录失败原因，**不得**改标为 `Inferred` 蒙混。

同时记录让命令跑起来的 workaround（如 PATH 隔离、环境变量），否则读者复现不了。

## 走查包结构：四节契约

每节都有非空产出义务。没有内容时必须写明「为何为空」，而不是删掉这一节。

| 节 | 产出义务 | 为空时 |
|---|---|---|
| ① 开场叙事 | 3–8 句，说清「谁受影响、发生了什么变化、为什么现在做」。**首句禁止以标识符、函数名、文件路径、命令名开头**；术语随文解释，不设独立词表 | 不允许为空 |
| ② 变更摘要 | `git diff --stat` 汇总行 + 按模块分组的「改了什么 · 为什么」 | 不允许为空 |
| ③ 验证证据 | 每条验收标准一行，带三级标记；`Measured` 条目必须紧随命令原文与输出片段的代码块 | 不允许为空 |
| ④ 评审门禁 | blast radius + 部署顺序 + merge checklist | 涉及迁移/开关时必填；否则写明「本次变更不涉及迁移与开关，blast radius 限于 `<路径>`」 |

「可视化证明」不设独立节——有图时作为 ③ 中某条 `Measured` 证据的附图，没有就不占位。

**设计理由（为何不照搬对照源的六节）**：`smallnest/goal-workflow` 的 `skills/walkthrough/SKILL.md` 用六节（开场 / 术语表 / 变更摘要 / 验证证据 / 可视化证明 / 评审门禁）。其中「可视化证明」对 CLI 仓库的多数 PR 无物可放，「术语表」与变更摘要重复度高。空节会训练出填占位符的习惯，正好违背该源自述的原则——"A walkthrough that overstates its evidence is worse than one with gaps"。

## 三级证据标注

### 标记语法

标记写在证据行行首，方括号包裹，机器可 grep：

```markdown
- [Measured] 新增 skill 通过结构校验
  ```
  $ make check-walkthrough-skill
  ✓ AC#1 开场段落未以标识符开头
  ```
- [Inferred] 不影响 CLI 二进制体积 —— build.rs 仅扫描 skills/ 生成清单，未改变嵌入逻辑（apps/cli/build.rs:75）
- [Unverified] 非工程读者可读性 —— 未经真实非工程读者试读
```

### 分级判定规则

- **`Measured`** —— 本次会话内真实执行过命令，且**输出原文已附**。只有结论没有输出块 ⇒ 降级为 `Inferred`。
- **`Inferred`** —— 从代码、配置、diff 读出的结论，未执行验证。必须写明推断依据的 `path:line`。
- **`Unverified`** —— 该条验收标准本次未能验证。**必须写明为何未验证**，不得省略此条目来让报告显得完整。

### 词汇复用

沿用 `gf-smell` 已确立的英文档位。`skills/gf-smell/SKILL.md:230` 定义了 `证据强度：Measured | Observed | Inferred`，本 skill 的 `Measured` / `Inferred` 与其逐字一致，`Unverified` 对应其「待测量候选（不计入严重度统计）」。

`Observed` 在本 skill 不使用——走查包不做静态观察。

理由：再造一套近义中文档位会让两份报告无法互读，读者需要学两套词汇。

## 失败测试溯源

存在失败测试时，③ 节必须给出三列表。**禁止写 "unrelated" / "与本次无关" 而不给 commit**。

| 失败用例 | 最后修改 commit | 是否 base 祖先 |
|---|---|---|
| `test_should_reject_invalid_path` | `93db94d` (2026-08-10) | ✅ 是 → 先于本次交付存在 |

算法（已在本仓库实测通过）：

```bash
H=$(git log -1 --format=%H -- "<测试文件>")
git merge-base --is-ancestor "$H" "$BASE_BRANCH" && echo "先于本次交付存在" || echo "本次引入"
```

判定列措辞固定为「先于本次交付存在」而非「无关」：`--is-ancestor` 只证明**该测试文件**的最后修改早于 base，不等于失败与本次变更无关（可能是本次改动触发了既有测试的潜在缺陷）。

溯源不到测试文件时（测试名与文件名不对应），该行标 `Unverified` 并说明原因。

## 强制机制

双层：SKILL.md 自检清单（管全部 6 条 AC）+ Makefile 校验器（只管机器能判真假的硬约束）。

### `make check-walkthrough-skill` 硬约束清单

与 `check-smell-skill` 同构（同样的 `FAIL=0` / 逐条 `✓`/`✗` / 末尾 `exit $FAIL` 骨架）。校验对象是 `skills/gf-walkthrough/SKILL.md`，路径硬编码，**不读 `~/.claude/skills/` 下的副本**。

| # | 校验项 | 判据 | 对应 AC |
|---|---|---|---|
| 1 | SKILL.md 存在且无语言专属标识 | 正文 grep 不到 `cargo` / `go.mod` / `package.json` / `pytest` / `mvn` 等 | 语言无关性 |
| 2 | 三档标记定义齐备 | 含 `Measured`、`Inferred`、`Unverified` 三个字面量 | AC#1 |
| 3 | `Measured` 必带输出块 | 扫描落盘报告：每个 `[Measured]` 行之后 3 行内须出现代码围栏 | AC#2 |
| 4 | 失败测试三列表结构 | 含表头字面量「失败用例」「最后修改 commit」「是否 base 祖先」 | AC#3 |
| 5 | 禁用词声明 | 含禁止 `unrelated` 的明文规则 | AC#3 |
| 6 | 开场首句约束 | 报告 ① 节首句非空，且首字符不是反引号或斜杠、首词不是全大写或含 `_` / `::` 的标识符 | AC#4 |
| 7 | blast radius 章节 | 报告含 `blast radius` 字面量 | AC#5 |
| 8 | 报告落盘位置 | `docs/walkthrough-*.md` 存在且非空；仓库内无散落在 `docs/` 之外的同名文件 | 落盘策略 |
| 9 | `allowed-tools` 边界 | 声明存在；含 `Write`（落盘报告所需），不含 `Edit`（禁止改动被走查的源码） | 补跑边界 |
| 10 | 复用语言探测 | 引用 `gf-quality/references/detector.md` | #328 教训 |
| 11 | 词数硬限 | SKILL.md ≤ 500 词（排除代码块、frontmatter、行内代码） | token 预算 |
| 12 | 报告模板存在 | `docs/superpowers/templates/walkthrough-report-template.md` 存在 | 外置去处 |

**计数命令缺陷（仓库既有问题，不在本票范围）**：`skill-conventions.md:21` 给出的统计命令因 `scalar(/.../g)` 在标量上下文返回布尔值而恒输出 `1`，致使 500 词硬限从未真正执行——实测 `gf-smell` 1878 词、`gf-review` 716 词、`gf-quality` 678 词、`gf-pr-review` 640 词，无一满足。第 11 项使用修正写法 `scalar(()=/\p{L}+/g)`。**本票不修改该文档**，建议单开票。

**前置条件**：第 3、6、7 项扫描的是落盘报告，第 8 项确认报告存在。RED 阶段报告尚未产出，此时第 3、6、7 项输出 `— 跳过（无报告）` 而非 `✗`，只有第 8 项计入失败。否则 RED→GREEN 的转绿路径会被伪失败淹没，看不出真实进度。

`allowed-tools` 取 `Read, Grep, Glob, Bash, Write`。`Write` 是落盘走查包所必需；`Edit` 被排除，因为走查包只描述变更、从不改动被走查的代码——这与 `gf-smell` 同时禁用 `Write` 和 `Edit` 的取舍不同，后者连报告都由调用方落盘。

第 6 项是唯一带启发式的判据。判不准时**放行并提示人工确认**，宁可漏报不误杀——校验器的作用是挡住明显违规，不是替代人的判断。

## 落盘与归档

命名对齐既有先例（`docs/code-review-report-pr317-2026-09-04.md`、`docs/smell-report-*.md`）：

```
docs/walkthrough-pr<N>-<YYYY-MM-DD>.md          # delivery_mode = pr
docs/walkthrough-<branch-slug>-<YYYY-MM-DD>.md  # delivery_mode = local_merge
```

归档沿用 `gf-review` 的既定规则：`docs/walkthrough-*.md` 超过 5 份时，按文件名内嵌日期把最旧的移入 `docs/reports-archive/<YYYY>-Q<N>/`，并更新 `docs/index.md`。

## 文件清单

写入白名单是封闭的。`.claude/` 与 `~/.claude/` 不在清单内。

```
skills/gf-walkthrough/SKILL.md                              新增，语言无关，零 references/<lang>.md
docs/superpowers/templates/walkthrough-report-template.md   新增，报告骨架 + 10 项自检清单
Makefile                                                    新增 check-walkthrough-skill 目标 + .PHONY 登记
docs/walkthrough-pr<N>-<date>.md                            AC#6 的实证产物
docs/index.md                                               索引更新
```

**为何有第 2 条**：SKILL.md 受 500 词硬限，四节契约 + 三级规则 + 溯源算法 + 自检清单 + 标准章节无法全部塞入。`skill-conventions.md` §1.3 的外置优先级表将「Compliance checklists / audit templates」指向 `docs/superpowers/templates/`，故报告骨架与自检清单外置，SKILL.md 只留单行链接。

这与「零 `references/`」不冲突：该约束针对**语言分层**（`references/<lang>.md`），模板外置是 token 预算的既定手段。

## 验证策略

### TDD 循环

校验器即测试：

1. **RED** —— 先写 `check-walkthrough-skill` 目标。此时 `skills/gf-walkthrough/SKILL.md` 不存在，10 项全 ✗。
2. **GREEN** —— 写 SKILL.md，让校验逐项转绿。
3. **REFACTOR** —— 精简措辞、压缩到 token 预算内，确认仍全绿。

### 验证范围与豁免

本变更不修改 Rust 源码，但 `apps/cli/build.rs:75` 有 `cargo:rerun-if-changed=skills/`——新增 skill 目录会重新生成内嵌清单。因此：

| 命令 | 是否运行 | 理由 |
|---|---|---|
| `cargo build` | ✅ | 确认 skills 清单重新生成无误 |
| `make check-walkthrough-skill` | ✅ | 本票的核心验证 |
| `make check-agent-sync` | ✅ | skill 变更的既定要求 |
| `cargo clippy --pedantic` | ❌ 豁免 | 无 Rust 源码改动 |
| `cargo audit` / `cargo deny` | ❌ 豁免 | 无依赖、许可证、供应链变更 |

### AC#6 的验证路径

「在一个真实 PR 上产出走查包」的验证**不依赖 skill 安装**。

实测确认 `~/.claude/skills/` 是 `make install-skills` 从 `skills/*` 复制的副本（非软链），且 `gf-smell` 至今未安装——即新写的 SKILL.md 在安装前对 Claude Code 不可见。

因此验证路径是：写完 SKILL.md → **按其协议手工执行一遍**产出走查包 → 用校验器验收。安装与否是用户侧动作，不进本票验收范围。

## 验收标准映射

| Issue #329 AC | 本设计对应章节 |
|---|---|
| AC#1 每条结论带三级标注 | 三级证据标注 · 标记语法；校验器 #2 |
| AC#2 `Measured` 必附命令原文与输出 | 分级判定规则；校验器 #3 |
| AC#3 失败测试三列溯源，禁写 unrelated | 失败测试溯源；校验器 #4、#5 |
| AC#4 开场不以标识符开头 | 四节契约 ①；校验器 #6 |
| AC#5 含 blast radius | 四节契约 ④；校验器 #7 |
| AC#6 真实 PR 上产出，非工程读者可读 | 验证策略 · AC#6 的验证路径 |

## 关联

- Issue #329（本票）
- Issue #327 `gf-smell` —— 证据档位词汇来源（`Measured` / `Inferred`）
- Issue #349 语言画像单源化 —— 本票零 `references/<lang>.md` 与其方向一致
- Issue #328（已关）—— 其「边界不清」的关闭教训直接塑造了本票的范围收缩
