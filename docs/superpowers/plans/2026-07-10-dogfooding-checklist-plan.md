# Phase 4 Dogfooding Checklist 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 创建 Phase 4 Dogfooding Checklist 文档，并集成到 gitflow-workflow Phase 4 流程中。

**Architecture:** 创建手动 checklist 文档，按平台分节列出风险驱动的验证步骤。更新 gitflow-workflow SKILL.md 的 Phase 4 执行步骤引用该 checklist。更新 docs/index.md 添加索引。

**Tech Stack:** Markdown, JSON (pending.json schema)

## Global Constraints

- Checklist 文档位于 `docs/specs/phase4-dogfooding-checklist.md`
- Bug 记录格式兼容现有 `.cache/bug-reports/pending.json` schema，新增 `source: "dogfooding"` 字段
- 不修改 Phase 4 现有步骤（pipeline-analyzer、issue-triage、review），仅在其后追加 dogfooding 步骤
- 所有文档使用中英双语标题，正文以英文为主，关键术语保留中文

---

### Task 1: 创建 Dogfooding Checklist 文档

**Files:**
- Create: `docs/specs/phase4-dogfooding-checklist.md`

**Interfaces:**
- Consumes: 设计文档 `docs/superpowers/specs/2026-07-10-dogfooding-checklist-design.md` 中的文档结构定义
- Produces: 完整的 checklist 文档，包含三平台验证步骤和 bug 记录模板

- [ ] **Step 1: 创建 docs/specs 目录（如不存在）**

```bash
mkdir -p docs/specs
```

- [ ] **Step 2: 创建 checklist 文档**

创建 `docs/specs/phase4-dogfooding-checklist.md`，内容如下：

```markdown
# Phase 4 Dogfooding Checklist

**适用版本:** v0.6.x+
**最后更新:** 2026-07-10

> 每次发布前执行此 checklist，用真实 workflow 场景验证核心命令。
> 发现 bug 时记录到 `.cache/bug-reports/pending.json`，所有检查项通过后才能发布。

## Prerequisites

- [ ] 当前版本已完成 Phase 1-3（issue → plan → implement → test → PR）
- [ ] 三个平台的认证状态正常（`gitflow-cli auth status --platform github/gitlab/gitcode`）
- [ ] 工作目录干净，无未提交变更

---

## GitHub Dogfooding

### 风险项：release 命令

> GitHub release 命令涉及版本标签创建和远程操作，历史上有过非交互模式兼容性问题。

- [ ] 创建 release：`gitflow-cli release create v0.x.x --notes "test release"`
- [ ] 删除 release：`gitflow-cli release delete v0.x.x --yes`
- [ ] 非交互模式验证：`echo "y" | gitflow-cli release create v0.x.x --notes "test"`
- [ ] 清理：`gitflow-cli release delete v0.x.x --yes`

### 验证要点

- release 创建后在 GitHub 网页可见
- 非交互模式下 `--yes` 标志正确传递，无交互式确认提示
- 删除操作幂等，重复删除不报错

---

## GitLab Dogfooding

### 风险项：issue label 中文编码

> GitLab API 在处理中文标签时可能出现编码问题，需验证 CRUD 操作。

- [ ] 创建中文标签：`gitflow-cli label create "测试标签" --color "#ff0000"`
- [ ] 创建带中文标签的 issue：`gitflow-cli issue create --title "Dogfooding test" --labels "测试标签"`
- [ ] 查询 issue 标签：`gitflow-cli issue view <n>` 确认标签正确显示
- [ ] 删除测试 issue：`gitflow-cli issue close <n>`
- [ ] 删除测试标签：`gitflow-cli label delete "测试标签" --yes`

### 验证要点

- 中文标签在 GitLab 网页正确显示，无乱码
- label CRUD 全流程无编码错误
- `--yes` 标志在删除操作中正确传递

---

## GitCode Dogfooding

### 风险项：pr merge 非交互模式

> Issue #70: `pr merge` 在非交互 shell 中因缺少 `--yes` 传递而失败。这是 dogfooding 发现的第一个 bug。

- [ ] 创建测试 PR：`gitflow-cli pr create --title "Dogfooding test" --body "test"`
- [ ] 非交互 merge：`gitflow-cli pr merge <n>`（在无 TTY 的 shell 中执行）
- [ ] 验证 `--yes` 传递：确认命令不提示确认，直接完成 merge
- [ ] 清理：删除测试分支

### 验证要点

- `pr merge` 在非交互 shell（`echo "y" | ...` 或 CI 环境）中正常完成
- 不出现 `confirmation required in non-interactive mode` 错误
- `--yes` 标志正确传递到 `gc pr merge` 底层命令

---

## Bug 记录模板

发现 bug 时，将以下 JSON 追加到 `.cache/bug-reports/pending.json`：

```json
{
  "id": "<生成 UUID>",
  "source": "dogfooding",
  "platform": "<github|gitlab|gitcode>",
  "command": "<失败的命令，如 pr merge>",
  "phase4_checklist_item": "<所属检查项，如 GitCode Dogfooding > pr merge 非交互模式>",
  "exit_code": "<命令退出码>",
  "error_code": "DOGFOODING_ERROR",
  "error_message": "<错误信息摘要>",
  "steps_to_reproduce": "<复现步骤>",
  "timestamp": "<ISO 8601 时间戳>",
  "dogfooding_version": "<当前版本，如 0.6.x>"
}
```

**注意：** `pending.json` 当前存储单条记录。如果已有内容，将其包装为数组后追加新条目，或按现有格式覆盖（取决于 `pending.json` 的实际 schema 演进）。

---

## Summary Report

所有检查项执行完成后，生成汇总报告：

```markdown
## Dogfooding Summary — v0.x.x

