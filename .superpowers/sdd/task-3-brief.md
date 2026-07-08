## Task 3: 重构 `resolve_global_hook_paths` 接受 `AgentPlatform`

**文件**: `apps/cli/src/commands/skills.rs`

**改动**:

1. 函数签名变为 `resolve_global_hook_paths(home: &Path, platform: AgentPlatform) -> (PathBuf, PathBuf, String)`
2. hook 脚本目录：`home.join(platform.hooks_dir_name())`
3. settings 路径：`home.join(platform.settings_file_path())`
4. command 路径：`bash ~/<hooks_dir_name>/auto-report-bug.sh`

**TDD**: 更新 `test_resolve_global_hook_paths_uses_claude_hooks_dir` 测试

