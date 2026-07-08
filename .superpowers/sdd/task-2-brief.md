## Task 2: 重构 `resolve_project_hook_paths` 接受 `AgentPlatform`

**文件**: `apps/cli/src/commands/skills.rs`

**改动**:

1. 函数签名变为 `resolve_project_hook_paths(repo: &Path, platform: AgentPlatform) -> (PathBuf, PathBuf, String)`
2. hook 脚本目录：`repo.join(platform.hooks_dir_name())`（Claude 下 = `repo/.claude/hooks/`）
3. settings 路径：`repo.join(platform.settings_file_path())`（Claude 下 = `repo/.claude/settings.json`）
4. command 路径：`bash "$(git rev-parse --show-toplevel 2>/dev/null || pwd)/<hooks_dir_name>/auto-report-bug.sh"`
5. 更新文档注释，删除旧的 "hook 脚本在 `hooks/` 不在 `.claude/hooks/`" 的说明

**TDD**: 更新 `test_resolve_project_hook_paths_uses_hooks_dir` 测试，验证新路径在 `.claude/hooks/` 下

