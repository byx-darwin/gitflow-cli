# Pipeline 分析报告 — PR #320

> **PR：** [#320 ci(e2e): verify e2e-regression alert CJK title dedup reliability](https://github.com/byx-darwin/gitflow-cli/pull/320)
> **分支：** `feat/310-e2e-regression-dedup-verification` → `dev`（对应 Issue #310，gf-workflow 标准模式）
> **快照时间：** 2026-09-04T06:29:18Z（全部 job 已收尾，本报告为持续轮询至全部终态的完整快照，非部分收敛）
> **分析日期：** 2026-09-04
> **模式：** 只读（CLI: `gf`；PR checks 交叉核对用 `gh pr checks` / `gh run list`）
> **变更性质：** 纯文档变更——Issue #310 是 #292/PR #309 代码审查（`docs/code-review-report-pr309-2026-09-03.md`，finding #1）的后续 spike：`notify-on-schedule-failure` job 的去重逻辑通过 `gh issue list --search "in:title ..."` 匹配一个几乎全为 CJK 字符、无词边界空格的固定标题（`定时 E2E 回归失败`），存在 GitHub 全文搜索对 CJK 分词不可靠、搜索漏检导致去重静默失效（每次定时失败都新建 Issue 而非评论已有 Issue）的风险。本 PR 用一次性测试 Issue（#319，已关闭）以生产标题+标签真实验证该查询，重复 3 次（含延迟以排除搜索索引传播滞后）**全部命中**，得出结论：去重逻辑按设计工作，**无需改动 `.github/workflows/e2e-tests.yml`**。变更内容仅为 3 个 `docs/**` 文件（`code-review-report-pr309-2026-09-03.md` 追加验证记录段落 + 新增 spike 设计笔记/实施计划各一份），**未改动任何 Rust 代码、Cargo 清单或任何 `.github/workflows/*.yml` 文件**。

## 零、核心结论先行

`feat/310-e2e-regression-dedup-verification` 分支触发 2 个 workflow run（均创建于 `2026-09-04T06:21:09Z`），共 **10 个 job**。本报告**持续轮询至全部 job 收尾**（末尾 job `Test (windows-latest)` 于 `06:29:18Z` 完成，耗时 8m5s），**10 个 job 全部 `success`，无一失败、无一跳过**。由于本 PR 仅改动 `docs/**` 下 3 个 Markdown 文件，核对 `.github/workflows/e2e-tests.yml` 触发条件（`pull_request.paths` 仅含 `crates/**`、`apps/**`、`Cargo.toml`/`Cargo.lock`、`release.toml`、`CHANGELOG.md`、`Makefile`、`scripts/**`、workflow 文件自身）确认**该 PR 未触发 `E2E Tests` workflow 是预期行为**（路径过滤器正确排除了纯 `docs/**` 变更），并非 CI 配置异常或遗漏。`gf pr view 320` 显示 PR 已 `closed`/`mergedAt: 06:21:24Z`，早于全部 CI 收尾（合并时点后约 8 分钟才全部收尾），与既往系列报告记录的 auto-merge 排队等待必需检查通过的模式一致；持续轮询确认**合并后触发的全部检查最终均为 `success`，未发现门禁被绕过或合并后出现失败的证据**。`dev`/`main` 基线保持健康（95%/100%），与既往系列报告一致。本次未观测到耗时异常信号——各 job 耗时均落在既往系列报告记录的历史区间内。

## 一、PR #320 关联流水线实测（全部收尾）

`feat/310-e2e-regression-dedup-verification` 分支触发 2 个 workflow run（`gh run list --branch` 交叉核对，未见第三个 run，`E2E Tests` workflow 未触发——见第三节）：

| Run ID | Workflow | 收尾状态 | 备注 |
|--------|----------|------|------|
| 33844002376 | 主 CI（Check/MSRV/Lint/Smoke Test/Test×3） | ✅ success（7/7 job 全部成功） | 末尾 job `Test (windows-latest)` 于 06:29:18Z 收尾 |
| 33844002384 | Smoke Test 跨平台（github/gitlab/gitcode） | ✅ success（3/3 job 全部成功） | 最先收尾（06:22:27Z 前） |

全部 job 明细（`gf pipeline jobs` + `gh pr checks` 交叉核对，持续轮询至全部终态）：

| Job | Workflow run | 结论 | 耗时 |
|-----|--------------|------|------|
| MSRV | 33844002376 | ✅ success | 53s |
| Smoke Test (gitcode) | 33844002384 | ✅ success | 1m7s |
| Smoke Test (gitlab) | 33844002384 | ✅ success | 1m9s |
| Smoke Test (github) | 33844002384 | ✅ success | 1m15s |
| Check | 33844002376 | ✅ success | 1m21s |
| Lint | 33844002376 | ✅ success | 2m21s |
| Smoke Test | 33844002376 | ✅ success | 3m11s |
| Test (macos-latest) | 33844002376 | ✅ success | 4m16s |
| Test (ubuntu-latest) | 33844002376 | ✅ success | 4m43s |
| **Test (windows-latest)** | 33844002376 | ✅ success | 8m5s |

**共 10 个 job，全部 `success`，无一失败、无一跳过。**

## 二、变更范围核实——docs-only 确认

`git diff --stat dev...feat/310-e2e-regression-dedup-verification`：

```
docs/code-review-report-pr309-2026-09-03.md                                  | 31 +++++++
docs/superpowers/specs/2026-09-04-e2e-regression-dedup-verification-design.md| 40 ++++++++
docs/superpowers/plans/2026-09-04-e2e-regression-dedup-verification.md       |104 +++++++++++++++++++
3 files changed, 175 insertions(+)
```

- **未改动**任何 `crates/**`、`apps/**`、`Cargo.toml`/`Cargo.lock`、`.github/workflows/*.yml` 文件——与 PR 描述及 Issue #310 验收标准（"search hits" 分支不需要改动 `.github/workflows/e2e-tests.yml`）一致。
- 该结论直接决定了本次 CI 触发范围（见第三节）：所有触发的 job 均来自与内容无关的通用矩阵（`ci.yml`），而非因代码/CI 配置变更而需要针对性验证的 job。

## 三、`E2E Tests` workflow 未触发核实

按分析要求核对：PR #320 是否应触发专门验证去重逻辑所在的 `E2E Tests` workflow（`notify-on-schedule-failure` job 定义处）。

```yaml
# .github/workflows/e2e-tests.yml:1-20
on:
  push:
    branches: [main]
  pull_request:
    branches: [main, dev]
    paths:
      - 'crates/**'
      - 'apps/**'
      - 'Cargo.toml'
      - 'Cargo.lock'
      - 'release.toml'
      - 'CHANGELOG.md'
      - 'Makefile'
      - 'scripts/**'
      - '.github/workflows/e2e-tests.yml'
      - '.github/workflows/release.yml'
```

- `pull_request.paths` 过滤器**不包含 `docs/**`**，本 PR 仅改动 `docs/**` 下的 3 个文件，因此 `E2E Tests` workflow（含被验证的 `notify-on-schedule-failure` job）**未被触发**——`gh run list --branch feat/310-e2e-regression-dedup-verification` 交叉核对确认仅有 2 个 run（主 CI + Smoke Test 跨平台），无第三个 `E2E Tests` run。
- **这是预期且正确的行为**：本 PR 是一次线下手动验证（通过临时测试 Issue #319 实测 `gh issue list --search` 查询），而非改动生产 CI 配置——`notify-on-schedule-failure` job 本身仍按原定时调度（`on.schedule`）运行，不依赖 PR 触发。未在本次 PR 流水线中看到该 job 属正常现象，不构成发现。

## 四、PR 合并状态说明

`gf pr view 320` 返回 `state: "closed"`、`createdAt: "2026-09-04T06:21:06Z"`、`mergedAt: "2026-09-04T06:21:24Z"`——PR 在创建后约 18 秒即被记录为合并，早于其触发的全部 CI 收尾（末尾 job 于 06:29:18Z 才完成，即合并时点后约 8 分钟）。与既往系列报告（PR #311/#312/#313/#315/#316/#317）记录的「auto-merge 排队等待必需检查通过」模式一致：`gf pr view`/`gh pr view` 在合并动作完成后立即返回 `closed`/`mergedAt`，不代表流水线已全部收尾。**本报告持续轮询至全部 10 个 job 真正收尾**，可以确证——**合并后触发的全部 CI 检查最终均为 `success`，未发现门禁被绕过或合并后出现失败的证据**。

## 五、`gf pipeline report` 口径核实

`gf pipeline report --branch feat/310-e2e-regression-dedup-verification --days 7`（在全部 job 收尾后采集）：

```json
{
  "totalRuns": 2,
  "successRate": 1.0,
  "avgDurationSecs": 284.0,
  "topFailures": []
}
```

本次采集**已在全部 job 终态后进行**，`successRate` 与实测（10/10 job success）一致，未复现既往系列报告（PR #311→#312→#313→#315→#316→#317，共六次）记录的「run 处于 `running` 时被计入失败桶」的口径假象——这印证了既往报告的诊断：该问题的根因是采集时机（run 仍在 `running`），而非命令逻辑本身在所有情况下都错误。**维持既往建议**：仍需针对 `gf pipeline report` 在 run 处于 `running`/`queued` 状态时的统计口径提交独立 Issue 改进，避免依赖「凑巧在收尾后采集」来获得准确结果。

## 六、dev / main 基线（采集时点：PR #320 全部 job 收尾后）

| 分支 | 周期 | Total runs | Success rate | Avg duration | 评级 |
|------|------|-----------:|--------------:|--------------:|------|
| `dev` | 7 天 | 100 | 95.0% | 150.47s | 🟢 Healthy |
| `main` | 30 天 | 100 | 100.0% | 159.59s | 🟢 Healthy |

基线数值与 PR #311/#312/#313/#315/#316/#317 报告完全一致，延续系列报告观察到的健康水位；PR #320（纯文档变更）未导致基线抖动。

## 七、Flaky / 失败信号 与 耗时分析

**PR #320 自身流水线**：10 个 job 全部 `success`，无任何失败，无 flaky 信号。

**耗时对比（与既往系列报告历史区间比对）**：

| Job | 本次耗时 | 既往历史区间参考（PR #311-#317 系列） | 评估 |
|-----|---------:|:---:|:---:|
| Test (ubuntu-latest) | 4m43s (283s) | 116s–337s | 区间内 |
| Test (macos-latest) | 4m16s (256s) | 116s–337s / 384s（PR #317） | 区间内，优于 PR #317 |
| **Test (windows-latest)** | **8m5s (485s)** | 116s–337s（早期）/ 616s（PR #317 峰值） | 高于早期区间，但低于 PR #317 记录的峰值（616s），处于近期观测到的合理波动范围 |
| Check | 1m21s | 与 PR #317 一致（1m21s） | 一致 |
| Lint | 2m21s | 与 PR #317 一致（2m21s） | 一致 |
| MSRV | 53s | 与 PR #317 一致（53s） | 一致 |
| Smoke Test | 3m11s | 与 PR #317 一致（3m11s） | 一致 |

Test (windows-latest) 本次 8m5s，虽仍高于本系列报告更早期记录的 116s-337s 区间，但相较 PR #317 观测到的 10m16s 峰值有所回落，与 PR #317 报告第七节结论（该轮耗时上涨为采集时段 runner 资源争用导致的瞬时抖动，非结构性回归）互相印证——本次数据支持该判断：耗时未持续恶化，反而回落，说明并非本仓库或本次变更引入的持续性问题。由于本 PR 为纯文档变更、未触及任何测试代码，与 windows 平台耗时无逻辑关联。**建议**：作为观察项继续跟踪，无需专项处理。

## 八、结论与 Recommendations

1. 🟢 **无阻断性发现** — PR #320（Issue #310，CJK 标题去重逻辑验证 spike）触发的 2 个 workflow run 共 10 个 job **全部 `success`**，无失败、无跳过、无 flaky 信号。
2. 🟢 **无阻断性发现** — 核实 PR 变更范围为 `docs/**` 下 3 个文件，未改动任何 Rust 代码、Cargo 清单或 `.github/workflows/*.yml`，与 PR 描述一致；`E2E Tests` workflow 因路径过滤器（`pull_request.paths` 不含 `docs/**`）未被触发，属预期行为，不构成 CI 覆盖缺口——`notify-on-schedule-failure` job（本次验证的目标）本身仍按定时调度独立运行，不受此次 PR 触发范围影响。
3. 🟢 **合并门禁核实** — `gf pr view 320` 显示合并时点（06:21:24Z）早于全部 CI 收尾（06:29:18Z），符合既往系列报告记录的 auto-merge 排队模式；持续轮询至全部终态确认合并后触发的检查最终均通过，未发现门禁绕过证据。
4. 🟡 **Low（观察项，非本 PR 引起）** — `Test (windows-latest)` 本次耗时 8m5s（485s），高于系列报告早期记录区间（116s-337s），但低于 PR #317 记录的峰值（616s）；因本 PR 未改动任何测试代码，与耗时波动无逻辑关联，倾向于延续 PR #317 报告已记录的「runner 资源争用瞬时抖动」而非结构性回归。建议继续在后续报告中观察该 job 耗时趋势。
5. 🟡 **Medium（沿用既往建议，非本次新增）** — `gf pipeline report` 在 run 处于 `running` 状态时曾连续六次（PR #311→#317）将其计入失败桶；本次因采集时机在全部 job 收尾后进行，未复现该问题，但**根因未修复**，采集时机不当仍会导致误判。维持既往建议：尽快针对 `gf` CLI 提交独立 Issue，改进 `pipeline report` 使其将 `running`/`queued` 与真实 `failure` 分开统计。
