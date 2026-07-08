## Task 1: 扩展 `AgentPlatform` 增加 hooks 和 settings 路径方法

**文件**: `apps/cli/src/commands/skills.rs`

**改动**:

1. 给 `AgentPlatform` 增加 `hooks_dir_name()` 方法：
   - Claude → `.claude/hooks`
   - Codex → `.codex/hooks`
   - OpenCode → `.opencode/hooks`
   - Gemini → `.gemini/hooks`
   - Copilot → `.copilot/hooks`

2. 给 `AgentPlatform` 增加 `settings_file_path()` 方法：
   - Claude → `.claude/settings.json`
   - 其他平台暂时返回 `.<platform>/settings.json`（后续 PR 按实际约定调整）

3. 增加测试覆盖新方法

**TDD**: 先写测试 → 实现 → 重构

