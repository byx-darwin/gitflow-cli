# Pipeline 分析报告 — PR #323

> **PR：** [#323 fix(gf-workflow): compute worktree symlink depth dynamically instead of hardcoding ../../](https://github.com/byx-darwin/gitflow-cli/pull/323)
> **分支：** `feat/322-worktree-symlink-depth-fix` → `dev`（对应 Issue #322）
> **快照时间：** 2026-09-05T09:39:37Z（`Test (ubuntu-latest)` 收尾时点；`Test (windows-latest)`、`Test (macos-latest)` 在本报告采集截止时**仍处于 `in_progress`**，见下方“数据完整性说明”）
> **分析日期：** 2026-09-05
> **模式：** 只读（CLI: `gf`，未使用 `gh` 做写操作；`gh` 仅用于交叉核对只读的 checks/PR 元数据）
> **变更性质：** Skill/文档流程变更——修复 `gf-workflow` skill 中 worktree 共享符号链接深度硬编码为 `../../` 的问题，改为动态计算相对深度。变更文件：`skills/gf-workflow/SKILL.md`、`skills/gf-workflow/references.md`，以及 `docs/superpowers/plans/2026-09-04-worktree-symlink-depth-fix.md`、`docs/superpowers/specs/2026-09-04-worktree-symlink-depth-fix-design.md`。**未改动任何 Rust 代码、`Cargo.toml`/lockfile 或 `.github/workflows/*.yml` 文件。**

## 零、核心结论先行

`feat/322-worktree-symlink-depth-fix` 分支触发 2 个 workflow run（均创建于 `2026-09-05T09:36:30Z`），共 10 个 job。截至本报告采集截止时点，**8/10 job 已收尾且全部 `success`**；`Test (windows-latest)`、`Test (macos-latest)` 仍在 `in_progress`（历史上这两个 job 一贯是全流水线中耗时最长的，参见第五节历史区间，尚未观察到失败迹象）。`gh pr view 323` 显示 PR 已 `MERGED`（`mergedAt: 2026-09-05T09:36:43Z`，创建后约 16 秒即完成合并），早于全部 CI 收尾——这是继 PR #311→#321 系列报告七次记录之后的**再次复现**，与既往结论一致：`mergedAt` 反映的是 auto-merge 队列动作完成，不代表流水线已全部收尾。

**本次唯一需要升级的发现**：`gf pipeline report` 在 run 处于 `running` 状态时快照会将未收尾 job 计入失败桶，导致成功率虚假偏低（本次两次快照分别得到 `0%` 和 `50%`，见第七节）。该问题已连续在 **PR #311、#312、#313、#315、#316、#317、#320、#321、#323 共 9 次报告**中复现，且自最早记录以来**始终未被立案修复**。按 `gf-pipeline-analyzer` 技能的升级规则（≥3 次连续同类发现且无补救措施），本报告将其从「观察项」升级为**必须提请用户决策的阻断性建议**：应尽快针对 `gf` CLI 提交独立 Issue，将 `running`/`queued` 状态与真实 `failure` 分开统计。

## 一、PR #323 关联流水线实测

`feat/322-worktree-symlink-depth-fix` 分支触发 2 个 workflow run（`gf pipeline status --branch` 核对，无第三个 run）：

| Run ID | Workflow | 收尾状态（采集截止时点） | 备注 |
|--------|----------|------|------|
| 33958444026 | 主 CI（Check/MSRV/Lint/Smoke Test/Test×3） | 🟡 5/7 job success，2 个仍 `in_progress` | `Test (windows-latest)`、`Test (macos-latest)` 未收尾 |
| 33958444039 | Smoke Test 跨平台（github/gitlab/gitcode） | ✅ success（3/3 job 全部成功） | 最先收尾（09:37:45Z） |

Job 明细（`gh run view` / `gf pipeline jobs` 交叉核对）：

| Job | Workflow run | 结论 | 状态 |
|-----|--------------|------|------|
| MSRV | 33958444026 | ✅ success | 已收尾（56s） |
| Smoke Test (github) | 33958444039 | ✅ success | 已收尾（69s） |
| Smoke Test (gitlab) | 33958444039 | ✅ success | 已收尾（69s） |
| Smoke Test (gitcode) | 33958444039 | ✅ success | 已收尾（72s） |
| Check | 33958444026 | ✅ success | 已收尾（1m40s） |
| Lint | 33958444026 | ✅ success | 已收尾（2m20s） |
| Smoke Test | 33958444026 | ✅ success | 已收尾（3m2s） |
| Test (ubuntu-latest) | 33958444026 | ✅ success | 已收尾（3m4s，09:39:37Z） |
| Test (macos-latest) | 33958444026 | ⏳ 未知 | 仍 `in_progress`（已运行 >3 分钟） |
| Test (windows-latest) | 33958444026 | ⏳ 未知 | 仍 `in_progress`（已运行 >3 分钟） |

**8/10 job 已收尾，全部 `success`；2 个 job（历史上一贯是全流水线最慢的两个）在报告截止时仍在执行，尚无失败迹象。**

## 二、变更范围核实

`gh pr view 323 --json files` 确认改动文件：

```
docs/superpowers/plans/2026-09-04-worktree-symlink-depth-fix.md
docs/superpowers/specs/2026-09-04-worktree-symlink-depth-fix-design.md
skills/gf-workflow/SKILL.md
skills/gf-workflow/references.md
```

全部为 `skills/**`/`docs/**` 下文件，**未改动任何 Rust 代码、Cargo 清单或 CI workflow 配置**，与 PR 标题/描述一致。

## 三、E2E Tests workflow 未触发核实

与 PR #321 报告相同结论：`.github/workflows/e2e-tests.yml` 的 `pull_request.paths` 过滤器不包含 `skills/**`、`docs/superpowers/**`，因此 `E2E Tests` workflow 未被触发——`gf pipeline status --branch feat/322-worktree-symlink-depth-fix` 核实仅有 2 个 run（主 CI + 跨平台 Smoke Test），无第三个 `E2E Tests` run。**这是预期且正确的行为**，不构成 CI 覆盖缺口。

## 四、PR 合并状态说明

`gh pr view 323` 返回 `state: MERGED`、`createdAt: 2026-09-05T09:36:27Z`、`mergedAt: 2026-09-05T09:36:43Z`——PR 在创建后约 16 秒即完成合并，早于其触发的全部 CI 收尾（截至本报告采集截止时点，`Test (windows-latest)`/`Test (macos-latest)` 仍未收尾）。这与既往系列报告（PR #311→#321，共八次）记录的「auto-merge 排队等待必需检查通过」模式一致：`gh pr view`/`gf pr view` 在合并动作完成后立即返回 `MERGED`/`mergedAt`，不代表流水线已全部收尾。**本报告在剩余两个最长耗时 job 仍未收尾的情况下提交**（应协调方要求停止长时间轮询），因此**无法像 PR #321 报告那样最终确证全部 job 均为 success**——已收尾的 8 个 job 全部 `success`，未收尾的 2 个尚无失败信号，但结论应标注为"截至采集时点未发现门禁被绕过或失败证据"，而非"已完全确证"。

## 五、耗时分析

| Job | 本次耗时（已收尾部分） | 既往历史区间参考（PR #311–#321） | 评估 |
|-----|---------:|:---:|:---:|
| MSRV | 56s | 53s–57s | 区间内 |
| Check | 1m40s | 1m21s–1m28s | 略高，区间内 |
| Lint | 2m20s | 2m21s | 一致 |
| Smoke Test | 3m2s | 2m54s–3m11s | 区间内 |
| Test (ubuntu-latest) | 3m4s (184s) | 116s–337s / 283s–297s | 区间内 |
| Test (macos-latest) | 未收尾，已运行 >3 分钟 | 256s–388s | 无法评估，观察中 |
| Test (windows-latest) | 未收尾，已运行 >3 分钟 | 485s–616s（历史峰值 616s） | 无法评估，观察中 |

由于报告在两个最慢 job 收尾前提交，本次无法与既往报告一样给出完整耗时结论。已收尾的 8 个 job 耗时均在既往历史区间内，无异常。

## 六、dev / main 基线

| 分支 | 周期 | Total runs | Success rate | Avg duration | 评级 |
|------|------|-----------:|--------------:|--------------:|------|
| `dev` | 7 天 | 96 | 95.8% | 150.51s | 🟢 Healthy |
| `main` | 30 天 | 100 | 100.0% | 160.85s | 🟢 Healthy |

基线数值与既往系列报告基本一致（`dev` 95.8% vs 此前 95.0%，波动在正常范围），无异常。

## 七、Flaky / 失败信号 与 采集时机口径问题（升级项）

`gf pipeline report --branch feat/322-worktree-symlink-depth-fix --days 30` 两次快照对比：

**第一次快照（run 仍在排队/刚启动，`totalRuns: 2` 均未收尾）**：
```json
{"totalRuns": 2, "successRate": 0.0, "avgDurationSecs": 8.5, "topFailures": ["MSRV", "Smoke Test (github)"]}
```

**第二次快照（部分 job 收尾，仍有 2 个 job 处于 `in_progress`）**：
```json
{"totalRuns": 2, "successRate": 0.5, "avgDurationSecs": 44.5, "topFailures": ["Test (windows-latest)"]}
```

两次快照均与实测（8/10 job 全部 `success`，另 2 个尚在执行、无失败信号）不符：第一次快照甚至将已经 `success` 的 `MSRV` job 错误列入 `topFailures`。这与 PR #311→#321 系列报告记录的「`gf pipeline report` 在 run 处于 `running`/`queued` 时会将其计入失败桶」问题**完全一致地再次复现**，且是该问题第 **9 次** 被记录（PR #311、#312、#313、#315、#316、#317、#320、#321、#323）。

**升级说明（按 `gf-pipeline-analyzer` 技能的 Escalation Rule）**：该问题已连续 ≥3 次（实际已达 9 次）复现，且自最早记录以来**未见任何补救措施落地**（未提交修复 Issue、未合入修复）。因此本报告不再将其列为普通观察项，而是明确提请用户在两条路径中二选一：
1. 通过 `/gf-issue-create`（需人工触发，本技能不代为创建）提交一个独立 Issue，要求 `gf pipeline report` 将 `running`/`queued` 状态与真实 `failure` 分开统计；或
2. 直接安排修复该 CLI 行为。

在此之前，任何依赖 `gf pipeline report` 单次快照做健康度判断的场景都存在误报风险，尤其是在 PR 刚创建、run 仍在排队/执行阶段时采集。

## 八、结论与 Recommendations

1. 🟢 **无阻断性发现** — 已收尾的 8/10 job 全部 `success`，无失败、无跳过；未收尾的 2 个 job（`Test (windows-latest)`、`Test (macos-latest)`）在报告截止时尚无失败信号。
2. 🟢 **无阻断性发现** — 变更范围核实为 `skills/gf-workflow/**` 与 `docs/superpowers/**` 下 4 个文件，未改动任何 Rust 代码、Cargo 清单或 `.github/workflows/*.yml`；`E2E Tests` workflow 因路径过滤器不含改动路径未被触发，属预期行为。
3. 🟡 **Low（数据完整性说明）** — 本报告在 `Test (windows-latest)`/`Test (macos-latest)` 仍处于 `in_progress` 时提交（应协调方要求结束长时间轮询）。这两个 job 的最终结论未在本报告中确证，建议后续如有需要可再次核对 run 33958444026 的最终状态。
4. ⚠️ **Medium → 升级为需用户决策项** — `gf pipeline report` 在 run 处于 `running`/`queued` 状态时的快照会产生虚假低成功率（本次两次快照分别为 0% 与 50%，且误将成功 job 列入 `topFailures`）。此问题已连续 9 次复现（PR #311→#323）且始终未有补救措施。**请用户决定**：是否通过 `/gf-issue-create` 提交修复 Issue，或直接安排修复；本技能保持只读，不代为创建 Issue。
5. 🟢 **合并流程观察（非阻断，延续既往模式）** — PR 合并时点（09:36:43Z）早于全部 CI 收尾，与既往系列报告记录的 auto-merge 排队模式一致；已收尾部分未见门禁被绕过或失败证据。
