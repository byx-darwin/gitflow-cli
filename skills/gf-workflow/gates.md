# 门控规则

本文档定义 Phase 之间的准入条件。编排器在进入下一 Phase 前必须校验这些规则。

## 门控定义

> **证据字段来源无关（Issue #141）:** 三个闸门的证据条件不因技能来源变化——
> `design_doc_path` / `spec_path` 等字段由两来源各自的产物填充
> （映射见 `references.md` → Source Branch Semantics），闸门只校验字段存在性与取值。
> mattpocock 路径额外产出 `ticket_refs`（可选字段，不参与闸门判定，Gate 2→3 展示用）。

### Gate 1→2: 需求澄清 → 计划制定

**条件:**
- `phases.1.status` 为 `complete`
- `phases.1.evidence.issue_url` 非空
- `phases.1.evidence.comment_id` 非空（审查证据）
- `phases.1.evidence.design_doc_path` 非空（设计文档路径）

**fast 模式豁免:**
- `comment_id` 可省略（issue-review 可选）
- `design_doc_path` 可省略（brainstorming 可选）

**standard 模式:** 无豁免（与 full 模式相同）

**失败处理:** 阻止进入 Phase 2，返回 Phase 1 执行

**自动流转:** Gate 1→2 通过后，**自动进入 Phase 2**（无需用户确认）

### Gate 2→3: 计划制定 → 执行

**条件:**
- `phases.2.status` 为 `complete`
- `phases.2.evidence.spec_path` 非空
- `phases.2.evidence.user_approved` 为 `true`

**fast 模式豁免:** `spec_path` 和 `user_approved` 可省略（writing-plans 可选）

**standard 模式:** 无豁免（与 full 模式相同，writing-plans 必选）

**失败处理:** 阻止进入 Phase 3，返回 Phase 2 修改计划

**暂停行为:** Gate 2→3 是**唯一需要用户确认的闸门**
- 编排器暂停并展示计划文档路径
- 等待用户输入: "approved" / "changes requested" / "rejected"
- 用户批准后，**自动进入 Phase 3**

**GO 闸门——执行模式选择（Issue #141）:** 用户批准后、进入 Phase 3 前，编排器必须提供执行模式选择：
① 后台代理（默认推荐，仅 superpowers 来源可用）② 手动新窗口 ③ 同会话执行（仅显式要求）。
mattpocock 来源下菜单自动裁剪为 ②③（`/implement` 为 user-invoked，后台代理无法调用）。
模式语义详见 `references.md` → Phase 3 Execution Modes。

提示文案须告知：Phase 3 Step 1 会先跑 **Worktree Preflight**，主工作区若有与本工作流无关的改动会再次中断询问。
此处**不新增闸门条件**——Gate 2→3 只校验合同证据；且模式 ①② 下建 worktree 的是执行者，
在闸门里查树状态保护不到它，preflight 必须随 handoff 下发。

### Gate 3→4: 执行 → 交付

**条件:**
- `phases.3.status` 为 `complete`
- 交付证据二选一（`delivery_mode` 缺省视为 `"pr"`）：
  - `delivery_mode == "pr"` → `phases.3.evidence.pr_url` 非空
  - `delivery_mode == "local_merge"` → `phases.3.evidence.merge_commit` 非空
- `phases.3.evidence.tests_passed` 为 `true`（两种交付方式均必须）

**本闸门不证明什么:** `tests_passed` 来自 Phase 3 Step 4 的**本地** `make test` / `cargo test`，
它是前置自检，**不是 CI 结论**。真正的合并闸门是平台的必需检查 + Step 5 的排队合并
（`gf pr merge --auto`）——平台会在检查不通过时拒绝合并，所以这里无需、也不应重复判定。

`merge_queued` **不作为闸门条件**：GitCode 无排队合并能力，若强制为 `true` 会把 GitCode
流程永久卡在 Phase 3。该字段只作审计记录（是否已排队 / 是否需人工合并）。

**无豁免**（任何模式都必须通过）

**失败处理:** 阻止进入 Phase 4，返回 Phase 3 TDD 循环

**自动流转:** Gate 3→4 通过后，**自动进入 Phase 4**（无需用户确认）

## 门控校验算法

