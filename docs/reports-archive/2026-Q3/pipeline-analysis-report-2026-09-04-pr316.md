# Pipeline 分析报告 — PR #316

> **PR：** [#316 chore(supply-chain): enforce cargo-vet gate and generate release SBOM](https://github.com/byx-darwin/gitflow-cli/pull/316)
> **分支：** `feat/296-cargo-vet-sbom` → `dev`（对应 Issue #296，gf-workflow 快速模式）
> **快照时间：** 2026-09-04T03:09:31Z（首次采集 03:06Z 附近；本报告为多次时点采样后的收敛快照，非持续轮询至全部收尾）
> **分析日期：** 2026-09-04
> **模式：** 只读（CLI: `gf`；PR checks 交叉核对用 `gh pr checks`）
> **变更性质：** 供应链治理变更——① `Makefile`：`audit` 目标将 `cargo vet check` 从「失败时静默跳过」（`2>/dev/null || echo "cargo-vet not configured..."`）改为**硬性门禁**（无 `||` 兜底，失败即中断）；新增 `sbom` 目标（`cargo cyclonedx` 生成 CycloneDX JSON，落地到 `target/sbom/gf.cdx.json`）；`install-tools` 新增 `cargo-cyclonedx` 安装。② `.github/workflows/ci.yml`：`lint`（原 `Lint`）job 在既有 `cargo deny check` + `cargo audit` 之后新增 `taiki-e/install-action@cargo-vet` + `cargo vet check` 两步。③ `.github/workflows/release.yml`：`release` job 新增 `dtolnay/rust-toolchain@stable` + `taiki-e/install-action@cargo-cyclonedx` + `cargo cyclonedx` 生成步骤，产物拷贝为 `release/gf-sbom.cdx.json`，位于既有 attestation 步骤之前。仓库根新增 `supply-chain/audits.toml`、`supply-chain/config.toml`（`cargo vet` 所需配置，`Lint` job 门禁的前置依赖）。

## 零、核心结论先行

采集窗口（03:06:44Z 创建 → 03:09:04Z 左右 `Lint` job 收尾）PR #316 关联的 3 个 workflow run 共 13 个 job：**已收尾的 6 个全部 `success`**（Check 1m25s、MSRV 53s、**Lint 2m33s**——含本 PR 新增的 `cargo vet check` 步骤、Smoke Test×3 platform 1m4s–1m12s），**无一失败**；其余 7 个 job（`Smoke Test`、`Test`×3 平台、`E2E Tests`×3 平台）在报告采集截止时仍 `pending`/`in_progress`。**本 PR 的核心新增门禁——`Lint` job 中的 `cargo vet check`——已实测通过**，说明新增的 `supply-chain/audits.toml`/`config.toml` 配置与现有依赖树审计状态一致，未触发门禁失败。`release.yml` 的新增 SBOM 生成步骤仅在 `release` workflow（tag 触发）中执行，本次 PR 未触发该 workflow，其正确性通过 YAML 语法解析 + 静态审阅确认（见第三节），**未经实际执行验证**。`dev`/`main` 基线保持健康（95%/100%），与既往系列报告一致。

## 一、PR #316 关联流水线实测

`feat/296-cargo-vet-sbom` 分支触发 3 个 workflow run（均创建于 `2026-09-04T03:06:44Z`）：

| Run ID | Workflow | 采集截止状态 | 备注 |
|--------|----------|------|------|
| 33832001998 | 主 CI（Check/MSRV/Lint/Smoke Test/Test×3） | 🟡 running（3/7 job 已完成，均 `success`；4 个仍 `in_progress`） | Lint 于 03:09Z 前后收尾（2m33s） |
| 33832001983 | Smoke Test 跨平台 | ✅ success（3/3 job 全部成功） | 收尾于快照前 |
| 33832002037 | E2E Tests（GitHub/GitLab/GitCode） | 🟡 running（0/3 job 已完成；3 个仍 `in_progress`） | — |

全部已采集到的 job 明细（`gf pipeline jobs` + `gh pr checks` 交叉核对，多次轮询取收敛结果）：

| Job | Workflow run | 状态 | 结论 | 耗时 |
|-----|--------------|------|------|------|
| Check | 33832001998 | completed | ✅ success | 1m25s |
| MSRV | 33832001998 | completed | ✅ success | 53s |
| **Lint**（含新增 `cargo vet check` 步骤） | 33832001998 | completed | ✅ **success** | 2m33s |
| Smoke Test (github) | 33832001983 | completed | ✅ success | 1m4s |
| Smoke Test (gitlab) | 33832001983 | completed | ✅ success | 1m7s |
| Smoke Test (gitcode) | 33832001983 | completed | ✅ success | 1m12s |
| Smoke Test | 33832001998 | pending/in_progress | — | — |
| Test (ubuntu-latest) | 33832001998 | pending/in_progress | — | — |
| Test (windows-latest) | 33832001998 | pending/in_progress | — | — |
| Test (macos-latest) | 33832001998 | pending/in_progress | — | — |
| E2E Tests (GitHub) | 33832002037 | pending/in_progress | — | — |
| E2E Tests (GitLab) | 33832002037 | pending/in_progress | — | — |
| E2E Tests (GitCode) | 33832002037 | pending/in_progress | — | — |

**共 13 个 job**：6 个已收尾，**全部 `success`，无一失败**；7 个仍 `pending`/`in_progress`，报告采集截止时无法判定终态，但历史同名 job（Test/E2E Tests）与本次改动（Makefile + CI workflow + 新增 supply-chain 配置文件）无逻辑关联，理论风险低。

## 二、PR 合并状态说明

`gh pr view 316` 返回 `state: "MERGED"`、`createdAt: "2026-09-04T03:06:41Z"`、`mergedAt: "2026-09-04T03:07:28Z"`——PR 在创建后约 47 秒即被记录为合并，早于其触发的 CI 全部收尾（含本 PR 新增的 `Lint`/`cargo vet check` 门禁在合并记录时点尚未收尾）。与既往系列报告（PR #313/#315）记录的「auto-merge 排队等待必需检查通过」模式一致：`gf pr view`/`gh pr view` 在合并动作完成后立即返回 `MERGED`/`mergedAt`，不代表流水线已全部收尾，也不代表新增门禁在合并前已强制验证通过。此处仅作记录；后续实测确认 `Lint`（含 `cargo vet check`）已于报告采集窗口内收尾为 `success`，未发现门禁被绕过的证据。

## 三、新增 CI/Release 步骤静态审阅（语法与配置一致性）

聚焦本次分析要求的核心问题：新增的 `Lint` job 步骤与 `release.yml` 新增步骤是否语法有效、不破坏 workflow。

1. **YAML 语法**：对 PR 分支上的 `.github/workflows/ci.yml`、`.github/workflows/release.yml` 用 `yaml.safe_load` 解析，**均 OK，无语法错误**。
2. **`ci.yml` `lint` job 新增步骤**（`cargo deny check` + `cargo audit` 之后）：
   ```yaml
   - name: Install cargo-vet
     uses: taiki-e/install-action@cargo-vet
   - name: cargo vet
     run: cargo vet check
   ```
   `taiki-e/install-action@<tool>` 是该 action 的标准调用形式（tool 名作为 ref），与仓库既有 `cargo-audit`/`cargo-deny`/`cargo-nextest` 等步骤风格一致。**实测已通过**（`Lint` job success，2m33s）。
3. **`Makefile` `audit` 目标行为变更**：从 `cargo vet check 2>/dev/null || echo "cargo-vet not configured..."`（失败静默跳过）改为无兜底的硬门禁 `cargo vet check`。该变更依赖仓库根新增的 `supply-chain/audits.toml` + `supply-chain/config.toml`（`cargo vet` 运行时的必需配置，缺失则命令报错退出）。已确认这两个文件随 PR 一并新增，`Lint` job 的实测通过间接验证了该配置对当前依赖树是有效的（未触发 `cargo vet check` 失败）。
4. **`release.yml` `release` job 新增 SBOM 步骤**：
   ```yaml
   - uses: dtolnay/rust-toolchain@stable
   - name: Install cargo-cyclonedx
     uses: taiki-e/install-action@cargo-cyclonedx
   - name: Generate SBOM
     run: |
       cargo cyclonedx --format json --describe binaries --spec-version 1.5
       cp apps/cli/gf_bin.cdx.json release/gf-sbom.cdx.json
   ```
   插入位置在既有 attestation 步骤之前（符合 PR 注释「签名会修改文件哈希，故须在 Sign 之前」的既定顺序约束，未破坏该顺序）。**`release.yml` 仅在 tag/release 触发时执行，PR #316 本身未触发该 workflow，因此该步骤未经实际 CI 执行验证**——仅完成 YAML 语法解析与静态审阅（步骤顺序、action 引用格式、产物路径与 `Makefile` `sbom` 目标的 `apps/cli/gf_bin.cdx.json` 命名一致）。**建议**：待下一次 release 触发时补充一次实测确认（不在本次只读分析范围内主动触发）。
5. **`Makefile` `sbom` 目标**新增 `.PHONY` 声明中的 `sbom` 已正确追加到既有列表，无遗漏。

## 四、`gf pipeline report` 口径假象（第五次复现，与 PR #311/#312/#313/#315 一致）

`gf pipeline report --branch feat/296-cargo-vet-sbom --days 7`（在 run 仍 running 时采集）：

```json
{
  "totalRuns": 3,
  "successRate": 0.0,
  "avgDurationSecs": 10.0,
  "topFailures": [""]
}
```

与既往四次报告（PR #311→#312→#313→#315，`successRate` 分别为 0.5/0.333/0.0/0.0）记录的同一类问题一致——命令将仍处于 `running`（`conclusion` 为空）的 run 计入失败桶。经 `gf pipeline status`/`gf pipeline jobs`/`gh pr checks` 逐 run、逐 job 交叉复核，**已完成的 6 个 job 全部为 `success`，无真实失败**。该问题已连续五次在系列报告中复现，**维持既往建议：提交独立 Issue 改进 `pipeline report` 的运行中状态统计口径**（不在本次只读分析范围内代为提交）。

## 五、dev / main 基线（采集时点：PR #316 触发后）

| 分支 | 周期 | Total runs | Success rate | Avg duration | 评级 |
|------|------|-----------:|--------------:|--------------:|------|
| `dev` | 7 天 | 100 | 95.0% | 150.47s | 🟢 Healthy |
| `main` | 30 天 | 100 | 100.0% | 159.59s | 🟢 Healthy |

基线数值与 PR #311/#312/#313/#315 报告完全一致，延续系列报告观察到的健康水位；PR #316 引入的新增门禁步骤未导致基线抖动。

## 六、Flaky / 失败信号

**PR #316 自身流水线**：已收尾的 6 个 job（含新增 `cargo vet check` 门禁）全部 `success`，未观察到任何失败。7 个 job 采集截止时仍在运行中，未发现异常延迟或卡死迹象。

历史观察清单沿用既往报告记录的 `dev` 分支单次 `Test (windows-latest)` 失败案例（run `33346653353`，2026-08-31，`commands::commit::tests::test_should_resolve_comment_body_from_file`），仍为 1 次，未达 flaky 判定阈值（≥2 次），且与 PR #316（改动范围限于 Makefile/CI workflow/supply-chain 配置，未触及该测试涉及的 `commit.rs` 逻辑）无关联。维持观察清单状态。

## 七、耗时分析

已收尾 job 耗时：MSRV 53s、Smoke Test×3 platform 1m4s–1m12s、Check 1m25s、**Lint 2m33s**。`Lint` 耗时较既往报告记录的历史区间（140s–223s 量级）略高，**主要归因于本 PR 新增的 `taiki-e/install-action@cargo-vet` 安装步骤与 `cargo vet check` 执行本身**（新增两个子步骤），属预期内的一次性门禁引入成本，非异常回归。其余仍在运行的 `Smoke Test`/`Test`×3 平台/`E2E Tests`×3 平台因采集截止时点尚未收尾，无法给出本次耗时数据；历史同名 job 区间为 116s–337s（Test）、140s–223s 量级（E2E Tests），与本次改动（不涉及测试矩阵、依赖版本或构建脚本本体）无理由预期显著偏离。

## 八、结论与 Recommendations

1. 🟢 **Low** — PR #316（Issue #296，cargo-vet 供应链审计门禁 + SBOM 生成）核心新增门禁 `Lint` job 中的 `cargo vet check` **实测通过**（2m33s），已收尾的 6 个 job 全部成功，无失败信号；7 个 job 仍在运行，按惯例本报告不持续轮询等待其全部收尾。**建议**：如需终态确认，可另行执行 `gh pr checks 316` 复核。
2. 🟡 **Low** — `release.yml` 新增的 SBOM 生成步骤（`cargo cyclonedx` + 拷贝为 `release/gf-sbom.cdx.json`）仅通过 YAML 语法解析与静态审阅确认，**未经实际 CI 执行验证**（该 workflow 仅在 release 触发时运行，PR 本身不会触发）。建议在下一次实际 release 流程中重点关注该步骤的首次执行结果（作为该 workflow 首次真实触发时的验证点，不建议为此单独触发一次 release）。
3. 🟡 **Medium** — `gf pipeline report` 命令在 run 处于 `running` 状态时持续将其计入失败桶，本次为**第五次连续复现**（PR #311→#312→#313→#315→#316）。维持既往建议：尽快针对 `gf` CLI 提交独立 Issue，改进 `pipeline report` 使其将 `running`/`queued` 与真实 `failure` 分开统计。
4. 🟡 **Low** — `commands::commit::tests::test_should_resolve_comment_body_from_file`（`apps/cli/src/commands/commit.rs:245`）在 `dev` 分支历史窗口内仍保持 1 次失败记录（run `33346653353`，2026-08-31），未达 flaky 判定阈值，且与 PR #316 无关联。继续维持观察清单状态。
