# Pipeline Analysis Report — 2026-09-19 (Issue #333, Phase 4 post-delivery)

## 背景

Issue #333（feat: gf-quality/gf-pr-review evidence grading）经 `gf-workflow` 通过
`local_merge` 合入 `dev`，合并提交 `ecd533a521e401e154032aaf83d14a75338edb47`。
因未执行 `git push`，本次交付**没有产生针对该变更的新 CI 运行**（参见
`gf-workflow-claude-symlink-depth-off-by-one` 相关记录：`dev push 不触发 CI`
系有意排除 dev push；local_merge 完全绕过 CI 早已在
`local_merge` 相关 MEMORY 条目中记录）。本报告因此改为对仓库近期流水线健康度做
一般性快照分析，作为 Phase 4 交付后检查的替代证据。

## 三维分析

### 1. 成功率趋势

| 分支 | 周期 | Total runs | Success rate | Avg duration | 评级 |
|------|------|-----------|---------------|---------------|------|
| `dev` | 30 天 | 100 | 96.0% | 133.36s | 🟢 Healthy |
| `dev` | 7 天 | 8 | 100.0% | 69.88s | 🟢 Healthy |
| `main` | 30 天 | 70 | 100.0% | 146.77s | 🟢 Healthy |

`dev` 分支最近 10 次运行（`gf pipeline status --branch dev`，2026-09-09 ~
2026-09-18）全部 `success`，7 天窗口成功率回升至 100%，30 天窗口的 4% 失败率
集中在早前的运行中，近期无新增失败。

### 2. 失败模式

- 30 天窗口内唯一记录的 top failure 为 `Test (windows-latest)`，与此前多份报告
  （`pr323`、`pr326`）中反复出现的 `Test (windows-latest)` / `Test (macos-latest)`
  收尾延迟/失败现象一致——这是一个跨报告持续出现的模式，而非单次偶发。
- 7 天窗口（`dev`）与 30 天窗口（`main`）均无失败记录，说明该问题目前发生频率
  低、非持续性，符合"flaky"而非"persistent"的判定标准（连续 ≥3 次才算 persistent，
  本次不满足）。

### 3. 耗时分布

- `dev` 30 天平均 133.36s，7 天平均 69.88s（明显下降，可能是近期改动使 CI 更轻量，
  或近期 push 次数少、样本更集中在小改动上）。
- `main` 30 天平均 146.77s，高于 `dev`，符合 main 分支通常运行更完整门禁
  （release-related jobs）的预期。
- 无单一显著异常耗时任务被数据标出；未见明显瓶颈需要立即处理。

## Escalation 检查（连续同水位复述规则）

复核 `docs/` 下最近 5 份历史报告（`pr320`、`pr321`、`issue324`、`pr323`、
`pr326`，均为 2026-09-04 ~ 2026-09-06）与本次报告，`dev` 分支评级已连续
**6 次**保持 🟢 Healthy（95.0% ~ 100.0%），期间未见任何针对性修复或 follow-up
Issue 记录在案。同时，`Test (windows-latest)` / `Test (macos-latest)` 的收尾
异常/失败模式也跨越了 `pr323`、`pr326` 与本次报告，同样未见修复动作。

**按 escalation rule 升级提示**：虽然整体评级健康，不构成紧急阻塞，但
「Windows/macOS Test job 收尾不稳定」这一模式已连续观测 3 次以上且始终未处理。
建议用户后续通过 `/gf-issue-create`（需人工触发，本 skill 不自动建 Issue）
针对该 job 收尾稳定性单独立项跟踪，或直接安排排查。

## 改进建议（按优先级）

1. **[P2 建议]** 为 `Test (windows-latest)` / `Test (macos-latest)` job 补充
   超时与收尾诊断（如 job 级 timeout、失败时上传诊断日志），便于下次复现时
   定位是资源限制还是测试本身不稳定。
2. **[P3 观察]** `dev` 7 天窗口耗时显著低于 30 天窗口（69.88s vs 133.36s），
   建议持续观察是否为样本量过小（仅 8 次）导致的统计噪音，而非真实改善。
3. **[P3 记录]** 本次 Issue #333 的交付（local_merge → dev，无 push）不会在
   CI 历史中留痕，属预期行为；如需该变更的 CI 证据，需后续通过 push/PR 触发。

## 数据来源

```
gf pipeline report --branch dev --days 30
gf pipeline report --branch dev --days 7
gf pipeline report --branch main --days 30
gf pipeline status --branch dev
```

- 采集时间：2026-09-19
- 平台：GitHub
- 工具：`gf` CLI（认证用户 `byx-darwin`）
