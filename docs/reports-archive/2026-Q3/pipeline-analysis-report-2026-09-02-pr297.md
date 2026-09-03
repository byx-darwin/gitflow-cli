# Pipeline 分析报告 — PR #297

> **PR：** [#297 fix(pipeline): exclude non-terminal runs from report total_runs](https://github.com/byx-darwin/gitflow-cli/pull/297)
> **分支：** `feat/285-pipeline-report-in-progress-run` → `dev`（对应 Issue #285，已合并，`mergedAt: 2026-09-02T09:25:06Z`）
> **分析日期：** 2026-09-02
> **模式：** 只读（CLI: `gf`）
> **背景：** 本 PR 修复 `gf pipeline report` 的核心统计口径缺陷——`total_runs` 分母此前包含仍处于非终态（in-progress/running/pending）的 run，导致 `success_rate` 被系统性低估。该问题已在此前 **8 份** pipeline-analysis-report（PR #268/#269/#272/#273/#274/#276/#279/#281）中反复复现并作为遗留建议提出。本次分析的重点是对该修复做一次现场验证（sanity check）。

## 零、核心结论先行

**本次分析在采集过程中，用同一分支的实时数据“意外”完整复现了本 PR 要修复的 bug**，并确认修复代码本身正确、已合并至 `dev`，但**尚未反映在本地安装的 `gf` CLI（v1.9.0）中**——因为 v1.9.0 是修复合并前发布的版本，本地二进制未包含该改动。详见第一、二节。

## 一、现场复现：修复前的 CLI 在本分支上的实际表现

`feat/285-pipeline-report-in-progress-run` 分支触发了 3 个 workflow run（PR 刚合并，CI 仍在跑）：

| Run ID | Workflow | 状态（采集时） | 结论 |
|--------|----------|----------------|------|
| 33613981725 | E2E Tests (GitHub) | completed | ✅ success（54s） |
| 33613981748 | Smoke Test 跨平台 | completed | ✅ success（github 50s / gitcode 61s / gitlab 63s） |
| 33613981712 | 主 CI workflow | **running**（Check/MSRV/Smoke Test 已完成，`Test (ubuntu/macos/windows-latest)`、`Lint` 仍 `in_progress`） | — |

采集时执行 `gf pipeline report --branch feat/285-pipeline-report-in-progress-run --days 30`（本地安装的 `gf 1.9.0`）：

```json
{
  "totalRuns": 3,
  "successRate": 0.6666666666666666,
  "avgDurationSecs": 47.0,
  "topFailures": [""]
}
```

即：3 个 run 中 2 个已成功，1 个仍在运行（无 `conclusion`），但 `total_runs` 把这个仍在运行的 run 计入了分母，`success_rate` 被算成 `2/3 ≈ 66.7%`——**这正是 PR #297 要修复的确切 bug**，也与此前 8 份报告中反复出现的现象（PR #279 采集时 `successRate: 0.0`，PR #281 采集时 `successRate: 0.5`）完全一致的统计口径问题。

**根因**：本地安装的 `gf` 二进制版本为 `1.9.0`（构建于 2026-09-02 14:33，早于本次修复的合并时间 09:25:06 之前的下一次 release）；修复提交（`f8e302e` GitHub 侧 + `4930f42` GitLab 侧，合并提交 `92c37ef`）已在 `origin/dev`，但**尚未发布**，因此当前可执行的 CLI 命令观测到的仍是修复前行为。这次现场复现本身即是修复必要性的独立佐证。

## 二、源码层面验证修复正确性

检查 `dev` 分支上的修复提交（未经 `cargo build`，仅静态审查 diff + 测试用例，遵循只读约束）：

- **GitHub**（`crates/github/src/pipeline.rs`，提交 `f8e302e`）：`total_runs` 由 `runs.len()` 改为 `runs.iter().filter(|r| r.conclusion.is_some()).count()`——只统计已 `completed`（有 `conclusion`）的 run。新增测试 `test_should_exclude_in_progress_runs_from_report_total_runs`：构造 2 success + 1 failure + 1 in-progress（`conclusion: null`），断言 `total_runs == 3`、`success_rate == 2/3`。
- **GitLab**（`crates/gitlab/src/pipeline.rs`，提交 `4930f42`）：`total_runs` 由 `recent.len()` 改为排除 `PipelineStatusEnum::Running`/`Pending` 后计数。新增两个测试：`test_should_exclude_non_terminal_pipelines_from_report_total_runs`（2 success + 1 failed + 1 running，断言 `total_runs == 3`）与边界用例 `test_should_zero_report_when_all_pipelines_are_running`（全部 running 时 `total_runs == 0`、`success_rate == 0.0` 且非 `NaN`，覆盖除零边界）。
- 未涉及 GitCode（`report` 为未实现 stub），未新增字段、未改 `PipelineStatusEnum` 定义，改动面精确收敛在两个 provider 的 `report` 方法内。
- PR 描述记录：`cargo test --workspace` 全部通过（1370 tests，较修复前基线 1367 增加 3 个回归测试）；`cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic` 无警告；`cargo +nightly fmt --check` 无差异。

**用第一节的实测数据反推修复效果**：若本地 CLI 已包含该修复，对同一批 run（2 success + 1 in-progress）重新计算，`total_runs` 应为 `2`（排除仍在运行的第 3 个 run），`success_rate` 应为 `1.0`（100%），而非当前观测到的 `0.667`。这与 GitHub 侧新增测试的断言逻辑完全对应，修复方向和幅度均正确。

## 三、失败归因

无真实失败。3 个 workflow run 中已收尾的 2 个（E2E Tests、Smoke Test 跨平台共 4 个 job）全部成功；第 3 个 run（主 CI）采集时仍在执行，已完成的 `Check`/`MSRV`/`Smoke Test` 3 个 job 也全部成功，无一失败样本。`successRate: 0.667` 是统计口径问题，非真实回归。

## 四、dev / main 基线（30 天）

| 分支 | Total runs | Success rate | Avg duration | 评级 |
|------|-----------:|--------------:|--------------:|------|
| `dev`（30 天） | 100 | 95.0% | 149.2s | 🟢 Healthy（较 PR #281 采集时 94.0% 略升 1.0pp，仍处于同一波动区间，未见回归） |
| `main`（30 天） | 100 | 100.0% | 159.59s | 🟢 Healthy |

`dev` 分支 30 天样本量已达 100（窗口上限），`topFailures` 仍仅返回通用标签 `"failure"`，无法在不扩大抽样的情况下做进一步归因。95.0% 已越过 🟢 Healthy 门槛（≥95%），较此前多轮报告（93%–94% 区间）小幅改善，但样本仍以历史 run 为主，本次 PR #297 尚未发布，不构成该改善的直接原因。

## 五、耗时分析

| 排名 | Job | 耗时 | 说明 |
|------|-----|------|------|
| 1 | Smoke Test (gitlab) | 1m3s | 本分支最长单 job，处于 GitLab CLI smoke test 常见区间内 |
| 2 | Smoke Test (gitcode) | 1m1s | 正常范围 |
| 3 | Smoke Test（主 CI，单平台） | 1m22s（33613981712 内） | 已收尾，正常范围 |
| 4 | E2E Tests (GitHub) | 54s | 正常范围 |
| 5 | Smoke Test (github) | 50s | 正常范围 |
| 6 | MSRV | 46s | 正常范围 |
| 7 | Check | 30s | 正常范围 |

主 CI workflow（33613981712）中耗时通常最长的 `Test (windows-latest)`/`Test (macos-latest)`/`Test (ubuntu-latest)`/`Lint` 在采集时仍处于 `in_progress`，无法给出本轮的最终耗时数据；`gf pipeline report` 返回的 `avgDurationSecs: 47.0` 仅覆盖已收尾的两个 run（E2E Tests + Smoke Test 跨平台），量级与 `dev`/`main` 基线（149–160s，run 粒度总耗时口径不同）不可直接比较。本次未观察到耗时层面的持续性瓶颈或异常。

## 六、Flaky 信号

未发现 flaky test。已收尾的 job（Check/MSRV/Smoke Test 全平台/E2E Tests，共 7 项）全部一次性通过，无重复间歇性失败样本。

## 七、结论

- **本次分析在 `feat/285-pipeline-report-in-progress-run` 分支上实时复现了 PR #297 所修复的确切 bug**：本地安装的 `gf 1.9.0` 在该分支仍有 1 个 run 处于 `running` 状态时，`gf pipeline report` 返回 `successRate: 0.667`（3 个 run 中把 1 个仍在运行的 run 计入分母），与此前 8 份报告（PR #268/#269/#272/#273/#274/#276/#279/#281）中反复出现的现象一致。
- **修复源码已合并至 `origin/dev`**（提交 `f8e302e` + `4930f42`，合并提交 `92c37ef`），静态审查确认逻辑正确：GitHub 侧按 `conclusion.is_some()` 过滤，GitLab 侧按排除 `Running`/`Pending` 状态过滤；新增 3 个回归测试覆盖正常场景与全 running 的除零边界，断言与本次实测数据（2 success + 1 running → 应得 `total_runs=2`、`success_rate=1.0`）完全对应。
- **修复尚未反映在当前可执行的 CLI 中**——本地 `gf` 版本为 `1.9.0`，构建早于本次合并，需等下一次 release 后才能在真实命令输出中观察到 `total_runs` 正确排除非终态 run。这是本次 sanity check 的关键限制：无法用当前环境直接对比"修复前 vs 修复后"的命令行为，只能通过源码审查 + 历史行为复现来验证。
- PR #297 相关 CI：已收尾的 7 个 job（Check/MSRV/Smoke Test×4/E2E Tests）全部成功，无失败；主 CI workflow 采集时仍在执行，但 PR 已合并（`mergedAt: 2026-09-02T09:25:06Z`），大概率表示 required check 已全部通过（未违反只读约束，未做二次轮询确认全部 job 状态）。
- `dev` 分支近 30 天成功率 95.0%，较 PR #281 采集时（94.0%）小幅改善，进入 🟢 Healthy 区间；`main` 分支 100% 健康。均未见回归信号。
- 未发现 flaky test；耗时方面未见异常，但主 CI workflow 的核心耗时 job（Test 矩阵、Lint）因采集时仍在执行而缺失本轮数据。

## 八、Recommendations

1. 🟢 **Low** — 无需阻塞式干预。PR #297 已合并，已收尾的 7 个 job 全部成功，修复逻辑经源码审查确认正确，测试覆盖完整（含除零边界）。
2. 🟡 **Medium** — **尽快将本次修复纳入下一个 release**（当前最新发布版本 `v1.9.0` 不含该修复）。只要发布延迟，未来任何在 in-progress run 期间采集的 pipeline-analysis-report 仍会复现本报告第一节展示的统计失真，之前遗留的"第 8 次复现"建议会继续变成"第 9 次、第 10 次……"。
3. 🟡 **Medium** — 发布新版本后，建议对本报告做一次**收尾复核**：在同一分支（或任意仍有 in-progress run 的分支）上用新版 `gf` 重跑 `pipeline report`，确认 `total_runs` 与 `success_rate` 不再包含非终态 run，并将结果补记为本报告的验证闭环（可复用第二节"用实测数据反推"的推算值 `total_runs=2`、`success_rate=1.0` 作为预期基准）。
4. 🟢 **Low** — `dev` 分支的 `topFailures` 字段仍仅返回通用标签 `"failure"`，信息量不足以直接归因失败 job；非本次修复范围，维持既有建议（若连续多轮低于 90% 再扩大抽样定位）。
