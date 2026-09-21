# Issue Triage Report — 2026-09-17

Skill: `gf-issue-triage` · Scope: all open Issues (repo-wide) · CLI: `gf`
Context: Phase 4 "Issue triage" step of `/gf-workflow` full-mode run (contract `wf-2026-09-16-002`) for Issue #329 (`feat(skills): 新增 gf-walkthrough`), delivered via local merge into `dev` (merge commit `3e9bcc5`).

**Binding override for this run**: no labels applied. This run is classification/reporting only — no `gf issue edit`, `gf label` mutation, or `gf issue comments` write was executed. Every command used was read-only (`gf issue list`, `gf issue view`, `gf issue comments`). Where the skill would normally add `type:*` / `priority:*` / `triage:done`, this report instead states what would be applied.

## Summary

- Open Issues scanned: 30
- Already `triage:done` (skipped, idempotent): **30 — all of them**
- Newly triaged this run: **0**

No open Issue required classification this run — every open Issue already carries a `type:*` label, a `priority:*` label, and `triage:done` from the prior run (`docs/issue-triage-report-2026-09-16.md`, contract `wf-2026-09-16-001`). Consequently the labelling override above has no practical effect this run: there was nothing to label even without the restriction. This is expected idempotent behavior per the skill's Test Scenario 5.

Since the 2026-09-16 run, the open-Issue set changed by exactly two Issues:
- **#327** (`gf-smell`) is now **closed** — it was the delivery target of the prior workflow run and has since been closed.
- **#93** (`多角色项目评估与 2026 下半年产品路线图`) is present in today's open list but was absent from the 2026-09-16 report despite being open since 2026-07-30 (`triage:done`, `type:feature`, `priority:high`, consistent with its child roadmap Issues #101–#103). This is a long-lived parent tracking Issue for the overall roadmap (comments show it decomposing into #90, #95–#103); its omission from the prior report appears to be that report's oversight, not a state change here. No action needed — its existing labels are correct and consistent.

## Ready-to-Close (flagged, not actioned)

**#329 `feat(skills): 新增 gf-walkthrough`** — delivered and merged locally into `dev` via merge commit `3e9bcc5` (this workflow run, `wf-2026-09-16-002`). `gf issue list --state open` confirms it is still open: a local merge carries no `Closes #N` linkage, so GitHub never auto-closed it. It already carries `type:feature`, `priority:medium`, `triage:done` from the prior run — no re-triage needed. Flagging for manual close by the workflow owner; this run does not close it (read-only scope, and closing is outside `gf-issue-triage`'s responsibility).

## Recommended New Issues (not created — verified not to exist yet)

Searched all 30 open Issue titles and bodies (`gf issue list --state open`, full-text) for each topic below; none has an existing Issue. Listed here per task instruction, not created.

