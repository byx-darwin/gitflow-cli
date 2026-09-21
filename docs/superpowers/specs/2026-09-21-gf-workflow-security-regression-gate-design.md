# gf-workflow: gf-security-check / gf-regression 阻断式接入设计

> **Issue:** #344
> **Workflow:** wf-2026-09-21-004
> **Date:** 2026-09-21
> **Status:** Approved (in-chat, section by section)

## 1. Context

`skills/gf-security-check/` 与 `skills/gf-regression/` 目前游离于 `gf-workflow` 四阶段编排之外，只能人工触发，实际交付流程里从未自动发生过。同类 skill（`gf-quality`、`gf-pr-review`、`gf-pipeline-analyzer`）在 `gf-workflow` 的阶段定义中都有明确归属和触发条件；这两个 skill 没有。

## 2. Goal

把两个 skill 接入 `gf-workflow` 编排，按改动面条件触发，阻断式：命中触发条件时必须通过检查（或显式豁免并写明理由）才能继续交付。

## 3. Non-Goals

- 不改变 `gf-security-check`、`gf-regression` 自身的实现逻辑——只改它们何时/如何被 `gf-workflow` 调用。
- 不引入新的 Phase（仍是四阶段），不改变 Phase 4 Step Matrix。
- 不做语义级 diff 分析（例如"这次改动是否真的改变了公共 API 签名"）——用路径规则做粗粒度判定，与 `gf-quality` 的语言检测同一惯例。

## 4. Insertion Point

新增 **Phase 3 Step 3**，插在现有 Step 2（执行引擎产出实现）之后、现有 Step 3（symlink 守卫 + 交付选择）之前。原 Step 3–7 顺延为 Step 4–8。

```
Step 1: worktree preflight
Step 2: 执行引擎（TDD 实现）
Step 3【新增】: 改动面检测 → 条件阻断 gf-security-check / gf-regression
Step 4（原3）: symlink 守卫 + 交付选择（本地合并 / PR）
Step 5（原4）: make test / cargo test
Step 6（原5）: 排队合并
Step 7（原6）: 更新 contract
Step 8（原7）: Gate 3→4
```

**理由**：local_merge 路径下，现 Step 3 会立刻把分支合并进 `base_branch`——检查必须在这之前完成，否则"阻断式"名不副实。

**模式适用范围**：Step 3 对 full/standard/fast 三种模式统一生效，不进 Phase 4 Step Matrix（那张表管的是"按模式开关"的报告步骤；这里是"按改动面触发"的阻断步骤，与模式无关——fast 模式做一次依赖升级同样要被拦下）。

## 5. Change-Surface Detection

复用 `gf-quality` 已修好的 base-ref 解析惯例，直接用 Phase 3 Step 1 已记录的 `base_branch`（不重新猜测，避免 GitLab/GitCode 默认分支不是 `main` 的老问题）：

```bash
diff_files="$(git diff --name-only "$base_branch"...HEAD)"
```

两条独立规则，各自判定：

| 检查 | 触发路径模式 |
|---|---|
| `gf-security-check` | `Cargo.toml`、`Cargo.lock`、`**/Cargo.toml`（workspace 内任意 crate）、`deny.toml` |
| `gf-regression` | `apps/cli/src/**`、`crates/core/src/**`、`crates/github/src/**`、`crates/gitlab/src/**`、`crates/gitcode/src/**` |

都不命中（例如纯文档/spec/skill 文本改动）→ `change_surface = "docs_only"`，两项都不跑，直接跳过整个 Step 3。

## 6. Blocking Behavior & Exemption

对命中的每一项检查，顺序执行、inline（[AUTO]，不走 Phase 4 那种 Agent 并行派发——这一步需要同步阻断并可能立刻与用户交互，和 Phase 2 的 `gf-quality` gate 是同一惯例）：

