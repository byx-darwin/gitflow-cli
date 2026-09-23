# Pipeline 分析报告 — PR #326

> **PR：** #326 → 分支 `feat/325-remove-mode1-background-agent` → `dev`（对应 Issue #325，gf-workflow Phase 4，工作流 `wf-2026-09-06-001`）
> **快照时间：** 2026-09-06T00:39:30Z（采集截止时点，见下方“数据完整性说明”）
> **分析日期：** 2026-09-06
> **模式：** 只读（CLI: `gf`，未使用 `gh`）

## 零、核心结论先行

`feat/325-remove-mode1-background-agent` 分支触发 2 个 workflow run，共 10 个 job。截至本报告采集截止时点，**7/10 job 已收尾且全部 `success`**；`Test (macos-latest)`、`Test (ubuntu-latest)`、`Test (windows-latest)` 仍在 `in_progress`，尚无失败迹象。**未发现任何真实 CI 失败、回归或阈值突破。**

**需要升级的发现（延续既往系列）**：`gf pipeline report` 在 run 处于 `running` 状态时快照仍将未收尾 job 计入失败桶，导致成功率虚假偏低（本次两次快照分别为 `0%` 和 `50%`，且首次快照将已经 `success` 的 `Check` job 误列入 `topFailures`）。值得特别注意的是，本仓库近期提交 `5051966 fix(github): gate pipeline report terminal state on run/job status, not conclusion presence` 已尝试修复该问题，但**本次实测确认该修复未能消除问题**——bug 在修复提交合入之后仍然复现。这是该问题第 **10 次** 被记录（PR #311、#312、#313、#315、#316、#317、#320、#321、#323、#326），且是首次记录在“已有修复提交落地”之后仍复现，问题性质从“未修复”升级为“修复无效”。

## 一、PR #326 关联流水线实测

`feat/325-remove-mode1-background-agent` 分支触发 2 个 workflow run（`gf pipeline status --branch` 核对，无第三个 run）：

| Run ID | Workflow | 收尾状态（采集截止时点） | 备注 |
|--------|----------|------|------|
| 34001732266 | Smoke Test 跨平台（github/gitlab/gitcode） | ✅ success（3/3 job 全部成功） | 最先收尾（00:38:14Z） |
| 34001732211 | 主 CI（Check/MSRV/Lint/Smoke Test/Test×3） | 🟡 4/7 job success，3 个仍 `in_progress` | `Test (macos-latest)`、`Test (ubuntu-latest)`、`Test (windows-latest)` 未收尾 |

Job 明细（`gf pipeline jobs` 核对）：

| Job | Workflow run | 结论 | 状态 |
|-----|--------------|------|------|
| Smoke Test (github) | 34001732266 | ✅ success | 已收尾（66s） |
| Smoke Test (gitlab) | 34001732266 | ✅ success | 已收尾（66s） |
| Smoke Test (gitcode) | 34001732266 | ✅ success | 已收尾（66s） |
| MSRV | 34001732211 | ✅ success | 已收尾（53s） |
| Check | 34001732211 | ✅ success | 已收尾（1m29s） |
| Lint | 34001732211 | ✅ success | 已收尾（2m24s） |
| Smoke Test | 34001732211 | ✅ success | 已收尾（2m58s） |
| Test (macos-latest) | 34001732211 | ⏳ 未知 | 仍 `in_progress`（已运行 >3 分钟） |
| Test (ubuntu-latest) | 34001732211 | ⏳ 未知 | 仍 `in_progress`（已运行 >3 分钟） |
| Test (windows-latest) | 34001732211 | ⏳ 未知 | 仍 `in_progress`（已运行 >3 分钟） |

**7/10 job 已收尾，全部 `success`；3 个 job（历史上一贯是全流水线最慢的三个）在报告截止时仍在执行，尚无失败迹象。**

## 二、数据完整性说明

本报告依据协调方明确指示（不等待 CI 全部收尾）在 3 个 `Test (*)` job 仍处于 `in_progress` 时提交。已收尾的 7 个 job 全部 `success`，未收尾的 3 个尚无失败信号，但结论应标注为"截至采集时点未发现失败证据"，而非"已完全确证"。若需最终确证，可后续再次核对 run `34001732211` 的最终状态。

## 三、dev / main 基线

