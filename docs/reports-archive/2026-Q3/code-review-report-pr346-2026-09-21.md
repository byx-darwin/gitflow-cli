# Code Review Report — Issue #346

> **交付方式：** `local_merge`（`git merge --no-ff`），无 PR 编号可挂载正式 GitHub review verdict。本报告是 `gf-workflow` Phase 4 的书面存档，`gf review approve/request-changes` 的 CLI 提交步骤因无 open PR 而不适用（`gf-review` skill 的 precondition 是 PR open）。
> **Merge commit:** `1b6bf81`
> **Diff range:** `cdf7946..1b6bf81`（`scripts/render-diff-review.py`、`scripts/tests/test_render_diff_review.py`、`Makefile`，另含设计文档/计划文档）
> **分析日期：** 2026-09-21
> **工作流：** `wf-2026-09-21-006`

## 背景

Issue #346 新增一个零依赖 Python3 脚本 `scripts/render-diff-review.py`（`scan`/`render` 两个子命令），把 `git diff` 渲染成自包含、可离线打开、支持行级锚定/hover 联动/折叠/拖拽记忆宽度的交互式审阅 HTML 页面，随附独立测试脚本（不接入 `make test`，同 #345 惯例）和新增 Makefile 目标 `render-diff-review`。

这是本仓库迄今体量最大的一次单 Issue 交付：8 个顺序实现任务，每个任务均由独立的一次性 reviewer subagent 做过任务级审查（干净），随后一个更强模型对整个分支做了一次 whole-branch 审查，发现 4 项 Important 级问题：

- 已在合并前修复（提交 `b2bb103`）：
  1. CLI 遇到不可读输入/subprocess 失败时应给出可读错误而非原始 Python traceback。
  2. 空 diff（无改动文件）时中间栏应显示友好空状态而非空白。
- 已推迟到新 Issue #399（不阻塞本次合并）：
  3. #398 落地后，annotation-card 逐行可能出现噪音/重复。
  4. diff 面板 hunk 边界之间缺少视觉分隔。

本报告不重复 8 次任务级审查和 1 次 whole-branch 审查已完成的逐行审查，而是执行 Phase 4 要求的独立形式化复核：合并态一致性抽查、独立跑测试、并用自选对抗输入实测验证两项声称已修复的问题是否真的体现在合并代码里（而非仅采信提交信息的描述）。

## 独立复核（不采信原描述，重新验证）

### 1. 声称修复 1——"可读 CLI 错误处理"：实测三种对抗输入，非采信提交信息

直接调用合并后的 `scripts/render-diff-review.py` CLI（不是读代码猜行为），构造三类会在朴素实现里抛出原始 traceback 的输入：

| 对抗输入 | 命令 | 实测输出 | 退出码 |
|---|---|---|---|
| 不存在的 git ref range | `scan "nonexistent-ref-xyz..HEAD"` | `render-diff-review: command failed (git diff nonexistent-ref-xyz..HEAD): fatal: ambiguous argument ...` | 1 |
| 不存在的 annotations 文件 | `render /tmp/does-not-exist.json` | `render-diff-review: file not found: /tmp/does-not-exist.json` | 1 |
| 损坏的 JSON | `render /tmp/bad.json`（内容 `{not valid json`） | `render-diff-review: invalid JSON in annotations file: Expecting property name enclosed in double quotes: line 1 column 2 (char 1)` | 1 |

三种情况均输出单行、人类可读的 stderr 消息，无 Python traceback，退出码非零。对应实现是 `main()` 里的 `try/except`（`FileNotFoundError`/`json.JSONDecodeError`/`subprocess.CalledProcessError`/`OSError` 分别捕获，`scripts/render-diff-review.py:477-505`），覆盖面与实测行为一致——**结论：该修复真实生效，非仅提交信息声称**。

### 2. 声称修复 2——"友好空 diff 提示"：实测验证

构造 `{"diff_range": "HEAD..HEAD", "files": []}` 的 annotations，跑 `render` 子命令，实测输出 HTML 中间栏内容：

```html
<p class="empty">当前没有改动</p>
```

