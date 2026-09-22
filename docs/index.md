# Documentation Index

## Getting Started

- [CLI Patterns](./cli-patterns.md) — argument parsing, error handling, output, and conventions for CLI tools.
- [Architecture](./architecture.md) — workspace layout rationale and dependency flow.
- [`architecture-diagram.dot`](./architecture-diagram.dot) / [`assets/architecture-diagram.svg`](./assets/architecture-diagram.svg) — dependency-derived architecture diagram, regenerated via `/gf-architecture-diagram` from `cargo metadata`; do not hand-edit the `.svg`.
- [`architecture-internals.dot`](./architecture-internals.dot) / [`assets/architecture-internals.svg`](./assets/architecture-internals.svg) — hand-maintained map of provider contracts, domain types, error flow, external CLIs, and the synchronous `git` lookup. Use it for design intent and runtime risks; the generated diagram above tracks crate dependencies.
- [Diff review semantic notes](./diff-review-semantic.md) — optional Claude Code CLI enrichment of pseudocode and visible calls for the offline diff review page.
- [Config](./config.md) — config file format, env vars, XDG directories, and `.env` loading.
- [Optional Jev decisions](./jev-decision.md) — typed decision CLI, limits, privacy, and calibration status (Issue #382).
- [Issue quality precheck](./issue-quality-precheck.md) — optional read-only Jev advice for requirement reviews (Issue #387).
- [PR review precheck](./pr-review-precheck.md) — optional read-only Jev risk triage before a full PR review (Issue #386).
- [Pipeline failure analysis](./pipeline-failure-analysis.md) — bounded read-only Jev classification and root-cause grouping for CI failures (Issue #385).
- [Decision offline evaluation](./decision-offline-evaluation.md) — versioned fixtures, saved responses, calibration, and report comparison (Issue #393).
- [Optional workflow recommendation](./workflow-recommendation.md) — deterministic mode routing with advisory Jev signals (Issue #383).
- [Workflow recommendation synthetic evaluation](./workflow-recommendation-evaluation-2026-09-22.md) — replayable mode, risk, coverage, latency, and cost measurements (Issue #383).
- [Optional gf Skill suggestion](./skill-suggestion.md) — read-only task routing over the bundled Skill catalog (Issue #384).
- [Skill suggestion synthetic evaluation](./skill-suggestion-evaluation-2026-09-22.md) — replayable top-1, top-3, abstention, error, latency, and cost measurements (Issue #384).
- [Jev Issue triage pilot evaluation](./jev-triage-evaluation-2026-09-22.md) — 22 labeled public Issues, six synthetic boundary cases, accuracy, coverage, latency, and cost (Issue #382).
- [Shell Completions](./shell-completions.md) — how to generate and install tab completions.

## Development

- [Superpowers Integration Guide](./integration-guide.md) — how gf skills integrate with Superpowers SDD workflow.
- [Read-only Skill permissions](./skill-read-only-permissions.md) — Claude Code tool limits and validator behavior for analysis-only skills (Issue #343).
- [Gitflow Workflow Guide](./gf-workflow-guide.md) — complete four-phase gated workflow (clarify → plan → execute → deliver) with examples.
- [gf-workflow-batch Skill](../skills/gf-workflow-batch/SKILL.md) — serial outer driver batch-processing multiple open Issues through gf-workflow (Issue #280).
- [gf-walkthrough Skill](../skills/gf-walkthrough/SKILL.md) — produces an offline delivery walkthrough (narrative, diff summary, evidence graded `Measured`/`Inferred`/`Unverified`, review-gate blast radius) anchored on a `<base>...<head>` diff; issues no verdict (Issue #329). Example: [walkthrough-feat-329-gf-walkthrough-2026-09-16.md](./walkthrough-feat-329-gf-walkthrough-2026-09-16.md) — the skill's own dogfooding pass on the change that built it.
- [gf-issue-decompose Skill](../skills/gf-issue-decompose/SKILL.md) — decomposes a requirement document into vertically sliced tickets with falsifiable, red-at-base acceptance criteria and explicit `Blocked by` edges, created in dependency order (Issue #330).
- [TDD Guide](./tdd.md) — test-driven development workflow with `make test-watch`.
- [Pre-commit Usage](./pre-commit-usage.md) — how to install and run repository pre-commit hooks.
- [Release](./release.md) — release checklist, changelog, and distribution packaging.
- [Release Workflow](./release-workflow.md) — `make release` / `make release-quick` interactive flow, version inference, and crates.io publishing.
- [E2E Test Setup Guide](./e2e-test-setup-guide.md) — how to configure credentials and fixtures for end-to-end tests.
- [Phase 4 Dogfooding Checklist](./specs/phase4-dogfooding-checklist.md) — pre-release verification checklist for GitHub/GitLab/GitCode core commands.

## Roadmap

- [多角色项目评估与产品路线图 v2](./superpowers/specs/2026-09-02-product-evaluation-roadmap-v2-design.md) — v1.9.0 五角色重新评估：增长阶段收尾（贡献者/CI 治理）+ 扩张阶段启动（MCP/e2e 补齐）。
- [多角色项目评估与产品路线图](./superpowers/specs/2026-07-31-product-evaluation-roadmap-design.md) — 五角色现状评估与 2026 下半年路线图（稳定化 → 增长 → 扩张，含官方网站与 GEO/SEO 方案）。
- [gf-workflow 双 skills 来源兼容设计](./superpowers/specs/2026-08-08-workflow-dual-skill-sources-design.md) — Issue #141：superpowers + mattpocock/skills 双来源检测、分支适配、GO 闸门与安装时硬阻断。
- [GitLab glab 1.113 兼容修复设计](./superpowers/specs/2026-08-18-gitlab-glab113-compat-design.md) — Issue #199：gf 写操作去 `--output json`、`auth status --show-token`、`mr update --draft`、`label edit --label-id`、`/work_items/N` 解析等。实施计划见 [plans/2026-08-18-gitlab-glab113-compat.md](./superpowers/plans/2026-08-18-gitlab-glab113-compat.md)。
- [GitHub (gh) 兼容性检查报告](./gh-compat-check-2026-08-18.md) — Issue #200 前置调研：gh 2.97 实测 + 源码审查，发现 `gh label view` 缺失致 `gf label edit` 假失败（P1）等。
- [GitHub gh 2.97 label edit 假失败修复设计](./superpowers/specs/2026-08-18-github-gh-label-edit-design.md) — Issue #200：`fetch_label` 改走 `gh api repos/{owner}/{repo}/labels/{name}`、`parse_gh_error` 仅在真实认证失败时提示登录、`label list --limit 100`。实施计划见 [plans/2026-08-18-github-gh-label-edit.md](./superpowers/plans/2026-08-18-github-gh-label-edit.md)。
- [Worktree 共享符号链接防误提交设计](./superpowers/specs/2026-09-04-worktree-symlink-exclude-guard-design.md) — Issue #318：`.cache/workflows`/`.claude` 符号链接写入共享 `info/exclude`（实测：worktree 无独立 exclude，公用主仓库文件）+ Phase 3 交付前新增符号链接提交检测。
- [Worktree 符号链接相对深度动态计算设计](./superpowers/specs/2026-09-04-worktree-symlink-depth-fix-design.md) — Issue #322：硬编码 `../../` 连单段 `worktree_path` 也无法到达仓库根，改为按段数动态计算（`ups = segs + 1`）+ 存在性自检。实施计划见 [plans/2026-09-04-worktree-symlink-depth-fix.md](./superpowers/plans/2026-09-04-worktree-symlink-depth-fix.md)。
- [GitLab glab 1.113.0 兼容性矩阵更新设计](./superpowers/specs/2026-08-18-gitlab-glab113-matrix-design.md) — Issue #198：glab 1.113.0 验证通过（smoke-test 54 passed）后更新 GitLab `tested_versions` + 重新生成矩阵文档 + 修复 `make compatibility-matrix` 包名。实施计划见 [plans/2026-08-18-gitlab-glab113-matrix.md](./superpowers/plans/2026-08-18-gitlab-glab113-matrix.md)。
- [兼容性矩阵 gf 版本派生设计](./superpowers/specs/2026-08-18-compat-matrix-cargo-pkg-version-design.md) — Issue #207：移除 JSON 冗余 `gitflow_cli_version` 字段，矩阵文档版本头部由 `env!("CARGO_PKG_VERSION")` 自动派生，发版无需手动同步版本元数据。
- [安装文档 Node.js 与技能来源前置条件设计](./superpowers/specs/2026-08-19-install-docs-node-prereq-design.md) — Issue #192：README / quickstart / workflow-guide 补齐前置条件（Node.js ≥ 22.20.0、Claude Code、技能来源），并在 `gf skills install` 硬阻断错误中内联 Node 版本提示。
- [自动上报 bug 加固设计](./superpowers/specs/2026-08-30-autoreport-bug-hardening-design.md) — 2026-08-18 多角色评估后续：归档限流、CI 环境硬拦截、`auto-report` 标签缺失早失败、非交互 Preview 默认改为 skip、首次端到端验证。实施计划见 [plans/2026-08-30-autoreport-bug-hardening.md](./superpowers/plans/2026-08-30-autoreport-bug-hardening.md)。
- [覆盖率度量口径统一设计](./superpowers/specs/2026-09-17-coverage-metric-unification-design.md) — Issues #340/#348/#354（milestone #2）：覆盖率工具统一到 `cargo-llvm-cov`、Gate 3 口径由「增量」改为总行覆盖 80%、空变更判 N/A、补写 `references/ruby.md`、去除 auto-fix 冲突、新增 skill 链接校验脚本。实测推翻 tarpaulin 37.55% 基线（llvm-cov 同口径 85.78%）。实施计划见 [plans/2026-09-17-coverage-metric-unification.md](./superpowers/plans/2026-09-17-coverage-metric-unification.md)。
- [EnvSource 注入消除测试级进程环境竞态设计](./superpowers/specs/2026-09-17-envsource-injection-design.md) — Issue #359：`temp_env` 写进程级环境变量与 `cargo test` 多线程并行竞态致恒红。在 `gitflow-cli-adapter-utils` 新增 `EnvSource`/`RealEnv` 抽象，三个 adapter provider 增加带默认值的 env 泛型参数，测试改为注入假 env，并移除 `temp-env`（含 `apps/cli` 的死依赖）。
- [gitcode 列表命令分页修复设计](./superpowers/specs/2026-09-18-gitcode-pagination-fix-design.md) — Issue #365：#360 在 gitcode 上未真正生效（阈值从 30 挪到 100，仍报 `truncated: false`）。首次获得 gitcode CLI 实测后，`issue`/`pr`/`label`/`milestone` 五处 list 改 `Paged` + `--per-page`/`--page`，`release list` 因实测无分页旗标改走 `gitcode api`，移除 `GITCODE_DEFAULT_LIST_LIMIT`，并新增 argv 回归护栏与公开仓库只读 e2e。实施计划见 [plans/2026-09-18-gitcode-pagination-fix.md](./superpowers/plans/2026-09-18-gitcode-pagination-fix.md)。
- [gf-workflow-batch 按依赖边拓扑排序取票设计](./superpowers/specs/2026-09-19-gf-workflow-batch-topo-order-design.md) — Issue #337：Pending Derivation 新增依赖解析阶段，扫描全部 open Issue 的 `Blocked by: #N` 声明，用 `gf issue view` 的 `state == closed` 判定前置是否完成，三色 DFS 检测成环（发现即报错终止、不派发任何 Issue），就绪集合内部保持原有编号升序。纯 prompt/伪代码改动，不涉及 Rust 代码。

## 官网与 GEO

- 官方网站：<https://byx-darwin.github.io/gitflow-cli>（源码见 `website/`）
- [用 gf 开发 gf：dogfooding 案例](https://byx-darwin.github.io/gitflow-cli/dogfooding/)（`website/src/pages/dogfooding.mdx`）— 四阶段编排的真实执行走查、契约测试与兼容性矩阵如何反哺质量。Issue #288。
- GEO 地基：`website/public/llms.txt`、`website/public/llms-full.txt`、`website/public/robots.txt`、`website/src/layouts/Base.astro`（JSON-LD）
- 演示资产：`docs/assets/demo.svg`（生成脚本 `scripts/gen-demo-svg.sh`）
- 设计文档：`docs/superpowers/specs/2026-07-31-v1.0-metadata-website-geo-design.md`

## Skill References

- [Skill Parameter References](./references/) — CLI parameter docs and reusable checklists consumed by skills: `gf-pr-params.md`, `gf-label-milestone-params.md`, `gf-label-stats-taxonomy.md`, `gf-pipeline-analyzer-params.md`, `gf-precommit-params.md`, `gf-precommit-hook-template.md`, `gf-quality-params.md`, `gf-release-helper-params.md`, `pr-review-checklist.md`.

## Templates

- [Report/Plan Templates](./templates/) — `pipeline-report.md`, `workflow-phases-detail.md`, `workflow-plan.md`, reused by `gf-workflow` and `gf-pipeline-analyzer` when generating reports.

## Research

- [Research Notes](./research/) — dogfooding language surveys (`dogfooding-go.md`, `dogfooding-node.md`, `dogfooding-python.md`), `quality-report.md`, and per-skill design/refactor analyses (`skill-analysis-*.md`, `skills-refactor-analysis.md`).

## Operations

- [Operations Guides](./operations/) — `content-matrix-guide.md`, `search-engine-submission-guide.md` for maintaining the docs website's content and search visibility.

## Reports Archive

Point-in-time reports generated by gf skills for audit trail, grouped by naming convention rather than indexed individually:

- `code-review-report-*.md` — PR code review findings from `/code-review` and `gf-pr-review`.
- `security-report-*.md` — dependency/lockfile-triggered audit findings from `gf-security-check`, run as the `gf-workflow` Phase 3 change-surface gate (Issue #344).
- `regression-report-*.md` — CLI-behavior-change-triggered smoke test results from `gf-regression`, run as the `gf-workflow` Phase 3 change-surface gate (Issue #344).
- `pipeline-analysis-report-*.md` — CI health snapshots from `gf-pipeline-analyzer`.
- `dogfooding-report-*.md` — end-to-end dogfooding pass results.
- `issue-triage-report-*.md` — issue classification runs from `gf-issue-triage`.
- `architecture-review-*.md` (+ `architecture-review-diagram.*`) — periodic architecture reviews.
- `test-report-*.md` — full regression test results.
- `smell-report-*.md` — code smell and complexity hotspot scans from `gf-smell`.
- `refactor-report-*.md` — behavior-preserving refactor runs from `gf-refactor`.
- `walkthrough-*.md` — delivery walkthrough narratives + graded verification evidence from `gf-walkthrough`.
- [Ad-hoc PR Reviews](./reviews/) — one-off review notes not matching the naming pattern above.
- `cli-compatibility.md`, `compatibility-matrix.md`, `geo-citation-check.md` — standalone compatibility/eval reports (see also [GitHub compat check](./gh-compat-check-2026-08-18.md) under Roadmap).
- `2026-08-18-autoreport-bug-multi-role-eval-report.md` — multi-role evaluation of the `gf-autoreport-bug` skill.

### Archiving Policy

To keep the `docs/` root from growing unbounded, each report family above is
capped in place: once a family has **more than 5** files directly under
`docs/`, the oldest ones (all but the 5 most recent, ordered by the
issue/PR number embedded in the filename) are moved into
[`docs/reports-archive/<YYYY>-Q<N>/`](./reports-archive/), bucketed by the
quarter of the report's own date. Archived reports stay in the archive
directory permanently — they are never moved back. `docs/index.md` keeps
summarizing by naming convention rather than listing archived files
individually.

`walkthrough-*.md` filenames embed a date rather than an issue/PR number
(`walkthrough-<slug>-<YYYY-MM-DD>.md`), so once this family exceeds **5**
files under `docs/`, the ordering for "oldest first" uses that embedded
date instead of an issue/PR number — otherwise the same rule applies:
the oldest move into `docs/reports-archive/<YYYY>-Q<N>/`, bucketed by the
quarter of the report's own date.

## Reference

- [Command Reference](./commands/) — detailed documentation for all `gf` commands.
  - [pr cleanup](./commands/pr-cleanup.md) — safely clean up branches and worktrees after PR merge.
- [CLAUDE.md](../CLAUDE.md) — agent guide with code style, security, and testing rules.
- [CONTRIBUTING.md](../CONTRIBUTING.md) — how to contribute.
- [SECURITY.md](../SECURITY.md) — security policy and vulnerability reporting.
