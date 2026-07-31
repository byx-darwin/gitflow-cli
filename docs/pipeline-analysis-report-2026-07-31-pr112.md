# 流水线分析报告 — PR #112（Issue #97）

- **日期**：2026-07-31
- **关联**：PR #112 · 分支 `feat/97-v1.0-metadata-website-geo` → `main` · 工作流 `wf-2026-07-31-001` Phase 4
- **分析方式**：只读（`gh pr checks` + `gitflow-cli pipeline report/status`），未触发/重跑任何流水线

## 三维分析

### 1. 成功率趋势（main · 近 7 天）

| 指标 | 值 |
|------|----|
| 总运行数 | 31 |
| 成功率 | 83.9%（🟡 ≥80% 阈值） |
| 平均耗时 | 129.6s |
| 主要失败 | `failure`（泛化，无单一主导模式） |

### 2. PR #112 当次检查状态

| 检查 | 状态 | 备注 |
|------|------|------|
| Build / Check / msrv / Smoke Test (github/gitlab/gitcode) / E2E (GitHub) / Test (ubuntu) | ✅ pass | 核心门禁全绿 |
| **Website → Build**（新增 `website.yml`） | ✅ pass (21s) | Astro 构建成功 |
| **Website → Deploy** | ⏭️ skipping | PR 上按设计跳过（仅 push main 部署）✅ 符合预期 |
| build-rust (windows/macos/ubuntu) / Test (windows/macos) / Lint | ⏳ pending | 分析时仍在运行，非失败 |

### 3. 耗时分布

- 平均 ~130s；E2E (51s) / Test ubuntu (1m7s) 为主要耗时项；网站构建 21s 轻量。
- 无异常长尾。

## 结论

- **pipeline_ok = true**（当前 PR 零失败；新增 `website.yml` 行为正确：PR 构建、push 部署）。
- 部分矩阵检查（windows/macos/Lint）分析时 pending；本地 `clippy -D warnings -W pedantic` 与 `make test`(959/959) 已绿，预期 CI 一致。

## 改进建议（按优先级）

1. **合并前复核 pending 检查转绿**（尤其 Lint=clippy 与 windows/macos 矩阵），确保全矩阵通过再合并。
2. **首次 push main 后验证 Pages 部署**：`website.yml` 的 deploy 需仓库 Settings → Pages Source=GitHub Actions（一次性配置，dogfooding 阶段处理）。
3. **7 天成功率 83.9% 含历史噪声**：建议 1.0 后建立月度流水线巡检（结合 `upstream-patrol.yml`），将成功率稳定到 ≥95%。
