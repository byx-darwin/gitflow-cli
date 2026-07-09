# 门控规则

本文档定义 Phase 之间的准入条件。编排器在进入下一 Phase 前必须校验这些规则。

## 门控定义

### Gate 1→2: 需求澄清 → 计划制定

**条件:**
- `phases.1.status` 为 `complete`
- `phases.1.evidence.issue_url` 非空
- `phases.1.evidence.comment_id` 非空（审查证据）
- `phases.1.evidence.design_doc_path` 非空（设计文档路径）

**fast 模式豁免:**
- `comment_id` 可省略（issue-review 可选）
- `design_doc_path` 可省略（brainstorming 可选）

**失败处理:** 阻止进入 Phase 2，返回 Phase 1 执行

**自动流转:** Gate 1→2 通过后，**自动进入 Phase 2**（无需用户确认）

### Gate 2→3: 计划制定 → 执行

**条件:**
- `phases.2.status` 为 `complete`
- `phases.2.evidence.spec_path` 非空
- `phases.2.evidence.user_approved` 为 `true`

**fast 模式豁免:** `spec_path` 和 `user_approved` 可省略（writing-plans 可选）

**失败处理:** 阻止进入 Phase 3，返回 Phase 2 修改计划

**暂停行为:** Gate 2→3 是**唯一需要用户确认的闸门**
- 编排器暂停并展示计划文档路径
- 等待用户输入: "approved" / "changes requested" / "rejected"
- 用户批准后，**自动进入 Phase 3**

### Gate 3→4: 执行 → 交付

**条件:**
- `phases.3.status` 为 `complete`
- `phases.3.evidence.pr_url` 非空
- `phases.3.evidence.tests_passed` 为 `true`

**无豁免**（任何模式都必须通过）

**失败处理:** 阻止进入 Phase 4，返回 Phase 3 TDD 循环

**自动流转:** Gate 3→4 通过后，**自动进入 Phase 4**（无需用户确认）

## 门控校验算法

```python
def check_gate(contract, target_phase):
    if target_phase == 2:
        evidence = contract["phases"]["1"]["evidence"]
        if contract["mode"] == "fast":
            # fast 模式豁免 comment_id 和 design_doc_path
            return contract["phases"]["1"]["status"] == "complete" \
                   and evidence.get("issue_url")
        return contract["phases"]["1"]["status"] == "complete" \
               and evidence.get("issue_url") \
               and evidence.get("comment_id") \
               and evidence.get("design_doc_path")
    elif target_phase == 3:
        if contract["mode"] == "fast":
            return True  # fast 模式跳过计划
        evidence = contract["phases"]["2"]["evidence"]
        return contract["phases"]["2"]["status"] == "complete" \
               and evidence.get("spec_path") \
               and evidence.get("user_approved")
    elif target_phase == 4:
        evidence = contract["phases"]["3"]["evidence"]
        return contract["phases"]["3"]["status"] == "complete" \
               and evidence.get("pr_url") \
               and evidence.get("tests_passed")
    return False
```

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
