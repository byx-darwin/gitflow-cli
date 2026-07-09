# 门控规则

本文档定义 Phase 之间的准入条件。编排器在进入下一 Phase 前必须校验这些规则。

## 门控定义

### Gate 1→2: 需求澄清 → 计划制定

**条件:**
- `phases.1.status` 为 `complete`
- `phases.1.evidence.issue_url` 非空
- `phases.1.evidence.comment_id` 非空（审查证据）

**fast 模式豁免:** `comment_id` 可省略（issue-review 可选）

**失败处理:** 阻止进入 Phase 2，返回 Phase 1 执行

### Gate 2→3: 计划制定 → 执行

**条件:**
- `phases.2.status` 为 `complete`
- `phases.2.evidence.spec_path` 非空
- `phases.2.evidence.user_approved` 为 `true`

**fast 模式豁免:** `spec_path` 和 `user_approved` 可省略（writing-plans 可选）

**失败处理:** 阻止进入 Phase 3，返回 Phase 2 执行

### Gate 3→4: 执行 → 交付

**条件:**
- `phases.3.status` 为 `complete`
- `phases.3.evidence.pr_url` 非空
- `phases.3.evidence.tests_passed` 为 `true`

**无豁免**（任何模式都必须通过）

**失败处理:** 阻止进入 Phase 4，返回 Phase 3 TDD 循环

## 门控校验算法

```python
def check_gate(contract, target_phase):
    if target_phase == 2:
        return contract["phases"]["1"]["status"] == "complete" \
               and contract["phases"]["1"]["evidence"].get("issue_url")
    elif target_phase == 3:
        if contract["mode"] == "fast":
            return True  # fast 模式跳过计划
        return contract["phases"]["2"]["status"] == "complete" \
               and contract["phases"]["2"]["evidence"].get("spec_path") \
               and contract["phases"]["2"]["evidence"].get("user_approved")
    elif target_phase == 4:
        return contract["phases"]["3"]["status"] == "complete" \
               and contract["phases"]["3"]["evidence"].get("pr_url") \
               and contract["phases"]["3"]["evidence"].get("tests_passed")
    return False
```

## 多 Agent 门控

当 Agent 从外部（Cursor/CI/Hook）接收流程时：

1. 读取 `.cache/workflows/active/<workflow_id>.json`
2. 检查 `current_phase` 确认当前位置
3. 校验目标 Phase 的 Gate 条件
4. 门控通过 → 进入目标 Phase
5. 门控失败 → 返回错误，交由原 Agent 补齐
