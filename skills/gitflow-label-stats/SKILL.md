---
name: gitflow-label-stats
description: 标签统计分析工作流 — 按标签分组统计 Issue 数量，分析优先级分布，识别未分类 Issue，输出标签统计报告
---

# gitflow-label-stats

对仓库中 Issue 的标签使用情况进行统计分析。通过调用 `gitflow issue list --label` 按标签多次获取数据，结合 `gitflow label list` 获取仓库全部标签定义，从标签分组计数、优先级分布、未分类 Issue 识别三个维度输出统计报告，帮助团队了解 Issue 管理健康度。

## 工作流

### 步骤 1：获取仓库标签列表

首先获取仓库中已定义的所有标签：

```bash
# 获取仓库中的所有标签
gitflow label list
```

记录以下信息：

- 标签名称
- 标签颜色
- 标签描述（如有）

**标签分类参考：**

| 类别 | 常见标签 | 说明 |
|------|----------|------|
| 类型 | `bug`, `enhancement`, `documentation`, `question` | Issue 的类型 |
| 优先级 | `priority:urgent`, `priority:high`, `priority:medium`, `priority:low` | 优先级 |
| 状态 | `triage:done`, `in-progress`, `blocked` | 工作状态 |
| 平台 | `github`, `gitlab`, `gitcode` | 平台相关 |
| 其他 | `good-first-issue`, `help-wanted`, `auto-reported` | 特殊标记 |

### 步骤 2：按标签分组统计

对每个标签，统计对应的 open Issue 数量：

```bash
# 按标签统计 Issue 数量（对每个标签执行）
gitflow issue list --label "<label>" --state open --limit 1000

# 同时获取 closed 数量（可选）
gitflow issue list --label "<label>" --state closed --limit 1000
```

**统计维度：**

| 维度 | 命令 | 说明 |
|------|------|------|
| Open Issues | `--state open` | 当前待处理的 Issue |
| Closed Issues | `--state closed` | 已关闭的 Issue |
| 总计 | 两者相加 | 该标签下的总 Issue 数 |

**汇总统计表：**

```markdown
### 标签分组统计

| 标签 | Open | Closed | 总计 | 占比 |
|------|------|--------|------|------|
| bug | <n> | <n> | <n> | <x%> |
| enhancement | <n> | <n> | <n> | <x%> |
| documentation | <n> | <n> | <n> | <x%> |
| priority:high | <n> | <n> | <n> | <x%> |
| ... | | | | |
```

### 步骤 3：分析优先级分布

统计各优先级标签下的 Issue 分布：

```bash
# 统计紧急优先级
gitflow issue list --label "priority:urgent" --state open

# 统计高优先级
gitflow issue list --label "priority:high" --state open

# 统计中优先级
gitflow issue list --label "priority:medium" --state open

# 统计低优先级
gitflow issue list --label "priority:low" --state open
```

**优先级分布分析：**

```markdown
### 优先级分布

| 优先级 | Open 数量 | 占比 | 健康度 |
|--------|-----------|------|--------|
| 🔴 紧急 (urgent) | <n> | <x%> | 正常/告警 |
| 🟠 高 (high) | <n> | <x%> | 正常/告警 |
| 🟡 中 (medium) | <n> | <x%> | 正常 |
| 🟢 低 (low) | <n> | <x%> | 正常 |
```

**健康度判断：**

| 条件 | 健康度 | 说明 |
|------|--------|------|
| urgent 占比 < 10% | 🟢 正常 | 紧急事项比例合理 |
| urgent 占比 10%~20% | 🟡 关注 | 紧急事项偏多 |
| urgent 占比 > 20% | 🔴 告警 | 紧急事项过多，需要调整 |
| high + urgent 占比 > 50% | 🔴 告警 | 优先级设置可能不合理 |

### 步骤 4：识别未分类 Issue

找出没有标签或分类不完整的 Issue：

```bash
# 获取所有 open issues
gitflow issue list --state open --limit 1000

# 筛选出没有标签的 Issue
# （通过对比每个 Issue 的标签字段是否为空）

# 筛选出没有类型标签的 Issue（缺少 type:* 或 bug/enhancement 等）
# 筛选出没有优先级标签的 Issue（缺少 priority:* 标签）
```

**未分类 Issue 分类：**

| 类型 | 判断标准 | 建议操作 |
|------|----------|----------|
| 完全未标记 | 无任何标签 | 需要分类（使用 `gitflow-issue-triage`） |
| 缺少类型 | 有优先级但无类型标签 | 补充类型标签 |
| 缺少优先级 | 有类型但无优先级标签 | 评估并添加优先级 |
| 缺少 triage 标记 | 未标记 `triage:done` | 可能需要 triage |

**未分类统计：**

```markdown
### 未分类 Issue

| 类型 | 数量 | 占比 |
|------|------|------|
| 完全未标记 | <n> | <x%> |
| 缺少类型 | <n> | <x%> |
| 缺少优先级 | <n> | <x%> |
| 已完整分类 | <n> | <x%> |
```

### 步骤 5：生成统计报告

汇总所有分析结果，生成完整的标签统计报告：

