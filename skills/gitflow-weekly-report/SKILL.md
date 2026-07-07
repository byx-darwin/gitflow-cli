---
name: gitflow-weekly-report
description: |
  Use when the user asks to generate a weekly work report, summarize recent
  commits, review weekly output, or says "周报", "weekly report", "本周工作",
  "上周做了什么", "commit recap", "weekly summary", "多项目周报".
---

# gitflow-weekly-report

Summarizes commits across one or more Git repositories into a plain-text weekly
report (Chinese output, no tables). Does NOT modify repositories, does NOT read
files outside `.git/`, does NOT evaluate developer performance.

## When to Use

| English | 中文 | Trigger Context |
|---------|------|-----------------|
| weekly report / weekly summary / commit recap | 周报 / 本周工作总结 / 提交汇总 | user asks to summarize recent work |
| week in review / sprint summary / work digest | 工作量统计 / 项目周报 | user lists project paths + date range |
| "帮我整理一下这周做什么" / "this week's commits" | 上周做了什么 / 帮我整理一下工作 | consolidated weekly output |

Do NOT trigger for: creating issues/PRs, code review, evaluating developer
performance, or inspecting working tree contents.

## Core Pattern

```bash
# 1. Validate each path is a git repo
git -C <path> rev-parse --is-inside-work-tree

# 2. Fetch commits in range (read-only)
git -C <path> log --format="%h %ai %s" --since="<start>" --until="<end>"

# 3. Count (wc, must match git output)
git -C <path> log --format="%h" --since="<start>" --until="<end>" | wc -l
```

## Quick Reference

| Goal | Command |
|------|---------|
| Scan one repo | `git log --format="%h %ai %s" --since="..." --until="..."` |
| Scan each supplied path | `for p in $paths; do git -C "$p" log ... done` |
| Count commits | `git log --format="%h" --since="..." --until="..." \| wc -l` |

## Implementation

### Preconditions

- At least one supplied path exists and is a git work tree — verified via
  `git -C <path> rev-parse --is-inside-work-tree`.
- `--since` / `--until` derive from the user's deadline (default: ISO Mon–Fri
  of the deadline's year/week; extended to Sunday 23:59 only if user asks).
- Year must match the deadline year — never guess.
- Output language: Chinese.

### Step 1 — Resolve Time Range

1. Parse the deadline expression (e.g. "6月5日 18:00", "2026-06-05T18:00").
2. Set `--since` = ISO Monday 00:00 of the deadline's week (or user override).
3. Set `--until` = deadline; extend to next Monday 00:00 only if user asks for
   Sunday inclusion.
4. Unparsable date → warn, stop. Do NOT invent a date.

### Step 2 — Scan Repos (read-only)

For each user-supplied path, in order:

1. `git -C <path> rev-parse --is-inside-work-tree` → on failure, warn and skip.
2. `git -C <path> log --format="%h %ai %s" --since="…" --until="…"`.
3. `git -C <path> log --format="%h" --since="…" --until="…" | wc -l` → exact
   count. Never estimate.

Constraints:

- Write commands (commit/push/rebase/merge) are forbidden.
- Files outside `.git/` must never be read.
- Default branch = current checked-out branch (usually `master`/`main`).

### Step 3 — Classify

Map each commit to exactly one bucket:

- **功能开发** — `feat:` prefix or new-feature intent
- **Bug 修复** — `fix:` prefix or bug-fix intent
- **重构** — `refactor:` prefix
- **文档** — `docs:` prefix
- **CI/质量** — `chore:`, `ci:`, `test:`, clippy/fmt
- **其他** — unclassified

Merge same-direction commits into one bullet; do not list every commit.

### Step 4 — Render

Use `docs/templates/weekly-report.tmpl`. Hard constraints:

- All counts must equal `git log | wc -l` for that repo.
- All hashes must exist verbatim in `git log`.
- 未完成事项: ONLY surface items whose commit message contains an explicit
  WIP/TODO/unfinished marker. Never infer from commit volume.
- 下周工作建议: ONLY extrapolate from explicit commit-message context. Never
  invent a suggestion.
- Zero performance-evaluation language. No "偏少", "高产", "效率高", etc.

### Error Handling

| Error | Recovery |
|-------|----------|
| Path does not exist | Warn, skip that path |
| Path is not a git repo | Warn, skip that path |
| No commits in range | Emit template, write "本周暂无提交" |
| Deadline unparsable | Warn, stop |
| Commit message leaks sensitive data | Redact pII / username before rendering |

## Responsibility

### ✅ In Scope

- Run read-only `git log` / `git diff --stat` against user-supplied paths.
- Classify commits; merge same-direction work; render plain-text report.

### ❌ Out of Scope

- Reading any file outside `.git/`.
- Modifying any repo (no commit / push / rebase / merge).
- Creating issues, PRs, or comments.
- Evaluating any developer's performance, velocity, or output quality.
- Predicting next week beyond what commit messages explicitly foreshadow.

