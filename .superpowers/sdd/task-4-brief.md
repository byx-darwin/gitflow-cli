## Task 4: 重构 `install_hook` 接受 `AgentPlatform`

**文件**: `apps/cli/src/commands/skills.rs`

**改动**:

1. 函数签名变为 `install_hook(global: bool, force: bool, platform: AgentPlatform) -> miette::Result<()>`
2. 调用 `resolve_*_hook_paths` 时传入 `platform`
3. 更新 `install_skills` 中调用 `install_hook` 的地方，传入平台信息