1. **Word-count enforcement is broken in `docs/superpowers/templates/skill-conventions.md:21`.** The documented check uses `scalar(/.../g)`, which in scalar context yields a boolean `1`, not a match count — so the 500-word hard limit on skills has never actually been enforced. Sibling skills measure 640–1878 words against that unenforced limit. Same ticket should also resolve the companion contradiction: §1.2's prose says inline code counts toward the limit, but the command strips code spans before counting — the same file measures 614 words under the prose rule vs 499 under the (broken) command. Suggested type/priority: `bug` / `high` (a documented hard gate that has silently never fired, affecting ~13 skills' compliance status).
2. **`check-walkthrough-skill` Makefile target, check #3 — `getline` window skips adjacent entries.** Its awk loop consumes up to 3 lines via `getline`, so a `- [Measured]` entry falling within 3 lines of a previously consumed entry is never validated — latent under-counting in what is meant to be an acceptance gate for `gf-walkthrough`. Suggested type/priority: `bug` / `medium` (gate integrity gap, but narrow blast radius — one Makefile target, one skill).
3. **Externalized skill templates under `docs/superpowers/templates/` are unreachable after `make install-skills`.** That target only `cp -r`s `skills/*` into `~/.claude/skills/`, so any skill referencing `docs/superpowers/templates/...` breaks post-install. Repo-wide pattern affecting roughly 13 skills, not specific to `gf-walkthrough`. Suggested type/priority: `bug` / `medium` (broad blast radius, but has a workaround — work from the repo checkout rather than the installed copy).
4. **`crates/e2e-github/tests/noauth.rs:1` overclaims "任何环境" (any environment).** The tests implicitly require the `gh` CLI to be installed; without it, assertions fail against install-guidance text rather than the intended login-guidance text, silently testing the wrong code path. Suggested type/priority: `bug` / `low` (test-suite correctness issue, not a shipped-behavior defect; narrow scope).
5. **`skills/` vs `~/.claude/skills/` install drift.** `make install-skills` is a plain `cp -r` with no diffing or manifest check; `gf-smell` (delivered under #327) exists in the repo but has never been installed, so it is invisible to Claude Code sessions relying on the installed copy. Overlaps conceptually with #342 (`check-agent-sync` gate gaps) and #347 (second install path via `.claude-plugin/`) but is distinct: this is about the *existing* `gf skills install` / `make install-skills` paths silently drifting, not about adding a new path. Suggested type/priority: `bug` / `medium` (concrete instance already observed with #327/gf-smell, not just theoretical).

## Full Priority-Ranked View (all 30 open Issues, existing labels — unchanged this run)

### 🔴 Urgent (0 — 0%)

None.

### 🟠 High (3 — 10%)

| # | Title | Type |
|---|-------|------|
| 93 | 多角色项目评估与 2026 下半年产品路线图（稳定化 → 增长 → 扩张） | feature |
| 324 | gf pipeline report 对 running/queued 状态的 run 计入失败桶 | bug |
| 341 | Makefile local-rebuild 含 cargo clean，违反 CLAUDE.md 硬禁令 | bug |

### 🟡 Medium (17 — 57%)

| # | Title | Type |
|---|-------|------|
| 101 | 贡献者路径 + 月度发布节奏 | feature |
| 102 | MCP 服务器（Agent 原生接口） | feature |
| 103 | 效率分析报表 + v1.1.0 + 2.0 预告 | feature |
| 329 | 新增 gf-walkthrough (**ready to close — see above**) | feature |
| 330 | 新增 gf-issue-decompose | feature |
| 331 | 新增 gf-architecture-diagram | feature |
| 332 | 新增 gf-refactor | feature |
| 333 | gf-quality / gf-review 引入三级证据标注 | enhancement |
| 334 | gf-pr-apply-feedback 补审查闭环契约 | enhancement |
| 335 | gf-issue-create / gf-issue-review 垂直切片 | enhancement |
| 336 | gf-workflow 补错误分类恢复表 | enhancement |
| 337 | gf-workflow-batch 按依赖边拓扑排序 | enhancement |
| 338 | gf-pr frontmatter 非法键与 gf-pr-review 引用断链 | bug |
| 340 | 覆盖率工具分裂 | bug |
| 342 | check-agent-sync 形同虚设 | enhancement |
| 344 | gf-security-check / gf-regression 接入 gf-workflow | enhancement |
| 348 | gf-quality references 三处功能性缺陷 | bug |

### 🟢 Low (10 — 33%)

| # | Title | Type |
|---|-------|------|
| 114 | 1.0 发布宣发文章 | docs |
| 188 | upstream CLI 新版本: gitcode 0.11.1 | enhancement |
| 227 | upstream CLI 新版本: gh 2.98.0 | enhancement |
| 240 | upstream CLI 新版本: glab 1.115.0 | enhancement |
| 339 | _common.sh 零调用方 | enhancement |
| 343 | 只读 skill 加 allowed-tools | enhancement |
| 345 | 渲染进度看板 HTML | enhancement |
| 346 | diff 交互式审阅网页渲染 | enhancement |
| 347 | 新增 .claude-plugin 清单 | enhancement |
| 349 | 语言画像单源化 | enhancement |

## Findings / Attention Items

1. **#329 is ready to close.** See "Ready-to-Close" above — delivered via local merge (`3e9bcc5`) but still open due to the missing `Closes #N` linkage. Flagged for manual close, not actioned here.
2. **#328 was closed as refuted**, correctly: comments confirm its original premise ("this repo is Rust, so it needs a modernization skill") was overturned, and its two salvageable pieces were explicitly re-homed — the semantic-safety-boundary content into #332, and the organizational-technique content into #349. Both target Issues already carry a "承接自 #328" section per their bodies. No orphaned content found; #328 needs no further action.
3. **Five gaps found during #329's delivery have no tracking Issue yet** — see "Recommended New Issues" above. None was created (out of scope for this read-only triage step); items 1 and 2 concern the same acceptance-gate family (skill word-count and walkthrough-check enforcement) and could reasonably be filed together or separately at the workflow owner's discretion.
4. **#93 reappearing vs. the 2026-09-16 report** is noted for completeness (see Summary) — it is a long-lived, correctly-labelled parent tracking Issue, not an anomaly requiring action.
5. Urgent-priority threshold respected: 0/30 marked urgent. High is 3/30 (10%, at the guideline's upper bound) — driven by #93 (large roadmap tracking Issue, high by design) plus two confirmed defects (#324, #341) already flagged in the prior run.
6. Idempotency confirmed: all 30 open Issues already carried `triage:done` from the prior run; zero redundant `gf issue add-label` calls were made (none were made at all, consistent with both idempotency and this run's read-only override).
7. No duplicates found among open Issues; no `type:unknown` (ambiguous) classifications needed.

## Labels That Would Be Applied (none actually applied — binding override)

No open Issue lacked `type:*` / `priority:*` / `triage:done`, so this run would have applied **zero** labels even without the override. The override is recorded here for auditability: had any Issue needed labelling, this run would have proposed the label set and stopped short of calling `gf issue add-label`, `gf issue edit`, or any comment-posting command, pending explicit user go-ahead.
