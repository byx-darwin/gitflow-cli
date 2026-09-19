# Pipeline Analysis Report — 2026-09-19 (wf-2026-09-19-004, Issue #336, Phase 4 post-delivery)

## 背景

Issue #336（gf-workflow Phase 3 执行引擎错误分类表，纯 specs/skills 文档变更）
经 `gf-workflow` 通过本地合并（`local_merge`）合入 `dev`，合并提交
`6e95d390d38c6c69b2be581b016ca12045873032`。与 Issue #333、#334、#335 交付时
相同，`dev` push 不触发 CI（`dev-push-does-not-trigger-ci` MEMORY 记录）、
`local_merge` 路径完全绕过 CI，因此本次交付本身**没有产生针对该变更的新 CI
运行**——`gf pipeline status --branch dev` 最新一条运行仍是
2026-09-19T07:47:39Z 的例行 `success`（run id `35430286802`），早于本次合并
提交，与该变更无关。本报告继续沿用前序报告的方式，对仓库近期流水线健康度
做一般性快照分析，并重点复核上一份报告（`wf-2026-09-19-003`，Issue #335）
提出的第 3 次 escalation 及其结果——follow-up Issue #373。

## 三维分析

### 1. 成功率趋势

| 分支 | 周期 | Total runs | Success rate | Avg duration | 评级 |
|------|------|-----------|---------------|---------------|------|
| `dev` | 30 天 | 100 | 96.0% | 126.50s | 🟢 Healthy |
| `dev` | 14 天 | 16 | 100.0% | 72.25s | 🟢 Healthy |
| `dev` | 7 天 | 8 | 100.0% | 70.38s | 🟢 Healthy |
| `main` | 30 天 | 69 | 100.0% | 146.30s | 🟢 Healthy |

`gf pipeline status --branch dev` 最近运行序列（2026-09-14 ~ 2026-09-19）全部
为 `success`，与前序报告观测一致：近期无新增失败，30 天窗口的 4% 失败率仍
集中在更早、超出 `status` 分页范围的历史运行中。

### 2. 失败模式

- 30 天窗口内唯一记录的 top failure 仍为 `Test (windows-latest)`，与
  `pr323`、`pr326`、`issue333`、`issue334`、`issue335` 报告中反复出现的
  `Test (windows-latest)` / `Test (macos-latest)` 收尾延迟/失败现象**数值上
  完全一致（同一条历史记录，未见新发生）**。
- 14 天与 7 天窗口（`dev`）、30 天窗口（`main`）均无失败记录——自上次报告
  以来没有产生新的该模式实例，模式本身未见恶化，也未见新的其他失败类型。

### 3. 耗时分布

- `dev` 30 天平均 126.50s，14 天平均 72.25s，7 天平均 70.38s，与
  `wf-2026-09-19-003` 报告的数值完全一致（同一时间窗口内无新运行样本变化）——
  短窗口显著低于 30 天窗口，说明历史失败/超时运行拉高了长窗口均值，近期常规
  运行本身耗时稳定且较低。
- `main` 30 天平均 146.30s，高于 `dev`，与预期一致（release 相关门禁更完整）。
- 未见新的单一显著耗时异常任务；无需立即处理的新增瓶颈。

## Escalation 检查（连续同水位复述规则）与 Issue #373 跟踪

`dev` 分支评级已连续 **9 次**（`pr320`、`pr321`、`issue324`、`pr323`、
`pr326`、`issue333`、`issue334`、`issue335`、本次 `wf-2026-09-19-004`）保持
🟢 Healthy（95.0% ~ 100.0%），整体健康度无异常。

关于 `Test (windows-latest)` / `Test (macos-latest)` job 收尾不稳定模式：

- `wf-2026-09-19-003`（Issue #335）报告已完成**第 3 次升级**，明确要求用户
  通过 `/gf-issue-create` 建立 follow-up Issue 或直接排查。
- 复核结果：**该 escalation 已产生实际行动** —— Issue
  [#373](https://github.com/byx-darwin/gitflow-cli/issues/373)
  `fix(ci): Windows/macOS Test job 收尾不稳定，连续 5 份 pipeline 报告未处理`
  已于 2026-09-19T10:50:49Z 创建（`bug` 标签，状态 `open`，作者
  `byx-darwin`），验收标准包含补充 job 级 timeout/诊断日志上传、根因定位
  （资源限制 vs 真实 flaky）、对应修复、以及"后续至少 3 次 pipeline 分析
  报告不再出现该模式的新记录"作为关闭条件。
- Issue #373 当前**尚无评论、尚无后续更新**（`createdAt` == `updatedAt` ==
  `2026-09-19T10:50:49Z`），符合其刚创建同日的预期，不构成异常。
- 自 Issue #373 创建以来，`dev`（7 天/14 天窗口）与 `main`（30 天窗口）**均
  未观测到 `Test (windows-latest)` / `Test (macos-latest)` 新发生的失败记录**
  ——30 天窗口中的唯一记录仍是此前报告已计入的同一条历史数据，未见新增。

**结论**：本次不再重复触发 escalation。上一轮升级已经产生了实质性的
remediation 动作（follow-up Issue #373 已立项，并带有可证伪的关闭条件），
且模式在本报告周期内未见新发生。后续报告应继续对照 Issue #373 的验收标准
（尤其是"后续至少 3 次 pipeline 分析报告不再出现该模式的新记录"），本次
计为该条件下的**第 1 次**"未再出现"的报告；若该模式连续 3 份报告
（含本次）均无新发生记录，且 Issue #373 完成对应验证项，可视为该 acceptance
criterion 满足。

## 改进建议（按优先级）

1. **[P2 跟踪]** 继续跟踪 Issue #373 的进展（job 级 timeout/诊断日志补充、
   根因定位、修复落地），后续报告应确认该问题是否解决，并计入其验收标准
   "连续 3 次报告无新记录"的计数（本次为第 1 次）。
2. **[P3 记录]** 本次 Issue #336 的交付（local_merge → dev，无 push）不会在
   CI 历史中留痕，属预期行为；如需该变更的 CI 证据，需后续通过 push/PR 触发。
3. **[P3 观察]** `dev`/`main` 整体健康度已连续 9 次报告保持 🟢 Healthy，
   无需额外行动；继续按现有节奏做常规快照分析即可。

## 数据来源

```
gf auth status
gf pipeline report --branch dev --days 30
gf pipeline report --branch dev --days 14
gf pipeline report --branch dev --days 7
gf pipeline report --branch main --days 30
gf pipeline status --branch dev
gf issue view 373
gf issue comments 373
```

- 采集时间：2026-09-19
- 平台：GitHub
- 工具：`gf` CLI（`./target/debug/gf`，认证用户 `byx-darwin`）
- 触发上下文：Issue #336 合并提交
  `6e95d390d38c6c69b2be581b016ca12045873032`（local merge 至 `dev`，docs-only
  变更，无 CI 运行产生）
- 前序报告：`docs/pipeline-report-wf-2026-09-19-003.md`（Issue #335，第 3 次
  escalation，触发 Issue #373 创建）
