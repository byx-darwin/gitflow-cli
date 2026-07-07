---
name: gitflow-workflow
description: |
  Use when the user wants a four-phase gated pipeline (clarify → plan → execute → post-delivery).
  当用户需要四阶段闸门驱动全流程开发时使用。
---

# gitflow-workflow — Gated Orchestrator

编排层只指挥；平台走 `gitflow-cli`，循环走 Superpowers。

## When to Use

| EN | ZH |
|----|----|
| full workflow | 全流程 |
| clarify → plan → execute → deliver | 需求→计划→执行→交付 |

## Core Pattern

```bash
gitflow-cli auth status
gitflow-cli issue list --state open --output json
```

## Quick Reference

| Phase | Sub-skill | Mode |
|-------|-----------|------|
| 1 | `brainstorming` | full ✅ / fast opt |
| 1 | `gitflow-issue-create` | always |
| 1 | `gitflow-issue-review` | full ✅ / fast opt |
| 2 | `writing-plans` | full ✅ / fast opt |
| 3 | `subagent-driven-development` | always |
| 4 | `pipeline-analyzer → issue-triage → review` | always |

Phase 3 内含 TDD / Review。见 workflow-phases-detail.md。

## Implementation

### Preconditions

`command -v gitflow-cli` · `auth status` ok · `git rev-parse` · issues open。

### Steps

1. **Phase 1** — brainstorming → issue-create → issue-review；审计回贴。
2. **Phase 2** — writing-plans + 质量关卡；worktree。
3. **Phase 3** — subagent + TDD + review；合 PR；`Closes #N`。
4. **Phase 4** — analyzer → triage → review。

进阶前必验合规清单。

### Error Handling

| Error | Recovery |
|-------|----------|
| 闸门证据缺失 | 🔒 再进；补齐 |
| worktree 泄露 | `worktree remove` + `branch -d` |
| 合 PR 后 issue 未关 | `issue close --yes` |
| auth 过期 | 重登 resume |
| 回滚 | Issue 留档 |

## Responsibility

In: 编排四阶段闸门 · 合规校验 · 路由到子 skill。
Out: 直接 git/gh · TDD · review。
Block: 跳 sub-skill · 跳 TDD/Review/Phase 4 · 合步骤 · 内联 · 空证据。

## Rationalization

| Excuse | Reality |
|--------|---------|
| 跳脑暴 | full 必须 |
| 无需计划文档 | 闸门不可省 |
| 一步跑完四阶段 | 须逐个调用 |
| PR 合完就结束 | Phase 4 强制 |

## Red Flags

🚩 直接写代码 — Phase 1 先 · 🚩 TDD 慢 — 强制 · 🚩 跳过评审 — 拒绝 · 🚩 合调用 — 保持原子

## Common Mistakes

❌ 跳过闸门证据 — 必输出合规 · ❌ 内联 sub skill — 只路由 · ❌ worktree 未清

## Trigger Keywords

| EN | ZH |
|----|----|
| full workflow end-to-end | 全流程开发 |
| four-phase gated pipeline | 四阶段闸门 |
| clarify plan execute deliver | 需求→计划→执行→交付 |

## Test Scenarios

### 1: Happy
"build auth refresh" → 闸门 → brainstorm → issue-create → review → plans → subagent (+TDD+review) → PR → Phase 4。

### 2: Negative
"just write the code" → redirect Phase 1。

### 3: Boundary
"quick fix" → fast 可跳 brainstorm/plans；仍须 issue-create、subagent、Phase 4。

### 4: Error
Phase 3 auth 报错 → `auth login` → resume。

## Success Criteria

- [ ] 四阶段 + 闸门证据
- [ ] 必选 sub-skills 全调用
- [ ] 编排层未内联 sub skill
- [ ] worktree 已清
- [ ] 回滚 (如有) 留档

## See Also

`/gitflow-issue-create` · `/gitflow-issue-review` — Phase 1
`/gitflow-review` · `/gitflow-pipeline-analyzer` · `/gitflow-issue-triage` — Phase 4
`/gitflow-repo` · `/gitflow-autoreport-bug` · `/gitflow-pr-apply-feedback`
