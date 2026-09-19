# 设计文档：gf-quality / gf-pr-review 三级证据标注与失败测试溯源

**Workflow:** `wf-2026-09-19-001`（full 模式）
**Issue:** #333
**上游依赖：** #329（已关闭，`gf-walkthrough` 已建立三级证据词汇与失败测试溯源算法，本票直接复用原文）

## 背景

实测 `skills/gf-quality/SKILL.md` 与 `skills/gf-pr-review/SKILL.md` 全文 grep 不到 evidence/证据/proof。两份报告只有结论（✅/⚠️/❌），没有「这个结论验证到什么程度」的标注。`gf-review`（提交裁决的技能）本身不分析代码、无报告模板，故不在本票改动范围。

## 范围

只改 `skills/gf-quality/SKILL.md` 与 `skills/gf-pr-review/SKILL.md` 两个文件。`gf-review` 不改。

## 词汇复用（与 #329 保持一致，不重新定义）

直接复制 `skills/gf-walkthrough/SKILL.md:36-44` 的 Evidence Grading 表格原文：

| Tier | Meaning | Hard rule |
|---|---|---|
| `Measured` | Command run this session | No output → downgrade to `Inferred`. |
| `Inferred` | Read from code/config/diff | Must cite `path:line`. |
| `Unverified` | Not verified this run | Must state why; never omitted. |

失败测试溯源表格式与算法复制自 `skills/gf-walkthrough/SKILL.md:46-53`：

```bash
H=$(git log -1 --format=%H -- "<test file>")
git merge-base --is-ancestor "$H" "$BASE" && echo "先于本次交付存在" || echo "本次引入"
```

三列：`失败用例 | 最后修改 commit | 是否 base 祖先`。禁止写 "unrelated"/"与本次改动无关" 而无 commit 溯源。

## 改动 1：`gf-quality`

### Step 2（Run Gates）表新增「证据等级」列

- Gate 命令实际跑过且有输出 → `Measured`
- Gate = `SKIPPED`（工具缺失）或 `N/A`（无该语言源文件）→ `Unverified`，并写明原因（复用现有 `N/A` vs `SKIPPED` 区分逻辑，不新增判定路径）
- Gate 3 (coverage) 的阈值来自配置而非实测 → 阈值行标 `Inferred`（须引用配置文件 `path:line`）；覆盖率数值本身仍是 `Measured`（工具输出）

### Step 3（Quality Report）失败测试扩展

Gate 2 (test) 失败时，报告在失败测试列表下追加三列溯源表（见上）。单语言/多语言报告模板均适用。

### 术语表

在 SKILL.md 新增一节「Evidence Grading」，内容为上述复用表格，标注来源 `gf-walkthrough`/`gf-smell`。

### Rationalization / Red Flags 补充

- 🚩 "跑过一次就行，标 Measured" — 无输出片段不得标 Measured
- ❌ **失败测试写 "与本次改动无关" 却无 commit 溯源** — 禁止

## 改动 2：`gf-pr-review`

### Step 2（Assess 6 Dimensions）逐维度标注

现有要求「✅ or ⚠️ with `path:line`」保留，追加证据等级：

- 纯读 diff 得出的判断 → `Inferred`（已有 `path:line` 要求，天然满足）
- 实际执行了验证命令（如 `gf pr diff` 确认无冲突、跑 lint 命令验证格式问题）→ `Measured`（须附命令原文与输出片段）
- 未核实、凭经验怀疑但未验证的疑点 → `Unverified`（须写明未验证原因），不得省略

### Step 3（Draft Conclusion）

逐维度结论沿用 Step 2 的证据等级标注，汇总进最终 conclusion。

### 术语表

同样新增「Evidence Grading」一节，内容与 gf-quality 完全一致（保证 #333 AC5：与 #329 术语一致）。

### Rationalization / Red Flags 补充

- 🚩 "读了 diff 就是 Measured" — 读 diff 得出的判断是 Inferred，不是 Measured
- ❌ **⚠️ 项只给结论无 path:line 或无证据等级** — 两者均为硬性要求

## 验收标准对照

| AC | 对应改动 |
|---|---|
| 1. 每条结论带三级标注 | gf-quality 逐 Gate + gf-pr-review 逐维度 |
| 2. Measured 项附命令与输出 | 两处 Evidence Grading 表格硬规则 |
| 3. 失败测试三列 | gf-quality Step 3 |
| 4. 禁止 unrelated 无溯源 | gf-quality Step 3 + 两处 Red Flags |
| 5. 术语与 #329 一致 | 两处直接复制 gf-walkthrough 原文表格 |

## Out of Scope

- `gf-review`（不分析代码，无报告模板）
- `docs/references/pr-review-checklist.md` 的六维度定义本身（本票只加证据等级列，不改维度内容）
- 新建 `references/<lang>.md`（复用现有 `gf-quality/references/detector.md`，不新增语言层）
