# gf Skills 多角色评审与未来发展路线图设计文档

- **日期**：2026-08-10
- **状态**：已批准
- **关联工作流**：`skills-review-roadmap-001`（full 模式四阶段编排）
- **范围**：6 角色 skills 评审 + 4 项目类型适用性分析 + 2026 H2 路线图更新 + Gap 分析

---

## 1. 背景与目标

gf 项目拥有 26 个 skills，覆盖工作流编排、质量门禁、PR/Issue 管理、发布、代码审查等完整工程循环。这些 skills 当前主要在 gf 自身项目（Rust CLI）中使用，但目标是支持更广泛的项目类型和用户群体。

本设计文档回答三个问题：

1. **现有 skills 质量如何** —— 从 6 个角色视角评审 26 个 skills，识别强项、弱项、改进机会。
2. **适用范围有多广** —— 分析 skills 在 Rust CLI / Web 前端 / 后端服务 / Monorepo 四类项目中的适用性。
3. **未来怎么走** —— 基于评审和适用性分析，更新 2026 H2 路线图，规划 skills 演进方向。

**边界条件**（已与产品方确认）：

| 条件 | 取值 |
|------|------|
| 评审方法 | Approach A: Comprehensive Deep Dive |
| 评审角色 | 6 个（产品负责人 / 架构师 / DevOps / 社区运营 / 终端用户 / AI Agent） |
| 项目类型 | 4 个（Rust CLI / Web App / Backend Service / Monorepo） |
| 路线图时间范围 | 2026 H2（7-12月），更新现有路线图 |
| 交付物 | Skills 评审报告 + 多项目适用性分析 + 发展路线图 + Gap 分析 + Issues |

---

## 2. 评审方法论

### 2.1 评审角色与标准

| 角色 | 评审维度 |
|------|----------|
| **产品负责人** | 价值定位清晰度、差异化程度、与路线图对齐度 |
| **架构师** | 职责边界、与其他 skills 的耦合度、可维护性 |
| **DevOps 工程师** | CI/CD 集成度、自动化程度、可观测性 |
| **社区运营** | 文档质量、上手难度、社区贡献友好度 |
| **终端用户** | 易用性、错误提示、学习曲线 |
| **AI Agent** | 触发词匹配准确度、上下文需求、输出可解析性 |

### 2.2 评分标准

**5 分制**：
- 5 = 优秀（无需改进）
- 4 = 好（小幅优化即可）
- 3 = 一般（需要改进）
- 2 = 待改进（明显不足）
- 1 = 差（严重问题）

### 2.3 评审输出格式

每个 skill 的评审结果包含：

```markdown
### gf-<skill-name>

**基本信息**
- 用途：<一句话描述>
- 触发词：<EN> / <ZH>
- 依赖：<列出依赖的其他 skills 或 CLI 命令>

**角色评审表**
| 角色 | 评分 | 评语 | 改进建议 |
|------|------|------|----------|
| 产品负责人 | 4/5 | ... | ... |
| 架构师 | 3/5 | ... | ... |
| DevOps 工程师 | 4/5 | ... | ... |
| 社区运营 | 3/5 | ... | ... |
| 终端用户 | 4/5 | ... | ... |
| AI Agent | 3/5 | ... | ... |

**综合评价**
- 平均分：X.X/5
- 优先级：P0/P1/P2/P3
- 关键改进项：<列出 Top 3>
```

---

## 3. Skills 评审报告结构

```markdown
# gf Skills 多角色评审报告

## 1. 执行摘要
   - 整体评分分布（按层级分组）
   - 关键发现（Top 3 强项 / Top 3 弱项）
   - 优先级建议（P0/P1/P2/P3 各多少个 skills）

## 2. 逐 Skill 评审（26 个）
   ### 编排层（3 个）
   - gf-workflow
   - gf-quality
   - gf-autoreport-bug

   ### 工作流层（12 个）
   - gf-issue / gf-issue-create / gf-issue-review / gf-issue-triage
   - gf-pr / gf-pr-create / gf-pr-review / gf-pr-inline-review / gf-pr-apply-feedback
   - gf-commit / gf-review / gf-precommit

   ### 核心命令层（11 个）
   - gf-auth / gf-repo / gf-repo-onboarding
   - gf-release / gf-release-helper
   - gf-pipeline-analyzer / gf-label-milestone / gf-label-stats
   - gf-security-check / gf-regression / gf-weekly-report

## 3. 横向分析
   - 评分分布图（按层级分组）
   - 角色间共识与分歧（哪些维度评分一致，哪些分歧大）
   - 跨 skill 共性问题（如文档质量普遍偏低）

## 4. 改进建议汇总
   - P0（立即）：评分 < 2 的维度
   - P1（本季度）：评分 < 3 的维度
   - P2（下季度）：评分 = 3 的维度
   - P3（未来）：增强项（评分 ≥ 4）
```

