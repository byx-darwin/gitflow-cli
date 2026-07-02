---
name: gitflow-issue-triage
description: Issue 分类分流工作流 — 获取所有 open issues，按类型和优先级分类，标记 triage 标签并输出分类报告
---

# gitflow-cli issue triage 工作流

引导用户对项目的所有 open issues 进行结构化分类和优先级评估，帮助团队快速了解待办事项的全貌，合理分配资源。分类完成后为每个 Issue 添加 `triage:done` 标签，并输出汇总报告。

## 工作流

### 步骤 1：获取所有 Open Issues

调用 `gitflow-cli issue list` 获取当前所有 open 状态的 issues：

```bash
gitflow-cli issue list --state open
```

记录以下信息：

- Issue 编号
- 标题
- 描述（前几行）
- 现有标签
- 创建时间
- 指派人（如有）

### 步骤 2：按类型分类

根据每个 Issue 的标题和描述，将其归入以下类型之一：

| 类型 | 标签 | 判断依据 |
|------|------|----------|
| 缺陷 | `type:bug` | 报告了错误行为、崩溃、异常 |
| 功能 | `type:feature` | 请求新功能或新模块 |
| 增强 | `type:enhancement` | 改进现有功能的体验或性能 |
| 文档 | `type:docs` | 文档缺失、错误、需要更新 |
| 问题 | `type:question` | 提问、讨论、需要澄清 |

**分类原则：**

- 如果 Issue 已有类型标签且合理，保留现有标签
- 如果无法明确分类，标记为 `type:unknown` 并在报告中说明
- 一个 Issue 只应有一个主类型标签

### 步骤 3：按优先级评估

根据以下标准评估每个 Issue 的优先级：

| 优先级 | 标签 | 判断标准 |
|--------|------|----------|
| 紧急 | `priority:urgent` | 影响生产环境、安全漏洞、阻塞其他工作 |
| 高 | `priority:high` | 核心功能缺陷、重要用户需求、近期 milestone 必须完成 |
| 中 | `priority:medium` | 一般功能需求、体验改进、非阻塞性问题 |
| 低 | `priority:low` | 锦上添花、文档小修正、未来考虑的功能 |

**评估参考因素：**

- 是否影响核心用户路径
- 涉及用户数量范围
- 是否有 workaround
- 是否关联即将发布的 milestone
- 是否为安全相关问题

### 步骤 4：标记已分类的 Issues

对每个已完成分类的 Issue，调用 `gitflow-cli issue label` 添加 `triage:done` 标签：

```bash
gitflow-cli issue label <issue-number> --label "triage:done"
```

同时添加对应的类型和优先级标签：

```bash
gitflow-cli issue label <issue-number> --label "type:bug" --label "priority:high"
```

### 步骤 5：输出分类报告

汇总所有分类结果，生成分类报告。格式：

```markdown
## Issue 分类报告

**分析时间:** <timestamp>
**Open Issues 总数:** <total>

### 按类型统计

| 类型 | 数量 | 占比 |
|------|------|------|
| 缺陷 (bug) | <n> | <p>% |
| 功能 (feature) | <n> | <p>% |
| 增强 (enhancement) | <n> | <p>% |
| 文档 (docs) | <n> | <p>% |
| 问题 (question) | <n> | <p>% |

### 按优先级统计

| 优先级 | 数量 | 占比 |
|--------|------|------|
| 紧急 (urgent) | <n> | <p>% |
| 高 (high) | <n> | <p>% |
| 中 (medium) | <n> | <p>% |
| 低 (low) | <n> | <p>% |

### 详细分类清单

#### 🔴 紧急 (Urgent)

| # | 标题 | 类型 | 建议 |
|---|------|------|------|
| <n> | <标题> | <类型> | 简要处理建议 |

#### 🟠 高 (High)

| # | 标题 | 类型 | 建议 |
|---|------|------|------|
| <n> | <标题> | <类型> | 简要处理建议 |

#### 🟡 中 (Medium)

| # | 标题 | 类型 | 建议 |
|---|------|------|------|
| <n> | <标题> | <类型> | 简要处理建议 |

#### 🟢 低 (Low)

| # | 标题 | 类型 | 建议 |
|---|------|------|------|
| <n> | <标题> | <类型> | 简要处理建议 |

### 行动建议

<!-- 基于分类结果给出团队行动建议，如：
- 建议立即处理紧急 issues
- 建议将高优先级 issues 纳入当前 sprint
- 建议关闭重复或过时的 issues
-->
```

## 使用示例

### 对一个项目的 open issues 进行全部分类

```bash
# 获取所有 open issues
gitflow-cli issue list --state open

# 假设返回 8 个 issues，逐一分析后打标签
gitflow-cli issue label 10 --label "type:bug" --label "priority:urgent" --label "triage:done"
gitflow-cli issue label 11 --label "type:feature" --label "priority:high" --label "triage:done"
gitflow-cli issue label 12 --label "type:enhancement" --label "priority:medium" --label "triage:done"
gitflow-cli issue label 13 --label "type:docs" --label "priority:low" --label "triage:done"
gitflow-cli issue label 14 --label "type:bug" --label "priority:high" --label "triage:done"
gitflow-cli issue label 15 --label "type:question" --label "priority:low" --label "triage:done"
gitflow-cli issue label 16 --label "type:feature" --label "priority:medium" --label "triage:done"
gitflow-cli issue label 17 --label "type:bug" --label "priority:medium" --label "triage:done"

# 输出分类报告到终端或文件
cat > /tmp/triage-report.md << 'EOF'
## Issue 分类报告

**分析时间:** 2026-07-02
**Open Issues 总数:** 8

### 按类型统计

| 类型 | 数量 | 占比 |
|------|------|------|
| 缺陷 (bug) | 3 | 37.5% |
| 功能 (feature) | 2 | 25.0% |
| 增强 (enhancement) | 1 | 12.5% |
| 文档 (docs) | 1 | 12.5% |
| 问题 (question) | 1 | 12.5% |

### 行动建议

- #10 (urgent bug) 建议立即修复，影响生产环境登录流程
- #11 (high feature) 建议纳入当前 sprint，与 roadmap 一致
- #14 (high bug) 建议在发布前修复
EOF
```

### 仅分类最近一周新增的 issues

```bash
# 先筛选最近创建的 issues
gitflow-cli issue list --state open --since 2026-06-25

# 对筛选出的 issues 执行分类流程
gitflow-cli issue label 20 --label "type:bug" --label "priority:high" --label "triage:done"
gitflow-cli issue label 21 --label "type:feature" --label "priority:medium" --label "triage:done"
```

## 注意事项

- 分类时应基于 Issue 的标题和描述做出判断，不要过度推测未说明的需求
- 优先级评估应参考项目的整体 roadmap 和当前 sprint 目标
- 如果 Issue 描述不足以判断类型或优先级，可以先标记为 `type:unknown` 或 `priority:medium`，并在报告中建议作者补充信息
- 分类结果不是最终结论，团队成员可以后续调整标签
- 避免在单个 Issue 上花费过多时间分析，保持 triage 效率
- 对于重复的 Issues，建议先标记为 `duplicate` 而非分类，并关联原始 Issue
- 分类报告建议保存在 `docs/` 或项目 wiki 中，方便团队回顾
