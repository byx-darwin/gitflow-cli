# Pipeline 分析报告 — PR #321

> **PR：** [#321 fix(gf-workflow): guard worktree shared symlinks against accidental commit](https://github.com/byx-darwin/gitflow-cli/pull/321)
> **分支：** `feat/318-worktree-symlink-exclude-guard` → `dev`（对应 Issue #318）
> **快照时间：** 2026-09-04T07:18:46Z（全部 job 已收尾，本报告为持续轮询至全部终态的完整快照，非部分收敛期采集）
> **分析日期：** 2026-09-04
> **模式：** 只读（CLI: `gf`，未使用 `gh`）
> **变更性质：** Skill/文档流程变更——`gf-workflow` skill 的 Worktree Preflight 会在每个 worktree 内创建两个共享符号链接（`.cache/workflows`、`.claude`），此前未在 `info/exclude` 中排除，存在被 `git add -A` / `git commit -a` 误提交的风险（已在下游项目 `iproost/proxy/api-src` 的提交 `e7f4254` 中验证真实发生）。本 PR 在 `skills/gf-workflow/SKILL.md` Phase 3 Step 1 补充 `info/exclude` 写入、Step 3 新增预交付 `create mode 120000` 扫描守卫，并在 `references.md` 补充说明与新增设计/计划文档。**未改动任何 Rust 代码、`Cargo.toml`/lockfile 或 `.github/workflows/*.yml` 文件**（`git diff --stat dev...feat/318-worktree-symlink-exclude-guard`：4 files changed, 502 insertions(+), 2 deletions(-)，全部为 `skills/**`/`docs/**` 下文件）。

## 零、核心结论先行

`feat/318-worktree-symlink-exclude-guard` 分支触发 2 个 workflow run（均创建于 `2026-09-04T07:08:43Z`），共 **10 个 job**。本报告**持续轮询至全部 job 收尾**（末尾 job `Test (windows-latest)` 于 `07:18:45Z` 完成，耗时 9m59s），**10 个 job 全部 `success`，无一失败、无一跳过**。核对 `.github/workflows/e2e-tests.yml` 的 `pull_request.paths` 过滤器（仅含 `crates/**`、`apps/**`、`Cargo.toml`/`Cargo.lock`、`release.toml`、`CHANGELOG.md`、`Makefile`、`scripts/**`、`docs/compatibility-matrix.md`、workflow 文件自身），确认本 PR **未触发 `E2E Tests` workflow 属预期行为**（路径过滤器正确排除了 `skills/**`/`docs/**` 下的变更文件）。`gf pr view 321` 显示 PR 已 `closed`/`mergedAt: 07:09:31Z`，早于全部 CI 收尾（合并时点后约 9 分钟才全部收尾），与既往系列报告（PR #311→#320）记录的 auto-merge 排队等待必需检查通过的模式一致；持续轮询确认**合并后触发的全部检查最终均为 `success`，未发现门禁被绕过或合并后出现失败的证据**。`dev`/`main` 基线保持健康（95%/100%），与既往系列报告一致。本次唯一观察项：`Test (windows-latest)` 耗时 9m59s，高于系列报告早期区间（116s–337s）且逼近 PR #317 记录的峰值（616s），详见第五节。

## 一、PR #321 关联流水线实测（全部收尾）

`feat/318-worktree-symlink-exclude-guard` 分支触发 2 个 workflow run（`gf pipeline status --branch` 核对，未见第三个 run，`E2E Tests` workflow 未触发——见第三节）：

| Run ID | Workflow | 收尾状态 | 备注 |
|--------|----------|------|------|
| 33847393243 | 主 CI（Check/MSRV/Lint/Smoke Test/Test×3） | ✅ success（7/7 job 全部成功） | 末尾 job `Test (windows-latest)` 于 07:18:45Z 收尾 |
| 33847393249 | Smoke Test 跨平台（github/gitlab/gitcode） | ✅ success（3/3 job 全部成功） | 最先收尾（07:10:36Z） |

全部 job 明细（`gf pipeline jobs --pipeline-id`，持续轮询至全部终态）：

| Job | Workflow run | 结论 | 耗时 |
|-----|--------------|------|------|
| MSRV | 33847393243 | ✅ success | 57s |
| Smoke Test (gitcode) | 33847393249 | ✅ success | 1m9s |
| Smoke Test (gitlab) | 33847393249 | ✅ success | 1m15s |
| Check | 33847393243 | ✅ success | 1m28s |
| Smoke Test (github) | 33847393249 | ✅ success | 1m13s |
| Lint | 33847393243 | ✅ success | 2m21s |
| Smoke Test | 33847393243 | ✅ success | 2m54s |
| Test (ubuntu-latest) | 33847393243 | ✅ success | 4m57s |
| Test (macos-latest) | 33847393243 | ✅ success | 6m28s |
| **Test (windows-latest)** | 33847393243 | ✅ success | 9m59s |

**共 10 个 job，全部 `success`，无一失败、无一跳过。**

## 二、变更范围核实

`git diff --stat dev...feat/318-worktree-symlink-exclude-guard`：

```
docs/superpowers/plans/2026-09-04-worktree-symlink-exclude-guard.md          | 343 ++++++++++++
docs/superpowers/specs/2026-09-04-worktree-symlink-exclude-guard-design.md   | 109 ++++
skills/gf-workflow/SKILL.md                                                  |   4 +-
skills/gf-workflow/references.md                                             |  48 +++
4 files changed, 502 insertions(+), 2 deletions(-)
```

- **未改动**任何 `crates/**`、`apps/**`、`Cargo.toml`/`Cargo.lock`、`.github/workflows/*.yml` 文件——与 PR 描述一致（"Docs/skill-process-only change — no Rust code, no `Cargo.toml`/lockfile touched"）。
- 该结论直接决定了本次 CI 触发范围（见第三节）：所有触发的 job 均来自与内容无关的通用矩阵（`ci.yml`），而非因代码/CI 配置变更而需要针对性验证的 job。

## 三、`E2E Tests` workflow 未触发核实

```yaml
# .github/workflows/e2e-tests.yml:1-21
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
      - '.github/workflows/cd.yml'
      - 'docs/compatibility-matrix.md'
```

- `pull_request.paths` 过滤器**不包含 `skills/**` 或本 PR 涉及的 `docs/superpowers/**`**，因此 `E2E Tests` workflow **未被触发**——`gf pipeline status --branch feat/318-worktree-symlink-exclude-guard` 核对确认仅有 2 个 run（主 CI + Smoke Test 跨平台），无第三个 `E2E Tests` run。
- **这是预期且正确的行为**：本 PR 是对 `gf-workflow` skill 文档/流程的修复（补充 `info/exclude` 写入与预交付扫描守卫），不涉及生产代码或 E2E 测试覆盖的路径，不构成 CI 覆盖缺口。

## 四、PR 合并状态说明

`gf pr view 321` 返回 `state: "closed"`、`createdAt: "2026-09-04T07:08:40Z"`、`mergedAt: "2026-09-04T07:09:31Z"`——PR 在创建后约 51 秒即被记录为合并，早于其触发的全部 CI 收尾（末尾 job 于 07:18:45Z 才完成，即合并时点后约 9 分钟）。与既往系列报告（PR #311→#320，共七次）记录的「auto-merge 排队等待必需检查通过」模式一致：`gf pr view` 在合并动作完成后立即返回 `closed`/`mergedAt`，不代表流水线已全部收尾。**本报告持续轮询至全部 10 个 job 真正收尾**，可以确证——**合并后触发的全部 CI 检查最终均为 `success`，未发现门禁被绕过或合并后出现失败的证据**。

## 五、耗时分析

**耗时对比（与既往系列报告历史区间比对，PR #311–#320）**：

| Job | 本次耗时 | 既往历史区间参考 | 评估 |
|-----|---------:|:---:|:---:|
| MSRV | 57s | 53s（PR #320 一致） | 区间内 |
| Check | 1m28s | 1m21s（PR #320） | 区间内，略高但接近 |
| Lint | 2m21s | 2m21s（PR #320 一致） | 一致 |
| Smoke Test | 2m54s | 3m11s（PR #320） | 区间内，优于上次 |
| Test (ubuntu-latest) | 4m57s (297s) | 116s–337s / 283s（PR #320） | 区间内 |
| Test (macos-latest) | 6m28s (388s) | 116s–337s / 256s（PR #320）/ 384s（PR #317） | 高于早期区间与 PR #320，与 PR #317 峰值持平 |
| **Test (windows-latest)** | **9m59s (599s)** | 116s–337s（早期）/ 485s（PR #320）/ 616s（PR #317 峰值） | 高于早期区间与 PR #320，逼近 PR #317 记录的历史峰值（616s），但未超过 |

`Test (windows-latest)` 与 `Test (macos-latest)` 本次耗时均高于近几轮报告（PR #320）观测值，逼近但未超过 PR #317 报告记录的历史峰值（windows 616s/macos 384s）。本 PR 为 skill/文档变更，未触及任何测试代码或 CI runner 配置，与耗时波动无逻辑关联，延续 PR #317/#320 报告已记录的「runner 资源争用瞬时抖动」结论，非结构性回归。**建议**：作为观察项继续跟踪；若后续报告持续逼近或超过 616s 峰值，建议单独立项排查 windows/macos runner 资源争用根因。

## 六、dev / main 基线（采集时点：PR #321 全部 job 收尾后）

| 分支 | 周期 | Total runs | Success rate | Avg duration | 评级 |
|------|------|-----------:|--------------:|--------------:|------|
| `dev` | 7 天 | 100 | 95.0% | 150.47s | 🟢 Healthy |
| `main` | 30 天 | 100 | 100.0% | 159.59s | 🟢 Healthy |

基线数值与 PR #311–#320 系列报告完全一致，延续系列报告观察到的健康水位；PR #321（skill/文档变更）未导致基线抖动。

## 七、Flaky / 失败信号

**PR #321 自身流水线**：10 个 job 全部 `success`，无任何失败，无 flaky 信号。

`gf pipeline report --branch feat/318-worktree-symlink-exclude-guard --days 7`（在全部 job 收尾后采集）：

```json
{
  "totalRuns": 2,
  "successRate": 1.0,
  "avgDurationSecs": 358.0,
  "topFailures": []
}
```

**采集时机口径说明（沿用既往系列报告的已知发现）**：本次分析在 run 仍处于 `running` 状态时首次采集 `gf pipeline report`，观测到 `successRate: 0.0`（`totalRuns: 2`，均因未收尾被计入失败桶）——这与 PR #311→#317 系列报告记录的「`gf pipeline report` 在 run 处于 `running`/`queued` 时会将其计入失败桶」问题**完全一致地复现**。在持续轮询至全部 job 真正收尾后重新采集，`successRate` 才更新为 `1.0`，与实测（10/10 job success）一致。**根因仍未修复**，本次报告再次印证：依赖单次快照采集 `gf pipeline report` 存在误判风险，尤其在 PR 刚创建、run 仍在排队/执行阶段时采集会得出虚假的 0% 成功率。**维持既往建议**：应尽快针对 `gf` CLI 提交独立 Issue，改进 `pipeline report` 使其将 `running`/`queued` 状态与真实 `failure` 分开统计，而非在快照时机不当时呈现误导性的健康度数字。

## 八、结论与 Recommendations

1. 🟢 **无阻断性发现** — PR #321（Issue #318，worktree 共享符号链接排除守卫）触发的 2 个 workflow run 共 10 个 job **全部 `success`**，无失败、无跳过、无 flaky 信号。
2. 🟢 **无阻断性发现** — 核实 PR 变更范围为 `skills/gf-workflow/**` 与 `docs/superpowers/**` 下 4 个文件，未改动任何 Rust 代码、Cargo 清单或 `.github/workflows/*.yml`，与 PR 描述一致；`E2E Tests` workflow 因路径过滤器不含改动路径未被触发，属预期行为，不构成 CI 覆盖缺口。
3. 🟢 **合并门禁核实** — `gf pr view 321` 显示合并时点（07:09:31Z）早于全部 CI 收尾（07:18:45Z），符合既往系列报告记录的 auto-merge 排队模式；持续轮询至全部终态确认合并后触发的检查最终均通过，未发现门禁绕过证据。
4. 🟡 **Low（观察项，非本 PR 引起）** — `Test (windows-latest)` 本次耗时 9m59s（599s）、`Test (macos-latest)` 本次耗时 6m28s（388s），均高于近几轮报告（PR #320）观测值，逼近但未超过 PR #317 记录的历史峰值（windows 616s/macos 384s）。本 PR 未改动任何测试代码或 CI 配置，与耗时波动无逻辑关联，倾向于延续「runner 资源争用瞬时抖动」结论。建议继续在后续报告中观察该趋势，若持续逼近/超过历史峰值应单独排查。
5. 🟡 **Medium（沿用既往建议并再次复现，本次新增证据）** — `gf pipeline report` 在 run 处于 `running` 状态时**本次分析中直接复现**：首次采集（run 未收尾）返回 `successRate: 0.0`，全部收尾后重新采集才更新为 `1.0`。这是继 PR #311→#317 系列报告六次记录后的**第七次复现**（PR #320 因采集时机恰好在收尾后而未复现，但根因当时已明确指出未修复）。维持并加强既往建议：尽快针对 `gf` CLI 提交独立 Issue，改进 `pipeline report` 使其将 `running`/`queued` 与真实 `failure` 分开统计，避免依赖「凑巧在收尾后采集」才能得到准确结果——本次复现证明该问题仍未解决，且是首次采集即触发（而非依赖特定采集时机）。
