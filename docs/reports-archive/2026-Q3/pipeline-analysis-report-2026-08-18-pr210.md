# Pipeline 分析报告 — wf-2026-08-18-005（PR #210）

> **工作流：** `wf-2026-08-18-005`（standard，Phase 4 Step 1）
> **PR：** [#210 docs(eval): 多角色评估报告 — 主动上报 bug 功能是否 OK](https://github.com/byx-darwin/gitflow-cli/pull/210)
> **分析日期：** 2026-08-18
> **分析模式：** 只读（未触发/重跑/取消任何流水线）

---

## 一、PR #210 CI 状态

**变更面：** docs-only（6 个 Markdown 文档，无 `.rs`/`.toml`/代码变更）

推送到 `feat/209-autoreport-bug-multi-role-eval` 触发 3 个 workflow 运行（均 09:39:29Z）：

| Run ID | Workflow | 结论 | 失败 Job |
|--------|----------|------|----------|
| 32122744247 | 主 CI（Check/Lint/Smoke/Test×3） | ✅ success | — |
| 32122744296 | Lint 类 workflow | ❌ failure | `Lint` |
| 32122744237 | build workflow | ❌ failure | `build-rust (ubuntu-latest)` |

**失败 Job 详情：**

| Run | 失败 Job | 失败步骤 | 根因 |
|-----|----------|----------|------|
| 32122744296 | `Lint` | cargo deny 检查 | `error[vulnerability]: h2 unbounded empty DATA frames` |
| 32122744237 | `build-rust (ubuntu-latest)` | **Check dependency licenses and advisories** | 同上（h2 v0.4.15） |

> 两处失败为**同一根因**：`cargo-deny` 检出 h2 漏洞。其余全部 job（Check / Smoke Test / Test ubuntu+windows+macos / build-rust macos+windows / msrv / build-binaries / release）均成功，**所有 233+ 单元测试通过**。

## 二、失败归因：预存供应链问题，与 PR 无关

- **证据 1（变更面）**：PR #210 为 docs-only，未触碰 `Cargo.toml`/`Cargo.lock`/依赖 → 不可能引入依赖漏洞。
- **证据 2（锁文件）**：`Cargo.lock` 中 `h2 = 0.4.15`，该版本存在 RUSTSEC advisory「unbounded empty DATA frames」（DoS）。
- **证据 3（dev 分支同现）**：dev 分支 08:22 的 5 个运行中 2 个同样失败（同为该 workflow 集），证明失败**预存于主干**，非本 PR 引入。

**判定：** `pipeline_ok = true`（就本工作流交付而言）。PR 的代码/测试面全部通过；CI 红色来自预存 h2 advisory，应在独立 Issue 跟踪修复，不阻塞 docs 交付。

## 三、三维分析（dev 分支，7 天）

数据源：`gf pipeline report --branch dev --days 7`

| 维度 | 指标 | 值 | 评估 |
|------|------|-----|------|
| 成功率趋势 | totalRuns / successRate | 79 runs / **89.9%** | 🟡 良好但接近 90% 阈值，主要拖累来自 h2 advisory 类失败 |
| 失败模式 | topFailures | `failure`（无细分标签） | 🟠 失败缺少结构化分类；当前已知主要类 = dependency advisory |
| 耗时分布 | avgDurationSecs | 190.4s | 🟢 可接受（~3.2min） |

## 四、改进建议（优先级排序）

| 优先级 | 建议 | 归属 |
|--------|------|------|
| **P0** | 升级/规避 `h2 v0.4.15` 漏洞（RUSTSEC：unbounded empty DATA frames）。选项：升级 h2 至修复版本，或在 `deny.toml` 暂时 ignore（需用户确认，禁止未经确认改 deny 策略） | 依赖治理 / `Cargo.lock` |
| P1 | 为 pipeline failure 增加结构化分类（区分 dependency-advisory / flaky / infra），降低噪音 | CI 配置 |
| P2 | 失败 Job 命名对齐（`Lint` vs `build-rust` 内嵌 deny 步骤），便于快速定位 | CI 配置 |

> ⚠️ 依 CLAUDE.md：`deny.toml` 策略变更需用户明确授权，本报告**只记录建议，不实施**。

## 五、结论

- PR #210 交付面（文档 + 测试）**通过**，`tests_passed = true`。
- CI 两处红色为**预存 h2 advisory**，独立跟踪修复即可，不阻塞本评估交付。
- `pipeline_ok = true`。