| 分支 | 周期 | Total runs | Success rate | Avg duration | 评级 |
|------|------|-----------:|--------------:|--------------:|------|
| `dev` | 7 天 | 91 | 95.6% | 144.31s | 🟢 Healthy |
| `main` | 30 天 | 100 | 100.0% | 160.85s | 🟢 Healthy |

基线数值与既往系列报告基本一致，无异常。

## 四、Flaky / 失败信号 与 采集时机口径问题（升级项，第 10 次记录）

`gf pipeline report --branch feat/325-remove-mode1-background-agent --days 7` 两次快照对比：

**第一次快照（`Check` job 实际已 `success`，但仍被计入失败桶）**：
```json
{"totalRuns": 2, "successRate": 0.0, "avgDurationSecs": 24.0, "topFailures": ["Check", "Smoke Test (gitcode)"]}
```

**第二次快照（7/10 job 已收尾且全部 success，3 个仍 `in_progress`）**：
```json
{"totalRuns": 2, "successRate": 0.5, "avgDurationSecs": 57.0, "topFailures": ["Test (macos-latest)"]}
```

两次快照均与实测不符：截至各自采集时点，已收尾的 job 从未出现过失败，但 `topFailures` 两次都把仍在执行或已经成功的 job 错误列入。这与既往系列报告记录的问题**完全一致地再次复现**。

**关键变化**：本仓库近期已合入修复提交 `5051966 fix(github): gate pipeline report terminal state on run/job status, not conclusion presence`（早于本次分析），修复意图正是"根据 run/job 的 status 而非 conclusion 是否存在来判定终态"。但本次实测证明该修复**未能消除**成功率虚假偏低与 `topFailures` 误报的问题。问题已连续 **10 次**（PR #311、#312、#313、#315、#316、#317、#320、#321、#323、#326）被记录，且是**首次在已有针对性修复提交落地之后仍复现**——说明修复要么覆盖不完整，要么未命中本报告触发的代码路径。

**升级说明（按 `gf-pipeline-analyzer` 技能的 Escalation Rule）**：该问题已连续 ≥3 次（实际已达 10 次）复现，其中最近一次修复尝试已被证明无效。因此本报告将其从"已知问题、修复中"升级为**需要重新排查的阻断性建议**，提请用户在以下路径中选择：
1. 通过 `/gf-issue-create`（需人工触发，本技能不代为创建）提交一个新的独立 Issue，说明 `5051966` 的修复未能解决 `gf pipeline report`（区别于 `gf pipeline status`/`jobs`）中 `running`/`queued` 状态被误计入失败桶的问题，并附上本报告第一次快照作为复现证据；或
2. 直接安排对 `gf pipeline report` 命令实现的复查（可能是该修复仅应用于其他报告渲染路径，而未覆盖 `pipeline report` 聚合逻辑本身）。

在此之前，任何依赖 `gf pipeline report` 单次快照做健康度判断的场景都存在误报风险，尤其是在 PR 刚创建、run 仍在排队/执行阶段时采集。**`gf pipeline status` 与 `gf pipeline jobs` 的逐 job 明细数据是准确的**，本报告的实测结论均基于这两个命令交叉核对，而非 `pipeline report` 的聚合快照。

## 五、结论与 Recommendations

1. 🟢 **无阻断性发现** — 已收尾的 7/10 job 全部 `success`，无失败、无跳过；未收尾的 3 个 job（`Test (macos-latest)`、`Test (ubuntu-latest)`、`Test (windows-latest)`）在报告截止时尚无失败信号。
2. 🟡 **Low（数据完整性说明）** — 本报告按协调方指示在 3 个 `Test (*)` job 仍处于 `in_progress` 时提交，未做最终确证；建议后续如有需要可再次核对 run `34001732211` 的最终状态。
3. ⚠️ **Medium → 升级为需用户决策项（第 10 次记录，首次记录"修复无效"）** — `gf pipeline report` 在 run 处于 `running`/`queued` 状态时的快照仍产生虚假低成功率（本次两次快照分别为 0% 与 50%，且误将成功/未失败的 job 列入 `topFailures`），即使在修复提交 `5051966` 合入之后依然复现。**请用户决定**：是否通过 `/gf-issue-create` 提交新 Issue 复查该修复的覆盖范围，或直接安排复查；本技能保持只读，不代为创建 Issue。