---

## 4. 多项目适用性分析结构

```markdown
# gf Skills 多项目适用性分析

## 1. 项目类型定义
   - **Rust CLI 项目**：当前参考项目，使用 Cargo + Rust 工具链
   - **Web 前端项目**：React / Vue / Next.js，使用 npm/pnpm/yarn
   - **后端服务项目**：Go / Java / Python，使用各自包管理器
   - **Monorepo 项目**：多包 / 多 crate，使用 workspaces / turborepo / nx

## 2. 适用性矩阵
   | Skill | Rust CLI | Web App | Backend | Monorepo | 备注 |
   |-------|----------|---------|---------|----------|------|
   | gf-workflow | ✅ 完全适用 | ⚠️ 需适配 | ⚠️ 需适配 | ✅ 适用 | 质量门禁需替换语言特定工具 |
   | gf-quality | ⚠️ Rust 专用 | ❌ 不适用 | ❌ 不适用 | ⚠️ 部分适用 | 需要多语言质量探针 |
   | gf-pr | ✅ 完全适用 | ✅ 完全适用 | ✅ 完全适用 | ✅ 完全适用 | 语言无关 |
   | ... | ... | ... | ... | ... | ... |

   图例：✅ 完全适用 | ⚠️ 需适配 | ❌ 不适用

## 3. 分析结论
   - **语言无关 skills**（X 个）：pr, issue, workflow, release 等
   - **语言相关 skills**（Y 个）：quality, precommit, commit 等
   - **适配工作量估算**：每个需适配 skill 约 X 人天

## 4. 多项目支持路线图
   - **短期（Q3）**：明确文档说明哪些 skills 适用于哪些项目类型
   - **中期（Q4）**：为 Web/后端项目适配质量门禁（gf-quality 多语言探针）
   - **长期（2027+）**：插件化质量探针，支持任意语言
```

---

## 5. 发展路线图（2026 H2 更新）

### 5.1 现状对齐

现有路线图阶段：
- **阶段一（稳定化）**：✅ 已完成（v1.0 发布，v1.1.0 发布）
- **阶段二（增长）**：进行中（官网 + GEO/SEO + 宣发）
- **阶段三（扩张）**：待启动（MCP 服务器 #102 + 分析报表 #103）

### 5.2 Skills 专项路线图

#### Q3 2026（7-9月）：Skills 质量巩固

**目标**：基于评审结果修复高优先级问题

**里程碑**：
- [ ] 所有 skills 评分 ≥ 3/5
- [ ] AI Agent 触发词优化完成（基于评审反馈）
- [ ] 文档标准化（所有 skills 包含 When to Use / 示例 / 限制）
- [ ] 完成 6 角色评审报告并创建改进 Issues

**关键交付物**：
- Skills 评审报告（26 个 skills × 6 角色）
- 改进 Issues（预计 10-15 个）

#### Q4 2026（10-12月）：多项目适用性扩展

**目标**：支持非 Rust 项目使用核心 skills

**里程碑**：
- [ ] gf-quality 多语言质量探针（Web/后端）
- [ ] 语言无关 skills 明确文档标注
- [ ] 3 个非 Rust 项目 dogfooding 案例
- [ ] 适用性分析报告发布

**关键交付物**：
- 多项目适用性矩阵
- gf-quality 多语言支持
- Dogfooding 案例文档

### 5.3 与现有路线图集成

- **MCP 服务器（#102）**：skills 可通过 MCP 暴露为 Agent 原生接口，减少对文件式 skills 的依赖
- **分析报表（#103）**：增加 skills 使用度分析维度（哪些 skills 最常用、哪些很少用）
- **2.0 预告**：插件化 skills 系统进入 2.0 待办（设计文档 §10"明确不做"清单）

---

## 6. Gap 分析与改进 Issues

### 6.1 Gap 分类框架

- **功能缺口**：缺少的 skills 或能力
- **质量缺口**：现有 skills 的低评分维度
- **适用性缺口**：多项目支持的限制
- **文档缺口**：信息不完整或不清晰
- **集成缺口**：与其他工具/平台的接缝问题

### 6.2 预期 Gaps（10-15 个）

基于初步评估，预期会发现以下类型的 gaps：

