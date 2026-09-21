# gf-workflow: gf-security-check / gf-regression 阻断式接入 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 把 `gf-security-check`、`gf-regression` 接入 `gf-workflow` 编排，作为 Phase 3 新增的阻断式 Step，按改动面（依赖/锁文件 vs. CLI 行为相关 crate 路径）条件触发，全模式统一生效。

**Architecture:** 在 `skills/gf-workflow/SKILL.md` 的 Phase 3 步骤表里插入新 Step 3（原 Step 3-7 顺延为 4-8），`skills/gf-workflow/gates.md` 的 Gate 3→4 追加两个证据字段的取值约束，`skills/gf-workflow/references.md` 新增 "Change-Surface Detection" 小节承载路径规则表和恢复语义。两个被接入的 skill 各自补一段报告落盘/归档约定（镜像 `gf-review` 已有的对称写法），`docs/index.md` 的 Reports Archive 一节同步登记这两类新报告。全部是文档/skill 定义改动，不涉及 Rust 代码。

**Tech Stack:** Markdown（skill 定义文件）、`make check-agent-sync`（Makefile 目标，校验 skill 文档交叉引用与 CLI 命令引用一致性）。

**Spec:** `docs/superpowers/specs/2026-09-21-gf-workflow-security-regression-gate-design.md`

## Global Constraints

- 不改动 `skills/gf-workflow/SKILL.md` 的 Phase 4 Step Matrix 表——新 Step 3 属于 Phase 3，对 full/standard/fast 三种模式统一生效，不按模式开关。
- 不改动 `gf-security-check`、`gf-regression` 自身的检测/测试逻辑——只补报告落盘路径与归档约定。
- Task 4（报告落盘约定）必须逐字镜像 `skills/gf-review/SKILL.md` 现有 "Report Output & Archiving" 小节的措辞与结构，仅替换文件名前缀与 skill 名，不得另造表述。
- 每个 Task 编辑完成后，验证手段统一为：`make check-agent-sync`（必须保持与改动前相同的 PASS 计数、不出现新的 mismatch）+ 人工读一遍 diff 确认 Markdown 表格/标题语法没有破损。这是文档/skill-only 改动，不跑 Rust build/test/clippy（按 CLAUDE.md 的文档改动验证规则）。
- Contract 证据字段新增（`change_surface`、`security_check`、`regression_check`）必须合并进 `phases.3.evidence`，不得覆盖 `branch`/`base_branch`/`worktree_path`/`worktree_preflight`/`unresolved_dirty_paths`/`delivery_mode`/`pr_url`/`merge_commit`/`tests_passed`/`merge_queued` 等既有字段。

---

## Task 1: `skills/gf-workflow/SKILL.md` — 插入 Phase 3 新 Step 3，Gate 3→4 前置证据说明同步更新

**Files:**
- Modify: `skills/gf-workflow/SKILL.md:352-358`（Phase 3 步骤表）
- Modify: `skills/gf-workflow/SKILL.md`（Red Flags 表，约第 189 行后追加一行；Rationalization 表，约第 210 行后追加一行）

**Interfaces:**
- Consumes：Phase 3 Step 1 已产出的 `base_branch`（既有字段，不新增）。
- Produces：新 Step 3 的 Output 列命名三个证据字段 `change_surface`、`security_check`、`regression_check`，供 Task 2（gates.md）、Task 3（references.md）引用这三个名字时保持一致。

- [ ] **Step 1: 修改 Phase 3 步骤表**

当前（`skills/gf-workflow/SKILL.md:352-358`）：

