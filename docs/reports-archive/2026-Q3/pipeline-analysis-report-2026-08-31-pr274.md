# Pipeline 分析报告 — PR #274

> **PR：** [#274 fix(gitlab): use git remote URL as --repo target for issue commands](https://github.com/byx-darwin/gitflow-cli/pull/274)
> **分支：** `feat/267-gitlab-issue-repo-target` → `dev`
> **分析日期：** 2026-08-31
> **模式：** 只读（CLI: `gf`）
> **背景：** 修复 Issue #267（GitLab `issue add-label`/`remove-label` 等命令在自建 GitLab Work Items 实例上失败）。本报告在等待剩余 job（`Test (windows-latest)`）收尾后采集最终状态。

## 一、PR #274 CI 状态

分支 `feat/267-gitlab-issue-repo-target` 共 3 个 workflow run：

| Run ID | 状态 | 结论 |
|--------|------|------|
| 33373171850 | completed | ✅ success（主 CI workflow，含 Lint/MSRV/Test×3/Check/Smoke Test） |
| 33373171872 | completed | ✅ success（E2E Tests (GitHub)） |
| 33373171897 | completed | ✅ success（Smoke Test 跨平台：gitlab/gitcode/github） |

`gf pipeline jobs` 逐一复查，全部 11 项 check 均已收尾且**无一失败**：

| Job | 所属 Run | 状态 | 结论 | 耗时 |
|-----|---------|------|------|------|
| Check | 33373171850 | completed | ✅ success | 32s |
| MSRV | 33373171850 | completed | ✅ success | 55s |
| Smoke Test | 33373171850 | completed | ✅ success | 1m04s |
| Test (ubuntu-latest) | 33373171850 | completed | ✅ success | 1m37s |
| Lint | 33373171850 | completed | ✅ success | 2m19s |
| Test (macos-latest) | 33373171850 | completed | ✅ success | 3m00s |
| Test (windows-latest) | 33373171850 | completed | ✅ success | 3m46s（本轮采集时初始为 `in_progress`，持续轮询至收尾，无失败） |
| E2E Tests (GitHub) | 33373171872 | completed | ✅ success | 43s |
| Smoke Test (gitlab) | 33373171897 | completed | ✅ success | 58s |
| Smoke Test (gitcode) | 33373171897 | completed | ✅ success | 51s |
| Smoke Test (github) | 33373171897 | completed | ✅ success | 1m05s |

`gf pipeline report --branch feat/267-gitlab-issue-repo-target --days 7` 采集初值 `successRate: 0.667`（3 次 run 中 1 次仍处于 `running`），这是既往报告（PR #268/#269/#272/#273）中已反复确认的统计口径问题：`report` 把「仍在 running、尚无 conclusion」的 run 计入非成功，并不代表真实失败。持续轮询至 `Test (windows-latest)` job 收尾后复查，三个 workflow run 全部 job 均为 `success`。

PR 当前状态为 `closed`（已合并进 `dev`，`dev` 分支合并提交 `541cd25 Merge pull request #274 from byx-darwin/feat/267-gitlab-issue-repo-target`）。

## 二、失败归因

无。本轮 3 个 workflow run、11 个 job 全部成功，无需归因分析。样本量小（3 次 run），不构成独立趋势判断。

## 三、dev / main 基线（7–14 天）

| 分支 | Total runs | Success rate | Avg duration | 评级 |
|------|-----------:|--------------:|--------------:|------|
| `dev`（7 天 / 14 天，结果一致） | 100 | 94.0% | 152.5s | 🟡 Watch（80–94% 区间，接近健康线，与 PR #272/#273 报告采集时一致，未见新增回归） |
| `main`（14 天） | 93 | 100.0% | 163.6s | 🟢 Healthy |

`dev` 的 `topFailures` 字段仅返回通用标签 `"failure"`，无法在不扩大抽样的情况下做进一步归因；本次未做扩大抽样，且该基线水位与近期 PR #272/#273 分析时一致，未观察到因本次合并引入的新增回归信号。

## 四、耗时分析

| 排名 | Job | 耗时 | 说明 |
|------|-----|------|------|
| 1 | Test (windows-latest) | 3m46s | 本轮最长单 job，高于 PR #273 同 job 记录（2m51s），但仍在 Windows runner 常见波动区间内，非持续性异常，建议后续 PR 继续观察是否形成趋势 |
| 2 | Test (macos-latest) | 3m00s | 略高于历史区间，同上，需持续观察 |
| 3 | Lint | 2m19s | 正常范围 |
| 4 | Test (ubuntu-latest) | 1m37s | 正常范围 |

三个 workflow run 整体平均耗时（`gf pipeline report`）约 41.7s（该字段按 run 粒度统计，主 CI workflow 因含多 job 并行执行，实际墙钟时间以最长 job 为准，本轮约 3 分 46 秒）。与 `dev`/`main` 基线（152–164s run 级平均）量级一致；但主 CI workflow 的墙钟耗时（约 3m53s，从 08:29:50 创建到 08:33:42 全部收尾）较 PR #273 同类 workflow（约 2m51s 收尾）明显更长，主要由 `Test (windows-latest)`/`Test (macos-latest)` 两个矩阵 job 拖长导致，属单次样本波动，暂未构成持续性耗时增长信号（样本量仅 1 次可比对）。

## 五、Flaky 信号

未发现 flaky test。本轮所有 job 均一次性通过，无重复间歇性失败样本。`gf pipeline report` 初始采集到的低成功率是 in-progress run 被计入非成功的统计口径问题，非真实的间歇性失败，与既往报告（PR #268、#269、#272、#273）结论一致——该问题为 `gf pipeline report` 工具自身的已知局限，建议后续在工具层面排除 `running` 状态的 run 再计算成功率。

## 六、结论

- PR #274 相关的三个 workflow run 全部收尾：全部 11 项 check（Lint / Test-ubuntu / Test-macos / Test-windows / MSRV / Check / Smoke Test 全平台 / E2E Tests）**无一失败**，所有 required check 均通过。
- PR 已合并进 `dev`（`541cd25`），Issue #267 的修复代码已进入主干。
- `feat/267-gitlab-issue-repo-target` 分支样本量仅 3 次 run，数据不足以支撑趋势判断；早前采集时的成功率波动是统计口径问题（in-progress run 被计入非成功），非真实回归。
- `dev` 分支近 7/14 天成功率 94%，处于 🟡 Watch 区间但未跌破 80% 告警线；`main` 分支近 14 天 100% 健康，均与该 PR 合并前的基线一致，未见回归。
- 未发现 flaky test（无重复间歇性失败样本）。耗时方面：`Test (windows-latest)`（3m46s）与 `Test (macos-latest)`（3m00s）较 PR #273 同 job 记录偏高，但样本量不足以判定为持续性瓶颈趋势，建议后续 PR 继续观察。

## 七、Recommendations

1. 🟢 **Low** — 无需阻塞式干预。PR #274 全部 check 已收尾且无失败信号，已合并进 `dev`，Issue #267 修复已进入主干。
2. 🟡 **Medium** — `Test (windows-latest)`/`Test (macos-latest)` 本轮耗时较 PR #273 同 job 明显偏高（3m46s/3m00s vs 2m51s/2m06s）。建议后续 2-3 次 PR 持续观察这两个 job 的耗时曲线，若形成持续增长趋势再排查依赖缓存或 runner 资源问题。
3. 🟡 **Medium** — 持续关注 `dev` 分支成功率（94%），若连续多轮低于 95% 建议扩大抽样定位具体失败 job（当前 `gf pipeline report` 的 `topFailures` 信息量不足以直接归因）。
4. 🟡 **Medium** — `gf pipeline report` 在被分析分支仍有 run 处于 `running` 状态时会低估成功率（把未收尾 run 计入非成功），这是第 5 次在报告中复现（PR #268/#269/#272/#273/#274）；建议在工具层面排除 in-progress run 后再计算成功率，避免每次人工复核。