```python
def check_gate(contract, target_phase):
    mode = contract["mode"]

    if target_phase == 2:
        evidence = contract["phases"]["1"]["evidence"]
        if mode == "fast":
            # fast 模式豁免 comment_id 和 design_doc_path
            return contract["phases"]["1"]["status"] == "complete" \
                   and evidence.get("issue_url")
        # standard 和 full: 所有证据必须
        return contract["phases"]["1"]["status"] == "complete" \
               and evidence.get("issue_url") \
               and evidence.get("comment_id") \
               and evidence.get("design_doc_path")

    elif target_phase == 3:
        if mode == "fast":
            return True  # fast 模式跳过计划
        # standard 和 full: spec + 批准必须
        evidence = contract["phases"]["2"]["evidence"]
        return contract["phases"]["2"]["status"] == "complete" \
               and evidence.get("spec_path") \
               and evidence.get("user_approved")

    elif target_phase == 4:
        evidence = contract["phases"]["3"]["evidence"]
        delivery_mode = evidence.get("delivery_mode", "pr")
        if delivery_mode == "local_merge":
            delivery_ok = bool(evidence.get("merge_commit"))
        else:
            delivery_ok = bool(evidence.get("pr_url"))
        return contract["phases"]["3"]["status"] == "complete" \
               and delivery_ok \
               and evidence.get("tests_passed")

    return False
```

### Phase 4 步骤选择

```python
def get_phase4_steps(mode):
    """按模式返回 Phase 4 步骤分组：parallel 组内的步骤单条消息内并发派发
    （各自 Agent 独立执行、互不读取彼此产出，因为三者只共享 Phase 3 的
    pr_url/branch，无真实数据依赖），全部返回后再按顺序执行 sequential 组。"""
    parallel = ["pipeline"]
    if mode == "full":
        parallel.append("triage")
    if mode in ("full", "standard"):
        parallel.append("review")

    sequential = []
    if mode == "full":
        sequential.append("dogfooding")  # 本地 checklist，成本低，不必并行
    sequential.append("branch_finish")

    return {"parallel": parallel, "sequential": sequential}
```

**并行派发约束：**
1. 单条消息内一次性发出 `len(parallel)` 个 Agent 调用（不要逐个 await），编排器等待全部返回后才进入 sequential 组
2. 每个子 Agent 独立读取仓库/PR 上下文（不共享编排器会话状态），并被显式告知：把完整报告写入磁盘、仅在回复里返回一行状态摘要（`✅ <step>: no findings` / `⚠️ <step>: N findings, see <path>`）——遵守 SKILL.md 「Reporting Granularity」的约定，避免并行派发把 token 成本推高抵消 Phase 4 报告瘦身的收益
3. 子 Agent 内部**不得**写 workflow contract；`evidence` 只能由编排器在全部并行结果返回后、进入 sequential 组之前统一写入（与现有「join 后才更新 contract」的模式一致，避免并发写入同一份 JSON 合同）
4. 任一并行步骤失败：编排器等待其余步骤完成后再统一处理失败（不要因为一个步骤失败就取消其余仍在跑的步骤），失败步骤的详情随一行摘要的 `⚠️` 提示一并给出

## 自动流转规则

编排器根据以下规则自动流转：

| Phase 转换 | 触发条件 | 行为 |
|-----------|---------|------|
| Phase 1 → Phase 2 | Gate 1→2 通过 | **自动进入** Phase 2 |
| Phase 2 → Phase 3 | Gate 2→3 通过 + `user_approved = true` | **暂停等待**用户审批，批准后**自动进入** Phase 3 |
| Phase 3 → Phase 4 | Gate 3→4 通过 | **自动进入** Phase 4 |
| Phase 4 完成 | 所有检查通过 | **归档合同**，工作流结束 |

**关键约束:**
1. 子 skill 完成后，编排器立即调用下一个子 skill，不等待用户输入
2. **唯一暂停点**：Gate 2→3（计划审批）
3. 每个 Phase 完成后，立即更新 contract，然后检查 gate 条件

## 多 Agent 门控

当 Agent 从外部（Cursor/CI/Hook）接收流程时：

1. 读取 `.cache/workflows/active/<workflow_id>.json`
2. 检查 `current_phase` 确认当前位置
3. 校验目标 Phase 的 Gate 条件
4. 门控通过 → 进入目标 Phase
5. 门控失败 → 返回错误，交由原 Agent 补齐
