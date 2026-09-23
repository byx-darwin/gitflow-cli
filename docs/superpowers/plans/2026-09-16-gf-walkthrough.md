# gf-walkthrough Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 新增 `skills/gf-walkthrough/`，产出可离线阅读的交付走查包，对每条验证结论强制三级证据标注。

**Architecture:** 纯文档变更，零 Rust 源码改动。SKILL.md 承载语言无关的执行协议；报告模板与自检清单外置到 `docs/superpowers/templates/`（token 预算所迫）；`make check-walkthrough-skill` 作为可执行的验收测试，扮演 TDD 中的测试角色。

**Tech Stack:** Markdown · GNU Make · POSIX shell · git plumbing（`git log` / `merge-base`）

**Spec:** `docs/superpowers/specs/2026-09-16-gf-walkthrough-design.md`

**Issue:** #329 · **Workflow:** `wf-2026-09-16-002`（full 模式）

## Global Constraints

以下约束对每个 Task 无条件生效。

- **写入白名单封闭为 5 条路径**，越界即为违规：
  1. `skills/gf-walkthrough/SKILL.md`
  2. `docs/superpowers/templates/walkthrough-report-template.md`
  3. `Makefile`
  4. `docs/walkthrough-pr<N>-<YYYY-MM-DD>.md`
  5. `docs/index.md`
- **`.claude/` 与 `~/.claude/` 严禁读写。** 不得运行 `make install-skills`（属部署操作，需用户显式许可）。
- **SKILL.md 词数硬上限 500**（排除代码块、frontmatter、行内代码）。计数用下方修正过的命令，**不得**使用 `skill-conventions.md:21` 的原样命令——该命令因 `scalar(/.../g)` 在标量上下文返回布尔值而恒输出 `1`。
- **SKILL.md 正文不得出现任何单一语言的工具名或文件名**（`cargo`、`go.mod`、`package.json`、`pytest`、`mvn` 等）。语言探测引用 `gf-quality/references/detector.md`。
- **本 skill 自身零 `references/<lang>.md`。**
- **本票不修改 `gf-workflow` 编排器**，不接入其 Phase 4 调度集。
- **SKILL.md 叙述正文用英文**，`description` 与触发词表保留中英双语（CLAUDE.md § Skill Documentation）。
- 不得运行 `cargo clean`。不得在未获许可时 commit、push、merge。

**词数计数命令（本计划唯一认可的写法）：**