**示例**：
- `[GAP-001]` gf-quality 仅支持 Rust，Web/后端项目无法使用（适用性缺口）
- `[GAP-002]` 部分 skills 触发词不够精准，AI Agent 误匹配（质量缺口）
- `[GAP-003]` 缺少 monorepo 场景的 worktree 管理 skill（功能缺口）
- `[GAP-004]` skills 文档缺少 When NOT to Use 部分（文档缺口）
- `[GAP-005]` gf-workflow 与 MCP 服务器的集成路径不清晰（集成缺口）

### 6.3 Issue 创建规范

每个 gap 对应一个 GitHub Issue：

```markdown
标题：<type>(skills): <description>
标签：type:enhancement / type:bug + priority:high/medium/low
关联：Refs #<评审报告 Issue>
退出标准：
- [ ] <明确的完成条件 1>
- [ ] <明确的完成条件 2>
```

### 6.4 优先级排序

- **P0（立即）**：阻塞性问题（评分 < 2）
- **P1（本季度）**：高优先级改进（评分 < 3）
- **P2（下季度）**：中优先级增强（评分 = 3）
- **P3（未来）**：锦上添花（评分 ≥ 4）

---

## 7. 退出标准

本 design doc 的退出标准：

- [ ] Skills 评审报告完成（26 个 skills × 6 角色）
- [ ] 多项目适用性分析完成（4 个项目类型）
- [ ] 2026 H2 路线图更新完成（Q3 + Q4 里程碑）
- [ ] Gap 分析完成，改进 Issues 已创建（预计 10-15 个）
- [ ] 所有文档已提交到 `docs/superpowers/specs/` 和 `docs/specs/`

---

## 8. 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 评审工作量过大（26 skills × 6 roles） | 延期 | 分批次评审：先编排层 → 工作流层 → 核心命令层 |
| AI Agent 角色评审主观性强 | 结果不可靠 | 基于实际使用经验，而非猜测 |
| 多项目适用性分析缺少真实案例 | 结论不准确 | 标记为"初步评估"，后续通过 dogfooding 验证 |
| 路线图与现有路线图冲突 | 执行混乱 | 明确 skills 路线图是现有路线图的补充，不替代 |

---

## 9. 附录

### 9.1 现有 Skills 清单

| 层级 | Skill | 用途 |
|------|-------|------|
| 编排层 | gf-workflow | 四阶段闸门驱动全流程 |
| 编排层 | gf-quality | 6-gate 质量门禁 |
| 编排层 | gf-autoreport-bug | 自动上报 bug |
| 工作流层 | gf-issue | Issue CRUD |
| 工作流层 | gf-issue-create | 创建 Issue |
| 工作流层 | gf-issue-review | 评审 Issue 质量 |
| 工作流层 | gf-issue-triage | 分类 Issue |
| 工作流层 | gf-pr | PR CRUD |
| 工作流层 | gf-pr-create | 创建 PR |
| 工作流层 | gf-pr-review | PR 整体评审 |
| 工作流层 | gf-pr-inline-review | PR 行内评审 |
| 工作流层 | gf-pr-apply-feedback | 应用 PR 反馈 |
| 工作流层 | gf-commit | 查看/评论 commit |
| 工作流层 | gf-review | 提交正式评审结论 |
| 工作流层 | gf-precommit | pre-commit 质量检查 |
| 核心命令层 | gf-auth | 认证管理 |
| 核心命令层 | gf-repo | 仓库操作 |
| 核心命令层 | gf-repo-onboarding | 生成入门指南 |
| 核心命令层 | gf-release | 管理 Git releases |
| 核心命令层 | gf-release-helper | 创建 release + changelog |
| 核心命令层 | gf-pipeline-analyzer | 分析 CI/CD pipeline |
| 核心命令层 | gf-label-milestone | 管理 labels/milestones |
| 核心命令层 | gf-label-stats | label 统计 |
| 核心命令层 | gf-security-check | 安全审计 |
| 核心命令层 | gf-regression | 回归测试 |
| 核心命令层 | gf-weekly-report | 生成周报 |

---

## 10. 参考文档

- [gf 多角色项目评估与产品路线图设计文档](./2026-07-31-product-evaluation-roadmap-design.md)
- [gf-workflow skill](../../skills/gf-workflow/SKILL.md)
- [gf-quality skill](../../skills/gf-quality/SKILL.md)
- Issue #102: MCP 服务器（Agent 原生接口）
- Issue #103: 效率分析报表 + v1.1.0 + 2.0 预告
