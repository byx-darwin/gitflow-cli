# gitflow-cli label-stats 完整标签分类参考

> 本文档为 `gitflow-label-stats` skill 的外部化引用。

## 标签分类体系

| 类别 | 常见标签 | 说明 |
|------|----------|------|
| 类型 | `bug`, `enhancement`, `documentation`, `question` | Issue 的类型 |
| 优先级 | `priority:urgent`, `priority:high`, `priority:medium`, `priority:low` | 优先级 |
| 状态 | `triage:done`, `in-progress`, `blocked` | 工作状态 |
| 平台 | `github`, `gitlab`, `gitcode` | 平台相关 |
| 其他 | `good-first-issue`, `help-wanted`, `auto-reported` | 特殊标记 |

## 优先级判断参考

| 紧急度 | 判断依据 |
|--------|----------|
| 紧急 | 影响生产环境、安全漏洞、阻塞其他工作 |
| 高 | 核心功能缺陷、重要用户需求、近期 milestone 必须完成 |
| 中 | 一般功能需求、体验改进、非阻塞性问题 |
| 低 | 锦上添花、文档小修正、未来考虑的功能 |

**评估参考因素：**

- 是否影响核心用户路径
- 涉及用户数量范围
- 是否有 workaround
- 是否关联即将发布的 milestone
- 是否为安全相关问题

## 同义词归一化表

| 归一化标签 | 同义词 / 变体 |
|------------|---------------|
| `bug` | `type:bug`, `defect` |
| `enhancement` | `type:enhancement`, `feature` |
| `documentation` | `type:docs`, `doc` |
| `priority:high` | `high-priority`, `p1` |
