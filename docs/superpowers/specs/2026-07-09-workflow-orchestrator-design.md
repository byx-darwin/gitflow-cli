# 四阶段编排器设计文档

**日期**: 2026-07-09
**状态**: 待实现

## 1. 背景

### 当前问题

现有 `skills/gitflow-workflow/SKILL.md` 是一个指导性 prompt，告诉 AI "应该按四阶段走"。但 AI 可以遵守也可以跳过，缺乏强制力。在实际执行中出现了以下问题：

- Phase 1（需求澄清）被跳过，直接进入设计
- issue-create 和 issue-review 被延后
- Phase 之间边界模糊

### 设计目标

1. **强制流程** - 不跳步、不合并 Phase
2. **多 Agent 兼容** - Claude Code / Cursor / CI 都能使用
3. **多流程并发** - 同时运行多个工作流
4. **可审计** - 每步有时间戳和执行者

## 2. 架构

### 2.1 核心概念：合同（Contract）

合同是 Phase 之间传递的结构化数据，也是多 Agent 兼容的关键接口。

```json
{
  "version": "1.0",
  "workflow_id": "wf-2026-07-09-001",
  "title": "feat: TOON 输出格式",
  "mode": "full",
  "created_at": "2026-07-09T02:59:32Z",
  "updated_at": "2026-07-09T03:30:00Z",
  "current_phase": 3,
  "phases": {
    "1": {
      "name": "需求澄清",
      "status": "complete",
      "started_at": "2026-07-09T02:59:32Z",
      "completed_at": "2026-07-09T03:10:00Z",
      "executor": "claude-code-3.7",
      "evidence": {
        "issue_url": "https://github.com/.../issues/74",
        "comment_id": 4921173903
      }
    },
    "2": {
      "name": "计划制定",
      "status": "complete",
      "started_at": "2026-07-09T03:10:00Z",
      "completed_at": "2026-07-09T03:20:00Z",
      "executor": "claude-code-3.7",
      "evidence": {
        "spec_path": "docs/superpowers/specs/2026-07-09-toon-output-format-design.md"
      }
    },
    "3": {
      "name": "执行",
      "status": "in_progress",
      "started_at": "2026-07-09T03:20:00Z",
      "completed_at": null,
      "executor": "subagent-task-3",
      "evidence": {}
    },
    "4": {
      "name": "交付",
      "status": "pending",
      "started_at": null,
      "completed_at": null,
      "executor": null,
      "evidence": {}
    }
  }
}
```

### 2.2 字段说明

#### 顶层字段

| 字段 | 类型 | 说明 |
|------|------|------|
| `version` | string | 合同版本（兼容未来升级） |
| `workflow_id` | string | 唯一标识，格式：`wf-YYYY-MM-DD-NNN` |
| `title` | string | 任务标题（从 Issue 标题同步） |
| `mode` | string | `full` 或 `fast` |
| `created_at` | string (ISO 8601) | 创建时间 |
| `updated_at` | string (ISO 8601) | 最后更新时间 |
| `current_phase` | number | 当前 Phase 编号 (1-4) |

#### Phase 字段

| 字段 | 类型 | 说明 |
|------|------|------|
| `name` | string | Phase 名称 |
| `status` | string | `pending` / `in_progress` / `complete` / `skipped` |
| `started_at` | string? | 开始时间 |
| `completed_at` | string? | 完成时间 |
| `executor` | string? | 执行者标识（agent 名称或 subagent ID） |
| `evidence` | object | Phase 产出的证据 |

#### 各 Phase 的 evidence

| Phase | evidence 字段 |
|-------|--------------|
| 1 | `issue_url`, `comment_id` (审查评论 ID) |
| 2 | `spec_path`, `user_approved` (bool) |
| 3 | `branch`, `pr_url`, `tests_passed` (bool) |
| 4 | `pipeline_ok` (bool), `review_report_path` |

### 2.3 门控合同（Gate Contract）

进入下一 Phase 前必须满足的条件：

```json
{
  "phase_1_to_2": {
    "requires": ["phases.1.status == complete"],
    "evidence_required": ["issue_url", "comment_id"]
  },
  "phase_2_to_3": {
    "requires": ["phases.2.status == complete", "user_approved == true"],
    "evidence_required": ["spec_path"]
  },
  "phase_3_to_4": {
    "requires": ["phases.3.status == complete", "tests_passed == true"],
    "evidence_required": ["pr_url"]
  }
}
```

## 3. 多 Agent 兼容

### 3.1 兼容策略

不同 Agent 共享合同格式，各自实现调度逻辑：

| Agent | 角色 | 合同使用方式 |
|-------|------|-------------|
| **Claude Code** | 完整编排器 | 用 SKILL.md 编排，读写合同 |
| **Cursor** | 客户端 agent | 读合同判断当前 Phase，按 `.cursorrules` 执行 |
| **CI / Hook** | 门控校验 | 读合同校验门控，拒绝不合规操作 |
| **CLI** | 辅助工具 | `gitflow-cli workflow status` 显示状态 |

### 3.2 接力示例

```
Claude Code 创建 Issue → 合同写入 phase 1 完成
Cursor 读取合同 → 检测 phase 1 已完成，开始 phase 2
子代理执行实现 → 合同更新 phase 3 状态
CI hook 校验合同 → 门控通过，允许合并
```

## 4. 多流程并发

### 4.1 文件结构

```
.cache/workflows/
├── index.json              ← 索引文件
├── active/
│   ├── wf-2026-07-09-001.json
│   └── wf-2026-07-09-002.json
└── archive/
    └── 2026-07/
        └── wf-2026-07-08-001.json
```

