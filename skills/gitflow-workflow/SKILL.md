---
name: gitflow-workflow
description: |
  Contract-driven four-phase gated pipeline. Use when the user wants a
  mandatory clarify → plan → execute → deliver workflow with JSON contract
  verification between phases. 当用户需要强制执行的四阶段闸门驱动全流程时使用。
---

# gitflow-workflow — 合同驱动四阶段闸门编排器

编排层只指挥；状态靠合同；门控不跳过。

## When to Use

| EN | ZH |
|----|----|
| full workflow | 全流程（默认） |
| clarify → plan → execute → deliver | 需求→计划→执行→交付 |
| contract-driven orchestration | 合同驱动编排 |

**模式自动判定**（进入 Phase 1 前执行）：
- 描述含"修复"/"typo"/"hotfix" → `fast`
- 描述含"新增"/"架构"/"重构" → `full`
- Issue 标签含 `good-first-issue` → `fast`
- 无法确定 → **询问用户**

## 模式对比

### 完整模式
全流程四阶段，必须调用：brainstorming → issue-create/review → writing-plans → subagent-driven-development → TDD → Phase 4

| Phase | Sub-skill | 说明 |
|-------|-----------|------|
| 1 | `superpowers:brainstorming` | 需求澄清 |
| 1 | `gitflow-issue-create` | 创建 Issue |
| 1 | `gitflow-issue-review` | 审查回贴 |
| 2 | `superpowers:writing-plans` | 制定计划 |
| 3 | `superpowers:subagent-driven-development` | 含 TDD + Code Review |
| 4 | `gitflow-pipeline-analyzer → gitflow-issue-triage → gitflow-review` | 交付后检查 |

### 快速模式 - 必须调用的 Skills 清单
Phase 1: gitflow-issue-create(必选), brainstorming(可选)
Phase 2: writing-plans(可选，可跳过)
Phase 3: subagent-driven-development(必选，含 TDD + Code Review)
Phase 4: gitflow-pipeline-analyzer → gitflow-issue-triage → gitflow-review(必选)

## 核心模式：合同（Contract）

每次工作流启动时，创建一个合同文件：

```
.cache/workflows/active/<workflow_id>.json
```

合同格式定义：`skills/gitflow-workflow/contract.schema.json`

