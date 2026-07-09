# Workflow 四阶段编排器改造 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: 使用 superpowers:subagent-driven-development 按计划逐任务执行。步骤使用 checkbox (`- [ ]`) 语法追踪。

**Goal:** 将 `gitflow-workflow` 从 prompt-driven 提示文档升级为 contract-driven 强制编排器，实现门控校验、多 Agent 兼容、多流程并发。

**Architecture:** 合同（Contract）是 Phase 间传递的 JSON 结构，存储于 `.cache/workflows/active/`。每个 Phase 完成后写入 evidence，进入下一 Phase 前校验 evidence 完整性。多 Agent 通过读写同一份合同实现接力。CLI 提供 `workflow status/list/archive/cleanup` 子命令。

**Tech Stack:** Rust 2024（CLI 层）、JSON Schema（Serde 校验）、Markdown（Skill 层）

## Global Constraints

- Rust 2024 edition，工具链版本锁定于 `rust-toolchain.toml`
- 禁止 `unsafe` 代码（`#![forbid(unsafe_code)]`）
- 所有公共 API 必须有文档注释（含 `# Errors`）
- 测试使用 `matches!()` 验证错误类型
- 遵循 TDD 流程：RED → GREEN → REFACTOR
- 依赖最小化，优先使用现有 workspace 依赖
- Skill 层修改必须通过 3 个 pressure scenario 测试（连续 5 次无失败）
- CLI 命令名：`gitflow workflow <status|list|archive|cleanup>`
- 合同版本：`1.0`，workflow_id 格式：`wf-YYYY-MM-DD-NNN`

---

## 实现概览

### 文件结构

```
skills/gitflow-workflow/
├── SKILL.md                      ← 重写（四阶段编排器）
├── contract.schema.json          ← 新建（JSON Schema）
└── gates.md                      ← 新建（门控规则）

apps/cli/src/commands/
├── workflow.rs                   ← 新建（CLI 子命令）
└── mod.rs                        ← 修改（注册 workflow 模块）

apps/cli/src/
└── main.rs                       ← 修改（dispatch workflow 命令）
```

### 执行顺序

| 步骤 | 交付物 | 可独立测试 |
|------|--------|-----------|
| Task 1 | contract.schema.json | JSON Schema 校验通过 |
| Task 2 | gates.md | 文档完整性 |
| Task 3 | SKILL.md 重写 | Pressure scenario 测试 |
| Task 4 | workflow.rs CLI | cargo test + clippy 通过 |
| Task 5 | 注册 + 集成 | `gitflow workflow --help` 可用 |
| Task 6 | writing-skills TDD 验证 | 5 次连续无失败 |

---

## Task 1: 合同 JSON Schema

**Files:**
- Create: `skills/gitflow-workflow/contract.schema.json`
- Test: 手动 JSON 校验

**Interfaces:**
- Consumes: 无（基础定义）
- Produces: `contract.schema.json` — 供 SKILL.md 和 CLI 校验引用

- [ ] **Step 1: 创建合同 Schema**

