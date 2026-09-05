# Design: gf pipeline report 误将 running/queued run/job 计入失败桶

**Issue:** #324
**Date:** 2026-09-05
**Scope:** Bounded fix (existing flow)

## Context

`gf pipeline report` 在目标分支存在尚未收尾（`running`/`queued`）的 workflow run 时，
会把这些未收尾的 run/job 错误计入失败桶，导致成功率快照虚假偏低，且 `topFailures`
会包含实际已经 `success` 的 job。已连续在 PR #311、#312、#313、#315、#316、#317、
#320、#321、#323 共 9 次 `gf-pipeline-analyzer` 分析报告中复现，自最早记录起从未修复。

复现证据（PR #323，`docs/pipeline-analysis-report-2026-09-05-pr323.md` 第七节）：
run 33958444026（10 个 job 中 2 个仍 `in_progress`）尚未收尾时，两次
`gf pipeline report` 快照分别得到 `successRate: 0.0` / `0.5`，且第一次快照将
已经 `success` 收尾的 `MSRV` job 错误列入 `topFailures`。

## Root Cause

`crates/github/src/pipeline.rs` 的 `report()` 方法只向 `gh run list` 请求
`databaseId,conclusion,createdAt,updatedAt` 字段——**没有请求 `status` 字段**，
且判断"run 是否已收尾"仅凭 `conclusion.is_some()`。这与同文件的 `status()` 方法
（用 `gh_status_to_enum(status, conclusion)`，以 `status == "completed"` 作为收尾
判据）以及 `crates/gitlab/src/pipeline.rs` 的 `report()`（用
`PipelineStatusEnum::Running | Pending` 排除未收尾项）不一致。

当 running/queued 的 run/job 在 `conclusion` 字段上呈现非 `None` 的值（不论具体原因，
只要不是标准的 JSON `null`），就会被 `conclusion.is_some()` 误判为"已收尾"，进而：
- 被计入 `total_runs`/`success_rate` 的分母；
- 在 `attribute_top_failures()` 中被当作失败类 run，触发 job 级归因查询，如果查询时
  目标 job 也尚未收尾，同样会被误判为失败 job，标签（job 名）被计入 `topFailures`。

`attribute_top_failures()` 还有第二处同类问题：它对 job 级 `conclusion` 直接调用
`is_failure_conclusion`，同样没有先检查 job 的 `status` 是否为 `completed`。

## Fix

1. **`crates/core/src/pipeline.rs`**：给 `PipelineStatusEnum` 增加
   `pub fn is_terminal(&self) -> bool`（`!matches!(Running | Pending)`），
   作为跨平台统一的"是否已收尾"判据，避免各平台实现各自为政。

2. **`crates/github/src/pipeline.rs`**：
   - `ReportRun` 增加 `status: String` 字段；`report()` 的 `--json` 查询串加上 `status`；
   - `report()` 对每个 run 用 `gh_status_to_enum(&run.status, run.conclusion.as_deref())`
     计算状态枚举，用 `is_terminal()` 过滤出真正收尾的 run，作为 `total_runs`/
     成功率分子分母的来源（`aggregate_report_metrics` 只接收已过滤的 terminal runs）；
   - `attribute_top_failures()`：run 级别改用状态枚举判断 `matches!(status_enum, Failed)`，
     而非直接读裸 `conclusion` 字符串；job 级别同理，用 `job.status` 算出状态枚举，
     先过滤 `is_terminal()`，只在已收尾的 job 里查找失败项；若过滤后无匹配（run 已收尾但
     job 数据滞后），回退到 run 的通用 `conclusion` 字符串（沿用现有回退语义）。

3. **`crates/gitlab/src/pipeline.rs`**：把 `report()` 中内联的
   `!matches!(p.status, PipelineStatusEnum::Running | Pending)` 替换为
   `p.status.is_terminal()`，消除重复逻辑，行为不变。

4. **GitCode**：`report()` 当前直接返回不支持错误，无需改动。

## Testing

在 `crates/github/src/pipeline.rs` 补充回归测试（`MockCommandRunner` 构造 fixture）：
- run 整体仍 `in_progress`（`status: "in_progress"`, `conclusion` 非标准值或空），
  部分 job 已 `success` → 断言不计入 `total_runs`，不出现在 `topFailures`；
- run 已 `completed`/`conclusion: "failure"`，但归因查询到的某个 job 仍
  `status: "in_progress"` → 断言该 job 不被当作失败标签，回退到通用 conclusion 字符串。

`crates/core/src/pipeline.rs` 为新增的 `is_terminal()` 补充单元测试
（覆盖 Running/Pending → false，Success/Failed/Cancelled → true）。

## Impact

3 个源文件（core + github + gitlab），不改变公开 API 签名，无迁移。复杂度评分 simple，
main agent 批量实现 + 单次代码审查。
