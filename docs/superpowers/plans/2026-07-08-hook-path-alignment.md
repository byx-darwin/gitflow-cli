# Issue #55 实现计划：对齐 hook 路径 + 多 Agent 平台抽象

## 目标

1. 项目级 hook 脚本从 `{repo}/hooks/` 迁移到 `{repo}/.claude/hooks/`（Claude 平台）
2. 代码层面抽象 `AgentPlatform` 的 hook 路径，为后续多平台支持铺路
3. `scripts/install.sh` 的 `HOOK_CONFIG` 与 Rust CLI 对齐（使用绝对路径）
4. `uninstall_hook` 同步更新，清理脚本文件 + settings 配置

## 非目标（后续 PR）

- 其他 Agent 平台（Codex/Gemini/Copilot/OpenCode）的实际 hooks 支持
- 不同平台 settings 文件格式（TOML/JSON）的差异化处理

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

## Task 2: 重构 `resolve_project_hook_paths` 接受 `AgentPlatform`

**文件**: `apps/cli/src/commands/skills.rs`

**改动**:

1. 函数签名变为 `resolve_project_hook_paths(repo: &Path, platform: AgentPlatform) -> (PathBuf, PathBuf, String)`
2. hook 脚本目录：`repo.join(platform.hooks_dir_name())`（Claude 下 = `repo/.claude/hooks/`）
3. settings 路径：`repo.join(platform.settings_file_path())`（Claude 下 = `repo/.claude/settings.json`）
4. command 路径：`bash "$(git rev-parse --show-toplevel 2>/dev/null || pwd)/<hooks_dir_name>/auto-report-bug.sh"`
5. 更新文档注释，删除旧的 "hook 脚本在 `hooks/` 不在 `.claude/hooks/`" 的说明

**TDD**: 更新 `test_resolve_project_hook_paths_uses_hooks_dir` 测试，验证新路径在 `.claude/hooks/` 下

## Task 3: 重构 `resolve_global_hook_paths` 接受 `AgentPlatform`

**文件**: `apps/cli/src/commands/skills.rs`

**改动**:

1. 函数签名变为 `resolve_global_hook_paths(home: &Path, platform: AgentPlatform) -> (PathBuf, PathBuf, String)`
2. hook 脚本目录：`home.join(platform.hooks_dir_name())`
3. settings 路径：`home.join(platform.settings_file_path())`
4. command 路径：`bash ~/<hooks_dir_name>/auto-report-bug.sh`

**TDD**: 更新 `test_resolve_global_hook_paths_uses_claude_hooks_dir` 测试

## Task 4: 重构 `install_hook` 接受 `AgentPlatform`

**文件**: `apps/cli/src/commands/skills.rs`

**改动**:

1. 函数签名变为 `install_hook(global: bool, force: bool, platform: AgentPlatform) -> miette::Result<()>`
2. 调用 `resolve_*_hook_paths` 时传入 `platform`
3. 更新 `install_skills` 中调用 `install_hook` 的地方，传入平台信息

## Task 5: 重构 `uninstall_hook` 接受 `AgentPlatform` + 清理脚本文件

**文件**: `apps/cli/src/commands/skills.rs`

**改动**:

1. 函数签名变为 `uninstall_hook(global: bool, platform: AgentPlatform) -> miette::Result<()>`
2. settings 路径按平台解析
3. **新增**：删除 hook 脚本文件（`auto-report-bug.sh`），如果 hook 目录为空则也删除目录
4. 更新 `uninstall_skills` 中调用

## Task 6: 更新 `InstallArgs` 使 `--agent` 对项目级也生效

**文件**: `apps/cli/src/commands/skills.rs`

**改动**:

1. 当前 `--agent` 参数 `requires = "global"`，即只在 `-g` 时有效
2. 改为项目级也支持 `--agent`（去掉 `requires = "global"` 约束）
3. 项目级默认值：`AgentPlatform::detect()`（与全局级一致）
4. 更新 `install_skills` 中 platform 的解析逻辑，使其在项目级和全局级都工作

## Task 7: 更新 `scripts/install.sh`

**文件**: `scripts/install.sh`

**改动**:

1. `HOOK_CONFIG` 常量中的 command 路径改为绝对路径：
   - 从: `bash hooks/auto-report-bug.sh`
   - 改为: `bash "$(git rev-parse --show-toplevel 2>/dev/null || pwd)/.claude/hooks/auto-report-bug.sh"`
2. hook 脚本目标路径改为 `.claude/hooks/auto-report-bug.sh`（在仓库内）
3. 验证步骤中的 hook 检测也更新为新路径

## Task 8: 更新文档

**文件**:
- `specs/gitflow-cli-design.md`
- `docs/integration-guide.md`

**改动**:
- 将所有 `hooks/auto-report-bug.sh` 路径引用更新为 `.claude/hooks/auto-report-bug.sh`
- 更新 settings.json 中的 command 路径示例

## Task 9: 质量关卡

**操作**:

```bash
cargo build
cargo test
cargo +nightly fmt
cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic
cargo audit
cargo deny check
```

## Task 10: 提交 + PR

1. 创建分支 `fix/issue-55`
2. 逐 Task 提交（或按逻辑分组）
3. 创建 PR，描述中包含 `Closes #55`
4. PR 描述包含所有改动摘要

## Task 11: 收尾

1. 验证 Issue #55 已自动关闭
2. 切回 main 分支
3. 清理 worktree 和分支

## 风险与注意事项

1. **向后兼容**: 已安装 hook 的用户，升级后旧路径 `{repo}/hooks/auto-report-bug.sh` 不会被自动清理。可在 `install_hook` 中加入迁移逻辑（如果旧路径存在，删除旧文件 + 旧 settings 中的旧 command）
2. **空目录清理**: `uninstall_hook` 删除脚本后，如果目录为空应一并删除
3. **`--agent` 约束放松**: 需验证项目级传 `--agent` 不会破坏现有行为
4. **install.sh 兼容性**: install.sh 只处理 Claude 平台（硬编码 `.claude/`），这与 Rust CLI 默认 Claude 平台一致
