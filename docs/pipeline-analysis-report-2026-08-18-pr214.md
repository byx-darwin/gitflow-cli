# Pipeline 分析报告 — wf-2026-08-18-007（PR #214）

> **工作流：** `wf-2026-08-18-007`（standard，Phase 4 Step 1）
> **PR：** [#214 fix: 主动上报bug 功能 P1/P2 遗留项](https://github.com/byx-darwin/gitflow-cli/pull/214)
> **分析日期：** 2026-08-18

## 一、PR #214 CI 状态

**分支 `feat/213-autoreport-bug-p1p2`：4/4 runs 全部 success ✅**

| Run | Workflow | 结论 |
|-----|----------|------|
| 32129562949 | Check/Lint/Smoke/Test | ✅ success |
| 32129563050 | Test matrix | ✅ success |
| 32129563110 | build | ✅ success |
| 32129563248 | build（含依赖审查） | ✅ success |

本 PR 变更面（Rust doctor + hook + skill 文档）全部通过 CI，无回归。

## 二、dev 分支基线（7 天）

| 指标 | 值 | 说明 |
|------|-----|------|
| successRate | 80%（80 runs） | 与前两轮观察一致（80-90% 区间）；h2 修复后无新增失败源 |
| 备注 | — | 本轮无 CI 失败，修复稳定性持续 |

## 三、结论

- `pipeline_ok = true`：PR #214 交付面全部通过 CI。
- 无新失败模式；品牌/skill 文档变更不触发 CI 风险。