```
命中 → ✋ PAUSE：展示匹配到的文件列表 + 即将运行的检查名
       用户选择：① 运行（默认） ② 跳过（必须填写理由，理由为空视为无效跳过）
① 运行 → 调用对应 skill → 判定：
    gf-security-check: 发现 Critical/High（未修补 CVE、硬编码密钥）→ FAIL
    gf-regression: smoke test 出现非预期 FAIL（非已知 flaky）→ FAIL
    FAIL → 阻断，不进入 Step 4（交付选择）；提示用户回到 Step 2 修复后重跑本 Step
    PASS → 继续
② 跳过 → 记录 exemption_reason，继续（不再运行该检查）
```

没有"静默跳过"路径：跳过必须显式选择且理由非空，否则视为无效、维持默认（运行）。

## 7. Contract Evidence Schema

`phases.3.evidence` 新增字段（不覆盖既有的 `branch`/`base_branch`/`worktree_path` 等）：

```json
{
  "change_surface": {
    "diff_files_sample": ["Cargo.lock", "apps/cli/src/commands/pr.rs"],
    "security_triggered": true,
    "regression_triggered": true
  },
  "security_check": {
    "status": "passed | failed | exempted | not_triggered",
    "report_path": "docs/security-report-<issue>-<date>.md",
    "exemption_reason": null
  },
  "regression_check": {
    "status": "passed | failed | exempted | not_triggered",
    "report_path": "docs/regression-report-<issue>-<date>.md",
    "exemption_reason": null
  }
}
```

`not_triggered` 区分"改动面未命中该规则"与"检查了但通过"（对应 `gf-quality` 里 `N/A` vs `SKIPPED` 的既有区分惯例）。

**Gate 3→4（现 Step 8）** 追加要求：`security_check.status ∈ {passed, exempted, not_triggered}` 且 `regression_check.status ∈ {passed, exempted, not_triggered}`。`failed` 理论上不会带着走到这个 Gate（Step 3 自己已经阻断），这条是防御性一致性检查。

## 8. Report Output & Archiving

两个 skill 目前都没有落盘报告的路径约定，新增（对称于 `gf-review` 已有的 `docs/code-review-report-pr<N>-<date>.md` 归档模式）：

- `gf-security-check` → `docs/security-report-<issue-number>-<YYYY-MM-DD>.md`
- `gf-regression` → `docs/regression-report-<issue-number>-<YYYY-MM-DD>.md`

各自独立计数：超过 5 份时，按文件名里的 issue number 排序，把超出的旧文件移入 `docs/reports-archive/<YYYY>-Q<N>/`（按该报告自己的日期分桶）。写入 `docs/index.md` → Reports Archive 一节，与 `gf-review` 的归档规则完全对称。

## 9. Files to Touch

- `skills/gf-workflow/SKILL.md` — Phase 3 步骤表插入新 Step 3，后续步骤重新编号；State Machine 图不变（仍是四阶段）。
- `skills/gf-workflow/gates.md` — Gate 3→4 的证据要求追加 `security_check.status`/`regression_check.status` 约束。
- `skills/gf-workflow/references.md` — 新增 "Change-Surface Detection" 小节，记录路径规则表 + base-ref 复用惯例；若有 Cross-Session Recovery 表，需要在 Phase 3 行补充新证据字段的加载说明。
- `skills/gf-security-check/SKILL.md`、`skills/gf-regression/SKILL.md` — 补充报告落盘路径约定（`docs/security-report-*.md` / `docs/regression-report-*.md`）及归档策略，与 `gf-review` 对称。
- `docs/index.md` — Reports Archive 一节追加这两类报告的归档规则。

## 10. Acceptance Criteria Mapping

| Issue #344 AC | 本设计对应章节 |
|---|---|
| 两个 skill 在阶段定义中有明确归属 | §4 Insertion Point |
| 依赖/锁文件变更时触发 security-check | §5 |
| CLI 行为变更时触发 regression | §5 |
| 文档类改动不触发 | §5（`docs_only` 分支） |
| 豁免路径有显式记录 | §6 |
| 触发结果写入 workflow 合同 | §7 |
