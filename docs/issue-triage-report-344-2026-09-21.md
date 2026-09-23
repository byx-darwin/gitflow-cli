# Issue Triage Report — Issue #344 Phase 4 (gf-workflow)

- **Date**: 2026-09-21
- **Context**: `gf-workflow` Phase 4 post-delivery check for Issue #344 (`feat(gf-workflow): insert Phase 3 change-surface gate for security/regression checks`), just merged into `dev` at `bb8318b`.
- **Skill**: `gf-issue-triage` (report-only run — **no labels were added or changed**, per the task's explicit no-side-effects instruction; this deviates from the skill's normal "apply labels" behavior by request).
- **Source**: `gf issue list --state open` (26 Issues returned, `pagination.truncated: false` — full coverage, per Issue #360's fix).

## Coverage

All 26 open Issues already carry `triage:done` plus one `type:*` and one `priority:*` label from a prior triage pass earlier today (updatedAt ~03:58–03:59Z, same day). This run re-verifies those classifications rather than performing a first pass, and specifically checks for staleness introduced by #344 landing.

## Priority-ranked summary

| Priority | Count | % |
|---|---|---|
| 🟠 high | 2 | 7.7% |
| 🟡 medium | 5 | 19.2% |
| 🟢 low | 19 | 73.1% |
| 🔴 urgent | 0 | 0% |

| Type | Count | % |
|---|---|---|
| type:feature | 12 | 46.2% |
| type:enhancement | 8 | 30.8% |
| type:bug | 4 | 15.4% |
| type:refactor (non-standard) | 1 | 3.8% |
| type:docs | 1 | 3.8% |

## Findings

### 1. Non-standard `type:refactor` label (#380) inconsistent with taxonomy and with #349

`skills/gf-issue-triage/SKILL.md` defines exactly five type labels: `type:bug`, `type:feature`, `type:enhancement`, `type:docs`, `type:question`. There is no `type:refactor` in the documented taxonomy, yet **Issue #380** ("refactor(core): created_at 在 Issue/PR/Review/Pipeline 多处仍回落 Utc::now()...") carries `type:refactor`.

Compare **Issue #349** ("refactor(skills): 语言画像单源化..."), a similarly refactor-flavored title, which was correctly mapped onto the standard `type:enhancement`. Same title-prefix pattern (`refactor(...)`), two different, inconsistent outcomes — one uses an off-taxonomy label, one uses the closest standard label.

**Recommendation** (not applied — report only): re-label #380 from `type:refactor` to `type:enhancement` (its closest analog per the documented taxonomy) or, if a `type:refactor` label is intentionally being introduced as a taxonomy extension, add it to `skills/gf-issue-triage/SKILL.md`'s Type labels table so the scheme and the applied labels stay in sync.

### 2. #344 landing raises the stakes on #343 (allowed-tools restriction for `gf-security-check`)

**Issue #343** ("feat(skills): 为只读类 skill 添加 allowed-tools 收敛权限") explicitly names `gf-security-check` as the flagship example of a skill that documents itself as read-only ("绝不自动修复") but still holds unrestricted `Write`/`Edit` tool access, with the constraint enforced only by prompt discipline rather than a hard `allowed-tools` gate.

#344 just wired `gf-security-check` (and `gf-regression`) into `gf-workflow`'s Phase 3 change-surface gate as an **automated, unattended** step — it now runs as part of the standard delivery path rather than only on manual invocation. This changes #343's risk calculus: an unenforced read-only contract on a skill that now executes automatically inside every qualifying workflow run is a materially higher-stakes gap than the same contract on a skill invoked only by explicit user request.

**Recommendation** (not applied — report only): consider re-prioritizing #343 from `priority:low` to `priority:medium` (or higher) to reflect that `gf-security-check`'s tool scope is no longer purely a manual-invocation concern. No relabeling was performed in this run.

### No obsoleted issues found

No open Issue duplicates or is fully superseded by #344's scope. The Jev-decision-layer epic (#382 and its children #383–#393, all mentioning `gf-workflow`/`gf-regression`/"阻断") are semantic-routing/evaluation proposals layered on top of the deterministic skills gf-workflow already calls; none of them propose the specific deterministic Phase 3 security/regression gate that #344 delivered, so none are made redundant by it. `docs/index.md`'s Reports Archive section already documents `security-report-*.md` / `regression-report-*.md` as artifacts of the #344 gate — no update needed there.

## Full classification (unchanged from prior triage pass; reviewed, not reapplied)

| # | Title | Type | Priority | Note |
|---|---|---|---|---|
| 396 | fix(gitlab): milestone 命令自建实例 404 | bug | high | ok |
| 395 | fix(github,gitcode): issue close/reopen 丢失 milestone | bug | medium | ok |
| 394 | fix(gitcode): pr close 反序列化报错 | bug | high | ok |
| 393 | feat(decision): Jev 离线评测与阈值校准 | feature | medium | ok |
| 392 | feat(workflow): Agent trace 无进展/循环检测 | feature | low | ok |
| 391 | feat(context): 选择性上下文压缩 | feature | low | ok |
| 390 | feat(query): 混合式自然语言过滤 | feature | low | ok |
| 389 | test(regression): 语义回归判定与对抗测试 | feature | low | ok |
| 388 | feat(workflow): Jev 语义规则检查 | feature | low | ok |
| 387 | feat(issue): 需求质量语义预检 | feature | low | ok |
| 386 | feat(review): PR 多维语义预筛 | feature | low | ok |
| 385 | feat(pipeline): CI 失败语义分类 | feature | low | ok |
| 384 | feat(skills): 任务到 Skill 置信度路由 | feature | low | ok |
| 383 | feat(workflow): 工作流模式推荐 | feature | low | ok |
| 382 | feat(decision): 引入 Jev 决策层（Issue triage 切片） | feature | medium | ok — epic parent |
| 380 | refactor(core): created_at 回落 Utc::now() | **type:refactor (non-standard)** | medium | see Finding 1 |
| 371 | docs(architecture-diagram): 语义架构信息丢失 | docs | low | ok |
| 370 | fix(architecture-diagram): Stage 5 检查恒真 | bug | medium | ok |
| 349 | refactor(skills): 语言画像单源化 | enhancement | low | ok |
| 347 | feat(dist): .claude-plugin 清单第二安装入口 | enhancement | low | ok |
| 346 | feat(skills): diff 交互式审阅网页渲染 | enhancement | low | ok |
| 345 | feat(workflow): 进度看板 HTML 渲染 | enhancement | low | ok |
| 343 | feat(skills): 只读 skill allowed-tools 收敛 | enhancement | low | see Finding 2 |
| 240 | upstream: glab 1.115.0 | enhancement | low | ok — bot-filed |
| 227 | upstream: gh 2.98.0 | enhancement | low | ok — bot-filed |
| 188 | upstream: gitcode 0.11.1 | enhancement | low | ok — bot-filed |

## Archiving

Adding this report brought `docs/issue-triage-report-*.md` to 6 files, exceeding the skill's 5-file retention threshold. Per `docs/index.md`'s Reports Archive policy, the oldest report by date — `issue-triage-report-2026-09-17-wf-001-milestone2.md` — was moved to `docs/reports-archive/2026-Q3/`, leaving 5 reports under `docs/`.
