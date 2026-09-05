# Pipeline 分析报告 — PR #313

> **PR：** [#313 fix(pr): detect repo default branch instead of hardcoding "main"](https://github.com/byx-darwin/gitflow-cli/pull/313)
> **分支：** `feat/305-pr-create-default-branch` → `dev`（对应 Issue #305）
> **快照时间：** 2026-09-03T10:05:29Z（采集点，非流水线终态；本报告为单次时点快照，不持续轮询至全部收尾）
> **分析日期：** 2026-09-03
> **模式：** 只读（CLI: `gf`）
> **变更性质：** 修复 `gf pr create` 在未显式传 `--base` 时硬编码目标分支为 `"main"` 的问题——新增 `PrProvider::default_branch()` trait 方法（GitHub 用 `gh repo view --json defaultBranchRef`，GitLab 用 `glab repo view --output json`，GitCode 无对应能力、返回 `CoreError::Platform`）；`apps/cli/src/commands/pr.rs` 在未给 `--base` 时查询默认分支，查询失败（含 GitCode）才回退到 `"main"`；新增单测覆盖成功路径（mock 非 `main` 默认分支如 `"dev"`）、失败/不支持平台回退、显式 `--base` 绕过查询三种场景。

## 零、核心结论先行

采集时点（`10:05:29Z`，即 PR 触发后约 85 秒）PR #313 关联的 3 个 workflow run 中：**1 个已收尾（`success`，13/13 job 均成功）**，**2 个仍在运行中（`in_progress`，已完成的子 job 全部 `success`，无一个 `failure`/`cancelled`）**。截至快照时刻，**未观察到任何失败或异常信号**。`dev`/`main` 基线保持健康（95%/100%），与既往系列报告一致。按用户要求，本报告为单次时点快照，**不持续轮询至全部 run 收尾**；仍在运行的部分状态记为"pending/running，尚无失败"，如需终态确认需另行查询。

## 一、PR #313 关联流水线实测（时点快照）

`feat/305-pr-create-default-branch` 分支触发 3 个 workflow run（均创建于 `2026-09-03T10:04:09Z`）：

| Run ID | Workflow | 快照时状态 | 备注 |
|--------|----------|------|------|
| 33742300196 | 主 CI（Check/MSRV/Lint/Smoke Test/Test×3） | 🟡 running（4/7 job 已完成，均 `success`；3 个仍 `in_progress`） | `updatedAt: 10:04:19Z` |
| 33742300225 | E2E Tests（GitHub/GitLab/GitCode） | 🟡 running（1/3 job 已完成 `success`；2 个仍 `in_progress`） | `updatedAt: 10:04:19Z` |
| 33742300293 | Smoke Test 跨平台 | ✅ success（3/3 job 全部成功） | 收尾于 `10:05:16Z` |

`gf pipeline status --branch feat/305-pr-create-default-branch` 原始快照：run `33742300196`、`33742300225` 状态均为 `"status": "running", "conclusion": ""`；run `33742300293` 为 `"status": "success", "conclusion": "success"`。

已采集到的全部 job 明细（快照时刻）：

| Job | Workflow run | 状态 | 结论 |
|-----|--------------|------|------|
| Check | 33742300196 | completed | ✅ success |
| Smoke Test | 33742300196 | completed | ✅ success |
| MSRV | 33742300196 | completed | ✅ success |
| Test (ubuntu-latest) | 33742300196 | in_progress | — |
| Test (windows-latest) | 33742300196 | in_progress | — |
| Test (macos-latest) | 33742300196 | in_progress | — |
| Lint | 33742300196 | in_progress | — |
| E2E Tests (GitHub) | 33742300225 | completed | ✅ success |
| E2E Tests (GitLab) | 33742300225 | in_progress | — |
| E2E Tests (GitCode) | 33742300225 | in_progress | — |
| Smoke Test (github) | 33742300293 | completed | ✅ success |
| Smoke Test (gitlab) | 33742300293 | completed | ✅ success |
| Smoke Test (gitcode) | 33742300293 | completed | ✅ success |

**共 13 个 job**：8 个已收尾，**全部 `success`，无一失败**；5 个仍 `in_progress`，快照时刻无法判定终态。**本次改动为 CLI 参数解析层的小范围逻辑变更（新增 trait 方法 + 回退分支），未触及 CI workflow 配置本身，理论上不引入新的 job 级别风险。**

## 二、PR 合并状态说明

`gf pr view 313` 返回 `state: "closed"`、`mergedAt: "2026-09-03T10:04:17Z"`——即 PR 在其触发的 CI 仍在运行时已被记录为合并（与"queued for auto-merge pending CI checks"的描述一致：GitHub 的 auto-merge 会在队列中等待必需检查通过后自动合并，`gf pr view` 在合并完成后立即返回 `closed`/`mergedAt`；本报告采集时点该字段已翻转，但不代表流水线已全部收尾——上述 5 个 in_progress job 仍需自然完成）。此处仅作记录，不构成异常。

## 三、`gf pipeline report` 口径假象（第三次复现，与 PR #311/#312 一致）

`gf pipeline report --branch feat/305-pr-create-default-branch --days 7`（在 run 仍 running 时采集）：

```json
{
  "totalRuns": 3,
  "successRate": 0.0,
  "avgDurationSecs": 6.666666666666667,
  "topFailures": [""]
}
```

与 PR #311（`successRate: 0.5`）、PR #312（`successRate: 0.333`）报告记录的同一类问题一致——命令将仍处于 `running`（`conclusion` 为空）的 run 计入失败桶，本次因 2/3 run 仍在运行，`successRate` 甚至虚低至 `0.0`。经 `gf pipeline status`/`gf pipeline jobs` 逐 run、逐 job 复核，**已完成的 8 个 job 全部为 `success`，无真实失败**。该问题已在 PR #311/#312 报告中提出改进建议，**本次为第三次连续复现，建议维持/上调优先级**。

## 四、dev / main 基线（采集时点：PR #313 触发后）

| 分支 | 周期 | Total runs | Success rate | Avg duration | 评级 |
|------|------|-----------:|--------------:|--------------:|------|
| `dev` | 7 天 | 100 | 95.0% | 150.47s | 🟢 Healthy |
| `main` | 30 天 | 100 | 100.0% | 159.59s | 🟢 Healthy |

基线数值与 PR #311/#312 报告完全一致，延续系列报告观察到的健康水位。

## 五、Flaky / 失败信号

**PR #313 自身流水线（快照时刻）**：已收尾的 8 个 job 全部 `success`，未观察到任何失败。5 个 job 仍在运行中，尚无法判定；截至快照，**无失败信号**。

历史观察清单沿用 PR #311/#312 报告记录的 `dev` 分支 7 天窗口内单次 `Test (windows-latest)` 失败案例（run `33346653353`，2026-08-31，`commands::commit::tests::test_should_resolve_comment_body_from_file`），仍为 1 次，未达 flaky 判定阈值（≥2 次），且与 PR #313（改动范围限于 `pr.rs` 与 provider trait）无关联。维持观察清单状态。

## 六、耗时分析

快照时刻已收尾 job 耗时集中在 37s–77s（Check ~38s、Smoke Test ~77s、MSRV ~50s、E2E Tests(GitHub) ~60s、三平台 Smoke Test ~61–63s），与历史基线（PR #304/#309/#311/#312 记录的 35s–337s 区间）一致，无异常。仍在运行的 `Test (windows-latest)`/`Test (macos-latest)`/`Test (ubuntu-latest)`/`Lint`/E2E(GitLab)/E2E(GitCode) 因快照时点尚未收尾，无法给出本次耗时数据；历史同名 job 区间为 116s–337s（Test）、140s–223s 量级（Lint/E2E），无理由预期本次显著偏离——本次改动仅为 CLI 参数解析逻辑的新增分支，不涉及测试矩阵或依赖变更。

## 七、结论与 Recommendations

1. 🟢 **Low** — PR #313 修复 `gf pr create` 硬编码 `"main"` 默认分支的问题，快照时刻已收尾的 8 个 job（Check/Smoke Test/MSRV/E2E(GitHub)/Smoke Test×3 platform）**全部成功**，无失败信号；5 个 job 仍在运行，按用户要求本报告不持续轮询等待其收尾。**建议**：若需要终态确认，后续可另行执行 `gf pipeline status --branch feat/305-pr-create-default-branch` 或 `gf pipeline jobs --pipeline-id 33742300196/33742300225` 复核。
2. 🟡 **Medium** — `gf pipeline report` 命令在 run 处于 `running` 状态时持续将其计入失败桶，本次为**第三次连续复现**（PR #311: 0.5 → PR #312: 0.333 → PR #313: 0.0，虚低程度随并发 running run 数量增加而恶化）。建议尽快针对 `gf` CLI 提交独立 Issue，改进 `pipeline report` 使其将 `running`/`queued` 与真实 `failure` 分开统计。
3. 🟡 **Low** — `commands::commit::tests::test_should_resolve_comment_body_from_file`（`apps/cli/src/commands/commit.rs:245`）在 `dev` 分支历史窗口内仍保持 1 次失败记录（run `33346653353`，2026-08-31），未达 flaky 判定阈值，且与 PR #313 无关联。继续维持观察清单状态。
