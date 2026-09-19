# Windows/macOS Test Job 收尾不稳定排查设计（#373）

**Status:** Approved（Bounded 路径，无需架构级评审）
**Workflow:** wf-2026-09-19-008

## 背景

近 5 份 pipeline 分析报告（`pr323`、`pr326`、`issue333`、`issue334`、`issue335`，跨度
2026-09-04 ~ 2026-09-19）持续观测到 `Test (windows-latest)` / `Test (macos-latest)`
job "收尾不稳定"的模式，issue333/334 报告已各升级提示，issue335 为第 3 次升级，
但一直没有 follow-up Issue 或修复动作落地。

## 排查方法与证据

不依赖历史报告的二手结论，直接拉取近 50 次真实 CI workflow run 数据核实：

```bash
gh run list --workflow=ci.yml --limit 50 --json databaseId,status,conclusion,createdAt,headBranch
```

过滤出非 `success` 的 run，逐个用 `gh run view <id> --json jobs` 核对具体哪个 job 失败，
再用 `gh run view <id> --log-failed` 看实际失败日志。

### 发现 1：绝大多数历史"失败"信号是分析工具自身的 bug，不是真实 CI 失败

核对 `pr323`、`pr326` 两份报告的原始数据：两份报告里被标记为"未收尾/可能失败"的
`Test (windows-latest)`/`Test (macos-latest)` 实际上是**报告生成时这两个 job 仍处于
`in_progress`**（历史上这两个 job 一贯是全流水线最慢的两个），并非真实失败。

`gf pipeline report` 在 run 处于非终态时生成快照，会把尚未收尾的 job 计入失败桶，导致
成功率虚假偏低——这个 bug 已经被**连续 10 份报告**记录（PR #311~#326），且报告记录
显示曾有一次专门修复尝试（`5051966`）经实测确认**未能消除问题**。这是一个独立、更严重
的缺陷，已拆分至 **Issue #378** 单独跟踪，不在本次 #373 范围内修复。

由于 windows/macos 恰好是最慢的两个 job，最容易撞上这个"快照时还没跑完"的窗口，才让
它们看起来像是"不稳定"——这是 #378 的次生表现，不是 #373 描述的 CI 基础设施问题本身。

### 发现 2：过去 ~3 周只有 1 次真实的、独立的 Windows 测试失败

| Run | 日期 | 分支 | 失败 job | 根因 |
|---|---|---|---|---|
| 33346653353 | 2026-08-31 | `dev` | 仅 `Test (windows-latest)` | `test_should_resolve_comment_body_from_file` 断言失败（`apps/cli/src/commands/commit.rs:245`），无法从日志取得更细的错误内容（测试只断言 `result.is_ok()`，未打印底层错误）。此后 20 天未再复现，样本量为 1，不具备可复现路径。 |
| 33707190436 | 2026-09-03 | `feat/291-e2e-gitcode-coverage`（旧 feature 分支） | **3 平台同时失败** | `gc` CLI 未安装导致 e2e-gitcode noauth 测试断言文案不匹配——真实测试逻辑 bug，与平台无关，且分支已过时，不属于本次排查范围 |

**结论：不存在系统性的 Windows/macOS CI 基础设施问题（非资源限制、非排队超时）。**
唯一一次真实失败大概率是瞬时环境噪音（如 Windows Defender 对刚写入的临时文件短暂
加锁——已知的 Windows CI 常见诱因），既无复现路径也无足够样本支撑定向修复。

## 修复方案

不追这次唯一的孤立 flake（没有复现路径，强行"修"等于猜），转为防御性加固，满足
Issue #373 验收标准第 1 条，同时为万一未来真的复现留下诊断证据：

1. `.github/workflows/ci.yml` 的 `test` job（第 35 行起）新增 `timeout-minutes: 15`——
   实测三个平台正常收尾都在 3~4 分钟内，15 分钟给足余量，既能兜住真正的 hang/资源
   排队场景，又不会误伤正常波动
2. 新增一个 `if: failure()` 的诊断步骤，失败时采集磁盘空间、内存、`rustc`/`cargo`
   版本信息，通过 `actions/upload-artifact@v4` 上传为 `test-diagnostics-${{ matrix.os }}`，
   为下次万一复现提供区分"资源限制 vs 测试本身"的证据

## 验收标准回应（对齐 #373 原始 AC）

- [x] AC1（补充 job 级 timeout 与失败诊断日志）——本次实现
- [x] AC2（定位根因：资源限制 vs 测试本身）——已定位：既非资源限制也非系统性测试
      flaky，是分析工具的历史计数 bug（已拆分 #378）+ 一次无法复现的孤立瞬时噪音
- [x] AC3（根因确认后完成对应修复）——根因不支持"修复测试代码"这个动作（无复现路径），
      对应的修复是（1）CI 加固（本次）（2）拆分独立 Issue #378 处理真正的病灶
- [ ] AC4（后续至少 3 次报告不再出现该模式）——依赖 #378 修复落地后的观察期，本次
      workflow 范围内无法验证，留给后续报告确认

## 范围外

- 分析工具的 in-progress 误计为失败 bug——已拆分至独立 Issue **#378**
- `test_should_resolve_comment_body_from_file` 的一次性失败——无复现路径，不修

## 测试策略

纯 CI YAML 配置改动，不涉及 Rust 代码。验证方式：
- 本地检查 YAML 语法正确（`yamllint` 或人工核对，pre-commit 的 `check yaml` hook 会兜底）
- 提交后观察下一次真实 CI run，确认 `timeout-minutes` 与诊断上传步骤按配置生效
