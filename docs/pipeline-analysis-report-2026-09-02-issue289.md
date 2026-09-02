# Pipeline Analysis Report — Issue #289

> **Issue:** [#289 chore(ci): CI 成功率 Watch 档根因归因 + gf-pipeline-analyzer 升级机制](https://github.com/byx-darwin/gitflow-cli/issues/289)
> **分支：** `dev`（对应本 issue 归因的目标分支）
> **分析日期：** 2026-09-02
> **模式：** 只读（CLI: `gf`，版本 `1.9.0`，本地构建自 `feat/289-pipeline-analyzer-escalation` 分支，已含本 issue 的 job 级归因改动，commit `7ef86b2`）
> **依赖：** #285（in-progress run 误判分母 bug）已 CLOSED/merged，修复已在 `dev` 上线；本次 30 天窗口内的 100 个 run 均已终态（无 `conclusion: null` 样本），统计口径可信。

## 零、核心结论先行

对 `dev` 分支近 30 天（100 个 run 窗口上限，覆盖 2026-08-28T07:24 至 2026-09-02T07:36）做扩大抽样归因，结论如下：

1. **成功率已回升至 🟢 Healthy 区间（95.0%），不再是 Issue 描述时的 🟡 Watch 档（~94%）。** 历史上确实存在 issue 中提到的连续 Watch 档现象（PR #272/#273/#274/#276/#279/#281 六份报告，93%–94% 区间），但该 streak 已在 PR #297 采集时（2026-09-02 更早时段）自然回升到 95.0% 并越过健康线，此后 PR #298、PR #300 及本次采集均稳定在 95.0%，**当前不存在需要触发新升级机制的活跃 Watch streak**（连续同水位定义详见 `skills/gf-pipeline-analyzer/SKILL.md` 新增的 Escalation Rule；本报告可作为该规则生效后的首份基线）。
2. **`topFailures` 已从通用 `"failure"` 标签升级为具体 job 名称**：`["Lint", "Test (windows-latest)"]`（按失败次数降序，同为 1 次时按字母序）。这是本次实现的 job 级归因能力（`crates/github/src/pipeline.rs::attribute_top_failures`）产出的真实数据，验证了该功能按预期工作。
3. 100 个 run 中仅 2 个失败（2%），且两次失败分别归因到两个不同 job、各仅 1 次，**均为一次性样本，不满足“同一 job 失败 ≥3 次”的持续性阈值，也不满足“≥2 次间歇性失败”的 flaky 判定阈值**。深入检查两次失败的完整日志后，未能确认可复现、可直接修复的代码级根因（详见「二、失败归因」）。**本报告如实标注为归因收窄但未完全定论，而非编造一个确定性根因。**

## 一、`gf pipeline report` 实测输出

```bash
$ gf pipeline report --branch dev --days 30
```

```json
{
  "totalRuns": 100,
  "successRate": 0.95,
  "avgDurationSecs": 149.2,
  "topFailures": ["Lint", "Test (windows-latest)"]
}
```

```bash
$ gf pipeline report --branch dev --days 7
```

```json
{
  "totalRuns": 100,
  "successRate": 0.95,
  "avgDurationSecs": 149.2,
  "topFailures": ["Lint", "Test (windows-latest)"]
}
```

`--days 7` 与 `--days 30` 输出完全一致：`gh run list --limit 100` 返回的 100 个 run 本身已全部落在最近约 5 天内（2026-08-28T07:24 ～ 2026-09-02T07:36），未触及 30 天窗口的边界，因此两个 `--days` 参数在当前提交速度下等价，不构成异常。

**#285 修复验证**：100 个 run 中 `conclusion` 字段全部非空（无 `in_progress`/`null` 样本落入本窗口），`total_runs` 分母未被未终态 run 污染，`successRate = 95/100 = 0.95` 计算口径可信，确认 #285 的修复在当前 `dev` 上线版本下有效。

## 二、失败归因

### 2.1 失败样本清单

对 100 个 run 中的 2 个失败 run 分别调用 `gh run view --json jobs`（即 job 级归因内部调用的同一路径）展开：

| Run ID | 触发时间 | run 级 conclusion | 失败 job | job 级 conclusion | 报告链接 |
|--------|----------|---------------------|----------|---------------------|----------|
| [33346653353](https://github.com/byx-darwin/gitflow-cli/actions/runs/33346653353) | 2026-08-31T01:08:14Z | failure | `Test (windows-latest)` | failure | 其余 6 个 job（Lint/Check/MSRV/Smoke Test/Test ubuntu/Test macos）均 success |
| [33151914908](https://github.com/byx-darwin/gitflow-cli/actions/runs/33151914908) | 2026-08-28T07:33:43Z | failure | `Lint` | failure | 其余 5 个 job（Check/Smoke Test/Test×3 平台）均 success |

`gf pipeline report` 输出的 `topFailures: ["Lint", "Test (windows-latest)"]` 与上表逐一对应，验证了归因逻辑（按失败次数降序，同为 1 次按字母序）正确落地。

### 2.2 逐 job 深入排查

**`Test (windows-latest)`（run 33346653353，2026-08-31）**

```
FAIL [   0.026s] ( 116/1384) gitflow-cli::bin/gf commands::commit::tests::test_should_resolve_comment_body_from_file
thread '...' panicked at apps\cli\src\commands\commit.rs:245:9:
assertion failed: result.is_ok()
```

该测试写入 `std::env::temp_dir().join("gitflow_test_commit_comment.md")`，调用 `resolve_comment_body(None, Some(path))` 后断言 `Ok`。核查结果：

- `resolve_comment_body`（`apps/cli/src/commands/commit.rs:176`）内部使用 `SafePath::new_allow_absolute`；`SafePath::validate`（`crates/core/src/lib.rs:290` 起）已对 Windows 盘符前缀（`Component::Prefix`）做了排除，`C:\...` 形式的绝对路径不会被误判为 ADS（`:`）注入，代码逻辑本身未见明显 bug。
- `apps/cli/src/commands/commit.rs` 自 2026-08-28（早于本次失败）起除版本号提交外无实质变更（`git log --since 2026-08-28 -- apps/cli/src/commands/commit.rs` 仅命中 release 提交），即失败发生前后该测试代码完全一致。
- 同一 job（`Test (windows-latest)`）在窗口内其余全部成功案例中均通过（例如 PR #300 报告记录的两次运行分别为 185s、172s 均 success），本地无 Windows 环境可复现。

**结论**：未能在不接触 Windows 运行时的前提下确认确定性代码根因；现有证据（单次出现、代码未变更、其余同 job 运行均通过）更倾向于一次性运行时环境因素（如临时目录被其他进程/杀毒软件短暂占用导致 `read_to_string` 失败），但不足以排除测试对共享临时文件路径缺乏隔离这一设计弱点。**如实标注为归因收窄但未完全定论**，不编造确定性根因。

**`Lint`（run 33151914908，2026-08-28）**

```
error: item in documentation is missing backticks
  --> crates/github/src/auth.rs:50:15
50 |     /// Note: AuthProvider doesn't use repo, so session.repo is ignored.
   = note: `-D clippy::doc-markdown` implied by `-D warnings`
```

核查结果：这是 `clippy::doc_markdown` 对文档注释中裸标识符 `AuthProvider` 的常规拦截，非环境性 flaky。检查当前 `dev` 分支该文件同一行，标识符已加反引号（`` `AuthProvider` ``），且本次改动前已运行 `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic` 全绿（见「四、验证结果」），确认该问题已在 2026-08-28 之后的某次提交中修复，是**历史遗留、已解决**的问题，非当前活跃根因。

### 2.3 归因结论

- `Test (windows-latest)`：单次样本，代码未变更、其余运行均通过，倾向一次性环境因素，**未完全定论**。
- `Lint`：已确认是历史 doc-markdown 违规，**已修复，非当前活跃问题**。
- 两者均不满足「持续性失败」（≥3 次同 job）或「flaky」（≥2 次间歇性）的判定阈值（见 SKILL.md Recurring Pattern Detection / Common Mistakes）。

## 三、dev / main 基线对比（30 天）

| 分支 | Total runs | Success rate | Avg duration | 评级 |
|------|-----------:|--------------:|--------------:|------|
| `dev`（30 天） | 100 | 95.0% | 149.2s | 🟢 Healthy（越过 95% 健康线，较 issue 描述时的 ~94% Watch 档已回升，见「零、核心结论先行」） |
| `main`（30 天） | 100 | 100.0% | 159.59s | 🟢 Healthy（`topFailures: []`，无失败样本） |

## 四、Watch 档 streak 历史回顾（issue 背景验证）

| 报告 | Success rate | 评级 |
|------|--------------:|------|
| PR #272/#273/#274 | 93%–94% | 🟡 Watch |
| PR #276 | 93.0% | 🟡 Watch |
| PR #279 | 94.0% | 🟡 Watch |
| PR #281 | 94.0% | 🟡 Watch |
| PR #297 | 95.0% | 🟢 Healthy（越线） |
| PR #298 | 95.0% | 🟢 Healthy |
| PR #300 | 95.0% | 🟢 Healthy |
| **本报告（issue #289）** | **95.0%** | **🟢 Healthy** |

Issue 描述的 "连续 5+ 份报告反复标注同一 Watch 水位却未见根因归因" 属实（PR #272/#273/#274/#276/#279/#281 共 6 份，93%–94% 区间）。该 streak 已在自然演进中于 PR #297 结束（回升至 95.0%），此后连续 4 份报告（含本报告）稳定在 🟢 Healthy。**这意味着 Escalation Rule 目前不会对 `dev` 分支触发升级提示**（当前 streak 是 4 份连续 Healthy，非未处理的 Watch/Alert 档）；升级机制的价值在于覆盖未来若再次出现类似 93%–94% 长期停滞而未处理的情况。

## 五、耗时分析

`dev` 30 天窗口 `avgDurationSecs: 149.2s`，与历次报告基本持平，无回归信号。逐 run 耗时明细本次未重复采集（PR #300 报告已针对同一批次数据给出逐 job 耗时表，`Test (windows-latest)` 稳定为全流水线最慢 job，172–185s 区间，属历史正常范围），本报告聚焦失败归因，不重复展开。

## 六、Flaky 信号

窗口内两次失败分别对应不同 job（`Lint`、`Test (windows-latest)`）、不同日期（2026-08-28、2026-08-31），中间及此后大量同 job 运行（含 PR #297/#298/#300 报告记录的多次 `Test (windows-latest)` 成功样本）均未复现。**不满足 SKILL.md 定义的 flaky 判定阈值（≥2 次间歇性失败）**，暂不标记为 flaky test；建议后续报告继续观察 `Test (windows-latest)` 是否再次出现同类失败，若累计 ≥2 次则应升级为 flaky 并纳入 Escalation Rule 的持续性判定。

## 七、结论

- `dev` 分支 30 天成功率 95.0%，`main` 分支 100.0%，均 🟢 Healthy；issue 描述的 Watch streak（PR #272–#281，93%–94%）已在 PR #297 后自然结束，当前连续 4 份报告稳定 Healthy，Escalation Rule 暂不触发。
- `topFailures` 已从通用 `"failure"` 标签升级为具体 job 名称（`Lint`、`Test (windows-latest)`），job 级归因功能（本 issue 新增）验证通过。
- 深入排查两次失败：`Lint` 失败为历史遗留、已修复的 doc-markdown 违规；`Test (windows-latest)` 失败为单次样本，代码未变更、其余运行均通过，归因收窄至该测试对共享临时文件路径的潜在隔离弱点，但**未能确认确定性根因**，如实标注为待观察项而非已解决问题。
- 已开具跟进 Issue [#301](https://github.com/byx-darwin/gitflow-cli/issues/301) 记录 `test_should_resolve_comment_body_from_file` 的具体失败证据与观察建议，避免归因结果无落地。

## 八、Recommendations

1. 🟢 **Low** — `dev`/`main` 均处于 Healthy 区间，无阻塞式发现，不影响当前交付。
2. 🟡 **Medium** — 跟进 Issue [#301](https://github.com/byx-darwin/gitflow-cli/issues/301)：观察 `Test (windows-latest)` 上 `commands::commit::tests::test_should_resolve_comment_body_from_file` 是否再次失败；已提议为该测试及 `issue.rs`/`pr.rs`/`release.rs` 中同模式的临时文件测试使用唯一路径替代固定文件名，避免共享路径风险，并按需为该测试添加短重试或诊断日志。
3. 🟢 **Low** — Escalation Rule（本 issue 新增）已落地至 `skills/gf-pipeline-analyzer/SKILL.md`，后续报告应基于本报告记录的 streak 历史（见「四」）延续计数，而非从零开始。
4. 🟢 **Low** — job 级 `topFailures` 归因（本 issue 新增，`crates/github/src/pipeline.rs`）目前仅覆盖 GitHub；GitLab/GitCode provider 的 `top_failures` 仍为占位/空实现，明确列为本次改动范围外（out of scope），如需覆盖需单独立项。
