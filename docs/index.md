# Documentation Index

## Getting Started

- [CLI Patterns](./cli-patterns.md) — argument parsing, error handling, output, and conventions for CLI tools.
- [Architecture](./architecture.md) — workspace layout rationale and dependency flow.
- [Config](./config.md) — config file format, env vars, XDG directories, and `.env` loading.
- [Shell Completions](./shell-completions.md) — how to generate and install tab completions.

## Development

- [Superpowers Integration Guide](./integration-guide.md) — how gf skills integrate with Superpowers SDD workflow.
- [Gitflow Workflow Guide](./gf-workflow-guide.md) — complete four-phase gated workflow (clarify → plan → execute → deliver) with examples.
- [TDD Guide](./tdd.md) — test-driven development workflow with `make test-watch`.
- [Pre-commit Usage](./pre-commit-usage.md) — how to install and run repository pre-commit hooks.
- [Release](./release.md) — release checklist, changelog, and distribution packaging.
- [Phase 4 Dogfooding Checklist](./specs/phase4-dogfooding-checklist.md) — pre-release verification checklist for GitHub/GitLab/GitCode core commands.

## Roadmap

- [多角色项目评估与产品路线图](./superpowers/specs/2026-07-31-product-evaluation-roadmap-design.md) — 五角色现状评估与 2026 下半年路线图（稳定化 → 增长 → 扩张，含官方网站与 GEO/SEO 方案）。
- [gf-workflow 双 skills 来源兼容设计](./superpowers/specs/2026-08-08-workflow-dual-skill-sources-design.md) — Issue #141：superpowers + mattpocock/skills 双来源检测、分支适配、GO 闸门与安装时硬阻断。
- [GitLab glab 1.113 兼容修复设计](./superpowers/specs/2026-08-18-gitlab-glab113-compat-design.md) — Issue #199：gf 写操作去 `--output json`、`auth status --show-token`、`mr update --draft`、`label edit --label-id`、`/work_items/N` 解析等。实施计划见 [plans/2026-08-18-gitlab-glab113-compat.md](./superpowers/plans/2026-08-18-gitlab-glab113-compat.md)。
- [GitHub (gh) 兼容性检查报告](./gh-compat-check-2026-08-18.md) — Issue #200 前置调研：gh 2.97 实测 + 源码审查，发现 `gh label view` 缺失致 `gf label edit` 假失败（P1）等。
- [GitHub gh 2.97 label edit 假失败修复设计](./superpowers/specs/2026-08-18-github-gh-label-edit-design.md) — Issue #200：`fetch_label` 改走 `gh api repos/{owner}/{repo}/labels/{name}`、`parse_gh_error` 仅在真实认证失败时提示登录、`label list --limit 100`。实施计划见 [plans/2026-08-18-github-gh-label-edit.md](./superpowers/plans/2026-08-18-github-gh-label-edit.md)。
- [GitLab glab 1.113.0 兼容性矩阵更新设计](./superpowers/specs/2026-08-18-gitlab-glab113-matrix-design.md) — Issue #198：glab 1.113.0 验证通过（smoke-test 54 passed）后更新 GitLab `tested_versions` + 重新生成矩阵文档 + 修复 `make compatibility-matrix` 包名。实施计划见 [plans/2026-08-18-gitlab-glab113-matrix.md](./superpowers/plans/2026-08-18-gitlab-glab113-matrix.md)。
- [兼容性矩阵 gf 版本派生设计](./superpowers/specs/2026-08-18-compat-matrix-cargo-pkg-version-design.md) — Issue #207：移除 JSON 冗余 `gitflow_cli_version` 字段，矩阵文档版本头部由 `env!("CARGO_PKG_VERSION")` 自动派生，发版无需手动同步版本元数据。
- [安装文档 Node.js 与技能来源前置条件设计](./superpowers/specs/2026-08-19-install-docs-node-prereq-design.md) — Issue #192：README / quickstart / workflow-guide 补齐前置条件（Node.js ≥ 22.20.0、Claude Code、技能来源），并在 `gf skills install` 硬阻断错误中内联 Node 版本提示。

## 官网与 GEO

- 官方网站：<https://byx-darwin.github.io/gitflow-cli>（源码见 `website/`）
- GEO 地基：`website/public/llms.txt`、`website/public/llms-full.txt`、`website/public/robots.txt`、`website/src/layouts/Base.astro`（JSON-LD）
- 演示资产：`docs/assets/demo.svg`（生成脚本 `scripts/gen-demo-svg.sh`）
- 设计文档：`docs/superpowers/specs/2026-07-31-v1.0-metadata-website-geo-design.md`

## Reference

- [Command Reference](./commands/) — detailed documentation for all `gf` commands.
  - [pr cleanup](./commands/pr-cleanup.md) — safely clean up branches and worktrees after PR merge.
- [CLAUDE.md](../CLAUDE.md) — agent guide with code style, security, and testing rules.
- [CONTRIBUTING.md](../CONTRIBUTING.md) — how to contribute.
- [SECURITY.md](../SECURITY.md) — security policy and vulnerability reporting.