```markdown
| 1 | **[AUTO/PAUSE]** Record `base_branch` via `git rev-parse --abbrev-ref HEAD`. ... | `branch`, `base_branch`, `worktree_path`, `worktree_preflight` |
| 2 | **[AUTO] Execution engine** ... | implementation |
| 3 | **[AUTO]** Pre-delivery symlink guard ... **① Local merge**: ... **② PR**: ... | `pr_url` or (`delivery_mode`, `merge_commit`) |
| 4 | **[AUTO]** `make test` or `cargo test` — **本地前置自检**，不等于 CI 把关 | `tests_passed` |
| 5 | **[AUTO]** 排队合并：`gf pr merge <n> --auto` ... | `merge_queued` |
| 6 | **[AUTO]** Update contract: `evidence = { branch, base_branch, worktree_path, worktree_preflight, unresolved_dirty_paths, delivery_mode, pr_url, merge_commit, tests_passed, merge_queued }` ... | — |
| 7 | **[AUTO]** Gate 3→4 — 交付证据二选一（`pr_url` 或 `merge_commit`，按 `delivery_mode`）+ `tests_passed = true` → **AUTO-ADVANCE to Phase 4**。... | — |
```

改为（插入新 Step 3，原 3-7 顺延为 4-8；Step 1、Step 2 原样不动）：

