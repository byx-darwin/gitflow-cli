# gf-pr-apply-feedback 审查闭环契约 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the open loop in `skills/gf-pr-apply-feedback/SKILL.md` — after push, bounce back to a `gf-pr-review` re-assessment, cap at 3 rounds with user escalation, dedupe rejected findings in-session, and short-circuit re-review on comment/doc-only diffs.

**Architecture:** Pure documentation change to one skill file. No Rust code, no new CLI subcommands — the loop is expressed as flowchart branches + Core Pattern pseudo-code that reuse existing `gf pr view` / `gf review` / `git push` / `git diff --stat` commands already documented elsewhere in this repo.

**Tech Stack:** Markdown (SKILL.md), Mermaid flowchart syntax, bash pseudo-code blocks (documentation only — not executed as a script in CI).

**Spec:** `docs/superpowers/specs/2026-09-19-gf-pr-apply-feedback-review-loop-design.md`

## Global Constraints

- Skill 源代码在 `skills/` 目录下 — 只改 `skills/gf-pr-apply-feedback/SKILL.md`，不要改 `.claude/skills/` 中的副本（来自项目 CLAUDE.md）。
- 不新增 CLI 子命令；复审复用 `gf-pr-review` 现有的 `gf review approve/request-changes` 输出（设计文档「范围」一节）。
- 拒绝记录仅会话内内存清单，不落盘、不发 PR 评论（设计文档「拒绝记录」一节）。
- 轮次上限固定为 3，从首轮修复（第一次 push）开始计数（设计文档「轮次上限」一节）。
- 纯注释/文档变化的 diff 短路跳过复审调用，判据是本轮 diff 内容而非评论文字（设计文档「消耗性重跑短路」一节）。
- 第 3 轮仍未清零 → 升级交回用户决策，不自动重试、不报错终止整个技能（设计文档「轮次上限」一节，与 `skills/gf-pipeline-analyzer/SKILL.md:101-109` Escalation Rule 语义一致）。
- 本次是纯文档改动：不涉及 `cargo build/test/clippy`；验证靠 markdown 自洽性 + `make check-agent-sync`（`scripts/validate-skill-commands.sh` + `scripts/validate-skill-links.sh` + `scripts/verify-skills-when-not-to-use.sh`）。

---

### Task 1: gf-pr-apply-feedback SKILL.md 审查闭环契约

**Files:**
- Modify: `skills/gf-pr-apply-feedback/SKILL.md:110-170`（Flowchart、Core Pattern 追加段、Error Handling、Responsibility、Test Scenarios、Success Criteria 六个章节）

**Interfaces:**
- Consumes: 无上游任务（本计划只有一个任务）。复用文中已有的 `gf pr view <pr>`、`gf review approve/request-changes/comment <n>`、`git push origin <pr-branch>` 命令名，不得改名。
- Produces: 无下游任务消费本任务产物；本任务是终态交付。

- [ ] **Step 1: 记录基线，确认当前文件内容与设计文档引用的行号一致**

```bash
git log -1 --format=%H -- skills/gf-pr-apply-feedback/SKILL.md
grep -n "DONE\|git push origin" skills/gf-pr-apply-feedback/SKILL.md
```

Expected: 命令能跑通，`grep` 输出包含设计文档提到的 `M[git push + pr comment summary] --> DONE`（第 131-132 行附近）。若行号已漂移，以 `grep` 实际输出为准调整后续 Step 2 的 old_string 定位，不要死板套用行号。

- [ ] **Step 2: 修改 Flowchart（第 110-134 行区块）**

将现有：

```mermaid
  NEXT --> K{More comments?}
  K -->|yes| E
  K -->|no| L{Push confirmed by user?}
  L -->|no| DONE
  L -->|yes| M[git push + pr comment summary]
  M --> DONE
  DONE --> END
```

替换为：

```mermaid
  NEXT --> K{More comments?}
  K -->|yes| E
  K -->|no| L{Push confirmed by user?}
  L -->|no| DONE
  L -->|yes| M[git push + pr comment summary]
  M --> N{Diff since last round: comment/docs only?}
  N -->|yes| DONE
  N -->|no| O[Bounce back: gf-pr-review]
  O --> P{Verdict: approve?}
  P -->|yes| DONE
  P -->|no, request-changes| Q[Filter out session-rejected findings]
  Q --> R{Any actionable findings remain?}
  R -->|no| DONE
  R -->|yes| S{Round count == 3?}
  S -->|no, increment round| E
  S -->|yes| T[Escalate: show remaining findings + round count, hand back to user]
  T --> END
  DONE --> END
```

- [ ] **Step 3: 修改 Core Pattern（第 51-59 行区块），在 push 之后追加回跳复审伪代码**

将现有 Core Pattern 代码块：