**Date:** YYYY-MM-DD
**Executor:** <name>
**Result:** PASS / FAIL

| Platform | Items | Passed | Failed | Notes |
|----------|-------|--------|--------|-------|
| GitHub   | 4     | 4      | 0      | OK    |
| GitLab   | 5     | 5      | 0      | OK    |
| GitCode  | 3     | 3      | 0      | OK    |

**Bugs Found:** 0
**Release Decision:** APPROVED / BLOCKED
```

- 如果所有检查项通过且无新增 bug → `Result: PASS`，`Release Decision: APPROVED`
- 如果有任何检查项失败或新增 bug → `Result: FAIL`，`Release Decision: BLOCKED`

---

## References

- Issue #70: `pr merge` 非交互式模式失败（Dogfooding 发现）
- Issue #73: Phase 4 Dogfooding 常态化
- Design Spec: `docs/superpowers/specs/2026-07-10-dogfooding-checklist-design.md`
- Bug Reports: `.cache/bug-reports/pending.json`
- Phase 4 Coverage TDD: `docs/superpowers/specs/2026-07-09-phase4-coverage-tdd-design.md`
```

- [ ] **Step 3: 验证文档格式**

```bash
# 验证文件存在
test -f docs/specs/phase4-dogfooding-checklist.md && echo "OK" || echo "MISSING"

# 验证关键章节存在
grep -c "## GitHub Dogfooding" docs/specs/phase4-dogfooding-checklist.md
grep -c "## GitLab Dogfooding" docs/specs/phase4-dogfooding-checklist.md
grep -c "## GitCode Dogfooding" docs/specs/phase4-dogfooding-checklist.md
grep -c "## Bug 记录模板" docs/specs/phase4-dogfooding-checklist.md
grep -c "## Summary Report" docs/specs/phase4-dogfooding-checklist.md
```

Expected: 所有 grep 返回 `1`

- [ ] **Step 4: Commit**

```bash
git add docs/specs/phase4-dogfooding-checklist.md
git commit -m "docs: add Phase 4 dogfooding checklist (#73)"
```

---

### Task 2: 更新 gitflow-workflow SKILL.md — Phase 4 新增 Dogfooding 步骤

**Files:**
- Modify: `.claude/skills/gitflow-workflow/SKILL.md:280-303` (Phase 4 执行步骤区域)

**Interfaces:**
- Consumes: `docs/specs/phase4-dogfooding-checklist.md`（Task 1 创建）
- Produces: 更新后的 Phase 4 执行步骤，包含 dogfooding checklist 引用

- [ ] **Step 1: 在 Phase 4 执行步骤中添加 dogfooding 步骤**

在 `.claude/skills/gitflow-workflow/SKILL.md` 的 Phase 4 部分，找到现有步骤 3（`gitflow-review`）之后，步骤 4（Update contract）之前，插入新的 dogfooding 步骤。

将现有的步骤 4（Update contract）改为步骤 5，步骤 5（Archive contract）改为步骤 6，步骤 6（COMPLETE）改为步骤 7。

在步骤 3 之后插入：

```markdown
4. **[AUTO]** Execute Dogfooding Checklist
   - Reference: `docs/specs/phase4-dogfooding-checklist.md`
   - Execute each platform's risk-driven checklist items
   - Record any bugs to `.cache/bug-reports/pending.json` with `source: "dogfooding"`
   - All items must pass; any failure blocks Phase 4 completion
   - Output: `dogfooding_passed = true/false`
```

更新步骤 5（原步骤 4）的 contract evidence，添加 `dogfooding_passed` 字段：

```markdown
5. **[AUTO]** Update contract
   ```json
   phases.4.evidence = {
     "pipeline_ok": true,
     "review_report_path": "...",
     "dogfooding_passed": true
   }
   phases.4.status = "complete"
   ```
```

- [ ] **Step 2: 更新 Phase 4 表格描述**

在文件开头的 Phase 表格中（约第 39 行），更新 Phase 4 描述：

将：
```
| 4 | `gitflow-pipeline-analyzer → gitflow-issue-triage → gitflow-review` | Post-delivery checks |
```

改为：
```
| 4 | `gitflow-pipeline-analyzer → gitflow-issue-triage → gitflow-review → dogfooding-checklist` | Post-delivery checks |
```

