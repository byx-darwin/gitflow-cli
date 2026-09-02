# Pipeline Report: 排除非终态 Run 的统计口径修复（设计）

- Issue: #285
- 关联: #284（多角色评估 v2 P0），历史 8 份 pipeline-analysis-report（PR #268/269/272/273/274/276/279/281）均出现该口径问题。
- 类型: bounded bug fix（无新子系统，无接口变更）

## 问题

`gf pipeline report` 统计成功率时，`total_runs` 用窗口内全部 run 数（`runs.len()` / `recent.len()`），包含仍在运行（in-progress/running/pending/queued）的非终态 run；而 `success_count`/`failure_counts` 只统计有确定结论（终态）的 run。结果：分子只含终态 run，分母含全部 run，`success_rate = success/total` 被非终态 run 静默拉低，历史报告中反复需要人工复核抵消。

不是字面意义上把 in-progress 计入 `failure_counts`，而是分母污染导致成功率被系统性低估——对读者而言观感等价于"in-progress 被误判为失败"。

## 根因定位

- **GitHub** — `crates/github/src/pipeline.rs::aggregate_report_metrics`（约 190-235 行）；`total_runs` 计算于约 364 行，取 `runs.len()`，早于终态过滤。
- **GitLab** — `crates/gitlab/src/pipeline.rs::report`（335-416 行）；`total_runs = recent.len()`，未排除 `PipelineStatusEnum::Running`/`Pending`。
- **GitCode** — `crates/gitcode/src/pipeline.rs::report` 为未实现 stub，直接返回 `Err`，本次不涉及。

## 方案（已确认：方案 A）

只改分母口径，不新增字段，不改 `PipelineStatusEnum` 定义，不涉及 GitCode：

1. **GitHub**：`total_runs` 改为仅统计终态 run（`conclusion.is_some()` 的 run 数），排除 `conclusion: None` 的 in-progress run。
2. **GitLab**：`total_runs` 改为排除 `PipelineStatusEnum::Running` / `PipelineStatusEnum::Pending`（非终态）后的计数。
3. `success_rate` 计算逻辑不变（`success_count / total_runs`），因分母已收窄为终态 run，数值随之修正。

## 测试计划（TDD）

- GitHub：构造 in-progress + success + failure 混合的 run 列表，RED 断言当前 `total_runs` 错误包含 in-progress → GREEN 修复后断言 `total_runs` 排除 in-progress、`success_rate` 只按终态计算。
- GitLab：构造 running + success + failed pipeline 混合列表，同样 RED → GREEN。
- 覆盖三态：in-progress / success / failure，按 Issue #285 验收标准要求。

## 验证

修复后重新生成一份 pipeline-analysis-report，确认统计口径已修复（Issue #285 验收标准第 4 条）。

## 范围边界（明确排除）

- 不新增 `in_progress_runs` 等新字段。
- 不改 `PipelineStatusEnum` 定义或状态映射函数（`gh_status_to_enum`、`parse_pipeline_status`）。
- 不涉及 GitCode（stub 未实现，无统计逻辑可修）。
