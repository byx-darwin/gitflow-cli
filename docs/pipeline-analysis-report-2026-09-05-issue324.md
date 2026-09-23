# Pipeline Analysis Report — dev 分支基线快照（Issue #324 本地合并跟进）

- **报告日期**：2026-09-05
- **分析目标分支**：`dev`
- **触发背景**：Issue #324（`gf pipeline report` 把 `running`/`queued` 状态的 run/job 误计入失败桶）已通过**本地 squash merge**方式合入 `dev`，提交 `50519664c3c0fdf723d224c5b42d6b81ea3bb778`，**未走 PR 流程**
- **分析范围**：只读分析（`gf pipeline report` / `gf pipeline status` / `gf pipeline jobs`），未触发/重跑/取消任何流水线，未修改代码、未提交、未推送
- **数据来源**：`gf` CLI（GitHub 平台）

## 一、关键前提核实：该 commit 尚未触发 CI

```
$ git rev-parse dev origin/dev
50519664c3c0fdf723d224c5b42d6b81ea3bb778   # dev（本地）
f35bd035442f09b319ef63e40f795650564cba5f   # origin/dev（远端）
```

本地 `dev` 领先 `origin/dev` 一个提交（`5051966`），**尚未 push**。GitHub Actions 仅对已推送的 ref 触发工作流，因此：

- **该 commit 目前没有对应的 CI run**，`gf pipeline status --branch dev` 返回的最新记录（run `33952435574`，2026-09-05T07:23:52Z）早于该 commit 的提交时间（2026-09-05T22:35:40+08:00 = 14:35:40Z），确认二者无关。
- 按用户要求，本报告改为对 `dev` 分支近期整体健康度做**基线快照**，作为该 commit push 后对比的参照系；commit 推送并触发 CI 后建议追加一次针对性分析。

## 二、dev 分支健康度基线（三维度）

### 2.1 成功率趋势

| 分支 | 周期 | Total runs | Success rate | Avg duration | 评级 |
|------|------|-----------:|--------------:|--------------:|------|
| `dev` | 7 天 | 95 | 95.79% | 151.64s | 🟢 Healthy |
| `dev` | 14 天 | 100 | 95.00% | 149.36s | 🟢 Healthy |

数值与 PR #311→#323 系列报告观测的历史水位（94.9%~95.8%）**完全一致**，无异常抖动，成功率评级维持 🟢 Healthy 未发生变化。

### 2.2 失败模式

- 14 天窗口 `topFailures`：`Lint`、`Test (windows-latest)`；7 天窗口收窄为仅 `Test (windows-latest)`。
- 追查 14 天窗口内唯一失败 run（`33346653353`，2026-08-31T01:08Z）：抽查其 job 明细，仅 `Test (windows-latest)` 一个 job `conclusion: failure`，其余 6 个 job（`Lint`/`Test (ubuntu-latest)`/`Check`/`Test (macos-latest)`/`Smoke Test`/`MSRV`）均 `success`。同批次另有两个并行 run（`33346653298`、`33346653261`）全部成功，说明这是**孤立的单次失败**，非该次提交本身系统性问题。
- 未观测到 ≥3 次连续失败的持久性模式，也未观测到"时而成功时而失败"的经典 flaky 特征（14 天内仅此一例失败），暂不判定为 flaky test，建议继续观察。

### 2.3 耗时分布

- 平均耗时 149~152s，与既往系列报告（PR #320/#321/#323 记录的 150~151s）基本持平，无劣化趋势，未见耗时瓶颈。

## 三、重要关联事项：Issue #324 修复的是已连续升级 9 次的已知缺陷

`5051966` 提交信息明确写明：

> Reproduced across 9 consecutive pipeline analysis reports (PR #311-#323).

这与本技能在 PR #321、#323 报告中记录的**升级项**完全对应——`gf pipeline report` 曾因 `conclusion.is_some()` 误判非终态 run/job 为失败，连续 9 次报告（PR #311→#323）复现且此前始终未有补救措施，已按升级规则从"观察项"升级为"阻断性建议"。

`5051966` 的修改内容（新增 `PipelineStatusEnum::is_terminal()`，在 run/job 两级均改为基于 `status == "completed"` 而非 `conclusion` 是否存在来判定终态）从代码层面直接对应此前 9 次报告描述的根因，**是该升级项的候选修复**。

**但此修复尚未经过 CI 验证**（因未推送，见第一节），因此：

- 升级链条（PR #311→#323，共 9 次复现）尚不能标记为"已解决"，只能标记为"**修复已在本地就绪，等待推送与远端 CI 验证**"。
- 建议：push 后立即用 `gf pipeline report` 对一个仍在 `running`/`queued` 状态的 run 做实测快照，确认不再被误计入失败桶，再正式关闭该升级项。

## 四、结论与 Recommendations

1. 🟢 **无阻断性发现（基线健康度）** — `dev` 分支 7/14 天成功率均为 🟢 Healthy（95.0%~95.8%），耗时无劣化，与历史水位一致。
2. 🟡 **Low（观察项，非新增）** — 14 天窗口内 1 次 `Test (windows-latest)` 孤立失败（2026-08-31），同批次并行 run 均成功，判定为单次偶发而非持久性/flaky 模式，建议继续观察，暂不需要单独立案。
3. ⚠️ **Medium（流程提醒，非 CI 健康度问题）** — Issue #324 的修复提交 `5051966` 通过本地 squash merge 合入 `dev` 但**尚未推送到 `origin/dev`**，因此没有对应的 CI run 可供验证。这不是流水线故障，而是交付流程未完成：建议尽快 push 该提交并观察其触发的 CI 结果，确认修复生效后再视为完整交付。
4. ℹ️ **信息性说明（延续 PR #311→#323 升级链条的状态更新）** — 本次分析确认 `5051966` 的代码改动直接针对此前连续 9 次报告描述的根因（`conclusion.is_some()` 误判非终态状态）。在 push 并经 CI 实测验证之前，该升级项状态更新为"修复已就绪、待验证"，尚不能关闭；验证通过后可在下一份报告中正式标记为已解决并终止升级链条。

## 五、原始数据

```json
// gf pipeline report --branch dev --days 7
{"totalRuns":95,"successRate":0.9578947368421052,"avgDurationSecs":151.6421052631579,"topFailures":["Test (windows-latest)"]}

// gf pipeline report --branch dev --days 14
{"totalRuns":100,"successRate":0.95,"avgDurationSecs":149.36,"topFailures":["Lint","Test (windows-latest)"]}
```

## 六、Escalation Rule 核对

- 成功率评级：连续多轮报告均为 🟢 Healthy，未发生同一劣化水位连续 ≥3 次无补救的情况，**不触发**评级维度的升级。
- `gf pipeline report` 误判 running/queued 为失败：此前已在 PR #323 报告中升级（9 次复现）。本次分析确认**补救措施已在本地代码落地**（`5051966`），链条尚未正式终止（待推送+CI 验证），本报告不重复升级措辞，仅做状态更新（见第三、四节）。