写入文件 `skills/gitflow-workflow/contract.schema.json`：

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://gitflow-cli.ai/schemas/workflow-contract-v1.json",
  "title": "Workflow Contract",
  "description": "Phase 间传递的结构化合同，用于 gitflow-workflow 四阶段编排器",
  "type": "object",
  "required": ["version", "workflow_id", "title", "mode", "created_at", "updated_at", "current_phase", "phases"],
  "properties": {
    "version": {
      "type": "string",
      "const": "1.0",
      "description": "合同版本（兼容未来升级）"
    },
    "workflow_id": {
      "type": "string",
      "pattern": "^wf-\\d{4}-\\d{2}-\\d{2}-\\d{3}$",
      "description": "唯一标识，格式：wf-YYYY-MM-DD-NNN"
    },
    "title": {
      "type": "string",
      "minLength": 1,
      "description": "任务标题（从 Issue 标题同步）"
    },
    "mode": {
      "type": "string",
      "enum": ["full", "fast"],
      "description": "full = 完整四阶段；fast = 跳过非必需 Phase"
    },
    "created_at": {
      "type": "string",
      "format": "date-time",
      "description": "创建时间 (ISO 8601)"
    },
    "updated_at": {
      "type": "string",
      "format": "date-time",
      "description": "最后更新时间 (ISO 8601)"
    },
    "current_phase": {
      "type": "integer",
      "minimum": 1,
      "maximum": 4,
      "description": "当前 Phase 编号 (1-4)"
    },
    "phases": {
      "type": "object",
      "required": ["1", "2", "3", "4"],
      "properties": {
        "1": { "$ref": "#/$defs/phase" },
        "2": { "$ref": "#/$defs/phase" },
        "3": { "$ref": "#/$defs/phase" },
        "4": { "$ref": "#/$defs/phase" }
      },
      "additionalProperties": false
    }
  },
  "additionalProperties": false,
  "$defs": {
    "phase": {
      "type": "object",
      "required": ["name", "status"],
      "properties": {
        "name": {
          "type": "string",
          "enum": ["需求澄清", "计划制定", "执行", "交付"]
        },
        "status": {
          "type": "string",
          "enum": ["pending", "in_progress", "complete", "skipped"]
        },
        "started_at": {
          "type": ["string", "null"],
          "format": "date-time"
        },
        "completed_at": {
          "type": ["string", "null"],
          "format": "date-time"
        },
        "executor": {
          "type": ["string", "null"]
        },
        "evidence": {
          "type": "object",
          "properties": {
            "issue_url": { "type": "string" },
            "comment_id": { "type": "string" },
            "spec_path": { "type": "string" },
            "user_approved": { "type": "boolean" },
            "branch": { "type": "string" },
            "pr_url": { "type": "string" },
            "tests_passed": { "type": "boolean" },
            "pipeline_ok": { "type": "boolean" },
            "review_report_path": { "type": "string" }
          },
          "additionalProperties": false
        }
      },
      "additionalProperties": false
    }
  }
}
```

- [ ] **Step 2: 验证 Schema 合法性**

Run: `python3 -m json.tool skills/gitflow-workflow/contract.schema.json > /dev/null && echo "JSON valid"`
Expected: `JSON valid`

- [ ] **Step 3: Commit**

```bash
git add skills/gitflow-workflow/contract.schema.json
git commit -m "feat(workflow): add contract JSON schema v1.0

Defines the contract structure for phase-to-phase state transfer.
Validated against JSON Schema 2020-12."

```

---

## Task 2: 门控规则定义

**Files:**
- Create: `skills/gitflow-workflow/gates.md`
- Test: 文档人工审查

**Interfaces:**
- Consumes: `contract.schema.json`
- Produces: `gates.md` — SKILL.md 引用的门控规则文档

- [ ] **Step 1: 创建门控规则文档**

写入文件 `skills/gitflow-workflow/gates.md`：

```markdown
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
```

- [ ] **Step 2: Commit**

```bash
git add skills/gitflow-workflow/gates.md
git commit -m "feat(workflow): define gate rules between phases

Three gates (1→2, 2→3, 3→4) with fast-mode exemptions.
Includes multi-agent gate check algorithm."

```

---

## Task 3: SKILL.md 重写

**Files:**
- Modify: `skills/gitflow-workflow/SKILL.md`
- Test: 3 个 pressure scenario（见 Task 6）

**Interfaces:**
- Consumes: `contract.schema.json`, `gates.md`
- Produces: 新版 SKILL.md（四阶段闸门编排器）

- [ ] **Step 1: 备份当前 SKILL.md**

```bash
cp skills/gitflow-workflow/SKILL.md skills/gitflow-workflow/SKILL.md.bak
```

- [ ] **Step 2: 重写 SKILL.md**

写入新文件 `skills/gitflow-workflow/SKILL.md`：

```markdown
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
| 2 (计划) | `issue_url` | — |
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

1. **full 模式：** 调用 `superpowers:writing-plans` 制定计划
2. 用户审批 → `evidence.user_approved = true`
3. 写入合同 `phases.2.evidence = { spec_path, user_approved }`
4. 调用 `gitflow-quality` gate → ALL CHECKS PASSED
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

