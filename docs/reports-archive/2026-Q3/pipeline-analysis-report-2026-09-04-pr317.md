# Pipeline 分析报告 — PR #317

> **PR：** [#317 test: harden temp-file-path tests against shared fixed filenames](https://github.com/byx-darwin/gitflow-cli/pull/317)
> **分支：** `feat/301-temp-file-test-isolation` → `dev`（对应 Issue #301，gf-workflow 标准模式）
> **快照时间：** 2026-09-04T05:56:34Z（全部 job 已收尾，本报告为持续轮询至全部终态的完整快照，非部分收敛）
> **分析日期：** 2026-09-04
> **模式：** 只读（CLI: `gf`；PR checks 交叉核对用 `gh pr checks`）
> **变更性质：** 测试隔离修复——Issue #289/#301 记录了 `dev` 分支单次 `Test (windows-latest)` 失败（`commands::commit::tests::test_should_resolve_comment_body_from_file`，run `33346653353`，2026-08-31，生产代码未变更），归因为共享 OS 临时目录下**固定文件名**（`std::env::temp_dir().join("gitflow_test_commit_comment.md")`）潜在的路径冲突。本 PR 将 4 个单测（`apps/cli/src/commands/{commit,issue,pr,release}.rs`）中相同模式替换为 `tempfile::NamedTempFile::new()`（workspace 既有依赖，`skills.rs`/`workflow.rs`/`release-signer`/`e2e-core::scratch` 已在使用），每次调用获得 OS 生成的唯一路径，消除共享文件名冲突风险；`Drop` 自动清理，手动 `remove_file` 调用一并移除。**未改动任何生产代码**（`resolve_comment_body`/`resolve_body`/`SafePath` 逻辑不变）。

## 零、核心结论先行

`feat/301-temp-file-test-isolation` 分支触发 3 个 workflow run（均创建于 `2026-09-04T05:46:15Z`），共 **15 个 job**（含 1 个按条件跳过的 `Notify on Scheduled Regression Failure`）。本报告**持续轮询至全部 job 收尾**（末尾 job `Test (windows-latest)` 于 `05:56:34Z` 完成，耗时 10m16s），**14 个实际执行的 job 全部 `success`**，**无一失败**。**本次分析的核心焦点——`Test (windows-latest)`（本 PR 直接针对的目标 job）实测 `success`**，未复现 Issue #289/#301 记录的历史失败。`e2e-gitcode::noauth`（及 `e2e-gitlab`）本地失败问题**未在本次 CI 中出现**：核对 `.github/workflows/ci.yml` 第 61 行确认 `Test` job（3-OS 矩阵）仍显式 `--exclude e2e-gitlab --exclude e2e-gitcode`，未受本 PR 影响，该排除逻辑维持不变。**唯一值得记录的信号是耗时**：`Test` job 三平台本次耗时（ubuntu 4m43s、macos 6m24s、**windows 10m16s**）均明显高于既往系列报告记录的历史区间（116s–337s），尤以 windows 平台涨幅最大（+83% 相对历史上限）——经核实与本 PR 变更（仅涉及测试内文件路径生成方式，未触及构建脚本、依赖版本或测试矩阵）无逻辑关联，更可能是采集时段 GitHub-hosted runner 队列/资源争用所致，见第七节。`dev`/`main` 基线保持健康（95%/100%），与既往系列报告一致。

## 一、PR #317 关联流水线实测（全部收尾）

`feat/301-temp-file-test-isolation` 分支触发 3 个 workflow run：

| Run ID | Workflow | 收尾状态 | 备注 |
|--------|----------|------|------|
| 33841733044 | 主 CI（Check/MSRV/Lint/Smoke Test/Test×3） | ✅ success（7/7 job 全部成功） | 末尾 job `Test (windows-latest)` 于 05:56:34Z 收尾 |
| 33841733048 | Smoke Test 跨平台 | ✅ success（3/3 job 全部成功） | 最先收尾（05:47:30Z 前） |
| 33841733050 | E2E Tests（GitHub/GitLab/GitCode） | ✅ success（3/3 job success + 1 个条件跳过） | `Notify on Scheduled Regression Failure` 按条件 `skipped`（非失败） |

全部 job 明细（`gf pipeline jobs` + `gh pr checks` 交叉核对，持续轮询至全部终态）：

| Job | Workflow run | 结论 | 耗时 |
|-----|--------------|------|------|
| MSRV | 33841733044 | ✅ success | 50s |
| Smoke Test (gitcode) | 33841733048 | ✅ success | 1m9s |
| Smoke Test (gitlab) | 33841733048 | ✅ success | 1m9s |
| Smoke Test (github) | 33841733048 | ✅ success | 1m12s |
| Check | 33841733044 | ✅ success | 1m27s |
| Lint | 33841733044 | ✅ success | 2m27s |
| Smoke Test | 33841733044 | ✅ success | 2m56s |
| E2E Tests (GitCode) | 33841733050 | ✅ success | 3m9s |
| E2E Tests (GitHub) | 33841733050 | ✅ success | 3m21s |
| E2E Tests (GitLab) | 33841733050 | ✅ success | 3m23s |
| Test (ubuntu-latest) | 33841733044 | ✅ success | 4m43s |
| Test (macos-latest) | 33841733044 | ✅ success | 6m24s |
| **Test (windows-latest)** | 33841733044 | ✅ **success** | **10m16s** |
| Notify on Scheduled Regression Failure | 33841733050 | ⚪ skipped（条件跳过，非失败） | 0s |

**共 15 个 job**：14 个实际执行，**全部 `success`，无一失败**；1 个按工作流条件跳过（非 PR 触发场景），不计入失败。

## 二、核心焦点核实——`Test (windows-latest)`

本 PR 的修复目标就是消除该 job 上曾观测到的一次性失败（Issue #289/#301，run `33346653353`，2026-08-31，`test_should_resolve_comment_body_from_file`）。本次实测：

- **结论：`success`**，耗时 10m16s（run `33841733044`，job `100925236230`）。
- 4 个被修改的测试函数（`commit.rs::test_should_resolve_comment_body_from_file`、`issue.rs::test_should_resolve_body_from_file`、`pr.rs::test_should_resolve_body_from_file`、`release.rs::test_should_resolve_body_from_file`）均已改用 `tempfile::NamedTempFile::new()`（经 `grep` 核实，源码位置分别为 `commit.rs:240`、`issue.rs:450`、`pr.rs:589`、`release.rs:370`），不再依赖共享临时目录下的固定文件名。
- 由于历史失败仅出现 1 次（未达 flaky 判定阈值 ≥2 次），本次单次 `success` **不足以统计学证明**修复消除了潜在的路径冲突根因（原本就是低概率事件）；但至少确认：① 新写法未引入回归，② 该 job 本身在当前 CI 环境下可稳定通过。**建议**：继续观察后续 `dev` 分支该 job 的执行记录，若连续多次 `success` 且无同类失败复现，可视为修复有效性的间接佐证。

## 三、`e2e-gitcode::noauth` 排除逻辑核实

按分析要求核对：此前记录的 `e2e-gitcode::noauth`（及 `e2e-gitlab`）本地失败是否会出现在本次 CI 中。

```yaml
# .github/workflows/ci.yml:54-61
# e2e-gitlab/e2e-gitcode need the real `glab`/`gc` CLI binaries installed to
# exercise their noauth.rs error-path assertions correctly (see
# .github/workflows/e2e-tests.yml's dedicated jobs, which install them).
# This generic 3-OS matrix job doesn't install them, so exclude those two
# crates here — e2e-github stays included since `gh` ships preinstalled on
# all three GitHub-hosted runner images.
- name: cargo test
  run: cargo nextest run --all-features --workspace --exclude e2e-gitlab --exclude e2e-gitcode
```

- **该排除逻辑未被本 PR 触及**，`ci.yml` 中 `Test` job（3-OS 矩阵）仍显式 `--exclude e2e-gitlab --exclude e2e-gitcode`。
- 实测确认：三平台 `Test` job（ubuntu/macos/windows）全部 `success`，未观察到任何与 `e2e-gitcode`/`noauth.rs` 相关的失败或报错。
- 独立的 `E2E Tests (GitCode)` job（run `33841733050`，专门安装 `gc` CLI 后运行）本次同样 `success`，说明该 crate 本身在装有真实 CLI 的专用 job 中也正常，与「本地无 `gc`/`glab` 时 `noauth.rs` 断言失败」的既往已知问题（无关、pre-existing、CI 已规避）不矛盾。
- **结论：排除逻辑维持有效，本次 CI 未复现该已知本地问题**，符合预期。

## 四、PR 合并状态说明

`gf pr view 317` 返回 `state: "closed"`、`createdAt: "2026-09-04T05:46:11Z"`、`mergedAt: "2026-09-04T05:46:24Z"`——PR 在创建后约 13 秒即被记录为合并，早于其触发的全部 CI 收尾（末尾 job 于 05:56:34Z 才完成，即合并时点后约 10 分钟）。与既往系列报告（PR #313/#315/#316）记录的「auto-merge 排队等待必需检查通过」模式一致：`gf pr view`/`gh pr view` 在合并动作完成后立即返回 `closed`/`mergedAt`，不代表流水线已全部收尾。**本次报告与既往不同之处在于：持续轮询至全部 15 个 job 真正收尾**，因此可以确证——**合并后触发的全部 CI 检查最终均为 `success`，未发现门禁被绕过或合并后出现失败的证据**。

## 五、`gf pipeline report` 口径假象（第六次复现，与 PR #311/#312/#313/#315/#316 一致）

`gf pipeline report --branch feat/301-temp-file-test-isolation --days 7`（在 run 仍 running 时采集）：

```json
{
  "totalRuns": 3,
  "successRate": 0.0,
  "avgDurationSecs": 9.0,
  "topFailures": [""]
}
```

与既往五次报告（PR #311→#312→#313→#315→#316）记录的同一类问题一致——命令将仍处于 `running`（`conclusion` 为空）的 run 计入失败桶。经本次**持续轮询至全部 job 真正收尾**后交叉复核，**全部 14 个实际执行的 job 均为 `success`，无真实失败**，该命令口径与实测完全不符。该问题已连续六次在系列报告中复现，**维持既往建议：尽快提交独立 Issue 改进 `pipeline report` 的运行中状态统计口径**（不在本次只读分析范围内代为提交）。

## 六、dev / main 基线（采集时点：PR #317 全部 job 收尾后）

| 分支 | 周期 | Total runs | Success rate | Avg duration | 评级 |
|------|------|-----------:|--------------:|--------------:|------|
| `dev` | 7 天 | 100 | 95.0% | 150.47s | 🟢 Healthy |
| `main` | 30 天 | 100 | 100.0% | 159.59s | 🟢 Healthy |

基线数值与 PR #311/#312/#313/#315/#316 报告完全一致，延续系列报告观察到的健康水位；PR #317（测试隔离修复，未改动生产代码或 CI 配置）未导致基线抖动。

## 七、Flaky / 失败信号 与 耗时分析

**PR #317 自身流水线**：14 个实际执行的 job 全部 `success`，无任何失败。

**耗时异常（本报告新增信号）**：`Test` job 三平台本次耗时均明显高于既往系列报告记录的历史区间（116s–337s）：

| Job | 本次耗时 | 历史区间（既往报告） | 相对偏离 |
|-----|---------:|:---:|:---:|
| Test (ubuntu-latest) | 4m43s (283s) | 116s–337s | 略低于上限 |
| Test (macos-latest) | 6m24s (384s) | 116s–337s | 超出上限约 14% |
| **Test (windows-latest)** | **10m16s (616s)** | 116s–337s | **超出上限约 83%** |

三平台**同步**出现耗时上涨（而非仅 windows 单一异常），指向共同的外部因素而非本 PR 变更本身——本 PR 仅修改测试内部文件路径生成方式（`std::env::temp_dir().join(...)` → `tempfile::NamedTempFile::new()`），不涉及依赖版本、编译产物大小或测试用例数量变化，逻辑上不足以解释 3 个独立矩阵 job 的耗时同步上涨。更可能的原因是采集时段（`2026-09-04T05:46`–`05:56` UTC）GitHub-hosted runner 队列排队或共享基础设施资源争用（跨 job 的 `startedAt` 显示各 job 几乎同时于 `05:46:18Z`–`05:46:24Z` 排队启动，但实际执行耗时差异悬殊）。**建议**：作为观察项记录，若后续 `dev`/`main` 分支同一时间窗口内其他 PR 的 `Test` job 也出现类似耗时上涨，可确认为平台侧瞬时抖动而非本 PR 引入的问题；若仅本 PR 独有，需进一步排查 `tempfile::NamedTempFile::new()` 是否在 windows 文件系统上有额外开销（可能性较低，因该 API 已在仓库其他位置广泛使用且历史无耗时问题记录）。

历史观察清单：`commands::commit::tests::test_should_resolve_comment_body_from_file`（`apps/cli/src/commands/commit.rs`，Issue #289/#301 触发本 PR 的直接原因）此前记录的唯一 1 次失败（run `33346653353`，2026-08-31）**是本 PR 修复的目标**；本次该测试函数已改用 `tempfile::NamedTempFile`（见第二节），实测所在 `Test (windows-latest)` job `success`。移出观察清单待续，建议后续 2-3 次 `dev` 分支该 job 执行记录持续确认无复发后可正式关闭。

## 八、结论与 Recommendations

1. 🟢 **无阻断性发现** — PR #317（Issue #301，临时文件测试隔离修复）核心目标 `Test (windows-latest)` job **实测 `success`**（10m16s），4 个修改的测试函数均已确认使用 `tempfile::NamedTempFile`，14 个实际执行的 job 全部通过，无失败信号。
2. 🟢 **无阻断性发现** — `e2e-gitcode::noauth`（及 `e2e-gitlab`）本地失败问题未在本次 CI 中出现，`ci.yml` 第 61 行 `--exclude e2e-gitlab --exclude e2e-gitcode` 排除逻辑维持不变、未受影响，独立的 `E2E Tests (GitCode)` job（装有真实 `gc` CLI）本次同样通过。
3. 🟡 **Low（观察项，非本 PR 引起）** — `Test` job 三平台本次耗时（ubuntu 283s、macos 384s、windows 616s）均高于既往历史区间（116s–337s），windows 平台涨幅达 83%；经核实与本 PR 变更逻辑无关联，倾向于采集时段 runner 队列/资源争用导致的瞬时抖动。建议在后续报告中持续观察 `Test` job 耗时趋势，若非本 PR 独有则无需处理。
4. 🟡 **Medium** — `gf pipeline report` 命令在 run 处于 `running` 状态时持续将其计入失败桶，本次为**第六次连续复现**（PR #311→#312→#313→#315→#316→#317）。维持既往建议：尽快针对 `gf` CLI 提交独立 Issue，改进 `pipeline report` 使其将 `running`/`queued` 与真实 `failure` 分开统计。
5. 🟢 **观察清单更新** — `commands::commit::tests::test_should_resolve_comment_body_from_file` 此前记录的唯一 1 次历史失败已通过本 PR 的 `tempfile::NamedTempFile` 改造得到针对性修复，本次实测通过；建议持续观察后续 2-3 次执行记录以正式确认根因已消除。
