# 删除自动上报 bug 功能与共建计划安装提示

- **Date**: 2026-09-01
- **Classification**: Bounded (existing flow, no new subsystem)
- **Workflow**: wf-2026-09-01-001

## Goal

移除"共建计划"体系（含"主动上传 bug"自动上报能力），以及 `gf skills install`
安装流程中"是否加入共建计划？"的交互提示。

## Scope（整体删除，已与用户确认）

- 连同 `gitflow.co_contribution` settings 字段、安装提示、`gf doctor` 健康检查项一起删除，
  不保留开关框架。
- `hooks/auto-report-bug.sh` 与 `skills/gf-autoreport-bug/` 一并删除。
- 清理现状类文档中的引用；`docs/superpowers/plans`、`specs`、`research`、报告类历史文档不动
  （时间快照）。

## Code Changes

- **`apps/cli/src/error_reporter.rs`**：整个模块删除；`main.rs` 移除
  `mod error_reporter;` 与 `report_error_noninteractive` 调用点。
- **`apps/cli/src/commands/skills.rs`**：删除共建计划相关函数
  （`try_enable_co_contribution`、`merge_co_contribution`、`merge_co_contribution_json`、
  `iso8601_utc_now_co_contribution`、`confirm`、`confirm_with_reader`，确认无其他调用方）；
  删除 auto-report-bug hook 安装/卸载相关函数（`install_hook`、`uninstall_hook`、
  `merge_stop_hook`、`resolve_hook_paths`、`resolve_global_hook_paths`、
  `resolve_project_hook_paths`、`build_auto_report_hook_cmd`、`autoreport_repo_slug*`、
  `is_safe_slug_segment`）及其调用点、`--report-bug` CLI 参数、对应测试。
- **`apps/cli/src/commands/doctor.rs`**：删除 `CoContributionCheck`、
  `co_contribution_check_items_with`、注册点、相关测试。
- 删除 `hooks/auto-report-bug.sh`、`skills/gf-autoreport-bug/`、
  `docs/references/gf-autoreport-bug-params.md`。

## Documentation Changes

清理 README.md、docs/architecture.md、docs/integration-guide.md、docs/index.md，以及
`skills/{gf-issue-create,gf-issue,gf-regression,gf-security-check}/SKILL.md` 中对该功能的引用。

## Testing

删除对应单测；跑 `make build && make test && make fmt && make clippy`（涉及公共 CLI 行为与
多文件删除，需要完整 Rust gate）。