对应实现 `render_html()` 里的 `if not files:` 分支（`scripts/render-diff-review.py:430-433`），且对应 CSS 规则 `#diff-pane > p.empty` 已加入 `STYLE`（非样式缺失的裸文字）。文件树/注释区在无文件时也能正常渲染空容器，不报错。**结论：该修复真实生效**。

### 3. 合并态一致性抽查（`git diff cdf7946..1b6bf81 -- scripts/ Makefile`）

- `Makefile` 新增 `render-diff-review` 目标，`RANGE` 环境变量可覆盖默认 `dev..HEAD`，`.PHONY` 列表已同步追加目标名——与设计文档 §4 描述一致。
- `scripts/render-diff-review.py`（509 行）+ `scripts/tests/test_render_diff_review.py`（542 行）为最终态，无中间任务留下的死代码或未接线函数；`grep -n "TODO\|todo!\|FIXME\|XXX"` 无命中。
- 提交历史（`cdf7946..1b6bf81`，14 个提交）显示 TDD 节奏完整：每个 Task 均有独立 RED/GREEN 提交，另有 2 个针对性修复提交（`b9cad2f` 收紧未识别 hunk 行的解析严格性、`e4267c0` 清理 Task 4 遗留的未使用 import）——均为质量收敛动作，非新增风险面。
- `.cache/diff-review/` 产物路径已被现有 `.gitignore` 的 `.cache/` 行覆盖，未新增/修改 `.gitignore`（符合计划 Global Constraints）。
- HTML 转义抽查：所有从 diff/文件系统读入的动态字符串（文件路径、行内容、`old_path`、`error` 消息）在写入模板前均经 `html.escape()`；测试套件里的 `test_html_injection_is_escaped` 用例覆盖了文件名和行内容两处注入点，实测未见遗漏点（如 `header_extra` 里的 `old_path` 拼接、`_render_annotation_cards` 里的 `error`/`path` 拼接均已转义）。
- JS 交互层无任何"接受/拒绝/写回"入口（`grep -i "accept\|reject\|approve\|apply-fix\|write-back"` 在渲染产物中无命中），符合"单向只读展示"的 Non-Goal。

### 4. AC8 真实历史 range 验收——独立复现，非采信

用本次交付自己的 range（`cdf7946..1b6bf81`）跑一遍完整 scan+render 流程并核对渲染行数与 `git diff --stat` 是否吻合：

```
rendered (render-diff-review.py scan): +2709 -1
git diff --stat cdf7946..1b6bf81:      2709 insertions(+), 1 deletion(-)
```

两边一致，AC8 的"渲染行数与 `git diff --stat` 吻合"字面要求成立。

### 5. 独立跑测试套件

```
$ python3 scripts/tests/test_render_diff_review.py
........................................
----------------------------------------------------------------------
Ran 40 tests in 0.394s

OK
```

40 项测试全过（计划最终态预期 34 项，实际交付又追加了 `test_html_injection_is_escaped`（rename/error 路径）等若干项，数量增加是测试覆盖增强，非回归）。本变更未触碰任何 Rust 源码/API/Cargo manifest，按仓库验证策略（"Docs-only/脚本-only 变更不强制跑 Rust build/test/clippy，除非影响行为"）不需要跑 `make build`/`make test`（Rust）/`cargo clippy`；已确认 `git diff --stat` 中无任何 `.rs`/`Cargo.toml` 文件被触碰。

## 遗留问题（已知，本报告不重复评估，仅确认未被静默丢弃）

- annotation-card 逐行噪音/重复风险（待 #398）→ 已开 Issue #399，未在本次范围内。
- diff 面板 hunk 边界无视觉分隔 → 已开 Issue #399，未在本次范围内。

两项均为 Minor/Important 但非阻塞性的 UX 打磨项，遵循"零依赖、离线可用、语言无关"定位下的合理取舍，且已有可追踪的后续 Issue，不构成本次合并的质量缺口。

## 结论

- 独立复核未发现新的 Critical/Important 级问题。
- 两项声称的合并前修复经对抗输入实测确认真实生效、实现正确。
- 合并态与设计文档/计划文档描述一致，无遗留 TODO/占位符/死代码。
- 测试套件独立运行通过（40/40）。
- AC8 真实 range 验收独立复现，行数吻合。

**Verdict: Approve**（无新增 Finding；两项已延后事项已有独立 Issue #399 跟踪）。
