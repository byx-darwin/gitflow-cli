# 多角色评估「主动上报bug 功能」执行计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 从 5 个角色 × 3 个层面，对「主动上报 bug 功能」端到端链路（CLI error_reporter → Stop Hook → gf-autoreport-bug skill → Issue）做出**仅评估**（不实施修复）的判定：OK / 有条件 OK / NOT OK，并产出分级改进建议（P0/P1/P2）。

**Architecture:** 评估链路分段取证 → 每角色给出独立判定 → 交叉合成总体判定 → 输出分级建议。所有结论必须引用可核验证据（文件:行号 / 命令输出），禁止臆测。

**Tech Stack:** Rust CLI（`error_reporter.rs`）、Bash Stop Hook（`auto-report-bug.sh`）、Claude Code skill（`gf-autoreport-bug`）、`gf` CLI、GitHub Issue API。

**Spec:** `docs/superpowers/specs/2026-08-18-autoreport-bug-multi-role-eval-design.md`（随计划传递，执行者需同时阅读）

## Global Constraints

- **仅评估，不实施任何代码/配置/skill 修复**。发现缺陷只记录为改进建议。
- 评估对象链路：`apps/cli/src/error_reporter.rs`（写入端）→ `.claude/settings.json` Stop Hook → `hooks/auto-report-bug.sh`（捕获端）→ `.claude/skills/gf-autoreport-bug/SKILL.md`（处理端）→ `docs/references/gf-autoreport-bug-params.md`（支撑文档）。
- Skill 源代码位于 `skills/gf-autoreport-bug/SKILL.md`；`.claude/skills/` 是 Claude Code 使用的副本。评估时两者都核对，但只报告差异，不修改。
- 每条评估结论必须有证据锚点（`文件:行号` 或 `gf <cmd>` 实测输出）；无证据则标记为「待核实」，不得臆测。
- 判定等级定义：**OK** = 链路各环正确且端到端可靠；**有条件 OK** = 核心可用但有确定缺陷需修复后才算可靠；**NOT OK** = 链路存在阻断性缺陷（如从未产出、误报/漏报系统性问题）。
- 交付物：评估报告 Markdown（`docs/` 下）+ Issue #209 评论摘要。

---

### Task 1: 证据基线固化（Evidence Baseline）

**Files:**
- Read: `apps/cli/src/error_reporter.rs`, `apps/cli/src/main.rs:140-170`
- Read: `.claude/settings.json`, `hooks/auto-report-bug.sh`, `hooks/tests/auto-report-bug.bats`
- Read: `.claude/skills/gf-autoreport-bug/SKILL.md`, `skills/gf-autoreport-bug/SKILL.md`, `docs/references/gf-autoreport-bug-params.md`
- Create: `docs/superpowers/plans/evidence-baseline-wf-2026-08-18-005.md`

**Interfaces:**
- Consumes: 设计文档 §4（已核实证据基线）、`wf-2026-08-18-005` 合同
- Produces: `evidence_baseline.md`（每个证据含 `file:line` 或命令输出 + 状态 🔴/🟠/🟢），供 Task 2-4 引用

- [ ] **Step 1: 逐环取证 — 写入端**

读取 `apps/cli/src/error_reporter.rs` 全文，记录：
- `maybe_report_error`（行 175-193）：`should_skip_reporting`（stderr 非 TTY）→ `is_co_contribution_enabled`（双 gate）
- `is_co_contribution_enabled`（行 204-220+）：项目/全局 settings 检查
- `write_to_disk`（含 `0o600` 权限）
- `sanitize_error_message`（token/家目录脱敏，正则）
- `generate_unique_id`
- `main.rs` 调用点（`report_error_noninteractive`，行 152-163）

输出每个事实：`<事实> → 证据: error_reporter.rs:<行>`。

- [ ] **Step 2: 逐环取证 — 触发端**

读取 `.claude/settings.json` 全文，记录 Stop Hook matcher（`gitflow`）与 command。运行 `gf issue list --search "auto-report" --state all` 验证零产出事实，记录输出。

- [ ] **Step 3: 逐环取证 — 捕获端**

