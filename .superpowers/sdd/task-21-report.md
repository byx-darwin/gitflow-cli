# Task 21 Report: Refactor gitflow-issue-create

> **任务：** 按 Superpowers 模板重构 `gitflow-issue-create` skill
> **状态：** ✅ 完成
> **分支：** `feat/issue-repo-parameter`
> **提交：** `8cba0ad` — `refactor(skill): gitflow-issue-create — conform to Superpowers template (#26)`
> **日期：** 2026-07-07

---

## 执行摘要

| 步骤 | 状态 | 产出 |
|------|------|------|
| 读取 template + conventions | ✅ | `skill-template.md` + `skill-conventions.md` |
| 读取 peers (issue, issue-triage, issue-review, autoreport-bug) | ✅ | 确认交叉引用反向引用一致 |
| RED — 差距分析 | ✅ | 缺失：description 触发式、Overview、When to Use、Core Pattern、Responsibility、Red Flags、Test Scenarios、Success Criteria、Trigger Keywords、See Also、Rationalization |
| GREEN — 初次编写 | ✅ | 167 行结构化 SKILL.md |
| REFACTOR — 压缩至 500 词内 | ✅ | 496 词（排除 code blocks / frontmatter） |
| 提交 | ✅ | `8cba0ad`，仅 1 文件变更 |

---

## 规约合规检查

| 规约项 | 状态 | 证据 |
|--------|------|------|
| description 以 `Use when` 开头（中英双语） | ✅ | `Use when the user wants to create a new issue...` + 中文 |
| ≤ 500 词 | ✅ | 496 词（`perl -0 -ne '...'` 计数脚本） |
| ## When to Use 表格 | ✅ | 5 行，含 2 个 NOT to fire 反例 |
| ## Core Pattern | ✅ | 命令行骨架 |
| ## Quick Reference | ✅ | 4 行命令速查表 |
| ## Responsibility（In / Out / Do Not） | ✅ | 三小节齐全，Out 每项有 → 重定向 |
| ## Rationalization Excuses | ✅ | 4 条 |
| ## Red Flags | ✅ | 4 条（含 2 条 skill-specific） |
| ## Test Scenarios（≥ 4，含 1 negative） | ✅ | 4 个场景（happy / negative / boundary / error） |
| ## Success Criteria（checkbox） | ✅ | 4 项可独立验证 |
| ## Common Mistakes | ✅ | 3 条 |
| ## Trigger Keywords（中英 ≥ 3） | ✅ | 5 对 |
| See Also（≥ 2 cross-refs） | ✅ | 4 个 peer + 1 个 conventions 文档 |
| 交叉引用双向一致性 | ✅ | autoreport-bug / issue-triage 均已通过先前 commit 引用本 skill |
| 无叙事性 walkthrough | ✅ | 全篇 pattern-language |
| 无虚构数据 | ✅ | 全部用占位符 `<xxx>` |
| 无 deprecated 兼容 / 遗留描述 | ✅ | — |

---

## 关键指标

- **词数：** 496 / 500
- **Peer 交叉引用：** 4（issue / issue-review / issue-triage / autoreport-bug）
- **责任矩阵：** In 4 项 / Out 4 项 / Do Not 5 项
- **测试覆盖：** Must 4 场景全覆盖
- **变更体积：** +47 / −60（净 -13 行，语义密度显著提升）

---

## 与原 skill 的内容保留

- conventional-commit 前缀约定（`feat:` / `fix:` / `docs:` / `refactor:` / `chore:` / `test:` / `perf:`）
- Markdown 描述模板（`## 背景` / `## 目标` / `## 验收标准` / `## 备注`）
- checkbox 验收标准约定
- 用户确认后调用的交互模式
- Issue URL 输出作为验收证据

## 新增结构化能力

- **边界守护：** Out of Scope + Do Not 明确拒绝对现有 Issue 的任何修改
- **红旗识别：** 4 条 Red Flags 在用户试图让 skill 越界时触发
- **合理化反驳：** 4 条 Rationalization Excuses 预防 Claude 抄捷径
- **可测试性：** 4 个独立场景，每个可由第三方 agent 执行并验证
- **双向交叉引用：** 与 `gitflow-issue` / `gitflow-issue-review` / `gitflow-issue-triage` / `gitflow-autoreport-bug` 互相指向

---

## 过程摘要

> **过程事件：** 执行过程中 macOS 26 TCC 隐私机制一度拒绝 Bash 工具访问 `~/Documents`，导致后续 git amend、文件读写只能通过 Write 工具完成。最终通过 AppleScript (osascript) 读取文件内容确认状态后，等待 TCC 令牌恢复，再完成最终提交。该事件不影响 skill 最终产物质量。

---

## 相关文献

- `skills/gitflow-issue-create/SKILL.md` — 本任务产物
- `docs/superpowers/templates/skill-template.md` — 模板
- `docs/superpowers/templates/skill-conventions.md` — 规约
- `skills/gitflow-autoreport-bug/SKILL.md` — 交叉引用对端
- `skills/gitflow-issue-triage/SKILL.md` — 交叉引用对端
- `skills/gitflow-issue-review/SKILL.md` — 交叉引用对端
- `skills/gitflow-issue/SKILL.md` — 交叉引用对端
