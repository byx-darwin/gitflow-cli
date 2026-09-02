# Pipeline 分析报告 — PR #300

> **PR：** [#300 docs(community): 发布 gf 开发 gf 的 dogfooding 案例文章](https://github.com/byx-darwin/gitflow-cli/pull/300)
> **分支：** `feat/288-dogfooding-case-study` → `dev`（对应 Issue #288，已合并，`mergedAt: 2026-09-02T10:05:12Z`）
> **分析日期：** 2026-09-02
> **模式：** 只读（CLI: `gf`，版本 `1.9.0`）
> **背景：** 本 PR 为纯文档改动（新增官网案例页 `website/src/pages/dogfooding.mdx` 并更新 README / `docs/index.md` / 官网首页 / `docs.astro` 的入口链接），未触碰 Rust 代码。按 CLAUDE.md 的 Required Validation 规则，文档改动跳过 Rust build/test/clippy。本报告对该 PR 相关的 CI/CD 流水线做交付后健康核查。

## 零、核心结论先行

PR #300 触发了 6 个 workflow run（两波，间隔约 40 秒）。采集初期有 2 个 run 处于 `running`（主 CI workflow 的 `Test (windows-latest)` / `Test (macos-latest)` 仍在执行），经持续轮询至全部收尾：**6 个 run、全部 job（合计 24 个，含 1 个 `skipped` 的 `Deploy` job）最终结论均为成功，无任何失败样本，也未观察到重试/间歇性失败迹象**。`gf pipeline report --branch feat/288-dogfooding-case-study` 在全部 run 终态后返回 `successRate: 1.0`、`avgDurationSecs: 106.33s`。`dev`/`main` 30 天基线均处于 🟢 Healthy 区间。唯一值得记录的观察点是：本 PR 触发了两波几乎重复的 workflow run（详见第一节），以及 `Test (windows-latest)` 是全流水线中最慢的 job（185s / 172s），但均在历史正常区间内。**结论：无阻塞性发现，PR #300 判定为“无异常”，健康、可安全视为已合并且无需额外处置。**

## 一、PR #300 关联流水线实测

`feat/288-dogfooding-case-study` 分支共触发 6 个 workflow run，分两波，间隔约 40 秒：

| Run ID | Workflow | 触发时间 | 状态（终态） | 结论 |
|--------|----------|----------|--------------|------|
| 33617515918 | Build/Deploy（官网） | 10:04:12 | completed | ✅ success（`Build` 18s success；`Deploy` skipped，符合非 `main` 分支不部署的预期） |
| 33617516012 | Smoke Test 跨平台 #1 | 10:04:12 | completed | ✅ success（gitlab 44s / github 43s / gitcode 42s，均 success） |
| 33617516066 | E2E Tests | 10:04:12 | completed | ✅ success（`E2E Tests (GitHub)` 53s） |
| 33617516068 | 主 CI workflow #1 | 10:04:12 | completed | ✅ success（Check/MSRV/Lint/Smoke Test/Test×3 全部 success，见下表） |
| 33617576530 | Smoke Test 跨平台 #2 | 10:04:52 | completed | ✅ success（github 48s / gitlab 61s / gitcode 57s，均 success） |
| 33617576590 | 主 CI workflow #2 | 10:04:52 | completed | ✅ success（Check/MSRV/Lint/Smoke Test/Test×3 全部 success，见下表） |

**观察**：同一次 push 触发了两组几乎重复的 run（第一波 10:04:12，含 Build/Deploy + Smoke Test + E2E Tests + 主 CI，共 4 个 run；第二波 10:04:52，仅 Smoke Test + 主 CI，共 2 个 run），两波的 `refName` 均为 `feat/288-dogfooding-case-study`。第二波未重新触发 Build/Deploy 与 E2E Tests，推测与 `gf pr merge` 流程中的分支同步/webhook 去抖行为有关，不构成失败，但存在轻微的 CI 资源重复消耗（详见「六、Recommendations」）。

已收尾 job 明细（全部 success，`Deploy` 为 skipped，无失败样本）：

| Job | Workflow run | 耗时 | 结论 |
|-----|--------------|------|------|
| Build | 33617515918 | 18s | ✅ success |
| Deploy | 33617515918 | 0s | ⏭️ skipped（非 `main` 分支，符合预期） |
| Smoke Test (github) | 33617516012 | 43s | ✅ success |
| Smoke Test (gitlab) | 33617516012 | 44s | ✅ success |
| Smoke Test (gitcode) | 33617516012 | 42s | ✅ success |
| E2E Tests (GitHub) | 33617516066 | 53s | ✅ success |
| Check | 33617516068 | 33s | ✅ success |
| MSRV | 33617516068 | 57s | ✅ success |
| Smoke Test | 33617516068 | 44s | ✅ success |
| Lint | 33617516068 | 88s | ✅ success |
| Test (ubuntu-latest) | 33617516068 | 65s | ✅ success |
| Test (macos-latest) | 33617516068 | 114s | ✅ success |
| Test (windows-latest) | 33617516068 | 185s | ✅ success |
| Smoke Test (github) | 33617576530 | 48s | ✅ success |
| Smoke Test (gitlab) | 33617576530 | 61s | ✅ success |
| Smoke Test (gitcode) | 33617576530 | 57s | ✅ success |
| Check | 33617576590 | 36s | ✅ success |
| MSRV | 33617576590 | 56s | ✅ success |
| Smoke Test | 33617576590 | 84s | ✅ success |
| Test (ubuntu-latest) | 33617576590 | 121s | ✅ success |
| Lint | 33617576590 | 130s | ✅ success |
| Test (macos-latest) | 33617576590 | 177s | ✅ success |
| Test (windows-latest) | 33617576590 | 172s | ✅ success |

`gf pipeline report --branch feat/288-dogfooding-case-study --days 30`（全部 run 终态后采集）输出：

```json
{
  "totalRuns": 6,
  "successRate": 1.0,
  "avgDurationSecs": 106.33333333333333,
  "topFailures": []
}
```

**注**：采集初期（部分 run 仍 `running`）曾观察到 `totalRuns: 6`、`successRate: 0.667`——这是此前多份报告（PR #268/#269/#272/#273/#274/#276/#279/#281/#297/#298）反复记录的已知统计口径缺陷（未终态 run 被计入 `total_runs` 分母），PR #297 已修复但截至本次采集尚未随新版本发布（本地/CI 仍为 `gf v1.9.0`）。待所有 run 收尾后复采，`successRate` 已正确收敛为 `1.0`，与最终真实结论一致。

## 二、失败归因

无真实失败。PR #300 相关的全部 6 个 workflow run、24 个 job（其中 1 个 `Deploy` 为符合预期的 `skipped`）最终结论全部为 success，无一次失败或需要重试的样本。PR 已于 `2026-09-02T10:05:12Z` 合并至 `dev`。采集过程中出现的 `successRate: 0.667` 中间态是已知的非终态 run 统计口径问题（见上节），非真实回归。

## 三、dev / main 基线（30 天）

| 分支 | Total runs | Success rate | Avg duration | 评级 |
|------|-----------:|--------------:|--------------:|------|
| `dev`（30 天） | 100 | 95.0% | 149.2s | 🟢 Healthy（与前序报告持平，无回归） |
| `main`（30 天） | 100 | 100.0% | 159.59s | 🟢 Healthy |

两个分支样本量均已达 100（窗口上限）。`dev` 分支 `topFailures` 仅返回通用标签 `"failure"`（历史遗留问题，非本次新增）；`main` 分支为空数组（无失败样本）。未见成功率或耗时层面的回归信号。

## 四、耗时分析

| 排名 | Job | 耗时 | 说明 |
|------|-----|------|------|
| 1 | Test (windows-latest)（run 33617516068） | 185s | 全流水线最慢 job，windows 平台历史上耗时通常高于 macos/ubuntu，属正常区间 |
| 2 | Test (macos-latest)（run 33617576590） | 177s | 正常范围 |
| 3 | Test (windows-latest)（run 33617576590） | 172s | 正常范围，与同 job 另一次运行（185s）耗时接近，波动 <10% |
| 4 | Lint（run 33617576590） | 130s | 正常范围 |
| 5 | Test (ubuntu-latest)（run 33617576590） | 121s | 正常范围 |
| 6 | Test (macos-latest)（run 33617516068） | 114s | 正常范围 |

`Test (windows-latest)` 是本轮唯二超过 150s 的 job 类型（两次运行分别为 185s、172s），是全流水线的耗时瓶颈，但两次数值相近、无异常拉长迹象，且与 `dev`/`main` 基线（149–160s，run 粒度总耗时口径不同，不可直接比较）量级一致。`gf pipeline report` 最终 `avgDurationSecs: 106.33s` 覆盖全部 6 个 run、24 个 job，未观察到持续性瓶颈或异常延长。

## 五、Flaky 信号

未发现 flaky test。全部 24 个 job（含两波近乎重复触发的 job）均一次性通过，无重复间歇性失败样本，也未见任何 job 被平台自动重试。

## 六、结论

- PR #300（纯文档改动，新增官网 dogfooding 案例页 + 4 处入口链接更新）相关的全部 6 个 workflow run、24 个 job（1 个 `Deploy` 为预期内 skipped）最终结论全部成功，无失败样本；PR 已于 `2026-09-02T10:05:12Z` 合并至 `dev`。
- 采集过程中曾观察到 `successRate: 0.667` 的中间态，是 PR #297 已修复但尚未随新版本发布的统计口径缺陷（非终态 run 计入分母）的再次复现；待全部 run 收尾后复采已正确收敛为 `1.0`，非真实回归。
- `dev` 分支近 30 天成功率 95.0%、`main` 分支 100.0%，均处于 🟢 Healthy 区间，与前序报告持平，无回归信号。
- 未发现 flaky test；耗时瓶颈集中在 `Test (windows-latest)`（172–185s），属历史正常区间，无异常延长。
- 唯一观察点：本次 push 触发了两波几乎重复的 workflow run（间隔约 40 秒），存在轻微 CI 资源重复消耗，但不影响正确性与本次合并健康度。
- **总体判定：无阻塞性发现（no findings），PR #300 可安全视为健康合并。**

## 七、Recommendations

1. 🟢 **Low** — 无需阻塞式干预。PR #300 已合并，全部 job 最终成功，文档改动按规则跳过 Rust 全量校验，符合项目规范。
2. 🟡 **Medium**（历史遗留，非本次新增）— PR #297 的统计口径修复应尽快纳入下一个 release（当前最新发布版本仍为 `v1.9.0`，不含该修复）。只要发布延迟，任何在 in-progress run 期间采集的 pipeline-analysis-report 都会继续复现本报告第一节展示的统计失真中间态。
3. 🟢 **Low**（新增观察，非阻塞）— 本次 push 触发了两波几乎重复的 workflow run（间隔约 40 秒，第二波仅重跑 Smoke Test + 主 CI，未重跑 Build/Deploy 与 E2E Tests），建议后续排查 `gf pr merge` / 分支同步流程是否存在重复触发 webhook 的情况，以减少 CI 资源消耗；不影响本次判定。
4. 🟢 **Low** — `dev` 分支的 `topFailures` 字段仍仅返回通用标签 `"failure"`，信息量不足以直接归因失败 job；非本次改动范围，维持既有建议（若连续多轮低于 90% 再扩大抽样定位）。