```markdown
## 标签统计报告

**仓库:** <owner>/<repo>
**统计时间:** <timestamp>
**Open Issues 总数:** <total>

### 标签分组统计

| 标签 | Open | Closed | 总计 | 占比 |
|------|------|--------|------|------|
| <label-1> | <n> | <n> | <n> | <x%> |
| <label-2> | <n> | <n> | <n> | <x%> |
| ... | | | | |

### 优先级分布

| 优先级 | Open 数量 | 占比 | 健康度 |
|--------|-----------|------|--------|
| 🔴 紧急 | <n> | <x%> | 🟢/🟡/🔴 |
| 🟠 高 | <n> | <x%> | 🟢/🟡/🔴 |
| 🟡 中 | <n> | <x%> | 🟢 |
| 🟢 低 | <n> | <x%> | 🟢 |

### 分类覆盖率

| 指标 | 数值 |
|------|------|
| 已完整分类 | <n> (<x%>) |
| 缺少类型 | <n> (<x%>) |
| 缺少优先级 | <n> (<x%>) |
| 完全未标记 | <n> (<x%>) |

### 改进建议

1. <!-- 具体建议 -->
2. <!-- ... -->
```

### 步骤 6：输出改进建议

根据统计分析结果，给出针对性的改进建议：

| 发现 | 建议 |
|------|------|
| 未分类 Issue > 30% | 运行 `gitflow-issue-triage` 对所有 Open Issue 进行分类 |
| 紧急优先级占比过高 | 重新评估优先级标准，避免将所有 Issue 标为紧急 |
| 某标签 Issue 积压过多 | 集中资源处理积压标签下的 Issue |
| 标签定义不一致 | 统一标签命名规范，合并语义重叠的标签 |
| 缺少 good-first-issue | 为新成员标记一些简单任务 |
| Closed 占比过低 | 加快 Issue 处理速度，定期清理过期 Issue |

## 使用示例

### 基本的标签统计

```bash
# 获取所有标签
gitflow label list

# 按主要标签统计 Open Issues
gitflow issue list --label bug --state open
gitflow issue list --label enhancement --state open
gitflow issue list --label documentation --state open

# 按优先级统计
gitflow issue list --label "priority:high" --state open
gitflow issue list --label "priority:medium" --state open
gitflow issue list --label "priority:low" --state open
```

### 完整的统计分析流程

```bash
# 1. 获取所有标签
gitflow label list > /tmp/labels.txt

# 2. 获取所有 Open Issues
gitflow issue list --state open --limit 1000 > /tmp/all-issues.txt

# 3. 逐个标签统计
while read -r LABEL; do
    COUNT=$(gitflow issue list --label "$LABEL" --state open | wc -l)
    echo "$LABEL: $COUNT"
done < /tmp/labels.txt

# 4. 汇总报告
echo "标签统计完成"
```

### 统计报告输出示例

```markdown
## 标签统计报告

**仓库:** org/gitflow-cli
**统计时间:** 2026-07-02
**Open Issues 总数:** 47

### 标签分组统计

| 标签 | Open | Closed | 总计 | 占比 |
|------|------|--------|------|------|
| bug | 12 | 28 | 40 | 25.5% |
| enhancement | 15 | 20 | 35 | 31.9% |
| documentation | 5 | 10 | 15 | 10.6% |
| priority:high | 8 | 15 | 23 | 17.0% |
| priority:medium | 20 | 25 | 45 | 42.6% |
| priority:low | 10 | 12 | 22 | 21.3% |
| good-first-issue | 6 | 8 | 14 | 12.8% |

### 优先级分布

| 优先级 | Open 数量 | 占比 | 健康度 |
|--------|-----------|------|--------|
| 🔴 紧急 | 3 | 6.4% | 🟢 正常 |
| 🟠 高 | 8 | 17.0% | 🟢 正常 |
| 🟡 中 | 20 | 42.6% | 🟢 正常 |
| 🟢 低 | 10 | 21.3% | 🟢 正常 |
| 未标记 | 6 | 12.8% | 🟡 关注 |

### 分类覆盖率

| 指标 | 数值 |
|------|------|
| 已完整分类 | 38 (80.9%) |
| 缺少类型 | 4 (8.5%) |
| 缺少优先级 | 3 (6.4%) |
| 完全未标记 | 2 (4.3%) |

### 改进建议

1. 🟡 有 9 个 Issue 分类不完整（19.1%），建议运行 `gitflow-issue-triage` 补充分类
2. 🟢 优先级分布健康，紧急事项占比合理
3. 🟢 good-first-issue 数量充足，有利于新成员参与
```

## 注意事项

- 统计结果依赖标签命名的一致性——同一概念的不同命名（如 `bug` vs `type:bug`）应视为同一类别
- `gitflow issue list --label` 支持多次使用进行 AND 过滤，但统计时应逐个标签独立统计
- 未分类 Issue 的识别需要对比每个 Issue 的标签列表与预期的标签分类体系
- 优先级分布的健康度阈值可根据团队实际工作方式调整
- 统计报告应定期生成（如每周或每个 Sprint），跟踪 Issue 管理改善趋势
- 对于已关闭 Issue 的统计，可以帮助了解历史问题分布和团队处理效率
- 如果发现标签体系本身存在问题（如标签过多或语义重叠），应建议团队统一标签规范
- 对于大型仓库（Issue 数量 > 1000），可能需要分批统计以避免超出 API 限制
