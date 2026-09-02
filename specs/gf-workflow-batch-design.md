# gf-workflow-batch 设计规格

- Issue: #280
- Workflow: wf-2026-09-02-001
- Status: draft（Phase 1 产出，待 Phase 2 转化为实施计划）

## Context

`gf-workflow` 目前是单 Issue 端到端设计：一个 contract 对应一个 issue，Bootstrap
阶段只读一次 open issue 列表挑一个来处理。当需要连续处理多个 open issue 时，
没有现成的批量机制，而直接在同一会话里循环调用会导致每个 issue 的 Phase 1-4
产物（review 报告、subagent 输出等）持续堆积在同一上下文里，几个 issue 就可能
把 context 打爆。

## Goal

新增一个"外层驱动器"（串行版），在不修改 `gf-workflow` 本身单 issue 端到端设计
的前提下，实现批量处理多个 open issue，同时控制上下文增长；并支持在没有可处理
issue 时，通过需求讨论一次性拆解创建多个新 issue，再自动接续批处理。

## Non-Goals

- 不做并行版本。多个 issue 并行会带来 base branch 漂移、approval 请求交错、
  并发 token 成本等问题，留作后续独立评估。
- 不修改 `gf-workflow` 本身的四阶段闸门逻辑或 contract schema。

## Design

### 组件

新增 skill：`skills/gf-workflow-batch/SKILL.md`，命令 `/gf-workflow-batch [--limit N] [--label <label>]`。

与现有 `skills/gf-*` 约定一致，作为独立编排器存在，不修改 `gf-workflow` 本身。

### 顶层流程

```
pending = 计算未被覆盖的 open issue

if pending 非空:
    进入批处理循环
else:
    进入需求讨论模式：
      1. 调用 superpowers:brainstorming 与用户讨论，将大需求拆解为多个独立子任务
      2. 对每个子任务调用 gf-issue-create 创建对应 Issue（仅建 Issue，不派发完整
         /gf-workflow —— 完整四阶段流程留给下面的批处理循环执行）
      3. 全部创建完成后，重新从磁盘计算 pending（自然包含刚创建的 issue）
      4. 自动进入批处理循环
```

### 批处理循环

```
loop:
  pending = (gf issue list --state open)
            - active/*.json 中未完成 contract（status != "complete"）覆盖的 issue
            - archive/**/*.json 中已完成 contract 覆盖的 issue
  if pending empty: break
  issue = pending[0]  # 按 issue number 升序
  dispatch Agent(subagent_type: 默认, prompt: "/gf-workflow #<issue>")
    — 非 fork：子代理不继承外层驱动器的对话历史
    — 串行阻塞等待该 Agent 完成，包括其内部 Gate 2→3 审批
      （子代理内 AskUserQuestion 原样弹给用户，驱动器不做自动批准）
  记录一行摘要：issue 号 / contract 路径 / pr_url or merge_commit / 成功|跳过|失败
  继续下一轮（重新从磁盘推导 pending，不复用内存中的旧列表）

print 汇总表
```

### Issue 覆盖判定

驱动器需要判断某个 open issue 是否已经被某个 contract（active 或 archive）覆盖：

1. **主匹配**：contract 的 `phases.1.evidence.issue_url` 与候选 issue 的 URL 一致。
   该字段在 Phase 1 的 `gf-issue-create` 步骤（对已存在的 issue 是"引用现有"）
   完成后写入，对绝大多数进行中的批量任务足够可靠。
2. **回退匹配**：若 `issue_url` 尚未写入（例如子代理在 Phase 1 早期即失败退出，
   还未跑到 `gf-issue-create`），退化为按 contract `title` 与 issue 标题精确
   字符串匹配。
3. **已知局限**：若一个 contract 既没有 `issue_url` 也没有可匹配的 `title`
   （标题被中途修改等极端情况），该 issue 存在被重复派发的风险。记录为已知限制，
   不在本次范围内进一步加固。

### 失败处理

单个 issue 的子代理执行失败（测试不过、用户在 Gate 2→3 选择 rejected 等）→
记录该 issue 为失败状态，继续处理下一个 pending issue，不中断整个批次。

### 摘要回传

复用 `gf-workflow` Phase 4 已有的"摘要优先，异常才展开"惯例：子代理只需回传
一行状态摘要（issue 号、contract 路径、`pr_url`/`merge_commit`、成功/跳过/失败），
不把完整的 Phase 1-4 产物带回外层对话。驱动器在全部处理完后打印汇总表。

### 无状态设计

批量进度不维护独立的对话记忆或专用状态文件——每一轮循环都从
`.cache/workflows/{active,archive}/` 磁盘状态重新推导 `pending` 列表。即使
驱动器所在会话被压缩、中断或重新拉起，重新调用 `/gf-workflow-batch` 即可从
磁盘状态自愈恢复批量进度（已完成的 issue 因 contract 归档而自动被排除，
进行中的 issue 因 active contract 存在而自动被排除）。

### 参数

- `--limit N`：限制单次运行处理的 issue 数量上限（默认不限制，处理全部 pending）。
- `--label <label>`：按标签过滤候选 issue（例如排除 `wontfix`）。
- 不传参数：处理全部未被覆盖的 open issue；若为空，转入需求讨论模式。

### 并发边界

仅支持串行处理。`gf-workflow-batch` 内部不得并行派发多个 `Agent` 调用；
每轮循环必须等待上一个 issue 的子代理完全结束（包括其 Phase 4）后，才计算
下一轮 pending 并派发下一个。文档需在 SKILL.md 中明确写明"仅支持串行"。

## Testing

- 覆盖判定逻辑（主匹配 + 回退匹配）需要针对 active/archive 双目录、
  issue_url 缺失场景分别验证。
- 失败隔离：模拟单个子代理返回失败状态，验证驱动器继续处理下一个 pending
  issue 而不是终止整个批次。
- 需求讨论模式：验证 pending 为空时正确触发 brainstorming + 批量
  `gf-issue-create`，创建完成后自动重新计算 pending 并接续批处理。
- `--limit`/`--label` 参数的过滤行为。

## Acceptance Criteria

（同 Issue #280）

- [ ] 新增外层驱动机制（`gf-workflow-batch` skill），能够按"无状态 + 隔离
      派发 + 摘要回传"设计批量处理多个 open issue
- [ ] 驱动器自身不维护批量进度的对话态，进度可从 `.cache/workflows/` 目录
      完全重建
- [ ] 每个 issue 仍完整走 `gf-workflow` 四阶段闸门流程，不跳过 Gate 2→3 审批
- [ ] 文档说明该功能仅支持串行处理，不支持多 issue 并行
- [ ] 无 pending issue 时，支持通过需求讨论拆解创建多个新 issue，并自动接续
      批处理（本次澄清新增）
