# Pipeline 分析报告 — PR #268

> **PR：** [#268 feat(gf-workflow): support local-merge as an alternative to PR delivery](https://github.com/byx-darwin/gitflow-cli/pull/268)
> **分支：** `feat/265-gf-workflow-local-merge-delivery` → `dev`
> **分析日期：** 2026-08-31
> **模式：** 只读（CLI: `gf`）

## 一、PR #268 CI 状态

分支 `feat/265-gf-workflow-local-merge-delivery` 共 2 个 workflow run：

| Run ID | Workflow | 状态 | 结论 |
|--------|----------|------|------|
| 33358261401 | Smoke Test (gitcode / gitlab / github) | completed | ✅ success（3/3 job 全绿） |
| 33358261387 | Check / Lint / MSRV / Smoke Test / Test (macos/ubuntu/windows) | running | 🕒 进行中，已完成 job 全部 success（MSRV、Smoke Test、Check），其余（Lint、Test×3 平台）仍在执行，**截至采集时无任何失败 job** |

> `gf pipeline report --branch feat/265-gf-workflow-local-merge-delivery --days 14` 初次返回 `successRate: 0.0`，原因是统计口径把「仍在 running、尚无 conclusion」的 run 计入非成功。逐 job 核实（`gf pipeline jobs`）后确认：**当前无真实失败，只是一个 run 尚未跑完**。PR 本身状态为 `closed`（已合并/关闭）。

## 二、失败归因

无。本轮未发现失败 job，无需归因分析。

## 三、dev / main 基线（14 天）

| 分支 | Total runs | Success rate | Avg duration | 评级 |
|------|-----------:|--------------:|--------------:|------|
| `dev` | 100 | 94% | 152.2s | 🟡 Watch（80–94% 区间，接近健康线） |
| `main` | 93 | 100% | 163.6s | 🟢 Healthy |

`dev` 的 `topFailures` 字段仅返回通用标签 `"failure"`，`gf` CLI 未提供更细粒度的失败 job 名称，无法在不引入额外抓取（如遍历所有失败 run 的 `pipeline jobs`）的情况下做进一步归因；本次未做扩大抽样。

## 四、结论

- PR #268 相关的两个 workflow run：1 个已成功完成，1 个仍在执行中，**已完成的 job 无一失败**，未观察到构建 / 测试 / lint 问题。
- `dev` 分支近 14 天成功率 94%，处于 🟡 Watch 区间但未跌破 80% 告警线；`main` 分支 100% 健康。
- 建议：待 33358261387 跑完后复核 `Lint` 与三平台 `Test` job 的最终结论；若后续 `dev` 成功率持续走低，需扩大抽样定位具体失败 job（目前 CLI 返回的 `topFailures` 信息量不足）。

## 五、Recommendations

1. 🟢 **Low** — 无需干预，PR #268 交付面暂无异常；等待剩余 run 完成后二次确认全绿。
2. 🟡 **Medium** — 关注 `dev` 分支成功率（94%），若连续多轮低于 95% 建议做失败模式深挖。