```bash
perl -0 -ne 's/^---\n.*?^---\n//ms; s/```.*?```//gs; s/`[^`]+`//g; print scalar(()=/\p{L}+/g), "\n"' skills/gf-walkthrough/SKILL.md
```

---

## File Structure

| 文件 | 职责 | 为何独立 |
|---|---|---|
| `skills/gf-walkthrough/SKILL.md` | 执行协议：何时用、四节契约、三级判定规则、溯源算法、边界 | Claude 全量载入，受 500 词硬限 |
| `docs/superpowers/templates/walkthrough-report-template.md` | 报告骨架 + 10 项自检清单 + 填写示例 | `skill-conventions.md` §1.3 指定的外置去处；内容随报告格式演进，与协议解耦 |
| `Makefile` 新增目标 | 可执行验收测试 | 与 `check-smell-skill` 同构，集中于既有构建入口 |
| `docs/walkthrough-pr<N>-<date>.md` | AC#6 的实证产物 | 交付物本身，非源码 |
| `docs/index.md` | 索引 | 既有约定 |

---

## Task 1: 校验器（RED）

先写测试。此时 `skills/gf-walkthrough/` 尚不存在，校验器必须报失败。

**Files:**
- Modify: `Makefile`（在 `check-smell-skill` 目标之后新增，并在 `.PHONY` 行登记）

**Interfaces:**
- Produces: `make check-walkthrough-skill` —— 退出码 0 表示全部硬约束通过，非 0 表示存在 `✗` 项。Task 2、3、4 均以它为验收闸门。

- [ ] **Step 1: 在 `Makefile` 中新增校验目标**

在 `check-smell-skill` 目标块结束后追加。注意 Makefile 配方中 `$` 需写作 `$$`，反引号命令替换用 `` `...` ``：

```makefile
check-walkthrough-skill: ## Verify gf-walkthrough skill meets Issue #329 acceptance criteria
	@S=skills/gf-walkthrough/SKILL.md; \
	T=docs/superpowers/templates/walkthrough-report-template.md; FAIL=0; \
	if [ ! -f "$$S" ]; then echo "✗ missing $$S"; exit 1; fi; \
	if grep -nEi 'cargo|go\.mod|package\.json|pyproject\.toml|pom\.xml|pytest|eslint|mvn|clippy' "$$S"; then \
		echo "✗ #1 SKILL.md 正文含单一语言的工具名或文件名"; FAIL=1; \
	else echo "✓ #1 正文无语言专属标识"; fi; \
	TIER=0; \
	for K in Measured Inferred Unverified; do \
		grep -qF "$$K" "$$S" || { echo "✗ #2 缺少证据档位 $$K"; TIER=1; FAIL=1; }; \
	done; \
	[ $$TIER -eq 0 ] && echo "✓ #2 三档标记齐备"; \
	for H in '失败用例' '最后修改 commit' '是否 base 祖先'; do \
		grep -qF "$$H" "$$S" || { echo "✗ #4 失败测试表缺少列「$$H」"; FAIL=1; }; \
	done; \
	grep -qF 'unrelated' "$$S" \
		&& echo "✓ #5 禁用词规则已声明" \
		|| { echo "✗ #5 未声明禁止写 unrelated"; FAIL=1; }; \
	A=`grep -m1 '^allowed-tools:' "$$S"`; \
	if [ -z "$$A" ]; then echo "✗ #9 缺少 allowed-tools"; FAIL=1; \
	elif echo "$$A" | grep -qE 'Edit'; then echo "✗ #9 allowed-tools 含 Edit"; FAIL=1; \
	elif ! echo "$$A" | grep -qE 'Write'; then echo "✗ #9 allowed-tools 缺 Write（落盘所需）"; FAIL=1; \
	else echo "✓ #9 工具集含 Write 不含 Edit"; fi; \
	grep -qF 'gf-quality/references/detector.md' "$$S" \
		&& echo "✓ #10 复用既有语言探测" \
		|| { echo "✗ #10 未引用 detector.md"; FAIL=1; }; \
	W=`perl -0 -ne 's/^---\n.*?^---\n//ms; s/\x60\x60\x60.*?\x60\x60\x60//gs; s/\x60[^\x60]+\x60//g; print scalar(()=/\p{L}+/g)' "$$S"`; \
	if [ "$$W" -le 500 ]; then echo "✓ #11 词数 $$W ≤ 500"; \
	else echo "✗ #11 词数 $$W 超出 500 硬限"; FAIL=1; fi; \
	if [ -f "$$T" ]; then echo "✓ #12 报告模板存在"; \
	else echo "✗ #12 缺少 $$T"; FAIL=1; fi; \
	REP=`ls docs/walkthrough-*.md 2>/dev/null | head -1`; \
	if [ -n "$$REP" ] && [ -s "$$REP" ]; then echo "✓ #8 走查包已落盘: $$REP"; \
	else echo "✗ #8 未找到非空的 docs/walkthrough-*.md"; FAIL=1; REP=""; fi; \
	STRAY=`find . -name 'walkthrough-*.md' -not -path './docs/*' -not -path './target/*' \
		-not -path './.worktree/*' -not -path './.git/*' 2>/dev/null | head -5`; \
	if [ -n "$$STRAY" ]; then echo "✗ #8 报告散落在 docs/ 之外: $$STRAY"; FAIL=1; fi; \
	if [ -z "$$REP" ]; then \
		echo "— #3 跳过（无报告）"; echo "— #6 跳过（无报告）"; echo "— #7 跳过（无报告）"; \
	else \
		MCNT=`grep -c '^- \[Measured\]' "$$REP"`; \
		FCNT=`awk '/^- \[Measured\]/{n=NR; f=0; for(i=1;i<=3;i++){if((getline line)>0){buf[i]=line; if(line ~ /^ *\x60\x60\x60/) f=1}}; if(!f) c++} END{print c+0}' "$$REP"`; \
		if [ "$$MCNT" -eq 0 ]; then echo "✗ #3 报告中无 [Measured] 条目"; FAIL=1; \
		elif [ "$$FCNT" -eq 0 ]; then echo "✓ #3 全部 $$MCNT 条 [Measured] 均附输出块"; \
		else echo "✗ #3 有 $$FCNT 条 [Measured] 未附输出块"; FAIL=1; fi; \
		OPEN=`awk '/^## /{if(seen){exit}; seen=1; next} seen && NF {print; exit}' "$$REP"`; \
		case "$$OPEN" in \
			'`'*|/*|\#*) echo "✗ #6 开场首句以标识符或路径开头: $$OPEN"; FAIL=1;; \
			*) echo "✓ #6 开场首句未以标识符或路径开头";; \
		esac; \
		grep -qiF 'blast radius' "$$REP" \
			&& echo "✓ #7 含 blast radius 说明" \
			|| { echo "✗ #7 报告缺少 blast radius 章节"; FAIL=1; }; \
	fi; \
	[ $$FAIL -eq 0 ] && echo "全部硬约束通过" || echo "存在未通过项"; \
	exit $$FAIL
```

- [ ] **Step 2: 在 `.PHONY` 行登记目标**

编辑 `Makefile` 第 293 行，在 `check-smell-skill` 之后插入 `check-walkthrough-skill`：

```makefile
        update-submodule check-agent-sync check-smell-skill check-walkthrough-skill release release-quick release-rehearse \
```

- [ ] **Step 3: 运行校验器，确认 RED**

Run: `make check-walkthrough-skill; echo "exit=$?"`
Expected: 输出 `✗ missing skills/gf-walkthrough/SKILL.md`，`exit=1`。

这是 TDD 的 RED 状态——测试存在且失败，原因是被测对象尚未实现。

- [ ] **Step 4: 暂不提交**

本仓库规定未获用户许可不得 commit。将本 Task 的改动留在工作区，由 gf-workflow Phase 3 的 Worktree Preflight 统一征求许可后提交。

---

## Task 2: SKILL.md 主体（GREEN）

**Files:**
- Create: `skills/gf-walkthrough/SKILL.md`

**Interfaces:**
- Consumes: Task 1 的 `make check-walkthrough-skill`
- Produces: SKILL.md，含三档字面量 `Measured` / `Inferred` / `Unverified`、失败测试三列表头、`unrelated` 禁用声明、`allowed-tools: Read, Grep, Glob, Bash, Write`、`gf-quality/references/detector.md` 引用

- [ ] **Step 1: 建目录并写 frontmatter 与开篇**

与 `gf-smell/SKILL.md:1-14` 同构：

```markdown
---
name: gf-walkthrough
description: |
  Use when the user wants a delivery walkthrough, a change narrative for
  non-engineers, or evidence-graded verification of what a change was tested to.
  当用户需要交付走查包、面向非工程读者的变更说明，或需要对验证程度分级标注时使用。
allowed-tools: Read, Grep, Glob, Bash, Write
---

# gf-walkthrough — Delivery Walkthrough & Evidence Grading

Produces an offline-readable delivery package: narrative, change summary,
graded evidence, review gate. Anchored on a `<base>...<head>` diff range;
PR data enriches when present. **Never issues a verdict** — that is `gf-review`.
```

- [ ] **Step 2: 写核心章节**

必须包含以下字面量，否则校验器第 2、4、5、10 项不过：

```markdown
## Evidence Grading

| Tier | Meaning | Hard rule |
|---|---|---|
| `Measured` | Command actually run this session | Output block MUST follow. Conclusion without output ⇒ downgrade to `Inferred`. |
| `Inferred` | Read from code, config, or diff | MUST cite `path:line`. |
| `Unverified` | Not verified this run | MUST state why. Never omit the row to look complete. |

Tiers `Measured` / `Inferred` are reused verbatim from `gf-smell` so both
reports read with one vocabulary. Re-running a command that fails ⇒
`Unverified` with the failure reason; relabelling it `Inferred` is forbidden.

## Failing Tests

| 失败用例 | 最后修改 commit | 是否 base 祖先 |
|---|---|---|

Writing `unrelated` without a commit hash is forbidden. Verdict wording is
fixed to "先于本次交付存在", never "无关" — ancestry proves the test file
predates the base, not that the failure is unrelated.

## Language Detection

Reuse `gf-quality/references/detector.md`. This skill ships no language layer.

## Re-run Allowlist

Evidence is assembled from existing records first. A gap MAY be filled by
re-running one **read-only** command:

| Allowed | Forbidden |
|---|---|
| `git log` / `diff` / `merge-base` / `show` | anything writing files |
| `gf pr view` / `gf pr checks` | anything pushing or changing branch state |
| read-only Gate Commands from the language layer | anything installing dependencies |

A re-run that fails ⇒ `Unverified` with the reason. Record any workaround that
made a command run, or the reader cannot reproduce it.
```

溯源算法写成代码块（不计入词数）：

```bash
H=$(git log -1 --format=%H -- "<test file>")
git merge-base --is-ancestor "$H" "$BASE" && echo "先于本次交付存在" || echo "本次引入"
```

- [ ] **Step 3: 补齐标准章节**

按 `skill-conventions.md` 要求补 `## When to Use`、`## When NOT to Use`、`## Responsibility`、`## Red Flags`、`## Rationalization Excuses`、`## Test Scenarios`、`## Success Criteria`、`## Trigger Keywords`、`## See Also`。

`## When NOT to Use` 必须含三行边界：

| Scenario | Why Not | Use Instead |
|---|---|---|
| Submitting an approve / request-changes verdict | This skill never issues verdicts | `/gf-review` |
| Six-dimension PR assessment | This skill narrates, does not assess | `/gf-pr-review` |
| Detecting code smells | Different problem class | `/gf-smell` |

`## See Also` 中声明 Phase 4 预期位置，并链接外置模板：

```markdown
- `docs/superpowers/templates/walkthrough-report-template.md` — report skeleton & self-check
- Intended to run at `gf-workflow` Phase 4 alongside `gf-review`; wiring it into the
  orchestrator's dispatch set is out of scope for this skill.
```

- [ ] **Step 4: 运行校验器**

Run: `make check-walkthrough-skill; echo "exit=$?"`
Expected: 第 1、2、4、5、9、10 项 `✓`（第 10 项依赖 Language Detection 段的 `detector.md` 引用）；第 12 项 `✗`（模板未建）；第 8 项 `✗`（报告未产出）；第 3、6、7 项 `— 跳过（无报告）`；第 11 项视首稿长度可能 `✗`。`exit=1`。

第 3、6、7 项输出「跳过」而非 `✗` 是本步骤的关键验证点——若它们报 `✗`，说明 Task 1 的跳过逻辑写错了，回到 Task 1 修正。

---

## Task 3: 报告模板外置 + 词数收敛（REFACTOR）

**Files:**
- Create: `docs/superpowers/templates/walkthrough-report-template.md`
- Modify: `skills/gf-walkthrough/SKILL.md`

**Interfaces:**
- Consumes: Task 2 的 SKILL.md
- Produces: 词数 ≤ 500 的 SKILL.md；含 10 项自检清单的模板文件

- [ ] **Step 1: 写报告模板**

把四节骨架、填写示例、10 项自检清单搬到模板：

```markdown
# Walkthrough Report Template

## ① 开场叙事

3–8 句：谁受影响、发生了什么变化、为什么现在做。
**首句禁止以标识符、函数名、文件路径、命令名开头。** 术语随文解释。

## ② 变更摘要

`git diff --stat` 汇总行 + 按模块分组的「改了什么 · 为什么」。

## ③ 验证证据

- [Measured] <结论>
  ```
  $ <命令原文>
  <输出片段>
  ```
- [Inferred] <结论> —— 依据 `path:line`
- [Unverified] <结论> —— 未验证原因：<原因>

让命令跑起来的 workaround 须一并记录，否则读者无法复现。

存在失败测试时：

| 失败用例 | 最后修改 commit | 是否 base 祖先 |
|---|---|---|

## ④ 评审门禁

blast radius + 部署顺序 + merge checklist。
不涉及迁移或开关时，须显式写明「本次变更不涉及迁移与开关，blast radius 限于 <路径>」，
**不得省略该节**。

## 自检清单（交付前逐项确认）

- [ ] 1. 每条验证结论都带 `Measured` / `Inferred` / `Unverified` 之一
- [ ] 2. 每条 `Measured` 紧随命令原文与输出块
- [ ] 3. 无「只有结论没有输出」却标 `Measured` 的条目
- [ ] 4. 每条 `Inferred` 写明依据的 `path:line`
- [ ] 5. 每条 `Unverified` 写明未验证原因
- [ ] 6. 补跑失败的条目标为 `Unverified`，未被改标为 `Inferred`
- [ ] 7. 失败测试三列齐全，无 `unrelated` 而缺 commit 的写法
- [ ] 8. 开场首句未以标识符、路径或命令名开头
- [ ] 9. ④ 节非空（不涉及迁移时亦显式写明）
- [ ] 10. 报告落盘于 `docs/walkthrough-*.md`
```

- [ ] **Step 2: 从 SKILL.md 删除已外置内容，替换为单行链接**

按 `skill-conventions.md` §1.3 的形式：

```markdown
See [report skeleton & self-check](../../docs/superpowers/templates/walkthrough-report-template.md).
```

- [ ] **Step 3: 测量词数**

Run:
```bash
perl -0 -ne 's/^---\n.*?^---\n//ms; s/```.*?```//gs; s/`[^`]+`//g; print scalar(()=/\p{L}+/g), "\n"' skills/gf-walkthrough/SKILL.md
```
Expected: ≤ 500。超出则继续压缩——优先手段是把叙述句改写为 `skill-conventions.md` §1.4 的模式语言（`[Condition] → [Action] → [Expected Result]`，每条 5–10 词），而非删除 Red Flags 或 Test Scenarios 条目。

- [ ] **Step 4: 运行校验器**

Run: `make check-walkthrough-skill; echo "exit=$?"`
Expected: 第 11、12 项转 `✓`；仅第 8 项仍 `✗`（报告未产出），第 3、6、7 项仍「跳过」。`exit=1`。

---

## Task 4: 实证走查包（AC#6）+ 索引

**Files:**
- Create: `docs/walkthrough-pr<N>-2026-09-16.md`（`<N>` 为本次交付的 PR 号；`local_merge` 路径则用 `docs/walkthrough-<branch-slug>-2026-09-16.md`）
- Modify: `docs/index.md`

**Interfaces:**
- Consumes: Task 3 的 SKILL.md 与模板
- Produces: 走查包，使校验器第 3、6、7、8 项转 `✓`

- [ ] **Step 1: 按 SKILL.md 协议手工产出走查包**

**不依赖 `make install-skills`。** 实测确认 `~/.claude/skills/` 是 `cp -r skills/*` 的副本且 `gf-smell` 至今未安装，故新 skill 在安装前对 Claude Code 不可见。执行者按 SKILL.md 与模板逐节填写即可。

走查对象为本次变更自身（`<base>...<head>`）。四节内容要点：

- ① 开场：首句从「这次变更给…带来什么」切入，不得以 `gf-walkthrough` 或任何路径开头
- ② 摘要：`git diff --stat` 真实输出 + 按文件分组说明
- ③ 证据：逐条对应 Issue #329 的 6 条 AC。其中 **AC#6「非工程读者能读懂」须标 `Unverified`** 并写明「未经真实非工程读者试读」——这是 Phase 1 评审（comment 5695703009）明确要求的诚实交付方式，标为 `Measured` 即自评造假
- ④ 门禁：本次不涉及迁移与开关，须显式写明并给出 blast radius 路径范围

- [ ] **Step 2: 运行校验器，确认全绿**

Run: `make check-walkthrough-skill; echo "exit=$?"`
Expected: 12 项全 `✓`，输出 `全部硬约束通过`，`exit=0`。

这是本计划的 GREEN 终态。

- [ ] **Step 3: 更新 `docs/index.md`**

在报告类目下新增一行指向走查包，并说明归档策略：`docs/walkthrough-*.md` 超过 5 份时，按文件名内嵌日期将最旧者移入 `docs/reports-archive/<YYYY>-Q<N>/`（沿用 `gf-review` 既定规则）。

- [ ] **Step 4: 运行交付前验证套件**

```bash
cargo build
make check-walkthrough-skill
make check-agent-sync
```

Expected: 三条全部成功。

`cargo build` 是必需的：`apps/cli/build.rs:75` 声明了 `cargo:rerun-if-changed=skills/`，新增 skill 目录会触发内嵌清单重新生成，须确认其未被破坏。

**显式豁免**（须在走查包 ③ 节以 `Inferred` 记录豁免理由）：

| 命令 | 豁免理由 |
|---|---|
| `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic` | 无 Rust 源码改动 |
| `cargo audit` / `cargo deny check` | 无依赖、许可证、供应链配置改动 |

---

## 验收标准映射

| Issue #329 AC | 实现位置 | 校验项 |
|---|---|---|
| AC#1 每条结论带三级标注 | Task 2 Step 2 · Task 3 Step 1 自检 #1 | #2 |
| AC#2 `Measured` 必附命令与输出 | Task 2 Step 2 · Task 3 Step 1 自检 #2、#3 | #3 |
| AC#3 失败测试三列溯源，禁写 `unrelated` | Task 2 Step 2 · Task 3 Step 1 自检 #7 | #4、#5 |
| AC#4 开场不以标识符开头 | Task 3 Step 1 · Task 4 Step 1 | #6 |
| AC#5 含 blast radius（含缺省分支） | Task 3 Step 1 ④ 节 · Task 4 Step 1 | #7 |
| AC#6 真实交付上产出，非工程读者可读 | Task 4 Step 1（标 `Unverified` 并记录原因） | #8 |

## Phase 1 评审缺口的吸收

| 评审缺口（comment 5695703009） | 吸收位置 |
|---|---|
| AC#5 未定义缺省分支 | Task 3 Step 1 ④ 节：「不涉及迁移或开关时须显式写明，不得省略该节」+ 自检 #9 |
| AC#6 无判定主体 | Task 4 Step 1：明确要求标 `Unverified` 并写明未试读 |
| 补跑失败禁止降级为 `Inferred` | Task 2 Step 2 Evidence Grading 段 + Task 3 Step 1 自检 #6 |

## 关联

- Issue #329（本票）· Issue #327 `gf-smell`（档位词汇来源）· Issue #349（语言画像单源化）
- Issue #328（已关）—— 其「边界不清」教训塑造了本计划的范围收缩

## 已知的仓库既有缺陷（不在本票范围）

`docs/superpowers/templates/skill-conventions.md:21` 给出的词数统计命令因 `scalar(/.../g)` 在标量上下文返回布尔值而恒输出 `1`，导致 500 词硬限从未真正执行（实测：`gf-smell` 1878 词、`gf-review` 716 词、`gf-quality` 678 词、`gf-pr-review` 640 词，无一满足）。本计划以修正后的命令自守该限，但**不修改该文档**——按 #328 的教训，夹带修复会模糊本票边界。建议单开票处理。
