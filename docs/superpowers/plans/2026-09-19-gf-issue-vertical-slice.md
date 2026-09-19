# gf-issue-create / gf-issue-review 垂直切片与可证伪验收 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `gf-issue-create` 的 Body 模板加入垂直切片约束与可证伪验收要求；`gf-issue-review` 从三维评分扩展为四维（新增「切片方向」），并让「验收标准清晰度」维度校验标准在 base commit 上是否已经为真。

**Architecture:** 纯文档改动，两个 SKILL.md 各一个 Task。不涉及 Rust 代码、不新增 CLI 子命令，复用两个 skill 现有的 `gf issue create` / `gf issue view` / `gf issue comment` 命令。

**Tech Stack:** Markdown (SKILL.md)。

**Spec:** `docs/superpowers/specs/2026-09-19-gf-issue-vertical-slice-design.md`

## Global Constraints

- Skill 源代码在 `skills/` 目录下 — 只改 `skills/gf-issue-create/SKILL.md` 与 `skills/gf-issue-review/SKILL.md`，不改 `.claude/skills/` 中的副本（来自项目 CLAUDE.md）。
- `gf-issue-review` 的判定标准（切片方向、验收标准可证伪）必须与 #330 共用同一套文字表述，不得重新定义（设计文档「与 #330 的一致性」一节）。
- 不新增 CLI 子命令；两个 skill 均继续复用各自现有的 `gf issue create` / `gf issue view` / `gf issue comment` 命令。
- 本次是纯文档改动：不涉及 `cargo build/test/clippy`；验证靠 markdown 自洽性 + `make check-agent-sync`（`validate-skill-commands.sh` + `validate-skill-links.sh` + `verify-skills-when-not-to-use.sh`）。

---

### Task 1: gf-issue-create — Body 模板加入垂直切片与可证伪验收约束

**Files:**
- Modify: `skills/gf-issue-create/SKILL.md:69-79`（Step 2 Body 模板）、`:151-154`（Success Criteria）

**Interfaces:**
- Consumes: 无上游任务。
- Produces: 无下游任务消费本任务产物——本任务独立可验证。

- [ ] **Step 1: 记录基线**

```bash
grep -n "^## Context\|^## Goal\|^## Acceptance Criteria" skills/gf-issue-create/SKILL.md
```

Expected: 命令跑通，输出第 72/74/76 行三个标题。若行号已漂移，以实际 `grep` 输出为准调整下一步的 old_string 定位。

- [ ] **Step 2: 替换 Step 2 Body 模板（第 69-79 行）**

将现有：

```markdown
### Step 2: Body — Markdown template.

```markdown
## Context

## Goal

## Acceptance Criteria
- [ ] …
- [ ] …
```
```

替换为：

```markdown
### Step 2: Body — Markdown template. Before filling it in, check **vertical slice**: does this ticket cover one end-to-end narrow path, or only a single layer (e.g. "only touch the data layer")? A layer-only ticket must be rejected or reshaped — see `smallnest/goal-workflow`'s `to-issues` skill for the pattern this follows.

```markdown
## Context

## Goal

## Acceptance Criteria
- [ ] … (state what observation would prove this false)
- [ ] …
```

Each Acceptance Criteria line must name the observation that would prove it false — never restate the Goal. Reject a criterion that is already true on the base commit (it constrains nothing), depends on another ticket to become checkable, or merely repeats the Goal in checklist form.
```

- [ ] **Step 3: 追加 Success Criteria（第 151-154 行末尾追加 2 条）**

在现有列表末尾追加：

```markdown
- [ ] Body covers a vertical, end-to-end slice — a ticket that only touches one layer (e.g. "data layer only") is rejected or reshaped
- [ ] Every Acceptance Criteria line names a falsifiable observation, not a restated Goal
```

- [ ] **Step 4: 运行 skill 校验**

```bash
make check-agent-sync
```

Expected: `validate-skill-commands.sh` / `validate-skill-links.sh` / `verify-skills-when-not-to-use.sh` 均输出 ✓，退出码 0。本任务未新增/删除文件链接、未引入新 `gf` 子命令，预期无破坏性变更。

- [ ] **Step 5: Commit**

```bash
git add skills/gf-issue-create/SKILL.md
git commit -m "feat(skills): gf-issue-create adds vertical-slice + falsifiable-acceptance constraints (#335)"
```

---

### Task 2: gf-issue-review — 三维扩四维 + base-commit-red 校验

**Files:**
- Modify: `skills/gf-issue-review/SKILL.md:4-5`（description frontmatter）、`:10`（intro）、`:61`（Quick Reference）、`:73-79`（Step 2 评分表）、`:91-95`（报告模板评分表）、`:175`（Success Criteria）

**Interfaces:**
- Consumes: Task 1 定义的「垂直切片」「可证伪验收」判定标准文字表述（复用同一份措辞，不重新定义）。
- Produces: 无下游任务消费本任务产物——本任务是终态交付。