读取 `hooks/auto-report-bug.sh` 全文，记录：
- TTY guard（行 `[ -t 1 ] || [ -t 0 ]`）
- `gh auth status`（非 `gf`）→ 与 skill 强制 gf 的冲突
- banner 文本引用 `gitflow-autoreport-bug`（过时名，行 11/133）vs 路径 `gf-autoreport-bug`（行 134）
- auth cache 机制（`.cache/auth-cache/{platform}.ttl`，TTL 86400）
- 读取 `hooks/tests/auto-report-bug.bats` 记录已覆盖的测试场景（无 pending / .invalid / auth fail / banner / cache hit）

- [ ] **Step 4: 逐环取证 — 处理端 + 支撑文档**

读取 `.claude/skills/gf-autoreport-bug/SKILL.md` 与 `skills/gf-autoreport-bug/SKILL.md` 全文，逐项核对：description（触发条件导向？）、结构章节、职责边界、去重命令（`--search` vs params doc `--state all` 不一致）、Issue 创建命令（`--label "auto-report"`）。核对两副本是否一致（`diff`）。

运行 `gf label list --repo byx-darwin/gitflow-cli` 验证 **`auto-report` label 是否存在**，记录输出。

- [ ] **Step 5: 固化为 evidence_baseline.md**

写入 `docs/superpowers/plans/evidence-baseline-wf-2026-08-18-005.md`，每条证据带 🔴/🟠/🟢 状态与 `file:line`/命令输出锚点。**验证：** 报告不包含任何无锚点的主张；每条可直接溯源。

---

### Task 2: 角色评估 — 产品负责人 / 架构师

**Files:**
- Read: `docs/superpowers/plans/evidence-baseline-wf-2026-08-18-005.md`
- Read: `.cache/bug-reports/pending.json`（实际积压实例）
- Create: `docs/superpowers/plans/role-eval-pm-architect.md`

**Interfaces:**
- Consumes: Task 1 `evidence_baseline.md`
- Produces: 产品负责人 + 架构师两角色评估结论（含证据引用）

- [ ] **Step 1: 产品负责人评估**

基于证据回答：
- 功能是否兑现设计意图（「收集真实 CLI 缺陷 → 自动化 Issue」）？
- 实际 pending.json 是用户输入错误（`Invalid state 'invalid'`）→ 误报判定；这对用户信任/社区噪音的影响？
- 「零产出」对价值兑现的判定权重？
- 判定：OK / 有条件 OK / NOT OK + 一句话理由。

- [ ] **Step 2: 架构师评估**

基于证据回答：
- 模块边界（写入端/捕获端/处理端）是否清晰？职责是否单一？
- 触发链确定性：matcher `gitflow` → banner → 模型自主加载 skill，几跳非确定性？
- 状态管理：`pending.json` 单文件覆盖写、`failed.log`、`.invalid` 重命名的健壮性？
- 平台可扩展性：`gf` 三平台设计 vs hook 内硬编码 `gh` 的矛盾？
- 判定 + 理由。

- [ ] **Step 3: 固化角色结论**

写入 role-eval 文件，每条结论带证据锚点。**验证：** 结论可溯源；无误报/臆测表述。

---

### Task 3: 角色评估 — DevOps / 开源社区运营 / 终端用户

**Files:**
- Read: `docs/superpowers/plans/evidence-baseline-wf-2026-08-18-005.md`
- Create: `docs/superpowers/plans/role-eval-devops-community-user.md`

**Interfaces:**
- Consumes: Task 1 `evidence_baseline.md`
- Produces: DevOps + 社区运营 + 终端用户三角色评估结论（含证据引用）

- [ ] **Step 1: DevOps 评估**

基于证据回答：
- 端到端可靠性：从未产出 Issue 的根因定位（触发链 / label 缺失 / 认证口径）？
- 幂等性：去重（`--search "[auto-report] {command} {error_code}"`）在零历史上是否有效？重复触发风险？
- 失败恢复：创建失败保留 pending + failed.log；重试机制？
- 判定 + 理由。

- [ ] **Step 2: 开源社区运营评估**

基于证据回答：
- 若功能正常工作，Issue 质量如何（`[auto-report]` 前缀、错误码、模板）？
- 误报（用户输入错误被上报）对维护者噪音的影响？
- 去重依赖 `command + error_code`，粒度是否合理（同命令不同场景合并）？
- 判定 + 理由。

- [ ] **Step 3: 终端用户评估**

基于证据回答：
- 打扰度：功能在非交互模式静默写入，用户是否感知？
- 可控性：`co_contribution` gate 是否可发现、可撤销？（全局 settings 为 true，项目无；文档是否说明？）
- 隐私安全：token/家目录脱敏、`0o600`、best-effort 不阻塞退出码 → 正面证据。
- 判定 + 理由。

