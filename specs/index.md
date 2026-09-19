# Specs Index

- [README](./README.md) — one-line note on what specs in this directory are for.
- [GitFlow CLI 设计规格](./gitflow-cli-design.md) — 跨平台 Git 工程化工作流编排框架完整设计。
- [TUI System Monitor Design](./tui-system-monitor-design.md) — Full design for the TUI demo app: layout, state, data flow, dependencies, "wow factor".
- [gf-workflow-batch 设计规格](./gf-workflow-batch-design.md) — 串行批量处理多个 open Issue 的外层驱动器设计（Issue #280）。
- [gf-workflow Phase 3 Mode ① 移除设计](./gf-workflow-mode1-removal-design.md) — 移除后台 agent 执行模式，改为二选一菜单（Issue #325）。
- [gf-smell 设计规格](./gf-smell-design.md) — 代码坏味道与复杂度热点检测 skill：语言无关三阶段判定协议 + 语言专属检测层（Issue #327）。

## 实现计划

- [Phase 1: Core + CLI 基础（MVP）](../docs/superpowers/plans/2026-07-01-phase1-core-cli-foundation.md) ✅ 已完成 (Issue #1)
- [Phase 2: GitHub 完整支持](../docs/superpowers/plans/2026-07-01-phase2-github-full.md) ✅ 已完成 (Issue #3)
- [Phase 3: GitLab + GitCode](../docs/superpowers/plans/2026-07-01-phase3-gitlab-gitcode.md) — 多平台支持 + Pipeline 通用接口
- [Phase 4: 编排层](../docs/superpowers/plans/2026-07-01-phase4-orchestration.md) ✅ 已完成 (Issue #5) — 全流程编排 + 质量关卡 + 一键安装
- [Phase 5: 完成度提升](../docs/superpowers/plans/2026-07-01-phase5-polish.md) — 剩余 Skills + Shell 补全 + Homebrew + 社区文档
- [gf-workflow Mode ① 移除实施计划](../docs/superpowers/plans/2026-09-06-gf-workflow-mode1-removal.md) — Issue #325，纯文档改动，3 个文件
- [gf-smell 实施计划](../docs/superpowers/plans/2026-09-16-gf-smell.md) — Issue #327，5 个 Task：验收断言（RED）→ SKILL.md → rust.md → 其余四语言层 → dogfooding 实跑
- [EnvSource 注入实施计划](../docs/superpowers/plans/2026-09-17-envsource-injection.md) — Issue #359，5 个 Task：EnvSource 抽象（adapter-utils）→ gitlab/github/gitcode 三个 provider 注入 env → 移除 temp-env 死依赖 + 全量验收。
- [gf-refactor 实施计划](../docs/superpowers/plans/2026-09-18-gf-refactor.md) — Issue #332，4 个 Task：验收断言（RED）→ SKILL.md（Fowler 52 手法目录 + 语义安全边界）→ rust.md → dogfooding 实跑（paging.rs Extract Function，原定 label.rs 因零测试覆盖按 When NOT to Refactor 改判）。
- [gf-quality/gf-pr-review 证据分级实施计划](../docs/superpowers/plans/2026-09-19-quality-review-evidence-grading.md) — Issue #333，4 个 Task：验收断言（RED）→ gf-quality SKILL.md（逐 Gate 证据等级 + 失败测试溯源表）→ gf-pr-review SKILL.md（逐维度证据等级）→ 校验 GREEN + gf-quality 实跑 dogfooding。

## 集成指南

- [Integration Guide](../docs/integration-guide.md) — 将 gf 集成到现有项目的完整指南。