### 4.2 索引文件

```json
{
  "workflows": [
    {
      "id": "wf-2026-07-09-001",
      "title": "feat: TOON 输出格式",
      "phase": 3,
      "status": "in_progress",
      "updated_at": "2026-07-09T03:20:00Z"
    },
    {
      "id": "wf-2026-07-09-002",
      "title": "fix: pr merge --yes",
      "phase": 4,
      "status": "complete",
      "updated_at": "2026-07-09T03:30:00Z"
    }
  ]
}
```

## 5. 模式判定

编排器在启动时自动判断模式：

```markdown
判定规则：
- 用户描述中包含"修复"、"typo"、"hotfix" → fast
- 用户描述中包含"新增"、"架构"、"重构" → full
- Issue 标签包含 `good-first-issue` → fast
- 无法确定 → 询问用户

快速模式跳过：
- Phase 1: brainstorming, issue-create, issue-review
- Phase 2: writing-plans
- 保留 Phase 3: subagent-driven-development
- 保留 Phase 4: 交付后检查
```

## 6. 生命周期管理

### 6.1 文件生命周期

| 位置 | 生命周期 | 删除方式 |
|------|---------|---------|
| `active/` | 流程运行中 | 完成后移到 archive |
| `archive/` | 完成后 90 天 | `make workflow-cleanup` 或自动 |
| git history | 永久 | 不删 |

### 6.2 清理命令

```bash
# 列出所有工作流
gitflow-cli workflow list

# 查看工作流详情
gitflow-cli workflow status <workflow_id>

# 清理过期工作流（默认 90 天）
gitflow-cli workflow cleanup --older-than 90

# 归档已完成工作流
gitflow-cli workflow archive <workflow_id>
```

## 7. 实现计划

| 步骤 | 文件 | 改动 |
|------|------|------|
| 1 | `skills/gitflow-workflow/SKILL.md` | 重写为四阶段编排器 |
| 2 | `skills/gitflow-workflow/contract.schema.json` | 合同 JSON Schema |
| 3 | `skills/gitflow-workflow/gates.md` | 门控规则定义 |
| 4 | `apps/cli/src/commands/workflow.rs` | CLI 子命令（可选） |

## 8. Skill 测试方案（writing-skills 流程）

本设计文档包含实现方案，但 **skill 文件本身的修改必须遵循 `superpowers:writing-skills` 的 TDD 流程**。

### 8.1 RED 阶段：压力场景

在修改 `skills/gitflow-workflow/SKILL.md` 之前，先定义 3 个 pressure scenario 作为 failing test。

#### Scenario A: 长上下文压力

```
前提: 对话已进行 30+ 轮，中间做过多个不同任务
触发: 用户说"帮我在 auth 模块加个缓存功能"
预期失败: AI 跳过 Phase 1/2，直接开始写代码
失败判定: AI 未调用 issue-create 或 writing-plans
```

#### Scenario B: 多任务并发压力

```
前提: 用户同时要求做 2 个功能（一个简单修复，一个复杂重构）
触发: "修一下 login 的那个 typo，顺便把 OAuth 流程重构成独立服务"
预期失败: AI 混淆两个 workflow 的合同，简单任务走了 full 流程，复杂任务漏了 Phase 2
失败判定: 两个 workflow 状态互相污染
```

#### Scenario C: 跨 Phase 跳步压力

```
前提: 用户要求跳过计划阶段
触发: "需求很清晰了，直接写代码吧"
预期失败: AI 服从用户，跳过 Phase 2 进入 Phase 3
失败判定: AI 未坚持门控规则（合同规定必须有 spec_path 才能进入 Phase 3）
```

### 8.2 RED 阶段执行步骤

1. 准备 pressure scenario 脚本
2. 用当前 (未修改的) workflow skill 执行一次
3. 记录 verbatim 失败模式：
   - AI 在哪里跳过了 Phase？
   - AI 用了什么合理化借口（rationalization）？
4. 归档到 `docs/superpowers/workflow-skill-red-results.md`

### 8.3 GREEN 阶段：Skill 编写

根据 RED 阶段发现的真实失败点编写 skill：
- 只针对已发现的问题写规则
- 不添加"可能用得上"的规则
- 每条规则对应至少一个失败场景

### 8.4 REFACTOR 阶段：堵漏洞

1. 用新 skill 重跑 3 个 pressure scenario
2. 如果任一步失败：
   - 记录新出现的 rationalization
   - 在 skill 中增加对应的 counter-rule
   - 重新测试
3. 直到 5 次连续无失败

- [ ] `skills/gitflow-workflow/SKILL.md` 包含完整的四阶段编排逻辑
- [ ] 合同 Schema 支持所有必需字段
- [ ] 模式自动判定（full / fast）
- [ ] 多流程并发（多个合同文件独立演进）
- [ ] 门控校验（跳过 Phase 时被阻止）
- [ ] CLI 命令 `gitflow-cli workflow status` 可读合同
- [ ] 生命周期管理（archive + cleanup）
- [ ] 与现有 `subagent-driven-development` 兼容

## 9. 风险与缓解

| 风险 | 缓解 |
|------|------|
| 合同文件损坏 | JSON Schema 校验 + 备份机制 |
| 并发写冲突 | 文件锁或每个 workflow 独立文件 |
| Agent 不遵循门控 | CI hook 强制校验 |
| archive 累积 | 自动清理 + git gc |