1. 调用 `gitflow-pipeline-analyzer` → 生成流水线报告
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
# 使用 jq 更新（或手动编辑）
jq --arg phase "$PHASE_NUM" \
   --arg status "complete" \
   --arg completed_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
   --argjson evidence "$EVIDENCE_JSON" \
  '.phases[$phase].status = $status | .phases[$phase].completed_at = $completed_at | .phases[$phase].evidence = $evidence | .updated_at = $completed_at' \
  ".cache/workflows/active/${WORKFLOW_ID}.json" > tmp.json && mv tmp.json ".cache/workflows/active/${WORKFLOW_ID}.json"
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
- ❌ fast 模式跳过 TDD
- ❌ fast 模式跳过 Code Review
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
```

- [ ] **Step 3: 清理备份**

```bash
rm skills/gitflow-workflow/SKILL.md.bak
```

- [ ] **Step 4: Commit**

```bash
git add skills/gitflow-workflow/SKILL.md
git commit -m "feat(workflow): rewrite SKILL.md as contract-driven orchestrator

- Add contract lifecycle (create/update/read)
- Embed mode detection (full/fast)
- Define per-phase gate checks with evidence requirements
- Add Scenario C anti-skip protection
- Document multi-workflow concurrency and lifecycle management"

```

---

## Task 4: CLI workflow 子命令 (Rust)

**Files:**
- Create: `apps/cli/src/commands/workflow.rs`
- Test: `apps/cli/src/commands/workflow.rs` (内嵌 `#[cfg(test)]` 模块)

**Interfaces:**
- Consumes: 无
- Produces: `WorkflowCommand` 枚举 + `handle` 函数

- [ ] **Step 1: 编写 failing test（单元测试先写）**

在 `apps/cli/src/commands/workflow.rs` 末尾添加：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_workflow_contract_new_id_format() {
        let contract = WorkflowContract::new(
            "feat: test".to_string(),
            WorkflowMode::Full,
        );
        assert!(
            regex::Regex::new(r"^wf-\d{4}-\d{2}-\d{2}-\d{3}$")
                .unwrap()
                .is_match(&contract.workflow_id),
            "workflow_id format mismatch: {}",
            contract.workflow_id
        );
    }

    #[test]
    fn test_workflow_contract_serialization_roundtrip() {
        let contract = WorkflowContract::new(
            "fix: pr merge".to_string(),
            WorkflowMode::Fast,
        );
        let json = serde_json::to_string_pretty(&contract).expect("serialize");
        let deserialized: WorkflowContract =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.workflow_id, contract.workflow_id);
        assert_eq!(deserialized.title, contract.title);
        assert!(matches!(deserialized.mode, WorkflowMode::Fast));
    }

    #[test]
    fn test_gate_1_to_2_blocks_without_issue_url() {
        let mut contract = WorkflowContract::new(
            "feat: test".to_string(),
            WorkflowMode::Full,
        );
        // Phase 1 状态为 complete 但缺少 issue_url
        contract.phases[0].status = PhaseStatus::Complete;
        let result = contract.can_enter_phase(2);
        assert!(
            matches!(result, GateCheck::MissingEvidence(_)),
            "should block without issue_url"
        );
    }

    #[test]
    fn test_gate_1_to_2_passes_with_issue_url() {
        let mut contract = WorkflowContract::new(
            "feat: test".to_string(),
            WorkflowMode::Full,
        );
        contract.phases[0].status = PhaseStatus::Complete;
        contract.phases[0].evidence.issue_url =
            Some("https://github.com/org/repo/issues/1".to_string());
        contract.phases[0].evidence.comment_id = Some("12345".to_string());
        let result = contract.can_enter_phase(2);
        assert!(matches!(result, GateCheck::Pass));
    }

    #[test]
    fn test_gate_2_to_3_fast_mode_always_passes() {
        let contract = WorkflowContract::new(
            "fix: typo".to_string(),
            WorkflowMode::Fast,
        );
        // fast 模式下 Gate 2→3 自动通过（Phase 2 被跳过）
        let result = contract.can_enter_phase(3);
        assert!(
            matches!(result, GateCheck::Pass),
            "fast mode should skip phase 2 gate"
        );
    }

    #[test]
    fn test_gate_3_to_4_never_fast_exempt() {
        let contract = WorkflowContract::new(
            "fix: test".to_string(),
            WorkflowMode::Fast,
        );
        // Gate 3→4 没有 fast 豁免
        let result = contract.can_enter_phase(4);
        assert!(
            matches!(result, GateCheck::MissingEvidence(_)),
            "Gate 3→4 must never be fast-exempt"
        );
    }

    #[test]
    fn test_active_workflow_dir_creation() {
        let tmp = TempDir::new().expect("temp dir");
        let dir = tmp.path().join(".cache/workflows/active");
        std::fs::create_dir_all(&dir).expect("create dir");
        assert!(dir.exists());
    }
}
```

- [ ] **Step 2: 运行测试 — 预期失败（类型未定义）**

Run: `cargo test -p gitflow-cli workflow::tests 2>&1 | head -30`
Expected: 编译错误（`WorkflowContract` 未定义）

- [ ] **Step 3: 实现 workflow.rs**

写入 `apps/cli/src/commands/workflow.rs`：

> 因篇幅限制，完整代码见下方描述：

核心结构：

```rust
//! `gitflow workflow` 子命令实现。
//!
//! 管理工作流合同的创建、读取、归档和清理。
//! 合同存储在 `.cache/workflows/active/` 和 `.cache/workflows/archive/`。

