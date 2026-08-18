# 设计文档 — 修复主动上报 bug 功能（h2 升级 + P0 修复）

> **设计日期：** 2026-08-18
> **工作流：** `wf-2026-08-18-006`（standard 模式）
> **来源：** 上轮评估 `wf-2026-08-18-005`（NOT OK 判定 + P0 建议）
> **依据：** `docs/2026-08-18-autoreport-bug-multi-role-eval-report.md` · `docs/superpowers/plans/evidence-baseline-wf-2026-08-18-005.md`

---

## 1. 目标

修复「主动上报 bug」功能端到端链路的 4 个阻断性问题：

| # | 子任务 | 类型 | 对应证据 |
|---|--------|------|----------|
| T1 | h2 v0.4.15 → 0.4.16+ 漏洞升级 | 依赖 | RUSTSEC-2026-0258 |
| T2 | 创建 `auto-report` label | 配置 | E39, E30 |
| T3 | 消除触发非确定性（技能名 + 指令 + matcher） | hook+settings | E14, E12, E21 |
| T4 | 写入端错误分类（拒绝用户输入错误） | Rust | E36, E37 |

## 2. 变更面

| 文件 | 变更 | 类型 |
|------|------|------|
| `Cargo.lock` | `h2` 升级至 0.4.16+ | 依赖 |
| GitHub 仓库 | 创建 `auto-report` label | 配置 |
| `hooks/auto-report-bug.sh` | 技能名修正 + 指令强化 | bash |
| `.claude/settings.json` | Stop matcher `gitflow` → `gf` | 配置 |
| `docs/integration-guide.md` | matcher 说明同步 | 文档 |
| `apps/cli/src/error_reporter.rs` | 错误分类过滤 + 测试 | Rust |
| `apps/cli/src/main.rs` | 用户输入错误标记上报 | Rust |
| `apps/cli/src/commands/issue.rs` | state 校验 miette code | Rust |
| `apps/cli/src/commands/pr.rs` | state 校验 miette code | Rust |
| `hooks/tests/auto-report-bug.bats` | banner 技能名断言 | 测试 |

## 3. 详细设计

### T1: h2 升级

- `cargo update -p h2` → 0.4.16+（RUSTSEC-2026-0258 patched in 0.4.16）
- **不修改 `deny.toml`**（升级是正确解法，无需 ignore）
- 验证：`cargo deny check advisories` 确认 h2 不再出现

### T2: auto-report label

- `gf label create --color d73a4a auto-report`（目标仓库默认 `byx-darwin/gitflow-cli`）
- 验证：`gf label list` 确认

### T3: 触发确定性

1. **技能名修正**：`hooks/auto-report-bug.sh:11,133` — `gitflow-autoreport-bug` → `gf-autoreport-bug`
2. **指令强化**：banner 从「请加载」改为结构化强制指令：
   `MUST load the gf-autoreport-bug skill now to process this error report.`
3. **matcher 更新**：`.claude/settings.json` Stop matcher `"gitflow"` → `"gf"`（CLI 已改名 gf，Stop reason 现为 gf 语境）
4. **文档同步**：`docs/integration-guide.md` matcher 说明更新
5. **测试**：`hooks/tests/auto-report-bug.bats` 增加「banner 含正确技能名」断言

### T4: 错误分类（核心）

**现状：** 所有命令错误冒泡到 `main.rs:144` 统一报 `CLI_ERROR`，用户输入错误（`--state invalid`）与真实缺陷无法区分。

**方案：miette code 标记 + reporter 过滤**

1. **标记错误类型**：新增 `UserInputError`（miette `Diagnostic`，`#[diagnostic(code = "gf::user_input")]`），用于参数/输入校验失败
2. **调用点改造**：`issue.rs:210` / `pr.rs:259` 的 state 校验 `Err(miette::miette!(...))` → `Err(UserInputError::new(...))`
3. **main.rs 分类**：`main.rs:144` 错误处理处，检测 miette code 是否为 `gf::user_input`：
   - 是 → `report_error_noninteractive(..., "USER_INPUT_ERROR")`
   - 否 → 保持 `CLI_ERROR`
4. **reporter 过滤**：`error_reporter.rs::maybe_report_error` 当 `error_code == "USER_INPUT_ERROR"` 时返回 `Ok(())` 不落盘
5. **其余错误**（运行时、平台、网络）仍报 `CLI_ERROR` → 正常上报

**TDD 测试：**
- `test_should_skip_user_input_error`：`maybe_report_error(..., "USER_INPUT_ERROR", ...)` → pending.json 不存在
- `test_should_report_real_error`：`maybe_report_error(..., "CLI_ERROR", ...)` → pending.json 存在
- `UserInputError` 的 code 提取测试

## 4. 验收标准

- [ ] `cargo deny check advisories` 无 h2 报警
- [ ] `auto-report` label 存在于仓库
- [ ] hook banner 引用 `gf-autoreport-bug`（非 gitflow-）；Stop matcher = `gf`
- [ ] 用户输入错误不写入 pending.json；真实缺陷正常写入
- [ ] `make test` + clippy 通过；hook bats 通过

## 5. 范围外（不做）

- 不实施 P1/P2 建议（去重粒度、co_contribution 可发现性、pending 多报告队列、品牌统一等）
- 不修改 `deny.toml` 策略
- 不创建新 Issue 跟踪（本设计只修复 P0）
