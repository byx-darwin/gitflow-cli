# Pipeline 分析报告 — PR #272

> **PR：** [#272 fix(gitlab): use glab issue update --label/--unlabel instead of nonexistent issue edit](https://github.com/byx-darwin/gitflow-cli/pull/272)
> **分支：** `feat/270-gitlab-label-fix` → `dev`（合并提交 `c7b71292bfd9ad8b38b7c6bf501ddf87829bb8c2`）
> **分析日期：** 2026-08-31
> **模式：** 只读（CLI: `gf`）

## 一、PR #272 CI 状态

分支 `feat/270-gitlab-label-fix` 共 3 个 workflow run：

| Run ID | 状态 | 结论 |
|--------|------|------|
| 33366450657 | completed | ✅ success |
| 33366450579 | completed | ✅ success |
| 33366450484 | completed | ✅ success |

`gh pr checks 272` 复查（采集时间 2026-08-31 07:15 UTC，全部 11 项 check 均已收尾）：

| Job | 状态 | 结论 |
|-----|------|------|
| Lint | completed | ✅ success |
| Test (ubuntu-latest) | completed | ✅ success |
| Test (macos-latest) | completed | ✅ success |
| Test (windows-latest) | completed | ✅ success（最初采集时仍 `in_progress`，后续复查已收尾且无失败） |
| MSRV | completed | ✅ success |
| Check | completed | ✅ success |
| Smoke Test | completed | ✅ success |
| Smoke Test (github/gitlab/gitcode) | completed | ✅ success |
| E2E Tests (GitHub) | completed | ✅ success |

`gf pipeline jobs --pipeline-id 33366450657`（Smoke Test 跨平台 workflow：gitcode / github / gitlab）与 `gf pipeline jobs --pipeline-id 33366450579`（E2E Tests (GitHub) workflow）的所有 job 均为 `completed` / `success`。

`gf pipeline report --branch feat/270-gitlab-label-fix --days 30/90` 早前采集返回 `successRate` 在 0.333～0.667 之间波动，原因与既往报告（PR #268、#269）一致：统计口径把「仍在 running、尚无 conclusion」的 run 计入非成功。截至最终复查（`gh pr checks 272`，2026-08-31 07:15 UTC），三个 workflow run 的全部 11 项 check（含此前仍在执行的 `Test (windows-latest)`）均已收尾且**无一失败**。

PR 本身状态为 `closed`（已合并入 `dev`，合并提交 `c7b71292bfd9ad8b38b7c6bf501ddf87829bb8c2`）。

## 二、失败归因

无。本轮未发现任何失败 job，无需归因分析。样本量过小（3 次 run），不构成有效趋势判断。

## 三、dev / main 基线（7–14 天）

| 分支 | Total runs | Success rate | Avg duration | 评级 |
|------|-----------:|--------------:|--------------:|------|
| `dev`（7 天 / 14 天，结果一致） | 100 | 94.0% | 152.2s | 🟡 Watch（80–94% 区间，接近健康线，与 PR #269 报告采集时一致，未见新增回归） |
| `main`（14 天） | 93 | 100.0% | 163.6s | 🟢 Healthy |

`dev` 的 `topFailures` 字段仅返回通用标签 `"failure"`，无法在不扩大抽样（遍历所有失败 run 的 `pipeline jobs`）的情况下做进一步归因；本次未做扩大抽样，且该基线水位与近期 PR #269 分析时一致，未观察到因本次合并引入的新增回归信号。

## 四、结论

- PR #272 相关的三个 workflow run 全部收尾：全部 11 项 check（Lint / Test-ubuntu / Test-macos / Test-windows / MSRV / Check / Smoke Test 全平台 / E2E Tests）**无一失败**。
- `feat/270-gitlab-label-fix` 分支样本量仅 3 次 run，数据不足以支撑趋势判断；早前采集时的成功率波动是统计口径问题（in-progress run 被计入非成功），非真实回归。
- `dev` 分支近 7/14 天成功率 94%，处于 🟡 Watch 区间但未跌破 80% 告警线；`main` 分支近 14 天 100% 健康，均与该 PR 合并前的基线一致。
- 未发现 flaky test（无重复间歇性失败样本），未发现耗时异常（平均耗时与历史基线相当）。

## 五、Recommendations

1. 🟢 **Low** — 无需干预。PR #272 全部 check 已收尾且无失败信号。
2. 🟡 **Medium** — 持续关注 `dev` 分支成功率（94%），若连续多轮低于 95% 建议扩大抽样定位具体失败 job（当前 `gf pipeline report` 的 `topFailures` 信息量不足以直接归因）。
