# Pipeline Analysis Report — 2026-09-19 (wf-2026-09-19-003, Issue #335, Phase 4 post-delivery)

## 背景

Issue #335（`gf-issue-create`/`gf-issue-review` 增加垂直切片 + 可证伪验收标准约束，
纯 skills/specs 文档变更）经 `gf-workflow` 通过本地合并（`local_merge`）合入 `dev`，
合并提交 `23505114b1431bae0618a4666f034e623dfc927e`。与 Issue #333、#334 交付时
相同，`dev` push 不触发 CI（`dev-push-does-not-trigger-ci` MEMORY 记录）、
`local_merge` 路径完全绕过 CI，因此本次交付本身**没有产生针对该变更的新 CI
运行**（`gf pipeline status --branch dev` 最新一条运行为 2026-09-19T07:47:39Z
的例行 `success`，早于本次合并提交，与该变更无关；变更所在的
`feat/335-vertical-slice-falsifiable-acceptance` 分支已随合并清理）。
本报告继续沿用 issue333/issue334 报告的方式，对仓库近期流水线健康度做一般性
快照分析，作为 Phase 4 交付后检查的替代证据。

## 三维分析

### 1. 成功率趋势

| 分支 | 周期 | Total runs | Success rate | Avg duration | 评级 |
|------|------|-----------|---------------|---------------|------|
| `dev` | 30 天 | 100 | 96.0% | 126.50s | 🟢 Healthy |
| `dev` | 14 天 | 16 | 100.0% | 72.25s | 🟢 Healthy |
| `dev` | 7 天 | 8 | 100.0% | 70.38s | 🟢 Healthy |
| `main` | 30 天 | 69 | 100.0% | 146.30s | 🟢 Healthy |

`gf pipeline status --branch dev` 返回的最近 29 条运行（2026-08-31 ~
2026-09-19）全部为 `success`，与 issue333/issue334 报告观测一致：近期无新增
失败，30 天窗口的 4% 失败率仍集中在更早（2026-08-20 ~ 2026-08-31 之间、超出
status 分页范围）的历史运行中。

### 2. 失败模式

- 30 天窗口内唯一记录的 top failure 依旧为 `Test (windows-latest)`，与
  issue333、issue334、`pr326`、`pr323` 报告中反复出现的 `Test (windows-latest)`
  / `Test (macos-latest)` 收尾延迟/失败现象一致——这是一个跨越至少 5 份报告
  持续出现的模式，而非单次偶发。
- 14 天与 7 天窗口（`dev`）、30 天窗口（`main`）均无失败记录，发生频率仍然
  低、非连续，按规则（连续 ≥3 次才判定 persistent）仍归类为 "flaky" 而非
  "persistent"。

### 3. 耗时分布

- `dev` 30 天平均 126.50s，14 天平均 72.25s，7 天平均 70.38s——短窗口显著低于
  30 天窗口，说明历史失败/超时运行（`Test (windows-latest)`）拉高了长窗口均值，
  近期常规运行本身耗时稳定且较低。
- `main` 30 天平均 146.30s，高于 `dev`，与预期一致（release 相关门禁更完整）。
- 未见新的单一显著耗时异常任务；无需立即处理的新增瓶颈。

## Escalation 检查（连续同水位复述规则）

复核历史报告序列：`pr320`、`pr321`、`issue324`、`pr323`、`pr326`
（2026-09-04 ~ 2026-09-06）、`issue333`、`issue334`（均 2026-09-19，本次之前
两次）与本次 `wf-2026-09-19-003`（Issue #335）报告，`dev` 分支评级已连续
**8 次**保持 🟢 Healthy（95.0% ~ 100.0%），期间仍未见任何针对性修复或
follow-up Issue 落地。

同时，`Test (windows-latest)` / `Test (macos-latest)` job 收尾不稳定的模式已
跨越 `pr323`、`pr326`、`issue333`、`issue334` 与本次报告——**这是该模式第 5
次被记录，此前 issue333、issue334 报告已各升级提示一次，至今仍未见任何修复
动作或 follow-up Issue**。

**按 escalation rule 再次升级（第 3 次）**：这不再是首次或第二次提示，而是对
同一未处理项的持续重复升级。建议用户不要再依赖后续报告的静默复述，而是：

1. 通过 `/gf-issue-create`（需人工触发，本 skill 不自动建 Issue）为
   "Windows/macOS Test job 收尾不稳定" 单独立项，标题建议包含
   `windows-latest` / `macos-latest` 关键词以便后续报告交叉引用；或
2. 直接安排排查（补充 job 级 timeout、失败时上传诊断日志），确认是资源限制
   还是测试本身的真实 flaky 行为。

在该 follow-up 落地之前，后续每一次流水线分析报告都应继续标注本 escalation
的累计次数，直至有实际修复或 Issue 记录为止。

## 改进建议（按优先级）

1. **[P1 升级 — 第 3 次]** `Test (windows-latest)` / `Test (macos-latest)`
   收尾不稳定已连续 5 份报告出现且经过两次 escalation 仍未处理，强烈建议
   立即创建 follow-up Issue 或直接排查，避免该问题在未来某次连续失败时才被
   动作（此时已从 flaky 累积证据转为需要主动介入的信号）。
2. **[P2 建议]** 为上述 job 补充超时与收尾诊断（job 级 timeout、失败时上传
   诊断日志），便于下次复现时快速定位是资源限制还是测试本身不稳定。
3. **[P3 记录]** 本次 Issue #335 的交付（local_merge → dev，无 push）不会在
   CI 历史中留痕，属预期行为；如需该变更的 CI 证据，需后续通过 push/PR 触发。

## 数据来源

```
gf pipeline report --branch dev --days 30
gf pipeline report --branch dev --days 14
gf pipeline report --branch dev --days 7
gf pipeline report --branch main --days 30
gf pipeline status --branch dev
```

- 采集时间：2026-09-19
- 平台：GitHub
- 工具：`gf` CLI（`./target/debug/gf`，认证用户 `byx-darwin`）
- 触发上下文：Issue #335 合并提交
  `23505114b1431bae0618a4666f034e623dfc927e`（local merge 至 `dev`，docs-only
  变更，无 CI 运行产生）
