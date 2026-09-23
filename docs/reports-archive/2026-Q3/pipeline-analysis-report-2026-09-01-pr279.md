# Pipeline 分析报告 — PR #279

> **PR：** [#279 refactor!: 删除自动上报bug功能与共建计划安装提示](https://github.com/byx-darwin/gitflow-cli/pull/279)
> **分支：** `feat/278-remove-autoreport-bug` → `dev`
> **分析日期：** 2026-09-01
> **模式：** 只读（CLI: `gf`）
> **背景：** 对应 Issue #278，移除自动上报 bug 功能与共建计划安装提示。

## 一、PR #279 CI 状态

分支 `feat/278-remove-autoreport-bug` 共 3 个 workflow run：

| Run ID | 状态 | 结论 |
|--------|------|------|
| 33479699198 | completed | ✅ success（主 CI workflow，含 MSRV/Lint/Check/Test×3/Smoke Test） |
| 33479699226 | completed | ✅ success（Smoke Test 跨平台：gitlab/github/gitcode） |
| 33479699253 | completed | ✅ success（E2E Tests (GitHub)） |

采集时（06:55 触发）主 CI workflow 中 `Lint`/`Test (macos-latest)`/`Test (windows-latest)` 仍为 `in_progress`；持续轮询（`gh pr checks 279`）至全部收尾后复查，`gf pipeline jobs` 逐一确认全部 11 项 check 均已收尾且**无一失败**：

| Job | 所属 Run | 状态 | 结论 | 耗时 |
|-----|---------|------|------|------|
| Check | 33479699198 | completed | ✅ success | 31s |
| MSRV | 33479699198 | completed | ✅ success | 43s |
| E2E Tests (GitHub) | 33479699253 | completed | ✅ success | 50s |
| Smoke Test (gitlab) | 33479699226 | completed | ✅ success | 55s |
| Smoke Test (gitcode) | 33479699226 | completed | ✅ success | 56s |
| Smoke Test (github) | 33479699226 | completed | ✅ success | 58s |
| Smoke Test | 33479699198 | completed | ✅ success | 1m13s |
| Test (ubuntu-latest) | 33479699198 | completed | ✅ success | 1m36s |
| Lint | 33479699198 | completed | ✅ success | 2m14s |
| Test (macos-latest) | 33479699198 | completed | ✅ success | 3m11s |
| Test (windows-latest) | 33479699198 | completed | ✅ success | 4m4s |

`gf pr checks`（经 `gh pr checks 279` 交叉验证）确认全部 11 项 required check 最终状态均为 `pass`，PR 处于可合并状态（未见 failing/pending 项）。

## 二、失败归因

无。本轮 3 个 workflow run、11 个 job 全部成功，无需归因分析。样本量小（3 次 run），不构成独立趋势判断。

## 三、dev / main 基线（30 天）

| 分支 | Total runs | Success rate | Avg duration | 评级 |
|------|-----------:|--------------:|--------------:|------|
| `dev`（30 天） | 100 | 94.0% | 151.98s | 🟡 Watch（80–94% 区间，接近健康线，与 PR #276 采集时的 93% 基本一致，未见新增回归） |
| `main`（30 天） | 100 | 100.0% | 157.88s | 🟢 Healthy |

`dev` 的 `topFailures` 字段仅返回通用标签 `"failure"`，无法在不扩大抽样的情况下做进一步归因；本次未做扩大抽样，且该基线水位与近期 PR #272/#273/#274/#276 分析时基本一致（93%–94% 区间波动），未观察到因本次合并引入的新增回归信号。

## 四、耗时分析

| 排名 | Job | 耗时 | 说明 |
|------|-----|------|------|
| 1 | Test (windows-latest) | 4m4s | 本轮最长单 job，较 PR #276 同 job 记录（3m51s）增加约 13s，仍在 Windows runner 常见波动区间内，非持续性异常（单点样本不足以判断趋势） |
| 2 | Test (macos-latest) | 3m11s | 较 PR #276（2m51s）增加约 20s，正常范围内波动 |
| 3 | Lint | 2m14s | 与 PR #276（2m13s）基本持平，正常范围 |
| 4 | Test (ubuntu-latest) | 1m36s | 较 PR #276（1m45s）略快，正常范围 |

`gf pipeline report --branch feat/278-remove-autoreport-bug --days 30` 在全部 job 收尾后复查得到 `avgDurationSecs: 122.0`（run 粒度平均，非墙钟总耗时）。主 CI workflow（33479699198）实际墙钟耗时以最长 job `Test (windows-latest)`（4m4s）为准，与 `dev`/`main` 基线（152–158s run 级平均）量级一致，未见持续性耗时增长信号（对比 PR #276：3m51s → PR #279：4m4s，差异 13s，属正常波动，未连续多轮增长）。

## 五、Flaky 信号

未发现 flaky test。本轮所有 job 均一次性通过，无重复间歇性失败样本。`gf pipeline report` 初始采集（`Lint`/`Test (macos-latest)`/`Test (windows-latest)` 仍处于 `in_progress` 时）返回 `successRate: 0.0`（3 次 run 全部因含未收尾 job 被计入非成功），这是既往报告（PR #268/#269/#272/#273/#274/#276）中已反复确认的统计口径问题：`report` 把「仍在 running、尚无 conclusion」的 run 计入非成功，并不代表真实失败。持续轮询至全部 job 收尾后复查，`successRate` 更新为 `1.0`，三个 workflow run 全部 job 均为 `success`。

## 六、结论

- PR #279 相关的三个 workflow run 全部收尾：全部 11 项 check（Lint / Test-ubuntu / Test-macos / Test-windows / MSRV / Check / Smoke Test 全平台 / E2E Tests）**无一失败**，所有 required check 均通过（`gh pr checks 279` 交叉验证一致）。
- `feat/278-remove-autoreport-bug` 分支样本量仅 3 次 run，数据不足以支撑独立趋势判断；采集初期的成功率为 0.0 是统计口径问题（in-progress run 被计入非成功），非真实回归，全部收尾后复查为 100%。
- `dev` 分支近 30 天成功率 94%，处于 🟡 Watch 区间但未跌破 80% 告警线，与 PR #276 采集时（93%）基本一致；`main` 分支近 30 天 100% 健康。均未见因本次合并引入的回归。
- 未发现 flaky test（无重复间歇性失败样本）。耗时方面：`Test (windows-latest)`（4m4s）与 `Test (macos-latest)`（3m11s）较 PR #276 同 job 记录（3m51s/2m51s）小幅上升（约 13–20s），单点样本未形成持续性瓶颈趋势，建议后续 PR 继续观察。

## 七、Recommendations

1. 🟢 **Low** — 无需阻塞式干预。PR #279 全部 check 已收尾且无失败信号，required check 全部通过，可正常推进合并流程（合并本身需用户确认）。
2. 🟡 **Medium** — 持续关注 `dev` 分支成功率（94%），若连续多轮低于 90% 建议扩大抽样定位具体失败 job（当前 `gf pipeline report` 的 `topFailures` 信息量不足以直接归因）。
3. 🟡 **Medium** — `gf pipeline report` 在被分析分支仍有 run 处于 `running`/`in_progress` job 时会低估成功率（把未收尾 run 计入非成功），这是第 7 次在报告中复现（PR #268/#269/#272/#273/#274/#276/#279）；建议在工具层面排除 in-progress run 后再计算成功率，避免每次人工复核。
4. 🟢 **Low** — `Test (windows-latest)` 与 `Test (macos-latest)` 耗时较上次分析（PR #276）小幅上升（13–20s），暂属正常波动区间，建议连续观察 2–3 轮 PR 后再判断是否形成持续性瓶颈趋势。