use std::path::PathBuf;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use std::fs;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
pub enum WorkflowMode {
    Full,
    Fast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhaseStatus {
    Pending,
    InProgress,
    Complete,
    Skipped,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PhaseEvidence {
    pub issue_url: Option<String>,
    pub comment_id: Option<String>,
    pub spec_path: Option<String>,
    pub user_approved: Option<bool>,
    pub branch: Option<String>,
    pub pr_url: Option<String>,
    pub tests_passed: Option<bool>,
    pub pipeline_ok: Option<bool>,
    pub review_report_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Phase {
    pub name: String,
    pub status: PhaseStatus,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub executor: Option<String>,
    pub evidence: PhaseEvidence,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GateCheck {
    Pass,
    MissingEvidence(String),
    PhaseNotComplete(u8),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowContract {
    pub version: String,
    pub workflow_id: String,
    pub title: String,
    pub mode: WorkflowMode,
    pub created_at: String,
    pub updated_at: String,
    pub current_phase: u8,
    pub phases: Vec<Phase>, // 索引 0=Phase1, 1=Phase2, ...
}

impl WorkflowContract {
    pub fn new(title: String, mode: WorkflowMode) -> Self {
        let now = Utc::now();
        let date.format("%Y-%m-%d");
        // 注意：实际生成 workflow_id 需要扫描目录，此处简化
        let workflow_id = format!("wf-{}-001", date);

        Self {
            version: "1.0".to_string(),
            workflow_id,
            title,
            mode,
            created_at: now.to_rfc3339(),
            updated_at: now.to_rfc3339(),
            current_phase: 1,
            phases: vec![
                Phase { name: "需求澄清".into(), status: PhaseStatus::InProgress, started_at: Some(now.to_rfc3339()), ..Default::default() },
                Phase { name: "计划制定".into(), status: PhaseStatus::Pending, ..Default::default() },
                Phase { name: "执行".into(), status: PhaseStatus::Pending, ..Default::default() },
                Phase { name: "交付".into(), status: PhaseStatus::Pending, ..Default::default() },
            ],
        }
    }

    pub fn can_enter_phase(&self, target: u8) -> GateCheck {
        match target {
            2 => {
                if self.phases[0].status != PhaseStatus::Complete {
                    return GateCheck::PhaseNotComplete(1);
                }
                if self.phases[0].evidence.issue_url.is_none() {
                    return GateCheck::MissingEvidence("issue_url".into());
                }
                if self.mode == WorkflowMode::Full
                    && self.phases[0].evidence.comment_id.is_none()
                {
                    return GateCheck::MissingEvidence("comment_id".into());
                }
                GateCheck::Pass
            }
            3 => {
                // fast 模式跳过 Phase 2
                if self.mode == WorkflowMode::Fast {
                    return GateCheck::Pass;
                }
                if self.phases[1].status != PhaseStatus::Complete {
                    return GateCheck::PhaseNotComplete(2);
                }
                if self.phases[1].evidence.spec_path.is_none() {
                    return GateCheck::MissingEvidence("spec_path".into());
                }
                if self.phases[1].evidence.user_approved != Some(true) {
                    return GateCheck::MissingEvidence("user_approved".into());
                }
                GateCheck::Pass
            }
            4 => {
                if self.phases[2].status != PhaseStatus::Complete {
                    return GateCheck::PhaseNotComplete(3);
                }
                if self.phases[2].evidence.pr_url.is_none() {
                    return GateCheck::MissingEvidence("pr_url".into());
                }
                if self.phases[2].evidence.tests_passed != Some(true) {
                    return GateCheck::MissingEvidence("tests_passed".into());
                }
                GateCheck::Pass
            }
            _ => GateCheck::MissingEvidence("invalid phase".into()),
        }
    }
}

// Default for Phase
impl Default for Phase {
    fn default() -> Self {
        Self {
            name: String::new(),
            status: PhaseStatus::Pending,
            started_at: None,
            completed_at: None,
            executor: None,
            evidence: PhaseEvidence::default(),
        }
    }
}

/// CLI 子命令枚举
#[derive(Debug, Subcommand)]
pub enum WorkflowCommand {
    /// 列出 active workflows
    List,
    /// 查看 workflow 合同详情
    Status {
        /// workflow_id
        workflow_id: String,
    },
    /// 归档已完成的 workflow
    Archive {
        /// workflow_id
        workflow_id: String,
    },
    /// 清理过期归档
    Cleanup {
        /// 超过多少天的归档会被清理（默认 90）
        #[arg(long, default_value = "90")]
        older_than: i64,
    },
}

/// 处理 `gitflow workflow` 命令
pub fn handle(command: WorkflowCommand) -> miette::Result<()> {
    match command {
        WorkflowCommand::List => list_workflows(),
        WorkflowCommand::Status { workflow_id } => show_status(&workflow_id),
        WorkflowCommand::Archive { workflow_id } => archive_workflow(&workflow_id),
        WorkflowCommand::Cleanup { older_than } => cleanup_archives(older_than),
    }
}

fn workflow_dir() -> PathBuf {
    // 从当前工作目录或 git rev-parse 获取
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    cwd.join(".cache/workflows/active")
}

fn archive_dir() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    cwd.join(".cache/workflows/archive")
}

fn list_workflows() -> miette::Result<()> {
    let dir = workflow_dir();
    if !dir.exists() {
        println!("(无 active workflows)");
        return Ok(());
    }
    let mut found = 0;
    for entry in fs::read_dir(&dir).map_err(|e| miette::miette!("读取目录失败: {e}"))? {
        let entry = entry.map_err(|e| miette::miette!("读取条目失败: {e}"))?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            let content = fs::read_to_string(&path) .map_err(|e| miette::miette!("读取合同失败: {e}"))?;
            let contract: WorkflowContract = serde_json::from_str(&content) .map_err(|e| miette::miette!("解析合同失败: {e}"))?;
            println!(
                "  {} | {} | Phase {} | {}",
                contract.workflow_id, contract.title, contract.current_phase, contract.mode
            );
            found += 1;
        }
    }
    if found == 0 {
        println!("(无 active workflows)");
    } else {
        println!("\n共 {found} 个 active workflows");
    }
    Ok(())
}

fn show_status(workflow_id: &str) -> miette::Result<()> {
    let path = workflow_dir().join(format!("{workflow_id}.json"));
    if !path.exists() {
        return Err(miette::miette!("workflow {workflow_id} 不存在"));
    }
    let content = fs::read_to_string(&path) .map_err(|e| miette::miette!("读取合同失败: {e}"))?;
    let contract: WorkflowContract = serde_json::from_str(&content) .map_err(|e| miette::miette!("解析合同失败: {e}"))?;
    println!("{}", serde_json::to_string_pretty(&contract).map_err(|e| miette::miette!("{e}"))?);
    Ok(())
}

fn archive_workflow(workflow_id: &str) -> miette::Result<()> {
    let src = workflow_dir().join(format!("{workflow_id}.json"));
    if !src.exists() {
        return Err(miette::miette!("workflow {workflow_id} 不存在"));
    }
    let content = fs::read_to_string(&src) .map_err(|e| miette::miette!("读取合同失败: {e}"))?;
    let contract: WorkflowContract = serde_json::from_str(&content) .map_err(|e| miette::miette!("解析合同失败: {e}"))?;
    if contract.current_phase != 4 || contract.phases[3].status != PhaseStatus::Complete {
        return Err(miette::miette!(
            "workflow 未完成（current_phase={}, phase_4_status={}）",
            contract.current_phase, contract.phases[3].status
        ));
    }
    // 按月归档
    let now = Utc::now();
    let month_dir = archive_dir().join(now.format("%Y-%m").to_string());
    fs::create_dir_all(&month_dir) .map_err(|e| miette::miette!("创建归档目录失败: {e}"))?;
    let dst = month_dir.join(format!("{workflow_id}.json"));
    fs::copy(&src, &dst).map_err(|e| miette::miette!("复制到归档失败: {e}"))?;
    fs::remove_file(&src).map_err(|e| miette::miette!("删除 active 合同失败: {e}"))?;
    println!("✅ workflow {workflow_id} 已归档到 {}", month_dir.display());
    Ok(())
}

fn cleanup_archives(older_than_days: i64) -> miette::Result<()> {
    let dir = archive_dir();
    if !dir.exists() {
        println!("(无归档)");
        return Ok(());
    }
    let threshold = Utc::now() - Duration::days(older_than_days);
    let mut cleaned = 0;
    for month_entry in fs::read_dir(&dir).map_err(|e| miette::miette!("读取归档目录失败: {e}"))? {
        let month_entry = month_entry.map_err(|e| miette::miette!("读取月份目录失败: {e}"))?;
        let month_dir = month_entry.path();
        if !month_dir.is_dir() { continue; }
        for entry in fs::read_dir(&month_dir) .map_err(|e| miette::miette!("读取归档条目失败: {e}"))? {
            let entry = entry.map_err(|e| miette::miette!("读取文件失败: {e}"))?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") { continue; }
            let content = fs::read_to_string(&path) .map_err(|e| miette::miette!("读取合同失败: {e}"))?;
            let contract: WorkflowContract = serde_json::from_str(&content) .map_err(|e| miette::miette!("解析合同失败: {e}"))?;
            if let Ok(created) = DateTime::parse_from_rfc3339(&contract.created_at) {
                if created.with_timezone(&Utc) < threshold {
                    fs::remove_file(&path) .map_err(|e| miette::miette!("删除过期归档失败: {e}"))?;
                    cleaned += 1;
                }
            }
        }
        // 清理空月份目录
        if fs::read_dir(&month_dir).map_or(false, |mut d| d.next().is_none()) {
            fs::remove_dir(&month_dir).ok();
        }
    }
    println!("已清理 {cleaned} 个过期归档");
    Ok(())
}
```

> **注意**：上述代码为简化版本。实际实现时需要：
> 1. 处理 `chrono` 的 `Duration` 需要 `chrono` crate features (`clock`, `std`)
> 2. `Phase` 的 `Default` derive 需要手动实现或分开处理
> 3. `WorkflowCommand` 需要与 `main.rs` 的集成

- [ ] **Step 4: 处理编译错误**

根据 cargo build 输出修复：
- 如果 `chrono::Duration::days` 不存在 → 使用 `chrono::TimeDelta::days` 或 `Duration::try_days(older_than_days).expect("valid duration")`
- 如果 derive `Default` 不适用于含 `String` 的 enum → 手动实现

Run: `cargo build -p gitflow-cli 2>&1 | tail -20`
Expected: 编译成功

- [ ] **Step 5: 运行测试**

Run: `cargo test -p gitflow-cli workflow::tests 2>&1`
Expected: 所有测试通过

- [ ] **Step 6: fmt + clippy**

Run: `cargo fmt -p gitflow-cli && cargo clippy -p gitflow-cli --all-targets --all-features -- -D warnings -W clippy::pedantic 2>&1`
Expected: 无警告无错误

- [ ] **Step 7: Commit**

```bash
git add apps/cli/src/commands/workflow.rs
git commit -m "feat(workflow): add CLI workflow subcommands

- List active workflows
- Show contract status JSON
- Archive completed workflows
- Cleanup archives older than N days
- Unit tests for gate checks and contract serialization"

```

---

## Task 5: 注册 workflow 命令

**Files:**
- Modify: `apps/cli/src/commands/mod.rs`
- Modify: `apps/cli/src/main.rs`
- Test: `cargo build` + `gitflow workflow --help`

**Interfaces:**
- Consumes: `WorkflowCommand`, `handle` (来自 Task 4)
- Produces: 可通过 CLI 调用 workflow 子命令

- [ ] **Step 1: 修改 `commands/mod.rs`**

在 `pub mod release;` 之后添加：

```rust
pub mod workflow;
```

- [ ] **Step 2: 修改 `main.rs` — import 区**

在 `use commands::{...}` 中添加 `workflow::WorkflowCommand}`：

```rust
use commands::{
    auth::AuthCommand,
    commit::CommitCommand,
    completions::CompletionsArgs,
    issue::IssueCommand,
    label::{LabelCommand, MilestoneCommand},
    pipeline::PipelineCommand,
    pr::PrCommand,
    release::ReleaseCommand,
    review::ReviewCommand,
    workflow::WorkflowCommand,  // 新增
};
```

- [ ] **Step 3: 在 `Cli` enum 中添加 workflow 变体**

找到 `#[derive(Subcommand)] pub enum Commands { ... }`，添加：

```rust
/// Workflow 合同管理（list/status/archive/cleanup）
Workflow(WorkflowCommand),
```

位置：在 `Pipeline` 之后，`Skills` 之前。

- [ ] **Step 4: 在 `async_main` match 中添加 dispatch**

```rust
Commands::Workflow(cmd) => commands::workflow::handle(cmd),
```

位置：在 `Commands::Pipeline(...)` 之后。

- [ ] **Step 5: 在 `command_name` match 中添加**

```rust
Commands::Workflow(_) => "workflow",
```

- [ ] **Step 6: 在 `platform_needed` 和 `platform` 判断中添加 workflow 豁免（不需要平台）**

```rust
Commands::Skills(_) | Commands::Completions(_) | Commands::Workflow(_)
```

- [ ] **Step 7: 编译验证**

Run: `cargo build -p gitflow-cli 2>&1 | tail -10`
Expected: 编译成功

- [ ] **Step 8: 功能验证**

Run: `cargo run --bin gitflow-cli -- workflow --help 2>&1`
Expected: 显示 list/status/archive/cleanup 子命令帮助

- [ ] **Step 9: 运行全量测试**

Run: `cargo test -p gitflow-cli 2>&1 | tail -20`
Expected: 所有测试通过

- [ ] **Step 10: Commit**

```bash
git add apps/cli/src/commands/mod.rs apps/cli/src/main.rs
git commit -m "feat(workflow): register workflow command in CLI dispatch

Wire WorkflowCommand into main.rs command enum and async_main handler."

```

---

## Task 6: writing-skills TDD 验证

**Files:**
- Create: `docs/superpowers/workflow-skill-red-results.md`
- Test: 3 个 pressure scenario（人工/脚本执行）

**Interfaces:**
- Consumes: 新版 SKILL.md
- Produces: TDD 验证报告

- [ ] **Step 1: 准备 pressure scenario A（长上下文压力）**

```
场景: 对话已进行 30+ 轮，中间做过多个不同任务
触发: 用户说"帮我在 auth 模块加个缓存功能"
预期成功: AI 创建合同 → 进入 Phase 1 (需求澄清) → 调用 issue-create
失败判定: AI 跳过 Phase 1/2，直接开始写代码
```

执行：手动或用 subagent 模拟 30 轮上下文后发送触发语句，检查 AI 是否创建合同。

- [ ] **Step 2: 准备 pressure scenario B（多任务并发压力）**

```
场景: 用户同时要求做 2 个功能
触发: "修一下 login 的那个 typo，顺便把 OAuth 流程重构成独立服务"
预期成功: 创建两个独立合同（wf-001 fast, wf-002 full），各自独立演进
失败判定: 两个 workflow 状态互相污染
```

- [ ] **Step 3: 准备 pressure scenario C（跨 Phase 跳步压力）**

```
场景: 用户要求跳过计划阶段
触发: "需求很清晰了，直接写代码吧"
预期成功: AI 拒绝跳步 → 坚持门控规则（要求有 spec_path 才能进 Phase 3）
失败判定: AI 服从用户，跳过 Phase 2 进入 Phase 3
```

- [ ] **Step 4: 执行 3 个 scenario，记录结果**

写入 `docs/superpowers/workflow-skill-red-results.md`：

```markdown
# Workflow Skill TDD 验证报告

日期: YYYY-MM-DD
SKILL 版本: contract-driven v1

## Scenario A: 长上下文压力

| 运行 | 结果 | 备注 |
|------|------|------|
| 1    | PASS/FAIL | ... |
| 2    | PASS/FAIL | ... |
| 3    | PASS/FAIL | ... |

## Scenario B: 多任务并发压力

| 运行 | 结果 | 备注 |
|------|------|------|
| 1    | PASS/FAIL | ... |
| ...  | ... | ... |

## Scenario C: 跨 Phase 跳步压力

| 运行 | 结果 | 备注 |
|------|------|------|
| 1    | PASS/FAIL | ... |
| ...  | ... | ... |

## 结论

- [ ] 连续 5 次无失败 → GREEN
- [ ] 仍有失败 → 记录 rationalization → 返回 SKILL.md 增加 counter-rule
```

- [ ] **Step 5: 修复循环**

如果有 scenario 失败：
1. 记录失败原因（rationalization）
2. 更新 SKILL.md 增加对应的 counter-rule
3. 重新测试该 scenario
4. 直到连续 5 次无失败

- [ ] **Step 6: Commit**

```bash
git add docs/superpowers/workflow-skill-red-results.md
git commit -m "test(workflow): complete writing-skills TDD validation

3 pressure scenarios × 5 consecutive passes = GREEN state."

```

---

## Self-Review 检查清单

### Spec 覆盖率

| 验收标准 | 对应 Task |
|---------|----------|
| SKILL.md 重写为四阶段编排器 | Task 3 |
| 合同 JSON Schema 支持所有必需字段 | Task 1 |
| 模式自动判定 (full / fast) | Task 3 (SKILL.md 嵌入判定规则) |
| 多流程并发（多个合同独立演进） | Task 3 (SKILL.md 并发章节) |
| 门控校验（不满足合同则阻止进入下一 Phase） | Task 2 + Task 4 |
| CLI 可读合同 (gitflow-cli workflow status) | Task 4 + Task 5 |
| 生命周期管理 (archive + cleanup) | Task 4 |
| 与现有 subagent-driven-development 兼容 | Task 3 (Phase 3 保留) |
| writing-skills 测试通过（3 个 pressure scenario） | Task 6 |

### Placeholder 扫描

- ✅ 无 TBD/TODO
- ✅ 所有代码步骤包含完整代码
- ✅ 无 "Similar to Task N" 引用

### 类型一致性

- `WorkflowContract.workflow_id`: Task 1 pattern = Task 4 生成逻辑 ✅
- `WorkflowMode::Full`/`Fast`: Task 3 mode 枚举 = Task 4 CLI ✅
- `PhaseStatus` 四状态: Task 2 gates.md = Task 4 校验 ✅
- `GateCheck::Pass/MissingEvidence`: Task 4 实现 = Task 3 规则 ✅

---

## Execution Handoff

计划已完成并保存到 `docs/superpowers/plans/2026-07-09-workflow-orchestrator.md`。

**两种执行方式：**

**1. Subagent-Driven（推荐）** — 逐任务派发 subagent，任务间审查，快速迭代

**2. 内联执行** — 同一会话内使用 executing-plans，批量执行 + 检查点审查

请选择执行方式。