- [ ] **Step 3: 更新 Fast Mode 的 Phase 4 描述**

在 Fast Mode 部分（约第 45 行），更新：

将：
```
Phase 4: gitflow-pipeline-analyzer → gitflow-issue-triage → gitflow-review (required)
```

改为：
```
Phase 4: gitflow-pipeline-analyzer → gitflow-issue-triage → gitflow-review → dogfooding-checklist (required)
```

- [ ] **Step 4: 验证文档格式**

```bash
# 验证 dogfooding 步骤已添加
grep -c "Execute Dogfooding Checklist" .claude/skills/gitflow-workflow/SKILL.md

# 验证 dogfooding_passed 字段已添加
grep -c "dogfooding_passed" .claude/skills/gitflow-workflow/SKILL.md

# 验证表格已更新
grep -c "dogfooding-checklist" .claude/skills/gitflow-workflow/SKILL.md
```

Expected: 所有 grep 返回 >= `1`

- [ ] **Step 5: Commit**

```bash
git add .claude/skills/gitflow-workflow/SKILL.md
git commit -m "docs(workflow): add dogfooding checklist to Phase 4 (#73)"
```

---

### Task 3: 更新 docs/index.md — 添加 Dogfooding Checklist 索引

**Files:**
- Modify: `docs/index.md`

**Interfaces:**
- Consumes: `docs/specs/phase4-dogfooding-checklist.md`（Task 1 创建）
- Produces: 更新后的文档索引

- [ ] **Step 1: 在 docs/index.md 中添加索引条目**

在 `docs/index.md` 的 `## Development` 部分，`- [Release](./release.md)` 之后添加：

```markdown
- [Phase 4 Dogfooding Checklist](./specs/phase4-dogfooding-checklist.md) — pre-release verification checklist for GitHub/GitLab/GitCode core commands.
```

- [ ] **Step 2: 验证索引格式**

```bash
# 验证索引条目已添加
grep -c "phase4-dogfooding-checklist" docs/index.md
```

Expected: 返回 `1`

- [ ] **Step 3: Commit**

```bash
git add docs/index.md
git commit -m "docs: add dogfooding checklist to docs index (#73)"
```

---

### Task 4: 验证所有交付物

**Files:**
- Validate: `docs/specs/phase4-dogfooding-checklist.md`
- Validate: `.claude/skills/gitflow-workflow/SKILL.md`
- Validate: `docs/index.md`

- [ ] **Step 1: 运行文档完整性验证**

```bash
# 1. Checklist 文档存在且包含所有必需章节
echo "=== Checklist document ==="
test -f docs/specs/phase4-dogfooding-checklist.md && echo "✅ File exists" || echo "❌ File missing"
for section in "GitHub Dogfooding" "GitLab Dogfooding" "GitCode Dogfooding" "Bug 记录模板" "Summary Report" "References"; do
  grep -q "$section" docs/specs/phase4-dogfooding-checklist.md && echo "✅ Section: $section" || echo "❌ Missing section: $section"
done

# 2. gitflow-workflow SKILL.md 包含 dogfooding 引用
echo ""
echo "=== gitflow-workflow SKILL.md ==="
grep -q "Execute Dogfooding Checklist" .claude/skills/gitflow-workflow/SKILL.md && echo "✅ Dogfooding step added" || echo "❌ Dogfooding step missing"
grep -q "dogfooding_passed" .claude/skills/gitflow-workflow/SKILL.md && echo "✅ dogfooding_passed field added" || echo "❌ dogfooding_passed field missing"
grep -q "dogfooding-checklist" .claude/skills/gitflow-workflow/SKILL.md && echo "✅ Table updated" || echo "❌ Table not updated"

# 3. docs/index.md 包含索引
echo ""
echo "=== docs/index.md ==="
grep -q "phase4-dogfooding-checklist" docs/index.md && echo "✅ Index entry added" || echo "❌ Index entry missing"

# 4. 交叉引用链接有效
echo ""
echo "=== Cross-references ==="
grep -q "docs/superpowers/specs/2026-07-10-dogfooding-checklist-design.md" docs/specs/phase4-dogfooding-checklist.md && echo "✅ Design spec reference" || echo "❌ Design spec reference missing"
grep -q ".cache/bug-reports/pending.json" docs/specs/phase4-dogfooding-checklist.md && echo "✅ Bug report path reference" || echo "❌ Bug report path missing"
```

- [ ] **Step 2: 运行 pre-commit hooks**

```bash
pre-commit run --files docs/specs/phase4-dogfooding-checklist.md .claude/skills/gitflow-workflow/SKILL.md docs/index.md
```

Expected: 所有 hooks 通过

- [ ] **Step 3: 最终确认**

检查 git log 确认所有 commit 已创建：

```bash
git log --oneline -5
```

Expected: 看到 3 个新 commit：
1. `docs: add Phase 4 dogfooding checklist (#73)`
2. `docs(workflow): add dogfooding checklist to Phase 4 (#73)`
3. `docs: add dogfooding checklist to docs index (#73)`