```bash
gf pr view <pr>                               # fetch + list pending
# prioritize → confirm each comment with user
git checkout <pr-branch>                                       # confirmed PR branch
# per comment: edit → test → commit (referencing reviewer + location)
# Note: resolve comments via platform web UI (GitHub/GitLab)
git push origin <pr-branch>                                    # ONLY after explicit confirmation
gf pr comment <pr> --body "<summary>"                 # notify
```

替换为（追加复审回跳分支，不改前半段）：

```bash
gf pr view <pr>                               # fetch + list pending
# prioritize → confirm each comment with user
git checkout <pr-branch>                                       # confirmed PR branch
# per comment: edit → test → commit (referencing reviewer + location)
# Note: resolve comments via platform web UI (GitHub/GitLab)
git push origin <pr-branch>                                    # ONLY after explicit confirmation
gf pr comment <pr> --body "<summary>"                 # notify

# Round loop: round starts at 1 on the first push above, caps at 3
DIFF_KIND=$(git diff --stat <last-round-sha>..HEAD)
if [[ only comments/docs changed in "$DIFF_KIND" ]]; then
  : # short-circuit — no re-review, DONE
else
  gf pr view <pr>                             # re-fetch verdict via gf-pr-review
  # gf-pr-review assesses 6 dimensions, submits via `gf review approve|request-changes <n>`
  # request-changes findings are filtered against this session's in-memory rejected list
  # (dimension + path:line fingerprint) before being re-confirmed with the user
  # if round == 3 and actionable findings remain: escalate to user, stop looping
fi
```

- [ ] **Step 4: 修改 Error Handling 表（第 98-108 行），新增一行退出码捕获护栏**

在现有表格末尾追加一行：

```markdown
| Test exit code captured after `\|\| true` (e.g. `wait ... \|\| true; EXIT=$?`) | `EXIT` is always 0 — capture immediately after the command, never after a `\|\| true` guard |
```

- [ ] **Step 5: 修改 Responsibility → In（第 72 行）**

将：

```markdown
**In:** prioritize · apply fixes (confirmed) · test · resolve · push (confirmed) · notify.
```

替换为：

```markdown
**In:** prioritize · apply fixes (confirmed) · test · resolve · push (confirmed) · notify · bounce back to `gf-pr-review` · escalate to user at round cap.
```

- [ ] **Step 6: 新增 4 个 Test Scenarios（追加在现有第 4 条 Error 场景之后，Success Criteria 之前，即第 148-149 行之间）**

```markdown
### 5: Loop Continues
- **Given** post-push `gf-pr-review` returns `request-changes` with 2 unresolved findings · **When** round < 3 · **Then** filter session-rejected findings, re-confirm remaining with user, loop back to fix step

### 6: Rejected Finding Not Repeated
- **Given** user rejected a finding at `path:line` in round 1 · **When** round 2 re-review flags the same `path:line` again · **Then** filtered out silently, not re-shown to user

### 7: Comment-Only Diff Short-Circuits
- **Given** round's diff is entirely comment/doc changes (`git diff --stat` shows no code lines) · **When** push completes · **Then** skip `gf-pr-review` bounce-back entirely, go straight to DONE

### 8: Round Cap Escalation
- **Given** round 3 push completes and `gf-pr-review` still returns `request-changes` with actionable findings · **When** loop would continue · **Then** stop automatic looping, show remaining findings + round count, hand decision back to user
```

- [ ] **Step 7: 修改 Success Criteria（第 150-156 行），追加 3 条**

在现有列表末尾追加：

```markdown
- [ ] Loop terminates on findings reaching zero, not on comment list exhaustion
- [ ] Round cap is 3; exceeding it escalates to the user instead of retrying silently
- [ ] Test exit codes are captured immediately after the command, never after `|| true`
```

- [ ] **Step 8: 本地自洽性检查——渲染 Mermaid 语法、确认无孤立节点**

```bash
grep -c '^  [A-Z] --> \|^  [A-Z]\[' skills/gf-pr-apply-feedback/SKILL.md
```

Expected: 命令执行成功（非零退出码即失败），人工确认新增节点 `N/O/P/Q/R/S/T` 均有入边和出边，无孤立悬空节点（`T --> END` 收尾，不再回到 `DONE`）。

- [ ] **Step 9: 运行 skill 校验**

```bash
make check-agent-sync
```

Expected: 三个子脚本（`verify-skills-when-not-to-use.sh`、`validate-skill-commands.sh`、`validate-skill-links.sh`）均输出 ✓，退出码 0。本任务未引入新 `gf` 子命令、未新增/删除文件链接，预期无破坏性变更。

- [ ] **Step 10: Commit**

```bash
git add skills/gf-pr-apply-feedback/SKILL.md
git commit -m "feat(skills): close review loop in gf-pr-apply-feedback with 3-round cap (#334)"
```