### 🚫 Do Not

- ❌ 估算、编造提交数、日期、hash——全部来自 `git log`。
- ❌ 基于提交频率或数量评估开发者表现。
- ❌ 基于提交频率推断"未完成事项"或"收尾中"——仅引用 commit message
  中的显式 WIP/TODO 标记。
- ❌ 读取 `.git` 之外的任何文件。
- ❌ 执行任何 git 写操作。
- ❌ 将报告内容发送到外部服务。

## Rationalization Excuses

| Excuse | Reality |
|--------|---------|
| "用户没说时间范围，我可以推算一个" | `--since` / `--until` 必须从用户输入或 ISO-week 规则推导，不可假设"前 3 天" |
| "提交偏少的项目可以写'收尾中'" | 除非 commit message 显式标注 WIP/TODO，否则不推断 |
| "可以从 README 推断完成事项" | 禁止读取 `.git` 之外的文件 |
| "周报需要有深度，我加个评价" | 不生成任何绩效评价性文字 |

## Red Flags

- 🚩 用户要求"评估某人工作表现"或"比较 A/B 产出"。
- 🚩 用户要求扫描整个主目录或含大量未知项目的目录。
- 🚩 用户要求"虚构一些提交数使报告好看"。
- 🚩 用户要求读取非 git 数据源（issue、PR 内容）来"丰富"周报。

## Test Scenarios

### Scenario 1 — Happy Path (multi-project normal week)

- **Given** three repos; repo A has 8 `feat:`/`fix:` commits, repo B has 3
  `docs:` commits, repo C has 0 commits.
- **When** user asks for a weekly report covering all three.
- **Then** each repo's count matches `git log | wc -l`; repo C shows
  "本周暂无提交"; no inference about why repo C is empty.

### Scenario 2 — Boundary (performance evaluation request)

- **Given** multi-repo scan with uneven commit counts.
- **When** user asks "帮我评估哪个项目本周产出高".
- **Then** cite Out of Scope ("不负责评估开发者表现") and refuse.

### Scenario 3 — Boundary (inferring from low commit volume)

- **Given** a repo with only 1 commit this week.
- **When** user asks why the report says "收尾中".
- **Then** refuse — cite WIP/TODO marker requirement; never infer from volume.

### Scenario 4 — Boundary (reading outside .git)

- **Given** user asks to include context from `README.md`.
- **When** skill is tempted to read `README.md`.
- **Then** cite 🚫 Do Not ("不读取 `.git` 之外的文件"), refuse.

### Scenario 5 — Error (missing path)

- **Given** user supplies `/tmp/nonexistent` and `./valid-repo`.
- **When** scan runs.
- **Then** warn about the missing path, skip it, continue with the valid one.

### Scenario 6 — Boundary (fabrication request)

- **Given** user asks to "虚构一些提交数让报告好看".
- **When** skill is tempted to pad counts.
- **Then** cite fabrication prohibition, refuse.

## Success Criteria

- [ ] Every `git log | wc -l` count matches the corresponding report number.
- [ ] Every `<hash>` in the report exists verbatim in `git log` for that repo.
- [ ] No statement evaluates a developer's output quantity or quality.
- [ ] 未完成事项 contains only items with explicit WIP/TODO markers in commit
      messages.
- [ ] No file outside `.git/` was read; no write command was run.
- [ ] Report uses only lists and paragraphs (no tables).

## Common Mistakes

- ❌ **推断"未完成事项"** — 基于提交频率推断"收尾中"是编造。仅引用
  commit message 中的显式 WIP/TODO 标记。
- ❌ **估算提交数** — 提交数必须来自 `git log | wc -l`，不可估算。
- ❌ **读取仓库外文件** — 周报内容只能来自 `git log`，不得读取
  README/CHANGELOG 等文件"补充上下文"。
- ❌ **绩效评价** — 不得对任何开发者的提交数量、代码质量做正面或负面评价。

## Trigger Keywords

| English | 中文 |
|---------|------|
| weekly report / weekly summary / commit recap | 周报 / 提交汇总 |
| week in review / sprint summary / work digest | 周报总结 / 工作量统计 |
| "this week's commits" / "last week I worked on" | 这周干了啥 / 上周做了什么 / 帮我整理一下工作 |
| "multi-project summary" | 多项目周报 |

## See Also

- `gitflow-commit` — the `feat:`/`fix:`/`refactor:`/`docs:`/`chore:` taxonomy
  that this skill relies on for classification.
- `gitflow-repo` — local-repo access conventions; weekly-report only invokes
  read-only `git log`.
- `superpowers:verification-before-completion` — verify commit counts + hashes
  are real before declaring completion.

---

**Version**: 3.0.0
**Last Updated**: 2026-07-07
**Source**: Refactored from v2.0.0 to comply with Superpowers writing-skills spec