```markdown
| 1 | **[AUTO/PAUSE]** Record `base_branch` via `git rev-parse --abbrev-ref HEAD`. ... | `branch`, `base_branch`, `worktree_path`, `worktree_preflight` |
| 2 | **[AUTO] Execution engine** ... | implementation |
| 3 | **[AUTO/PAUSE]** 改动面检测——阻断式接入 `gf-security-check` / `gf-regression`（Issue #344）。用 Step 1 已记录的 `base_branch`（不重新猜测）计算 `diff_files=$(git diff --name-only "$base_branch"...HEAD)`。按 `references.md` → Change-Surface Detection 的两条路径规则独立判定：命中 `Cargo.toml`/`Cargo.lock`/`**/Cargo.toml`/`deny.toml` → security 触发；命中 `apps/cli/src/**`、`crates/core/src/**`、`crates/github/src/**`、`crates/gitlab/src/**`、`crates/gitcode/src/**` → regression 触发。两者都未命中 → `change_surface = "docs_only"`，两项均记 `not_triggered`，跳过本 Step 其余动作，直接进入 Step 4。命中的每一项：✋ PAUSE 展示匹配文件 + 即将运行的检查名，问用户 ① 运行（默认）② 跳过（须给非空理由；理由为空或未填视为无效选择，按①处理，不得静默跳过）。① 运行 → inline 调用对应 skill（不走 Phase 4 式的并行 Agent 派发——本 Step 需要同步阻断并可能与用户交互，做法与 Phase 2 的 `gf-quality` gate 一致）：`gf-security-check` 出现 Critical/High 级发现（未修补 CVE、硬编码密钥）判 FAIL；`gf-regression` smoke test 出现非已知 flaky 的 FAIL 判 FAIL。FAIL → 阻断，不进入 Step 4；提示用户回到 Step 2 修复后重跑本 Step。PASS → 继续。② 跳过 → 记 `exemption_reason`，不运行该检查，继续。报告落盘路径与归档规则见各自 SKILL.md 的 "Report Output & Archiving" 小节。本 Step 对 full/standard/fast 三种模式统一生效，不进 Phase 4 Step Matrix（那张表管的是按模式开关的报告步骤，这里是按改动面触发的阻断步骤）。 | `change_surface`, `security_check`, `regression_check` |
| 4 | **[AUTO]** Pre-delivery symlink guard ... **① Local merge**: ... **② PR**: ... | `pr_url` or (`delivery_mode`, `merge_commit`) |
| 5 | **[AUTO]** `make test` or `cargo test` — **本地前置自检**，不等于 CI 把关 | `tests_passed` |
| 6 | **[AUTO]** 排队合并：`gf pr merge <n> --auto` ... | `merge_queued` |
| 7 | **[AUTO]** Update contract: `evidence = { branch, base_branch, worktree_path, worktree_preflight, unresolved_dirty_paths, change_surface, security_check, regression_check, delivery_mode, pr_url, merge_commit, tests_passed, merge_queued }` ... | — |
| 8 | **[AUTO]** Gate 3→4 — 交付证据二选一（`pr_url` 或 `merge_commit`，按 `delivery_mode`）+ `tests_passed = true` + `security_check.status`/`regression_check.status` 均不为 `failed`（见 `gates.md`） → **AUTO-ADVANCE to Phase 4**。... | — |
```

（原 Step 3、4、5、6 的省略号部分`...`保持原文一字不改，只是行号/序号整体顺延一位；仅 Step 7 的 evidence 列表新增 `change_surface, security_check, regression_check` 三个字段，Step 8 的说明追加 Gate 3→4 新增的取值约束提示。）

- [ ] **Step 2: Red Flags 表追加一行**

在 "About to delete untracked files to make the tree look clean" 那一行之后追加：

```markdown
| About to let a user "跳过" security/regression check without a reason | **STOP** — 跳过必须有非空理由并写入 `exemption_reason`，理由为空按默认（运行）处理，不得静默跳过 |
```

- [ ] **Step 3: Rationalization 表追加一行**

在 "It's our own workflow doc, auto-commit it" 那一行之后追加：

```markdown
| "改动面没命中依赖/CLI路径，可以不检测直接跳" | No — 未命中时的正确行为是把 `change_surface` 记为 `docs_only` 并把两项检查记 `not_triggered`，而不是跳过整个 Step 3 的判定动作本身；判定动作永远要跑，只是判定结果决定要不要真的调用 skill |
```

- [ ] **Step 4: 验证**

```bash
make check-agent-sync
```

预期：与改动前相同的 PASS 计数（Commands in CLI / Files scanned / Refs checked 均不变，Mismatches 仍为 0），因为本 Task 未新增/删除任何 `## See Also` 引用或 CLI 命令引用。人工确认新插入的表格行没有破坏 Markdown 表格语法（每行 `|` 数量一致）。

- [ ] **Step 5: Commit**

```bash
git add skills/gf-workflow/SKILL.md
git commit -m "feat(gf-workflow): insert Phase 3 change-surface gate for security/regression checks"
```

---

## Task 2: `skills/gf-workflow/gates.md` — Gate 3→4 追加证据取值约束

**Files:**
- Modify: `skills/gf-workflow/gates.md`（"### Gate 3→4: 执行 → 交付" 小节，约第 55-70 行；`check_gate()` 函数的 `elif target_phase == 4:` 分支，约第 130-138 行）

**Interfaces:**
- Consumes：Task 1 产出的证据字段名 `security_check.status`、`regression_check.status`（取值集合 `passed|failed|exempted|not_triggered`）。
- Produces：无新字段名——只是给已存在（Task 1 之后）的字段追加一条闸门断言，不产出新接口。

- [ ] **Step 1: 修改 Gate 3→4 的条件说明**

当前：

```markdown
### Gate 3→4: 执行 → 交付

**条件:**
- `phases.3.status` 为 `complete`
- 交付证据二选一（`delivery_mode` 缺省视为 `"pr"`）：
  - `delivery_mode == "pr"` → `phases.3.evidence.pr_url` 非空
  - `delivery_mode == "local_merge"` → `phases.3.evidence.merge_commit` 非空
- `phases.3.evidence.tests_passed` 为 `true`（两种交付方式均必须）
```

改为：

```markdown
### Gate 3→4: 执行 → 交付

**条件:**
- `phases.3.status` 为 `complete`
- 交付证据二选一（`delivery_mode` 缺省视为 `"pr"`）：
  - `delivery_mode == "pr"` → `phases.3.evidence.pr_url` 非空
  - `delivery_mode == "local_merge"` → `phases.3.evidence.merge_commit` 非空
- `phases.3.evidence.tests_passed` 为 `true`（两种交付方式均必须）
- `phases.3.evidence.security_check.status` ∈ `{passed, exempted, not_triggered}`（Issue #344；防御性检查——Phase 3 新 Step 3 已经在 `failed` 时阻断，正常情况下不会带着 `failed` 走到这里）
- `phases.3.evidence.regression_check.status` ∈ `{passed, exempted, not_triggered}`（同上）
```

- [ ] **Step 2: 修改 `check_gate()` 的 `target_phase == 4` 分支**

当前：

```python
    elif target_phase == 4:
        evidence = contract["phases"]["3"]["evidence"]
        delivery_mode = evidence.get("delivery_mode", "pr")
        if delivery_mode == "local_merge":
            delivery_ok = bool(evidence.get("merge_commit"))
        else:
            delivery_ok = bool(evidence.get("pr_url"))
        return contract["phases"]["3"]["status"] == "complete" \
               and delivery_ok \
               and evidence.get("tests_passed")
```

改为：

```python
    elif target_phase == 4:
        evidence = contract["phases"]["3"]["evidence"]
        delivery_mode = evidence.get("delivery_mode", "pr")
        if delivery_mode == "local_merge":
            delivery_ok = bool(evidence.get("merge_commit"))
        else:
            delivery_ok = bool(evidence.get("pr_url"))
        ok_statuses = {"passed", "exempted", "not_triggered"}
        security_ok = evidence.get("security_check", {}).get("status") in ok_statuses
        regression_ok = evidence.get("regression_check", {}).get("status") in ok_statuses
        return contract["phases"]["3"]["status"] == "complete" \
               and delivery_ok \
               and evidence.get("tests_passed") \
               and security_ok \
               and regression_ok
```

- [ ] **Step 3: 验证**

```bash
make check-agent-sync
python3 -c "
import ast
src = open('skills/gf-workflow/gates.md').read()
start = src.index('def check_gate')
end = src.index('def get_phase4_steps')
code = src[start:end]
ast.parse(code)
print('check_gate() 语法通过')
"
```

预期：`make check-agent-sync` PASS 计数不变；Python 语法解析成功（这是文档里嵌的示例代码，不会被实际执行，但必须保持语法正确，因为 `gates.md` 一直把它当作可读的伪代码规范维护）。

- [ ] **Step 4: Commit**

```bash
git add skills/gf-workflow/gates.md
git commit -m "feat(gf-workflow): Gate 3->4 asserts security/regression check status"
```

---

## Task 3: `skills/gf-workflow/references.md` — 新增 Change-Surface Detection 小节 + Cross-Session Recovery 补充

**Files:**
- Modify: `skills/gf-workflow/references.md`（在 `### Worktree Preflight (Phase 3 Step 1)` 小节之后、`### Why the Symlink Depth Is Computed, Not Hardcoded` 之前插入新 `### Change-Surface Detection (Phase 3 Step 3, Issue #344)` 小节；`## Cross-Session Recovery` 小节的 Phase 3 一行追加说明）

**Interfaces:**
- Consumes：Task 1 中 SKILL.md 新 Step 3 文本里引用的 `references.md → Change-Surface Detection` 锚点——本 Task 必须产出这个确切标题的小节，否则 Task 1 里的引用是悬空链接。
- Produces：两张路径规则表（供任何人工阅读者查表，不是被其他文件按字段名引用的接口）。

- [ ] **Step 1: 插入 Change-Surface Detection 小节**

在 `references.md` 里找到 `### Worktree Preflight (Phase 3 Step 1)` 小节的结尾（该小节结束于 `### Why the Symlink Depth Is Computed, Not Hardcoded` 标题之前），在两者之间插入：

```markdown
### Change-Surface Detection (Phase 3 Step 3, Issue #344)

Phase 3 Step 3 决定是否阻断式运行 `gf-security-check` / `gf-regression`，判定依据是 Step 1 已经记录的 `base_branch`——**不重新猜测 base ref**，直接复用：

```bash
diff_files="$(git diff --name-only "$base_branch"...HEAD)"
```

（`gf-quality` 的 Gate 3 曾经因为用 `${BASE_REF:-origin/main}` 猜测 base ref，在 GitLab/GitCode 默认分支不是 `main` 时解析失败；这里直接吃 Phase 3 Step 1 的既有产出，不重蹈覆辙。）

两条独立规则，各自判定，互不影响：

| 检查 | 触发路径模式 |
|---|---|
| `gf-security-check` | `Cargo.toml`、`Cargo.lock`、`**/Cargo.toml`（workspace 内任意 crate）、`deny.toml` |
| `gf-regression` | `apps/cli/src/**`、`crates/core/src/**`、`crates/github/src/**`、`crates/gitlab/src/**`、`crates/gitcode/src/**` |

都不命中（例如纯文档/spec/skill 文本改动）→ `change_surface = "docs_only"`，两项检查都记 `not_triggered`，不实际调用任何一个 skill。

**为什么必须在 Step 3（旧 Step 3 交付选择之前）而不是 Phase 4：** local_merge 路径下，旧的 Step 3（现 Step 4）会立刻把分支合并进 `base_branch`。如果检查放在 Phase 4（交付之后），发现问题时代码已经进了 `base_branch`，"阻断式"就名不副实了——所以必须卡在合并动作发生之前。
```

- [ ] **Step 2: Cross-Session Recovery 小节追加一行说明**

当前（`## Cross-Session Recovery` 小节里的加载清单）：

```markdown
3. Load context based on mode and current_phase:
   • Phase 1: No doc needed (start fresh)
   • Phase 2: Read design_doc_path; check mode for Phase 1 exemptions
   • Phase 3: Read spec_path (plan document); check mode for fast skip
   • Phase 4: Read pr_url + review reports; use get_phase4_steps(mode) to determine remaining steps
```

改为：

```markdown
3. Load context based on mode and current_phase:
   • Phase 1: No doc needed (start fresh)
   • Phase 2: Read design_doc_path; check mode for Phase 1 exemptions
   • Phase 3: Read spec_path (plan document); check mode for fast skip; also reload `change_surface`/`security_check`/`regression_check` evidence (Issue #344) alongside `branch`/`worktree_path` — a session resuming mid-Phase-3 must not re-ask a change-surface question already answered before the interruption
   • Phase 4: Read pr_url + review reports; use get_phase4_steps(mode) to determine remaining steps
```

- [ ] **Step 3: 验证**

```bash
make check-agent-sync
grep -n "Change-Surface Detection" skills/gf-workflow/references.md
grep -n "Change-Surface Detection" skills/gf-workflow/SKILL.md
```

预期：`make check-agent-sync` PASS 计数不变；两个 grep 都应各自命中一次（`references.md` 是新小节标题本身，`SKILL.md` 是 Task 1 里 Step 3 文本里的引用），确认锚点没有悬空。

- [ ] **Step 4: Commit**

```bash
git add skills/gf-workflow/references.md
git commit -m "docs(gf-workflow): document change-surface detection rules and recovery semantics"
```

---

## Task 4: 报告落盘约定 — `gf-security-check`、`gf-regression`、`docs/index.md`

批量处理：三处都是同形状的机械性文档追加（镜像 `gf-review` 已有的 "Report Output & Archiving" 写法），风险低，合并为一个 Task 一次提交。

**Files:**
- Modify: `skills/gf-security-check/SKILL.md`（追加 `## Report Output & Archiving` 小节）
- Modify: `skills/gf-regression/SKILL.md`（追加 `## Report Output & Archiving` 小节）
- Modify: `docs/index.md`（`## Reports Archive` 小节的项目符号列表追加两行）

**Interfaces:**
- Consumes：Task 1 新 Step 3 文本里承诺的 "报告落盘路径与归档规则见各自 SKILL.md 的 Report Output & Archiving 小节"——本 Task 必须让这句话指向真实存在的小节。
- Produces：`docs/security-report-<issue-number>-<YYYY-MM-DD>.md`、`docs/regression-report-<issue-number>-<YYYY-MM-DD>.md` 两个文件名约定，供 Phase 3 Step 3 实际运行时使用（本计划不生成任何实际报告文件，只登记约定）。

- [ ] **Step 1: `gf-security-check` 追加 Report Output & Archiving 小节**

参照 `skills/gf-review/SKILL.md` 现有写法（原文）：

```markdown
## Report Output & Archiving

When invoked as the `gf-review` step of `gf-workflow` Phase 4 (full/standard
mode), the accompanying code review findings persisted for audit trail go to
`docs/code-review-report-pr<N>-<YYYY-MM-DD>.md`. Once
`code-review-report-*.md` files under `docs/` exceed 5, move all but the 5
most recent (ordered by the PR number embedded in the filename) into
`docs/reports-archive/<YYYY>-Q<N>/`, bucketed by each report's own date. See
`docs/index.md` → Reports Archive for the full policy.
```

在 `skills/gf-security-check/SKILL.md` 里，找到 `## When NOT to Use` 小节之后、`## Quality Pipeline` 或第一个流程小节之前的位置（即紧跟在 skill 简介/CLI Requirement/Preconditions/When to Use/When NOT to Use 这组固定小节之后），插入：

```markdown
## Report Output & Archiving

When invoked as the Phase 3 change-surface gate step of `gf-workflow`
(Issue #344), the audit-trail report goes to
`docs/security-report-<issue-number>-<YYYY-MM-DD>.md`. Once
`security-report-*.md` files under `docs/` exceed 5, move all but the 5
most recent (ordered by the issue number embedded in the filename) into
`docs/reports-archive/<YYYY>-Q<N>/`, bucketed by each report's own date. See
`docs/index.md` → Reports Archive for the full policy.
```

- [ ] **Step 2: `gf-regression` 追加 Report Output & Archiving 小节**

在 `skills/gf-regression/SKILL.md` 里同样位置（固定小节组之后）插入：

```markdown
## Report Output & Archiving

When invoked as the Phase 3 change-surface gate step of `gf-workflow`
(Issue #344), the audit-trail report goes to
`docs/regression-report-<issue-number>-<YYYY-MM-DD>.md`. Once
`regression-report-*.md` files under `docs/` exceed 5, move all but the 5
most recent (ordered by the issue number embedded in the filename) into
`docs/reports-archive/<YYYY>-Q<N>/`, bucketed by each report's own date. See
`docs/index.md` → Reports Archive for the full policy.
```

- [ ] **Step 3: `docs/index.md` 的 Reports Archive 列表追加两行**

当前（`docs/index.md` 的 `## Reports Archive` 小节列表，第 72-83 行附近）：

```markdown
- `code-review-report-*.md` — PR code review findings from `/code-review` and `gf-pr-review`.
- `pipeline-analysis-report-*.md` — CI health snapshots from `gf-pipeline-analyzer`.
```

改为（在这两行之间插入两条新行，保持字母/主题相关性不强求排序，追加在 `code-review-report-*.md` 之后即可）：

```markdown
- `code-review-report-*.md` — PR code review findings from `/code-review` and `gf-pr-review`.
- `security-report-*.md` — dependency/lockfile-triggered audit findings from `gf-security-check`, run as the `gf-workflow` Phase 3 change-surface gate (Issue #344).
- `regression-report-*.md` — CLI-behavior-change-triggered smoke test results from `gf-regression`, run as the `gf-workflow` Phase 3 change-surface gate (Issue #344).
- `pipeline-analysis-report-*.md` — CI health snapshots from `gf-pipeline-analyzer`.
```

- [ ] **Step 4: 验证**

```bash
make check-agent-sync
grep -n "Report Output & Archiving" skills/gf-security-check/SKILL.md skills/gf-regression/SKILL.md
grep -n "security-report-\*.md\|regression-report-\*.md" docs/index.md
```

预期：`make check-agent-sync` PASS 计数不变（新增的 `docs/index.md` → Reports Archive 一节的引用不是 `## See Also` 交叉引用，不计入 check-agent-sync 的 skill 互链校验范围，这个校验只统计 skill 与 skill 之间、skill 与 CLI 命令之间的引用一致性）；两条 grep 均应各命中两次（两个 skill 各一次；`docs/index.md` 两个文件名模式各一次）。

- [ ] **Step 5: Commit**

```bash
git add skills/gf-security-check/SKILL.md skills/gf-regression/SKILL.md docs/index.md
git commit -m "docs: register security/regression report archiving convention (Issue #344)"
```

---

## Final Validation (whole-branch, before delivery)

```bash
make check-agent-sync
```

预期：与 Task 1 之前记录的基线 PASS 计数完全一致（本计划全程不新增/删除任何 skill 的 `## See Also` 条目或 CLI 命令引用，只新增小节内容与表格行），且无新增 mismatch。人工通读一遍四个 Task 的合并 diff（`git log --oneline`、`git diff <base>...HEAD`），确认：Phase 3 步骤编号在 SKILL.md 全文中没有遗漏未顺延的旧引用（搜索 `Step 3`、`Step 4` 等字面量在其余小节里是否还有指向旧编号的地方）。
