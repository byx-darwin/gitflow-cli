## Task 5: 重构 `uninstall_hook` 接受 `AgentPlatform` + 清理脚本文件

**文件**: `apps/cli/src/commands/skills.rs`

**改动**:

1. 函数签名变为 `uninstall_hook(global: bool, platform: AgentPlatform) -> miette::Result<()>`
2. settings 路径按平台解析
3. **新增**：删除 hook 脚本文件（`auto-report-bug.sh`），如果 hook 目录为空则也删除目录
4. 更新 `uninstall_skills` 中调用

