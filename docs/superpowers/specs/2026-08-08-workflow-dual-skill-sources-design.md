# gf-workflow 双 skills 来源兼容设计（Issue #141）

> **Workflow**: `wf-2026-08-08-001` · **Phase**: 1 (Clarification) · **Mode**: full
> **日期**: 2026-08-08 · **状态**: 待用户评审
> **Issue**: [#141](https://github.com/byx-darwin/gitflow-cli/issues/141)（含 6 条决策评论，本设计全部采纳，除第 13 节记录的显式偏离）

## 1. 背景与目标

`gf-workflow` 目前硬编码 superpowers 命名空间子技能（`superpowers:brainstorming` / `writing-plans` / `subagent-driven-development`，SKILL.md 共 10 处引用）。用户若以 [mattpocock/skills](https://github.com/mattpocock/skills)（plugin 名 `mattpocock-skills`，marketplace 名 `mattpocock`）作为技能来源，gf-workflow 无法运行。

**目标**：gf-workflow 在 superpowers 与 mattpocock/skills 两种技能来源下均可运行，按来源分支适配各自步骤；编排骨架（契约、四阶段、三闸门、gf-* 步骤）恒定。

**骨架恒定原则**（Issue 评论 #3 结论）：gf-workflow 的闸门/契约对 superpowers 是冗余保险，对 mattpocock 是必需的强制层（mattpocock 体系无 gate 概念、强制力分散）。因此分支适配只替换"执行引擎"，不改骨架。

### 两来源本质差异（设计约束）

| | superpowers | mattpocock/skills |
|---|---|---|
| 定位 | 完整方法论（"给 agent 立规矩"） | 可组合工具箱（"给人递工具"） |
| 组合模型 | 技能链（终态硬编码下一跳） | 扁平流程地图（`/skill` 散文引用） |
| 触发模型 | 模型全自动触发（session hook + 1% 规则） | **user-invoked 硬约束**：`to-spec`/`to-tickets`/`implement`/`triage`/`grill-me`/`handoff` 均 `disable-model-invocation: true`——"nothing but the human can fire it"；仅 `grilling`/`tdd`/`code-review`/`research` 等底层技能模型可达 |
| 状态载体 | 会话期产物（design doc、plan、todo） | 持久项目记忆（`CONTEXT.md` + ADR + `.scratch/<feature>/issues/` 票据） |
| 主线 token | ≈14k + subagent 扇出 | ≈4.8k，不触发零消耗 |

**关键推论**（Issue 评论 #2/#3）：编排器（模型）在 mattpocock 路径**无权自动调用** user-invoked 技能，mattpocock 路径必须采用**暂停语义**（✋ 提示用户敲 slash 命令），成为"人工驾驶流水线"（5 个人触点）；superpowers 路径保持"全自动流水线"（2 个人触点）。

## 2. 已确认设计决策（来自 Issue 正文与评论）

1. **互斥安装假设**：只检测"哪个在场"，不处理共存自动优先级（两者都在时询问用户，见 §4）
2. **按来源分支适配**：SKILL.md 用角色别名写分支步骤；映射表单点维护于 `references.md`
3. **缺失时启动询问**：两来源皆无时询问（内联继续 / 中止），不静默降级
4. **双哨兵防碰撞**：mattpocock 判定用 `to-spec` + `grilling` 双哨兵同时命中（`tdd`/`research` 等通用裸名易与他方技能包碰撞）
5. **Issue 创建权统一**：mattpocock 路径 `to-spec` 约束**只写本地、不发布 tracker**；Issue 一律由 `gf-issue-create` 创建，保证 `issue_url` 语义一致（必须是 gf CLI 平台上的 issue）
6. **Phase 4 骨架不变**：`gf-review`/`gf-issue-triage` 两来源保留；不引入 mattpocock `triage`（两套 label 配置体系不互通，替换会断开 `gf-label-stats` 统计链）
7. **安装时硬阻断**：`gf skills install` 前置检测，两来源皆无 → 安装引导 + 非 0 退出（接受"仅用 gf-issue/gf-pr 的用户也被拦"的代价，保证"装了 gf-workflow 就必然能跑"）
8. **Phase 3 GO 闸门人为选择执行模式**：计划批准与执行授权分离（见 §7）

## 3. 产物布局

Canonical 源在 `skills/gf-workflow/`（git-tracked）；`.claude/skills/gf-workflow/` 为 git-ignored 安装副本，随改动同步。

| # | 文件 | 变更 |
|---|---|---|
| 1 | `skills/gf-workflow/SKILL.md` | 新增「Skill Source Resolution」节（检测算法、结果矩阵、契约记录）；各 Phase 的 `superpowers:*` 硬编码改为**角色别名**；新增暂停语义规则与 GO 闸门执行模式选择；Red Flags / Rationalization Table 增补 |
| 2 | `skills/gf-workflow/references.md` | 新增「双来源映射表」+「来源分支语义」两节——别名→技能映射、调用形态、哨兵权威清单、证据映射、to-spec 约束话术，**单点维护**（上游改名只改此处） |
| 3 | `skills/gf-workflow/gates.md` | 微调：证据字段来源无关说明；Gate 2→3 增加"执行模式选择"子步骤描述；mattpocock 路径 Gate 2→3 展示票据清单 + blocking edges |
| 4 | `skills/gf-workflow/contract.schema.json` | 顶层新增 `skill_source`（新合同必填）；phase evidence 新增 `ticket_refs`（可选数组） |
| 5 | `apps/cli/src/commands/workflow.rs` | `WorkflowContract` 增 `skill_source`、`PhaseEvidence` 增 `ticket_refs`（serde 往返保真）+ 测试 |
| 6 | `apps/cli/src/commands/skills.rs` | `install_skills` 新增 Step 0 安装时来源检测 + 硬阻断 + 共享哨兵常量 + 测试 |
| 7 | `docs/gf-workflow-guide.md` | 新增「技能来源适配」节：检测概述、来源 × Phase 矩阵、安装时阻断说明 |
| 8 | `docs/integration-guide.md` | 新增 mattpocock/skills 集成章：映射表、前置条件、暂停语义、GO 闸门 |
| 9 | `.claude/skills/gf-workflow/` | 安装副本同步（供当前会话与 dogfooding） |

**不改**：`docs/research/skill-analysis-gf-workflow.md`（历史分析文档，定格于 Issue #38）。

## 4. 来源检测与 `skill_source`

### 4.1 哨兵清单（权威定义，Rust 与 references.md 共享）

| 来源 | 哨兵 | 命名空间形 | 裸名形 |
|---|---|---|---|
| superpowers | `brainstorming` | `superpowers:brainstorming`（单哨兵，命名空间无歧义） | `brainstorming` + `writing-plans`（**双哨兵同时命中**，裸名脆弱从严） |
| mattpocock | `to-spec` + `grilling`（**双哨兵同时命中**） | `mattpocock-skills:to-spec` + `mattpocock-skills:grilling` | `to-spec` + `grilling` |

裸名形覆盖 skills.sh / symlink 安装；裸名检测一律双哨兵，防与他方技能包同名碰撞。部分命中（如只有 `to-spec` 无 `grilling`）视同缺失，报告中列出缺项。

### 4.2 运行时检测（编排器层，技能清单探测）

**时机**：Bootstrap 中、合同创建**之前**。

**机制**：自省当前会话 available-skills 清单（"可调用即被检测"，天然覆盖 plugin/skills.sh/symlink 全部安装形态）。文件系统探测仅作检测失败时的**报错诊断辅助**，不作判定依据。

**结果矩阵**：

| 检测结果 | 行为 | `skill_source` |
|---|---|---|
| 仅 superpowers | 直接采用 | `superpowers` |
| 仅 mattpocock | 直接采用 | `mattpocock` |
| 两者都在 | **询问用户**本次用哪个，无默认优先级 | 用户选择 |
| 两者皆无 | **询问**：内联继续 / 中止；中止则不创建合同 | `inline`（内联时） |

**记录**：jq 写入合同顶层 `.skill_source`（新合同必填）。检测应无感——跑 `/gf-workflow` 即自动选择；失败时给出安装命令（`claude plugins install superpowers` / `claude plugins install mattpocock-skills` / `npx skills@latest add mattpocock/skills`），不静默猜测。

**跨会话恢复**：沿用合同记录的 `skill_source`，但恢复时重新验证哨兵仍在场；消失则按"两者皆无"重新询问（防工作流中途用户新装/卸载另一来源导致路径漂移）。

### 4.3 安装时检测（`gf skills install` Step 0，Rust 进程）

Rust 进程看不到 LLM 的 skills 清单，只能探测文件系统/插件注册表：

- **plugin 形态**：解析 `~/.claude/plugins/installed_plugins.json`，匹配 `superpowers@*` / `mattpocock-skills@*` 键
- **裸名形态**：扫描 `~/.claude/skills/`，按多哨兵判定（mattpocock：`to-spec` + `grilling`；superpowers 裸名：`brainstorming` + `writing-plans` 双命中——裸名脆弱，从严）

**行为**：

```
gf skills install
  ├─ Step 0: 检测来源
  │    ├─ 任一在场 → 继续安装，输出 detected_source 提示
  │    └─ 两者皆无 → 输出三条安装引导后【硬阻断】（退出码非 0），不写入任何 gf-* skill
  └─ Step 1: 照常安装 gf-* skills
```

哨兵规则与运行时检测共享同一份权威定义（Rust 常量 + references.md 文档 + 测试显式枚举防漂移），避免"CLI 说装了、编排器说没装"的分裂。

**边界**（计划阶段定稿）：`--agent` 非 claude 时的检查策略；`npx skills` 安装位置的精确探测路径。

## 5. 双来源映射表（references.md 单点维护）

| 角色别名 | superpowers | mattpocock | 调用形态 |
|---|---|---|---|
| 澄清步骤 | `brainstorming` | `grilling` | 模型可触发 / 模型可触发 |
| 规格步骤 | （并入 brainstorming design doc） | ✋ `/to-spec`（约束只写本地） | — / user-invoked |
| Issue 创建 | `gf-issue-create` | `gf-issue-create`（创建权统一，不变） | gf CLI |
| Issue 审查 | `gf-issue-review` | `gf-issue-review`（不变） | gf CLI |
| 计划步骤 | `writing-plans` | ✋ `/to-tickets` | 模型可触发 / user-invoked |
| 质量门 | `gf-quality` | `gf-quality`（不变） | gf CLI |
| 执行引擎 | GO 闸门选择：`subagent-driven-development`（同会话）/ `executing-plans`（新窗口）/ 后台代理 | ✋ `/implement` 逐票据（内部强制 `/tdd`） | 按模式 / user-invoked |
| 执行期审查 | SDD 内置双阶段审查 | `code-review`（`implement` 内部驱动，模型可触发） | — |
| 交付审查 | `gf-review` | `gf-review`（不变，**不附加** code-review，见 §13） | gf skill |
| Triage | `gf-issue-triage` | `gf-issue-triage`（不变） | gf skill |
| 流水线分析 | `gf-pipeline-analyzer` | 同左（不变） | gf skill |

SKILL.md 各 Phase 步骤只引用角色别名；实际技能名与调用形态由此表解析。

## 6. Phase 分支语义

### 6.1 不变量（两来源共享）

四阶段状态机、三道闸门、契约证据字段语义、全部 gf-* 步骤、TDD 强制、代码审查强制、模式矩阵（full/standard/fast）。**superpowers 路径 Phase 1/2 行为零变化**；唯一变化是 Phase 3 GO 闸门执行模式选择（§7，Issue 评论 #6 决策）。

### 6.2 mattpocock 路径

**前置条件**：`docs/agents/issue-tracker.md` 存在（`setup-mat-pocock-skills` 产物）。缺失 → 询问用户：先运行 `setup-mat-pocock-skills`（一次性配置）或中止。

**Phase 1（人工触点 ✋×1）**：

1. `grilling`（模型可触发，自动）：设计树拷问至共享理解
2. ✋ **暂停**：编排器提示用户运行 `/to-spec`，附约束指令："综合当前对话写 spec 到本地文件 `docs/specs/<workflow-id>-spec.md`，**不发布 tracker**、不打标签"
3. 编排器校验本地 spec 文件存在。**回退方案**（约束不可靠时）：编排器根据 grilling 记录自行撰写 design doc，绕过 `to-spec`——plugin 只读无法改其 SKILL.md，约束只能靠调用指令传达，回退必须存在
4. `gf-issue-create` 创建 Issue（引用本地 spec），`gf-issue-review` 审查 → 证据 `issue_url`、`comment_id`、`design_doc_path`（本地 spec 路径）

**Phase 2（人工触点 ✋×1）**：

1. ✋ **暂停**：提示用户运行 `/to-tickets`，参数指向 Phase 1 spec。`to-tickets` 按其 tracker 配置产出票据（本地文件 `.scratch/<feature>/issues/NN-slug.md` 或真实 tracker issue），自带拆解 quiz 与用户确认；其自身规则"do NOT close or modify any parent issue"与 gf-workflow 兼容
2. 编排器记录 `ticket_refs`（票据文件路径或 URL 数组），`spec_path` = Phase 1 spec 文件（`to-tickets` 消费的对象，复用 Phase 1 产物）
3. `gf-quality` 质量门 → Gate 2→3：展示**票据清单 + blocking edges** 供审批，并选择执行模式（§7）

**Phase 3（人工触点 ✋×N，N = 票据数）**：

1. 创建 worktree/分支（执行模式决定由谁创建，§7）
2. 按依赖序（frontier）逐票据 ✋ `/implement`（内部驱动 `/tdd` 强制 + `/code-review` + commit）；票据间建议 `/clear`（mattpocock 上下文纪律），恢复靠契约 + 票据文件
3. `gf-pr-create`（`Closes #<issue>`）+ `make test` → 证据写回

**Phase 4**：与 superpowers 完全相同（gf-pipeline-analyzer → gf-issue-triage[full] → gf-review → dogfooding[full] → Branch Finish → 归档）。`code-review` 已在 Phase 3 由 `/implement` 内部执行，不重复。

### 6.3 fast / standard 模式 × 来源

模式矩阵语义不变，仅执行引擎按来源替换：

- **fast + mattpocock**：`grilling` 可选、`/to-spec` 可选（跳过时 `gf-issue-create` 直接建 Issue，fast 豁免 `design_doc_path` 照旧）；Phase 2 可跳过；Phase 3 `✋ /implement` + `/tdd` 强制
- **standard/full + mattpocock**：完整 §6.2 流程

### 6.4 inline 来源（两者皆无且用户选择继续）

编排器自执行各阶段步骤：自问澄清问题、自写 design doc、自写计划、自跑 TDD + 审查 subagent。证据字段形态与 superpowers 路径一致。降级但显式——合同记录 `skill_source: "inline"` 供审计。

## 7. Phase 3 GO 闸门：计划批准 + 执行模式选择

**异议背景**（Issue 评论 #6）：同会话 SDD"自动触发"把两个同意混为一谈——Gate 2→3 批准计划 ≠ 授权数小时自主执行 + subagent token 扇出；且 SDD Continuous execution 禁止任务间暂停，一旦启动即绑架当前会话。superpowers 本有两种执行技能：`subagent-driven-development`（current session）与 `executing-plans`（separate session with review checkpoints，即"新窗口执行"的官方形态，上下文靠 plan 文档 + 契约 + design doc 三件持久产物恢复）。

**决策**：Gate 2→3 = 计划批准 + 执行模式选择。同会话 SDD 移出默认选项，仅显式要求时可用。

```
⏸ Gate 2→3 approved
      ↓ 用户选择执行模式（模式菜单按来源裁剪）
      ├─ ① 后台代理 ⭐默认推荐
      │    派发 isolation: worktree + run_in_background 后台代理
      │    交接：契约路径 + plan 文档 + 引擎指令
      │    完成：task-notification 回原窗口 + 证据写回契约
      ├─ ② 手动新窗口
      │    输出开窗指引：worktree 路径（或创建命令）+ 契约恢复口令
      │    新窗口自行创建分支/worktree（原 Phase 3 Step 1 移入执行侧）
      │    用户回原窗口报告，编排器校验证据
      └─ ③ 同会话执行（仅显式要求）
           现行为：编排器创建 worktree，同会话驱动执行引擎
      ↓
  执行（SDD / executing-plans / 逐票据 implement）→ gf-pr-create + make test → Gate 3→4 自动
```

| | ① 后台代理 | ② 手动新窗口 |
|---|---|---|
| 操作成本 | 编排器一条派发指令 | 手动 cd + 开会话 + 恢复口令 |
| 完成通知 | 自动 task-notification | 用户自行回报 |
| worktree | 平台托管或代理自建 `feat/<issue>` | 新窗口自行创建 |
| 中途交互 | ⚠️ 弱交互：findings 与 plan 冲突需 SendMessage 回主会话或 ledger 停泊事后裁决 | 新窗口内直接对话 |
| 适用 | 默认路径 | 超长任务 / 想全程跟进 |

**来源约束**：

- superpowers 路径：①②③ 均可用（SDD / executing-plans 均为 model-invoked）
- mattpocock 路径：`/implement` 是 user-invoked，后台代理（模型身份）**无法调用** → 菜单裁剪为 ②③；后台代理仅能覆盖模型可达子集（grilling/tdd/code-review），不足以承担完整 Phase 3

**质量保障**：后台代理/新窗口若走 `executing-plans`（轻量路径），每任务自动审查缺位 → 由阶段闸门补偿（make test PR 前强制 + Phase 4 gf-review）；走 SDD 则自带每任务双阶段审查。plan 文档成为新窗口唯一执行依据，Phase 2 产出质量与 Gate 2→3 审批价值被强化。

## 8. 契约与 Schema 变更

### 8.1 contract.schema.json

```json
"skill_source": {
  "type": "string",
  "enum": ["superpowers", "mattpocock", "inline"],
  "description": "启动时检测的技能来源；新合同必填，跨会话恢复沿用"
}
```

置于顶层，加入 `required`（新合同必填；`additionalProperties: false` 保持）。

phase evidence（`$defs.phase.evidence`）新增：

```json
"ticket_refs": {
  "type": "array",
  "items": { "type": "string" },
  "description": "to-tickets 产出的票据清单（文件路径或 URL），mattpocock 路径 Phase 2 记录"
}
```

可选字段，旧合同仍合法。

### 8.2 Rust（apps/cli/src/commands/workflow.rs）

- `WorkflowContract` 增 `#[serde(skip_serializing_if = "Option::is_none")] pub skill_source: Option<String>`（`Option` 保证读取旧归档合同向后兼容）
- `PhaseEvidence` 增 `#[serde(skip_serializing_if = "Option::is_none")] pub ticket_refs: Option<Vec<String>>`
- `new_contract` 置 `None`（编排器检测后 jq 补写，不给 CLI 加参数——KISS）
- 新增测试：含两新字段的合同反序列化 → 再序列化字段保留（延续"Rust 类型始终与合同 schema 对齐"的既有测试约束）；`status` 输出包含字段
- **已知既有问题（不在本 Issue 修复）**：schema `version` const 为 `"1.1"` 而 Rust `new_contract` 写 `"1.0"`——记录为已知偏差，避免 scope 蔓延

### 8.3 Rust（apps/cli/src/commands/skills.rs）

`install_skills` 增 Step 0（§4.3）：哨兵常量 + 文件系统探测 + 硬阻断（非 0 退出）+ 三条安装引导输出 + 单元测试（临时目录注入，覆盖 plugin 形态 installed_plugins.json 解析与裸名目录探测）。

## 9. 文档同步（AC）

- `docs/gf-workflow-guide.md`：新增「技能来源适配」节——检测概述、来源 × Phase 触点矩阵（superpowers 2 触点 / mattpocock 5 触点）、安装时硬阻断说明、GO 闸门执行模式
- `docs/integration-guide.md`：新增 mattpocock/skills 集成章——映射表、前置条件（`setup-matt-pocock-skills` → `docs/agents/issue-tracker.md`）、暂停语义、token 经济学对比（≈14k vs ≈4.8k）
- `docs/index.md`：登记本设计文档

## 10. 验收标准汇总（Issue 正文 + 评论）

**运行时**：

- [ ] 启动时检测已安装来源（技能清单探测 + 双哨兵），结果写入契约 `skill_source`，跨会话恢复沿用（恢复时重验在场性）
- [ ] 两来源皆无时启动询问（内联 / 中止），不静默继续
- [ ] 两者共存时询问用户选择，无默认优先级
- [ ] Phase 1-3 按来源分支：superpowers 路径除 GO 闸门外行为不变；mattpocock 路径 `grilling`/`to-spec`（P1）、`to-tickets`（P2）、`implement`+`tdd`（P3），暂停语义触点 ✋×5
- [ ] mattpocock 路径 `to-spec` 约束只写本地不发布；Issue 创建权统一归 `gf-issue-create`，无重复 Issue；约束失效时有回退（编排器自写 design doc）
- [ ] mattpocock 路径前置检查 `docs/agents/issue-tracker.md`，缺失时询问

**契约与 Schema**：

- [ ] `contract.schema.json` 支持 `skill_source`（新合同必填）与 `ticket_refs`（可选）
- [ ] Rust `WorkflowContract`/`PhaseEvidence` 与 schema 对齐，往返保真
- [ ] Gate 2→3 mattpocock 路径展示票据清单 + blocking edges；`spec_path` = to-tickets 消费的 spec 文件

**Phase 3 GO 闸门**：

- [ ] Gate 2→3 批准后提供执行模式选择：后台代理（默认推荐）/ 手动新窗口；同会话执行仅显式要求时可用
- [ ] 后台代理模式：worktree 隔离 + 契约/plan 交接 + 完成通知回原窗口 + 证据写回契约
- [ ] 新窗口模式：开窗指引含 worktree 路径与契约恢复方式；新窗口可自行创建分支/worktree
- [ ] mattpocock 来源下模式菜单自动裁剪（后台代理不可用于 `/implement`）

**安装时**：

- [ ] `gf skills install` 安装前检测两来源，任一在场则继续并提示 detected source
- [ ] 两者皆无时输出三条安装引导并硬阻断（退出码非 0），不写入任何 gf-* skill
- [ ] 安装时检测与运行时检测哨兵规则一致（共享权威定义）

**文档与验证**：

- [ ] 同步更新 `docs/gf-workflow-guide.md` 与 `docs/integration-guide.md`
- [ ] 两种来源分别端到端跑通一次完整工作流（§11）

## 11. 验证策略

1. **Rust 门禁**：`make build` / `make test` / `make fmt` / `make clippy`（含 pedantic）——workflow.rs 与 skills.rs 均有改动
2. **Skill/文档校验**：安装副本同步、markdown 校对、索引更新
3. **E2E superpowers**：本 workflow 实例（`wf-2026-08-08-001`）自身；本 workflow 的 Gate 2→3 即首次 dogfood GO 闸门执行模式选择（新行为落地前的手工预演）
4. **E2E mattpocock**（真实插件安装，本 workflow Phase 4 期间）：
   - 用户安装：`/plugin marketplace add mattpocock/skills` → `/plugin install mattpocock-skills@mattpocock`（交互式，用户操作）
   - 用 feature 分支新版 gf-workflow 在独立验证仓库（真实 GitHub tracker）跑完整四阶段小需求
   - 验证点：双来源共存询问（届时 superpowers 也在场）、✋ 暂停触点序列、to-spec 本地约束生效、无重复 Issue、`ticket_refs` 记录、Gate 2→3 票据清单 + 模式裁剪（无后台代理项）、Phase 4 骨架
   - 验证后清理测试产物；插件去留由用户决定

## 12. 计划阶段开放问题

1. **Branch Finish 时序**（Issue 评论 #6 协调点 1）：Phase 4 Branch Finish 需占用主 worktree（checkout + pull + 删分支）；GO 闸门后原窗口忙新业务时需延后确认，或改用 `git -C` 免 checkout 实现
2. **后台代理弱交互裁决协议**（协调点 3）：findings 与 plan 冲突时 SendMessage 上报 vs ledger 停泊事后裁决，需定义
3. `npx skills`（skills.sh）裸名安装形态的精确探测路径（安装位置可能非 `~/.claude/skills/`）
4. `to-spec` 本地写约束的调用指令措辞与回退触发条件（如何判定"约束失效"）
5. `gf skills install --agent` 非 claude 时 Step 0 的处理策略
6. E2E 验证仓库选择与 `setup-matt-pocock-skills` tracker 配置（GitHub vs local markdown）
7. ~~契约 ID 多窗口竞态~~（协调点 2）——**已由 Issue #142 解决**：`create_workflow_at` 使用 `create_new`（O_EXCL）+ 扫描 active/archive 全月份目录 + 冲突顺延重试，无需额外工作

## 13. 决策记录与偏离

| 决策点 | 结论 | 依据 |
|---|---|---|
| 分支组织方式 | 方案 B：SKILL.md 检测 + 别名，references.md 单点映射（否决：A 内联双列——token 膨胀；C 拆两个 skill——违背自动检测；D 安装期生成变体——静默过期风险 + 安装期弱检测 + 违反 AC"启动时检测"） | 2026-08-08 brainstorming 用户确认 |
| 检测机制 | 技能清单探测为主，文件系统仅诊断 fallback | Issue 评论 #1 + 用户确认 |
| Phase 4 审查技能 | **两来源均保持 gf-review，mattpocock 路径不附加 code-review** | ⚠️ **显式偏离** Issue 评论 #1 Q2 结论（原结论为增量附加 Spec 轴）。用户 2026-08-08 明确拍板"不附加"。理由：mattpocock 路径 code-review 已在 Phase 3 由 `/implement` 内部驱动执行，Phase 4 重复全 diff 审查为噪音 |
| 运行时两来源共存 | 询问用户，无默认优先级 | 互斥安装假设的缺口补丁（E2E 必经场景），2026-08-08 用户确认 |
| schema version 偏差（1.1 vs 1.0） | 记录为已知问题，本 Issue 不修 | scope 纪律 |
