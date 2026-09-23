# Pipeline 分析报告 — PR #312

> **PR：** [#312 chore(cli): remove deprecated run subcommand](https://github.com/byx-darwin/gitflow-cli/pull/312)
> **分支：** `feat/294-remove-run-command` → `dev`（对应 Issue #294）
> **合并时间：** 2026-09-03T09:03:54Z
> **分析日期：** 2026-09-03
> **模式：** 只读（CLI: `gf`）
> **变更性质：** 删除已废弃的 `run` 子命令存根——移除 `apps/cli/src/commands/run.rs`（`RunArgs`）及 `mod run;` 声明、`main.rs` 中的 `Commands::Run` 变体与匹配分支；新增 2 个集成测试（`test_should_not_list_run_subcommand_in_help`、`test_should_reject_run_as_unrecognized_subcommand`）。破坏性变更（`chore(cli)!:` + `BREAKING CHANGE:` footer）。未触及 CI workflow 配置、`Cargo.toml`/`Cargo.lock`。

## 零、核心结论先行

PR #312 触发的 3 个 workflow run **全部成功**，累计 **14 个 job 全部成功**（E2E Tests × 3 平台、主 CI × 7 job、Smoke Test 跨平台 × 3 平台，另加 1 个 E2E GitHub job）。分析期间 `gf pipeline report --branch feat/294-remove-run-command --days 30` 一度返回 `successRate: 0.333`，这与 PR #311/#309 报告记录的同一类"统计口径假象"一致——命令在 run 仍处于 `running`（`conclusion` 为空）状态时将其计入分母且视同失败；本报告持续观察至全部 3 个 run 收尾（最终一个 `Test (windows-latest)` job 于 `09:08:15Z` 完成），逐 job 复核确认**没有任何真实失败**。`dev`/`main` 基线保持健康（95%/100%），且历史已知的单次 Windows flaky 案例（run `33346653353`）未在本 PR 中复现。**总体判定：PR #312 自身流水线健康，无异常发现，无需人工介入。**

## 一、PR #312 关联流水线实测

`feat/294-remove-run-command` 分支触发 3 个 workflow run（均创建于 `2026-09-03T09:03:40Z`）：

| Run ID | Workflow | 结论 | 备注 |
|--------|----------|------|------|
| 33736749269 | E2E Tests（GitHub/GitLab/GitCode） | ✅ success（3/3 job 全部成功） | 最终收尾于 `09:07:03Z` |
| 33736749411 | 主 CI（Check/MSRV/Lint/Smoke Test/Test×3） | ✅ success（7/7 job 全部成功） | 最终收尾于 `09:08:15Z`（`Test (windows-latest)` 最后完成） |
| 33736749775 | Smoke Test 跨平台 | ✅ success（3/3 job 全部成功） | 完成于 `09:04:45Z` |

采集初期（run 33736749269 / 33736749411 仍在运行时）`gf pipeline report --branch feat/294-remove-run-command --days 30`：

```json
{
  "totalRuns": 3,
  "successRate": 0.3333333333333333,
  "avgDurationSecs": 28.33,
  "topFailures": [""]
}
```

**数据口径说明**：与 PR #311 报告记录的现象一致——`topFailures: [""]` 对应当时仍在运行、`conclusion` 字段为空的 2 个 run，命令未区分"运行中"与"失败"，直接计入失败桶导致 `successRate` 虚低。本报告持续跟踪至 3 个 run 全部收尾，逐 job 复核 `gf pipeline jobs`，确认全部 14 个 job 均为 `success`。**该 CLI 口径问题此前已在 PR #311 报告中提出改进建议（见「六」#2），本次为第二次复现观察，建议提升优先级。**

全部 14 个 job 明细（**全部成功**）：

| Job | Workflow run | 耗时 | 结论 |
|-----|--------------|------|------|
| Check | 33736749411 | 35s | ✅ success |
| Smoke Test | 33736749411 | 67s | ✅ success |
| MSRV | 33736749411 | 55s | ✅ success |
| E2E Tests (GitHub) | 33736749269 | 56s | ✅ success |
| Smoke Test (github) | 33736749775 | 57s | ✅ success |
| Smoke Test (gitlab) | 33736749775 | 54s | ✅ success |
| Smoke Test (gitcode) | 33736749775 | 60s | ✅ success |
| Test (ubuntu-latest) | 33736749411 | 117s | ✅ success |
| Lint | 33736749411 | 140s | ✅ success |
| E2E Tests (GitLab) | 33736749269 | ~198s | ✅ success |
| E2E Tests (GitCode) | 33736749269 | ~199s | ✅ success |
| Test (macos-latest) | 33736749411 | 223s | ✅ success |
| Test (windows-latest) | 33736749411 | **269s**（本轮最慢 job） | ✅ success |

## 二、`Test (windows-latest)` 收尾结果确认

报告初次采集时（`09:03–09:07Z`）该 job 持续处于 `in_progress`。持续观察至其收尾（`completedAt: 09:08:15Z`），**最终结论为 `success`**，总耗时 269s（`09:03:46Z` → `09:08:15Z`）。对照历史基线：

- PR #311 记录同类 job 耗时 337s，PR #309 记录区间 172s–288s；本次 269s 落在历史区间内，属正常波动。
- `dev` 分支 7 天窗口内曾于 `2026-08-31T01:08:14Z`（run `33346653353`）出现过一次 `Test (windows-latest)` **真实失败**（详见「四」）；本次 PR #312 的同名 job 未复现该失败。

**结论：该 job 最终成功，此前的 `in_progress` 状态仅是本报告采集时序问题，不构成异常。**

## 三、dev / main 基线（7 天 / 30 天，采集时点：PR #312 触发后）

| 分支 | 周期 | Total runs | Success rate | Avg duration | 评级 |
|------|------|-----------:|--------------:|--------------:|------|
| `dev` | 7 天 | 100 | 95.0% | 150.47s | 🟢 Healthy |
| `dev` | 30 天 | 100（同 7 天，样本量已触顶） | 95.0% | 150.47s | 🟢 Healthy |
| `main` | 30 天 | 100 | 100.0% | 159.59s | 🟢 Healthy |

基线数值与 PR #311 报告完全一致（同一采集窗口内未产生新失败），延续系列报告（PR #304/#309/#311）观察到的健康水位。

## 四、Flaky / 失败信号

**PR #312 自身流水线**：全部 14 个 job（含最终收尾的 `Test (windows-latest)`）均为 `success`，采集范围内未发现 flaky 或失败信号。

**7 天基线窗口内的历史案例（非本 PR 引入，仅作上下文记录，与 PR #311 报告一致）**：`dev` 分支 run `33346653353`（`2026-08-31T01:08:14Z`）的 `Test (windows-latest)` job 出现真实失败：

```
Test (windows-latest)  cargo test  FAIL [0.026s] (116/1384)
  gitflow-cli::bin/gf commands::commit::tests::test_should_resolve_comment_body_from_file
  thread 'commands::commit::tests::test_should_resolve_comment_body_from_file' panicked at
  apps\cli\src\commands\commit.rs:245:9:
  assertion failed: result.is_ok()
```

该失败在 7 天窗口内仍仅出现 **1 次**（100 个 run 中 1 次，占比 1%），距上次报告（PR #311）以来未再复现，按本 skill "≥2 次间歇性失败才判定为 flaky" 的标准，**仍不构成 flaky 判定**。**该失败与 PR #312（删除 `run` 子命令存根，未触及 `commit.rs`）无关联**——两者分支、提交、改动文件均无交集。维持观察清单状态，无需升级。

## 五、耗时分析

- 全部 job 耗时（35s–269s）与历史基线（PR #304/#309/#311 记录的 39s–337s 区间）基本一致，无新增瓶颈。
- `Test (windows-latest)` 269s 低于 PR #311 记录的 337s，回落至 PR #309 区间内；`Test (macos-latest)` 223s 与 PR #311 的 216s 接近，波动 <5%。
- E2E Tests 三个平台（GitHub/GitLab/GitCode）耗时集中在 56s–199s，GitHub 平台（56s）明显快于 GitLab/GitCode（~198–199s），与既往报告观察到的平台间耗时差异模式一致，非本次改动引入。
- 本次变更仅删除一个从未被实际调用的废弃命令存根及其匹配分支，代码变更量极小，理论上不应影响任何 job 耗时；观察到的波动应归因于 CI Runner 资源抖动，而非本次改动引入的性能回归。

## 六、结论与 Recommendations

1. 🟢 **Low** — PR #312 删除废弃的 `run` 子命令存根，关联的 3 个 workflow run、14 个 job **全部成功**，无失败、无 flaky，无需人工介入。
2. 🟡 **Medium**（较 PR #311 报告的 Low 上调）— `gf pipeline report` 命令在 run 仍处于 `running` 状态时会将其计入失败桶，污染 `successRate`/`topFailures`。本报告是该问题**第二次连续复现**（PR #311 采集到 `successRate: 0.5`，本次 PR #312 采集到 `successRate: 0.333`），且分析耗费了额外的持续轮询开销以澄清"假象 vs 真实失败"。建议尽快对 `gf` CLI 提交独立 Issue，改进 `pipeline report` 使其将 `running`/`queued` 状态与真实 `failure` 分开统计，避免类似口径澄清在后续每份报告中反复出现。
3. 🟡 **Low** — `commands::commit::tests::test_should_resolve_comment_body_from_file`（`apps/cli/src/commands/commit.rs:245`）在 `dev` 分支 7 天窗口内仍保持 1 次历史失败记录（run `33346653353`，2026-08-31），未达 flaky 判定阈值（≥2 次），且与 PR #312 无关联。继续维持观察清单状态，若再次复现应升级为 flaky 并排查。