- [ ] **Step 4: 固化角色结论**

写入 role-eval 文件。**验证：** 结论可溯源；终端用户安全结论引用脱敏正则与 `0o600` 具体行号。

---

### Task 4: 综合判定 + 分级改进建议

**Files:**
- Read: Task 2/3 角色评估产物
- Create: `docs/2026-08-18-autoreport-bug-multi-role-eval-report.md`（最终交付物）

**Interfaces:**
- Consumes: Task 1 evidence_baseline + Task 2/3 角色结论
- Produces: 最终评估报告（含总体判定 + P0/P1/P2 建议表）

- [ ] **Step 1: 汇总五角色判定**

合并 Task 2/3 五个角色的 OK/有条件 OK/NOT OK 结论，统计并交叉引用。识别所有角色一致认同的缺陷（高置信）。

- [ ] **Step 2: 形成总体判定**

按设计文档 §6 判定标准给出总体判定，并给出 3-5 条核心理由（每条带证据）。

- [ ] **Step 3: 分级改进建议**

将全部缺陷整理为建议表，每条含：`[P0/P1/P2] 建议 | 归属环节 | 证据 | 期望效果`。

分级规则：
- **P0（阻断正确性）**：端到端从未产出（触发链断裂）、`--label auto-report` 缺失导致创建失败、误报系统性
- **P1（重要）**：`gh` vs `gf` 认证口径不一、banner 过时技能名、去重命令不一致、`co_contribution` 不可发现
- **P2（打磨）**：description 触发条件导向、Red Flags/When to Use 结构、token 效率、skill 副本同步

- [ ] **Step 4: 写最终报告**

写入 `docs/2026-08-18-autoreport-bug-multi-role-eval-report.md`（完整报告）。**验证：** 报告含五角色判定表、总体判定、P0/P1/P2 建议表、全部证据可溯源；无「待核实」悬空项（若有则标注由谁核实）。

---

### Task 5: 交付报告 + Issue #209 评论

**Files:**
- Read: `docs/2026-08-18-autoreport-bug-multi-role-eval-report.md`
- Modify: 无（不改代码/skill）
- Create: `/tmp/issue-209-report-summary.md`（临时）

**Interfaces:**
- Consumes: Task 4 最终报告
- Produces: Issue #209 评论（评估摘要 + 报告路径）

- [ ] **Step 1: 生成评论摘要**

从最终报告提炼：总体判定、五角色判定表、Top 改进建议（P0 全部 + P1 代表）、报告路径。写入 `/tmp/issue-209-report-summary.md`。

- [ ] **Step 2: 发评论（需用户确认）**

展示摘要，获用户确认后执行 `gf issue comment 209 --body-file /tmp/issue-209-report-summary.md`。

- [ ] **Step 3: 清理临时文件**

`rm -f /tmp/issue-209-report-summary.md`。**验证：** 评论 id 返回；无残留临时文件。

---

## Self-Review（写后自检）

**Spec 覆盖：**
- §1 评估目标 → Task 4 总体判定 ✅
- §2 全链路范围 → Task 1 逐环取证 ✅
- §3 5角色×3层面 → Task 2（产品/架构）+ Task 3（DevOps/社区/用户）✅（3 层面已在各任务中按子项覆盖：文档规范性、职责边界、可测试性&可靠性）
- §4 证据基线 → Task 1 固化 + 新证据（label 缺失、banner 过时名）✅
- §5 交付物 → Task 4 报告 + Task 5 Issue 评论 ✅
- §6 判定标准 → Task 4 Step 2 ✅

**类型一致性：** `evidence_baseline.md` 路径在 Task 1 产出、Task 2/3 消费，命名一致；`role-eval-*.md` 在 Task 2/3 产出、Task 4 消费，路径一致。

**无占位符：** 每个 Step 均含具体动作与验证方式，无「填写细节」「适当处理」类表述。

**注意：** 本任务为**仅评估**任务，Phase 3 执行阶段不使用传统 TDD（无代码产出）。「测试」即每个 Step 末尾的**验证**（证据可溯源检查）。这与标准模式下 subagent-driven-development 的 TDD 强制要求不冲突——评估任务的「RED」是「结论必须有证据」，「GREEN」是「结论有证据可溯源」。