### 合同示例

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
        "comment_id": "4921173903"
      }
    },
    "2": {
      "name": "计划制定",
      "status": "complete",
      "started_at": "2026-07-09T03:10:00Z",
      "completed_at": "2026-07-09T03:20:00Z",
      "executor": "claude-code-3.7",
      "evidence": {
        "spec_path": "docs/superpowers/specs/2026-07-09-toon-design.md",
        "user_approved": true
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

## 门控规则

完整门控定义：`skills/gitflow-workflow/gates.md`

**关键原则：** 进入下一 Phase 前，必须校验前一 Phase 的 evidence 完整性。门控失败时，**阻止进入**并返回修复。

| 转入 Phase | 必需 evidence | fast 模式豁免 |
|-----------|--------------|--------------|
| 2 (计划) | `issue_url` + `comment_id` | `comment_id` 可省 |
| 3 (执行) | `spec_path` + `user_approved` | ✅ 可跳过 |
| 4 (交付) | `pr_url` + `tests_passed` | — |

## Phase 执行流程

### Phase 1: 需求澄清

**入口条件：** 无
**出口条件：** 合同 `phases.1.status = complete`

1. 读取 Open Issues → `gitflow-cli issue list --state open`
2. **full 模式：** 调用 `superpowers:brainstorming` 澄清需求
3. 调用 `gitflow-issue-create` 创建 Issue
4. **full 模式：** 调用 `gitflow-issue-review` 审计回贴
5. 写入合同 `phases.1.evidence = { issue_url, comment_id }`
6. 更新 `phases.1.status = complete`

**门控 1→2 校验：**
- `issue_url` 非空 ✅
- `comment_id` 非空（fast 模式可豁免）

### Phase 2: 计划制定

**入口条件：** Gate 1→2 通过
**出口条件：** 合同 `phases.2.status = complete`

1. **full 模式：** 调用 `superpowers:writing-plans` 制定完整计划
2. 用户审批 → `evidence.user_approved = true`
3. 写入合同 `phases.2.evidence = { spec_path, user_approved }`
4. 调用 `gitflow-quality` gate → ALL CHECKS PASSED
   - Build 检查 / Test 检查 / Coverage 检查 / Format 检查 / Static 检查 / Pre-commit 检查
5. 更新 `phases.2.status = complete`

**门控 2→3 校验：**
- `spec_path` 非空
- `user_approved = true`
- fast 模式：跳过本 Phase，直接进入 Phase 3

### Phase 3: 执行

**入口条件：** Gate 2→3 通过（或 fast 模式跳过 Phase 2）
**出口条件：** 合同 `phases.3.status = complete`

1. 创建 worktree
2. 调用 `superpowers:subagent-driven-development`
3. 内含 TDD 循环：RED → GREEN → REFACTOR
4. 调用 `gitflow-pr-create` 创建 PR
5. 写入合同 `phases.3.evidence = { branch, pr_url, tests_passed }`
6. 更新 `phases.3.status = complete`

**门控 3→4 校验：**
- `pr_url` 非空
- `tests_passed = true`

### Phase 4: 交付后检查

**入口条件：** Gate 3→4 通过
**出口条件：** 合同 `phases.4.status = complete`

1. 调用 `gitflow-pipeline-analyzer` → 生成流水线分析报告
2. 调用 `gitflow-issue-triage` → 生成 Issue 分类报告
3. 调用 `gitflow-review` → 生成代码审查报告
4. 写入合同 `phases.4.evidence = { pipeline_ok, review_report_path }`
5. 更新 `phases.4.status = complete`
6. 归档合同 → `.cache/workflows/archive/YYYY-MM/`

## 合同操作 API

### 创建合同

```bash
# 生成 workflow_id（按日期序号）
DATE=$(date -u +%Y-%m-%d)
COUNT=$(ls .cache/workflows/active/ 2>/dev/null | grep "wf-${DATE}" | wc -l)
WORKFLOW_ID="wf-${DATE}-$(printf '%03d' $((COUNT + 1)))"

# 初始化合同
cat > ".cache/workflows/active/${WORKFLOW_ID}.json" << EOF
{
  "version": "1.0",
  "workflow_id": "${WORKFLOW_ID}",
  "title": "<issue_title>",
  "mode": "<full|fast>",
  "created_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "updated_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "current_phase": 1,
  "phases": {
    "1": { "name": "需求澄清", "status": "in_progress", "started_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)", "completed_at": null, "executor": null, "evidence": {} },
    "2": { "name": "计划制定", "status": "pending", "started_at": null, "completed_at": null, "executor": null, "evidence": {} },
    "3": { "name": "执行", "status": "pending", "started_at": null, "completed_at": null, "executor": null, "evidence": {} },
    "4": { "name": "交付", "status": "pending", "started_at": null, "completed_at": null, "executor": null, "evidence": {} }
  }
}
EOF
```

### 更新合同（Phase 完成时）

```bash
# 使用 jq 更新（或手动编辑）；临时文件按 workflow_id 命名，避免并发覆盖
COMPLETED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
jq --arg phase "$PHASE_NUM" \
   --arg status "complete" \
   --arg completed_at "$COMPLETED_AT" \
   --argjson evidence "$EVIDENCE_JSON" \
  '.phases[$phase].status = $status | .phases[$phase].completed_at = $completed_at | .phases[$phase].evidence = $evidence | .updated_at = $completed_at' \
  ".cache/workflows/active/${WORKFLOW_ID}.json" > "${WORKFLOW_ID}.tmp" \
  && mv "${WORKFLOW_ID}.tmp" ".cache/workflows/active/${WORKFLOW_ID}.json"
```

### 推进到下一 Phase（门控通过后）

```bash
# 递增 current_phase，并将下一 Phase 置为 in_progress；同样使用 workflow_id 命名临时文件
jq --arg next "$((PHASE_NUM + 1))" \
   --arg started_at "$COMPLETED_AT" \
  '.current_phase = ($next | tonumber) | .phases[$next].status = "in_progress" | .phases[$next].started_at = $started_at | .updated_at = $started_at' \
  ".cache/workflows/active/${WORKFLOW_ID}.json" > "${WORKFLOW_ID}.tmp" \
  && mv "${WORKFLOW_ID}.tmp" ".cache/workflows/active/${WORKFLOW_ID}.json"
```

### 读取合同

```bash
# 读取当前合同状态
jq '.current_phase, .phases | to_entries[] | select(.value.status == "in_progress") | .key' \
  ".cache/workflows/active/${WORKFLOW_ID}.json"
```

## 强制执行规则

### 禁止行为
- ❌ 跳过 Phase 4（任何模式）
- ❌ 快速模式禁止跳过 TDD
- ❌ 快速模式禁止跳过 Code Review
- ❌ 合并多个 Phase 为一步
- ❌ 门控未通过时进入下一 Phase
- ❌ 用户要求跳步时妥协（Scenario C 防护）

### Scenario C 防护
当用户说"需求很清晰了，直接写代码吧"时：
1. 识别这是 **Phase 2→3 的跳步请求**
2. 检查合同：`phases.2.evidence.spec_path` 是否存在
3. **不存在时拒绝跳步**：返回 Phase 2 制定计划
4. **fast 模式例外**：若已判定 fast 模式，允许跳过 Phase 2

## 多流程并发

多个 workflow_id 独立存储，互不干扰：

```
.cache/workflows/
├── active/
│   ├── wf-2026-07-09-001.json  ← feat: TOON
│   └── wf-2026-07-09-002.json  ← fix: pr merge
└── archive/
    └── 2026-07/
        └── wf-2026-07-08-001.json
```

**隔离原则：** 每个 workflow 使用独立的 worktree、branch、合同文件。

## 生命周期管理

| 状态 | 位置 | 保留期 | 清理方式 |
|------|------|--------|---------|
| active | `.cache/workflows/active/` | 运行中 | 完成后移 archive |
| archive | `.cache/workflows/archive/YYYY-MM/` | 90 天 | `gitflow workflow cleanup --older-than 90` |

## CLI 集成

```bash
# 列出 active workflows
gitflow workflow list

# 查看合同详情
gitflow workflow status <workflow_id>

# 归档已完成的 workflow
gitflow workflow archive <workflow_id>

# 清理过期归档
gitflow workflow cleanup --older-than 90
```

## Error Handling

| Error | Recovery |
|-------|----------|
| 合同文件不存在 | 创建新合同（从 Phase 1 开始） |
| 合同 JSON 损坏 | 从备份恢复或重建 |
| 门控校验失败 | 返回当前 Phase 补齐 evidence |
| worktree 泄露 | `git worktree remove` + `branch -d` |
| auth 过期 | `gitflow-cli auth status` 检查，重登 |

## Common Mistakes

❌ 跳过门控校验 · ❌ 内联 sub skill · ❌ 合同未更新就进下一 Phase · ❌ worktree 未清 · ❌ 用户跳步请求时让步
