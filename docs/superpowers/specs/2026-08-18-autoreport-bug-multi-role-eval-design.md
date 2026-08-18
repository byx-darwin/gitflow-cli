# 多角色评估设计：主动上报 bug 功能（gf-autoreport-bug）是否 OK

> **设计日期：** 2026-08-18
> **工作流：** `wf-2026-08-18-005`（standard 模式）
> **任务来源：** 用户发起「启动多角色评估主动上报bug 功能是否ok」
> **评估对象：** `gf-autoreport-bug` skill + `hooks/auto-report-bug.sh`（Stop Hook）+ `apps/cli/src/error_reporter.rs`（pending.json 写入端）+ 触发与去重机制
> **方法论基准：** Issue #93（多角色项目评估：产品负责人 / 架构师 / DevOps / 开源社区运营 / 终端用户）

---

## 1. 评估目标

判断「主动上报 bug 功能」**是否 OK**——即端到端链路（CLI 失败 → 写入报告 → Hook 捕获 → 分析 → 去重 → 创建 Issue → 清理）在当前实现下能否稳定、正确地履行其设计意图（收集真实 CLI 缺陷并自动化为 Issue）。

**产出形态：** 仅评估报告 + 改进建议清单。**不实施代码修复**（修复需另起 workflow）。

## 2. 评估范围（全链路）

| 环节 | 代码/资产 | 评估问题 |
|------|-----------|----------|
| 写入端 | `apps/cli/src/error_reporter.rs` + `main.rs` | 触发条件是否明确？是否误报/漏报？数据是否安全？ |
| 触发端 | `.claude/settings.json` Stop Hook | matcher 覆盖是否正确？是否非确定性？ |
| 捕获端 | `hooks/auto-report-bug.sh` | 校验/认证/兜底逻辑是否健壮？CLI 一致性？ |
| 处理端 | `gf-autoreport-bug` skill | 文档规范性、职责边界、去重、Issue 创建、清理 |
| 支撑文档 | `docs/references/gf-autoreport-bug-params.md` | 与 skill 的一致性（命令、schema、去重） |

## 3. 评估框架：5 角色 × 3 层面

### 3.1 角色视角

| 角色 | 关注点 |
|------|--------|
| 产品负责人 | 功能是否兑现设计意图；对用户/社区的价值；误报/漏报影响 |
| 架构师 | 模块边界、触发链确定性、状态管理（pending.json）、可扩展性 |
| DevOps 工程师 | 端到端可靠性、认证/去重/清理、幂等性、失败恢复 |
| 开源社区运营 | Issue 质量、重复噪音、对贡献者的打扰程度、流程可理解性 |
| 终端用户 | 是否打扰、是否透明可控（co_contribution 开关）、隐私安全 |

### 3.2 评估层面

| 层面 | 子项 |
|------|------|
| ① 文档规范性 | description 是否触发条件导向；Overview/When to Use/Red Flags 结构；token 效率 |
| ② 职责边界 | 只报告不修复；范围声明；禁止行为与反制表 |
| ③ 可测试性 & 可靠性 | 单元/集成/端到端测试覆盖；触发确定性；错误处理路径；幂等性 |

## 4. 已核实的证据基线（评估起点）

> 以下均为本评估启动时实测/代码审阅所得，作为各角色判定的证据锚点。

### 4.1 积压实例：pending.json 未处理（约 9.5h）

```json
{
  "id": "00000000000067e286117f2a69acddeb",
  "source": "cli",
  "command": "issue",
  "platform": "github",
  "exit_code": 1,
  "error_code": "CLI_ERROR",
  "error_message": "Invalid state 'invalid'. Expected 'open', 'closed', or 'all'.",
  "timestamp": "2026-08-18T06:59:16Z"
}
```

- 该报告为**用户输入错误**（传了非法 `--state` 值），不是 CLI 缺陷 → **误报**（把用户传参错误当成 bug 上报）
- 未被处理：说明自动触发链**没有在真实场景中确定性工作**（或用户手动忽略了 banner）

### 4.2 零产出：从未成功创建过 `[auto-report]` Issue

- `gf issue list --search "auto-report"`（open + all）均返回空 → 功能历史上**从未落地**
- 去重逻辑在没有历史的前提下形同虚设；也无真实数据校验 `gf issue create --label auto-report` 是否可用

### 4.3 触发链脆弱性（多跳非确定性）

| 跳 | 依赖 | 风险 |
|----|------|------|
| 1 | Hook matcher = `gitflow` | Stop reason 不含该词 → 不触发 |
| 2 | `auto-report-bug.sh` 仅打印 banner「请加载 skill」 | **不会自动调用 skill**，依赖模型看见 banner 后自主决策 → 非确定性 |
| 3 | skill 端到端依赖 `gf auth` / Issue API | 若失败仅输出提示，不重试 |

### 4.4 双重隐藏闸门

- `maybe_report_error` 要求 **stderr 非 TTY**（`should_skip_reporting`）
- **且** `gitflow.co_contribution = true`（项目或全局 `settings.json`；当前仓库 `.claude/settings.json` 无此标志）
- → 用户很可能**根本不知道**功能存在/被 gate；无文档说明该开关

### 4.5 CLI 不一致

- Hook 用 `gh auth status` 做认证；skill 强制 `gf` CLI（`gf auth status`）→ 认证判定口径不一致
- 去重命令：skill 版 `gf issue list --search "[auto-report] {command} {error_code}"`（无 `--state all`）vs params doc 版（有 `--state all`）→ 不一致

### 4.6 安全良好项（✓）

- GitHub token / 家目录路径脱敏（`error_reporter.rs` 正则）
- `pending.json` 权限 `0o600`
- best-effort 写入，失败不阻塞退出码
- 报告来源 `source: "cli"`、`id` 防碰撞

## 5. 交付物

1. 本设计文档（`docs/superpowers/specs/2026-08-18-autoreport-bug-multi-role-eval-design.md`）
2. Issue：记录多角色评估任务（`wf-2026-08-18-005` 关联）
3. 评估报告正文（作为 Issue body 或独立报告文档，交付时产出）

## 6. 判定标准

- **判定等级：** OK / 有条件 OK / NOT OK（分角色给结论，最终给总体判定）
- **改进建议分级：** P0（阻断正确性）/ P1（重要）/ P2（打磨），每条标注负责角色与建议归属（skill 文档 / hook / error_reporter / settings）
