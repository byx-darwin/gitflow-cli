# Pipeline 分析报告 — PR #309

> **PR：** [#309 ci(e2e): alert on scheduled e2e regression failure](https://github.com/byx-darwin/gitflow-cli/pull/309)
> **分支：** `feat/292-e2e-failure-alert` → `dev`（对应 Issue #292）
> **合并时间：** 2026-09-03T07:50:03Z
> **分析日期：** 2026-09-03
> **模式：** 只读（CLI: `gf`）

## 零、核心结论先行

PR #309 触发的 3 个 workflow run **全部成功**（`successRate: 1.0`），13 个 job 全绿，无失败、无 flaky 信号。本 PR 新增的 `.github/workflows/e2e-tests.yml::notify-on-schedule-failure` job 在本次 `pull_request` 触发的运行中被正确评估为 `skipped`（其条件 `github.event_name == 'schedule' && contains(needs.*.result, 'failure')` 未满足），证明 YAML 语法有效、job 依赖（`needs: [e2e-github, e2e-gitlab, e2e-gitcode]`）与条件表达式解析正常，未在 Actions 侧引入语法或调度异常。所需的 `e2e-regression` label 已存在于仓库中，`issue.write` 所需的 job 级 `permissions` 也已按最小权限声明。**总体判定：无异常发现，PR #309 是一次干净的 CI 配置变更。**

## 一、PR #309 关联流水线实测

`feat/292-e2e-failure-alert` 分支触发 3 个 workflow run（均创建于 `2026-09-03T07:49:03Z`，合并发生于 `07:50:03Z`）：

| Run ID | Workflow | 结论 | 耗时（最慢 job） |
|--------|----------|------|-------------------|
| 33730059050 | Smoke Test 跨平台 | ✅ success（3 job 全部成功） | ~64s |
| 33730059103 | E2E Tests（GitHub/GitLab/GitCode + 新增 notify job） | ✅ success（4 job：3 成功 + 1 skipped） | ~206s |
| 33730059129 | 主 CI（Check/MSRV/Lint/Smoke Test/Test×3） | ✅ success（7 job 全部成功） | ~280s |

`gf pipeline report --branch feat/292-e2e-failure-alert --days 7`：

```json
{
  "totalRuns": 3,
  "successRate": 1.0,
  "avgDurationSecs": 184.0,
  "topFailures": []
}
```

已收尾 job 明细（13 个 job：13 成功，其中 1 个按预期 skipped）：

| Job | Workflow run | 耗时 | 结论 |
|-----|--------------|------|------|
| Check | 33730059129 | 39s | ✅ success |
| MSRV | 33730059129 | 55s | ✅ success |
| Smoke Test | 33730059129 | 73s | ✅ success |
| Lint | 33730059129 | 139s | ✅ success |
| Test (ubuntu-latest) | 33730059129 | 117s | ✅ success |
| Test (macos-latest) | 33730059129 | 160s | ✅ success |
| Test (windows-latest) | 33730059129 | **280s** | ✅ success（本轮最慢 job，与历史基线量级相符，非异常） |
| Smoke Test (gitlab) | 33730059050 | 57s | ✅ success |
| Smoke Test (github) | 33730059050 | 59s | ✅ success |
| Smoke Test (gitcode) | 33730059050 | 60s | ✅ success |
| E2E Tests (GitHub) | 33730059103 | 55s | ✅ success |
| E2E Tests (GitLab) | 33730059103 | 146s | ✅ success |
| E2E Tests (GitCode) | 33730059103 | 199s | ✅ success |
| **Notify on Scheduled Regression Failure**（新增） | 33730059103 | 0s | ✅ **skipped**（预期行为，见下） |

## 二、新增 `notify-on-schedule-failure` job 验证

**触发条件正确性**：本次运行由 `pull_request` 事件触发（非 `schedule`），job 的 `if: always() && github.event_name == 'schedule' && contains(needs.*.result, 'failure')` 条件评估为假，Actions 将其正确标记为 `skipped`（`completedAt` 与 `startedAt` 相同，`07:52:25Z`，即无实际执行耗时）——这与 PR 描述"该 job 仅在 schedule 触发时运行"的设计意图完全一致，**说明表达式语法有效，未阻塞或影响同一 workflow 内其余 3 个 job 的正常执行**。

**结构性检查**（源码走读 `origin/dev:.github/workflows/e2e-tests.yml` 第 202–245 行）：
- `needs: [e2e-github, e2e-gitlab, e2e-gitcode]` 正确引用了同 workflow 内已存在的 3 个 job id，无拼写错误。
- `permissions: { contents: read, issues: write }` 采用 job 级最小权限声明，未提升 workflow 顶层权限，符合最小权限原则；`issues: write` 足以支撑 `gh issue create`/`gh issue comment`。
- 去重逻辑（`gh issue list --label e2e-regression --search "in:title 定时 E2E 回归失败"` → 存在则 `comment`，否则 `create`）复用了 `upstream-patrol.yml` 已验证过的范式。
- 所需 label `e2e-regression` 已存在于仓库（`gf label list` 确认），首次真实触发时不会因 label 缺失而报错。

**尚未验证的部分（设计使然，非本次可测范围）**：该 job 仅在每周一 `02:00 UTC` 的 `schedule` 触发下才会真正执行到 `Create or update regression issue` 步骤，本次 PR/push 触发的运行链路中始终停在 `if` 条件判断即 `skipped`，因此 `gh issue create`/`comment` 的实际调用（含 `GH_TOKEN` 是否具备跨仓库写权限、`gh issue list --search` 的语法有效性等）尚未被真实执行路径覆盖，需等待下一次 `schedule` 触发（2026-09-08 前后）或维护者手动 `workflow_dispatch` 一次全平台失败场景来验证。

## 三、dev / main 基线（7 天 / 30 天，采集时点：PR #309 合并后）

| 分支 | 周期 | Total runs | Success rate | Avg duration | 评级 |
|------|------|-----------:|--------------:|--------------:|------|
| `dev` | 7 天 | 100 | 95.0% | 150.47s | 🟢 Healthy |
| `dev` | 30 天 | 100 | 95.0% | 150.47s | 🟢 Healthy |
| `main` | 30 天 | 100 | 100.0% | 159.59s | 🟢 Healthy |

两个周期样本量均已达窗口上限 100，基线延续此前报告（PR #304/#306 系列）观察到的健康水位，PR #309 未引入新的失败样本。

**合并后 dev 分支无新增 workflow run**：`gf pipeline status --branch dev` 显示合并时间 `07:50:03Z` 之后无新记录（最新一条创建于 `07:46:02Z`，早于合并）。这是**预期行为**而非异常——`.github/workflows/e2e-tests.yml` 的 `push` 触发仅限 `branches: [main]`，且 `ci.yml` 此前已通过提交 `da1f2b9`（"stop re-running the full matrix on every push to dev"）停止对 `dev` push 的全量矩阵重跑；PR 的 `pull_request` run 已覆盖同一 commit，故合并动作本身不会、也不应触发额外 run。

## 四、耗时分析

未见与本次变更相关的耗时异常。`Test (windows-latest)` 280s 与 `E2E Tests (GitCode)` 199s 与历史基线量级一致（对照 PR #304/#306 报告记录的 172s–288s 区间）；新增的 `notify-on-schedule-failure` job 本身耗时 0s（skipped），对整体 workflow 耗时无影响。

## 五、Flaky 信号

本次采集范围内**未发现 flaky test**，3 个 workflow run 全部一次性成功，无需重跑或观察间歇性失败。

## 六、结论

- PR #309 关联的 3 个 workflow run、13 个 job **全部成功**，无失败、无 flaky。
- 新增的 `notify-on-schedule-failure` job 在本次 `pull_request` 触发路径下按设计正确 `skipped`，证明 YAML 语法有效、`needs`/`if` 表达式解析正常、job 级权限声明到位，未对同一 workflow 内其余 job 造成任何阻塞或副作用。
- `dev`/`main` 历史基线均为 🟢 Healthy（95.0%/100.0%，均 100 次运行样本），PR #309 未引入新的失败样本，也未产生额外的 dev 分支 workflow run（符合既有的 push-trigger 收窄设计）。
- 唯一的剩余风险点是**该 job 的真实执行路径（`schedule` 触发 + 存在失败平台）尚未被任何一次 run 实际覆盖过**，这是设计使然（每周一次触发），不构成本次交付的阻塞项，但建议在下一次定时触发后做一次实测确认（或由维护者手动 `workflow_dispatch` + 临时构造失败条件验证一次）。

## 七、Recommendations

1. 🟢 **Low** — 待下一次 `schedule` 触发（下周一 02:00 UTC）或一次人工验证后，确认 `gh issue create`/`gh issue comment` 分支在真实失败场景下按预期工作（PR 描述中已做过 `env -i` + mocked `gh` 的隔离 dry-run，但线上 `GH_TOKEN` 实际权限与 `gh issue list --search` 语义仍建议做一次真实环境复核）。
2. 🟢 **Low** — 无其他行动项；本次变更范围小（纯 CI 配置新增），未触及 Rust 源码，风险面很窄，当前状态可视为已交付完成。