- [ ] **Step 1: 记录基线**

```bash
grep -n "Three dimensions\|Three-dimensional\|three dimensions" skills/gf-issue-review/SKILL.md
```

Expected: 命令跑通，输出第 4、10、61、175 行附近的匹配（措辞略有大小写差异，以实际输出为准调整后续 Step 的 old_string 定位）。

- [ ] **Step 2: 更新 frontmatter description（第 4-5 行）**

将：

```markdown
  Use when the user wants to analyze an Issue's requirement completeness (title clarity, description sufficiency, acceptance criteria) and post findings as an Issue comment.
  当用户希望分析 Issue 需求完整性（标题清晰度、描述充分度、验收标准）并回写评论时使用。
```

替换为：

```markdown
  Use when the user wants to analyze an Issue's requirement completeness (title clarity, description sufficiency, acceptance criteria, slice direction) and post findings as an Issue comment.
  当用户希望分析 Issue 需求完整性（标题清晰度、描述充分度、验收标准、切片方向）并回写评论时使用。
```

- [ ] **Step 3: 更新 intro（第 10 行）**

将：

```markdown
Three-dimensional Issue requirement review — title clarity / description sufficiency / acceptance criteria — emits a structured analysis report, then posts it as an Issue comment. Does not edit the Issue itself.
```

替换为：

```markdown
Four-dimensional Issue requirement review — title clarity / description sufficiency / acceptance criteria / slice direction — emits a structured analysis report, then posts it as an Issue comment. Does not edit the Issue itself.
```

- [ ] **Step 4: 更新 Quick Reference（第 61 行）**

将：

```markdown
**Three dimensions:** Title clarity · Description sufficiency · Acceptance criteria
```

替换为：

```markdown
**Four dimensions:** Title clarity · Description sufficiency · Acceptance criteria · Slice direction
```

- [ ] **Step 5: 更新 Step 2 评分表（第 73-79 行）**

将：

```markdown
### Step 2: Score each dimension 🟢/🟡/🔴

| Dimension | Checks |
|-----------|--------|
| Title | conventional prefix · scope · unambiguous · length |
| Description | context · goal · constraints · references |
| Acceptance | `- [ ]` format · verifiable · happy + error paths |
```

替换为：

```markdown
### Step 2: Score each dimension 🟢/🟡/🔴

| Dimension | Checks |
|-----------|--------|
| Title | conventional prefix · scope · unambiguous · length |
| Description | context · goal · constraints · references |
| Acceptance | `- [ ]` format · verifiable · happy + error paths · **each line states the observation that would prove it false; a criterion already true on the base commit is flagged 🔴 (constrains nothing)** |
| Slice Direction | ticket covers one end-to-end narrow path, not a single layer (e.g. "data layer only" is a 🔴 layer-only slice) |
```

- [ ] **Step 6: 更新报告模板评分表（第 91-95 行）**

将：

```markdown
| Dimension | Rating | Notes |
|------|------|------|
| Title Clarity | 🟢/🟡/🔴 | <brief> |
| Description Sufficiency | 🟢/🟡/🔴 | <brief> |
| Acceptance Criteria Clarity | 🟢/🟡/🔴 | <brief> |
```

替换为：

```markdown
| Dimension | Rating | Notes |
|------|------|------|
| Title Clarity | 🟢/🟡/🔴 | <brief> |
| Description Sufficiency | 🟢/🟡/🔴 | <brief> |
| Acceptance Criteria Clarity | 🟢/🟡/🔴 | <brief> |
| Slice Direction | 🟢/🟡/🔴 | <brief> |
```

- [ ] **Step 7: 更新 Success Criteria（第 175 行）**

将：

```markdown
- [ ] Three-dimension scorecard produced
```

替换为：

```markdown
- [ ] Four-dimension scorecard produced, including Slice Direction
- [ ] Acceptance criteria already true on the base commit are flagged, not silently accepted
```

- [ ] **Step 8: 运行 skill 校验**

```bash
make check-agent-sync
```

Expected: `validate-skill-commands.sh` / `validate-skill-links.sh` / `verify-skills-when-not-to-use.sh` 均输出 ✓，退出码 0。

- [ ] **Step 9: 人工核对与 #330 判定标准一致**

```bash
grep -n "端到端\|vertical slice\|可证伪\|falsifiable" skills/gf-issue-decompose/SKILL.md
```

Expected: 若 #330 已落地对应表述，人工比对本次改动的「Slice Direction」「observation that would prove it false」措辞与 `gf-issue-decompose` 是否矛盾；若 #330 尚未落地（本仓库当前状态），记录为待 #330 落地后回归核对的已知依赖，不阻塞本 Task 提交。

- [ ] **Step 10: Commit**

```bash
git add skills/gf-issue-review/SKILL.md
git commit -m "feat(skills): gf-issue-review adds slice-direction dimension + base-commit-red check (#335)"
```
